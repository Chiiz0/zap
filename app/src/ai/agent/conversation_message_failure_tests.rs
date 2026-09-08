use std::collections::HashMap;

use ai::skills::SkillPathOrigin;
use chrono::{Local, TimeZone as _};
use warp_multi_agent_api as api;
use warpui::{App, EntityId, ModelContext};

use super::{AIConversation, AIConversationId, UpdateConversationError};
use crate::ai::agent::api::user_inputs_from_messages;
use crate::ai::agent::task::{TaskId, UpdateTaskError};
use crate::ai::agent::{AIAgentExchangeId, MessageId};
use crate::ai::blocklist::history_model::BlocklistAIHistoryModel;
use crate::ai::blocklist::{RequestInput, ResponseStreamId};
use crate::test_util::ai_agent_tasks::{
    create_api_subtask, create_api_task, create_message, create_subagent_tool_call_message,
};

fn user_message(id: &str, task_id: &str, query: &str) -> api::Message {
    let mut message = create_message(id, task_id);
    message.message = Some(api::message::Message::UserQuery(api::message::UserQuery {
        query: query.into(),
        ..Default::default()
    }));
    message
}

fn invalid_skill_message(task_id: &str) -> api::Message {
    let mut message = create_message("invalid-skill", task_id);
    message.message = Some(api::message::Message::ToolCall(api::message::ToolCall {
        tool_call_id: "invalid-skill-call".into(),
        tool: Some(api::message::tool_call::Tool::ReadSkill(
            api::message::tool_call::ReadSkill::default(),
        )),
    }));
    message
}

fn restored_conversation(tasks: Vec<api::Task>) -> AIConversation {
    let mut conversation =
        AIConversation::new_restored(AIConversationId::new(), tasks, None).unwrap();
    // 共享会话会从服务端消息重建输入，同时跳过持久化；可额外检查失败批次的输入回滚。
    conversation.is_viewing_shared_session = true;
    conversation
}

fn conversation_with_history() -> AIConversation {
    restored_conversation(vec![create_api_task(
        "root",
        vec![
            user_message("old-query", "root", "已有问题"),
            create_message("old-output", "root"),
        ],
    )])
}

fn start_request(
    conversation: &mut AIConversation,
    task_id: &str,
    stream_id: &ResponseStreamId,
    terminal_surface_id: EntityId,
    query: &str,
    ctx: &mut ModelContext<BlocklistAIHistoryModel>,
) -> AIAgentExchangeId {
    let input = user_inputs_from_messages(&[user_message("request-query", task_id, query)]);
    conversation
        .update_for_new_request_input(
            RequestInput {
                conversation_id: conversation.id(),
                input_messages: HashMap::from([(TaskId::new(task_id.into()), input)]),
                working_directory: Some("/workspace".into()),
                model_id: "test-model".into(),
                coding_model_id: "test-model".into(),
                cli_agent_model_id: "test-model".into(),
                computer_use_model_id: "test-model".into(),
                shared_session_response_initiator: None,
                request_start_ts: Local.timestamp_opt(1_700_000_000, 0).unwrap(),
                supported_tools_override: None,
            },
            stream_id.clone(),
            terminal_surface_id,
            ctx,
        )
        .unwrap();
    conversation
        .initialize_output_for_response_stream(
            stream_id,
            api::response_event::StreamInit {
                request_id: "test-request".into(),
                ..Default::default()
            },
            terminal_surface_id,
            ctx,
        )
        .unwrap();
    conversation
        .new_exchange_ids_for_response(stream_id)
        .next()
        .unwrap()
}

fn add_messages(
    conversation: &mut AIConversation,
    task_id: &str,
    stream_id: &ResponseStreamId,
    terminal_surface_id: EntityId,
    messages: Vec<api::Message>,
    ctx: &mut ModelContext<BlocklistAIHistoryModel>,
) -> Result<(), UpdateConversationError> {
    conversation.apply_client_action(
        stream_id,
        terminal_surface_id,
        api::client_action::Action::AddMessagesToTask(api::client_action::AddMessagesToTask {
            task_id: task_id.into(),
            messages,
        }),
        &SkillPathOrigin::Local,
        ctx,
    )
}

#[test]
fn rejected_tool_message_preserves_history_and_allows_follow_up() {
    App::test((), |mut app| async move {
        let history = app.add_model(|_| BlocklistAIHistoryModel::new_for_test());
        history.update(&mut app, |_, ctx| {
            let mut conversation = conversation_with_history();
            let stream_id = ResponseStreamId::new_for_test();
            let terminal_surface_id = EntityId::new();
            let exchange_id = start_request(
                &mut conversation,
                "root",
                &stream_id,
                terminal_surface_id,
                "检查 LocalMind",
                ctx,
            );
            add_messages(
                &mut conversation,
                "root",
                &stream_id,
                terminal_surface_id,
                vec![create_message("before-error", "root")],
                ctx,
            )
            .unwrap();
            let source_before = conversation
                .get_root_task()
                .unwrap()
                .source()
                .unwrap()
                .clone();
            let exchange_before = conversation.exchange_with_id(exchange_id).unwrap().clone();

            let result = add_messages(
                &mut conversation,
                "root",
                &stream_id,
                terminal_surface_id,
                vec![
                    user_message("rejected-query", "root", "不应残留的输入"),
                    create_message("rejected-output", "root"),
                    invalid_skill_message("root"),
                ],
                ctx,
            );

            assert!(matches!(
                result,
                Err(UpdateConversationError::UpdateTask(
                    UpdateTaskError::ConversionError(_)
                ))
            ));
            assert_eq!(
                conversation.get_root_task().unwrap().source(),
                Some(&source_before)
            );
            let exchange = conversation.exchange_with_id(exchange_id).unwrap();
            assert_eq!(exchange.input, exchange_before.input);
            assert_eq!(
                exchange.added_message_ids,
                exchange_before.added_message_ids
            );
            assert_eq!(
                *exchange.output_status.output().unwrap().get(),
                *exchange_before.output_status.output().unwrap().get()
            );

            add_messages(
                &mut conversation,
                "root",
                &stream_id,
                terminal_surface_id,
                vec![create_message("after-error", "root")],
                ctx,
            )
            .unwrap();
            conversation
                .mark_request_completed(&stream_id, terminal_surface_id, ctx)
                .unwrap();
            let follow_up_stream = ResponseStreamId::new_for_test();
            let follow_up_exchange = start_request(
                &mut conversation,
                "root",
                &follow_up_stream,
                terminal_surface_id,
                "继续检查",
                ctx,
            );
            add_messages(
                &mut conversation,
                "root",
                &follow_up_stream,
                terminal_surface_id,
                vec![create_message("follow-up-output", "root")],
                ctx,
            )
            .unwrap();

            assert_eq!(
                conversation.latest_user_query().as_deref(),
                Some("继续检查")
            );
            assert_eq!(
                conversation.latest_exchange().unwrap().id,
                follow_up_exchange
            );
            assert_eq!(
                conversation
                    .get_root_task()
                    .unwrap()
                    .messages()
                    .map(|message| message.id.as_str())
                    .collect::<Vec<_>>(),
                vec![
                    "old-query",
                    "old-output",
                    "before-error",
                    "after-error",
                    "follow-up-output"
                ]
            );
        });
    });
}

#[test]
fn missing_request_metadata_does_not_remove_existing_task() {
    App::test((), |mut app| async move {
        let history = app.add_model(|_| BlocklistAIHistoryModel::new_for_test());
        history.update(&mut app, |_, ctx| {
            let mut conversation = conversation_with_history();
            let source_before = conversation
                .get_root_task()
                .unwrap()
                .source()
                .unwrap()
                .clone();
            let exchange_count = conversation.exchange_count();

            let result = add_messages(
                &mut conversation,
                "root",
                &ResponseStreamId::new_for_test(),
                EntityId::new(),
                vec![create_message("untracked-output", "root")],
                ctx,
            );

            assert!(matches!(
                result,
                Err(UpdateConversationError::NoPendingRequest)
            ));
            assert_eq!(
                conversation.get_root_task().unwrap().source(),
                Some(&source_before)
            );
            assert_eq!(conversation.exchange_count(), exchange_count);
        });
    });
}

#[test]
fn rejected_message_removes_only_its_new_empty_exchange() {
    App::test((), |mut app| async move {
        let history = app.add_model(|_| BlocklistAIHistoryModel::new_for_test());
        history.update(&mut app, |_, ctx| {
            let mut conversation = restored_conversation(vec![
                create_api_task(
                    "root",
                    vec![
                        user_message("old-query", "root", "已有问题"),
                        create_subagent_tool_call_message("child-call", "root", "child", None),
                    ],
                ),
                create_api_subtask(
                    "child",
                    "root",
                    vec![create_message("child-output", "child")],
                ),
            ]);
            let stream_id = ResponseStreamId::new_for_test();
            let terminal_surface_id = EntityId::new();
            start_request(
                &mut conversation,
                "child",
                &stream_id,
                terminal_surface_id,
                "子任务跟进",
                ctx,
            );
            let root_before = conversation
                .get_root_task()
                .unwrap()
                .source()
                .unwrap()
                .clone();
            let exchange_count = conversation.exchange_count();

            let result = add_messages(
                &mut conversation,
                "root",
                &stream_id,
                terminal_surface_id,
                vec![invalid_skill_message("root")],
                ctx,
            );

            assert!(matches!(
                result,
                Err(UpdateConversationError::UpdateTask(
                    UpdateTaskError::ConversionError(_)
                ))
            ));
            assert_eq!(
                conversation.get_root_task().unwrap().source(),
                Some(&root_before)
            );
            assert_eq!(conversation.exchange_count(), exchange_count);
            assert_eq!(
                conversation
                    .new_exchange_ids_for_response(&stream_id)
                    .count(),
                1
            );
            assert!(conversation.hidden_exchanges.is_empty());

            add_messages(
                &mut conversation,
                "root",
                &stream_id,
                terminal_surface_id,
                vec![create_message("root-resumed", "root")],
                ctx,
            )
            .unwrap();
            assert_eq!(
                conversation
                    .new_exchange_ids_for_response(&stream_id)
                    .count(),
                2
            );
            assert!(
                conversation
                    .get_root_task()
                    .unwrap()
                    .messages()
                    .any(|message| message.id == "root-resumed")
            );
        });
    });
}

#[test]
fn failed_message_keeps_existing_task_transaction_checkpoint() {
    App::test((), |mut app| async move {
        let history = app.add_model(|_| BlocklistAIHistoryModel::new_for_test());
        history.update(&mut app, |_, ctx| {
            let mut conversation = conversation_with_history();
            let stream_id = ResponseStreamId::new_for_test();
            let terminal_surface_id = EntityId::new();
            let exchange_id = start_request(
                &mut conversation,
                "root",
                &stream_id,
                terminal_surface_id,
                "检查日志",
                ctx,
            );
            let source_before = conversation
                .get_root_task()
                .unwrap()
                .source()
                .unwrap()
                .clone();
            conversation.begin_transaction();

            let result = add_messages(
                &mut conversation,
                "root",
                &stream_id,
                terminal_surface_id,
                vec![invalid_skill_message("root")],
                ctx,
            );
            assert!(matches!(
                result,
                Err(UpdateConversationError::UpdateTask(
                    UpdateTaskError::ConversionError(_)
                ))
            ));
            add_messages(
                &mut conversation,
                "root",
                &stream_id,
                terminal_surface_id,
                vec![create_message("transaction-output", "root")],
                ctx,
            )
            .unwrap();
            conversation.rollback_transaction(&stream_id);

            assert_eq!(
                conversation.get_root_task().unwrap().source(),
                Some(&source_before)
            );
            let exchange = conversation.exchange_with_id(exchange_id).unwrap();
            assert!(
                !exchange
                    .added_message_ids
                    .contains(&MessageId::new("invalid-skill".into()))
            );
            assert!(
                !exchange
                    .added_message_ids
                    .contains(&MessageId::new("transaction-output".into()))
            );
            assert!(
                exchange
                    .output_status
                    .output()
                    .unwrap()
                    .get()
                    .messages
                    .is_empty()
            );
            assert_eq!(
                conversation
                    .new_exchange_ids_for_response(&stream_id)
                    .collect::<Vec<_>>(),
                vec![exchange_id]
            );
        });
    });
}
