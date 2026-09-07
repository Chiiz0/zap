//! 自动摘要期间待发输入的恢复记录；重启后只供用户手动继续。

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use warp_multi_agent_api as api;

use crate::ai::agent::conversation::AIConversationId;
use crate::ai::agent::task::TaskId;
use crate::ai::agent::{
    AIAgentAttachment, AIAgentContext, AIAgentExchange, AIAgentExchangeId, AIAgentInput,
    AIAgentOutputStatus, CancellationReason, FinishedAIAgentOutput, UserQueryMode,
};
use crate::ai::blocklist::RequestInput;
use crate::ai::llms::LLMId;
use crate::terminal::shared_session::ParticipantId;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct CompactionPersistenceError(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingCompactionQuery {
    pub task_id: TaskId,
    pub query: String,
    pub context: Arc<[AIAgentContext]>,
    pub referenced_attachments: HashMap<String, AIAgentAttachment>,
    pub user_query_mode: UserQueryMode,
}

impl PendingCompactionQuery {
    pub fn to_input(&self) -> AIAgentInput {
        AIAgentInput::UserQuery {
            query: self.query.clone(),
            context: self.context.clone(),
            referenced_attachments: self.referenced_attachments.clone(),
            user_query_mode: self.user_query_mode,
            static_query_type: None,
            running_command: None,
            intended_agent: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PendingCompactionRecovery {
    /// 同时作为恢复 exchange 的稳定 ID，防止旧流消费新请求的记录。
    pub id: AIAgentExchangeId,
    /// 本次运行中承载待发输入的 exchange；重载后仍用稳定的 id 识别恢复行。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_exchange_id: Option<AIAgentExchangeId>,
    pub queries: Vec<PendingCompactionQuery>,
    /// 工具续接没有新问题时，只恢复手动继续所需上下文，不重放工具结果。
    pub resume_context: Arc<[AIAgentContext]>,
    pub resume_task_id: Option<TaskId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_request_id: Option<String>,
    pub request_start_ts: DateTime<Local>,
    pub working_directory: Option<String>,
    pub model_id: LLMId,
    pub coding_model_id: LLMId,
    pub cli_agent_model_id: LLMId,
    pub computer_use_model_id: LLMId,
    /// 原请求已经开始续发时，用其持久化消息识别尚未清除的旧恢复记录。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_request_id: Option<String>,
}

impl PendingCompactionRecovery {
    pub fn from_request(request: &RequestInput) -> Self {
        let mut tasks = request.input_messages.iter().collect::<Vec<_>>();
        tasks.sort_by(|(left, _), (right, _)| left.cmp(right));
        let queries: Vec<_> = tasks
            .iter()
            .flat_map(|(task_id, inputs)| {
                inputs.iter().filter_map(move |input| {
                    let AIAgentInput::UserQuery {
                        query,
                        context,
                        referenced_attachments,
                        user_query_mode,
                        ..
                    } = input
                    else {
                        return None;
                    };
                    Some(PendingCompactionQuery {
                        task_id: (*task_id).clone(),
                        query: query.clone(),
                        context: context.clone(),
                        referenced_attachments: referenced_attachments.clone(),
                        user_query_mode: *user_query_mode,
                    })
                })
            })
            .collect();
        let resume_context = if queries.is_empty() {
            tasks
                .iter()
                .flat_map(|(_, inputs)| inputs.iter())
                .find_map(AIAgentInput::context)
                .map(Arc::from)
                .unwrap_or_default()
        } else {
            Arc::default()
        };
        Self {
            id: AIAgentExchangeId::new(),
            owner_exchange_id: None,
            queries,
            resume_context,
            resume_task_id: tasks.first().map(|(task_id, _)| (*task_id).clone()),
            anchor_message_id: None,
            summary_request_id: None,
            request_start_ts: request.request_start_ts,
            working_directory: request.working_directory.clone(),
            model_id: request.model_id.clone(),
            coding_model_id: request.coding_model_id.clone(),
            cli_agent_model_id: request.cli_agent_model_id.clone(),
            computer_use_model_id: request.computer_use_model_id.clone(),
            resumed_request_id: None,
        }
    }

    /// 旧问题即使仍留在历史中，也不能接管另一条较新输入的“继续”操作。
    pub fn is_resume_target(&self, exchange_id: AIAgentExchangeId) -> bool {
        self.id == exchange_id || self.owner_exchange_id == Some(exchange_id)
    }

    pub fn to_request_input(
        &self,
        conversation_id: AIConversationId,
        fallback_task_id: &TaskId,
        response_initiator: Option<ParticipantId>,
    ) -> RequestInput {
        let mut input_messages: HashMap<TaskId, Vec<AIAgentInput>> = HashMap::new();
        for query in &self.queries {
            input_messages
                .entry(query.task_id.clone())
                .or_default()
                .push(query.to_input());
        }
        if input_messages.is_empty() {
            input_messages.insert(
                self.resume_task_id
                    .as_ref()
                    .unwrap_or(fallback_task_id)
                    .clone(),
                vec![AIAgentInput::ResumeConversation {
                    context: self.resume_context.clone(),
                }],
            );
        }
        RequestInput {
            conversation_id,
            input_messages,
            working_directory: self.working_directory.clone(),
            model_id: self.model_id.clone(),
            coding_model_id: self.coding_model_id.clone(),
            cli_agent_model_id: self.cli_agent_model_id.clone(),
            computer_use_model_id: self.computer_use_model_id.clone(),
            shared_session_response_initiator: response_initiator,
            request_start_ts: self.request_start_ts,
            supported_tools_override: None,
        }
    }

    /// 只用关联的请求 ID 去重；正文相同的旧问题不能证明这次输入已经发送。
    pub fn is_in_persisted_messages(&self, messages: &[&api::Message]) -> bool {
        let Some(request_id) = self.resumed_request_id.as_deref() else {
            return false;
        };
        if request_id.is_empty() {
            return false;
        }
        if self.queries.is_empty() {
            return messages
                .iter()
                .any(|message| message.request_id == request_id);
        }
        self.queries.iter().all(|query| {
            messages.iter().any(|message| {
                message.request_id == request_id
                    && matches!(&message.message, Some(api::message::Message::UserQuery(input))
                        if input.query == query.query)
            })
        })
    }

    pub fn to_cancelled_exchange(&self) -> AIAgentExchange {
        let input = if self.queries.is_empty() {
            vec![AIAgentInput::ResumeConversation {
                context: self.resume_context.clone(),
            }]
        } else {
            self.queries
                .iter()
                .map(PendingCompactionQuery::to_input)
                .collect()
        };
        AIAgentExchange {
            id: self.id,
            input,
            output_status: AIAgentOutputStatus::Finished {
                finished_output: FinishedAIAgentOutput::Cancelled {
                    output: None,
                    reason: CancellationReason::ManuallyCancelled,
                },
            },
            added_message_ids: HashSet::new(),
            start_time: self.request_start_ts,
            finish_time: Some(self.request_start_ts),
            time_to_first_token_ms: None,
            working_directory: self.working_directory.clone(),
            model_id: self.model_id.clone(),
            coding_model_id: self.coding_model_id.clone(),
            cli_agent_model_id: self.cli_agent_model_id.clone(),
            computer_use_model_id: self.computer_use_model_id.clone(),
            request_cost: None,
            response_initiator: None,
        }
    }
}

#[cfg(test)]
#[path = "recovery_tests.rs"]
mod tests;
