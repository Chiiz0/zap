use ::settings::Setting as _;
use uuid::Uuid;
use warpui::App;

use super::*;
use crate::ai::blocklist::context_model::{PendingAttachment, PendingFile};
use crate::ai::blocklist::controller::response_stream::StreamCancellation;
use crate::ai::blocklist::{QueuedQuery, QueuedQueryOrigin};
use crate::settings::{AISettings, AgentProvider, AgentProviderApiType, AgentProviderModel};
use crate::test_util::ai_agent_tasks::{create_api_task, create_message};
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
                            context_snapshot: None,
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
                        clear_input_on_success: true,
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
        let stream_id = ResponseStreamId::new_local();
        let (input, context, controller) = terminal.update(&mut app, |terminal, ctx| {
            let input = terminal.input().clone();
            let controller = terminal.ai_controller().clone();
            let context = controller.as_ref(ctx).context_model.clone();
            input.update(ctx, |input, ctx| {
                input.replace_buffer_content("/plan 原问题", ctx)
            });
            controller.update(ctx, |_, ctx| {
                ctx.emit(BlocklistAIControllerEvent::CompactingInput {
                    stream_id: stream_id.clone(),
                    query: "原问题".to_owned(),
                    user_query_mode: UserQueryMode::Plan,
                });
            });
            (input, context, controller)
        });
        context.update(&mut app, |context, ctx| {
            context.set_pending_context_selected_text(
                Some("后来新增的上下文".to_owned()),
                true,
                ctx,
            );
        });
        controller.update(&mut app, |_, ctx| {
            ctx.emit(BlocklistAIControllerEvent::SubmittedCompactedInput {
                stream_id,
                submitted: true,
            });
        });
        input.read(&app, |input, ctx| {
            assert_eq!(input.buffer_text(ctx), "/plan 原问题")
        });
        context.read(&app, |context, _| {
            assert_eq!(
                context.pending_context_selected_text().map(String::as_str),
                Some("后来新增的上下文")
            );
        });
    });
}

#[test]
fn compacted_input_clear_removes_unchanged_plain_and_slash_drafts() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let init = crate::ai::agent_providers::prompt_renderer::render_init_project_command(None);
        let init_with_args =
            crate::ai::agent_providers::prompt_renderer::render_init_project_command(Some(
                "关注测试",
            ));
        for (draft, query, user_query_mode) in [
            ("原问题", "原问题", UserQueryMode::Normal),
            ("/plan 原问题", "原问题", UserQueryMode::Plan),
            ("/init", init.as_str(), UserQueryMode::Normal),
            (
                "/init 关注测试",
                init_with_args.as_str(),
                UserQueryMode::Normal,
            ),
        ] {
            let stream_id = ResponseStreamId::new_local();
            let (input, controller) = terminal.update(&mut app, |terminal, ctx| {
                let input = terminal.input().clone();
                let controller = terminal.ai_controller().clone();
                input.update(ctx, |input, ctx| input.replace_buffer_content(draft, ctx));
                controller.update(ctx, |_, ctx| {
                    ctx.emit(BlocklistAIControllerEvent::CompactingInput {
                        stream_id: stream_id.clone(),
                        query: query.to_owned(),
                        user_query_mode,
                    });
                });
                (input, controller)
            });
            controller.update(&mut app, |_, ctx| {
                ctx.emit(BlocklistAIControllerEvent::SubmittedCompactedInput {
                    stream_id,
                    submitted: true,
                });
            });
            input.read(&app, |input, ctx| {
                assert!(input.buffer_text(ctx).is_empty(), "未清空草稿：{draft}")
            });
        }
    });
}

#[test]
fn old_compaction_completion_does_not_consume_new_compaction_draft() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let old_stream = ResponseStreamId::new_local();
        let new_stream = ResponseStreamId::new_local();
        let (input, controller) = terminal.update(&mut app, |terminal, ctx| {
            let input = terminal.input().clone();
            let controller = terminal.ai_controller().clone();
            input.update(ctx, |input, ctx| {
                input.replace_buffer_content("旧问题", ctx)
            });
            controller.update(ctx, |_, ctx| {
                ctx.emit(BlocklistAIControllerEvent::CompactingInput {
                    stream_id: old_stream.clone(),
                    query: "旧问题".to_owned(),
                    user_query_mode: UserQueryMode::Normal,
                });
            });
            (input, controller)
        });
        input.update(&mut app, |input, ctx| {
            input.replace_buffer_content("/plan 新问题", ctx)
        });
        controller.update(&mut app, |_, ctx| {
            ctx.emit(BlocklistAIControllerEvent::CompactingInput {
                stream_id: new_stream.clone(),
                query: "新问题".to_owned(),
                user_query_mode: UserQueryMode::Plan,
            });
        });
        controller.update(&mut app, |_, ctx| {
            ctx.emit(BlocklistAIControllerEvent::SubmittedCompactedInput {
                stream_id: old_stream,
                submitted: false,
            });
        });
        input.read(&app, |input, ctx| {
            assert_eq!(input.buffer_text(ctx), "/plan 新问题")
        });
        controller.update(&mut app, |_, ctx| {
            ctx.emit(BlocklistAIControllerEvent::SubmittedCompactedInput {
                stream_id: new_stream,
                submitted: true,
            });
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
            let summary_stream = ResponseStreamId::new_local();
            let summary_exchange = BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                history.update_conversation_for_new_request_input(
                    request_input(conversation_id, root.clone(), AIAgentInput::SummarizeConversation {
                        prompt: None, overflow: true, context: Default::default(),
                    }),
                    summary_stream.clone(), terminal.id(), ctx,
                ).unwrap();
                let conversation = history.conversation_mut(&conversation_id).unwrap();
                let exchange_id = conversation.new_exchange_ids_for_response(&summary_stream).next().unwrap();
                conversation.cleanup_completed_response_stream(&summary_stream);
                exchange_id
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
                    context_snapshot: None,
                    allow_auto_compaction: false,
                    request_input: request_input(conversation_id, root, query_input("先前未发的问题")),
                    query_metadata: None,
                    default_to_follow_up_on_success: true,
                    can_attempt_resume_on_error: true,
                    is_queued_prompt: false,
                }, Some(summary_exchange), None, ctx);
                assert!(controller.has_active_stream_for_conversation(conversation_id, ctx));
            });
            BlocklistAIHistoryModel::handle(ctx).read(ctx, |history, _| {
                let conversation = history.conversation(&conversation_id).unwrap();
                assert_eq!(*conversation.status(), ConversationStatus::InProgress);
                assert!(conversation.latest_exchange().unwrap().input.iter().any(|input| {
                    matches!(input, AIAgentInput::UserQuery { query, .. } if query == "新的进行中请求")
                }));
                assert_eq!(conversation.all_exchanges().len(), 2);
                assert!(conversation.all_exchanges().iter().any(|exchange| {
                    exchange.id == summary_exchange && exchange.output_status.is_cancelled() && exchange.input.iter().any(|input| {
                        matches!(input, AIAgentInput::UserQuery { query, .. } if query == "先前未发的问题")
                    })
                }));
            });
        });
    });
}

#[test]
fn cancelled_compaction_preserves_new_turn_and_fires_queue_only_after_it_finishes() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let old_id = ResponseStreamId::new_local();
        let new_id = ResponseStreamId::new_local();
        let (conversation_id, controller, old_stream, new_stream) =
            terminal.update(&mut app, |terminal, ctx| {
                let controller = terminal.ai_controller().clone();
                let (conversation_id, root) =
                    BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                        let conversation_id =
                            history.start_new_conversation(terminal.id(), false, false, false, ctx);
                        let root = history
                            .conversation(&conversation_id)
                            .unwrap()
                            .get_root_task_id()
                            .clone();
                        history
                            .update_conversation_for_new_request_input(
                                request_input(
                                    conversation_id,
                                    root.clone(),
                                    AIAgentInput::SummarizeConversation {
                                        prompt: None,
                                        overflow: true,
                                        context: Default::default(),
                                    },
                                ),
                                old_id.clone(),
                                terminal.id(),
                                ctx,
                            )
                            .unwrap();
                        history
                            .update_conversation_for_new_request_input(
                                request_input(conversation_id, root.clone(), query_input("新问题")),
                                new_id.clone(),
                                terminal.id(),
                                ctx,
                            )
                            .unwrap();
                        history.initialize_output_for_response_stream(
                            &new_id,
                            conversation_id,
                            terminal.id(),
                            warp_multi_agent_api::response_event::StreamInit {
                                request_id: "新问题的输出".to_owned(),
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
                        (conversation_id, root)
                    });
                let old_stream = ctx.add_model(|_| ResponseStream::new_for_test(old_id.clone()));
                let new_stream = ctx.add_model(|_| ResponseStream::new_for_test(new_id.clone()));
                controller.update(ctx, |controller, ctx| {
                    controller.pending_byop_compaction_requests.insert(
                        old_id.clone(),
                        PendingByopCompactionRequest {
                            request: PendingByopRequest {
                                context_snapshot: None,
                                allow_auto_compaction: false,
                                request_input: request_input(
                                    conversation_id,
                                    root,
                                    query_input("已被替代的旧问题"),
                                ),
                                query_metadata: None,
                                default_to_follow_up_on_success: true,
                                can_attempt_resume_on_error: false,
                                is_queued_prompt: false,
                            },
                            clear_input_on_success: false,
                        },
                    );
                    controller.register_mock_stream_for_test(
                        new_id.clone(),
                        conversation_id,
                        new_stream.clone(),
                        ctx,
                    );
                });
                QueuedQueryModel::handle(ctx).update(ctx, |queue, ctx| {
                    queue.append(
                        conversation_id,
                        QueuedQuery::new(
                            "排队的问题".to_owned(),
                            QueuedQueryOrigin::QueueSlashCommand,
                        ),
                        ctx,
                    );
                });
                (conversation_id, controller, old_stream, new_stream)
            });
        controller.update(&mut app, |controller, ctx| {
            controller.handle_response_stream_event(
                false,
                &ResponseStreamEvent::AfterStreamFinished {
                    cancellation: Some(StreamCancellation {
                        reason: CancellationReason::FollowUpSubmitted {
                            is_for_same_conversation: true,
                        },
                        conversation_id,
                    }),
                },
                &old_stream,
                ctx,
            );
        });
        QueuedQueryModel::handle(&app).read(&app, |queue, _| {
            assert_eq!(
                queue.queue(conversation_id).len(),
                1,
                "新请求进行中不能提前处理队列"
            );
        });
        controller.read(&app, |controller, ctx| {
            assert!(controller.has_active_stream_for_conversation(conversation_id, ctx));
        });
        terminal.update(&mut app, |terminal, ctx| {
            BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                history.mark_response_stream_completed_successfully(
                    &new_id,
                    conversation_id,
                    terminal.id(),
                    ctx,
                );
            });
            controller.update(ctx, |controller, ctx| {
                controller.handle_response_stream_event(
                    false,
                    &ResponseStreamEvent::AfterStreamFinished { cancellation: None },
                    &new_stream,
                    ctx,
                );
            });
        });
        QueuedQueryModel::handle(&app).read(&app, |queue, _| {
            assert!(
                queue.queue(conversation_id).is_empty(),
                "新请求成功后应继续发送队列"
            );
        });
        BlocklistAIHistoryModel::handle(&app).read(&app, |history, _| {
            assert!(history.conversation(&conversation_id).unwrap().all_exchanges().iter().any(|exchange| {
                exchange.input.iter().any(|input| matches!(input, AIAgentInput::UserQuery { query, .. } if query == "排队的问题"))
            }));
        });
    });
}

#[test]
fn delayed_compaction_does_not_capture_a_draft_edited_while_waiting_for_tools() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let stream_id = ResponseStreamId::new_local();
        let (input, controller) = terminal.update(&mut app, |terminal, ctx| {
            let input = terminal.input().clone();
            let controller = terminal.ai_controller().clone();
            input.update(ctx, |input, ctx| {
                input.replace_buffer_content("等待工具时编辑的新草稿", ctx)
            });
            controller.update(ctx, |_, ctx| {
                ctx.emit(BlocklistAIControllerEvent::CompactingInput {
                    stream_id: stream_id.clone(),
                    query: "旧请求".to_owned(),
                    user_query_mode: UserQueryMode::Normal,
                });
            });
            (input, controller)
        });
        controller.update(&mut app, |_, ctx| {
            ctx.emit(BlocklistAIControllerEvent::SubmittedCompactedInput {
                stream_id,
                submitted: true,
            });
        });
        input.read(&app, |input, ctx| {
            assert_eq!(input.buffer_text(ctx), "等待工具时编辑的新草稿")
        });
    });
}

#[test]
fn blocked_compaction_resume_preserves_unchanged_draft_and_releases_snapshot() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let stream_id = ResponseStreamId::new_local();
        let (input, controller) = terminal.update(&mut app, |terminal, ctx| {
            let input = terminal.input().clone();
            let controller = terminal.ai_controller().clone();
            input.update(ctx, |input, ctx| {
                input.replace_buffer_content("/plan 原问题", ctx)
            });
            controller.update(ctx, |_, ctx| {
                ctx.emit(BlocklistAIControllerEvent::CompactingInput {
                    stream_id: stream_id.clone(),
                    query: "原问题".to_owned(),
                    user_query_mode: UserQueryMode::Plan,
                });
            });
            (input, controller)
        });
        controller.update(&mut app, |_, ctx| {
            ctx.emit(BlocklistAIControllerEvent::SubmittedCompactedInput {
                stream_id: stream_id.clone(),
                submitted: false,
            });
        });
        input.read(&app, |input, ctx| {
            assert_eq!(input.buffer_text(ctx), "/plan 原问题")
        });
        // 即便旧流的成功事件重复到达，失败时已经释放的快照也不能再次清空草稿。
        controller.update(&mut app, |_, ctx| {
            ctx.emit(BlocklistAIControllerEvent::SubmittedCompactedInput {
                stream_id,
                submitted: true,
            });
        });
        input.read(&app, |input, ctx| {
            assert_eq!(input.buffer_text(ctx), "/plan 原问题")
        });
    });
}

#[test]
fn retained_compaction_input_preserves_the_actual_summary_error() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        terminal.update(&mut app, |terminal, ctx| {
            let stream_id = ResponseStreamId::new_local();
            let (conversation_id, root, exchange_id) = BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                let conversation_id = history.start_new_conversation(terminal.id(), false, false, false, ctx);
                let root = history.conversation(&conversation_id).unwrap().get_root_task_id().clone();
                history.update_conversation_for_new_request_input(
                    request_input(conversation_id, root.clone(), AIAgentInput::SummarizeConversation {
                        prompt: None, overflow: true, context: Default::default(),
                    }), stream_id.clone(), terminal.id(), ctx,
                ).unwrap();
                history.mark_response_stream_completed_with_error(
                    RenderableAIError::Other {
                        error_message: "摘要提供商鉴权失败".to_owned(), will_attempt_resume: false,
                        waiting_for_network: false, is_user_error: true,
                    }, false, &stream_id, conversation_id, terminal.id(), ctx,
                );
                let exchange_id = history.conversation(&conversation_id).unwrap().new_exchange_ids_for_response(&stream_id).next().unwrap();
                (conversation_id, root, exchange_id)
            });
            terminal.ai_controller().update(ctx, |controller, ctx| {
                controller.retain_input_after_failed_compaction(PendingByopRequest {
                    context_snapshot: None,
                    allow_auto_compaction: false,
                    request_input: request_input(conversation_id, root, query_input("待发原问题")),
                    query_metadata: None, default_to_follow_up_on_success: true,
                    can_attempt_resume_on_error: false, is_queued_prompt: false,
                }, Some(exchange_id), None, ctx);
            });
            BlocklistAIHistoryModel::handle(ctx).read(ctx, |history, _| {
                let exchange = history.conversation(&conversation_id).unwrap().exchange_with_id(exchange_id).unwrap();
                assert!(exchange.input.iter().any(|input| matches!(input, AIAgentInput::UserQuery { query, .. } if query == "待发原问题")));
                assert!(matches!(&exchange.output_status, AIAgentOutputStatus::Finished {
                    finished_output: FinishedAIAgentOutput::Error {
                        error: RenderableAIError::Other { error_message, is_user_error: true, .. }, ..
                    },
                } if error_message == "摘要提供商鉴权失败"));
            });
        });
    });
}

#[test]
fn readiness_compaction_preserves_context_added_while_waiting_with_unchanged_text() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let mut provider = AgentProvider::new_empty();
        provider.id = "readiness-context-provider".to_owned();
        provider.api_type = AgentProviderApiType::OpenAi;
        provider.base_url = "http://localhost:1/v1".to_owned();
        let mut model = AgentProviderModel::from_id("context-model".to_owned());
        model.context_window = 200_000;
        provider.models.push(model);
        AISettings::handle(&app).update(&mut app, |settings, ctx| {
            settings
                .agent_providers
                .set_value(vec![provider.clone()], ctx)
                .unwrap();
        });
        for changed_context in [false, true] {
            let (conversation_id, controller, input, context, summary_id) =
                terminal.update(&mut app, |terminal, ctx| {
                    let controller = terminal.ai_controller().clone();
                    let input = terminal.input().clone();
                    let context = controller.as_ref(ctx).context_model.clone();
                    context.update(ctx, |context, ctx| context.reset_context_to_default(ctx));
                    input.update(ctx, |input, ctx| {
                        input.replace_buffer_content("相同的原问题", ctx)
                    });
                    let (conversation_id, root) =
                        BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                            let id = history.start_new_conversation(
                                terminal.id(),
                                false,
                                false,
                                false,
                                ctx,
                            );
                            let root = "readiness-root";
                            let messages = (0..4)
                                .flat_map(|turn| {
                                    let mut user = create_message(&format!("user-{turn}"), root);
                                    user.request_id = format!("request-{turn}");
                                    user.message =
                                        Some(message::Message::UserQuery(message::UserQuery {
                                            query: format!("历史问题{turn}"),
                                            ..Default::default()
                                        }));
                                    let mut answer =
                                        create_message(&format!("answer-{turn}"), root);
                                    answer.request_id = user.request_id.clone();
                                    answer.message =
                                        Some(message::Message::AgentOutput(message::AgentOutput {
                                            text: "x".repeat(90_000),
                                        }));
                                    [user, answer]
                                })
                                .collect();
                            history
                                .conversation_mut(&id)
                                .unwrap()
                                .upgrade_optimistic_root_to_server_task_for_test(create_api_task(
                                    root, messages,
                                ));
                            (id, TaskId::new(root.to_owned()))
                        });
                    let snapshot = context.as_ref(ctx).pending_context_snapshot();
                    if changed_context {
                        context.update(ctx, |context, ctx| {
                            context.append_pending_attachments(
                                vec![PendingAttachment::File(PendingFile {
                                    file_name: "later.txt".to_owned(),
                                    file_path: "/tmp/later.txt".into(),
                                    mime_type: "text/plain".to_owned(),
                                })],
                                ctx,
                            );
                            context.set_pending_context_selected_text(
                                Some("后来添加的选区".to_owned()),
                                true,
                                ctx,
                            );
                        });
                    }
                    let summary_id = controller.update(ctx, |controller, ctx| {
                        let mut pending_input =
                            request_input(conversation_id, root, query_input("相同的原问题"));
                        pending_input.model_id = crate::ai::agent_providers::llm_id::encode(
                            &provider.id,
                            "context-model",
                        );
                        controller.pending_byop_requests.insert(
                            conversation_id,
                            PendingByopRequest {
                                context_snapshot: Some(snapshot),
                                allow_auto_compaction: true,
                                request_input: pending_input,
                                query_metadata: None,
                                default_to_follow_up_on_success: true,
                                can_attempt_resume_on_error: false,
                                is_queued_prompt: false,
                            },
                        );
                        assert!(controller.flush_pending_byop_request_after_finished_action(
                            conversation_id,
                            ctx
                        ));
                        let (summary_id, pending) = controller
                            .pending_byop_compaction_requests
                            .iter()
                            .find(|(_, pending)| {
                                pending.request.request_input.conversation_id == conversation_id
                            })
                            .expect("预检应确实触发自动摘要");
                        assert_eq!(pending.clear_input_on_success, !changed_context);
                        summary_id.clone()
                    });
                    (conversation_id, controller, input, context, summary_id)
                });
            // 模拟摘要后续发成功的通知，确认旧请求只能清理它真正消费的草稿上下文。
            controller.update(&mut app, |_, ctx| {
                ctx.emit(BlocklistAIControllerEvent::SubmittedCompactedInput {
                    stream_id: summary_id,
                    submitted: true,
                });
            });
            input.read(&app, |input, ctx| {
                assert_eq!(
                    input.buffer_text(ctx),
                    if changed_context {
                        "相同的原问题"
                    } else {
                        ""
                    }
                );
            });
            context.read(&app, |context, _| {
                assert_eq!(
                    context.pending_attachments().len(),
                    usize::from(changed_context)
                );
                assert_eq!(
                    context.pending_context_selected_text().map(String::as_str),
                    changed_context.then_some("后来添加的选区")
                );
            });
            controller.update(&mut app, |controller, ctx| {
                controller.cancel_conversation_progress(
                    conversation_id,
                    CancellationReason::ManuallyCancelled,
                    ctx,
                );
            });
        }
    });
}

#[test]
fn queued_request_does_not_clear_unsubmitted_draft_context() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let terminal = add_window_with_terminal(&mut app, None);
        let (input, context) = terminal.update(&mut app, |terminal, ctx| {
            let controller = terminal.ai_controller().clone();
            let input = terminal.input().clone();
            let context = controller.as_ref(ctx).context_model.clone();
            input.update(ctx, |input, ctx| {
                input.replace_buffer_content("还没发送的新草稿", ctx)
            });
            context.update(ctx, |context, ctx| {
                context.set_pending_context_selected_text(
                    Some("新草稿的选区".to_owned()),
                    true,
                    ctx,
                );
                context.append_pending_attachments(
                    vec![PendingAttachment::File(PendingFile {
                        file_name: "new-draft.txt".to_owned(),
                        file_path: "/tmp/new-draft.txt".into(),
                        mime_type: "text/plain".to_owned(),
                    })],
                    ctx,
                );
            });
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
            controller.update(ctx, |controller, ctx| {
                controller
                    .send_request_input(
                        request_input(conversation_id, root, query_input("已排队的旧问题")),
                        None,
                        true,
                        false,
                        true,
                        ctx,
                    )
                    .unwrap();
            });
            (input, context)
        });
        input.read(&app, |input, ctx| {
            assert_eq!(input.buffer_text(ctx), "还没发送的新草稿")
        });
        context.read(&app, |context, _| {
            assert_eq!(context.pending_attachments().len(), 1);
            assert_eq!(
                context.pending_context_selected_text().map(String::as_str),
                Some("新草稿的选区")
            );
        });
    });
}
