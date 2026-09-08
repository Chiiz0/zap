//! `read_skill`:读取 Zap 的 Skill markdown 模板。
//!
//! Skill 是用户/项目预定义的可复用工作流(`SKILL.md` 文件 + 可选元数据)。
//! 模型读 skill 后能按用户期望的步骤推进任务。warp 自家维护一个 `SkillManager`
//! 索引所有可用 skill,既可以用 name(frontmatter `name` 字段)也可以用绝对路径或
//! bundled id 引用。
//!
//! ## 入参契约
//!
//! BYOP 路径暴露 `name` 字段,值取自 system prompt `<available_skills><skill><name>`。
//! `from_args` 保留模型传入的名称,由流式适配层在本轮技能清单中解析真实引用。
//! 解析时校验会话主机,避免本地和不同远端的同名技能相互串用。
//! 同名技能可通过清单中的完整路径区分,兼容本地 bundled 的 `@warp-skill:<id>`。
//!
//! ## 使用建议(写到 description)
//!
//! 模型可在以下场景主动调:
//! - 用户提到 skill 名 / 文件名 / 路径
//! - 任务匹配某 skill 描述(如"做 PR review" 触发 `review` skill)

use ai::skills::{SkillPathOrigin, SkillReference};
use anyhow::Result;
use api::message::tool_call::read_skill::SkillReference as ApiSkillReference;
use serde::Deserialize;
use serde_json::{Value, json};
use warp_multi_agent_api as api;
use warp_util::local_or_remote_path::LocalOrRemotePath;

use super::OpenAiTool;
use crate::ai::skills::SkillDescriptor;

#[derive(Debug, thiserror::Error)]
pub enum SkillResolutionError {
    #[error("{}", crate::t!("ai-error-skill-not-available"))]
    NotAvailable,
    #[error("{}", crate::t!("ai-error-skill-name-ambiguous"))]
    Ambiguous,
}

#[derive(Debug, Deserialize)]
struct Args {
    name: String,
}

fn parameters() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "Skill 名称(与 system prompt 内 <available_skills><skill><name> 字段完全一致),同名时传清单中的完整 skill_path。"
            }
        },
        "required": ["name"],
        "additionalProperties": false
    })
}

fn from_args(args: &str) -> Result<api::message::tool_call::Tool> {
    let parsed: Args = serde_json::from_str(args)?;
    Ok(api::message::tool_call::Tool::ReadSkill(
        api::message::tool_call::ReadSkill {
            skill_reference: None,
            name: parsed.name,
        },
    ))
}

/// 只解析本轮已列出的技能,并在生成客户端 action 前还原完整路径和主机来源。
pub fn resolve_reference(
    read_skill: &mut api::message::tool_call::ReadSkill,
    skills: &[SkillDescriptor],
    path_origin: &SkillPathOrigin,
) -> Result<(), SkillResolutionError> {
    let mut candidates = skills.iter().filter(|skill| {
        let same_host = match path_origin {
            SkillPathOrigin::Local => matches!(
                &skill.reference,
                SkillReference::Path(LocalOrRemotePath::Local(_))
                    | SkillReference::BundledSkillId(_)
            ),
            SkillPathOrigin::Remote { host_id } => matches!(
                &skill.reference,
                SkillReference::Path(LocalOrRemotePath::Remote(path)) if &path.host_id == host_id
            ),
            // 历史显示身份和缺失主机身份都不能用于执行。
            SkillPathOrigin::RestoredDisplayOnly | SkillPathOrigin::Unavailable => false,
        };
        same_host
            && (skill.name == read_skill.name
                || match &skill.reference {
                    SkillReference::Path(path) => path.display_path() == read_skill.name,
                    SkillReference::BundledSkillId(id) => {
                        format!("@warp-skill:{id}") == read_skill.name
                    }
                })
    });
    let selected = candidates
        .next()
        .ok_or(SkillResolutionError::NotAvailable)?;
    if candidates.any(|candidate| candidate.reference != selected.reference) {
        return Err(SkillResolutionError::Ambiguous);
    }
    read_skill.skill_reference = Some(match &selected.reference {
        SkillReference::Path(path) => ApiSkillReference::SkillPath(path.display_path()),
        SkillReference::BundledSkillId(id) => ApiSkillReference::BundledSkillId(id.clone()),
    });
    Ok(())
}

fn result_to_json(result: &api::message::tool_call_result::Result) -> Option<Value> {
    use api::message::tool_call_result::Result as R;
    use api::read_skill_result::Result as SR;
    let r = match result {
        R::ReadSkill(r) => r,
        _ => return None,
    };
    let value = match &r.result {
        Some(SR::Success(s)) => {
            // FileContent { file_path, content, line_range } 直接是单个 message
            // 不是 oneof,无须解包 inner content。
            let (path, content) = s
                .content
                .as_ref()
                .map(|c| (c.file_path.clone(), c.content.clone()))
                .unwrap_or_default();
            json!({ "status": "ok", "path": path, "content": content })
        }
        Some(SR::Error(e)) => json!({ "status": "error", "message": e.message }),
        None => json!({ "status": "cancelled" }),
    };
    Some(value)
}

pub static READ_SKILL: OpenAiTool = OpenAiTool {
    name: "read_skill",
    description: include_str!("../prompts/tool_descriptions/read_skill.md"),
    parameters,
    from_args,
    result_to_json,
};
