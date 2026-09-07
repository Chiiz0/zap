use super::*;
use crate::ai::agent::task::TaskId;
use crate::ai::agent::{AIAgentActionId, AIAgentActionResultType, RequestCommandOutputResult};
use warp_terminal::model::BlockId;

fn shell_history() -> api::Message {
    make_tool_call_carrier_message(
        "task-1",
        "request-1",
        "call-1",
        "run_shell_command",
        r#"{"command":"docker logs --since 2h app"}"#,
    )
}

fn params_with(messages: Vec<api::Message>, input: Vec<AIAgentInput>) -> RequestParams {
    RequestParams::new_for_test(
        input,
        vec![api::Task {
            id: "task-1".to_owned(),
            messages,
            dependencies: None,
            description: String::new(),
            summary: String::new(),
            server_data: String::new(),
        }],
    )
}

#[test]
fn oversized_persisted_log_is_bounded_on_replay_and_protocol_switch() {
    let original =
        json!({"status": "completed", "exit_code": 0, "output": "x".repeat(4_358_559)}).to_string();
    let params = params_with(
        vec![
            shell_history(),
            make_tool_call_result_message(
                "task-1",
                "request-1",
                "call-1".to_owned(),
                original.clone(),
            ),
        ],
        Vec::new(),
    );

    for api_type in [
        AgentProviderApiType::OpenAi,
        AgentProviderApiType::OpenAiResp,
        AgentProviderApiType::Anthropic,
    ] {
        let (request, report) = build_chat_request(
            &params,
            true,
            false,
            false,
            api_type,
            attachment_caps::AttachmentCaps::default(),
        )
        .unwrap();
        let results: Vec<_> = request
            .messages
            .iter()
            .flat_map(|message| message.content.tool_responses())
            .collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].call_id, "call-1");
        assert!(results[0].content.len() <= 32_768);
        assert_eq!(report.truncated_results, 1);
        assert_eq!(params.tasks[0].messages[1].server_message_data, original);

        if api_type == AgentProviderApiType::OpenAiResp {
            let payload = genai::responses::build_request_payload(
                "test-model",
                request,
                &ChatOptions::default(),
                true,
            )
            .unwrap();
            let output = payload["input"]
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["type"] == "function_call_output")
                .unwrap();
            assert_eq!(output["call_id"], "call-1");
            assert!(output["output"].as_str().unwrap().len() <= 32_768);
        }
    }
}

#[test]
fn current_action_result_is_bounded_without_losing_running_command_control() {
    let result = AIAgentActionResult {
        id: AIAgentActionId::from("call-1".to_owned()),
        task_id: TaskId::new("task-1".to_owned()),
        result: AIAgentActionResultType::RequestCommandOutput(
            RequestCommandOutputResult::LongRunningCommandSnapshot {
                block_id: BlockId::from("command-1".to_owned()),
                command: "docker logs --since 2h app".to_owned(),
                grid_contents: "日志\n".repeat(100_000),
                cursor: String::new(),
                is_alt_screen_active: false,
            },
        ),
    };
    let original = result.clone();
    let params = params_with(
        vec![shell_history()],
        vec![AIAgentInput::ActionResult {
            result,
            context: Arc::from([]),
        }],
    );

    let (request, report) = build_chat_request(
        &params,
        true,
        false,
        false,
        AgentProviderApiType::OpenAi,
        attachment_caps::AttachmentCaps::default(),
    )
    .unwrap();

    let results: Vec<_> = request
        .messages
        .iter()
        .flat_map(|message| message.content.tool_responses())
        .collect();
    assert_eq!(report.truncated_results, 1);
    let content: Value = serde_json::from_str(&results[0].content).unwrap();
    assert_eq!(content["command_id"], "command-1");
    assert_eq!(content["status"], "running");
    assert_eq!(content["is_alt_screen_active"], false);
    let AIAgentInput::ActionResult { result, .. } = &params.input[0] else {
        panic!("应保留当前结果")
    };
    assert_eq!(result.result, original.result);
}

#[test]
fn responses_delta_output_is_also_bounded() {
    let mut state_message =
        make_tool_call_carrier_message("task-1", "request-1", "call-1", "run_shell_command", "{}");
    state_message.server_message_data = encode_provider_response_state(&ProviderResponseState {
        response_items: vec![json!({"type":"function_call", "call_id":"call-1", "name":"run_shell_command", "arguments":"{}"})],
        ..Default::default()
    }).unwrap();
    let params = params_with(
        vec![
            state_message,
            make_tool_call_result_message(
                "task-1",
                "request-1",
                "call-1".to_owned(),
                json!({"output":"x".repeat(1_000_000)}).to_string(),
            ),
        ],
        Vec::new(),
    );

    let (request, report) = build_chat_request(
        &params,
        false,
        false,
        false,
        AgentProviderApiType::OpenAiResp,
        attachment_caps::AttachmentCaps::default(),
    )
    .unwrap();

    assert_eq!(report.truncated_results, 1);
    let results: Vec<_> = request
        .messages
        .iter()
        .flat_map(|message| message.content.tool_responses())
        .collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].call_id, "call-1");
    assert!(results[0].content.len() <= 32_768);
}
