use super::*;
use crate::ai::agent::conversation::{AIConversation, AIConversationId};
use crate::ai::agent_providers::chat_stream::collect_linearized_task_messages;
use crate::ai::byop_compaction::commit::commit_summarization;
use crate::test_util::ai_agent_tasks::{
    create_api_subtask, create_api_task, create_subagent_tool_call_message,
};

fn user(id: &str) -> api::Message {
    api::Message {
        id: id.to_owned(),
        task_id: "root".to_owned(),
        request_id: format!("request-{id}"),
        message: Some(api::message::Message::UserQuery(api::message::UserQuery {
            query: format!("问题-{id}"),
            ..Default::default()
        })),
        ..Default::default()
    }
}

fn output(id: &str, request_id: &str, text: &str) -> api::Message {
    api::Message {
        id: id.to_owned(),
        task_id: "root".to_owned(),
        request_id: request_id.to_owned(),
        message: Some(api::message::Message::AgentOutput(
            api::message::AgentOutput {
                text: text.to_owned(),
            },
        )),
        ..Default::default()
    }
}

fn conversation(messages: Vec<api::Message>) -> AIConversation {
    AIConversation::new_restored(
        AIConversationId::new(),
        vec![api::Task {
            id: "root".to_owned(),
            messages,
            ..Default::default()
        }],
        None,
    )
    .unwrap()
}

fn prepare(messages: &[api::Message]) -> CompactionPlan {
    prepare_plan(
        &messages.iter().collect::<Vec<_>>(),
        &CompactionState::default(),
        &CompactionConfig {
            tail_turns: 1,
            ..Default::default()
        },
        ModelLimit::FALLBACK,
        |_| true,
    )
    .unwrap()
}

#[test]
fn empty_response_never_commits_an_old_answer() {
    let messages = vec![user("u1"), output("old", "old-request", "旧回答")];
    let plan = prepare(&messages);
    let mut conversation = conversation(messages);

    assert!(!commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        true
    ));
    assert!(conversation.compaction_state.completed().is_empty());
}

#[test]
fn whitespace_and_other_requests_are_not_a_summary() {
    let messages = vec![user("u1"), output("old", "old-request", "旧回答")];
    let plan = prepare(&messages);
    let mut current = messages;
    current.push(output("empty", "summary-request", " \n\t "));
    current.push(output("unrelated", "another-request", "另一流的回答"));
    let mut conversation = conversation(current);

    assert!(!commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        false
    ));
    assert!(
        conversation
            .compaction_state
            .hidden_message_ids()
            .is_empty()
    );
}

#[test]
fn fixed_head_preserves_tail_and_messages_added_during_summarization() {
    let messages = vec![
        user("u1"),
        output("a1", "r1", "旧历史"),
        user("u2"),
        output("a2", "r2", "需要保留的最近回答"),
    ];
    let plan = prepare(&messages);
    let mut current = messages;
    current.push(output("summary", "summary-request", "本次摘要"));
    current.push(user("new-user"));
    current.push(output("new-answer", "new-request", "较晚的另一条回答"));
    let mut conversation = conversation(current);

    assert!(commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        true
    ));

    let completed = conversation.compaction_state.completed().last().unwrap();
    assert_eq!(completed.head_message_ids, ["u1", "a1"]);
    assert_eq!(completed.user_msg_id, "u1");
    assert_eq!(completed.tail_start_id.as_deref(), Some("u2"));
    assert_eq!(completed.summary_text.as_deref(), Some("本次摘要"));
    let hidden = conversation.compaction_state.hidden_message_ids();
    assert!(!hidden.contains("u2"));
    assert!(!hidden.contains("a2"));
    assert!(!hidden.contains("new-user"));
    assert!(!hidden.contains("new-answer"));
}

#[test]
fn changed_source_content_prevents_commit() {
    let messages = vec![user("u1"), output("a1", "r1", "旧内容")];
    let plan = prepare(&messages);
    let mut conversation = conversation(vec![
        user("u1"),
        output("a1", "r1", "已改变的内容"),
        output("summary", "summary-request", "旧内容的摘要"),
    ]);

    assert!(!commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        true
    ));
    assert!(conversation.compaction_state.completed().is_empty());
}

#[test]
fn missing_selected_message_prevents_commit() {
    let plan = prepare(&[user("u1"), output("a1", "r1", "旧内容")]);
    let mut conversation = conversation(vec![
        user("u1"),
        output("summary", "summary-request", "摘要"),
    ]);

    assert!(!commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        true
    ));
}

#[test]
fn new_messages_inserted_inside_the_head_prevent_commit() {
    let plan = prepare(&[user("u1"), output("a1", "r1", "旧内容")]);
    let mut conversation = conversation(vec![
        user("u1"),
        user("inserted"),
        output("a1", "r1", "旧内容"),
        output("summary", "summary-request", "摘要"),
    ]);

    assert!(!commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        true
    ));
}

#[test]
fn duplicate_finished_events_do_not_commit_twice() {
    let messages = vec![user("u1"), output("a1", "r1", "旧内容")];
    let plan = prepare(&messages);
    let mut current = messages;
    current.push(output("summary", "summary-request", "摘要"));
    let mut conversation = conversation(current);

    assert!(commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        true
    ));
    assert!(!commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        true
    ));
    assert_eq!(conversation.compaction_state.completed().len(), 1);
}

#[test]
fn all_current_summary_parts_and_reasoning_carriers_are_hidden() {
    let messages = vec![user("u1"), output("a1", "r1", "旧内容")];
    let plan = prepare(&messages);
    let mut current = messages;
    current.push(api::Message {
        id: "reasoning".to_owned(),
        task_id: "root".to_owned(),
        request_id: "summary-request".to_owned(),
        message: Some(api::message::Message::AgentReasoning(
            api::message::AgentReasoning {
                reasoning: "摘要思考".to_owned(),
                ..Default::default()
            },
        )),
        ..Default::default()
    });
    current.push(output("summary-1", "summary-request", "第一部分"));
    current.push(output("summary-2", "summary-request", "第二部分"));
    let mut conversation = conversation(current);

    assert!(commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        true
    ));

    let completed = conversation.compaction_state.completed().last().unwrap();
    assert_eq!(
        completed.summary_text.as_deref(),
        Some("第一部分\n\n第二部分")
    );
    assert_eq!(
        completed.summary_message_ids,
        ["reasoning", "summary-1", "summary-2"]
    );
    assert!(
        conversation
            .compaction_state
            .hidden_message_ids()
            .contains("reasoning")
    );
}

#[test]
fn plan_preserves_input_order_despite_timestamp_order() {
    let mut first = user("first");
    first.timestamp = Some(prost_types::Timestamp {
        seconds: 20,
        nanos: 0,
    });
    let mut second = user("second");
    second.timestamp = Some(prost_types::Timestamp {
        seconds: 10,
        nanos: 0,
    });
    let messages = vec![first, output("a1", "r1", "第一轮回答"), second];

    let plan = prepare(&messages);

    assert_eq!(plan.head_message_ids, ["first", "a1"]);
    assert_eq!(plan.tail_start_id.as_deref(), Some("second"));
    assert_eq!(plan.head_end(&messages.iter().collect::<Vec<_>>()), Some(2));
}

#[test]
fn request_budget_shrinks_only_complete_turns() {
    let messages = [
        user("u1"),
        output("a1", "r1", "第一轮"),
        user("u2"),
        output("a2", "r2", "第二轮"),
        user("u3"),
        output("a3", "r3", "第三轮"),
    ];
    let plan = prepare_plan(
        &messages.iter().collect::<Vec<_>>(),
        &CompactionState::default(),
        &CompactionConfig {
            tail_turns: 0,
            ..Default::default()
        },
        ModelLimit::FALLBACK,
        |plan| plan.head_message_ids.len() <= 3,
    )
    .unwrap();

    assert_eq!(plan.head_message_ids, ["u1", "a1"]);
    assert_eq!(plan.tail_start_id.as_deref(), Some("u2"));
}

#[test]
fn no_fitting_complete_turn_produces_no_plan() {
    let messages = [user("u1"), output("a1", "r1", "大内容")];

    let plan = prepare_plan(
        &messages.iter().collect::<Vec<_>>(),
        &CompactionState::default(),
        &CompactionConfig::default(),
        ModelLimit::FALLBACK,
        |_| false,
    );

    assert!(plan.is_none());
}

#[test]
fn oversized_single_turn_compacts_as_a_whole_when_request_fits() {
    let messages = [
        api::Message {
            message: Some(api::message::Message::UserQuery(api::message::UserQuery {
                query: "x".repeat(40_000),
                ..Default::default()
            })),
            ..user("u1")
        },
        output("a1", "r1", "短回答"),
    ];

    let plan = prepare_plan(
        &messages.iter().collect::<Vec<_>>(),
        &CompactionState::default(),
        &CompactionConfig::default(),
        ModelLimit::FALLBACK,
        |_| true,
    )
    .unwrap();

    assert_eq!(plan.head_message_ids, ["u1", "a1"]);
    assert_eq!(plan.tail_start_id, None);
}

#[test]
fn expanding_a_split_turn_keeps_tool_pairs_and_preserves_later_turns() {
    let messages = [
        api::Message {
            message: Some(api::message::Message::UserQuery(api::message::UserQuery {
                query: "x".repeat(40_000),
                ..Default::default()
            })),
            ..user("u1")
        },
        api::Message {
            id: "call".to_owned(),
            task_id: "root".to_owned(),
            message: Some(api::message::Message::ToolCall(api::message::ToolCall {
                tool_call_id: "tool-1".to_owned(),
                ..Default::default()
            })),
            ..Default::default()
        },
        user("u2"),
        api::Message {
            id: "result".to_owned(),
            task_id: "root".to_owned(),
            message: Some(api::message::Message::ToolCallResult(
                api::message::ToolCallResult {
                    tool_call_id: "tool-1".to_owned(),
                    ..Default::default()
                },
            )),
            ..Default::default()
        },
        user("u3"),
        output("a3", "r3", "保留最近回答"),
    ];

    let plan = prepare_plan(
        &messages.iter().collect::<Vec<_>>(),
        &CompactionState::default(),
        &CompactionConfig {
            tail_turns: 3,
            preserve_recent_tokens: Some(100),
            ..Default::default()
        },
        ModelLimit::FALLBACK,
        |_| true,
    )
    .unwrap();

    assert_eq!(plan.head_message_ids, ["u1", "call", "u2", "result"]);
    assert_eq!(plan.tail_start_id.as_deref(), Some("u3"));
}

#[test]
fn expanding_a_split_turn_still_requires_request_to_fit() {
    let messages = [
        api::Message {
            message: Some(api::message::Message::UserQuery(api::message::UserQuery {
                query: "x".repeat(40_000),
                ..Default::default()
            })),
            ..user("u1")
        },
        output("a1", "r1", "短回答"),
    ];

    let plan = prepare_plan(
        &messages.iter().collect::<Vec<_>>(),
        &CompactionState::default(),
        &CompactionConfig::default(),
        ModelLimit::FALLBACK,
        |_| false,
    );

    assert!(plan.is_none());
}

#[test]
fn shrinking_never_separates_a_tool_call_from_its_result() {
    let messages = [
        user("u1"),
        api::Message {
            id: "call".to_owned(),
            task_id: "root".to_owned(),
            message: Some(api::message::Message::ToolCall(api::message::ToolCall {
                tool_call_id: "tool-1".to_owned(),
                ..Default::default()
            })),
            ..Default::default()
        },
        user("u2"),
        api::Message {
            id: "result".to_owned(),
            task_id: "root".to_owned(),
            message: Some(api::message::Message::ToolCallResult(
                api::message::ToolCallResult {
                    tool_call_id: "tool-1".to_owned(),
                    ..Default::default()
                },
            )),
            ..Default::default()
        },
        user("u3"),
    ];
    let plan = prepare_plan(
        &messages.iter().collect::<Vec<_>>(),
        &CompactionState::default(),
        &CompactionConfig {
            tail_turns: 0,
            ..Default::default()
        },
        ModelLimit::FALLBACK,
        |plan| plan.head_message_ids.len() <= 2,
    );

    assert!(plan.is_none());
}

#[test]
fn new_plan_uses_previous_summary_and_requires_new_history() {
    let messages = vec![user("u1"), output("a1", "r1", "旧内容")];
    let first_plan = prepare(&messages);
    let mut current = messages;
    current.push(output("summary", "summary-request", "第一次摘要"));
    let mut original = conversation(current.clone());
    assert!(commit_summarization(
        &mut original,
        &first_plan,
        "summary-request",
        true
    ));
    let state = original.compaction_state.clone();

    let empty_plan = prepare_plan(
        &current.iter().collect::<Vec<_>>(),
        &state,
        &CompactionConfig::default(),
        ModelLimit::FALLBACK,
        |_| true,
    );
    assert!(empty_plan.is_none());

    current.push(user("u2"));
    current.push(output("a2", "r2", "新增信息"));
    let next_plan = prepare_plan(
        &current.iter().collect::<Vec<_>>(),
        &state,
        &CompactionConfig::default(),
        ModelLimit::FALLBACK,
        |_| true,
    )
    .unwrap();
    assert_eq!(next_plan.previous_summary.as_deref(), Some("第一次摘要"));

    current.push(output(
        "summary-2",
        "summary-request-2",
        "第一次摘要和新增信息",
    ));
    let mut updated = conversation(current);
    updated.compaction_state = state;
    assert!(commit_summarization(
        &mut updated,
        &next_plan,
        "summary-request-2",
        true
    ));
    assert_eq!(
        updated.compaction_state.previous_summary(),
        Some("第一次摘要和新增信息")
    );
}

#[test]
fn real_subtask_history_commits_using_serializer_order_and_user_deduplication() {
    let root_user = user("u1");
    let mut copied_user = root_user.clone();
    copied_user.id = "child-user-copy".to_owned();
    copied_user.task_id = "child".to_owned();
    copied_user.timestamp = Some(prost_types::Timestamp {
        seconds: 1,
        nanos: 0,
    });
    let mut child_answer = output("child-answer", "child-request", "子任务结果");
    child_answer.task_id = "child".to_owned();
    child_answer.timestamp = Some(prost_types::Timestamp {
        seconds: 2,
        nanos: 0,
    });
    let mut root = create_api_task(
        "root",
        vec![
            root_user,
            create_subagent_tool_call_message("spawn", "root", "child", None),
            output("root-answer", "root-request", "根任务结果"),
            user("tail-user"),
            output("tail-answer", "tail-request", "保留最近一轮"),
        ],
    );
    let child = create_api_subtask("child", "root", vec![copied_user, child_answer]);
    let original_tasks = vec![child.clone(), root.clone()];
    let plan = prepare_plan(
        &collect_linearized_task_messages(&original_tasks),
        &CompactionState::default(),
        &CompactionConfig {
            tail_turns: 1,
            ..Default::default()
        },
        ModelLimit::FALLBACK,
        |_| true,
    )
    .unwrap();
    assert_eq!(
        plan.head_message_ids,
        ["u1", "spawn", "child-answer", "root-answer"]
    );
    root.messages
        .push(output("summary", "summary-request", "包含子任务结果的摘要"));
    let mut conversation =
        AIConversation::new_restored(AIConversationId::new(), vec![root, child], None).unwrap();

    assert!(commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        true
    ));

    let hidden = conversation.compaction_state.hidden_message_ids();
    assert!(hidden.contains("child-answer"));
    assert!(!hidden.contains("tail-user"));
    assert!(!hidden.contains("tail-answer"));
}

#[test]
fn inactive_subtask_messages_do_not_invalidate_an_active_task_plan() {
    let mut root = create_api_task(
        "root",
        vec![
            user("u1"),
            create_subagent_tool_call_message("spawn", "root", "child", None),
            output("root-answer", "root-request", "已合并子任务结果"),
            user("tail-user"),
            output("tail-answer", "tail-request", "保留最近一轮"),
        ],
    );
    let mut child_answer = output("child-answer", "child-request", "已完成子任务");
    child_answer.task_id = "child".to_owned();
    let child = create_api_subtask("child", "root", vec![child_answer]);
    let active_tasks = vec![root.clone()];
    let plan = prepare_plan(
        &collect_linearized_task_messages(&active_tasks),
        &CompactionState::default(),
        &CompactionConfig {
            tail_turns: 1,
            ..Default::default()
        },
        ModelLimit::FALLBACK,
        |_| true,
    )
    .unwrap();
    root.messages
        .push(output("summary", "summary-request", "摘要"));
    let mut conversation =
        AIConversation::new_restored(AIConversationId::new(), vec![root, child], None).unwrap();

    assert!(commit_summarization(
        &mut conversation,
        &plan,
        "summary-request",
        true
    ));
    assert!(
        !conversation
            .compaction_state
            .hidden_message_ids()
            .contains("child-answer")
    );
}

#[test]
fn shrinking_an_incremental_plan_keeps_previous_coverage_and_unselected_tail() {
    let first_messages = vec![user("u1"), output("a1", "r1", "最早的信息")];
    let first_plan = prepare(&first_messages);
    let mut current = first_messages;
    current.push(output("summary-1", "summary-request-1", "最早信息的摘要"));
    let mut previous = conversation(current.clone());
    assert!(commit_summarization(
        &mut previous,
        &first_plan,
        "summary-request-1",
        true
    ));
    let state = previous.compaction_state.clone();
    current.push(user("u2"));
    current.push(output("a2", "r2", "可归并的新信息"));
    current.push(user("u3"));
    current.push(output("a3", "r3", "这轮不放进摘要"));
    let plan = prepare_plan(
        &current.iter().collect::<Vec<_>>(),
        &state,
        &CompactionConfig {
            tail_turns: 0,
            ..Default::default()
        },
        ModelLimit::FALLBACK,
        |plan| plan.head_message_ids.len() <= 5,
    )
    .unwrap();
    assert_eq!(plan.head_message_ids, ["u1", "a1", "summary-1", "u2", "a2"]);
    assert_eq!(plan.previous_summary.as_deref(), Some("最早信息的摘要"));
    assert_eq!(plan.tail_start_id.as_deref(), Some("u3"));
    current.push(output(
        "summary-2",
        "summary-request-2",
        "最早信息和第二轮信息的摘要",
    ));
    let mut updated = conversation(current);
    updated.compaction_state = state;

    assert!(commit_summarization(
        &mut updated,
        &plan,
        "summary-request-2",
        true
    ));

    let hidden = updated.compaction_state.hidden_message_ids();
    assert!(hidden.contains("u1"));
    assert!(hidden.contains("summary-1"));
    assert!(hidden.contains("u2"));
    assert!(!hidden.contains("u3"));
    assert!(!hidden.contains("a3"));
}
