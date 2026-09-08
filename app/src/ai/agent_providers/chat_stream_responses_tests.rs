use std::path::PathBuf;

use ai::skills::{SkillProvider, SkillReference, SkillScope};
use mockito::Matcher;
use warp_util::local_or_remote_path::LocalOrRemotePath;

use super::*;
use crate::ai::agent::api::{
    ConversionParams, ConvertAPIMessageToClientOutputMessage, MaybeAIAgentOutputMessage,
};
use crate::ai::agent::task::TaskId;
use crate::ai::agent::{AIAgentActionType, AIAgentOutputMessageType};
use crate::ai::agent_providers::responses::response_request_context_fingerprint;
use crate::settings::{AgentProviderResponsesOptions, ResponsesStateModeSetting};

const COMPLETED_RESPONSES_STREAM: &str = concat!(
    "data: {\"type\":\"response.output_text.delta\",\"sequence_number\":1,\"delta\":\"完成\"}\n\n",
    "data: {\"type\":\"response.completed\",\"sequence_number\":2,\"response\":{\"id\":\"resp_after\",\"status\":\"completed\",\"output\":[{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"完成\"}]}],\"usage\":{\"input_tokens\":1000,\"output_tokens\":10,\"total_tokens\":1010}}}\n\n",
);

fn responses_continuation_params() -> RequestParams {
    let mut params = RequestParams::new_for_test(
        vec![AIAgentInput::UserQuery {
            query: "继续".to_owned(),
            context: Arc::from([]),
            static_query_type: None,
            referenced_attachments: HashMap::new(),
            user_query_mode: UserQueryMode::default(),
            running_command: None,
            intended_agent: None,
        }],
        vec![],
    );
    params.context_window_limit = Some(64_000);
    params
}

async fn collect_responses_request(
    params: RequestParams,
    base_url: String,
    responses: AgentProviderResponsesOptions,
) -> Vec<api::ResponseEvent> {
    let (_cancel, cancellation_rx) = futures::channel::oneshot::channel();
    let stream = generate_byop_output(ByopOutputInput {
        params,
        base_url,
        api_key: "test-key".to_owned(),
        model_id: "gpt-5.6".to_owned(),
        api_type: AgentProviderApiType::OpenAiResp,
        reasoning_effort: crate::settings::ReasoningEffortSetting::Auto,
        extra_headers: vec![],
        responses,
        task_id: "task-1".to_owned(),
        target_task_id: "task-1".to_owned(),
        needs_create_task: false,
        lrc_command_id: None,
        lrc_should_spawn_subagent: false,
        context_window: Some(64_000),
        cancellation_rx,
        attachment_caps: Default::default(),
    })
    .await
    .expect("有效的 Responses 请求不应被本地估算阻断");
    stream
        .map(|event| event.expect("提供商应正常完成响应"))
        .collect()
        .await
}

#[cfg(windows)]
const STREAM_SKILL_PATH: &str = r"C:\Users\member\.agents\skills\localmind\SKILL.md";
#[cfg(not(windows))]
const STREAM_SKILL_PATH: &str = "/home/member/.agents/skills/localmind/SKILL.md";

fn skill_stream_params() -> RequestParams {
    let mut params = responses_continuation_params();
    let AIAgentInput::UserQuery { context, .. } = &mut params.input[0] else {
        panic!("技能流测试必须从用户输入开始");
    };
    *context = vec![AIAgentContext::Skills {
        skills: vec![SkillDescriptor {
            reference: SkillReference::Path(LocalOrRemotePath::Local(PathBuf::from(
                STREAM_SKILL_PATH,
            ))),
            name: "localmind".to_owned(),
            description: "测试技能".to_owned(),
            scope: SkillScope::Home,
            provider: SkillProvider::Agents,
            icon_override: None,
        }],
    }]
    .into();
    params
}

fn skill_responses_stream(final_name: &str) -> String {
    // 中间帧已经是合法参数；终帧的权威条目可以覆盖它，不能提前留下可执行动作。
    let events = [
        json!({
            "type": "response.output_item.added",
            "sequence_number": 1,
            "output_index": 0,
            "item": {
                "type": "function_call", "id": "fc_skill", "call_id": "skill-call",
                "name": "read_skill", "arguments": ""
            }
        }),
        json!({
            "type": "response.function_call_arguments.delta",
            "sequence_number": 2,
            "output_index": 0,
            "delta": r#"{"name":"localmind"}"#
        }),
        json!({
            "type": "response.completed",
            "sequence_number": 3,
            "response": {
                "id": "resp_skill", "status": "completed",
                "output": [{
                    "type": "function_call", "id": "fc_skill", "call_id": "skill-call",
                    "name": "read_skill", "arguments": json!({ "name": final_name }).to_string()
                }],
                "usage": { "input_tokens": 1000, "output_tokens": 10, "total_tokens": 1010 }
            }
        }),
    ];
    events
        .iter()
        .map(|event| format!("data: {event}\n\n"))
        .collect()
}

fn skill_stream_messages(events: &[api::ResponseEvent]) -> Vec<api::Message> {
    events
        .iter()
        .filter_map(|event| {
            let Some(api::response_event::Type::ClientActions(actions)) = &event.r#type else {
                return None;
            };
            Some(&actions.actions)
        })
        .flatten()
        .flat_map(|action| match &action.action {
            Some(api::client_action::Action::AddMessagesToTask(add)) => add.messages.clone(),
            Some(api::client_action::Action::UpdateTaskMessage(_)) => {
                panic!("技能参数应在终帧一次生成，不应更新中间占位卡");
            }
            // 本测试只收集消息；初始化和完成事件不写入任务历史。
            _ => vec![],
        })
        .collect()
}

#[tokio::test]
async fn responses技能终帧不可用时不遗留中间动作且下一轮可以重试() {
    let mut server = mockito::Server::new_async().await;
    let count = server
        .mock("POST", "/v1/responses/input_tokens")
        .with_header("content-type", "application/json")
        .with_body(r#"{"input_tokens":1000}"#)
        .expect(1)
        .create_async()
        .await;
    let response = server
        .mock("POST", "/v1/responses")
        .with_header("content-type", "text/event-stream")
        .with_body(skill_responses_stream("not-listed"))
        .expect(1)
        .create_async()
        .await;
    let mut params = skill_stream_params();

    let events = collect_responses_request(
        params.clone(),
        format!("{}/v1", server.url()),
        AgentProviderResponsesOptions::default(),
    )
    .await;

    count.assert_async().await;
    response.assert_async().await;
    let messages = skill_stream_messages(&events);
    let tool_calls = messages
        .iter()
        .filter_map(|message| {
            let Some(api::message::Message::ToolCall(call)) = &message.message else {
                return None;
            };
            Some((message, call))
        })
        .collect::<Vec<_>>();
    assert_eq!(tool_calls.len(), 1, "不能残留已经解析的中间 ReadSkill 动作");
    let (carrier, call) = tool_calls[0];
    assert_eq!(call.tool_call_id, "skill-call");
    assert!(call.tool.is_none());
    assert_eq!(
        serialize_outgoing_tool_call(call, None, &carrier.server_message_data),
        ("read_skill".to_owned(), json!({ "name": "not-listed" }))
    );
    let results = messages
        .iter()
        .filter_map(|message| {
            let Some(api::message::Message::ToolCallResult(result)) = &message.message else {
                return None;
            };
            Some((message, result))
        })
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.tool_call_id, "skill-call");
    let error: Value = serde_json::from_str(&results[0].0.server_message_data).unwrap();
    assert_eq!(error["error"], "skill_unavailable");

    params.tasks = vec![api::Task {
        id: "task-1".to_owned(),
        messages,
        ..Default::default()
    }];
    params.byop_target_task_id = Some("task-1".to_owned());
    assert_eq!(
        classify_byop_controller_readiness(&params).state,
        ReadinessState::Ready
    );
    let request = build_chat_request(
        &params,
        true,
        false,
        false,
        AgentProviderApiType::OpenAiResp,
        Default::default(),
    )
    .expect("错误工具回执应允许重新构建下一轮请求")
    .0;
    // 原生 Responses 历史以 Custom 条目保留在 sidecar 中；验证真正出站的 JSON，
    // 而非只检查通用 ToolCall 视图，后者有意跳过这些条目以避免重复回放。
    let payload =
        genai::responses::build_request_payload("gpt-5.6", request, &ChatOptions::default(), true)
            .expect("错误回执必须能序列化为下一轮 Responses 请求");
    let replayed_items = payload["input"].as_array().expect("input 必须是数组");
    let replayed_calls = replayed_items
        .iter()
        .filter(|item| item["type"] == "function_call")
        .collect::<Vec<_>>();
    assert_eq!(replayed_calls.len(), 1);
    assert_eq!(replayed_calls[0]["call_id"], "skill-call");
    assert_eq!(replayed_calls[0]["name"], "read_skill");
    let replayed_arguments: Value =
        serde_json::from_str(replayed_calls[0]["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(replayed_arguments, json!({ "name": "not-listed" }));
    let replayed_results = replayed_items
        .iter()
        .filter(|item| item["type"] == "function_call_output")
        .collect::<Vec<_>>();
    assert_eq!(replayed_results.len(), 1);
    assert_eq!(replayed_results[0]["call_id"], "skill-call");
    let replayed_error: Value =
        serde_json::from_str(replayed_results[0]["output"].as_str().unwrap()).unwrap();
    assert_eq!(replayed_error["error"], "skill_unavailable");
}

#[tokio::test]
async fn responses已列出技能只在终帧生成一次可执行动作() {
    let mut server = mockito::Server::new_async().await;
    let count = server
        .mock("POST", "/v1/responses/input_tokens")
        .with_header("content-type", "application/json")
        .with_body(r#"{"input_tokens":1000}"#)
        .expect(1)
        .create_async()
        .await;
    let response = server
        .mock("POST", "/v1/responses")
        .with_header("content-type", "text/event-stream")
        .with_body(skill_responses_stream("localmind"))
        .expect(1)
        .create_async()
        .await;

    let events = collect_responses_request(
        skill_stream_params(),
        format!("{}/v1", server.url()),
        AgentProviderResponsesOptions::default(),
    )
    .await;

    count.assert_async().await;
    response.assert_async().await;
    let messages = skill_stream_messages(&events);
    let tool_messages = messages
        .iter()
        .filter(|message| matches!(message.message, Some(api::message::Message::ToolCall(_))))
        .collect::<Vec<_>>();
    assert_eq!(tool_messages.len(), 1);
    let task_id = TaskId::new("task-1".to_owned());
    let output = tool_messages[0]
        .clone()
        .to_client_output_message(ConversionParams {
            task_id: &task_id,
            current_todo_list: None,
            active_code_review: None,
            skill_path_origin: &SkillPathOrigin::Local,
        })
        .unwrap();
    let MaybeAIAgentOutputMessage::Message(output) = output else {
        panic!("终帧技能必须产生客户端消息");
    };
    let AIAgentOutputMessageType::Action(action) = output.message else {
        panic!("终帧技能必须产生客户端动作");
    };
    let AIAgentActionType::ReadSkill(request) = action.action else {
        panic!("终帧动作必须读取技能");
    };
    assert_eq!(
        request.skill,
        SkillReference::Path(LocalOrRemotePath::Local(PathBuf::from(STREAM_SKILL_PATH)))
    );
    assert!(!messages.iter().any(|message| matches!(
        message.message,
        Some(api::message::Message::ToolCallResult(_))
    )));
}

#[tokio::test]
async fn responses续接以远端计数为准不被旧历史估算阻断() {
    let mut server = mockito::Server::new_async().await;
    let base_url = format!("{}/v1", server.url());
    let count = server
        .mock("POST", "/v1/responses/input_tokens")
        .match_body(Matcher::PartialJson(
            json!({"previous_response_id": "resp_before"}),
        ))
        .with_header("content-type", "application/json")
        .with_body(r#"{"input_tokens":1000}"#)
        .expect(1)
        .create_async()
        .await;
    let response = server
        .mock("POST", "/v1/responses")
        .match_body(Matcher::PartialJson(
            json!({"previous_response_id": "resp_before"}),
        ))
        .with_header("content-type", "text/event-stream")
        .with_body(COMPLETED_RESPONSES_STREAM)
        .expect(1)
        .create_async()
        .await;
    let mut params = responses_continuation_params();
    let (request, _) = build_chat_request(
        &params,
        true,
        false,
        false,
        AgentProviderApiType::OpenAiResp,
        Default::default(),
    )
    .expect("短请求应能构造");
    let tools = request.tools.as_ref().map(|tools| {
        tools
            .iter()
            .map(|tool| serde_json::to_value(tool).expect("工具应能序列化"))
            .collect::<Vec<_>>()
    });
    let state = ProviderResponseState {
        response_id: Some("resp_before".to_owned()),
        request_context_fingerprint: Some(response_request_context_fingerprint(
            &base_url,
            Some("gpt-5.6"),
            request.system.as_deref(),
            tools.as_deref(),
        )),
        state_mode: Some(ResponsesStateModeSetting::PreviousResponse),
        ..Default::default()
    };
    // 远端状态已压缩；本地仍保留完整原文，不能据其字符数拒绝合法的续接。
    let mut previous_output =
        make_agent_output_message("task-1", "request-before", "x".repeat(160_000));
    previous_output.server_message_data =
        encode_provider_response_state(&state).expect("状态应能编码");
    params.tasks = vec![api::Task {
        id: "task-1".to_owned(),
        messages: vec![
            make_user_query_message("task-1", "request-before", "先前任务".to_owned(), &[]),
            previous_output,
        ],
        ..Default::default()
    }];
    params.byop_target_task_id = Some("task-1".to_owned());

    let events = collect_responses_request(
        params,
        base_url,
        AgentProviderResponsesOptions {
            state_mode: ResponsesStateModeSetting::PreviousResponse,
            ..Default::default()
        },
    )
    .await;

    count.assert_async().await;
    response.assert_async().await;
    assert!(events.iter().any(|event| matches!(
        &event.r#type,
        Some(api::response_event::Type::Finished(finished))
            if matches!(finished.reason.as_ref(), Some(api::response_event::stream_finished::Reason::Done(_)))
    )));
}

#[tokio::test]
async fn responses服务端压缩不被固定本地预留比例阻断() {
    let mut server = mockito::Server::new_async().await;
    let count = server
        .mock("POST", "/v1/responses/input_tokens")
        .with_header("content-type", "application/json")
        .with_body(r#"{"input_tokens":52000}"#)
        .expect(1)
        .create_async()
        .await;
    let response = server
        .mock("POST", "/v1/responses")
        .match_body(Matcher::PartialJson(json!({
            "context_management": [{"type": "compaction", "compact_threshold": 40_000}]
        })))
        .with_header("content-type", "text/event-stream")
        .with_body(COMPLETED_RESPONSES_STREAM)
        .expect(1)
        .create_async()
        .await;

    let events = collect_responses_request(
        responses_continuation_params(),
        format!("{}/v1", server.url()),
        AgentProviderResponsesOptions {
            compact_threshold: 40_000,
            ..Default::default()
        },
    )
    .await;

    count.assert_async().await;
    response.assert_async().await;
    assert!(events.iter().any(|event| matches!(
        &event.r#type,
        Some(api::response_event::Type::Finished(finished))
            if matches!(finished.reason.as_ref(), Some(api::response_event::stream_finished::Reason::Done(_)))
    )));
}

fn generated_provider_state_message(events: &[api::ResponseEvent]) -> api::Message {
    events
        .iter()
        .filter_map(|event| {
            let Some(api::response_event::Type::ClientActions(actions)) = &event.r#type else {
                return None;
            };
            Some(&actions.actions)
        })
        .flatten()
        .find_map(|action| {
            let Some(api::client_action::Action::UpdateTaskMessage(update)) = &action.action else {
                return None;
            };
            update.message.as_ref().filter(|message| {
                decode_provider_response_state(&message.server_message_data).is_some()
            })
        })
        .expect("真实响应应生成可持久化的 Responses 状态")
        .clone()
}

#[tokio::test]
async fn responses相同裁剪历史继续复用链且同长度内容变化会重建() {
    let mut server = mockito::Server::new_async().await;
    let base_url = format!("{}/v1", server.url());
    let count = server
        .mock("POST", "/v1/responses/input_tokens")
        .with_header("content-type", "application/json")
        .with_body(r#"{"input_tokens":1000}"#)
        .expect(3)
        .create_async()
        .await;
    let first_response = server
        .mock("POST", "/v1/responses")
        .match_request(|request| {
            let body: Value = serde_json::from_slice(request.body().expect("请求应包含正文"))
                .expect("请求应为 JSON");
            let output = body["input"]
                .as_array()
                .expect("完整回放应包含 input 数组")
                .iter()
                .find(|item| item["type"] == "function_call_output")
                .and_then(|item| item["output"].as_str());
            body.get("previous_response_id").is_none()
                && output.is_some_and(|output| {
                    output.len() <= 32_768 && output.contains("_infinishell_truncated")
                })
        })
        .with_header("content-type", "text/event-stream")
        .with_body(COMPLETED_RESPONSES_STREAM)
        .expect(1)
        .create_async()
        .await;
    let options = AgentProviderResponsesOptions {
        state_mode: ResponsesStateModeSetting::PreviousResponse,
        ..Default::default()
    };
    let mut params = responses_continuation_params();
    params.tasks = vec![api::Task {
        id: "task-1".to_owned(),
        messages: vec![
            make_user_query_message("task-1", "request-before", "读取日志".to_owned(), &[]),
            make_tool_call_carrier_message(
                "task-1",
                "request-before",
                "call-log",
                "run_shell_command",
                r#"{"command":"cat application.log"}"#,
            ),
            make_tool_call_result_message(
                "task-1",
                "request-before",
                "call-log".to_owned(),
                json!({"status": "completed", "exit_code": 0, "output": "x".repeat(160_000)})
                    .to_string(),
            ),
        ],
        ..Default::default()
    }];
    params.byop_target_task_id = Some("task-1".to_owned());

    let first_events =
        collect_responses_request(params.clone(), base_url.clone(), options.clone()).await;
    first_response.assert_async().await;
    first_response.remove_async().await;
    // 从实际流事件取出状态载体，下一轮使用与控制器持久化一致的 sidecar。
    let first_state = generated_provider_state_message(&first_events);
    assert_eq!(
        decode_provider_response_state(&first_state.server_message_data)
            .expect("响应状态应合法")
            .response_id
            .as_deref(),
        Some("resp_after")
    );
    params.tasks[0].messages.push(first_state);
    let Some(AIAgentInput::UserQuery { query, .. }) = params.input.first_mut() else {
        panic!("下一轮应包含用户输入");
    };
    *query = "解释刚才的结果".to_owned();

    let second_response = server
        .mock("POST", "/v1/responses")
        .match_body(Matcher::PartialJson(
            json!({"previous_response_id": "resp_after"}),
        ))
        .with_header("content-type", "text/event-stream")
        .with_body(COMPLETED_RESPONSES_STREAM.replace("resp_after", "resp_second"))
        .expect(1)
        .create_async()
        .await;
    let second_events =
        collect_responses_request(params.clone(), base_url.clone(), options.clone()).await;
    second_response.assert_async().await;
    second_response.remove_async().await;
    params.tasks[0]
        .messages
        .push(generated_provider_state_message(&second_events));

    // 原文字节数不变，但实际发出的日志头尾已改变，不能只按长度误复用旧链。
    params.tasks[0].messages[2].server_message_data =
        json!({"status": "completed", "exit_code": 0, "output": "y".repeat(160_000)}).to_string();
    let third_response = server
        .mock("POST", "/v1/responses")
        .match_request(|request| {
            let body: Value = serde_json::from_slice(request.body().expect("请求应包含正文"))
                .expect("请求应为 JSON");
            let output = body["input"]
                .as_array()
                .expect("变更后的完整回放应包含 input 数组")
                .iter()
                .find(|item| item["type"] == "function_call_output")
                .and_then(|item| item["output"].as_str());
            body.get("previous_response_id").is_none()
                && output.is_some_and(|output| {
                    output.len() <= 32_768
                        && output.contains("_infinishell_truncated")
                        && output.contains("yyyy")
                })
        })
        .with_header("content-type", "text/event-stream")
        .with_body(COMPLETED_RESPONSES_STREAM.replace("resp_after", "resp_third"))
        .expect(1)
        .create_async()
        .await;

    collect_responses_request(params, base_url, options).await;

    count.assert_async().await;
    third_response.assert_async().await;
}

#[test]
fn provider_state完整回放未知responses条目() {
    let raw_item = json!({
        "type": "program",
        "id": "prog_1",
        "caller_id": "root_1",
        "future_field": [1, 2, 3]
    });
    let state = ProviderResponseState {
        response_id: Some("resp_1".to_owned()),
        conversation_id: Some("conv_1".to_owned()),
        request_context_fingerprint: Some("fingerprint".to_owned()),
        last_sequence_number: Some(42),
        state_mode: Some(crate::settings::ResponsesStateModeSetting::Conversation),
        encrypted_reasoning_items: vec!["signature".to_owned()],
        response_items: vec![raw_item.clone()],
        unknown_events: vec![json!({"type": "response.future.delta", "value": true})],
    };

    let encoded = encode_provider_response_state(&state).expect("状态应能编码");
    let decoded = decode_provider_response_state(&encoded).expect("状态应能解码");

    assert_eq!(decoded.response_id.as_deref(), Some("resp_1"));
    assert_eq!(decoded.last_sequence_number, Some(42));
    assert_eq!(decoded.response_items, vec![raw_item]);
    assert_eq!(decoded.unknown_events[0]["value"], true);
}

#[test]
fn provider_state向后兼容旧sidecar() {
    let encoded = format!(
        "{PROVIDER_RESPONSE_STATE_PREFIX}{}",
        json!({
            "response_id": "resp_old",
            "encrypted_reasoning_items": ["legacy-signature"]
        })
    );

    let decoded = decode_provider_response_state(&encoded).expect("旧 sidecar 应能解码");

    assert_eq!(decoded.response_id.as_deref(), Some("resp_old"));
    assert_eq!(decoded.encrypted_reasoning_items, ["legacy-signature"]);
    assert!(decoded.response_items.is_empty());
    assert!(decoded.unknown_events.is_empty());
}

#[test]
fn ptc只允许无副作用且可验证的工具() {
    for name in [
        "read_files",
        "grep",
        "file_glob",
        "read_shell_command_output",
        "read_skill",
        "read_documents",
        tools::webfetch::TOOL_NAME,
        tools::websearch::TOOL_NAME,
    ] {
        assert!(ptc_allows_tool(name), "{name} 应允许由 PTC 调用");
    }
    for name in [
        "run_shell_command",
        "apply_file_diffs",
        "write_to_long_running_shell_command",
        "create_documents",
        "edit_documents",
        "update_machine_memory",
        "run_command_on_hosts",
        "todowrite",
        "ask_user_question",
    ] {
        assert!(!ptc_allows_tool(name), "{name} 不应允许由 PTC 调用");
    }
}

#[test]
fn gpt56高级能力识别别名变体和快照() {
    for model in [
        "gpt-5.6",
        "gpt-5.6-sol",
        "gpt-5.6-terra-2026-08-01",
        "openai/gpt-5.6-luna",
    ] {
        assert!(is_gpt56_model(model), "{model} 应识别为 GPT-5.6 系列");
    }
    for model in ["gpt-5.5", "gpt-5.4", "gpt-4o"] {
        assert!(!is_gpt56_model(model), "{model} 不应识别为 GPT-5.6 系列");
    }
}

#[test]
fn responses出站工具使用规范化后的strict_schema() {
    let params = RequestParams::new_for_test(Vec::new(), Vec::new());
    let tools = build_tools_array(&params, false, true);
    let shell = tools
        .iter()
        .find(|tool| tool.name == "run_shell_command".into())
        .expect("应包含 run_shell_command");

    assert_eq!(shell.strict, Some(true));
    let schema = shell.schema.as_ref().expect("strict tool 应包含 schema");
    assert_eq!(
        schema["required"],
        json!([
            "command",
            "is_read_only",
            "is_risky",
            "uses_pager",
            "wait_until_complete"
        ])
    );
    assert_eq!(
        schema["properties"]["is_read_only"]["type"],
        json!(["boolean", "null"])
    );
}

#[test]
fn responses工具调用在本地解析前删除可选null() {
    let mut call = ToolCall {
        call_id: "call_1".to_owned(),
        fn_name: "run_shell_command".to_owned(),
        fn_arguments: json!({
            "command": "pwd",
            "is_read_only": null,
            "uses_pager": null,
            "is_risky": null,
            "wait_until_complete": null
        }),
        thought_signatures: None,
    };

    normalize_responses_tool_call_arguments(&mut call);

    assert_eq!(call.fn_arguments, json!({"command": "pwd"}));
    parse_incoming_tool_call(&call, None, &[], &SkillPathOrigin::Local)
        .expect("恢复缺省参数后应能转换为内部工具调用");
}

#[test]
fn responses混合本地与审批工具结果保持就绪() {
    let task_id = "task-1";
    let request_id = "request-1";
    let web_call_1 = "web-call-1";
    let web_call_2 = "web-call-2";
    let shell_call_1 = "shell-call-1";
    let shell_call_2 = "shell-call-2";
    let state = ProviderResponseState {
        response_items: vec![
            json!({"type": "function_call", "call_id": web_call_1, "name": "websearch", "arguments": "{}"}),
            json!({"type": "function_call", "call_id": web_call_2, "name": "websearch", "arguments": "{}"}),
            json!({"type": "function_call", "call_id": shell_call_1, "name": "run_shell_command", "arguments": "{}"}),
            json!({"type": "function_call", "call_id": shell_call_2, "name": "run_shell_command", "arguments": "{}"}),
        ],
        ..Default::default()
    };
    let mut state_message = make_reasoning_message(task_id, request_id, String::new());
    state_message.server_message_data = encode_provider_response_state(&state).expect("应编码状态");
    let messages = vec![
        state_message,
        make_tool_call_carrier_message(
            task_id,
            request_id,
            shell_call_1,
            "run_shell_command",
            "{}",
        ),
        make_tool_call_carrier_message(
            task_id,
            request_id,
            shell_call_2,
            "run_shell_command",
            "{}",
        ),
        make_tool_call_carrier_message(task_id, request_id, web_call_1, "websearch", "{}"),
        make_tool_call_result_message(
            task_id,
            request_id,
            web_call_1.to_owned(),
            json!({"_byop_intercepted": true, "status": "ok"}).to_string(),
        ),
        make_tool_call_carrier_message(task_id, request_id, web_call_2, "websearch", "{}"),
        make_tool_call_result_message(
            task_id,
            request_id,
            web_call_2.to_owned(),
            json!({"_byop_intercepted": true, "status": "ok"}).to_string(),
        ),
        make_tool_call_result_message(
            task_id,
            "byop-preflight",
            shell_call_1.to_owned(),
            json!({"status": "ok"}).to_string(),
        ),
        make_tool_call_result_message(
            task_id,
            "byop-preflight",
            shell_call_2.to_owned(),
            json!({"status": "ok"}).to_string(),
        ),
    ];
    let params = RequestParams::new_for_test(
        Vec::new(),
        vec![api::Task {
            id: task_id.to_owned(),
            messages,
            dependencies: None,
            description: String::new(),
            summary: String::new(),
            server_data: String::new(),
        }],
    );

    let report = classify_byop_controller_readiness(&params);

    assert_eq!(report.state, ReadinessState::Ready);

    let request = build_chat_request(
        &params,
        true,
        false,
        false,
        AgentProviderApiType::OpenAiResp,
        attachment_caps::AttachmentCaps::default(),
    )
    .expect("Responses 原始条目和工具结果应形成合法请求")
    .0;
    let result_ids = request
        .messages
        .iter()
        .flat_map(|message| message.content.tool_responses())
        .map(|response| response.call_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        result_ids,
        [web_call_1, web_call_2, shell_call_1, shell_call_2]
    );
}

#[test]
fn responses原始调用沿用本地载体的运行状态() {
    let task_id = "task-1";
    let request_id = "request-1";
    let call_id = "shell-call-1";
    let state = ProviderResponseState {
        response_items: vec![json!({
            "type": "function_call",
            "call_id": call_id,
            "name": "run_shell_command",
            "arguments": "{}"
        })],
        ..Default::default()
    };
    let mut state_message = make_reasoning_message(task_id, request_id, String::new());
    state_message.server_message_data = encode_provider_response_state(&state).expect("应编码状态");
    let tool_call_message =
        make_tool_call_carrier_message(task_id, request_id, call_id, "run_shell_command", "{}");
    let tool_call_message_id = tool_call_message.id.clone();
    let params = RequestParams::new_for_test(
        Vec::new(),
        vec![api::Task {
            id: task_id.to_owned(),
            messages: vec![state_message, tool_call_message],
            dependencies: None,
            description: String::new(),
            summary: String::new(),
            server_data: String::new(),
        }],
    );

    let report = classify_byop_controller_readiness_with_live_tool_calls(
        &params,
        vec![LiveToolCall::new(
            ToolCallRef::new(
                ToolCallKey::new(task_id, tool_call_message_id, call_id),
                RedactedToolKind::new("run_shell_command"),
            ),
            LiveToolCallState::Running,
        )],
    );

    assert!(matches!(
        report.state,
        ReadinessState::PendingToolResults { .. }
    ));
}

#[test]
fn responses状态续接包含已持久化的工具输出() {
    let task_id = "task-1";
    let request_id = "request-1";
    let call_id = "fc_websearch";
    let state = ProviderResponseState {
        response_id: Some("resp-1".to_owned()),
        response_items: vec![json!({
            "type": "function_call",
            "call_id": call_id,
            "name": "websearch",
            "arguments": "{}"
        })],
        ..Default::default()
    };
    let mut state_message = make_reasoning_message(task_id, request_id, String::new());
    state_message.server_message_data = encode_provider_response_state(&state).expect("应编码状态");
    let messages = vec![
        state_message,
        make_tool_call_result_message(
            task_id,
            "byop-preflight",
            call_id.to_owned(),
            json!({"status": "cancelled"}).to_string(),
        ),
    ];
    let input = AIAgentInput::UserQuery {
        query: "继续".to_owned(),
        context: Arc::<[AIAgentContext]>::from([]),
        static_query_type: None,
        referenced_attachments: HashMap::new(),
        user_query_mode: UserQueryMode::default(),
        running_command: None,
        intended_agent: None,
    };
    let params = RequestParams::new_for_test(
        vec![input],
        vec![api::Task {
            id: task_id.to_owned(),
            messages,
            dependencies: None,
            description: String::new(),
            summary: String::new(),
            server_data: String::new(),
        }],
    );

    let request = build_chat_request(
        &params,
        false,
        false,
        false,
        AgentProviderApiType::OpenAiResp,
        attachment_caps::AttachmentCaps::default(),
    )
    .expect("状态续接请求应合法")
    .0;

    assert_eq!(
        request
            .messages
            .first()
            .expect("工具输出应位于新用户输入之前")
            .content
            .tool_responses()
            .first()
            .map(|response| response.call_id.as_str()),
        Some(call_id)
    );
    assert!(
        request
            .messages
            .iter()
            .any(|message| message.role == ChatRole::User),
        "状态续接仍应包含用户的新输入"
    );

    let payload = genai::responses::build_request_payload(
        "gpt-5.6",
        request.with_previous_response_id("resp-1"),
        &ChatOptions::default(),
        true,
    )
    .expect("应构造 Responses 请求体");
    let input = payload["input"].as_array().expect("input 应为数组");
    let output = input
        .iter()
        .find(|item| item["type"] == "function_call_output")
        .expect("previous_response_id 续接必须携带工具输出");
    assert_eq!(output["call_id"], call_id);
    assert_eq!(payload["previous_response_id"], "resp-1");
}
