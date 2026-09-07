use chrono::TimeZone as _;

use super::*;
use crate::ai::agent::{
    AIAgentActionId, AIAgentActionResult, AIAgentActionResultType, AnyFileContent, FileContext,
    ImageContext, RequestCommandOutputResult,
};
use crate::ai::byop_compaction::state::CompactionState;

fn request(input: AIAgentInput) -> RequestInput {
    RequestInput {
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
    }
}

fn query() -> AIAgentInput {
    AIAgentInput::UserQuery {
        query: "分析附件".into(),
        context: Arc::from([
            AIAgentContext::SelectedText("选中的日志".into()),
            AIAgentContext::Image(ImageContext {
                data: "aW1hZ2U=".into(),
                mime_type: "image/png".into(),
                file_name: "screenshot.png".into(),
                is_figma: false,
            }),
            AIAgentContext::File(FileContext::new(
                "report.pdf".into(),
                AnyFileContent::BinaryContent(vec![37, 80, 68, 70]),
                None,
                None,
            )),
        ]),
        referenced_attachments: HashMap::from([(
            "attachment".into(),
            AIAgentAttachment::PlainText("附件正文".into()),
        )]),
        user_query_mode: UserQueryMode::Plan,
        static_query_type: None,
        running_command: None,
        intended_agent: None,
    }
}

#[test]
fn pending_compaction_roundtrip_preserves_multimodal_query_for_manual_continue() {
    let original = request(query());
    let mut state = CompactionState::default();
    state.set_pending_recovery(PendingCompactionRecovery::from_request(&original));
    let persisted = serde_json::to_string(&state).unwrap();
    let restored: CompactionState = serde_json::from_str(&persisted).unwrap();
    let resumed = restored.pending_recovery().unwrap().to_request_input(
        original.conversation_id,
        &TaskId::new("root".into()),
        None,
    );

    assert_eq!(resumed.input_messages, original.input_messages);
    assert_eq!(resumed.model_id, original.model_id);
    assert_eq!(resumed.coding_model_id, original.coding_model_id);
    assert_eq!(resumed.cli_agent_model_id, original.cli_agent_model_id);
    assert_eq!(
        resumed.computer_use_model_id,
        original.computer_use_model_id
    );
    assert_eq!(resumed.request_start_ts, original.request_start_ts);
    assert_eq!(resumed.working_directory, original.working_directory);
    assert!(
        restored
            .pending_recovery()
            .unwrap()
            .resume_context
            .is_empty()
    );
}

#[test]
fn pending_tool_continuation_restores_context_without_replaying_tool_results() {
    let context: Arc<[AIAgentContext]> =
        Arc::from([AIAgentContext::SelectedText("工具上下文".into())]);
    let original = request(AIAgentInput::ActionResult {
        result: AIAgentActionResult {
            id: AIAgentActionId::from("tool-call".to_owned()),
            task_id: TaskId::new("root".into()),
            result: AIAgentActionResultType::RequestCommandOutput(
                RequestCommandOutputResult::CancelledBeforeExecution,
            ),
        },
        context: context.clone(),
    });
    let pending = PendingCompactionRecovery::from_request(&original);
    let resumed =
        pending.to_request_input(original.conversation_id, &TaskId::new("root".into()), None);

    assert_eq!(
        resumed.input_messages[&TaskId::new("root".into())],
        vec![AIAgentInput::ResumeConversation { context }],
    );
    assert!(pending.queries.is_empty());
    assert!(pending.to_cancelled_exchange().output_status.is_finished());
}

#[test]
fn stale_compaction_stream_cannot_clear_or_rebind_new_recovery() {
    let old = PendingCompactionRecovery::from_request(&request(query()));
    let new = PendingCompactionRecovery::from_request(&request(query()));
    let new_id = new.id;
    let mut state = CompactionState::default();
    state.set_pending_recovery(new);

    assert!(!state.clear_pending_recovery(old.id));
    assert!(!state.mark_recovery_resumed(old.id, "old-request".into()));
    assert!(!state.set_recovery_owner(old.id, AIAgentExchangeId::new()));
    assert_eq!(state.pending_recovery().unwrap().id, new_id);
    assert_eq!(state.pending_recovery().unwrap().resumed_request_id, None);
    assert_eq!(state.pending_recovery().unwrap().owner_exchange_id, None);
}

#[test]
fn recovery_owner_roundtrip_preserves_only_the_associated_continue_targets() {
    let pending = PendingCompactionRecovery::from_request(&request(query()));
    let pending_id = pending.id;
    let owner = AIAgentExchangeId::new();
    let unrelated = AIAgentExchangeId::new();
    let mut state = CompactionState::default();
    state.set_pending_recovery(pending);
    assert!(state.set_recovery_owner(pending_id, owner));

    let persisted = serde_json::to_string(&state).unwrap();
    let restored: CompactionState = serde_json::from_str(&persisted).unwrap();
    let pending = restored.pending_recovery().unwrap();

    assert!(pending.is_resume_target(pending_id));
    assert!(pending.is_resume_target(owner));
    assert!(!pending.is_resume_target(unrelated));
}

#[test]
fn replacing_pending_recovery_preserves_cancelled_input_without_duplicate_archive() {
    let old = PendingCompactionRecovery::from_request(&request(query()));
    let old_id = old.id;
    let original_input = old.queries[0].to_input();
    let new = PendingCompactionRecovery::from_request(&request(query()));
    let new_id = new.id;
    let mut state = CompactionState::default();
    state.set_pending_recovery(old.clone());
    state.set_pending_recovery(old);
    assert!(state.retained_recoveries().is_empty());
    state.set_pending_recovery(new.clone());
    state.set_pending_recovery(new);

    let restored: CompactionState =
        serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();

    assert_eq!(restored.retained_recoveries().len(), 1);
    assert_eq!(restored.retained_recoveries()[0].id, old_id);
    assert_eq!(
        restored.retained_recoveries()[0].queries[0].to_input(),
        original_input
    );
    assert_eq!(restored.pending_recovery().unwrap().id, new_id);
    assert!(
        !restored
            .pending_recovery()
            .unwrap()
            .is_resume_target(old_id)
    );
}

#[test]
fn pending_recovery_without_an_owner_remains_backward_compatible() {
    let pending = PendingCompactionRecovery::from_request(&request(query()));
    let serialized = serde_json::to_value(&pending).unwrap();
    assert!(serialized.get("owner_exchange_id").is_none());

    let restored: PendingCompactionRecovery = serde_json::from_value(serialized).unwrap();

    assert_eq!(restored.owner_exchange_id, None);
    assert!(restored.is_resume_target(pending.id));
    assert!(!restored.is_resume_target(AIAgentExchangeId::new()));
}

#[test]
fn legacy_compaction_sidecar_has_no_pending_recovery() {
    let state: CompactionState =
        serde_json::from_str(r#"{"version":3,"markers":{},"completed":[]}"#).unwrap();

    assert!(state.pending_recovery().is_none());
    assert!(state.retained_recoveries().is_empty());
    assert!(state.completed().is_empty());
}
