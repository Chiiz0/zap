use uuid::Uuid;
use warpui::App;

use super::*;
use crate::ai::blocklist::context_model::{PendingAttachment, PendingFile};
use crate::ai::blocklist::controller::response_stream::StreamCancellation;
use crate::test_util::terminal::{add_window_with_terminal, initialize_app_for_terminal_view};

fn query_input(query: &str) -> AIAgentInput {
    AIAgentInput::UserQuery {
        query: query.to_owned(),
        context: Default::default(),
        static_query_type: None,
        referenced_attachments: Default::default(),
        user_query_mode: Default::default(),
        running_command: None,
        intended_agent: None,
    }
}

fn request_input(
    conversation_id: AIConversationId,
    task_id: TaskId,
    input: AIAgentInput,
) -> RequestInput {
    RequestInput {
        conversation_id,
        input_messages: HashMap::from([(task_id, vec![input])]),
        working_directory: None,
        model_id: LLMId::from("原请求模型"),
        coding_model_id: LLMId::from("测试编程模型"),
        cli_agent_model_id: LLMId::from("测试命令模型"),
        computer_use_model_id: LLMId::from("测试桌面模型"),
        shared_session_response_initiator: None,
        request_start_ts: Local::now(),
        supported_tools_override: None,
    }
}

#[test]
fn byop_compaction_continues_only_after_committed_summary_without_new_work() {
    assert!(can_continue_after_byop_compaction(true, false, false));
    assert!(!can_continue_after_byop_compaction(false, false, false));
    assert!(!can_continue_after_byop_compaction(true, true, false));
    assert!(!can_continue_after_byop_compaction(true, false, true));
}

#[test]
fn automatic_summary_targets_original_conversation_and_model() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        terminal.update(&mut app, |terminal, ctx| {
            let (target, root_task) = BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                let target = history.start_new_conversation(terminal.id(), false, false, false, ctx);
                let root = history.conversation(&target).unwrap().get_root_task_id().clone();
                history.start_new_conversation(terminal.id(), false, false, false, ctx);
                (target, root)
            });
            let original = request_input(target, root_task.clone(), query_input("待发问题"));
            terminal.ai_controller().update(ctx, |controller, ctx| {
                let summary = controller.byop_compaction_request_input(target, Some(&original), ctx).unwrap();
                assert_eq!(summary.conversation_id, target);
                assert_eq!(summary.model_id, LLMId::from("原请求模型"));
                assert!(matches!(summary.input_messages[&root_task].as_slice(), [AIAgentInput::SummarizeConversation { overflow: true, .. }]));
                assert_eq!(summary.supported_tools_override, Some(vec![]));
                assert!(matches!(original.input_messages[&root_task].as_slice(), [AIAgentInput::UserQuery { query, .. }] if query == "待发问题"));
            });
        });
    });
}

fn interrupted_compaction_keeps_input(cancellation: Option<CancellationReason>) {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let (conversation_id, input, context) = terminal.update(&mut app, |terminal, ctx| {
            let controller = terminal.ai_controller().clone();
            let input = terminal.input().clone();
            let context = controller.as_ref(ctx).context_model.clone();
            let stream_id = ResponseStreamId::new_local();
            let (conversation_id, root_task) =
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let conversation_id =
                        history.start_new_conversation(terminal.id(), false, false, false, ctx);
                    let root_task = history
                        .conversation(&conversation_id)
                        .unwrap()
                        .get_root_task_id()
                        .clone();
                    history
                        .update_conversation_for_new_request_input(
                            request_input(
                                conversation_id,
                                root_task.clone(),
                                AIAgentInput::SummarizeConversation {
                                    prompt: None,
                                    overflow: true,
                                    context: Default::default(),
                                },
                            ),
                            stream_id.clone(),
                            terminal.id(),
                            ctx,
                        )
                        .unwrap();
                    (conversation_id, root_task)
                });
            let stream = ctx.add_model(|_| ResponseStream::new_for_test(stream_id.clone()));
            controller.update(ctx, |controller, ctx| {
                controller.register_mock_stream_for_test(
                    stream_id.clone(),
                    conversation_id,
                    stream.clone(),
                    ctx,
                );
                controller.pending_byop_compaction_requests.insert(
                    stream_id.clone(),
                    PendingByopCompactionRequest {
                        request: PendingByopRequest {
                            allow_auto_compaction: false,
                            request_input: request_input(
                                conversation_id,
                                root_task,
                                query_input("摘要前的原问题"),
                            ),
                            query_metadata: None,
                            default_to_follow_up_on_success: true,
                            can_attempt_resume_on_error: true,
                            is_queued_prompt: false,
                        },
                        original_query: Some("摘要前的原问题".to_owned()),
                        context_snapshot: context.as_ref(ctx).pending_context_snapshot(),
                    },
                );
            });
            input.update(ctx, |input, ctx| {
                input.replace_buffer_content("后来编辑的新草稿", ctx)
            });
            context.update(ctx, |context, ctx| {
                context.append_pending_attachments(
                    vec![PendingAttachment::File(PendingFile {
                        file_name: "new.txt".to_owned(),
                        file_path: "/tmp/new.txt".into(),
                        mime_type: "text/plain".to_owned(),
                    })],
                    ctx,
                );
            });
            controller.update(ctx, |controller, ctx| {
                if cancellation.is_some() {
                    controller
                        .in_flight_response_streams
                        .cleanup_stream(&stream_id);
                }
                controller.handle_response_stream_event(
                    false,
                    &ResponseStreamEvent::AfterStreamFinished {
                        cancellation: cancellation.map(|reason| StreamCancellation {
                            reason,
                            conversation_id,
                        }),
                    },
                    &stream,
                    ctx,
                );
                assert!(controller.pending_byop_compaction_requests.is_empty());
                assert!(!controller.has_active_stream_for_conversation(conversation_id, ctx));
            });
            (conversation_id, input, context)
        });
        input.read(&app, |input, ctx| {
            assert_eq!(input.buffer_text(ctx), "后来编辑的新草稿")
        });
        context.read(&app, |context, _| {
            assert_eq!(context.pending_attachments()[0].file_name(), "new.txt")
        });
        BlocklistAIHistoryModel::handle(&app).read(&app, |history, _| {
            assert!(history.conversation(&conversation_id).unwrap().all_exchanges().iter().any(|exchange| {
                exchange.input.iter().any(|input| matches!(input, AIAgentInput::UserQuery { query, .. } if query == "摘要前的原问题"))
            }));
        });
    });
}

#[test]
fn failed_summary_keeps_original_input_and_new_draft() {
    interrupted_compaction_keeps_input(None);
}

#[test]
fn cancelled_summary_keeps_original_input_and_new_draft() {
    interrupted_compaction_keeps_input(Some(CancellationReason::ManuallyCancelled));
}

#[test]
fn compacted_input_clear_preserves_new_attachments() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let (input, context) = terminal.update(&mut app, |terminal, ctx| {
            let input = terminal.input().clone();
            let controller = terminal.ai_controller().clone();
            let context = controller.as_ref(ctx).context_model.clone();
            input.update(ctx, |input, ctx| {
                input.replace_buffer_content("原问题", ctx)
            });
            let snapshot = context.as_ref(ctx).pending_context_snapshot();
            context.update(ctx, |context, ctx| {
                context.set_pending_context_selected_text(
                    Some("后来新增的上下文".to_owned()),
                    true,
                    ctx,
                );
            });
            controller.update(ctx, |_, ctx| {
                ctx.emit(BlocklistAIControllerEvent::SubmittedCompactedInput {
                    query: "原问题".to_owned(),
                    context_snapshot: snapshot,
                })
            });
            (input, context)
        });
        input.read(&app, |input, ctx| {
            assert_eq!(input.buffer_text(ctx), "原问题")
        });
        context.read(&app, |context, _| {
            assert_eq!(
                context.pending_context_selected_text().map(String::as_str),
                Some("后来新增的上下文")
            )
        });
    });
}

#[test]
fn compacted_input_clear_removes_unchanged_draft() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let input = terminal.update(&mut app, |terminal, ctx| {
            let input = terminal.input().clone();
            let controller = terminal.ai_controller().clone();
            input.update(ctx, |input, ctx| {
                input.replace_buffer_content("原问题", ctx)
            });
            let snapshot = controller
                .as_ref(ctx)
                .context_model
                .as_ref(ctx)
                .pending_context_snapshot();
            controller.update(ctx, |_, ctx| {
                ctx.emit(BlocklistAIControllerEvent::SubmittedCompactedInput {
                    query: "原问题".to_owned(),
                    context_snapshot: snapshot,
                })
            });
            input
        });
        input.read(&app, |input, ctx| {
            assert!(input.buffer_text(ctx).is_empty())
        });
    });
}

#[test]
fn internal_summary_finishes_exchange_without_releasing_prompt_queue() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        terminal.update(&mut app, |terminal, ctx| {
            BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                let conversation_id =
                    history.start_new_conversation(terminal.id(), false, false, false, ctx);
                let root = history
                    .conversation(&conversation_id)
                    .unwrap()
                    .get_root_task_id()
                    .clone();
                let stream_id = ResponseStreamId::new_local();
                history
                    .update_conversation_for_new_request_input(
                        request_input(
                            conversation_id,
                            root,
                            AIAgentInput::SummarizeConversation {
                                prompt: None,
                                overflow: true,
                                context: Default::default(),
                            },
                        ),
                        stream_id.clone(),
                        terminal.id(),
                        ctx,
                    )
                    .unwrap();
                history.initialize_output_for_response_stream(
                    &stream_id,
                    conversation_id,
                    terminal.id(),
                    warp_multi_agent_api::response_event::StreamInit {
                        request_id: "本次摘要请求".to_owned(),
                        conversation_id: "服务端会话".to_owned(),
                        run_id: Uuid::new_v4().to_string(),
                    },
                    ctx,
                );
                history.update_conversation_status(
                    terminal.id(),
                    conversation_id,
                    ConversationStatus::InProgress,
                    ctx,
                );
                history.mark_response_stream_completed_for_compaction(
                    &stream_id,
                    conversation_id,
                    terminal.id(),
                    ctx,
                );
                let conversation = history.conversation(&conversation_id).unwrap();
                assert_eq!(*conversation.status(), ConversationStatus::InProgress);
                assert!(
                    conversation
                        .all_exchanges()
                        .iter()
                        .all(|exchange| exchange.output_status.is_finished())
                );
            });
        });
    });
}

#[test]
fn blocked_compaction_continuation_keeps_new_draft_and_context() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let (input, context) = terminal.update(&mut app, |terminal, ctx| {
            let input = terminal.input().clone();
            let controller = terminal.ai_controller().clone();
            let context = controller.as_ref(ctx).context_model.clone();
            let (conversation_id, root) =
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let id =
                        history.start_new_conversation(terminal.id(), false, false, false, ctx);
                    (
                        id,
                        history
                            .conversation(&id)
                            .unwrap()
                            .get_root_task_id()
                            .clone(),
                    )
                });
            input.update(ctx, |input, ctx| {
                input.replace_buffer_content("新草稿", ctx)
            });
            context.update(ctx, |context, ctx| {
                context.set_pending_context_selected_text(Some("新选区".to_owned()), true, ctx)
            });
            controller.update(ctx, |controller, ctx| {
                controller
                    .complete_byop_blocked_request(
                        request_input(conversation_id, root, query_input("原问题")),
                        conversation_id,
                        LLMId::from("原请求模型"),
                        true,
                        "模拟恢复后被预算拦截".to_owned(),
                        ctx,
                    )
                    .unwrap();
            });
            (input, context)
        });
        input.read(&app, |input, ctx| {
            assert_eq!(input.buffer_text(ctx), "新草稿")
        });
        context.read(&app, |context, _| {
            assert_eq!(
                context.pending_context_selected_text().map(String::as_str),
                Some("新选区")
            )
        });
    });
}

#[test]
fn retained_compaction_input_does_not_finish_new_request() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        terminal.update(&mut app, |terminal, ctx| {
            let (conversation_id, root) = BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                let id = history.start_new_conversation(terminal.id(), false, false, false, ctx);
                history.update_conversation_status(terminal.id(), id, ConversationStatus::InProgress, ctx);
                (id, history.conversation(&id).unwrap().get_root_task_id().clone())
            });
            let id = ResponseStreamId::new_local();
            BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                history.update_conversation_for_new_request_input(
                    request_input(conversation_id, root.clone(), query_input("新的进行中请求")),
                    id.clone(),
                    terminal.id(),
                    ctx,
                ).unwrap();
            });
            let stream = ctx.add_model(|_| ResponseStream::new_for_test(id.clone()));
            terminal.ai_controller().update(ctx, |controller, ctx| {
                controller.register_mock_stream_for_test(id, conversation_id, stream, ctx);
                assert!(controller.has_active_stream_for_conversation(conversation_id, ctx));
                controller.retain_input_after_failed_compaction(PendingByopRequest {
                    allow_auto_compaction: false,
                    request_input: request_input(conversation_id, root, query_input("先前未发的问题")),
                    query_metadata: None,
                    default_to_follow_up_on_success: true,
                    can_attempt_resume_on_error: true,
                    is_queued_prompt: false,
                }, None, ctx);
                assert!(controller.has_active_stream_for_conversation(conversation_id, ctx));
            });
            BlocklistAIHistoryModel::handle(ctx).read(ctx, |history, _| {
                let conversation = history.conversation(&conversation_id).unwrap();
                assert_eq!(*conversation.status(), ConversationStatus::InProgress);
                assert!(conversation.all_exchanges().iter().any(|exchange| {
                    exchange.output_status.is_cancelled() && exchange.input.iter().any(|input| {
                        matches!(input, AIAgentInput::UserQuery { query, .. } if query == "先前未发的问题")
                    })
                }));
            });
        });
    });
}
