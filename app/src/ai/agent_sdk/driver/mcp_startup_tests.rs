use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use futures::FutureExt as _;
use uuid::Uuid;
use warp_core::features::FeatureFlag;
use warpui::{App, Entity, ModelHandle, SingletonEntity};

use super::{
    ModelEventWaitError, await_initial_global_startup, await_model_event,
    wait_for_file_based_mcps_running, wait_for_initial_global_scan,
};
use crate::ai::mcp::file_based_manager::FileBasedMCPManagerEvent;
use crate::ai::mcp::{
    FileBasedMCPManager, FileMCPWatcher, FileMCPWatcherEvent, MCPProvider, MCPServerState,
    ParsedTemplatableMCPServerResult, TemplatableMCPServerManager,
};
use crate::auth::AuthStateProvider;
use crate::settings::{AISettings, FocusedTerminalInfo};
use crate::warp_managed_paths_watcher::warp_managed_mcp_config_path;
use crate::workspaces::user_workspaces::UserWorkspaces;

struct Waiter;

impl Entity for Waiter {
    type Event = usize;
}

#[test]
fn timed_out_wait_removes_its_subscription_before_another_wait() {
    App::test((), |mut app| async move {
        let waiter = app.add_model(|_| Waiter);
        let emitter = app.add_model(|_| Waiter);
        let calls = Rc::new(Cell::new(0));
        let old_calls = calls.clone();
        let old_wait = waiter.update(&mut app, |_, ctx| {
            await_model_event(&emitter, Duration::ZERO, ctx, move |event, _| {
                old_calls.set(old_calls.get() + 1);
                Some(*event)
            })
        });
        assert_eq!(old_wait.await, Err(ModelEventWaitError::TimedOut));

        let new_wait = waiter.update(&mut app, |_, ctx| {
            await_model_event(&emitter, Duration::from_secs(1), ctx, |event, _| {
                (*event == 2).then_some(*event)
            })
        });
        emitter.update(&mut app, |_, ctx| ctx.emit(1));
        emitter.update(&mut app, |_, ctx| ctx.emit(2));
        assert_eq!(new_wait.await, Ok(2));
        assert_eq!(calls.get(), 0);
    });
}

#[test]
fn initial_global_scan_wait_does_not_finish_before_completion_event() {
    App::test((), |mut app| async move {
        let waiter = app.add_model(|_| Waiter);
        let manager = app.add_singleton_model(|_| FileBasedMCPManager::default());
        let uuid = Uuid::new_v4();
        let wait = waiter.update(&mut app, |_, ctx| {
            wait_for_initial_global_scan(Duration::from_secs(1), ctx)
        });
        futures::pin_mut!(wait);
        manager.update(&mut app, |_, ctx| {
            ctx.emit(FileBasedMCPManagerEvent::ServersChanged)
        });
        assert!(futures::poll!(&mut wait).is_pending());

        manager.update(&mut app, |_, ctx| {
            ctx.emit(FileBasedMCPManagerEvent::InitialGlobalMcpScanComplete {
                wait_server_uuids: vec![uuid],
            });
        });
        assert_eq!(wait.await, vec![uuid]);
    });
}

#[test]
fn initial_global_scan_timeout_is_nonfatal() {
    let _flag_guard = FeatureFlag::FileBasedMcp.override_enabled(true);
    App::test((), |mut app| async move {
        let waiter = app.add_model(|_| Waiter);
        app.add_singleton_model(|_| FileBasedMCPManager::default());
        let foreground = waiter.update(&mut app, |_, ctx| ctx.spawner());
        assert_eq!(
            await_initial_global_startup(&foreground, Duration::ZERO).await,
            Ok(())
        );
    });
}

#[test]
fn disabled_file_based_mcp_skips_initial_global_wait() {
    let _flag_guard = FeatureFlag::FileBasedMcp.override_enabled(false);
    App::test((), |mut app| async move {
        let waiter = app.add_model(|_| Waiter);
        let foreground = waiter.update(&mut app, |_, ctx| ctx.spawner());
        assert_eq!(
            await_initial_global_startup(&foreground, Duration::from_secs(20)).now_or_never(),
            Some(Ok(()))
        );
    });
}

fn setup_file_server(
    app: &mut App,
) -> (
    ModelHandle<Waiter>,
    ModelHandle<TemplatableMCPServerManager>,
    Uuid,
) {
    let waiter = app.add_model(|_| Waiter);
    let watcher = app.add_singleton_model(|_| FileMCPWatcher::new_inert());
    app.add_singleton_model(AISettings::new_with_defaults);
    app.add_singleton_model(|_| AuthStateProvider::new_for_test());
    app.add_singleton_model(UserWorkspaces::default_mock);
    app.add_singleton_model(FocusedTerminalInfo::new);
    app.add_singleton_model(FileBasedMCPManager::new);
    let manager = app.add_singleton_model(|_| TemplatableMCPServerManager::default());
    let config = warp_managed_mcp_config_path().expect("测试需要全局 MCP 路径");
    let servers = ParsedTemplatableMCPServerResult::from_user_json(
        r#"{"global": {"command": "test-server"}}"#,
    )
    .unwrap();
    let uuid = servers[0]
        .templatable_mcp_server_installation
        .as_ref()
        .unwrap()
        .uuid();
    watcher.update(app, |_, ctx| {
        ctx.emit(FileMCPWatcherEvent::ConfigParsed {
            config_path: config.config_path,
            root_path: config.root_path,
            provider: MCPProvider::InfiniShell,
            servers,
        });
    });
    (waiter, manager, uuid)
}

#[test]
fn initial_global_scan_completion_waits_for_queued_autostart() {
    let _flag_guard = FeatureFlag::FileBasedMcp.override_enabled(true);
    App::test((), |mut app| async move {
        let (waiter, manager, uuid) = setup_file_server(&mut app);
        FileMCPWatcher::handle(&app).update(&mut app, |_, ctx| {
            ctx.emit(FileMCPWatcherEvent::InitialGlobalScanComplete);
        });
        let scan = waiter.update(&mut app, |_, ctx| {
            wait_for_initial_global_scan(Duration::ZERO, ctx)
        });
        let uuids = scan.await;
        assert_eq!(uuids, vec![uuid]);
        assert!(
            manager
                .read(&app, |manager, _| manager.get_server_state(uuid))
                .is_none()
        );
        let wait = waiter.update(&mut app, |_, ctx| {
            wait_for_file_based_mcps_running(uuids, Duration::from_secs(1), ctx)
        });
        futures::pin_mut!(wait);
        assert!(futures::poll!(&mut wait).is_pending());
        manager.update(&mut app, |manager, ctx| {
            manager.change_server_state(uuid, MCPServerState::Starting, ctx);
        });
        assert!(futures::poll!(&mut wait).is_pending());
        manager.update(&mut app, |manager, ctx| {
            manager.change_server_state(uuid, MCPServerState::Running, ctx);
        });
        wait.await;
    });
}

#[test]
fn initial_global_server_wait_settles_when_startup_fails() {
    let _flag_guard = FeatureFlag::FileBasedMcp.override_enabled(true);
    App::test((), |mut app| async move {
        let (waiter, manager, uuid) = setup_file_server(&mut app);
        let wait = waiter.update(&mut app, |_, ctx| {
            wait_for_file_based_mcps_running(vec![uuid], Duration::from_secs(1), ctx)
        });
        futures::pin_mut!(wait);
        assert!(futures::poll!(&mut wait).is_pending());
        manager.update(&mut app, |manager, ctx| {
            manager.change_server_state(uuid, MCPServerState::FailedToStart, ctx);
        });
        wait.await;
    });
}

#[test]
fn initial_global_server_wait_accepts_stopped_and_removed_servers() {
    let _flag_guard = FeatureFlag::FileBasedMcp.override_enabled(true);
    App::test((), |mut app| async move {
        let (waiter, manager, uuid) = setup_file_server(&mut app);
        manager.update(&mut app, |manager, ctx| {
            manager.change_server_state(uuid, MCPServerState::NotRunning, ctx);
        });
        let wait = waiter.update(&mut app, |_, ctx| {
            wait_for_file_based_mcps_running(
                vec![uuid, Uuid::new_v4()],
                Duration::from_secs(1),
                ctx,
            )
        });
        assert_eq!(wait.now_or_never(), Some(()));
    });
}

#[test]
fn initial_global_server_wait_timeout_allows_later_wait_to_complete() {
    let _flag_guard = FeatureFlag::FileBasedMcp.override_enabled(true);
    App::test((), |mut app| async move {
        let (waiter, manager, uuid) = setup_file_server(&mut app);
        let wait = waiter.update(&mut app, |_, ctx| {
            wait_for_file_based_mcps_running(vec![uuid], Duration::ZERO, ctx)
        });
        wait.await;
        let next_wait = waiter.update(&mut app, |_, ctx| {
            wait_for_file_based_mcps_running(vec![uuid], Duration::from_secs(1), ctx)
        });
        manager.update(&mut app, |manager, ctx| {
            manager.change_server_state(uuid, MCPServerState::Running, ctx);
        });
        next_wait.await;
    });
}
