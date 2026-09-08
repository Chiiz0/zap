//! Utility functions for working with skills.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};

use ai::skills::{
    ParsedSkill, SKILL_PROVIDER_DEFINITIONS, SkillPathOrigin, SkillProvider, home_skills_path,
    provider_parent_directory_for_skills_root, provider_rank,
};
use warp_core::ui::Icon;
use warp_core::ui::appearance::Appearance;
use warp_core::ui::theme::color::internal_colors;
use warp_util::local_or_remote_path::LocalOrRemotePath;
use warpui::prelude::MouseStateHandle;
use warpui::{AppContext, Element, EventContext, SingletonEntity};

use super::{SkillDescriptor, SkillManager};
use crate::ai::blocklist::view_util::render_provider_icon_button;
use crate::warp_managed_paths_watcher::warp_managed_skill_dirs;

/// Tries to insert or update a skill descriptor in the deduplication map.
///
/// 去重键是 **(skill 名, 归属目录)**,而不是上游的 (归属目录, 内容哈希)。
///
/// 上游按内容哈希去重只覆盖了 `npx skills` 把同一份文件软链到多个 provider 目录的场景;
/// 同名但内容已经漂移(例如 `~/.agents/skills/deploy` 与 `~/.claude/skills/deploy` 各自
/// 被改过)时会同时保留两份,进入 system prompt 后 agent 会看到两个同名 skill 而无法
/// 判别该用哪个。按 (名字, 目录) 去重可以同时覆盖两种情况:
///
/// 1. **provider rank 小者胜**:依 [`SKILL_PROVIDER_DEFINITIONS`] 顺序(index 0 = 最高优先级),
///    例如 `Agents > Zap > Claude > …`。
/// 2. **同 rank 时 reference 路径短者胜**:取为稳定 tiebreak。
///
/// 目录仍是键的一部分,所以同名 skill 分别存在于多个目录(例如 repo root + subdir)时
/// 各自保留,由调用方按路径上下文处理。
#[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
fn try_insert_skill(
    dedup_map: &mut HashMap<(String, LocalOrRemotePath), SkillDescriptor>,
    descriptor: SkillDescriptor,
    dir_path: &LocalOrRemotePath,
) {
    match dedup_map.entry((descriptor.name.clone(), dir_path.clone())) {
        Entry::Vacant(e) => {
            e.insert(descriptor);
        }
        Entry::Occupied(mut e) => {
            // Prefer the skill from the higher-priority provider, falling back to
            // the shorter reference so the winner is deterministic.
            let new_rank = provider_rank(descriptor.provider);
            let existing_rank = provider_rank(e.get().provider);
            if new_rank < existing_rank
                || (new_rank == existing_rank
                    && descriptor.reference.to_string().len() < e.get().reference.to_string().len())
            {
                e.insert(descriptor);
            }
        }
    }
}

/// Accumulates file-backed skills from one or more catalogs and keeps the best
/// representative for each owning-directory-and-name pair.
#[derive(Default)]
#[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
pub(crate) struct SkillDeduplicator {
    dedup_map: HashMap<(String, LocalOrRemotePath), SkillDescriptor>,
}

#[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
impl SkillDeduplicator {
    pub(crate) fn insert(&mut self, dir_path: &LocalOrRemotePath, skill: &ParsedSkill) {
        try_insert_skill(
            &mut self.dedup_map,
            SkillDescriptor::from(skill.clone()),
            dir_path,
        );
    }

    pub(crate) fn extend_paths(
        &mut self,
        skill_paths: &[(LocalOrRemotePath, LocalOrRemotePath)],
        skills_by_path: &HashMap<LocalOrRemotePath, ParsedSkill>,
    ) {
        for (dir_path, path) in skill_paths {
            if let Some(skill) = skills_by_path.get(path) {
                self.insert(dir_path, skill);
            }
        }
    }

    /// **P0-3 prompt cache 补漏**:返回 Vec 按 `(name, reference)` 字典序排序。
    /// 原因:`HashMap::into_values()` 迭代顺序不稳定,该返回值会进入 system prompt 的
    /// skills section,顺序漂移就会让全部上游供应商(Anthropic / OpenAI / DeepSeek)的
    /// prompt cache 全序失效。与 P0-3 MCP tools 排序同性质。
    /// reference 作为稳定排序的次级键,保证同名 skill 的输出顺序也可复现。
    pub(crate) fn into_descriptors(self) -> Vec<SkillDescriptor> {
        let mut descriptors: Vec<SkillDescriptor> = self.dedup_map.into_values().collect();
        descriptors.sort_by(|a, b| {
            a.name
                .cmp(&b.name)
                .then_with(|| a.reference.to_string().cmp(&b.reference.to_string()))
        });
        descriptors
    }
}

/// Deduplicates paths from one indexed catalog by **name and owning directory**, keeping the
/// single best representative per [`SKILL_PROVIDER_DEFINITIONS`] (index 0 = highest priority).
///
/// 覆盖的场景:
/// - `npx skills` 软链同一份 skill 到 `~/.agents/skills/` / `~/.warp/skills/` / `~/.claude/skills/`
///   (同目录同名不同 provider) → 保留高优先级 provider。
/// - 同名但内容已漂移(同目录不同 provider) → 同样保留高优先级 provider。
/// - 同名 skill 同时存在于多个目录(例如 repo root + subdir) → 各自保留,让调用方按路径上下文处理。
///
/// Each element of `skill_paths` is a `(dir_path, skill_file_path)` tuple where
/// `dir_path` is the directory that owns the skill.
#[cfg(test)]
pub(crate) fn unique_skills(
    skill_paths: &[(LocalOrRemotePath, LocalOrRemotePath)],
    skills_by_path: &HashMap<LocalOrRemotePath, ParsedSkill>,
) -> Vec<SkillDescriptor> {
    let mut deduplicator = SkillDeduplicator::default();
    deduplicator.extend_paths(skill_paths, skills_by_path);
    deduplicator.into_descriptors()
}

/// 列出当前 working directory 适用的全部 skills。
///
/// **设计说明**:上游的 `list_skills_if_changed` 在云端协议下做差量发送(对比上轮已发的
/// `conversation.latest_skills()`,未变化时返回 `None`)以节省上行 token —— warp 后端
/// 维护会话状态,首轮收到后保留即可。项目去云端后,BYOP 走 OpenAI/Anthropic 等无状态
/// `/chat/completions`,system prompt 每轮在客户端完整重渲染,数据必须每轮都送达,
/// 否则第二轮起 system prompt 里 skills section 会消失。
/// 因此简化为每轮全量返回。
pub fn list_skills(
    working_directory: Option<&LocalOrRemotePath>,
    path_origin: &SkillPathOrigin,
    ctx: &AppContext,
) -> Vec<SkillDescriptor> {
    // SSH 启动或切换主机时 cwd 可能未知或仍属于上一会话,来源必须以执行主机为准。
    let working_directory = match path_origin {
        SkillPathOrigin::Local => {
            working_directory.filter(|path| matches!(path, LocalOrRemotePath::Local(_)))
        }
        SkillPathOrigin::Remote { host_id } => working_directory.filter(
            |path| matches!(path, LocalOrRemotePath::Remote(path) if &path.host_id == host_id),
        ),
        SkillPathOrigin::Unavailable | SkillPathOrigin::RestoredDisplayOnly => return Vec::new(),
    };
    SkillManager::as_ref(ctx).get_skills_for_working_directory_with_origin(
        working_directory,
        path_origin,
        ctx,
    )
}

/// Renders an 'open skill' button for blocklist AI actions and the code diff view.
pub fn render_skill_button<F>(
    button_label: &str,
    button_handle: MouseStateHandle,
    appearance: &Appearance,
    skill_provider: SkillProvider,
    icon_override: Option<Icon>,
    on_click: F,
) -> Box<dyn Element>
where
    F: FnMut(&mut EventContext) + 'static,
{
    let theme = appearance.theme();
    let logo_fill = internal_colors::fg_overlay_6(theme);

    let icon = icon_override.unwrap_or_else(|| skill_provider.icon());

    let color = if icon_override.is_some() {
        logo_fill
    } else {
        skill_provider.icon_fill(logo_fill)
    };

    render_provider_icon_button(
        button_label,
        button_handle,
        appearance,
        icon,
        color,
        on_click,
    )
}

/// Returns a branded icon override for well-known skill names.
pub fn icon_override_for_skill_name(name: &str) -> Option<Icon> {
    match name {
        "stripe-projects-cli" => Some(Icon::StripeLogo),
        _ => None,
    }
}

/// Home(用户级)skills 根目录不总能靠路径结构识别出来。
///
/// 上游的 [`provider_parent_directory_for_skills_root`] 用 `<provider>/skills` 的字面组件去匹配
/// (`.agents/skills`、`.claude/skills` …),而 Zap 的用户级 skills 目录是
/// `warp_core::paths::warp_home_skills_dir()`(`~/.infinishell/skills`),
/// 以及 `warp_managed_skill_dirs()` 里 Zap 自管的目录 —— 这些目录名不在
/// [`SKILL_PROVIDER_DEFINITIONS`] 的字面表里,纯结构匹配一律落空。
///
/// 因此这里显式按各 provider 的 home skills 根目录做前缀剥离,
/// 拿到根目录下的第一层组件(即 skill 名)后拼出 `SKILL.md`。
fn home_skill_path_from_local_path(file_path: &Path) -> Option<PathBuf> {
    for definition in SKILL_PROVIDER_DEFINITIONS.iter() {
        let home_skill_dirs = if definition.provider == SkillProvider::InfiniShell {
            warp_managed_skill_dirs()
        } else {
            home_skills_path(definition.provider).into_iter().collect()
        };
        for home_skills_dir in home_skill_dirs {
            if let Ok(relative_path) = file_path.strip_prefix(&home_skills_dir) {
                let skill_name = relative_path.components().next()?;
                return Some(home_skills_dir.join(skill_name).join("SKILL.md"));
            }
        }
    }
    None
}

pub fn skill_path_from_location(location: &LocalOrRemotePath) -> Option<LocalOrRemotePath> {
    // Home skills 目录必须先于结构匹配处理,见 `home_skill_path_from_local_path`。
    if let LocalOrRemotePath::Local(path) = location
        && let Some(skill_path) = home_skill_path_from_local_path(path)
    {
        return Some(LocalOrRemotePath::Local(skill_path));
    }

    let mut current = Some(location.clone());
    while let Some(candidate_skill_dir) = current {
        if candidate_skill_dir
            .parent()
            .and_then(|provider_dir| provider_parent_directory_for_skills_root(&provider_dir))
            .is_some()
        {
            return Some(candidate_skill_dir.join("SKILL.md"));
        }
        current = candidate_skill_dir.parent();
    }
    None
}

#[cfg(test)]
#[path = "skill_utils_tests.rs"]
mod tests;
