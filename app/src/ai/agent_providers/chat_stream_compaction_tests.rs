use super::*;
use crate::settings::AgentProviderModel;

fn provider(api_type: AgentProviderApiType, window: u32) -> AgentProvider {
    let mut provider = AgentProvider::new_empty();
    provider.api_type = api_type;
    provider.base_url = "http://localhost:1/v1".to_owned();
    let mut model = AgentProviderModel::from_id("test-model".to_owned());
    model.context_window = window;
    provider.models.push(model);
    provider
}

fn history_params(text_bytes: usize) -> RequestParams {
    RequestParams::new_for_test(
        vec![],
        vec![api::Task {
            id: "task-1".to_owned(),
            messages: (0..4)
                .flat_map(|turn| {
                    [
                        make_user_query_message(
                            "task-1",
                            &format!("req-{turn}"),
                            format!("task {turn}"),
                            &[],
                        ),
                        make_agent_output_message(
                            "task-1",
                            &format!("req-{turn}"),
                            "x".repeat(text_bytes),
                        ),
                    ]
                })
                .collect(),
            ..Default::default()
        }],
    )
}

#[test]
fn all_byop_protocols_preflight_without_provider_usage() {
    for api_type in [
        AgentProviderApiType::OpenAi,
        AgentProviderApiType::OpenAiResp,
        AgentProviderApiType::Anthropic,
        AgentProviderApiType::Gemini,
        AgentProviderApiType::Ollama,
    ] {
        let provider = provider(api_type, 200_000);
        assert!(!byop_request_needs_compaction(
            &history_params(20),
            &provider,
            "test-model"
        ));
        assert!(byop_request_needs_compaction(
            &history_params(90_000),
            &provider,
            "test-model"
        ));
    }
}

#[test]
fn compaction_uses_smaller_profile_window_and_reserves_summary_budget() {
    let mut provider = provider(AgentProviderApiType::OpenAi, 200_000);
    provider.models[0].max_output_tokens = 2_048;
    let mut params = history_params(10_000);
    assert!(!byop_request_needs_compaction(
        &params,
        &provider,
        "test-model"
    ));
    params.context_window_limit = Some(32_000);
    assert!(byop_request_needs_compaction(
        &params,
        &provider,
        "test-model"
    ));
    params.input = vec![AIAgentInput::SummarizeConversation {
        prompt: None,
        overflow: true,
        context: Arc::from([]),
    }];
    let original = params.tasks.clone();
    params.compaction_plan = prepare_byop_compaction(
        &params,
        &provider,
        "test-model",
        &byop_compaction::CompactionConfig::default(),
    );
    assert_eq!(
        params.compaction_plan.as_ref().unwrap().max_output_tokens,
        2_048
    );
    let (request, _) = build_byop_preflight_request(&params, &provider, "test-model").unwrap();
    assert!(request.tools.is_none());
    assert!(
        request_budget::estimated_input_tokens(&request)
            <= request_budget::input_token_budget(Some(32_000))
    );
    assert_eq!(params.tasks, original);
}

#[test]
fn summaries_require_a_snapshot_and_complete_non_tool_output() {
    let mut params = history_params(20);
    params.input = vec![AIAgentInput::SummarizeConversation {
        prompt: None,
        overflow: true,
        context: Arc::from([]),
    }];
    assert!(
        build_byop_preflight_request(
            &params,
            &provider(AgentProviderApiType::OpenAi, 200_000),
            "test-model"
        )
        .is_err()
    );
    assert!(!valid_summary_end(&StreamEnd::default(), ""));
    let authoritative = StreamEnd {
        captured_stop_reason: Some(StopReason::Completed("stop".to_owned())),
        captured_content: Some(MessageContent::from("complete summary")),
        ..Default::default()
    };
    assert!(!valid_summary_end(&authoritative, "partial"));
    assert!(!valid_summary_end(&authoritative, ""));
    assert!(valid_summary_end(&authoritative, "complete summary"));
    for reason in [
        "length",
        "max_tokens",
        "incomplete",
        "tool_calls",
        "content_filter",
        "cancelled",
    ] {
        assert!(!valid_summary_end(
            &StreamEnd {
                captured_stop_reason: Some(StopReason::from(reason.to_owned())),
                ..Default::default()
            },
            ""
        ));
    }
    assert!(valid_summary_end(
        &StreamEnd {
            captured_stop_reason: Some(StopReason::from("stop".to_owned())),
            ..Default::default()
        },
        ""
    ));
    assert!(!valid_summary_end(
        &StreamEnd {
            captured_content: Some(MessageContent::from_parts(vec![ContentPart::ToolCall(
                ToolCall {
                    call_id: "call-1".to_owned(),
                    fn_name: "run_shell_command".to_owned(),
                    fn_arguments: json!({}),
                    thought_signatures: None
                }
            )])),
            ..Default::default()
        },
        ""
    ));
}

#[test]
fn finished_usage_counts_cache_once_and_keeps_unknown_usage_unknown() {
    let usage = TokenUsage {
        model_id: "byop-model".to_owned(),
        total_input: 100,
        output: 10,
        input_cache_read: 80,
        input_cache_write: 15,
        cost_in_cents: 0.0,
    };
    let event = make_finished_done(usage.clone(), Some(200));
    let Some(api::response_event::Type::Finished(finished)) = event.r#type else {
        panic!("需要完成事件")
    };
    assert_eq!(finished.token_usage, [usage]);
    let metadata = finished.conversation_usage_metadata.unwrap();
    assert_eq!(metadata.context_window_usage, 0.55);
    assert_eq!(metadata.total_input_tokens, 100);
    let event = make_finished_done(TokenUsage::default(), Some(200));
    let Some(api::response_event::Type::Finished(finished)) = event.r#type else {
        panic!("需要完成事件")
    };
    assert!(finished.token_usage.is_empty());
    assert!(finished.conversation_usage_metadata.is_none());
}

async fn mock_stream(
    sse: String,
    mut params: RequestParams,
) -> Vec<Result<api::ResponseEvent, Arc<AIApiError>>> {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/chat/completions")
        .with_header("content-type", "text/event-stream")
        .with_body(sse)
        .create_async()
        .await;
    params.context_window_limit = Some(200_000);
    let (_cancel, cancellation_rx) = futures::channel::oneshot::channel();
    let stream = generate_byop_output(ByopOutputInput {
        params,
        base_url: format!("{}/v1", server.url()),
        api_key: "test-key".to_owned(),
        model_id: "test-model".to_owned(),
        api_type: AgentProviderApiType::OpenAi,
        reasoning_effort: crate::settings::ReasoningEffortSetting::Auto,
        extra_headers: vec![],
        responses: Default::default(),
        task_id: "task-1".to_owned(),
        target_task_id: "task-1".to_owned(),
        needs_create_task: false,
        lrc_command_id: None,
        lrc_should_spawn_subagent: false,
        context_window: Some(200_000),
        cancellation_rx,
        attachment_caps: Default::default(),
    })
    .await
    .unwrap();
    let events = stream.collect().await;
    mock.assert_async().await;
    events
}

#[tokio::test]
async fn provider_stream_delivers_usage_to_controller() {
    let sse = concat!(
        "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"done\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":100,\"completion_tokens\":10,\"total_tokens\":110,\"prompt_tokens_details\":{\"cached_tokens\":80}}}\n\n",
        "data: [DONE]\n\n"
    );
    let events = mock_stream(sse.to_owned(), history_params(20)).await;
    let finished = events
        .iter()
        .find_map(|event| match &event.as_ref().unwrap().r#type {
            Some(api::response_event::Type::Finished(finished)) => Some(finished),
            _ => None,
        })
        .unwrap();
    assert_eq!(finished.token_usage.len(), 1);
    assert_eq!(finished.token_usage[0].total_input, 100);
    assert_eq!(finished.token_usage[0].output, 10);
    assert_eq!(finished.token_usage[0].input_cache_read, 80);
}

#[tokio::test]
async fn summary_tool_call_is_rejected_before_any_client_action() {
    let mut params = history_params(20);
    params.input = vec![AIAgentInput::SummarizeConversation {
        prompt: None,
        overflow: true,
        context: Arc::from([]),
    }];
    params.compaction_plan = prepare_byop_compaction(
        &params,
        &provider(AgentProviderApiType::OpenAi, 200_000),
        "test-model",
        &byop_compaction::CompactionConfig::default(),
    );
    let sse = concat!(
        "data: {\"choices\":[{\"index\":0,\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call-1\",\"type\":\"function\",\"function\":{\"name\":\"run_shell_command\",\"arguments\":\"{}\"}}]},\"finish_reason\":null}]}\n\n",
        "data: [DONE]\n\n"
    );
    let events = mock_stream(sse.to_owned(), params).await;
    assert!(events.iter().any(Result::is_err));
    assert!(
        events
            .iter()
            .filter_map(|event| event.as_ref().ok())
            .all(|event| matches!(event.r#type, Some(api::response_event::Type::Init(_))))
    );
}

#[tokio::test]
async fn summary_requires_explicit_success_before_finished_event() {
    for stop in [Some("stop"), None] {
        let mut params = history_params(20);
        params.input = vec![AIAgentInput::SummarizeConversation {
            prompt: None,
            overflow: true,
            context: Arc::from([]),
        }];
        params.compaction_plan = prepare_byop_compaction(
            &params,
            &provider(AgentProviderApiType::OpenAi, 200_000),
            "test-model",
            &byop_compaction::CompactionConfig::default(),
        );
        let sse = format!(
            "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
            json!({"choices": [{"index": 0, "delta": {"content": "concise summary"}, "finish_reason": null}]}),
            json!({"choices": [{"index": 0, "delta": {}, "finish_reason": stop}]})
        );
        let events = mock_stream(sse, params).await;
        let finished = events.iter().any(|event| {
            event.as_ref().is_ok_and(|event| {
                matches!(event.r#type, Some(api::response_event::Type::Finished(_)))
            })
        });
        assert_eq!(finished, stop.is_some());
        assert_eq!(events.iter().any(Result::is_err), stop.is_none());
    }
}

#[test]
fn responses_chain_changes_when_local_summary_or_pruning_changes() {
    let mut params = history_params(20);
    params.compaction_state = Some(Default::default());
    assert!(local_compaction_fingerprint(&params).is_none());
    let first_id = params.tasks[0].messages[0].id.clone();
    params
        .compaction_state
        .as_mut()
        .unwrap()
        .mark_tool_compacted(first_id.clone(), 1);
    let pruned = local_compaction_fingerprint(&params);
    assert!(pruned.is_some());
    params
        .compaction_state
        .as_mut()
        .unwrap()
        .mark_tool_compacted(first_id.clone(), 2);
    assert_eq!(local_compaction_fingerprint(&params), pruned);
    params.compaction_state.as_mut().unwrap().push_completed(
        byop_compaction::state::CompletedCompaction {
            user_msg_id: first_id.clone(),
            assistant_msg_id: "summary-1".to_owned(),
            summary_message_ids: vec!["summary-1".to_owned()],
            head_message_ids: vec![first_id],
            tail_start_id: None,
            summary_text: Some("summary".to_owned()),
            auto: true,
            overflow: true,
        },
    );
    assert_ne!(local_compaction_fingerprint(&params), pruned);
}
