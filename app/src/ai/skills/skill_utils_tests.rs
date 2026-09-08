use std::path::PathBuf;

use ai::skills::{ParsedSkill, SkillProvider, SkillReference, SkillScope};
use repo_metadata::repositories::DetectedRepositories;
use repo_metadata::{DirectoryWatcher, RepoMetadataModel};
use warp_core::features::FeatureFlag;
use warp_util::host_id::HostId;
use warp_util::local_or_remote_path::LocalOrRemotePath;
use warp_util::remote_path::RemotePath;
use warp_util::standardized_path::StandardizedPath;
use warpui::{App, ModelHandle};
use watcher::HomeDirectoryWatcher;

use super::*;
use crate::ai::skills::BundledSkillActivation;
use crate::settings::AISettings;
use crate::warp_managed_paths_watcher::WarpManagedPathsWatcher;

#[cfg(feature = "local_fs")]
fn initialize_skill_catalog(app: &mut App) -> ModelHandle<SkillManager> {
    // 测试目录由各用例显式提供,避免异步探测覆盖其中的 bundled 目录。
    let _bundled = FeatureFlag::BundledSkills.override_enabled(false);
    app.add_singleton_model(DirectoryWatcher::new);
    app.add_singleton_model(AISettings::new_with_defaults);
    app.add_singleton_model(|_| DetectedRepositories::default());
    app.add_singleton_model(RepoMetadataModel::new);
    app.add_singleton_model(HomeDirectoryWatcher::new_for_test);
    app.add_singleton_model(WarpManagedPathsWatcher::new_for_testing);
    app.add_singleton_model(SkillManager::new)
}

fn catalog_skill(name: &str, path: LocalOrRemotePath, scope: SkillScope) -> ParsedSkill {
    ParsedSkill {
        name: name.to_owned(),
        description: name.to_owned(),
        path,
        content: name.to_owned(),
        line_range: None,
        provider: SkillProvider::Agents,
        scope,
    }
}

#[test]
#[cfg(feature = "local_fs")]
fn list_skills_isolates_ssh_catalog_from_local_and_other_hosts() {
    App::test((), |mut app| async move {
        let manager = initialize_skill_catalog(&mut app);
        let _bundled = FeatureFlag::BundledSkills.override_enabled(true);
        let host_id = HostId::new("remote-host".to_owned());
        let other_host_id = HostId::new("other-host".to_owned());
        let other_home = LocalOrRemotePath::Remote(RemotePath::new(
            other_host_id.clone(),
            StandardizedPath::try_new("/home/user").unwrap(),
        ));
        let local_home = LocalOrRemotePath::Local(dirs::home_dir().unwrap());
        let local_cwd = LocalOrRemotePath::Local(std::env::temp_dir().join("ssh-skill-repo"));

        manager.update(&mut app, |manager, _| {
            manager.handle_skills_added(vec![
                catalog_skill(
                    "localmind",
                    local_home.join(".agents/skills/localmind/SKILL.md"),
                    SkillScope::Home,
                ),
                catalog_skill(
                    "local-project",
                    local_cwd.join(".agents/skills/local/SKILL.md"),
                    SkillScope::Project,
                ),
                catalog_skill(
                    "remote-project",
                    remote_location("/repo/.agents/skills/project/SKILL.md"),
                    SkillScope::Project,
                ),
                catalog_skill(
                    "other-project",
                    LocalOrRemotePath::Remote(RemotePath::new(
                        other_host_id.clone(),
                        StandardizedPath::try_new("/repo/.agents/skills/project/SKILL.md").unwrap(),
                    )),
                    SkillScope::Project,
                ),
            ]);
            manager.set_remote_home_skills(
                host_id.clone(),
                remote_location("/home/user"),
                vec![catalog_skill(
                    "remote-home",
                    remote_location("/home/user/.agents/skills/home/SKILL.md"),
                    SkillScope::Home,
                )],
            );
            manager.set_remote_home_skills(
                other_host_id.clone(),
                other_home.clone(),
                vec![catalog_skill(
                    "other-home",
                    other_home.join(".agents/skills/home/SKILL.md"),
                    SkillScope::Home,
                )],
            );
            manager.add_bundled_skill_for_testing(
                "bundled",
                catalog_skill(
                    "local-bundled",
                    LocalOrRemotePath::Local(std::env::temp_dir().join("bundled/SKILL.md")),
                    SkillScope::Bundled,
                ),
                BundledSkillActivation::Always,
            );
            manager.add_remote_bundled_skill_for_testing(
                host_id.clone(),
                "bundled",
                catalog_skill(
                    "remote-bundled",
                    remote_location("/bundled/SKILL.md"),
                    SkillScope::Bundled,
                ),
                BundledSkillActivation::Always,
            );
            manager.add_remote_bundled_skill_for_testing(
                other_host_id,
                "bundled",
                catalog_skill(
                    "other-bundled",
                    other_home.join("bundled/SKILL.md"),
                    SkillScope::Bundled,
                ),
                BundledSkillActivation::Always,
            );
        });

        manager.read(&app, |_, ctx| {
            let local_skills = list_skills(Some(&local_cwd), &SkillPathOrigin::Local, ctx);
            assert!(local_skills.iter().any(|skill| skill.name == "localmind"));

            let skills = list_skills(
                Some(&remote_location("/repo")),
                &SkillPathOrigin::Remote { host_id },
                ctx,
            );
            let mut names: Vec<_> = skills.iter().map(|skill| skill.name.as_str()).collect();
            names.sort();
            assert_eq!(names, ["remote-bundled", "remote-home", "remote-project"]);
            assert!(skills.iter().all(|skill| matches!(
                &skill.reference,
                SkillReference::Path(LocalOrRemotePath::Remote(path))
                    if path.host_id == HostId::new("remote-host".to_owned())
            )));
        });
    });
}

#[test]
#[cfg(feature = "local_fs")]
fn list_skills_keeps_remote_home_and_bundled_when_cwd_is_unknown() {
    App::test((), |mut app| async move {
        let manager = initialize_skill_catalog(&mut app);
        let _bundled = FeatureFlag::BundledSkills.override_enabled(true);
        let host_id = HostId::new("remote-host".to_owned());
        manager.update(&mut app, |manager, _| {
            manager.handle_skills_added(vec![
                catalog_skill(
                    "localmind",
                    LocalOrRemotePath::Local(dirs::home_dir().unwrap())
                        .join(".agents/skills/localmind/SKILL.md"),
                    SkillScope::Home,
                ),
                catalog_skill(
                    "remote-project",
                    remote_location("/repo/.agents/skills/project/SKILL.md"),
                    SkillScope::Project,
                ),
            ]);
            manager.set_remote_home_skills(
                host_id.clone(),
                remote_location("/home/user"),
                vec![catalog_skill(
                    "remote-home",
                    remote_location("/home/user/.agents/skills/home/SKILL.md"),
                    SkillScope::Home,
                )],
            );
            manager.add_remote_bundled_skill_for_testing(
                host_id.clone(),
                "bundled",
                catalog_skill(
                    "remote-bundled",
                    remote_location("/bundled/SKILL.md"),
                    SkillScope::Bundled,
                ),
                BundledSkillActivation::Always,
            );
        });

        let skills = manager.read(&app, |_, ctx| {
            list_skills(None, &SkillPathOrigin::Remote { host_id }, ctx)
        });
        let mut names: Vec<_> = skills.iter().map(|skill| skill.name.as_str()).collect();
        names.sort();
        assert_eq!(names, ["remote-bundled", "remote-home"]);
    });
}

#[test]
#[cfg(feature = "local_fs")]
fn list_skills_discards_cwd_from_a_different_execution_host() {
    App::test((), |mut app| async move {
        let manager = initialize_skill_catalog(&mut app);
        let _bundled = FeatureFlag::BundledSkills.override_enabled(false);
        let local_cwd = LocalOrRemotePath::Local(std::env::temp_dir().join("ssh-skill-repo"));
        let other_cwd = LocalOrRemotePath::Remote(RemotePath::new(
            HostId::new("other-host".to_owned()),
            StandardizedPath::try_new("/repo").unwrap(),
        ));
        manager.update(&mut app, |manager, _| {
            manager.handle_skills_added(vec![
                catalog_skill(
                    "localmind",
                    local_cwd.join(".agents/skills/localmind/SKILL.md"),
                    SkillScope::Project,
                ),
                catalog_skill(
                    "other-project",
                    other_cwd.join(".agents/skills/project/SKILL.md"),
                    SkillScope::Project,
                ),
            ]);
        });

        manager.read(&app, |_, ctx| {
            let origin = SkillPathOrigin::Remote {
                host_id: HostId::new("remote-host".to_owned()),
            };
            assert_eq!(list_skills(Some(&local_cwd), &origin, ctx), Vec::new());
            assert_eq!(list_skills(Some(&other_cwd), &origin, ctx), Vec::new());
        });
    });
}

#[test]
#[cfg(feature = "local_fs")]
fn list_skills_omits_every_catalog_when_execution_host_is_unavailable() {
    App::test((), |mut app| async move {
        let manager = initialize_skill_catalog(&mut app);
        let _bundled = FeatureFlag::BundledSkills.override_enabled(true);
        let local_cwd = LocalOrRemotePath::Local(std::env::temp_dir().join("ssh-skill-repo"));
        manager.update(&mut app, |manager, _| {
            manager.handle_skills_added(vec![
                catalog_skill(
                    "localmind",
                    LocalOrRemotePath::Local(dirs::home_dir().unwrap())
                        .join(".agents/skills/localmind/SKILL.md"),
                    SkillScope::Home,
                ),
                catalog_skill(
                    "local-project",
                    local_cwd.join(".agents/skills/project/SKILL.md"),
                    SkillScope::Project,
                ),
            ]);
            manager.add_bundled_skill_for_testing(
                "bundled",
                catalog_skill(
                    "local-bundled",
                    LocalOrRemotePath::Local(std::env::temp_dir().join("bundled/SKILL.md")),
                    SkillScope::Bundled,
                ),
                BundledSkillActivation::Always,
            );
        });

        manager.read(&app, |_, ctx| {
            assert_eq!(
                list_skills(None, &SkillPathOrigin::Unavailable, ctx),
                Vec::new()
            );
            assert_eq!(
                list_skills(Some(&local_cwd), &SkillPathOrigin::Unavailable, ctx),
                Vec::new()
            );
            assert_eq!(
                list_skills(Some(&local_cwd), &SkillPathOrigin::RestoredDisplayOnly, ctx),
                Vec::new()
            );
        });
    });
}

fn remote_location(path: &str) -> LocalOrRemotePath {
    LocalOrRemotePath::Remote(RemotePath::new(
        HostId::new("remote-host".to_string()),
        StandardizedPath::try_new(path).unwrap(),
    ))
}

#[test]
fn skill_path_from_unix_encoded_remote_location() {
    let location = remote_location("/repo/.agents/skills/deploy/scripts/run.sh");

    assert_eq!(
        skill_path_from_location(&location),
        Some(remote_location("/repo/.agents/skills/deploy/SKILL.md"))
    );
}

#[test]
fn skill_path_from_windows_encoded_remote_location() {
    let location = remote_location(r"C:\repo\.agents\skills\deploy\scripts\run.ps1");

    assert_eq!(
        skill_path_from_location(&location),
        Some(remote_location(r"C:\repo\.agents\skills\deploy\SKILL.md"))
    );
}

#[test]
fn test_skill_path_from_file_path_warp_home_skill() {
    let Some(warp_home_skills_dir) = warp_core::paths::warp_home_skills_dir() else {
        eprintln!("Skipping test: InfiniShell home skills directory not available");
        return;
    };
    let warp_home_skill = warp_home_skills_dir
        .join("my-skill")
        .join("assets")
        .join("image.png");
    // 上游把 `skill_path_from_file_path(&Path)` 换成了 `skill_path_from_location(&LocalOrRemotePath)`,
    // 覆盖的行为(从技能内任意文件回溯出 SKILL.md)没变,这里只迁移调用形状。
    let result = skill_path_from_location(&LocalOrRemotePath::Local(warp_home_skill));
    assert_eq!(
        result,
        Some(LocalOrRemotePath::Local(
            warp_home_skills_dir.join("my-skill").join("SKILL.md")
        ))
    );
}

#[test]
fn test_unique_skills_dedupes_identical_skills_same_dir() {
    let shared_skill_dir = PathBuf::from("/home/user");
    let skill_path1 = shared_skill_dir.join(".agents/skills/my-skill/SKILL.md");
    let skill_path2 = shared_skill_dir.join(".claude/skills/my-skill/SKILL.md");

    let content = "---\nname: test-skill\ndescription: A test skill\n---\nContent here";
    let skill = ParsedSkill {
        path: LocalOrRemotePath::Local(skill_path1.clone()),
        name: "test-skill".to_string(),
        description: "A test skill".to_string(),
        content: content.to_string(),
        line_range: Some(8..18),
        provider: SkillProvider::Agents,
        scope: SkillScope::Project,
    };

    let skill2 = ParsedSkill {
        path: LocalOrRemotePath::Local(skill_path2.clone()),
        name: "test-skill".to_string(),
        description: "A test skill".to_string(),
        content: content.to_string(),
        line_range: Some(8..18),
        provider: SkillProvider::Claude,
        scope: SkillScope::Project,
    };

    let mut skills_by_path = HashMap::new();
    skills_by_path.insert(LocalOrRemotePath::Local(skill_path1.clone()), skill);
    skills_by_path.insert(LocalOrRemotePath::Local(skill_path2.clone()), skill2);

    let skill_paths = vec![
        (
            LocalOrRemotePath::Local(shared_skill_dir.clone()),
            LocalOrRemotePath::Local(skill_path1),
        ),
        (
            LocalOrRemotePath::Local(shared_skill_dir),
            LocalOrRemotePath::Local(skill_path2),
        ),
    ];

    let result = unique_skills(&skill_paths, &skills_by_path);
    assert_eq!(result.len(), 1);
    // Agents has higher priority (index 0) than Claude, so it should be preferred
    assert_eq!(result[0].provider, SkillProvider::Agents);
}

#[test]
fn test_unique_skills_keeps_same_provider_skills_from_different_dirs() {
    let home_dir = PathBuf::from("/home/user");
    let project_dir = PathBuf::from("/home/user/projects/repo");
    let home_path = home_dir.join(".agents/skills/my-skill/SKILL.md");
    let project_path = project_dir.join(".agents/skills/my-skill/SKILL.md");

    let content = "---\nname: test-skill\ndescription: A test skill\n---\nContent here";
    let home_skill = ParsedSkill {
        path: LocalOrRemotePath::Local(home_path.clone()),
        name: "test-skill".to_string(),
        description: "A test skill".to_string(),
        content: content.to_string(),
        line_range: Some(8..18),
        provider: SkillProvider::Agents,
        scope: SkillScope::Project,
    };

    let project_skill = ParsedSkill {
        path: LocalOrRemotePath::Local(project_path.clone()),
        name: "test-skill".to_string(),
        description: "A test skill".to_string(),
        content: content.to_string(),
        line_range: Some(8..18),
        provider: SkillProvider::Agents,
        scope: SkillScope::Project,
    };

    let mut skills_by_path = HashMap::new();
    skills_by_path.insert(LocalOrRemotePath::Local(home_path.clone()), home_skill);
    skills_by_path.insert(
        LocalOrRemotePath::Local(project_path.clone()),
        project_skill,
    );

    let skill_paths = vec![
        (
            LocalOrRemotePath::Local(home_dir),
            LocalOrRemotePath::Local(home_path.clone()),
        ),
        (
            LocalOrRemotePath::Local(project_dir),
            LocalOrRemotePath::Local(project_path.clone()),
        ),
    ];

    let result = unique_skills(&skill_paths, &skills_by_path);
    assert_eq!(result.len(), 2, "同名 + 同 provider 跨目录应各自保留");
    assert!(
        result.iter().any(|skill| skill.reference
            == SkillReference::Path(LocalOrRemotePath::Local(home_path.clone()))),
        "应保留 home 目录里的同名 skill,实际={result:?}"
    );
    assert!(
        result.iter().any(|skill| skill.reference
            == SkillReference::Path(LocalOrRemotePath::Local(project_path.clone()))),
        "应保留 project 目录里的同名 skill,实际={result:?}"
    );
}

#[test]
fn test_unique_skills_name_dedup_same_name_different_providers() {
    let shared_skill_dir = PathBuf::from("/home/user");
    let skill_path1 = shared_skill_dir.join(".agents/skills/my-skill/SKILL.md");
    let skill_path2 = shared_skill_dir.join(".claude/skills/my-skill/SKILL.md");

    let content1 = "---\nname: test-skill\ndescription: A test skill\n---\nContent here";
    let content2 = "---\nname: test-skill\ndescription: A test skill\n---\nDifferent content";

    let skill1 = ParsedSkill {
        path: LocalOrRemotePath::Local(skill_path1.clone()),
        name: "test-skill".to_string(),
        description: "A test skill".to_string(),
        content: content1.to_string(),
        line_range: Some(8..18),
        provider: SkillProvider::Agents,
        scope: SkillScope::Project,
    };

    let skill2 = ParsedSkill {
        path: LocalOrRemotePath::Local(skill_path2.clone()),
        name: "test-skill".to_string(),
        description: "A test skill".to_string(),
        content: content2.to_string(),
        line_range: Some(8..18),
        provider: SkillProvider::Claude,
        scope: SkillScope::Project,
    };

    let mut skills_by_path = HashMap::new();
    skills_by_path.insert(LocalOrRemotePath::Local(skill_path1.clone()), skill1);
    skills_by_path.insert(LocalOrRemotePath::Local(skill_path2.clone()), skill2);

    let skill_paths = vec![
        (
            LocalOrRemotePath::Local(shared_skill_dir.clone()),
            LocalOrRemotePath::Local(skill_path1),
        ),
        (
            LocalOrRemotePath::Local(shared_skill_dir),
            LocalOrRemotePath::Local(skill_path2),
        ),
    ];

    let result = unique_skills(&skill_paths, &skills_by_path);
    assert_eq!(
        result.len(),
        1,
        "同名不同内容不同 provider 应 name-dedup,仅保留最高优先级 provider"
    );
    assert_eq!(
        result[0].provider,
        SkillProvider::Agents,
        "name-dedup 应保留高优先级 provider(Agents > Claude)"
    );
}
