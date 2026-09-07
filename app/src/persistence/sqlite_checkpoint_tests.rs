use diesel::Connection;
use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use futures::channel::oneshot;
use futures::executor::block_on;
use serde_json::json;
use warp_multi_agent_api as api;

use super::{
    MAX_TASK_BLOB_BYTES, deduplicate_events, handle_model_event, setup_database, start_writer,
};
use crate::app_state::AppState;
use crate::persistence::agent::read_agent_conversation_by_id;
use crate::persistence::model::AgentConversationData;
use crate::persistence::{ModelEvent, next_conversation_write_revision};

fn task(text: &str) -> api::Task {
    api::Task {
        id: "checkpoint-task".to_owned(),
        messages: vec![api::Message {
            id: "checkpoint-message".to_owned(),
            task_id: "checkpoint-task".to_owned(),
            request_id: "checkpoint-request".to_owned(),
            message: Some(api::message::Message::UserQuery(api::message::UserQuery {
                query: text.to_owned(),
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn checkpoint(
    tasks: Vec<api::Task>,
    name: &str,
) -> (ModelEvent, oneshot::Receiver<Result<(), String>>) {
    let (completion, receiver) = oneshot::channel();
    let conversation_data = serde_json::from_value(json!({
        "server_conversation_token": null,
        "agent_name": name,
    }))
    .unwrap();
    (
        ModelEvent::CheckpointMultiAgentConversation {
            conversation_id: "checkpoint-conversation".to_owned(),
            revision: next_conversation_write_revision(),
            updated_tasks: tasks,
            conversation_data,
            completion,
        },
        receiver,
    )
}

#[test]
fn checkpoint_acknowledges_only_after_snapshot_is_visible_to_another_connection() {
    let tempdir = tempfile::tempdir().unwrap();
    let database_path = tempdir.path().join("warp.sqlite");
    let conn = setup_database(&database_path).unwrap();
    let writer = start_writer(conn, database_path.clone()).unwrap();
    let expected = task("摘要前必须保留的原始输入");
    let (event, completion) = checkpoint(vec![expected.clone()], "已提交");

    writer.sender.send(event).unwrap();
    assert_eq!(block_on(completion).unwrap(), Ok(()));

    let mut reader = SqliteConnection::establish(database_path.to_str().unwrap()).unwrap();
    let restored = read_agent_conversation_by_id(&mut reader, "checkpoint-conversation")
        .unwrap()
        .unwrap();
    assert_eq!(restored.tasks, [expected]);
    let restored_data: AgentConversationData =
        serde_json::from_str(&restored.conversation.conversation_data).unwrap();
    assert_eq!(restored_data.agent_name.as_deref(), Some("已提交"));
    drop(reader);
    writer.sender.send(ModelEvent::Terminate).unwrap();
    writer.handle.join().unwrap();
}

#[test]
fn failed_checkpoint_rolls_back_metadata_and_preserves_the_previous_tasks() {
    let tempdir = tempfile::tempdir().unwrap();
    let database_path = tempdir.path().join("warp.sqlite");
    let mut conn = setup_database(&database_path).unwrap();
    let original = task("已保存的原始输入");
    let (event, completion) = checkpoint(vec![original.clone()], "旧元数据");
    handle_model_event(event, &mut conn).unwrap();
    assert_eq!(block_on(completion).unwrap(), Ok(()));
    conn.batch_execute(
        "CREATE TRIGGER reject_checkpoint BEFORE INSERT ON agent_tasks \
         BEGIN SELECT RAISE(ABORT, 'checkpoint rejected'); END;",
    )
    .unwrap();
    let (event, completion) = checkpoint(vec![task("不能提交的新输入")], "新元数据");

    assert!(handle_model_event(event, &mut conn).is_err());
    assert!(block_on(completion).unwrap().is_err());

    let mut reader = SqliteConnection::establish(database_path.to_str().unwrap()).unwrap();
    let restored = read_agent_conversation_by_id(&mut reader, "checkpoint-conversation")
        .unwrap()
        .unwrap();
    assert_eq!(restored.tasks, [original]);
    let restored_data: AgentConversationData =
        serde_json::from_str(&restored.conversation.conversation_data).unwrap();
    assert_eq!(restored_data.agent_name.as_deref(), Some("旧元数据"));
}

#[test]
fn oversized_task_cannot_receive_a_successful_checkpoint_acknowledgement() {
    let tempdir = tempfile::tempdir().unwrap();
    let mut conn = setup_database(&tempdir.path().join("warp.sqlite")).unwrap();
    let original = task("完整的旧快照");
    let (event, completion) = checkpoint(vec![original.clone()], "旧元数据");
    handle_model_event(event, &mut conn).unwrap();
    assert_eq!(block_on(completion).unwrap(), Ok(()));
    let (event, completion) = checkpoint(vec![task(&"x".repeat(MAX_TASK_BLOB_BYTES))], "新元数据");

    assert!(handle_model_event(event, &mut conn).is_err());
    assert!(block_on(completion).unwrap().unwrap_err().contains("上限"));

    let restored = read_agent_conversation_by_id(&mut conn, "checkpoint-conversation")
        .unwrap()
        .unwrap();
    assert_eq!(restored.tasks, [original]);
    let restored_data: AgentConversationData =
        serde_json::from_str(&restored.conversation.conversation_data).unwrap();
    assert_eq!(restored_data.agent_name.as_deref(), Some("旧元数据"));
}

#[test]
fn checkpoint_rejects_a_snapshot_whose_tasks_were_removed_during_the_transaction() {
    let tempdir = tempfile::tempdir().unwrap();
    let mut conn = setup_database(&tempdir.path().join("warp.sqlite")).unwrap();
    conn.batch_execute(
        "CREATE TRIGGER remove_checkpoint_task AFTER INSERT ON agent_tasks \
         BEGIN DELETE FROM agent_tasks WHERE task_id = NEW.task_id; END;",
    )
    .unwrap();
    let (event, completion) = checkpoint(vec![task("不能遗漏的任务")], "快照");

    assert!(handle_model_event(event, &mut conn).is_err());
    assert!(block_on(completion).unwrap().is_err());
    assert!(
        read_agent_conversation_by_id(&mut conn, "checkpoint-conversation")
            .unwrap()
            .is_none()
    );
}

#[test]
fn paused_writer_rejects_checkpoint_instead_of_acknowledging_it() {
    let tempdir = tempfile::tempdir().unwrap();
    let database_path = tempdir.path().join("warp.sqlite");
    let conn = setup_database(&database_path).unwrap();
    let writer = start_writer(conn, database_path).unwrap();
    let (event, completion) = checkpoint(vec![task("暂停期间不能保存")], "快照");

    writer
        .sender
        .send(ModelEvent::PauseAndRemoveDatabase)
        .unwrap();
    writer.sender.send(event).unwrap();
    assert_eq!(
        block_on(completion).unwrap(),
        Err("SQLite 写入器已暂停".to_owned())
    );

    writer.sender.send(ModelEvent::Terminate).unwrap();
    writer.handle.join().unwrap();
}

#[test]
fn closed_writer_cancels_checkpoint_acknowledgement() {
    let tempdir = tempfile::tempdir().unwrap();
    let database_path = tempdir.path().join("warp.sqlite");
    let conn = setup_database(&database_path).unwrap();
    let writer = start_writer(conn, database_path).unwrap();
    writer.sender.send(ModelEvent::Terminate).unwrap();
    writer.handle.join().unwrap();
    let (event, completion) = checkpoint(vec![task("关闭后不能保存")], "快照");

    let failed_send = writer.sender.send(event).unwrap_err();
    drop(failed_send);

    assert!(block_on(completion).is_err());
}

#[test]
fn snapshot_deduplication_preserves_each_checkpoint_and_its_order() {
    let tempdir = tempfile::tempdir().unwrap();
    let mut conn = setup_database(&tempdir.path().join("warp.sqlite")).unwrap();
    let (first, first_completion) = checkpoint(vec![task("第一份输入")], "第一份");
    let (second, second_completion) = checkpoint(vec![task("第二份输入")], "第二份");
    let snapshot = AppState {
        windows: vec![],
        active_window_index: None,
        block_lists: Default::default(),
        running_mcp_servers: Default::default(),
    };
    let mut events = deduplicate_events(vec![
        ModelEvent::Snapshot(snapshot.clone()),
        first,
        ModelEvent::Snapshot(snapshot.clone()),
        second,
        ModelEvent::Snapshot(snapshot),
    ]);

    assert_eq!(events.len(), 3);
    handle_model_event(events.remove(0), &mut conn).unwrap();
    assert_eq!(block_on(first_completion).unwrap(), Ok(()));
    let first_restored = read_agent_conversation_by_id(&mut conn, "checkpoint-conversation")
        .unwrap()
        .unwrap();
    assert_eq!(first_restored.tasks, [task("第一份输入")]);
    handle_model_event(events.remove(0), &mut conn).unwrap();
    assert_eq!(block_on(second_completion).unwrap(), Ok(()));
    let second_restored = read_agent_conversation_by_id(&mut conn, "checkpoint-conversation")
        .unwrap()
        .unwrap();
    assert_eq!(second_restored.tasks, [task("第二份输入")]);
    assert!(matches!(events[0], ModelEvent::Snapshot(_)));
}

#[test]
fn checkpoint_pending_record_and_summary_survive_a_late_older_snapshot() {
    let tempdir = tempfile::tempdir().unwrap();
    let database_path = tempdir.path().join("warp.sqlite");
    let conn = setup_database(&database_path).unwrap();
    let writer = start_writer(conn, database_path.clone()).unwrap();
    let old_snapshot = ModelEvent::UpdateMultiAgentConversation {
        conversation_id: "checkpoint-conversation".to_owned(),
        revision: next_conversation_write_revision(),
        updated_tasks: vec![task("旧历史")],
        conversation_data: serde_json::from_value(json!({
            "server_conversation_token": null,
        }))
        .unwrap(),
    };
    let expected = task("摘要后最新历史");
    let (mut event, completion) = checkpoint(vec![expected.clone()], "最新快照");
    // 持久化层把 sidecar 视为不透明数据，必须逐字保留恢复记录及摘要。
    let sidecar = json!({
        "pending_recovery": {"query": "尚未续发的原问题"},
        "completed": [{"summary_text": "已提交摘要"}],
    })
    .to_string();
    let ModelEvent::CheckpointMultiAgentConversation {
        conversation_data, ..
    } = &mut event
    else {
        panic!("需要带确认的快照事件");
    };
    conversation_data.compaction_state_json = Some(sidecar.clone());

    writer.sender.send(event).unwrap();
    assert_eq!(block_on(completion).unwrap(), Ok(()));
    writer.sender.send(old_snapshot).unwrap();
    writer.sender.send(ModelEvent::Terminate).unwrap();
    writer.handle.join().unwrap();

    let restored = super::read_test_agent_conversation(&database_path, "checkpoint-conversation")
        .unwrap()
        .unwrap();
    assert_eq!(restored.tasks, [expected]);
    let restored_data: AgentConversationData =
        serde_json::from_str(&restored.conversation.conversation_data).unwrap();
    assert_eq!(restored_data.compaction_state_json, Some(sidecar));
    assert_eq!(restored_data.agent_name.as_deref(), Some("最新快照"));
}

#[test]
fn checkpoint_superseded_by_a_newer_snapshot_receives_an_error() {
    let tempdir = tempfile::tempdir().unwrap();
    let database_path = tempdir.path().join("warp.sqlite");
    let conn = setup_database(&database_path).unwrap();
    let writer = start_writer(conn, database_path.clone()).unwrap();
    let (old_checkpoint, completion) = checkpoint(vec![task("旧待发输入")], "旧快照");
    let expected = task("后来提交的新请求");
    let newer_snapshot = ModelEvent::UpdateMultiAgentConversation {
        conversation_id: "checkpoint-conversation".to_owned(),
        revision: next_conversation_write_revision(),
        updated_tasks: vec![expected.clone()],
        conversation_data: serde_json::from_value(json!({
            "server_conversation_token": null,
            "agent_name": "新请求",
        }))
        .unwrap(),
    };

    writer.sender.send(newer_snapshot).unwrap();
    writer.sender.send(old_checkpoint).unwrap();
    assert_eq!(
        block_on(completion).unwrap(),
        Err("会话快照已被更新的状态取代".to_owned())
    );

    let restored = super::read_test_agent_conversation(&database_path, "checkpoint-conversation")
        .unwrap()
        .unwrap();
    assert_eq!(restored.tasks, [expected]);
    writer.sender.send(ModelEvent::Terminate).unwrap();
    writer.handle.join().unwrap();
}

#[test]
fn failed_newer_checkpoint_does_not_suppress_an_older_valid_snapshot() {
    let tempdir = tempfile::tempdir().unwrap();
    let database_path = tempdir.path().join("warp.sqlite");
    let conn = setup_database(&database_path).unwrap();
    let writer = start_writer(conn, database_path.clone()).unwrap();
    let expected = task("仍可恢复的完整输入");
    let (valid, valid_completion) = checkpoint(vec![expected.clone()], "完整快照");
    let (oversized, oversized_completion) =
        checkpoint(vec![task(&"x".repeat(MAX_TASK_BLOB_BYTES))], "无法完整保存");

    writer.sender.send(oversized).unwrap();
    assert!(block_on(oversized_completion).unwrap().is_err());
    writer.sender.send(valid).unwrap();
    assert_eq!(block_on(valid_completion).unwrap(), Ok(()));

    let restored = super::read_test_agent_conversation(&database_path, "checkpoint-conversation")
        .unwrap()
        .unwrap();
    assert_eq!(restored.tasks, [expected]);
    writer.sender.send(ModelEvent::Terminate).unwrap();
    writer.handle.join().unwrap();
}
