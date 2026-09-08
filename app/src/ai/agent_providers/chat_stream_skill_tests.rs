use std::path::PathBuf;

use ai::skills::{SkillPathOrigin, SkillProvider, SkillReference, SkillScope};
use genai::chat::ToolCall;
use serde_json::json;
use warp_multi_agent_api as api;
use warp_util::host_id::HostId;
use warp_util::local_or_remote_path::LocalOrRemotePath;
use warp_util::remote_path::RemotePath;
use warp_util::standardized_path::StandardizedPath;

use super::{make_tool_call_message, parse_incoming_tool_call, serialize_outgoing_tool_call};
use crate::ai::agent::api::{
    ConversionParams, ConvertAPIMessageToClientOutputMessage, MaybeAIAgentOutputMessage,
};
use crate::ai::agent::task::TaskId;
use crate::ai::agent::{AIAgentActionType, AIAgentOutputMessageType};
use crate::ai::agent_providers::tools::skill::SkillResolutionError;
use crate::ai::skills::SkillDescriptor;

#[cfg(windows)]
const LOCAL_SKILL_PATH: &str = r"C:\Users\local\.agents\skills\localmind\SKILL.md";
#[cfg(not(windows))]
const LOCAL_SKILL_PATH: &str = "/Users/local/.agents/skills/localmind/SKILL.md";

fn descriptor(name: &str, reference: SkillReference) -> SkillDescriptor {
    SkillDescriptor {
        reference,
        name: name.to_owned(),
        description: "测试技能".to_owned(),
        scope: SkillScope::Home,
        provider: SkillProvider::Agents,
        icon_override: None,
    }
}

fn remote_reference(host: &str, path: &str) -> SkillReference {
    SkillReference::Path(LocalOrRemotePath::Remote(RemotePath::new(
        HostId::new(host.to_owned()),
        StandardizedPath::try_new(path).unwrap(),
    )))
}

fn remote_origin(host: &str) -> SkillPathOrigin {
    SkillPathOrigin::Remote {
        host_id: HostId::new(host.to_owned()),
    }
}

fn read_skill_call(name: &str) -> ToolCall {
    ToolCall {
        call_id: "read-skill-call".to_owned(),
        fn_name: "read_skill".to_owned(),
        fn_arguments: json!({ "name": name }),
        thought_signatures: None,
    }
}

fn action_reference(
    tool: api::message::tool_call::Tool,
    origin: &SkillPathOrigin,
) -> SkillReference {
    let task_id = TaskId::new("read-skill-task".to_owned());
    // 走应用实际使用的转换入口，避免旧的 TryFrom 丢失 SSH 主机身份。
    let output = make_tool_call_message("read-skill-task", "request", "read-skill-call", tool)
        .to_client_output_message(ConversionParams {
            task_id: &task_id,
            current_todo_list: None,
            active_code_review: None,
            skill_path_origin: origin,
        })
        .unwrap();
    let MaybeAIAgentOutputMessage::Message(output) = output else {
        panic!("读取技能必须产生客户端动作");
    };
    let AIAgentOutputMessageType::Action(action) = output.message else {
        panic!("读取技能消息必须转换为动作");
    };
    assert_eq!(action.task_id, task_id);
    let AIAgentActionType::ReadSkill(request) = action.action else {
        panic!("客户端动作必须保留读取技能类型");
    };
    request.skill
}

#[test]
fn local_skill_name_resolves_before_client_conversion_and_history_preserves_name() {
    let reference = SkillReference::Path(LocalOrRemotePath::Local(PathBuf::from(LOCAL_SKILL_PATH)));
    let skills = vec![descriptor("localmind", reference.clone())];

    let tool = parse_incoming_tool_call(
        &read_skill_call("localmind"),
        None,
        &skills,
        &SkillPathOrigin::Local,
    )
    .unwrap();
    let api::message::tool_call::Tool::ReadSkill(read_skill) = &tool else {
        panic!("解析结果必须是读取技能工具");
    };
    assert_eq!(read_skill.name, "localmind");
    assert_eq!(
        read_skill.skill_reference,
        Some(
            api::message::tool_call::read_skill::SkillReference::SkillPath(
                LOCAL_SKILL_PATH.to_owned()
            )
        )
    );
    let history = serialize_outgoing_tool_call(
        &api::message::ToolCall {
            tool_call_id: "read-skill-call".to_owned(),
            tool: Some(tool.clone()),
        },
        None,
        "",
    );
    assert_eq!(
        history,
        ("read_skill".to_owned(), json!({ "name": "localmind" }))
    );
    assert_eq!(action_reference(tool, &SkillPathOrigin::Local), reference);
}

#[test]
fn ssh_skill_name_ignores_same_name_on_local_and_other_remote_hosts() {
    let reference = remote_reference("ssh-host", "/home/member/.agents/skills/localmind/SKILL.md");
    let skills = vec![
        descriptor(
            "localmind",
            SkillReference::Path(LocalOrRemotePath::Local(PathBuf::from(LOCAL_SKILL_PATH))),
        ),
        descriptor(
            "localmind",
            remote_reference(
                "other-host",
                "/home/member/.agents/skills/localmind/SKILL.md",
            ),
        ),
        descriptor("localmind", reference.clone()),
    ];
    let origin = remote_origin("ssh-host");

    let tool =
        parse_incoming_tool_call(&read_skill_call("localmind"), None, &skills, &origin).unwrap();

    assert_eq!(action_reference(tool, &origin), reference);
}

#[test]
fn ssh_windows_skill_display_path_preserves_windows_path_and_host() {
    let reference = remote_reference(
        "windows-host",
        r"C:\Users\member\.agents\skills\localmind\SKILL.md",
    );
    let skills = vec![descriptor("localmind", reference.clone())];
    let origin = remote_origin("windows-host");

    let tool = parse_incoming_tool_call(
        &read_skill_call(r"C:\Users\member\.agents\skills\localmind\SKILL.md"),
        None,
        &skills,
        &origin,
    )
    .unwrap();

    assert_eq!(action_reference(tool, &origin), reference);
}

#[test]
fn local_origin_rejects_skill_listed_only_on_remote_host() {
    let skills = vec![descriptor(
        "localmind",
        remote_reference("ssh-host", "/home/member/.agents/skills/localmind/SKILL.md"),
    )];

    let error = parse_incoming_tool_call(
        &read_skill_call("localmind"),
        None,
        &skills,
        &SkillPathOrigin::Local,
    )
    .unwrap_err();

    assert!(matches!(
        error.downcast_ref::<SkillResolutionError>(),
        Some(SkillResolutionError::NotAvailable)
    ));
}

#[test]
fn unknown_origin_cannot_execute_listed_skills() {
    let skills = vec![descriptor(
        "localmind",
        SkillReference::Path(LocalOrRemotePath::Local(PathBuf::from(LOCAL_SKILL_PATH))),
    )];

    let unavailable = parse_incoming_tool_call(
        &read_skill_call("localmind"),
        None,
        &skills,
        &SkillPathOrigin::Unavailable,
    )
    .unwrap_err();
    let display_only = parse_incoming_tool_call(
        &read_skill_call("localmind"),
        None,
        &skills,
        &SkillPathOrigin::RestoredDisplayOnly,
    )
    .unwrap_err();
    let unknown_host = parse_incoming_tool_call(
        &read_skill_call("localmind"),
        None,
        &skills,
        &remote_origin("unknown-host"),
    )
    .unwrap_err();

    assert!(matches!(
        unavailable.downcast_ref::<SkillResolutionError>(),
        Some(SkillResolutionError::NotAvailable)
    ));
    assert!(matches!(
        display_only.downcast_ref::<SkillResolutionError>(),
        Some(SkillResolutionError::NotAvailable)
    ));
    assert!(matches!(
        unknown_host.downcast_ref::<SkillResolutionError>(),
        Some(SkillResolutionError::NotAvailable)
    ));
}

#[test]
fn local_bundled_reference_resolves_to_bundled_id() {
    let reference = SkillReference::BundledSkillId("troubleshooting".to_owned());
    let mut skill = descriptor("diagnose", reference.clone());
    skill.scope = SkillScope::Bundled;
    skill.provider = SkillProvider::InfiniShell;

    let tool = parse_incoming_tool_call(
        &read_skill_call("@warp-skill:troubleshooting"),
        None,
        &[skill],
        &SkillPathOrigin::Local,
    )
    .unwrap();

    assert_eq!(action_reference(tool, &SkillPathOrigin::Local), reference);
}

#[test]
fn ssh_bundled_skill_uses_remote_file_instead_of_local_bundled_id() {
    let reference = remote_reference(
        "ssh-host",
        "/home/member/.infinishell/skills/diagnose/SKILL.md",
    );
    let mut remote_skill = descriptor("diagnose", reference.clone());
    remote_skill.scope = SkillScope::Bundled;
    remote_skill.provider = SkillProvider::InfiniShell;
    let skills = vec![
        descriptor(
            "diagnose",
            SkillReference::BundledSkillId("troubleshooting".to_owned()),
        ),
        remote_skill,
    ];
    let origin = remote_origin("ssh-host");

    let tool =
        parse_incoming_tool_call(&read_skill_call("diagnose"), None, &skills, &origin).unwrap();
    let error = parse_incoming_tool_call(
        &read_skill_call("@warp-skill:troubleshooting"),
        None,
        &skills,
        &origin,
    )
    .unwrap_err();

    assert_eq!(action_reference(tool, &origin), reference);
    assert!(matches!(
        error.downcast_ref::<SkillResolutionError>(),
        Some(SkillResolutionError::NotAvailable)
    ));
}

#[test]
fn ambiguous_skill_name_requires_exact_available_path() {
    let reference = remote_reference("ssh-host", "/repo/.agents/skills/localmind/SKILL.md");
    let skills = vec![
        descriptor(
            "localmind",
            remote_reference("ssh-host", "/home/member/.agents/skills/localmind/SKILL.md"),
        ),
        descriptor("localmind", reference.clone()),
    ];
    let origin = remote_origin("ssh-host");

    let error = parse_incoming_tool_call(&read_skill_call("localmind"), None, &skills, &origin)
        .unwrap_err();
    let tool = parse_incoming_tool_call(
        &read_skill_call("/repo/.agents/skills/localmind/SKILL.md"),
        None,
        &skills,
        &origin,
    )
    .unwrap();

    assert!(matches!(
        error.downcast_ref::<SkillResolutionError>(),
        Some(SkillResolutionError::Ambiguous)
    ));
    assert_eq!(action_reference(tool, &origin), reference);
}

#[test]
fn absolute_skill_path_not_in_request_list_is_unavailable() {
    let error = parse_incoming_tool_call(
        &read_skill_call("/home/member/.agents/skills/localmind/SKILL.md"),
        None,
        &[],
        &remote_origin("ssh-host"),
    )
    .unwrap_err();

    assert!(matches!(
        error.downcast_ref::<SkillResolutionError>(),
        Some(SkillResolutionError::NotAvailable)
    ));
}

#[test]
fn incomplete_skill_arguments_cannot_become_executable_tool() {
    let reference = remote_reference("ssh-host", "/home/member/.agents/skills/localmind/SKILL.md");
    let skills = vec![descriptor("localmind", reference.clone())];
    let origin = remote_origin("ssh-host");
    let mut call = read_skill_call("localmind");
    call.fn_arguments = json!(r#"{"name":"local"#);

    assert!(parse_incoming_tool_call(&call, None, &skills, &origin).is_err());

    call.fn_arguments = json!(r#"{"name":"localmind"}"#);
    let tool = parse_incoming_tool_call(&call, None, &skills, &origin).unwrap();

    assert_eq!(action_reference(tool, &origin), reference);
}
