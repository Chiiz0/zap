//! 显式授权后手动运行的真实网关冒烟；只发送合成历史，不读取真实会话或项目。
//! 必须显式设置 INFINISHELL_BYOP_SMOKE_PROVIDER_ID、INFINISHELL_BYOP_SMOKE_MODEL_ID
//! 和 INFINISHELL_BYOP_SMOKE_API_KEY，提供商与模型必须已存在于配置中。

use std::env;
use std::path::PathBuf;
use std::time::Duration;

use field_mask::FieldMaskOperation;
use url::Url;
use warpui_extras::user_preferences::UserPreferences;
use warpui_extras::user_preferences::toml_backed::TomlBackedUserPreferences;

use super::*;
use crate::ai::agent::conversation::{AIConversation, AIConversationId};
use crate::ai::agent_providers::llm_id;
use crate::ai::byop_compaction::commit::commit_summarization;
use crate::settings::{AgentProviderModel, ReasoningEffortSetting};

const SYNTHETIC_TASK_ID: &str = "byop-live-synthetic-task";
const SYNTHETIC_LABEL: &str = "blue-kite-472";

struct LiveTarget {
    provider: AgentProvider,
    model: AgentProviderModel,
    api_key: String,
}

struct LiveReply {
    request_id: String,
    text: String,
    provider_output_usage_observed: bool,
}

fn load_live_target() -> Result<LiveTarget, &'static str> {
    let provider_id = env::var("INFINISHELL_BYOP_SMOKE_PROVIDER_ID")
        .ok()
        .filter(|id| !id.trim().is_empty())
        .ok_or("missing_smoke_provider_id")?;
    let model_id = env::var("INFINISHELL_BYOP_SMOKE_MODEL_ID")
        .ok()
        .filter(|id| !id.trim().is_empty())
        .ok_or("missing_smoke_model_id")?;
    let settings_path = env::var_os("INFINISHELL_BYOP_SMOKE_SETTINGS_PATH")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|path| path.join(".infinishell/settings.toml")))
        .ok_or("missing_settings_path")?;
    let (preferences, parse_error) = TomlBackedUserPreferences::new(settings_path);
    if parse_error.is_some() {
        return Err("invalid_settings");
    }
    // 不走 Setting::read_from_preferences，避免其 debug 日志打印额外请求头。
    let providers_json = preferences
        .read_value_with_hierarchy("providers", Some("agents.warp_agent"))
        .map_err(|_| "unreadable_provider_config")?
        .ok_or("missing_provider_config")?;
    let providers: Vec<AgentProvider> =
        serde_json::from_str(&providers_json).map_err(|_| "invalid_provider_config")?;
    let mut provider = providers
        .into_iter()
        .find(|provider| provider.id == provider_id)
        .ok_or("provider_not_configured")?;
    // 本测试依赖 Responses 的原始终止条目，在本地工具拦截器运行前拒绝工具调用。
    if provider.api_type != AgentProviderApiType::OpenAiResp {
        return Err("live_smoke_requires_responses_provider");
    }
    let model = provider
        .models
        .iter()
        .find(|model| model.id == model_id)
        .cloned()
        .ok_or("model_not_configured")?;
    let api_key = env::var("INFINISHELL_BYOP_SMOKE_API_KEY")
        .ok()
        .filter(|key| !key.is_empty())
        .ok_or("missing_smoke_api_key")?;
    let endpoint = Url::parse(&provider.base_url).map_err(|_| "invalid_endpoint")?;
    if !endpoint.username().is_empty() || endpoint.password().is_some() {
        return Err("endpoint_contains_credentials");
    }
    let host = endpoint.host_str().ok_or("missing_endpoint_host")?;
    let safe_endpoint = format!("{}://{host}", endpoint.scheme());
    eprintln!(
        "{}",
        json!({"byop_live_smoke": "target", "endpoint": safe_endpoint, "model": model.id})
    );
    // 仅调整本次冒烟的内存副本，禁止提供商自行执行程序化或多代理工具。
    provider.responses.background = false;
    provider.responses.programmatic_tool_calling = false;
    provider.responses.multi_agent_beta = false;
    Ok(LiveTarget {
        provider,
        model,
        api_key,
    })
}

fn synthetic_input(query: &str) -> AIAgentInput {
    AIAgentInput::UserQuery {
        query: query.to_owned(),
        context: Arc::from([]),
        static_query_type: None,
        referenced_attachments: HashMap::new(),
        user_query_mode: UserQueryMode::default(),
        running_command: None,
        intended_agent: None,
    }
}

fn require_non_tool_message(message: &api::Message) -> Result<(), &'static str> {
    if !matches!(
        message.message,
        Some(
            api::message::Message::UserQuery(_)
                | api::message::Message::AgentOutput(_)
                | api::message::Message::AgentReasoning(_)
        )
    ) {
        return Err("unexpected_model_tool_or_action");
    }
    if let Some(state) = decode_provider_response_state(&message.server_message_data) {
        // 状态事件早于客户端内置工具的分发；即使工具只出现在终止帧也在此中止。
        if state.response_items.iter().any(|item| {
            !matches!(
                item.get("type").and_then(Value::as_str),
                Some("message" | "reasoning" | "compaction")
            )
        }) {
            return Err("unexpected_provider_tool_item");
        }
    }
    Ok(())
}

fn apply_live_action(
    task: &mut api::Task,
    action: api::client_action::Action,
) -> Result<(), &'static str> {
    match action {
        api::client_action::Action::AddMessagesToTask(add) => {
            if add.task_id != task.id {
                return Err("unexpected_task");
            }
            for message in add.messages {
                require_non_tool_message(&message)?;
                if task
                    .messages
                    .iter()
                    .any(|existing| existing.id == message.id)
                {
                    return Err("duplicate_message");
                }
                task.messages.push(message);
            }
        }
        api::client_action::Action::UpdateTaskMessage(update) => {
            if update.task_id != task.id {
                return Err("unexpected_task");
            }
            let message = update.message.ok_or("missing_update_message")?;
            require_non_tool_message(&message)?;
            let existing = task
                .messages
                .iter_mut()
                .find(|existing| existing.id == message.id)
                .ok_or("update_without_original")?;
            *existing = FieldMaskOperation::update(
                &api::MESSAGE_DESCRIPTOR,
                existing,
                &message,
                update.mask.ok_or("missing_update_mask")?,
            )
            .apply()
            .map_err(|_| "invalid_update_mask")?;
        }
        api::client_action::Action::AppendToMessageContent(append) => {
            if append.task_id != task.id {
                return Err("unexpected_task");
            }
            let message = append.message.ok_or("missing_append_message")?;
            require_non_tool_message(&message)?;
            let existing = task
                .messages
                .iter_mut()
                .find(|existing| existing.id == message.id)
                .ok_or("append_without_original")?;
            *existing = FieldMaskOperation::append(
                &api::MESSAGE_DESCRIPTOR,
                existing,
                &message,
                append.mask.ok_or("missing_append_mask")?,
            )
            .apply()
            .map_err(|_| "invalid_append_mask")?;
        }
        // 真实冒烟只持久化消息，任何执行或额外客户端动作都直接失败。
        _ => return Err("unexpected_client_action"),
    }
    Ok(())
}

async fn live_request(
    target: &LiveTarget,
    params: &mut RequestParams,
    request_number: u8,
) -> Result<LiveReply, &'static str> {
    let (_cancel, cancellation_rx) = futures::channel::oneshot::channel();
    let input = ByopOutputInput {
        params: params.clone(),
        base_url: target.provider.base_url.clone(),
        api_key: target.api_key.clone(),
        model_id: target.model.id.clone(),
        api_type: target.provider.api_type,
        reasoning_effort: ReasoningEffortSetting::Low,
        extra_headers: target.provider.extra_headers.clone(),
        responses: target.provider.responses.clone(),
        task_id: SYNTHETIC_TASK_ID.to_owned(),
        target_task_id: SYNTHETIC_TASK_ID.to_owned(),
        needs_create_task: false,
        lrc_command_id: None,
        lrc_should_spawn_subagent: false,
        context_window: (target.model.context_window > 0).then_some(target.model.context_window),
        cancellation_rx,
        attachment_caps: attachment_caps::resolve_for_model(
            &target.provider.id,
            target.provider.api_type,
            &target.model,
        ),
    };
    let result = tokio::time::timeout(Duration::from_secs(120), async {
        let mut stream = generate_byop_output(input)
            .await
            .map_err(|_| "request_preparation_failed")?;
        let mut request_id = String::new();
        let mut finished = None;
        while let Some(event) = stream.next().await {
            let event = event.map_err(|_| "provider_stream_failed")?;
            match event.r#type {
                Some(api::response_event::Type::Init(init)) => request_id = init.request_id,
                Some(api::response_event::Type::ClientActions(actions)) => {
                    for action in actions.actions {
                        apply_live_action(
                            &mut params.tasks[0],
                            action.action.ok_or("missing_client_action")?,
                        )?;
                    }
                }
                Some(api::response_event::Type::Finished(value)) => {
                    if !matches!(
                        value.reason,
                        Some(api::response_event::stream_finished::Reason::Done(_))
                    ) {
                        return Err("provider_did_not_complete");
                    }
                    finished = Some(value);
                }
                _ => return Err("unexpected_response_event"),
            }
            let produced_bytes: usize = params.tasks[0]
                .messages
                .iter()
                .filter(|message| message.request_id == request_id)
                .filter_map(|message| match &message.message {
                    Some(api::message::Message::AgentOutput(output)) => Some(output.text.len()),
                    Some(api::message::Message::AgentReasoning(reasoning)) => {
                        Some(reasoning.reasoning.len())
                    }
                    _ => None,
                })
                .sum();
            if produced_bytes > 24 * 1024 {
                return Err("smoke_output_too_large");
            }
        }
        let finished = finished.ok_or("missing_finished_event")?;
        let text = params.tasks[0]
            .messages
            .iter()
            .filter(|message| message.request_id == request_id)
            .filter_map(|message| match &message.message {
                Some(api::message::Message::AgentOutput(output)) => Some(output.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");
        if request_id.is_empty() || text.trim().is_empty() {
            return Err("empty_completed_answer");
        }
        let input_tokens = finished
            .token_usage
            .iter()
            .map(|usage| usage.total_input)
            .max()
            .unwrap_or_default();
        let output_tokens = finished
            .token_usage
            .iter()
            .map(|usage| usage.output)
            .max()
            .unwrap_or_default();
        eprintln!(
            "{}",
            json!({
                "byop_live_smoke": "request",
                "request_number": request_number,
                "status": "completed",
                "input_tokens_provider_or_exact_preflight": input_tokens,
                "provider_output_tokens": output_tokens
            })
        );
        Ok(LiveReply {
            request_id,
            text,
            provider_output_usage_observed: output_tokens > 0,
        })
    })
    .await
    .map_err(|_| "request_timeout")?;
    if let Err(status) = result.as_ref() {
        eprintln!(
            "{}",
            json!({"byop_live_smoke": "request", "request_number": request_number, "status": status})
        );
    }
    result
}

#[tokio::test]
#[ignore = "需显式授权真实网关调用，并由运行者安全注入 INFINISHELL_BYOP_SMOKE_API_KEY"]
async fn live_byop_large_log_summary_and_resume() {
    let target = load_live_target().unwrap_or_else(|status| panic!("真实冒烟配置失败: {status}"));
    let synthetic_log = format!(
        "BEGIN_SMOKE\n{}\nEND_SMOKE",
        "synthetic diagnostic line without commands or private data\n".repeat(80_000)
    );
    let mut params = RequestParams::new_for_test(
        vec![synthetic_input(
            "Without calling any tool, inspect the supplied historical log and reply only LOG_OK if BEGIN_SMOKE and END_SMOKE are both present.",
        )],
        vec![api::Task {
            id: SYNTHETIC_TASK_ID.to_owned(),
            messages: vec![
                make_user_query_message(
                    SYNTHETIC_TASK_ID,
                    "synthetic-history",
                    "This is a synthetic data-only task. The release label is blue-kite-472. Preserve this exact label through any summary. Never execute commands or call tools.".to_owned(),
                    &[],
                ),
                make_tool_call_carrier_message(
                    SYNTHETIC_TASK_ID,
                    "synthetic-history",
                    "synthetic-log-call",
                    "run_shell_command",
                    r#"{"command":"cat synthetic.log"}"#,
                ),
                make_tool_call_result_message(
                    SYNTHETIC_TASK_ID,
                    "synthetic-history",
                    "synthetic-log-call".to_owned(),
                    json!({"status": "completed", "exit_code": 0, "output": synthetic_log})
                        .to_string(),
                ),
            ],
            ..Default::default()
        }],
    );
    params.model = llm_id::encode(&target.provider.id, &target.model.id);
    params.context_window_limit =
        (target.model.context_window > 0).then_some(target.model.context_window);
    let original_log_bytes = params.tasks[0].messages[2].server_message_data.len();
    assert!(original_log_bytes > 4_000_000);
    let (request, report) =
        build_byop_preflight_request(&params, &target.provider, &target.model.id)
            .unwrap_or_else(|_| panic!("合成日志请求构造失败"));
    assert_eq!(report.truncated_results, 1);
    let result = request
        .messages
        .iter()
        .flat_map(|message| message.content.tool_responses())
        .next()
        .expect("出站请求应保留工具结果");
    assert!(result.content.len() <= 32_768);
    assert!(result.content.contains("BEGIN_SMOKE") && result.content.contains("END_SMOKE"));

    let first = live_request(&target, &mut params, 1)
        .await
        .unwrap_or_else(|status| panic!("大日志真实请求失败: {status}"));
    assert!(first.text.trim() == "LOG_OK", "大日志头尾验证未通过");
    assert_eq!(
        params.tasks[0].messages[2].server_message_data.len(),
        original_log_bytes
    );

    params.input = vec![AIAgentInput::SummarizeConversation {
        prompt: Some("Keep the summary under 160 words. Preserve the exact synthetic release label and the no-tools constraint. Do not copy diagnostic lines.".to_owned()),
        overflow: false,
        context: Arc::from([]),
    }];
    let cfg = byop_compaction::CompactionConfig {
        tail_turns: 0,
        ..Default::default()
    };
    let mut plan = prepare_byop_compaction(&params, &target.provider, &target.model.id, &cfg)
        .expect("合成历史应能生成固定摘要计划");
    plan.max_output_tokens = plan.max_output_tokens.min(2_048);
    params.compaction_plan = Some(plan.clone());
    let (summary_request, _) =
        build_byop_preflight_request(&params, &target.provider, &target.model.id)
            .unwrap_or_else(|_| panic!("摘要请求构造失败"));
    assert!(summary_request.tools.is_none());
    let summary = live_request(&target, &mut params, 2)
        .await
        .unwrap_or_else(|status| panic!("真实摘要失败: {status}"));
    assert!(
        summary.text.contains(SYNTHETIC_LABEL),
        "真实摘要丢失合成约束"
    );
    let mut conversation =
        AIConversation::new_restored(AIConversationId::new(), params.tasks.clone(), None)
            .unwrap_or_else(|_| panic!("真实事件构成的合成历史恢复失败"));
    assert!(commit_summarization(
        &mut conversation,
        &plan,
        &summary.request_id,
        false
    ));
    assert_eq!(conversation.compaction_state.completed().len(), 1);
    params.compaction_state = Some(conversation.compaction_state.clone());
    params.compaction_plan = None;
    params.input = vec![synthetic_input(
        "Without calling any tool, return only the exact synthetic release label from the earlier task. Do not include quotes or explanation.",
    )];
    let (resumed_request, report) =
        build_byop_preflight_request(&params, &target.provider, &target.model.id)
            .unwrap_or_else(|_| panic!("摘要后续接请求构造失败"));
    assert_eq!(report.truncated_results, 0);
    assert!(
        resumed_request
            .messages
            .iter()
            .all(|message| message.content.tool_responses().is_empty())
    );
    let resumed = live_request(&target, &mut params, 3)
        .await
        .unwrap_or_else(|status| panic!("摘要后真实续接失败: {status}"));
    assert!(
        resumed.text.trim() == SYNTHETIC_LABEL,
        "摘要后合成约束未保留"
    );
    assert_eq!(
        params.tasks[0].messages[2].server_message_data.len(),
        original_log_bytes
    );
    let provider_usage_observed = first.provider_output_usage_observed
        && summary.provider_output_usage_observed
        && resumed.provider_output_usage_observed;
    eprintln!(
        "{}",
        json!({
            "byop_live_smoke": "complete",
            "generation_requests": 3,
            "status": "passed",
            "provider_usage_observed_on_all_requests": provider_usage_observed,
            "missing_usage_coverage": "已有 mock 回归；本冒烟不伪造网关缺 usage，输入统计可能含精确预检计数"
        })
    );
}
