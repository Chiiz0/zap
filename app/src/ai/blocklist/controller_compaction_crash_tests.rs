//! 用独立测试进程和临时 SQLite 库验证强杀恢复，不启动真实 GUI 或读取用户会话。

use std::collections::HashMap;
use std::io::{BufRead as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use ::settings::Setting as _;
use command::blocking::Command;
use futures::channel::oneshot;
use serde_json::Value;
use warp_multi_agent_api as api;
use warpui::{App, SingletonEntity as _};

use super::*;
use crate::ai::agent::{AnyFileContent, FileContext, ImageContext};
use crate::ai::agent_providers::AgentProviderSecrets;
use crate::ai::blocklist::history_model::convert_persisted_conversation_to_ai_conversation_with_metadata;
use crate::ai::byop_compaction::state::CompactionState;
use crate::persistence::{ModelEvent, read_test_agent_conversation, start_test_writer};
use crate::settings::{AISettings, AgentProvider, AgentProviderApiType, AgentProviderModel};
use crate::terminal::general_settings::GeneralSettings;
use crate::terminal::view::load_ai_conversation::{
    RestoreConversationEntryBehavior, RestoredAIConversation,
};
use crate::test_util::ai_agent_tasks::{create_api_task, create_message};
use crate::test_util::terminal::{add_window_with_terminal, initialize_app_for_terminal_view};

const ROOT_TASK: &str = "crash-test-root";
const PROVIDER_ID: &str = "isolated-crash-test-provider";
const MODEL_ID: &str = "crash-test-model";
const QUERY: &str = "请分析这些附件中的强杀恢复问题";
const DB_ENV: &str = "INFINISHELL_BYOP_CRASH_TEST_DB";
const STAGE_ENV: &str = "INFINISHELL_BYOP_CRASH_TEST_STAGE";
const READY_PREFIX: &str = "BYOP_COMPACTION_CRASH_READY ";
const PARTIAL_SSE: &str = "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"测试摘要\"},\"finish_reason\":null}]}\n\n";
const COMPLETE_SSE: &str = concat!(
    "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"完整的测试摘要或恢复回答\"},\"finish_reason\":null}]}\n\n",
    "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":100,\"completion_tokens\":10,\"total_tokens\":110}}\n\n",
    "data: [DONE]\n\n",
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrashStage {
    DuringSummary,
    BeforeResumeAcknowledgement,
}

impl CrashStage {
    fn name(self) -> &'static str {
        match self {
            Self::DuringSummary => "during-summary",
            Self::BeforeResumeAcknowledgement => "before-resume-ack",
        }
    }
}

struct KillTestChild(Child);

impl Drop for KillTestChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn signal_ready(stage: CrashStage, conversation_id: &str) {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{READY_PREFIX}{} {conversation_id}", stage.name()).unwrap();
    stdout.flush().unwrap();
}

fn original_input() -> AIAgentInput {
    AIAgentInput::UserQuery {
        query: QUERY.to_owned(),
        context: Arc::from([
            AIAgentContext::SelectedText("选中的测试日志".to_owned()),
            AIAgentContext::Image(ImageContext {
                data: "aW1hZ2U=".to_owned(),
                mime_type: "image/png".to_owned(),
                file_name: "synthetic.png".to_owned(),
                is_figma: false,
            }),
            AIAgentContext::File(FileContext::new(
                "synthetic.txt".to_owned(),
                AnyFileContent::StringContent("完整的文本附件".to_owned()),
                None,
                None,
            )),
            AIAgentContext::File(FileContext::new(
                "synthetic.pdf".to_owned(),
                AnyFileContent::BinaryContent(vec![37, 80, 68, 70]),
                None,
                None,
            )),
        ]),
        referenced_attachments: HashMap::from([(
            "synthetic-reference".to_owned(),
            AIAgentAttachment::PlainText("引用附件的原始正文".to_owned()),
        )]),
        user_query_mode: UserQueryMode::Plan,
        static_query_type: None,
        running_command: None,
        intended_agent: None,
    }
}

fn history_messages() -> Vec<api::Message> {
    (0..4)
        .flat_map(|turn| {
            let mut user = create_message(&format!("history-user-{turn}"), ROOT_TASK);
            user.request_id = format!("history-request-{turn}");
            user.message = Some(message::Message::UserQuery(message::UserQuery {
                query: format!("历史问题{turn}"),
                ..Default::default()
            }));
            let mut output = create_message(&format!("history-output-{turn}"), ROOT_TASK);
            output.request_id = user.request_id.clone();
            output.message = Some(message::Message::AgentOutput(message::AgentOutput {
                text: "x".repeat(90_000),
            }));
            [user, output]
        })
        .collect()
}

fn request_for(
    conversation_id: AIConversationId,
    model_id: LLMId,
    input: AIAgentInput,
) -> RequestInput {
    RequestInput {
        conversation_id,
        input_messages: HashMap::from([(TaskId::new(ROOT_TASK.to_owned()), vec![input])]),
        working_directory: None,
        model_id: model_id.clone(),
        coding_model_id: model_id.clone(),
        cli_agent_model_id: model_id.clone(),
        computer_use_model_id: model_id,
        shared_session_response_initiator: None,
        request_start_ts: Local::now(),
        supported_tools_override: None,
    }
}

fn configure_provider(app: &mut App, base_url: &str) -> LLMId {
    let mut provider = AgentProvider::new_empty();
    provider.id = PROVIDER_ID.to_owned();
    provider.api_type = AgentProviderApiType::OpenAi;
    provider.base_url = format!("{base_url}/v1");
    let mut model = AgentProviderModel::from_id(MODEL_ID.to_owned());
    model.context_window = 200_000;
    model.image = Some(true);
    model.pdf = Some(true);
    provider.models.push(model);
    AISettings::handle(app).update(app, |settings, ctx| {
        settings
            .agent_providers
            .set_value(vec![provider], ctx)
            .unwrap();
        settings.byop_compaction_auto.set_value(true, ctx).unwrap();
    });
    // initialize_app_for_terminal_view 已注册 noop secure_storage，测试 key 不接触系统密钥库。
    AgentProviderSecrets::handle(app).update(app, |secrets, ctx| {
        secrets.set(PROVIDER_ID, "synthetic-test-key".to_owned(), ctx);
    });
    crate::ai::agent_providers::llm_id::encode(PROVIDER_ID, MODEL_ID)
}

fn start_checkpoint_relay(
    real_writer: mpsc::SyncSender<ModelEvent>,
    stage: CrashStage,
) -> mpsc::SyncSender<ModelEvent> {
    let (sender, receiver) = mpsc::sync_channel(1024);
    std::thread::spawn(move || {
        while let Ok(event) = receiver.recv_timeout(Duration::from_secs(60)) {
            if let ModelEvent::CheckpointMultiAgentConversation {
                conversation_id,
                revision,
                updated_tasks,
                conversation_data,
                completion,
            } = event
            {
                let state: CompactionState = serde_json::from_str(
                    conversation_data.compaction_state_json.as_deref().unwrap(),
                )
                .unwrap();
                let stop_before_ack = stage == CrashStage::BeforeResumeAcknowledgement
                    && state
                        .pending_recovery()
                        .is_some_and(|pending| pending.resumed_request_id.is_some());
                if stop_before_ack {
                    assert_eq!(state.completed().len(), 1, "续发检查点应包含已提交摘要");
                    let (proxy_completion, proxy_receiver) = oneshot::channel();
                    real_writer
                        .send(ModelEvent::CheckpointMultiAgentConversation {
                            conversation_id: conversation_id.clone(),
                            revision,
                            updated_tasks,
                            conversation_data,
                            completion: proxy_completion,
                        })
                        .unwrap();
                    assert_eq!(futures::executor::block_on(proxy_receiver).unwrap(), Ok(()));
                    signal_ready(stage, &conversation_id);
                    // 保存原 completion，父进程必须在确认到达客户端之前强杀。
                    std::thread::park_timeout(Duration::from_secs(60));
                    drop(completion);
                    return;
                }
                real_writer
                    .send(ModelEvent::CheckpointMultiAgentConversation {
                        conversation_id,
                        revision,
                        updated_tasks,
                        conversation_data,
                        completion,
                    })
                    .unwrap();
            } else {
                real_writer.send(event).unwrap();
            }
        }
    });
    sender
}

#[test]
#[ignore = "仅由下面两个强杀测试在独立子进程中启动"]
fn byop_compaction_crash_probe() {
    let Ok(stage_name) = std::env::var(STAGE_ENV) else {
        return;
    };
    let stage = match stage_name.as_str() {
        "during-summary" => CrashStage::DuringSummary,
        "before-resume-ack" => CrashStage::BeforeResumeAcknowledgement,
        value => panic!("未知强杀测试阶段：{value}"),
    };
    let database_path = PathBuf::from(std::env::var_os(DB_ENV).expect("必须指定独立测试数据库"));
    assert!(!database_path.exists(), "子进程只能初始化新的临时数据库");
    // 父测试异常退出时，子进程也不会无限保留线程或测试数据库句柄。
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(60));
        std::process::exit(97);
    });
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let writer = start_test_writer(&database_path).unwrap();
        let relay = start_checkpoint_relay(writer.sender.clone(), stage);
        GlobalResourceHandlesProvider::handle(&app).update(&mut app, |resources, _| {
            resources.set_model_event_sender_for_test(relay);
        });
        let conversation_for_signal = Arc::new(Mutex::new(None::<String>));
        let signal_id = conversation_for_signal.clone();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_header("content-type", "text/event-stream")
            .with_chunked_body(move |body| {
                if stage == CrashStage::DuringSummary {
                    body.write_all(PARTIAL_SSE.as_bytes())?;
                    body.flush()?;
                    signal_ready(stage, signal_id.lock().unwrap().as_deref().unwrap());
                    std::thread::park_timeout(Duration::from_secs(60));
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "父测试未及时强杀",
                    ));
                }
                body.write_all(COMPLETE_SSE.as_bytes())
            })
            .create();
        let model_id = configure_provider(&mut app, &server.url());
        let terminal = add_window_with_terminal(&mut app, None);
        terminal.update(&mut app, |terminal, ctx| {
            let conversation_id =
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let id =
                        history.start_new_conversation(terminal.id(), false, false, false, ctx);
                    history
                        .conversation_mut(&id)
                        .unwrap()
                        .upgrade_optimistic_root_to_server_task_for_test(create_api_task(
                            ROOT_TASK,
                            history_messages(),
                        ));
                    id
                });
            *conversation_for_signal.lock().unwrap() = Some(conversation_id.to_string());
            let request = request_for(conversation_id, model_id, original_input());
            terminal.ai_controller().update(ctx, |controller, ctx| {
                controller
                    .send_request_input_with_compaction(
                        request, None, true, false, false, true, None, None, ctx,
                    )
                    .unwrap();
                assert_eq!(
                    controller.pending_byop_compaction_requests.len(),
                    1,
                    "必须实际触发自动摘要"
                );
            });
        });
        futures::future::pending::<()>().await;
        drop(mock);
        drop(server);
        drop(writer);
    });
}

fn kill_at_checkpoint(stage: CrashStage, database_path: &Path) -> AIConversationId {
    let probe_name = format!(
        "{}::byop_compaction_crash_probe",
        module_path!().split_once("::").unwrap().1,
    );
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            &probe_name,
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(DB_ENV, database_path)
        .env(STAGE_ENV, stage.name())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut child = KillTestChild(child);
    let stdout = child.0.stdout.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines() {
            let line = line.unwrap();
            // libtest 的 `test ...` 前缀不换行，哨兵可能紧接在该前缀之后。
            if let Some((_, ready)) = line.split_once(READY_PREFIX) {
                let _ = sender.send(ready.to_owned());
                return;
            }
        }
    });
    let ready = receiver
        .recv_timeout(Duration::from_secs(45))
        .expect("子进程未在限时内到达指定的真实网络/事务阶段");
    let (actual_stage, conversation_id) = ready.split_once(' ').unwrap();
    assert_eq!(actual_stage, stage.name());
    child.0.kill().unwrap();
    assert!(
        !child.0.wait().unwrap().success(),
        "必须强杀，不能依赖正常退出刷盘"
    );
    reader.join().unwrap();
    AIConversationId::try_from(conversation_id.to_owned()).unwrap()
}

fn crash_and_resume(stage: CrashStage) {
    let tempdir = tempfile::tempdir().unwrap();
    let database_path = tempdir.path().join("isolated-crash.sqlite");
    let conversation_id = kill_at_checkpoint(stage, &database_path);
    let persisted = read_test_agent_conversation(&database_path, &conversation_id.to_string())
        .unwrap()
        .unwrap();
    let restored =
        convert_persisted_conversation_to_ai_conversation_with_metadata(persisted).unwrap();
    let pending = restored.compaction_state.pending_recovery().unwrap();
    let recovery_id = pending.id;
    assert_eq!(
        pending.queries[0].to_input(),
        original_input(),
        "强杀后问题、模式和全部附件必须完整"
    );
    assert_eq!(restored.status(), &ConversationStatus::Cancelled);
    assert_eq!(
        restored
            .all_exchanges()
            .into_iter()
            .filter(|exchange| exchange.id == recovery_id)
            .count(),
        1
    );
    assert_eq!(
        restored
            .all_linearized_messages()
            .into_iter()
            .filter(|message| message.id.starts_with("history-"))
            .cloned()
            .collect::<Vec<_>>(),
        history_messages()
    );
    assert_eq!(
        restored.compaction_state.completed().len(),
        usize::from(stage == CrashStage::BeforeResumeAcknowledgement)
    );
    assert_eq!(
        pending.resumed_request_id.is_some(),
        stage == CrashStage::BeforeResumeAcknowledgement
    );
    let second_load = read_test_agent_conversation(&database_path, &conversation_id.to_string())
        .unwrap()
        .unwrap();
    let second_load =
        convert_persisted_conversation_to_ai_conversation_with_metadata(second_load).unwrap();
    assert_eq!(
        second_load
            .all_exchanges()
            .into_iter()
            .filter(|exchange| exchange.id == recovery_id)
            .count(),
        1,
        "重复打开历史不能重复恢复问题"
    );

    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let writer = start_test_writer(&database_path).unwrap();
        GlobalResourceHandlesProvider::handle(&app).update(&mut app, |resources, _| {
            resources.set_model_event_sender_for_test(writer.sender.clone());
        });
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let received = requests.clone();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .match_request(move |request| {
                received
                    .lock()
                    .unwrap()
                    .push(serde_json::from_slice(request.body().unwrap()).unwrap());
                true
            })
            .with_header("content-type", "text/event-stream")
            .with_body(COMPLETE_SSE)
            .expect_at_least(1)
            .create();
        configure_provider(&mut app, &server.url());
        let terminal = add_window_with_terminal(&mut app, None);
        let controller = terminal.update(&mut app, |terminal, ctx| {
            terminal.restore_conversation_after_view_creation(
                RestoredAIConversation::new(restored),
                true,
                RestoreConversationEntryBehavior::EnterRestoredConversation,
                ctx,
            );
            let controller = terminal.ai_controller().clone();
            controller.update(ctx, |controller, ctx| {
                controller.resume_conversation(conversation_id, false, false, vec![], ctx);
            });
            controller
        });
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let completed = controller.read(&app, |controller, ctx| {
                let conversation = BlocklistAIHistoryModel::as_ref(ctx).conversation(&conversation_id).unwrap();
                !controller.has_active_stream_for_conversation(conversation_id, ctx)
                    && conversation.all_linearized_messages().iter().any(|message| matches!(
                        &message.message, Some(message::Message::UserQuery(query)) if query.query == QUERY
                    ))
            });
            if completed {
                break;
            }
            assert!(Instant::now() < deadline, "手动继续没有完成原始请求");
            warpui::r#async::Timer::after(Duration::from_millis(10)).await;
        }
        let saved = BlocklistAIHistoryModel::handle(&app).update(&mut app, |history, ctx| {
            history
                .conversation_mut(&conversation_id)
                .unwrap()
                .checkpoint_compaction_recovery_state(ctx)
                .unwrap()
        });
        assert_eq!(saved.unwrap().await.unwrap(), Ok(()));
        let outgoing = requests.lock().unwrap();
        let original_requests = outgoing
            .iter()
            .filter(|body| {
                body["messages"].as_array().unwrap().iter().any(|message| {
                    message["role"] == "user" && message["content"].to_string().contains(QUERY)
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(original_requests.len(), 1, "手动继续只能发送一次原始问题");
        let submitted = original_requests[0].to_string();
        assert!(submitted.contains("aW1hZ2U="), "续发必须携带原图片内容");
        assert!(
            submitted.contains("完整的文本附件"),
            "续发必须携带原文件正文"
        );
        drop(outgoing);
        mock.assert();

        let persisted = read_test_agent_conversation(&database_path, &conversation_id.to_string())
            .unwrap()
            .unwrap();
        let reloaded =
            convert_persisted_conversation_to_ai_conversation_with_metadata(persisted).unwrap();
        assert!(
            reloaded.compaction_state.pending_recovery().is_none(),
            "已持久化原请求不能再次待发"
        );
        assert_eq!(reloaded.all_linearized_messages().iter().filter(|message| matches!(
            &message.message, Some(message::Message::UserQuery(query)) if query.query == QUERY
        )).count(), 1);
        writer.sender.send(ModelEvent::Terminate).unwrap();
        writer.handle.join().unwrap();
    });
}

#[test]
fn byop_compaction_kill_during_summary_recovers_pending_question_and_attachments() {
    crash_and_resume(CrashStage::DuringSummary);
}

#[test]
fn byop_compaction_kill_after_commit_before_resume_ack_recovers_without_duplicate_send() {
    crash_and_resume(CrashStage::BeforeResumeAcknowledgement);
}

#[test]
fn byop_compaction_cancel_wins_when_checkpoint_ack_is_already_ready() {
    let tempdir = tempfile::tempdir().unwrap();
    let database_path = tempdir.path().join("cancel-checkpoint.sqlite");
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let writer = start_test_writer(&database_path).unwrap();
        let real_writer = writer.sender.clone();
        let (relay, events) = mpsc::sync_channel(1024);
        let (held_ack_sender, held_ack_receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut held = false;
            while let Ok(event) = events.recv_timeout(Duration::from_secs(30)) {
                if !held
                    && let ModelEvent::CheckpointMultiAgentConversation {
                        conversation_id,
                        revision,
                        updated_tasks,
                        conversation_data,
                        completion,
                    } = event
                {
                    held = true;
                    let (proxy_completion, proxy_receiver) = oneshot::channel();
                    real_writer
                        .send(ModelEvent::CheckpointMultiAgentConversation {
                            conversation_id,
                            revision,
                            updated_tasks,
                            conversation_data,
                            completion: proxy_completion,
                        })
                        .unwrap();
                    assert_eq!(futures::executor::block_on(proxy_receiver).unwrap(), Ok(()));
                    held_ack_sender.send(completion).unwrap();
                } else {
                    real_writer.send(event).unwrap();
                }
            }
        });
        GlobalResourceHandlesProvider::handle(&app).update(&mut app, |resources, _| {
            resources.set_model_event_sender_for_test(relay);
        });
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let received = requests.clone();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .match_request(move |request| {
                received
                    .lock()
                    .unwrap()
                    .push(serde_json::from_slice(request.body().unwrap()).unwrap());
                true
            })
            .with_header("content-type", "text/event-stream")
            .with_body(COMPLETE_SSE)
            .expect(1)
            .create();
        let model_id = configure_provider(&mut app, &server.url());
        let terminal = add_window_with_terminal(&mut app, None);
        let (conversation_id, controller) = terminal.update(&mut app, |terminal, ctx| {
            let conversation_id =
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let id =
                        history.start_new_conversation(terminal.id(), false, false, false, ctx);
                    history
                        .conversation_mut(&id)
                        .unwrap()
                        .upgrade_optimistic_root_to_server_task_for_test(create_api_task(
                            ROOT_TASK,
                            history_messages(),
                        ));
                    id
                });
            let controller = terminal.ai_controller().clone();
            controller.update(ctx, |controller, ctx| {
                controller
                    .send_request_input_with_compaction(
                        request_for(conversation_id, model_id.clone(), original_input()),
                        None,
                        true,
                        false,
                        false,
                        true,
                        None,
                        None,
                        ctx,
                    )
                    .unwrap();
                let acknowledgement = held_ack_receiver
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
                controller.cancel_conversation_progress(
                    conversation_id,
                    CancellationReason::ManuallyCancelled,
                    ctx,
                );
                // 未让 App 的异步发送任务获得首次 poll；此时取消与事务确认同时就绪。
                let _ = acknowledgement.send(Ok(()));
                assert!(
                    requests.lock().unwrap().is_empty(),
                    "检查点放行前不得发送 HTTP 请求"
                );
                let mut new_input = original_input();
                if let AIAgentInput::UserQuery { query, .. } = &mut new_input {
                    *query = "取消之后的新问题".to_owned();
                }
                controller
                    .send_request_input_with_compaction(
                        request_for(conversation_id, model_id, new_input),
                        None,
                        true,
                        false,
                        false,
                        false,
                        None,
                        None,
                        ctx,
                    )
                    .unwrap();
            });
            (conversation_id, controller)
        });
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if controller.read(&app, |controller, ctx| {
                !controller.has_active_stream_for_conversation(conversation_id, ctx)
            }) {
                break;
            }
            assert!(Instant::now() < deadline, "后续新请求未正常结束");
            warpui::r#async::Timer::after(Duration::from_millis(10)).await;
        }
        mock.assert();
        let outgoing = requests.lock().unwrap();
        assert_eq!(outgoing.len(), 1, "被取消的摘要必须始终发送零次 HTTP 请求");
        assert!(outgoing[0].to_string().contains("取消之后的新问题"));
        BlocklistAIHistoryModel::handle(&app).read(&app, |history, _| {
            assert_eq!(
                history.conversation(&conversation_id).unwrap().status(),
                &ConversationStatus::Success,
                "旧流确认迟到不得覆盖后续新请求状态"
            );
        });
        writer.sender.send(ModelEvent::Terminate).unwrap();
        writer.handle.join().unwrap();
    });
}

#[test]
fn byop_compaction_explicitly_disabled_persistence_still_sends_summary_and_original_query() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        GeneralSettings::handle(&app).update(&mut app, |settings, ctx| {
            settings
                .persist_conversations
                .set_value(false, ctx)
                .unwrap();
        });
        app.update(|ctx| {
            assert!(
                GlobalResourceHandlesProvider::as_ref(ctx)
                    .get()
                    .model_event_sender
                    .is_none()
            );
        });
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_header("content-type", "text/event-stream")
            .with_body(COMPLETE_SSE)
            .expect(2)
            .create();
        let model_id = configure_provider(&mut app, &server.url());
        let terminal = add_window_with_terminal(&mut app, None);
        let (conversation_id, controller) = terminal.update(&mut app, |terminal, ctx| {
            let conversation_id =
                BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                    let id =
                        history.start_new_conversation(terminal.id(), false, false, false, ctx);
                    history
                        .conversation_mut(&id)
                        .unwrap()
                        .upgrade_optimistic_root_to_server_task_for_test(create_api_task(
                            ROOT_TASK,
                            history_messages(),
                        ));
                    id
                });
            let controller = terminal.ai_controller().clone();
            controller.update(ctx, |controller, ctx| {
                controller
                    .send_request_input_with_compaction(
                        request_for(conversation_id, model_id, original_input()),
                        None,
                        true,
                        false,
                        false,
                        true,
                        None,
                        None,
                        ctx,
                    )
                    .unwrap();
                assert_eq!(controller.pending_byop_compaction_requests.len(), 1);
            });
            (conversation_id, controller)
        });
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let finished = controller.read(&app, |controller, ctx| {
                let conversation = BlocklistAIHistoryModel::as_ref(ctx).conversation(&conversation_id).unwrap();
                !controller.has_active_stream_for_conversation(conversation_id, ctx)
                    && conversation.all_linearized_messages().iter().any(|message| matches!(
                        &message.message, Some(message::Message::UserQuery(query)) if query.query == QUERY
                    ))
            });
            if finished {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "主动关闭持久化不能阻断摘要或原请求"
            );
            warpui::r#async::Timer::after(Duration::from_millis(10)).await;
        }
        mock.assert();
        BlocklistAIHistoryModel::handle(&app).read(&app, |history, _| {
            let conversation = history.conversation(&conversation_id).unwrap();
            assert_eq!(conversation.compaction_state.completed().len(), 1);
            assert_eq!(conversation.status(), &ConversationStatus::Success);
        });
    });
}

#[test]
fn byop_compaction_missing_writer_blocks_http_and_preserves_input_with_save_error() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        GeneralSettings::handle(&app).update(&mut app, |settings, ctx| {
            settings.persist_conversations.set_value(true, ctx).unwrap();
        });
        app.update(|ctx| {
            assert!(
                GlobalResourceHandlesProvider::as_ref(ctx)
                    .get()
                    .model_event_sender
                    .is_none()
            );
        });
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_header("content-type", "text/event-stream")
            .with_body(COMPLETE_SSE)
            .expect(0)
            .create();
        let model_id = configure_provider(&mut app, &server.url());
        let terminal = add_window_with_terminal(&mut app, None);
        terminal.update(&mut app, |terminal, ctx| {
            let conversation_id = BlocklistAIHistoryModel::handle(ctx).update(ctx, |history, ctx| {
                let id = history.start_new_conversation(terminal.id(), false, false, false, ctx);
                history.conversation_mut(&id).unwrap().upgrade_optimistic_root_to_server_task_for_test(
                    create_api_task(ROOT_TASK, history_messages()),
                );
                id
            });
            terminal.ai_controller().update(ctx, |controller, ctx| {
                controller.send_request_input_with_compaction(
                    request_for(conversation_id, model_id, original_input()),
                    None, true, false, false, true, None, None, ctx,
                ).unwrap();
                assert!(!controller.has_active_stream_for_conversation(conversation_id, ctx));
                assert!(controller.pending_byop_compaction_requests.is_empty());
            });
            BlocklistAIHistoryModel::handle(ctx).read(ctx, |history, _| {
                let conversation = history.conversation(&conversation_id).unwrap();
                let exchange = conversation.latest_visible_exchange().unwrap();
                assert_eq!(exchange.input, vec![original_input()]);
                assert!(!conversation.is_exchange_hidden(exchange.id));
                assert!(matches!(&exchange.output_status, AIAgentOutputStatus::Finished {
                    finished_output: FinishedAIAgentOutput::Error {
                        error: RenderableAIError::Other { error_message, will_attempt_resume: false, .. }, ..
                    }
                } if error_message == &crate::t!("ai-error-compaction-save-failed")));
            });
        });
        mock.assert();
    });
}
