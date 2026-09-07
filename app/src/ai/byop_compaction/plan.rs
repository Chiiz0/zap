//! 摘要请求发出前固定覆盖范围；生成和提交共用同一份计划，不能按完成时的历史重选。

use std::collections::HashSet;

use warp_multi_agent_api as api;

use super::algorithm::{MessageRef, select};
use super::config::CompactionConfig;
use super::message_view::{build_tool_name_lookup, project};
use super::overflow::ModelLimit;
use super::state::CompactionState;

#[derive(Debug, Clone)]
pub struct CompactionPlan {
    pub head_message_ids: Vec<String>,
    pub tail_start_id: Option<String>,
    /// 摘要插入到覆盖区开头，不能借用保留区内的真实用户消息。
    pub anchor_message_id: String,
    pub previous_summary: Option<String>,
    pub max_output_tokens: u32,
    source_messages: Vec<api::Message>,
    original_message_ids: HashSet<String>,
    original_task_ids: HashSet<String>,
    previous_summary_id: Option<String>,
}

impl CompactionPlan {
    /// 请求构造只能使用准备时完全相同的前缀，不允许悄悄换成新的覆盖区。
    pub fn head_end(&self, messages: &[&api::Message]) -> Option<usize> {
        let count = self.source_messages.len();
        (messages.len() >= count
            && messages[..count]
                .iter()
                .zip(&self.source_messages)
                .all(|(current, original)| *current == original))
        .then_some(count)
    }

    pub(super) fn is_current(&self, messages: &[&api::Message], state: &CompactionState) -> bool {
        if state.completed().last().map(|item| &item.assistant_msg_id)
            != self.previous_summary_id.as_ref()
            || state.previous_summary() != self.previous_summary.as_deref()
        {
            return false;
        }
        let ids: HashSet<&str> = self.head_message_ids.iter().map(String::as_str).collect();
        let selected: Vec<&api::Message> = messages
            .iter()
            .copied()
            .filter(|message| ids.contains(message.id.as_str()))
            .collect();
        if self.head_end(&selected) != Some(selected.len()) {
            return false;
        }
        let Some(last_head_position) = messages
            .iter()
            .rposition(|message| ids.contains(message.id.as_str()))
        else {
            return false;
        };
        // 在已选择的子任务区间内并发插入的新消息没有参与摘要，不能随旧区间一起隐藏。
        messages[..=last_head_position].iter().all(|message| {
            !self.original_task_ids.contains(&message.task_id)
                || self.original_message_ids.contains(&message.id)
        })
    }

    pub(super) fn is_new_message(&self, message_id: &str) -> bool {
        !self.original_message_ids.contains(message_id)
    }
}

/// `messages` 必须使用请求序列化的 DFS 顺序。`fits` 检查完整的摘要请求，包括提示词、
/// 前次摘要和附件；放不下时只缩短完整轮次，不能截掉单条消息内容后声称覆盖了原文。
pub fn prepare_plan(
    messages: &[&api::Message],
    state: &CompactionState,
    cfg: &CompactionConfig,
    model: ModelLimit,
    fits: impl Fn(&CompactionPlan) -> bool,
) -> Option<CompactionPlan> {
    let hidden = state.hidden_message_ids();
    let visible: Vec<&api::Message> = messages
        .iter()
        .copied()
        .filter(|message| !hidden.contains(&message.id))
        .collect();
    if visible.is_empty() {
        return None;
    }
    let tool_names = build_tool_name_lookup(visible.iter().copied());
    let views = project(&visible, state, &tool_names);
    let selected = select(&views, cfg, model, |slice| {
        slice.iter().map(MessageRef::estimate_size).sum()
    });
    let head_end = visible
        .get(selected.head_end)
        .and_then(|tail| messages.iter().position(|message| message.id == tail.id))
        .unwrap_or(messages.len());

    // 优先按原来的尾部策略向前找完整轮次；切点落在第一轮内部时，允许向后扩到
    // 最近的完整轮次，避免有足够摘要预算却因没有更早用户边界而拒绝压缩。
    // 扩展候选仍须通过工具闭合和实际请求预算检查，不直接丢弃保留区。
    let candidate_ends = (1..=head_end)
        .rev()
        .chain(head_end.saturating_add(1)..=messages.len());
    for end in candidate_ends {
        let at_user_boundary = messages.get(end).is_none_or(|message| {
            matches!(message.message, Some(api::message::Message::UserQuery(_)))
                && !hidden.contains(&message.id)
        });
        if !at_user_boundary || !has_closed_tool_groups(&messages[..end], &hidden) {
            continue;
        }
        if !messages[..end].iter().any(|message| {
            !hidden.contains(&message.id)
                && matches!(
                    message.message,
                    Some(
                        api::message::Message::UserQuery(_)
                            | api::message::Message::AgentOutput(_)
                            | api::message::Message::ToolCall(_)
                            | api::message::Message::ToolCallResult(_)
                    )
                )
        }) {
            continue;
        }
        let plan = CompactionPlan {
            head_message_ids: messages[..end]
                .iter()
                .map(|message| message.id.clone())
                .collect(),
            tail_start_id: messages.get(end).map(|message| message.id.clone()),
            anchor_message_id: messages[0].id.clone(),
            previous_summary: state.previous_summary().map(str::to_owned),
            max_output_tokens: u32::try_from(model.max_output).unwrap_or(u32::MAX),
            source_messages: messages[..end]
                .iter()
                .map(|message| (*message).clone())
                .collect(),
            original_message_ids: messages.iter().map(|message| message.id.clone()).collect(),
            original_task_ids: messages
                .iter()
                .map(|message| message.task_id.clone())
                .collect(),
            previous_summary_id: state
                .completed()
                .last()
                .map(|item| item.assistant_msg_id.clone()),
        };
        if fits(&plan) {
            return Some(plan);
        }
    }
    None
}

fn has_closed_tool_groups(messages: &[&api::Message], hidden: &HashSet<String>) -> bool {
    let mut pending = HashSet::new();
    for message in messages {
        if hidden.contains(&message.id) {
            continue;
        }
        if let Some(api::message::Message::ToolCall(call)) = &message.message {
            if !matches!(call.tool, Some(api::message::tool_call::Tool::Subagent(_))) {
                pending.insert((&message.task_id, &call.tool_call_id));
            }
        }
        if let Some(api::message::Message::ToolCallResult(result)) = &message.message {
            pending.remove(&(&message.task_id, &result.tool_call_id));
        }
    }
    pending.is_empty()
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;
