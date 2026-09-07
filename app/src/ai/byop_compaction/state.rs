//! 压缩 sidecar 状态 — 挂在 `AIConversation` 上,与 warp `api::Message` 协议解耦。
//!
//! 因为 warp 的 `api::Message` 来自外部 protobuf 依赖 (`warp_multi_agent_api`),
//! 无法新增字段标记 `is_summary` / `compacted` 等;本 sidecar 用 message_id 索引
//! 把这些"压缩元数据"挂在 conversation 这一侧。
//!
//! 序列化版本号 [`CompactionState::VERSION`] 在 schema 演进时手动 bump,
//! 反序列化失败的旧 conversation 退化为 `Default`(等价"从未被压缩")。

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::ai::agent::AIAgentExchangeId;

#[path = "recovery.rs"]
mod recovery;
pub use recovery::{CompactionPersistenceError, PendingCompactionRecovery};

/// 触发压缩的来源。`Auto` 仅由 token-overflow 自动触发,`Manual` 是 /compact /compact-and。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompactionTrigger {
    Manual,
    Auto,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMarker {
    /// 这条 assistant message 是一份摘要,内容用于在请求拼装时替换前面的历史。
    #[serde(default)]
    pub is_summary: bool,
    /// 这条 user message 是一次 compaction 触发占位(opencode `parts.some(p => p.type === "compaction")`)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compaction_trigger: Option<CompactionTrigger>,
    /// 这条 ToolCallResult 的 output 已被 prune,投影时替换为占位符。Unix epoch ms。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_output_compacted_at: Option<u64>,
    /// 自动续跑时合成的 user "Continue..." synthetic message 标记
    /// (对齐 opencode `metadata.compaction_continue`)。
    #[serde(default)]
    pub synthetic_continue: bool,
}

/// 一个已完成的压缩区间(对齐 opencode `completedCompactions()` 返回项)。
///
/// `user_msg_id` 是触发摘要的 user message(带 compaction_trigger marker),
/// `assistant_msg_id` 是合成的摘要 AgentOutput message。两者在 [`CompactionState::hidden_message_ids`]
/// 中视为已被覆盖,投影时跳过 — 但摘要文本本身会被取出代填到 head 区。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedCompaction {
    pub user_msg_id: String,
    pub assistant_msg_id: String,
    /// 本次摘要流产生的正文和思考载体，全部隐藏，避免原始 Responses 状态再次回放。
    #[serde(default)]
    pub summary_message_ids: Vec<String>,
    /// 本次摘要覆盖的 head 区 message ids,投影普通请求时全部隐藏。
    #[serde(default)]
    pub head_message_ids: Vec<String>,
    /// tail 起点 message id,用于 split 验证 / debug。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tail_start_id: Option<String>,
    /// 摘要内容(从 assistant message 直接取也可,但缓存到 state 方便 build_prompt 拿 previous_summary)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_text: Option<String>,
    pub auto: bool,
    pub overflow: bool,
}

/// 与 `AIConversation` 一同持久化的 sidecar 表。
///
/// 默认值 = 空表 = 未压缩状态,完全无侵入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionState {
    /// schema 版本,演进时 bump。
    #[serde(default = "CompactionState::current_version")]
    pub version: u32,
    #[serde(default)]
    markers: HashMap<String, MessageMarker>,
    #[serde(default)]
    completed: Vec<CompletedCompaction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pending_recovery: Option<PendingCompactionRecovery>,
    /// 已被后续请求接管的待发输入，仅保留停止行，不作为“继续”的恢复目标。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    retained_recoveries: Vec<PendingCompactionRecovery>,
}

impl Default for CompactionState {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            markers: HashMap::new(),
            completed: Vec::new(),
            pending_recovery: None,
            retained_recoveries: Vec::new(),
        }
    }
}

impl CompactionState {
    pub const VERSION: u32 = 4;
    fn current_version() -> u32 {
        Self::VERSION
    }

    pub fn marker(&self, msg_id: &str) -> Option<&MessageMarker> {
        self.markers.get(msg_id)
    }

    /// 写一个 marker(merge 到已有 marker 上,而不是覆盖整个 marker)。
    pub fn upsert_marker(&mut self, msg_id: impl Into<String>, f: impl FnOnce(&mut MessageMarker)) {
        let entry = self.markers.entry(msg_id.into()).or_default();
        f(entry);
    }

    /// 标记一条 ToolCallResult 的 output 已 prune。
    pub fn mark_tool_compacted(&mut self, msg_id: impl Into<String>, now_ms: u64) {
        self.upsert_marker(msg_id, |m| m.tool_output_compacted_at = Some(now_ms));
    }

    /// 推一次完成的压缩。
    pub fn push_completed(&mut self, c: CompletedCompaction) {
        // 同步把 user 与 assistant 各自打上 marker(便于投影时单独识别)。
        self.upsert_marker(c.user_msg_id.clone(), |m| {
            m.compaction_trigger = Some(if c.auto {
                CompactionTrigger::Auto
            } else {
                CompactionTrigger::Manual
            });
        });
        self.upsert_marker(c.assistant_msg_id.clone(), |m| m.is_summary = true);
        for message_id in &c.summary_message_ids {
            self.upsert_marker(message_id.clone(), |m| m.is_summary = true);
        }
        self.completed.push(c);
    }

    /// 标记一条 synthetic "Continue..." user message(auto+overflow 路径合成)。
    pub fn mark_synthetic_continue(&mut self, msg_id: impl Into<String>) {
        self.upsert_marker(msg_id, |m| m.synthetic_continue = true);
    }

    /// 取最后一次完成的压缩(用于 [`super::prompt::build_prompt`] 的增量摘要锚点)。
    pub fn previous_summary(&self) -> Option<&str> {
        self.completed
            .last()
            .and_then(|c| c.summary_text.as_deref())
    }

    pub fn completed(&self) -> &[CompletedCompaction] {
        &self.completed
    }

    pub fn pending_recovery(&self) -> Option<&PendingCompactionRecovery> {
        self.pending_recovery.as_ref()
    }

    pub fn set_pending_recovery(&mut self, recovery: PendingCompactionRecovery) {
        let recovery_id = recovery.id;
        if let Some(previous) = self.pending_recovery.replace(recovery)
            && previous.id != recovery_id
        {
            // 旧输入尚未进入正式消息时，下一次摘要不能覆盖其唯一持久化副本。
            self.retained_recoveries
                .retain(|retained| retained.id != previous.id);
            self.retained_recoveries.push(previous);
        }
        self.retained_recoveries
            .retain(|retained| retained.id != recovery_id);
    }

    pub fn retained_recoveries(&self) -> &[PendingCompactionRecovery] {
        &self.retained_recoveries
    }

    pub fn clear_retained_recovery(&mut self, id: AIAgentExchangeId) {
        self.retained_recoveries
            .retain(|retained| retained.id != id);
    }

    /// 只关联当前恢复记录，迟到的旧流不能改变新问题的继续目标。
    pub fn set_recovery_owner(
        &mut self,
        id: AIAgentExchangeId,
        exchange_id: AIAgentExchangeId,
    ) -> bool {
        if let Some(pending) = self
            .pending_recovery
            .as_mut()
            .filter(|pending| pending.id == id)
        {
            pending.owner_exchange_id = Some(exchange_id);
            true
        } else {
            false
        }
    }

    pub fn clear_pending_recovery(&mut self, id: AIAgentExchangeId) -> bool {
        if self
            .pending_recovery
            .as_ref()
            .is_some_and(|pending| pending.id == id)
        {
            self.pending_recovery = None;
            true
        } else {
            false
        }
    }

    pub fn mark_recovery_resumed(&mut self, id: AIAgentExchangeId, request_id: String) -> bool {
        if let Some(pending) = self
            .pending_recovery
            .as_mut()
            .filter(|pending| pending.id == id)
        {
            pending.resumed_request_id = Some(request_id);
            true
        } else {
            false
        }
    }

    /// 所有应在拼请求时跳过的 message id(对齐 opencode `hidden`):
    /// 已完成压缩的每个区间的 head_message_ids + user_msg_id + assistant_msg_id。
    ///
    /// 注:这只是"原本要从历史里隐去的 message id 集",**不**包含摘要本身 —
    /// 摘要文本由请求投影在 compaction trigger user_msg_id 位置插入合成消息覆盖。
    pub fn hidden_message_ids(&self) -> HashSet<String> {
        let mut out = HashSet::new();
        for c in &self.completed {
            out.extend(c.head_message_ids.iter().cloned());
            out.insert(c.user_msg_id.clone());
            out.insert(c.assistant_msg_id.clone());
            out.extend(c.summary_message_ids.iter().cloned());
        }
        out
    }

    /// 调试 / 测试入口:看一条 marker 是否存在。
    #[cfg(test)]
    pub(crate) fn marker_count(&self) -> usize {
        self.markers.len()
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;

    fn cc(uid: &str, aid: &str, auto: bool) -> CompletedCompaction {
        CompletedCompaction {
            user_msg_id: uid.to_string(),
            assistant_msg_id: aid.to_string(),
            summary_message_ids: vec![aid.to_string()],
            head_message_ids: Vec::new(),
            tail_start_id: None,
            summary_text: Some(format!("summary-{aid}")),
            auto,
            overflow: false,
        }
    }

    #[test]
    fn push_completed_marks_both_messages() {
        let mut s = CompactionState::default();
        s.push_completed(cc("u1", "a1", true));
        assert!(s.marker("u1").unwrap().compaction_trigger == Some(CompactionTrigger::Auto));
        assert!(s.marker("a1").unwrap().is_summary);
    }

    #[test]
    fn previous_summary_returns_last() {
        let mut s = CompactionState::default();
        s.push_completed(cc("u1", "a1", false));
        s.push_completed(cc("u2", "a2", false));
        assert_eq!(s.previous_summary(), Some("summary-a2"));
    }

    #[test]
    fn hidden_message_ids_covers_all_completed() {
        let mut s = CompactionState::default();
        s.push_completed(cc("u1", "a1", false));
        s.push_completed(cc("u2", "a2", false));
        let h = s.hidden_message_ids();
        assert!(h.contains("u1"));
        assert!(h.contains("a1"));
        assert!(h.contains("u2"));
        assert!(h.contains("a2"));
        assert_eq!(h.len(), 4);
    }

    #[test]
    fn hidden_message_ids_includes_head_message_ids() {
        let mut s = CompactionState::default();
        let mut c = cc("u1", "a1", false);
        c.head_message_ids = vec!["h1".to_string(), "h2".to_string(), "u1".to_string()];
        s.push_completed(c);
        let h = s.hidden_message_ids();
        assert!(h.contains("h1"));
        assert!(h.contains("h2"));
        assert!(h.contains("u1"));
        assert!(h.contains("a1"));
        assert_eq!(h.len(), 4);
    }

    #[test]
    fn v1_completed_compaction_deserializes_to_empty_head_message_ids() {
        let json = r#"{
            "user_msg_id":"u1",
            "assistant_msg_id":"a1",
            "tail_start_id":null,
            "summary_text":"summary",
            "auto":false,
            "overflow":false
        }"#;
        let c: CompletedCompaction = serde_json::from_str(json).unwrap();
        assert!(c.head_message_ids.is_empty());
    }

    #[test]
    fn upsert_marker_merges() {
        let mut s = CompactionState::default();
        s.upsert_marker("m1", |m| m.is_summary = true);
        s.upsert_marker("m1", |m| m.synthetic_continue = true);
        let m = s.marker("m1").unwrap();
        assert!(m.is_summary);
        assert!(m.synthetic_continue);
        assert_eq!(s.marker_count(), 1);
    }

    #[test]
    fn default_serializable_roundtrip() {
        let s = CompactionState::default();
        let j = serde_json::to_string(&s).unwrap();
        let back: CompactionState = serde_json::from_str(&j).unwrap();
        assert_eq!(back.version, CompactionState::VERSION);
        assert!(back.completed.is_empty());
    }
}
