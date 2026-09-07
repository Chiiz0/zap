use super::{MAX_RESULT_BYTES, MAX_TOTAL_RESULT_BYTES, TRUNCATION_KEY, apply};
use genai::chat::{
    Binary, ChatMessage, ChatRequest, ContentPart, MessageContent, ToolCall, ToolResponse,
};
use serde_json::{Value, json};

fn shell_result(call_id: &str, output: &str) -> ChatMessage {
    ToolResponse::new(
        call_id,
        json!({
            "status": "completed",
            "exit_code": 0,
            "command_id": "command-17",
            "output": output,
        })
        .to_string(),
    )
    .into()
}

fn tool_responses(request: &ChatRequest) -> Vec<&ToolResponse> {
    request
        .messages
        .iter()
        .flat_map(|message| message.content.tool_responses())
        .collect()
}

#[test]
fn megabytes_of_shell_logs_preserve_execution_metadata_and_both_ends() {
    let output = format!("日志开始\n{}\n日志结束", "x".repeat(4_360_000));
    let mut request = ChatRequest::new(vec![shell_result("call-17", &output)]);

    let report = apply(&mut request, Some(200_000)).expect("数 MB 日志应能裁剪后发送");

    let responses = tool_responses(&request);
    let response = responses[0];
    let content: Value = serde_json::from_str(&response.content).expect("结果必须仍为有效 JSON");
    assert_eq!(report.truncated_results, 1);
    assert_eq!(response.call_id, "call-17");
    assert_eq!(content["status"], "completed");
    assert_eq!(content["exit_code"], 0);
    assert_eq!(content["command_id"], "command-17");
    assert!(
        content["output"]
            .as_str()
            .unwrap()
            .starts_with("日志开始\n")
    );
    assert!(content["output"].as_str().unwrap().ends_with("\n日志结束"));
    assert!(content[TRUNCATION_KEY]["original_bytes"].as_u64().unwrap() > 4_360_000);
    assert!(serde_json::to_vec(&response.content).unwrap().len() <= MAX_RESULT_BYTES);
}

#[test]
fn oversized_command_metadata_cannot_discard_running_command_control() {
    let mut request = ChatRequest::new(vec![
        ToolResponse::new(
            "call-1",
            json!({
                "status": "running",
                "command": "echo hello\n".repeat(10_000),
                "command_id": "command-1",
                "exit_code": 0,
                "is_alt_screen_active": true,
                "output": "log\n".repeat(100_000),
            })
            .to_string(),
        )
        .into(),
    ]);

    apply(&mut request, Some(200_000)).expect("长命令应只缩减不影响轮询的字段");

    let responses = tool_responses(&request);
    let content: Value = serde_json::from_str(&responses[0].content).unwrap();
    assert_eq!(content["status"], "running");
    assert_eq!(content["command_id"], "command-1");
    assert_eq!(content["exit_code"], 0);
    assert_eq!(content["is_alt_screen_active"], true);
    assert!(content.get("command").is_none());
    assert!(serde_json::to_vec(&responses[0].content).unwrap().len() <= MAX_RESULT_BYTES);
}

#[test]
fn unrepresentable_control_fields_stop_the_request_instead_of_losing_command_id() {
    let mut request = ChatRequest::new(vec![
        ToolResponse::new(
            "call-1",
            json!({"status": "running", "command_id": "x".repeat(40_000), "output": "log"})
                .to_string(),
        )
        .into(),
    ]);

    assert!(apply(&mut request, Some(200_000)).is_err());
}

#[test]
fn trimming_preserves_tool_call_pairing_and_response_caller() {
    let call = ToolCall {
        call_id: "nested-call-1".to_owned(),
        fn_name: "run_shell_command".to_owned(),
        fn_arguments: json!({"command": "docker logs --since 2h localmind_affine_server"}),
        thought_signatures: Some(vec!["signature-1".to_owned()]),
    };
    let caller = json!({"type": "programmatic", "tool_id": "parent-call"});
    let mut request = ChatRequest::new(vec![
        ChatMessage::from(vec![call]),
        ChatMessage::from(
            ToolResponse::new("nested-call-1", "日志".repeat(40_000))
                .with_response_caller(Some(caller.clone()), Some("parent-call".to_owned())),
        ),
    ]);
    let original_call = serde_json::to_value(&request.messages[0]).unwrap();

    apply(&mut request, Some(200_000)).expect("嵌套调用结果应可裁剪");

    assert_eq!(request.messages.len(), 2);
    assert_eq!(
        serde_json::to_value(&request.messages[0]).unwrap(),
        original_call
    );
    let responses = tool_responses(&request);
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].call_id, "nested-call-1");
    assert_eq!(responses[0].caller.as_ref(), Some(&caller));
    assert_eq!(responses[0].caller_id.as_deref(), Some("parent-call"));
}

#[test]
fn trimming_outbound_copy_keeps_local_history_complete() {
    let output = "完整的本地日志\n".repeat(40_000);
    let local_history = ChatRequest::new(vec![shell_result("call-1", &output)]);
    let original = serde_json::to_value(&local_history).unwrap();
    let mut outbound = local_history.clone();

    apply(&mut outbound, Some(200_000)).expect("出站副本应可裁剪");

    assert_eq!(serde_json::to_value(&local_history).unwrap(), original);
    assert_ne!(serde_json::to_value(&outbound).unwrap(), original);
    let responses = tool_responses(&local_history);
    let content: Value = serde_json::from_str(&responses[0].content).unwrap();
    assert_eq!(content["output"], output);
    assert!(content.get(TRUNCATION_KEY).is_none());
}

#[test]
fn parallel_large_results_share_one_budget_and_favor_recent_output() {
    let output = "x".repeat(200_000);
    let mut request = ChatRequest::new(vec![
        shell_result("call-1", &output),
        shell_result("call-2", &output),
        shell_result("call-3", &output),
        shell_result("call-4", &output),
        shell_result("call-5", &output),
        shell_result("call-6", &output),
    ]);

    let report = apply(&mut request, Some(200_000)).expect("并行工具结果应共享总预算");

    let responses = tool_responses(&request);
    let sent_bytes: usize = responses
        .iter()
        .map(|response| serde_json::to_vec(&response.content).unwrap().len())
        .sum();
    assert_eq!(report.truncated_results, 6);
    assert_eq!(
        responses
            .iter()
            .map(|response| response.call_id.as_str())
            .collect::<Vec<_>>(),
        ["call-1", "call-2", "call-3", "call-4", "call-5", "call-6"]
    );
    assert!(sent_bytes <= MAX_TOTAL_RESULT_BYTES);
    assert!(responses.iter().all(|response| {
        serde_json::to_vec(&response.content).unwrap().len() <= MAX_RESULT_BYTES
            && serde_json::from_str::<Value>(&response.content).unwrap()["status"] == "completed"
    }));
    assert!(responses[5].content.len() > responses[0].content.len());
}

#[test]
fn unicode_and_escaped_logs_stay_valid_within_serialized_budget() {
    let output = format!(
        "起点🌍\n{}\n终点🚀",
        "中文🦀\0\u{1b}\t\r\n\\\"".repeat(40_000)
    );
    let mut request = ChatRequest::new(vec![shell_result("call-1", &output)]);

    apply(&mut request, Some(200_000)).expect("转义字符不能突破出站预算");

    let responses = tool_responses(&request);
    let content: Value =
        serde_json::from_str(&responses[0].content).expect("裁剪不得破坏 UTF-8 或 JSON");
    let kept_output = content["output"].as_str().unwrap();
    assert!(kept_output.starts_with("起点🌍\n"));
    assert!(kept_output.ends_with("\n终点🚀"));
    assert!(kept_output.contains("中文🦀\0\u{1b}\t\r\n\\\""));
    assert!(serde_json::to_vec(&responses[0].content).unwrap().len() <= MAX_RESULT_BYTES);
}

#[test]
fn oversized_plain_text_is_explicitly_wrapped_as_a_partial_preview() {
    let output = format!("开头\n{}\n结尾", "普通文本".repeat(40_000));
    let mut request = ChatRequest::new(vec![ToolResponse::new("call-1", output).into()]);

    apply(&mut request, None).expect("非 JSON 工具结果也应受预算保护");

    let responses = tool_responses(&request);
    let content: Value = serde_json::from_str(&responses[0].content).unwrap();
    assert!(content["preview"].as_str().unwrap().starts_with("开头\n"));
    assert!(content["preview"].as_str().unwrap().ends_with("\n结尾"));
    assert!(content[TRUNCATION_KEY]["note"].as_str().is_some());
    assert!(serde_json::to_vec(&responses[0].content).unwrap().len() <= MAX_RESULT_BYTES);
}

#[test]
fn repeated_bounded_history_keeps_its_fingerprint_but_changed_output_invalidates_it() {
    let history = ChatRequest::new(vec![shell_result("call-1", &"x".repeat(200_000))]);
    let mut first = history.clone();
    let first_report = apply(&mut first, Some(200_000)).unwrap();
    let mut repeated = history;
    repeated.messages.push(ChatMessage::user("继续"));
    let repeated_report = apply(&mut repeated, Some(200_000)).unwrap();
    let mut changed = ChatRequest::new(vec![shell_result("call-1", &"y".repeat(200_000))]);
    let changed_report = apply(&mut changed, Some(200_000)).unwrap();

    assert!(first_report.truncated_fingerprint.is_some());
    assert_eq!(
        first_report.truncated_fingerprint,
        repeated_report.truncated_fingerprint
    );
    assert_ne!(
        first_report.truncated_fingerprint,
        changed_report.truncated_fingerprint
    );
}

#[test]
fn short_results_remain_byte_for_byte_unchanged() {
    let mut request = ChatRequest::new(vec![
        ChatMessage::user("检查服务状态"),
        ToolResponse::new("call-1", "{ \"status\": \"ok\", \"output\": \"正常\" }").into(),
        ToolResponse::new("call-2", "").into(),
        ToolResponse::new("call-3", "普通文本\n").into(),
    ])
    .with_system("只读检查");
    let original = serde_json::to_value(&request).unwrap();

    let report = apply(&mut request, Some(200_000)).expect("短请求应原样通过");

    assert_eq!(report.truncated_results, 0);
    assert_eq!(serde_json::to_value(&request).unwrap(), original);
}

#[test]
fn unknown_and_zero_windows_still_enforce_hard_output_limits() {
    let output = "x".repeat(200_000);
    let mut unknown = ChatRequest::new(vec![shell_result("call-1", &output)]);
    let mut zero = unknown.clone();

    let unknown_report = apply(&mut unknown, None).expect("未知模型应使用兜底窗口");
    let zero_report = apply(&mut zero, Some(0)).expect("零窗口应使用兜底窗口");

    assert_eq!(unknown_report.truncated_results, 1);
    assert_eq!(zero_report.truncated_results, 1);
    assert_eq!(
        serde_json::to_value(&unknown).unwrap(),
        serde_json::to_value(&zero).unwrap()
    );
    assert!(
        serde_json::to_vec(&tool_responses(&unknown)[0].content)
            .unwrap()
            .len()
            <= MAX_RESULT_BYTES
    );
}

#[test]
fn small_model_window_accepts_short_conversations() {
    let mut request = ChatRequest::new(vec![
        ChatMessage::user("服务是否正常？"),
        shell_result("call-1", "正常"),
    ]);
    let original = serde_json::to_value(&request).unwrap();

    let report = apply(&mut request, Some(4_096)).expect("不能让输出预留吃满小模型窗口");

    assert_eq!(report.truncated_results, 0);
    assert_eq!(serde_json::to_value(&request).unwrap(), original);
}

#[test]
fn small_window_image_is_not_rejected_by_a_fixed_token_estimate() {
    let mut request = ChatRequest::new(vec![ChatMessage::user(MessageContent::from_parts(vec![
        ContentPart::Text("识别这张小图".to_owned()),
        ContentPart::Binary(Binary::from_base64("image/png", "AAAA", None)),
    ]))]);
    let original = serde_json::to_value(&request).unwrap();

    apply(&mut request, Some(8_192)).expect("图片估算不是实际 token，不能据此拒绝请求");

    assert_eq!(serde_json::to_value(&request).unwrap(), original);
}

#[test]
fn estimated_overflow_keeps_non_tool_text_and_still_bounds_logs() {
    let mut request = ChatRequest::new(vec![
        ChatMessage::user("hello ".repeat(10_000)),
        shell_result("call-1", &"日志".repeat(100_000)),
    ]);
    let original_user = serde_json::to_value(&request.messages[0]).unwrap();

    let report =
        apply(&mut request, Some(32_768)).expect("保守估算只能建议压缩，不能假装提供商已拒绝正文");

    assert_eq!(
        serde_json::to_value(&request.messages[0]).unwrap(),
        original_user
    );
    assert_eq!(report.truncated_results, 1);
    assert!(
        serde_json::to_vec(&tool_responses(&request)[0].content)
            .unwrap()
            .len()
            <= MAX_RESULT_BYTES
    );
}

#[test]
fn small_model_window_reduces_tool_output_below_the_default_cap() {
    let mut request = ChatRequest::new(vec![shell_result("call-1", &"x".repeat(200_000))]);

    let report = apply(&mut request, Some(4_096)).expect("小模型也应能通过裁剪恢复");

    assert_eq!(report.truncated_results, 1);
    assert!(serde_json::to_vec(&request).unwrap().len() <= 6_144);
    let responses = tool_responses(&request);
    let content: Value = serde_json::from_str(&responses[0].content).unwrap();
    assert_eq!(content["status"], "completed");
}

#[test]
fn oversized_non_tool_estimate_preserves_text_for_provider_validation() {
    let mut request = ChatRequest::from_user("x".repeat(1_000_000));
    let original = serde_json::to_value(&request).unwrap();

    let report = apply(&mut request, Some(200_000)).expect("不以估算冒充提供商的超限判定");

    assert_eq!(report.truncated_results, 0);
    assert_eq!(serde_json::to_value(&request).unwrap(), original);
}

#[test]
fn multimodal_attachment_remains_intact_when_tool_results_are_trimmed() {
    let attachment = ChatMessage::user(MessageContent::from_parts(vec![
        ContentPart::Text("结合截图分析日志".to_owned()),
        ContentPart::Binary(Binary::from_base64(
            "image/png",
            "AAAA".repeat(250_000),
            Some("截图.png".to_owned()),
        )),
    ]));
    let original_attachment = serde_json::to_value(&attachment).unwrap();
    let mut request = ChatRequest::new(vec![
        attachment,
        shell_result("call-1", &"x".repeat(200_000)),
    ]);

    let report = apply(&mut request, Some(200_000)).expect("附件不能按 base64 字节误判为文本超限");

    assert_eq!(report.truncated_results, 1);
    assert_eq!(
        serde_json::to_value(&request.messages[0]).unwrap(),
        original_attachment
    );
    assert!(
        serde_json::to_vec(&tool_responses(&request)[0].content)
            .unwrap()
            .len()
            <= MAX_RESULT_BYTES
    );
}

#[test]
fn old_and_new_large_results_are_bounded_without_touching_short_history() {
    let output = "x".repeat(200_000);
    let mut request = ChatRequest::new(vec![
        shell_result("old-call", &output),
        ChatMessage::assistant("服务已启动，继续检查错误日志。"),
        shell_result("short-call", "没有错误"),
        ChatMessage::user("继续"),
        shell_result("new-call", &output),
    ]);
    let original_short = serde_json::to_value(&request.messages[2]).unwrap();
    let original_assistant = serde_json::to_value(&request.messages[1]).unwrap();
    let original_user = serde_json::to_value(&request.messages[3]).unwrap();

    let report = apply(&mut request, Some(200_000)).expect("旧历史和新结果都应纳入同一预算");

    let responses = tool_responses(&request);
    assert_eq!(report.truncated_results, 2);
    assert_eq!(request.messages.len(), 5);
    assert_eq!(
        serde_json::to_value(&request.messages[2]).unwrap(),
        original_short
    );
    assert_eq!(
        serde_json::to_value(&request.messages[1]).unwrap(),
        original_assistant
    );
    assert_eq!(
        serde_json::to_value(&request.messages[3]).unwrap(),
        original_user
    );
    assert_eq!(responses[0].call_id, "old-call");
    assert_eq!(responses[2].call_id, "new-call");
    assert!(serde_json::to_vec(&responses[0].content).unwrap().len() <= MAX_RESULT_BYTES);
    assert!(serde_json::to_vec(&responses[2].content).unwrap().len() <= MAX_RESULT_BYTES);
}
