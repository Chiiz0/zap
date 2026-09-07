use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;

use futures::stream::AbortHandle;
use repo_metadata::repositories::RepoDetectionSource;
use repo_metadata::{RepositoryUpdate, TargetFile};
use settings::SettingsMode;
use warpui::{App, Entity, ModelHandle};

use super::{
    FileMCPConfigDiagnostic, FileMCPConfigDiagnosticKind, FileMCPConfigParseOutcome,
    FileMCPWatcher, FileMCPWatcherEvent, InFlightParse, config_change_flags, home_subdir_to_watch,
    parse_mcp_config_file, providers_in_scope, should_watch_repository, substitute_env_vars,
};
use crate::ai::mcp::MCPProvider;

fn cleanup_env_vars(vars: &[&str]) {
    for var in vars {
        // TODO: Audit that the environment access only happens in single-threaded code.
        unsafe { env::remove_var(var) };
    }
}

#[test]
fn abort_config_parse_cancels_and_removes_inflight_task() {
    let config_path = PathBuf::from("/tmp/.mcp.json");
    let key = (config_path.clone(), MCPProvider::InfiniShell);
    let (abort_handle, _abort_registration) = AbortHandle::new_pair();
    let observed_handle = abort_handle.clone();
    let mut watcher = FileMCPWatcher {
        in_flight_parses: HashMap::from([(
            key.clone(),
            InFlightParse {
                generation: 0,
                abort_handle,
            },
        )]),
        ..FileMCPWatcher::new_inert()
    };

    watcher.abort_config_parse(&config_path, MCPProvider::InfiniShell);

    assert!(observed_handle.is_aborted());
    assert!(!watcher.in_flight_parses.contains_key(&key));
}

#[derive(Default)]
struct ScanEvents(Vec<&'static str>);

impl Entity for ScanEvents {
    type Event = ();
}

fn record_scan_events(
    app: &mut App,
    watcher: &ModelHandle<FileMCPWatcher>,
) -> ModelHandle<ScanEvents> {
    let events = app.add_model(|_| ScanEvents::default());
    events.update(app, |_, ctx| {
        ctx.subscribe_to_model(watcher, |events, _, event, _| {
            events.0.push(match event {
                FileMCPWatcherEvent::ConfigParsed { .. } => "parsed",
                FileMCPWatcherEvent::ConfigRemoved { .. } => "removed",
                FileMCPWatcherEvent::ConfigError { .. } => "error",
                FileMCPWatcherEvent::InitialGlobalScanComplete => "complete",
                FileMCPWatcherEvent::CloudEnvMcpScanComplete { .. } => "cloud",
            });
        });
    });
    events
}

#[test]
fn initial_global_scan_ignores_superseded_parse_callbacks() {
    App::test((), |mut app| async move {
        let key = (
            PathBuf::from("/home/test/.claude.json"),
            MCPProvider::Claude,
        );
        let watcher = app.add_model(|_| FileMCPWatcher {
            initial_global_pending: Some(HashSet::from([key.clone()])),
            in_flight_parses: HashMap::from([(
                key.clone(),
                InFlightParse {
                    generation: 2,
                    abort_handle: AbortHandle::new_pair().0,
                },
            )]),
            ..FileMCPWatcher::new_inert()
        });
        let events = record_scan_events(&mut app, &watcher);

        watcher.update(&mut app, |watcher, ctx| {
            watcher.complete_config_parse(
                key.clone(),
                1,
                PathBuf::from("/home/test"),
                FileMCPConfigParseOutcome::Missing,
                ctx,
            );
        });
        events.read(&app, |events, _| assert!(events.0.is_empty()));
        watcher.read(&app, |watcher, _| {
            assert_eq!(watcher.in_flight_parses.get(&key).unwrap().generation, 2);
            assert_eq!(watcher.initial_global_pending.as_ref().unwrap().len(), 1);
        });

        watcher.update(&mut app, |watcher, ctx| {
            watcher.complete_config_parse(
                key,
                2,
                PathBuf::from("/home/test"),
                FileMCPConfigParseOutcome::Parsed(Vec::new()),
                ctx,
            );
        });
        events.read(&app, |events, _| {
            assert_eq!(events.0, ["parsed", "complete"])
        });
    });
}

#[test]
fn initial_global_scan_waits_for_last_source_and_settles_parse_errors() {
    App::test((), |mut app| async move {
        let key = (
            PathBuf::from("/home/test/.claude.json"),
            MCPProvider::Claude,
        );
        let other = (
            PathBuf::from("/home/test/.codex/config.toml"),
            MCPProvider::Codex,
        );
        let watcher = app.add_model(|_| FileMCPWatcher {
            initial_global_pending: Some(HashSet::from([key.clone(), other.clone()])),
            in_flight_parses: HashMap::from([(
                key.clone(),
                InFlightParse {
                    generation: 1,
                    abort_handle: AbortHandle::new_pair().0,
                },
            )]),
            ..FileMCPWatcher::new_inert()
        });
        let events = record_scan_events(&mut app, &watcher);
        watcher.update(&mut app, |watcher, ctx| {
            let diagnostic = FileMCPConfigDiagnostic {
                config_path: key.0.clone(),
                provider: key.1,
                kind: FileMCPConfigDiagnosticKind::Parse,
                message: "invalid config".to_owned(),
            };
            watcher.complete_config_parse(
                key,
                1,
                PathBuf::from("/home/test"),
                FileMCPConfigParseOutcome::Error(diagnostic),
                ctx,
            );
        });
        events.read(&app, |events, _| assert_eq!(events.0, ["error"]));
        watcher.update(&mut app, |watcher, ctx| {
            watcher.remove_config(other.0.clone(), PathBuf::from("/home/test"), other.1, ctx);
        });
        events.read(&app, |events, _| {
            assert_eq!(events.0, ["error", "removed", "complete"])
        });

        watcher.update(&mut app, |watcher, ctx| {
            watcher.remove_config(other.0, PathBuf::from("/home/test"), other.1, ctx);
        });
        events.read(&app, |events, _| {
            assert_eq!(events.0, ["error", "removed", "complete", "removed"])
        });
    });
}

#[test]
fn initial_global_scan_removal_prevents_late_parse_from_restoring_servers() {
    App::test((), |mut app| async move {
        let key = (
            PathBuf::from("/home/test/.claude.json"),
            MCPProvider::Claude,
        );
        let watcher = app.add_model(|_| FileMCPWatcher {
            initial_global_pending: Some(HashSet::from([key.clone()])),
            in_flight_parses: HashMap::from([(
                key.clone(),
                InFlightParse {
                    generation: 1,
                    abort_handle: AbortHandle::new_pair().0,
                },
            )]),
            ..FileMCPWatcher::new_inert()
        });
        let events = record_scan_events(&mut app, &watcher);
        watcher.update(&mut app, |watcher, ctx| {
            watcher.remove_config(key.0.clone(), PathBuf::from("/home/test"), key.1, ctx);
            watcher.complete_config_parse(
                key,
                1,
                PathBuf::from("/home/test"),
                FileMCPConfigParseOutcome::Parsed(Vec::new()),
                ctx,
            );
        });
        events.read(&app, |events, _| {
            assert_eq!(events.0, ["removed", "complete"])
        });
        watcher.read(&app, |watcher, _| {
            assert!(watcher.in_flight_parses.is_empty())
        });
    });
}

#[test]
fn repository_discovery_is_surface_aware() {
    assert!(should_watch_repository(
        RepoDetectionSource::TerminalNavigation,
        SettingsMode::Gui
    ));
    assert!(!should_watch_repository(
        RepoDetectionSource::ProjectRulesIndexing,
        SettingsMode::Gui
    ));
    assert!(!should_watch_repository(
        RepoDetectionSource::CodeReviewInitialization,
        SettingsMode::Gui
    ));

    assert!(should_watch_repository(
        RepoDetectionSource::TerminalNavigation,
        SettingsMode::Tui
    ));
    assert!(!should_watch_repository(
        RepoDetectionSource::ProjectRulesIndexing,
        SettingsMode::Tui
    ));
    assert!(!should_watch_repository(
        RepoDetectionSource::CodeReviewInitialization,
        SettingsMode::Tui
    ));
}

#[test]
fn global_provider_initial_scans_cover_claude_codex_and_agents() {
    let home = PathBuf::from("/home/test");

    assert_eq!(home_subdir_to_watch(MCPProvider::Claude), None);
    assert_eq!(
        home.join(MCPProvider::Claude.home_config_path()),
        home.join(".claude.json")
    );

    for (provider, subdir, config) in [
        (MCPProvider::Codex, ".codex", ".codex/config.toml"),
        (MCPProvider::Agents, ".agents", ".agents/.mcp.json"),
    ] {
        assert_eq!(home_subdir_to_watch(provider), Some(PathBuf::from(subdir)));
        let discovered =
            providers_in_scope(home.clone(), home.join(subdir)).collect::<HashSet<_>>();
        assert!(
            discovered.contains(&(provider, home.join(config))),
            "{provider:?} config should be included in its home subdirectory scan"
        );
    }
}

#[test]
fn project_initial_scan_covers_each_supported_provider_config() {
    let repo = PathBuf::from("/work/repository");
    let discovered = providers_in_scope(repo.clone(), repo.clone()).collect::<HashSet<_>>();

    for provider in [
        MCPProvider::InfiniShell,
        MCPProvider::Claude,
        MCPProvider::Codex,
        MCPProvider::Agents,
    ] {
        assert!(
            discovered.contains(&(provider, repo.join(provider.project_config_path()))),
            "{provider:?} project config should be included in a repository scan"
        );
    }
}

#[test]
fn incremental_updates_detect_each_supported_provider_config() {
    let repo = PathBuf::from("/work/repository");
    for provider in [
        MCPProvider::InfiniShell,
        MCPProvider::Claude,
        MCPProvider::Codex,
        MCPProvider::Agents,
    ] {
        let config_path = repo.join(provider.project_config_path());
        let mut added = RepositoryUpdate::default();
        added
            .added
            .insert(TargetFile::new(config_path.clone(), false));
        assert_eq!(config_change_flags(&added, &config_path), (false, true));

        let mut deleted = RepositoryUpdate::default();
        deleted
            .deleted
            .insert(TargetFile::new(config_path.clone(), false));
        assert_eq!(config_change_flags(&deleted, &config_path), (true, false));
    }
}
#[test]
fn test_substitute_env_vars_success() {
    let test_vars = ["FOO", "BAZ", "REPEATED"];

    // Setup environment variables
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { env::set_var("FOO", "bar") };
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { env::set_var("BAZ", "qux") };
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { env::set_var("REPEATED", "value") };

    // Test 1: Single variable substitution
    let input = r#"{"key": "${FOO}"}"#;
    let result = substitute_env_vars(input).expect("Single variable substitution should succeed");
    assert_eq!(
        result, r#"{"key": "bar"}"#,
        "Single variable FOO should be replaced with 'bar'"
    );

    // Test 2: Multiple different variables
    let input = r#"{"key": "${FOO}", "other": "${BAZ}"}"#;
    let result = substitute_env_vars(input).expect("Multiple variable substitution should succeed");
    assert_eq!(
        result, r#"{"key": "bar", "other": "qux"}"#,
        "Multiple variables FOO and BAZ should be replaced"
    );

    // Test 3: Multiple occurrences of same variable
    let input = r#"{"a": "${REPEATED}", "b": "${REPEATED}", "c": "prefix_${REPEATED}_suffix"}"#;
    let result = substitute_env_vars(input).expect("Repeated variable substitution should succeed");
    assert_eq!(
        result, r#"{"a": "value", "b": "value", "c": "prefix_value_suffix"}"#,
        "All occurrences of REPEATED should be replaced with 'value', including within context"
    );

    // Cleanup
    cleanup_env_vars(&test_vars);
}

#[test]
fn test_substitute_env_vars_missing_or_empty() {
    // Test 1: Missing variable
    // Ensure MISSING_VAR is not set
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { env::remove_var("MISSING_VAR") };

    let input = r#"{"key": "${MISSING_VAR}"}"#;
    let result = substitute_env_vars(input);
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Missing or empty environment variable: MISSING_VAR"),
        "Error message should mention MISSING_VAR, got: {err_msg}"
    );

    // Test 2: Empty variable
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { env::set_var("EMPTY_VAR", "") };

    let input = r#"{"key": "${EMPTY_VAR}"}"#;
    let result = substitute_env_vars(input);
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Missing or empty environment variable: EMPTY_VAR"),
        "Error message should mention EMPTY_VAR, got: {err_msg}"
    );

    // Cleanup
    cleanup_env_vars(&["EMPTY_VAR"]);
}

#[tokio::test]
async fn parse_outcomes_distinguish_missing_invalid_and_valid_configs() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = directory.path().join(".mcp.json");

    assert!(matches!(
        parse_mcp_config_file(&path, MCPProvider::InfiniShell).await,
        FileMCPConfigParseOutcome::Missing
    ));

    std::fs::write(&path, "{invalid").expect("invalid config should be written");
    match parse_mcp_config_file(&path, MCPProvider::InfiniShell).await {
        FileMCPConfigParseOutcome::Error(diagnostic) => {
            assert_eq!(diagnostic.kind, FileMCPConfigDiagnosticKind::Parse);
        }
        _ => panic!("invalid JSON should produce a parse diagnostic"),
    }

    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::remove_var("WARP_MCP_TEST_MISSING") };
    std::fs::write(
        &path,
        r#"{"mcpServers":{"test":{"command":"${WARP_MCP_TEST_MISSING}"}}}"#,
    )
    .expect("missing-env config should be written");
    match parse_mcp_config_file(&path, MCPProvider::InfiniShell).await {
        FileMCPConfigParseOutcome::Error(diagnostic) => {
            assert_eq!(
                diagnostic.kind,
                FileMCPConfigDiagnosticKind::MissingEnvironmentVariable
            );
        }
        _ => panic!("missing env should produce a diagnostic"),
    }

    std::fs::write(
        &path,
        r#"{"mcpServers":{"test":{"command":"test-command"}}}"#,
    )
    .expect("valid config should be written");
    match parse_mcp_config_file(&path, MCPProvider::InfiniShell).await {
        FileMCPConfigParseOutcome::Parsed(servers) => assert_eq!(servers.len(), 1),
        _ => panic!("valid config should produce one server"),
    }
}
