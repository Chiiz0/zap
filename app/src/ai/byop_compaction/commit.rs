//! 把刚刚完成的 SummarizeConversation 流的产出写回 conversation.compaction_state —
//! 对齐 opencode `compaction.ts processCompaction` 末尾的状态变更 + bus.publish(Compacted)。
//!
//! 本模块独立于 controller,作为可单元测试的 helper(虽然真实调用站点在 controller.rs)。

use std::collections::HashSet;

use warp_multi_agent_api as api;

use super::algorithm::prune_decisions;
use super::config::CompactionConfig;
use super::message_view::{build_tool_name_lookup, project};
use super::plan::CompactionPlan;
use super::state::CompletedCompaction;
use crate::ai::agent::conversation::AIConversation;

/// 只提交这次成功请求产生的非空摘要，覆盖范围完全由发送前的计划决定。
/// 空响应、旧回答、源历史变化和重复完成事件都不得改变压缩状态。
pub fn commit_summarization(
    conversation: &mut AIConversation,
    plan: &CompactionPlan,
    request_id: &str,
    overflow: bool,
) -> bool {
    let mut all_msgs = conversation.all_linearized_messages();
    // 与请求的 DFS 线性化使用相同的用户消息去重规则。
    let mut seen_user_queries = HashSet::new();
    all_msgs.retain(|message| {
        if let Some(api::message::Message::UserQuery(query)) = &message.message {
            message.request_id.is_empty()
                || seen_user_queries.insert((&message.request_id, &query.query))
        } else {
            true
        }
    });
    if request_id.is_empty() || !plan.is_current(&all_msgs, &conversation.compaction_state) {
        return false;
    }
    let produced: Vec<&api::Message> = all_msgs
        .iter()
        .copied()
        .filter(|message| message.request_id == request_id && plan.is_new_message(&message.id))
        .collect();
    if produced.iter().any(|message| {
        matches!(
            message.message,
            Some(api::message::Message::ToolCall(_) | api::message::Message::ToolCallResult(_))
        )
    }) {
        return false;
    }
    let outputs: Vec<(&str, &str)> = produced
        .iter()
        .filter_map(|message| {
            if let Some(api::message::Message::AgentOutput(output)) = &message.message {
                (!output.text.trim().is_empty())
                    .then_some((message.id.as_str(), output.text.as_str()))
            } else {
                None
            }
        })
        .collect();
    let Some((assistant_id, _)) = outputs.last() else {
        return false;
    };
    let summary_text = outputs
        .iter()
        .map(|(_, text)| *text)
        .collect::<Vec<_>>()
        .join("\n\n");
    let completed = CompletedCompaction {
        user_msg_id: plan.anchor_message_id.clone(),
        assistant_msg_id: (*assistant_id).to_owned(),
        summary_message_ids: produced
            .iter()
            .filter(|message| {
                matches!(
                    message.message,
                    Some(
                        api::message::Message::AgentOutput(_)
                            | api::message::Message::AgentReasoning(_)
                    )
                )
            })
            .map(|message| message.id.clone())
            .collect(),
        head_message_ids: plan.head_message_ids.clone(),
        tail_start_id: plan.tail_start_id.clone(),
        summary_text: Some(summary_text),
        auto: overflow,
        overflow,
    };
    conversation.compaction_state.push_completed(completed);
    true
}

/// 在每次 LLM 请求前自动跑 prune — 1:1 对齐 opencode `compaction.ts:297-341`。
///
/// 计算决策(哪些 ToolCallResult 的 output 应被替换为占位)然后写入
/// `conversation.compaction_state.markers.tool_output_compacted_at`。
/// 实际替换发生在 `chat_stream::build_chat_request` 投影时(读 marker)。
///
/// `cfg.prune == false` 时 no-op。
pub fn prune_now(conversation: &mut AIConversation, cfg: &CompactionConfig) -> usize {
    if !cfg.prune {
        return 0;
    }
    let all_msgs: Vec<&api::Message> = conversation.all_linearized_messages();
    if all_msgs.is_empty() {
        return 0;
    }
    let tool_names = build_tool_name_lookup(all_msgs.iter().copied());
    let state_snapshot = conversation.compaction_state.clone();
    let views = project(&all_msgs, &state_snapshot, &tool_names);
    // 用 trait 引用避免泛型推导歧义
    let views_ref: &[_] = &views;
    let decisions = prune_decisions::<super::message_view::WarpMessageView<'_>>(views_ref);
    if decisions.is_empty() {
        return 0;
    }
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let count = decisions.len();
    for (msg_id, _call_id) in decisions {
        // msg_id 是 ToolCallResult 的 message id;mark_tool_compacted 会在 marker 上写时间戳
        conversation
            .compaction_state
            .mark_tool_compacted(msg_id, now_ms);
    }
    log::info!("[byop-compaction] pruned {count} tool output(s)");
    count
}
