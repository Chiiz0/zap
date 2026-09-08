use ai::agent::action_result::{AnyFileContent, FileContext};
use ai::skills::SkillReference;
#[cfg(feature = "local_fs")]
use ai::skills::parse_skill;
use futures::future::{BoxFuture, FutureExt};
use warpui::{Entity, ModelContext, SingletonEntity};

use super::{ActionExecution, AnyActionExecution, ExecuteActionInput, PreprocessActionInput};
#[cfg(feature = "local_fs")]
use crate::ai::agent::AIAgentActionResultType;
use crate::ai::agent::{AIAgentActionType, ReadSkillRequest, ReadSkillResult};
#[cfg(feature = "local_fs")]
use crate::ai::skills::extract_skill_parent_directory;
use crate::ai::skills::{SkillManager, SkillTelemetryEvent};
use crate::send_telemetry_from_ctx;

pub struct ReadSkillExecutor;

impl ReadSkillExecutor {
    pub fn new() -> Self {
        Self
    }

    pub(super) fn should_autoexecute(
        &self,
        _input: ExecuteActionInput,
        _ctx: &mut ModelContext<Self>,
    ) -> bool {
        // User-created skills are readable on demand.
        true
    }

    pub(super) fn execute(
        &mut self,
        input: ExecuteActionInput,
        ctx: &mut ModelContext<Self>,
    ) -> impl Into<AnyActionExecution> + use<> {
        let ExecuteActionInput { action, .. } = input;
        let AIAgentActionType::ReadSkill(ReadSkillRequest { skill: skill_ref }) = &action.action
        else {
            return ActionExecution::InvalidAction;
        };

        let manager = SkillManager::as_ref(ctx);

        // Cache hit:proto 的 `SkillReference::Path(p)` 在这一步只在 p 恰好就是
        // 索引中真实 SKILL.md 绝对路径时命中。
        if let Some(skill) = manager.skill_by_reference(skill_ref) {
            send_telemetry_from_ctx!(
                SkillTelemetryEvent::Read {
                    reference: skill_ref.clone(),
                    name: Some(skill.name.clone()),
                    scope: Some(skill.scope),
                    provider: Some(skill.provider),
                    error: false,
                },
                ctx
            );
            return success_execution(skill);
        }

        // BYOP 已在生成 action 前按会话清单解析名称；执行器只接受真实引用。
        // 不从全局名称索引兜底，避免命中本机或其他远端的同名技能。

        // Cache miss 兜底:对于 `SkillReference::Path` 形式的引用,
        // 如果路径形状是合法的 skill 文件
        // (`.../<provider>/skills/<name>/SKILL.md` 或 warp managed skill 目录下),
        // 直接读盘解析,修复 issue #99 中描述的「skill 已存在但 cache 未热」场景。
        //
        // 设计取舍:
        // - 不主动 warm SkillManager cache。Cache 由 SkillWatcher 单向维护,
        //   在这里写入会破坏数据流。重复 read_skill 同一路径会重复读盘,
        //   但 SKILL.md 通常很小,可忽略。
        // - `extract_skill_parent_directory` 只校验路径形状,与 cache hit 时
        //   返回的 path 安全等级一致 —— 都不限定家目录前缀。这是有意的:
        //   project 内 skill (`/some/repo/.agents/skills/...`) 也需要能读。
        // - Windows 下正则用反斜杠分隔,Linux 风格 `/home/<u>/...` 路径会被
        //   拒绝;这意味着本兜底对 "Windows 主进程 + WSL session" 不生效,
        //   是 issue #99 的已知限制(见 PR 描述)。
        // Cache miss fallback 仅在拥有本地文件系统的构建中可用;
        // WASM 等无 fs 构建里 `extract_skill_parent_directory` / `parse_skill`
        // 不存在,自然也无从读盘。
        // `parse_skill` 只能读本地文件系统,因此远程 `LocalOrRemotePath` 不走这条兜底。
        #[cfg(feature = "local_fs")]
        if let SkillReference::Path(path) = skill_ref
            && let Some(local_path) = path.to_local_path()
        {
            if extract_skill_parent_directory(path).is_ok() {
                let path = local_path.to_path_buf();
                let skill_ref_for_async = skill_ref.clone();
                return ActionExecution::new_async(
                    async move { parse_skill(&path) },
                    move |parsed, _app| match parsed {
                        Ok(skill) => AIAgentActionResultType::ReadSkill(ReadSkillResult::Success {
                            content: FileContext::new(
                                skill.path.display_path(),
                                AnyFileContent::StringContent(skill.content.clone()),
                                skill.line_range.clone(),
                                None,
                            ),
                        }),
                        Err(err) => AIAgentActionResultType::ReadSkill(ReadSkillResult::Error(
                            format!("Skill not found: {skill_ref_for_async:?} ({err})"),
                        )),
                    },
                );
            }
        }

        send_telemetry_from_ctx!(
            SkillTelemetryEvent::Read {
                reference: skill_ref.clone(),
                name: None,
                scope: None,
                provider: None,
                error: true,
            },
            ctx
        );
        ActionExecution::Sync(
            ReadSkillResult::Error(format!("Skill not found: {:?}", skill_ref)).into(),
        )
    }

    pub(super) fn preprocess_action(
        &mut self,
        _input: PreprocessActionInput,
        _ctx: &mut ModelContext<Self>,
    ) -> BoxFuture<'static, ()> {
        futures::future::ready(()).boxed()
    }
}

/// Build a sync success execution from a parsed skill.
///
/// 抽出 helper 是为了让 `ActionExecution<T>` 的泛型 `T` 在 `success_execution`
/// 和 `new_async` 两条路径里推导到相同类型(否则 Rust 会要求函数显式声明返回类型)。
fn success_execution(
    skill: &ai::skills::ParsedSkill,
) -> ActionExecution<anyhow::Result<ai::skills::ParsedSkill>> {
    let content = FileContext::new(
        // `ParsedSkill::path` 现在是 `LocalOrRemotePath`,用 `display_path()` 取
        // 可显示的路径字符串(本地即原路径,远程为 `host:path` 形式)。
        skill.path.display_path(),
        AnyFileContent::StringContent(skill.content.clone()),
        skill.line_range.clone(),
        None,
    );
    ActionExecution::Sync(ReadSkillResult::Success { content }.into())
}

impl Entity for ReadSkillExecutor {
    type Event = ();
}

#[cfg(test)]
#[path = "read_skill_tests.rs"]
mod tests;
