use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Local, TimeZone as _};
use warp_multi_agent_api as api;

use super::{AIConversation, AIConversationId, ConversationStatus};
use crate::ai::agent::task::TaskId;
use crate::ai::agent::{AIAgentAttachment, AIAgentContext, AIAgentInput, UserQueryMode};
use crate::ai::blocklist::RequestInput;
use crate::ai::byop_compaction::state::{
    CompactionState, CompletedCompaction, PendingCompactionRecovery,
};
use crate::ai::llms::LLMId;
use crate::persistence::ModelEvent;
use crate::persistence::model::AgentConversationData;
use crate::test_util::ai_agent_tasks::{create_api_task, create_message};

fn pending_query() -> PendingCompactionRecovery {
    let input = AIAgentInput::UserQuery {
        query: "待发问题".into(),
        context: Arc::from([AIAgentContext::SelectedText("原始日志片段".into())]),
        referenced_attachments: HashMap::from([(
            "file".into(),
            AIAgentAttachment::PlainText("原始附件".into()),
        )]),
        user_query_mode: UserQueryMode::Plan,
        static_query_type: None,
        running_command: None,
        intended_agent: None,
    };
    let request = RequestInput {
        conversation_id: AIConversationId::new(),
        input_messages: HashMap::from([(TaskId::new("root".into()), vec![input])]),
        working_directory: Some("/workspace".into()),
        model_id: LLMId::from("byop:original"),
        coding_model_id: LLMId::from("coding"),
        cli_agent_model_id: LLMId::from("cli"),
        computer_use_model_id: LLMId::from("computer"),
        shared_session_response_initiator: None,
        request_start_ts: Local.timestamp_opt(1_700_000_000, 0).unwrap(),
        supported_tools_override: None,
    };
    let mut pending = PendingCompactionRecovery::from_request(&request);
    pending.anchor_message_id = Some("old-output".into());
    pending.summary_request_id = Some("summary-request".into());
    pending
}

fn user_message(id: &str, request_id: &str, query: &str) -> api::Message {
    let mut message = create_message(id, "root");
    message.request_id = request_id.into();
    message.message = Some(api::message::Message::UserQuery(api::message::UserQuery {
        query: query.into(),
        ..Default::default()
    }));
    message
}

fn output_message(id: &str, request_id: &str) -> api::Message {
    let mut message = create_message(id, "root");
    message.request_id = request_id.into();
    message
}

fn conversation_data(state: &CompactionState) -> AgentConversationData {
    serde_json::from_value(serde_json::json!({
        "server_conversation_token": null,
        "compaction_state_json": serde_json::to_string(state).unwrap(),
    }))
    .unwrap()
}

fn restored(messages: Vec<api::Message>, state: &CompactionState) -> AIConversation {
    AIConversation::new_restored(
        AIConversationId::new(),
        vec![create_api_task("root", messages)],
        Some(conversation_data(state)),
    )
    .unwrap()
}

#[test]
fn interrupted_first_summary_restores_original_input_after_partial_summary() {
    let pending = pending_query();
    let pending_id = pending.id;
    let original_input = pending.queries[0].to_input();
    let mut state = CompactionState::default();
    state.set_pending_recovery(pending);
    let messages = vec![
        user_message("old-query", "old-request", "原会话问题"),
        output_message("old-output", "old-request"),
        output_message("partial-summary", "summary-request"),
    ];
    let conversation = restored(messages.clone(), &state);

    assert_eq!(conversation.status(), &ConversationStatus::Cancelled);
    assert_eq!(
        conversation.root_task_exchanges().last().unwrap().id,
        pending_id
    );
    assert_eq!(
        conversation.exchange_with_id(pending_id).unwrap().input,
        vec![original_input]
    );
    assert_eq!(
        conversation.all_linearized_messages(),
        messages.iter().collect::<Vec<_>>()
    );
    assert!(conversation.added_exchanges_by_response.is_empty());
    assert!(
        conversation
            .compaction_state
            .pending_recovery()
            .unwrap()
            .is_resume_target(conversation.latest_visible_exchange().unwrap().id)
    );

    let ModelEvent::UpdateMultiAgentConversation {
        conversation_data, ..
    } = conversation.updated_conversation_state_event()
    else {
        panic!("应产生完整会话快照");
    };
    assert!(
        conversation_data.compaction_state_json.is_some(),
        "首次摘要尚未完成也必须保存待发输入"
    );
}

#[test]
fn committed_summary_and_pending_attachments_survive_repeated_reload() {
    let pending = pending_query();
    let pending_id = pending.id;
    let mut state = CompactionState::default();
    state.set_pending_recovery(pending);
    state.push_completed(CompletedCompaction {
        user_msg_id: "old-query".into(),
        assistant_msg_id: "summary-output".into(),
        summary_message_ids: vec!["summary-output".into()],
        head_message_ids: vec!["old-query".into(), "old-output".into()],
        tail_start_id: None,
        summary_text: Some("已成功的摘要".into()),
        auto: true,
        overflow: true,
    });
    let original_messages = vec![
        user_message("old-query", "old-request", "原会话问题"),
        output_message("old-output", "old-request"),
        output_message("summary-output", "summary-request"),
    ];
    let first = restored(original_messages.clone(), &state);
    let ModelEvent::UpdateMultiAgentConversation {
        updated_tasks,
        conversation_data,
        ..
    } = first.updated_conversation_state_event()
    else {
        panic!("应产生完整会话快照");
    };
    let second =
        AIConversation::new_restored(first.id(), updated_tasks, Some(conversation_data)).unwrap();

    assert_eq!(
        second.compaction_state.previous_summary(),
        Some("已成功的摘要")
    );
    assert_eq!(
        second
            .all_exchanges()
            .into_iter()
            .filter(|exchange| exchange.id == pending_id)
            .count(),
        1
    );
    assert_eq!(
        second.exchange_with_id(pending_id).unwrap().input,
        first.exchange_with_id(pending_id).unwrap().input
    );
    assert_eq!(
        second.all_linearized_messages(),
        original_messages.iter().collect::<Vec<_>>()
    );
    assert_eq!(second.status(), &ConversationStatus::Cancelled);
}

#[test]
fn old_pending_input_restores_before_newer_user_exchange_without_changing_its_status() {
    let pending = pending_query();
    let pending_id = pending.id;
    let mut state = CompactionState::default();
    state.set_pending_recovery(pending);
    let conversation = restored(
        vec![
            user_message("old-query", "old-request", "原会话问题"),
            output_message("old-output", "old-request"),
            output_message("partial-summary", "summary-request"),
            user_message("new-query", "new-request", "后来的新问题"),
            output_message("new-output", "new-request"),
        ],
        &state,
    );

    assert_eq!(conversation.status(), &ConversationStatus::Success);
    assert_eq!(
        conversation.latest_user_query().as_deref(),
        Some("后来的新问题")
    );
    assert_eq!(
        conversation
            .root_task_exchanges()
            .flat_map(|exchange| exchange.input.iter())
            .filter_map(AIAgentInput::display_query)
            .collect::<Vec<_>>(),
        vec!["原会话问题", "/plan 待发问题", "后来的新问题"],
    );
    assert_eq!(
        conversation.root_task_exchanges().nth(2).unwrap().id,
        pending_id
    );
    assert!(
        !conversation
            .compaction_state
            .pending_recovery()
            .unwrap()
            .is_resume_target(conversation.latest_visible_exchange().unwrap().id)
    );
}

#[test]
fn newer_identical_query_does_not_resume_the_older_pending_input() {
    let pending = pending_query();
    let pending_id = pending.id;
    let mut state = CompactionState::default();
    state.set_pending_recovery(pending);
    let conversation = restored(
        vec![
            user_message("old-query", "old-request", "原会话问题"),
            output_message("old-output", "old-request"),
            output_message("partial-summary", "summary-request"),
            user_message("new-query", "new-request", "待发问题"),
            output_message("new-output", "new-request"),
        ],
        &state,
    );

    let pending = conversation.compaction_state.pending_recovery().unwrap();
    assert!(!pending.is_resume_target(conversation.latest_visible_exchange().unwrap().id));
    assert!(conversation.exchange_with_id(pending_id).is_some());
    assert_eq!(conversation.status(), &ConversationStatus::Success);
    assert_eq!(
        conversation.latest_user_query().as_deref(),
        Some("待发问题")
    );
}

#[test]
fn persisted_resumed_request_does_not_restore_or_resend_stale_pending_input() {
    let mut pending = pending_query();
    let pending_id = pending.id;
    pending.resumed_request_id = Some("resumed-request".into());
    let mut state = CompactionState::default();
    state.set_pending_recovery(pending);
    let conversation = restored(
        vec![
            user_message("old-query", "old-request", "原会话问题"),
            output_message("old-output", "old-request"),
            output_message("summary-output", "summary-request"),
            user_message("resumed-query", "resumed-request", "待发问题"),
        ],
        &state,
    );

    assert!(conversation.compaction_state.pending_recovery().is_none());
    assert!(conversation.exchange_with_id(pending_id).is_none());
    assert_eq!(conversation.root_task_exchanges().count(), 3);
}

#[test]
fn earlier_identical_query_does_not_consume_unsent_recovery() {
    let mut pending = pending_query();
    let pending_id = pending.id;
    pending.resumed_request_id = Some("not-yet-persisted".into());
    let mut state = CompactionState::default();
    state.set_pending_recovery(pending);
    let conversation = restored(
        vec![
            user_message("old-query", "old-request", "待发问题"),
            output_message("old-output", "old-request"),
            output_message("summary-output", "summary-request"),
        ],
        &state,
    );

    assert!(conversation.compaction_state.pending_recovery().is_some());
    assert!(conversation.exchange_with_id(pending_id).is_some());
    assert_eq!(conversation.status(), &ConversationStatus::Cancelled);
}

#[test]
fn pending_input_can_restore_when_no_source_task_was_persisted() {
    let pending = pending_query();
    let pending_id = pending.id;
    let mut state = CompactionState::default();
    state.set_pending_recovery(pending);
    let conversation = AIConversation::new_restored_synthesizing_on_empty(
        AIConversationId::new(),
        vec![],
        Some(conversation_data(&state)),
    )
    .unwrap();

    assert!(conversation.exchange_with_id(pending_id).is_some());
    assert_eq!(conversation.status(), &ConversationStatus::Cancelled);
    assert!(conversation.all_linearized_messages().is_empty());
}

#[test]
fn replacing_cancelled_compaction_preserves_both_inputs_and_only_resumes_latest() {
    let first = pending_query();
    let first_id = first.id;
    let first_input = first.queries[0].to_input();
    let mut second = pending_query();
    second.queries[0].query = "第二个待发问题".into();
    second.queries[0].referenced_attachments = HashMap::from([(
        "second-file".into(),
        AIAgentAttachment::PlainText("第二个原始附件".into()),
    )]);
    second.request_start_ts = Local.timestamp_opt(1_700_000_060, 0).unwrap();
    second.summary_request_id = Some("second-summary-request".into());
    let second_id = second.id;
    let second_input = second.queries[0].to_input();
    let mut state = CompactionState::default();
    state.set_pending_recovery(first);
    let mut conversation = restored(
        vec![
            user_message("old-query", "old-request", "原会话问题"),
            output_message("old-output", "old-request"),
        ],
        &state,
    );
    assert_eq!(conversation.status(), &ConversationStatus::Cancelled);
    conversation.compaction_state.set_pending_recovery(second);

    let ModelEvent::UpdateMultiAgentConversation {
        updated_tasks,
        conversation_data,
        ..
    } = conversation.updated_conversation_state_event()
    else {
        panic!("应产生完整会话快照");
    };
    let reloaded =
        AIConversation::new_restored(conversation.id(), updated_tasks, Some(conversation_data))
            .unwrap();

    assert_eq!(
        reloaded.exchange_with_id(first_id).unwrap().input,
        vec![first_input]
    );
    assert_eq!(
        reloaded.exchange_with_id(second_id).unwrap().input,
        vec![second_input]
    );
    assert_eq!(
        reloaded
            .root_task_exchanges()
            .flat_map(|exchange| exchange.input.iter())
            .filter_map(AIAgentInput::display_query)
            .collect::<Vec<_>>(),
        vec!["原会话问题", "/plan 待发问题", "/plan 第二个待发问题"]
    );
    assert_eq!(reloaded.latest_visible_exchange().unwrap().id, second_id);
    let pending = reloaded.compaction_state.pending_recovery().unwrap();
    assert!(pending.is_resume_target(second_id));
    assert!(!pending.is_resume_target(first_id));
    assert_eq!(reloaded.all_linearized_messages().len(), 2);
}

#[test]
fn retained_input_is_persisted_without_current_pending_or_completed_summary() {
    let first = pending_query();
    let first_id = first.id;
    let first_input = first.queries[0].to_input();
    let mut second = pending_query();
    second.queries[0].query = "已发送的第二个问题".into();
    second.resumed_request_id = Some("second-request".into());
    second.request_start_ts = Local.timestamp_opt(1_700_000_060, 0).unwrap();
    let mut state = CompactionState::default();
    state.set_pending_recovery(first);
    state.set_pending_recovery(second);
    let conversation = restored(
        vec![
            user_message("old-query", "old-request", "原会话问题"),
            output_message("old-output", "old-request"),
            user_message("second-query", "second-request", "已发送的第二个问题"),
            output_message("second-output", "second-request"),
        ],
        &state,
    );
    assert!(conversation.compaction_state.pending_recovery().is_none());
    assert!(conversation.compaction_state.completed().is_empty());

    let ModelEvent::UpdateMultiAgentConversation {
        updated_tasks,
        conversation_data,
        ..
    } = conversation.updated_conversation_state_event()
    else {
        panic!("应产生完整会话快照");
    };
    assert!(conversation_data.compaction_state_json.is_some());
    let reloaded =
        AIConversation::new_restored(conversation.id(), updated_tasks, Some(conversation_data))
            .unwrap();

    assert_eq!(
        reloaded.exchange_with_id(first_id).unwrap().input,
        vec![first_input]
    );
    assert!(reloaded.compaction_state.pending_recovery().is_none());
    assert_eq!(reloaded.root_task_exchanges().count(), 3);
    assert_eq!(
        reloaded.latest_user_query().as_deref(),
        Some("已发送的第二个问题")
    );
    assert_eq!(reloaded.status(), &ConversationStatus::Success);
}

#[test]
fn retained_input_with_formal_source_is_removed_without_duplicate_display() {
    let mut first = pending_query();
    let first_id = first.id;
    first.resumed_request_id = Some("first-request".into());
    let mut second = pending_query();
    second.queries[0].query = "第二个待发问题".into();
    second.anchor_message_id = Some("first-output".into());
    second.request_start_ts = Local.timestamp_opt(1_700_000_060, 0).unwrap();
    let second_id = second.id;
    let mut state = CompactionState::default();
    state.set_pending_recovery(first);
    state.set_pending_recovery(second);
    let conversation = restored(
        vec![
            user_message("old-query", "old-request", "原会话问题"),
            output_message("old-output", "old-request"),
            user_message("first-query", "first-request", "待发问题"),
            output_message("first-output", "first-request"),
        ],
        &state,
    );

    assert!(
        conversation
            .compaction_state
            .retained_recoveries()
            .is_empty()
    );
    assert!(conversation.exchange_with_id(first_id).is_none());
    assert_eq!(conversation.root_task_exchanges().count(), 3);
    assert_eq!(
        conversation.latest_visible_exchange().unwrap().id,
        second_id
    );
    assert_eq!(
        conversation.compaction_state.pending_recovery().unwrap().id,
        second_id
    );
}
