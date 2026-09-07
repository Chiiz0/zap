pub(crate) mod convert_conversation;
mod convert_from;
mod convert_to;

use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

pub use ai::agent::convert::ConvertToAPITypeError;
use ai::api_keys::ApiKeyManager;
pub use convert_from::{
    ConversionParams, ConvertAPIMessageToClientOutputMessage, MaybeAIAgentOutputMessage,
    MessageToAIAgentOutputMessageError, user_inputs_from_messages,
};
use futures_lite::Stream;
use mcp::TemplatableMCPServerInfo;
use serde::Serialize;
use warp_core::channel::{Channel, ChannelState};
use warp_core::execution_mode::AppExecutionMode;
use warp_core::features::FeatureFlag;
use warp_core::user_preferences::GetUserPreferences;
use warpui::{AppContext, EntityId, SingletonEntity as _};

use super::{
    AIAgentInput, MCPContext, MCPServer, RequestMetadata, RunningCommand, ServerOutputId,
    Suggestions,
};
use crate::ai::agent::conversation::AIConversationId;
use crate::ai::ambient_agents::AmbientAgentTaskId;
use crate::ai::api_error::AIApiError;
use crate::ai::blocklist::{BlocklistAIPermissions, RequestInput, SessionContext};
use crate::ai::execution_profiles::AIExecutionProfileAppExt;
use crate::ai::execution_profiles::profiles::AIExecutionProfilesModel;
use crate::ai::facts::{AIFact, AIFactObjectModel};
use crate::ai::llms::{LLMId, LLMPreferences};
use crate::ai::machine_memory::{self, MachineMemoryContext};
use crate::ai::mcp::TemplatableMCPServerManager;
use crate::ai::project_agent_context::{self, ProjectAgentContext};
use crate::cloud_object::StoredObject;
use crate::cloud_object::model::generic_string_model::GenericStringObjectId;
use crate::cloud_object::model::persistence::ObjectStoreModel;
use crate::settings::AISettings;
use crate::terminal::safe_mode_settings::get_secret_obfuscation_mode;
use crate::workspaces::user_workspaces::UserWorkspaces;

/// Unique, server-generated conversation-scoped token to be roundtripped to the API when sending
/// requests that follow-up within a given conversation.
#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerConversationToken(String);

impl ServerConversationToken {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn debug_link(&self) -> String {
        format!(
            "{}://debug/maa/{}",
            ChannelState::url_scheme(),
            self.as_str()
        )
    }

    pub fn debugging_payload(&self, request_id: Option<&ServerOutputId>) -> String {
        self.debugging_payload_for_channel(request_id, ChannelState::channel())
    }

    fn debugging_payload_for_channel(
        &self,
        request_id: Option<&ServerOutputId>,
        channel: Channel,
    ) -> String {
        if channel.is_dogfood() {
            match request_id {
                Some(request_id) => format!("{}?request={request_id}", self.debug_link()),
                None => self.debug_link(),
            }
        } else {
            match request_id {
                Some(request_id) => format!(
                    "{{\"request_id\":\"{request_id}\",\"conversation_id\":\"{}\"}}",
                    self.as_str()
                ),
                None => format!("{{\"conversation_id\":\"{}\"}}", self.as_str()),
            }
        }
    }

    pub fn conversation_link(&self) -> String {
        format!(
            "{}://conversation/{}",
            ChannelState::url_scheme(),
            self.as_str()
        )
    }
}

impl From<ServerConversationToken> for String {
    fn from(value: ServerConversationToken) -> Self {
        value.0
    }
}

#[derive(Debug, Clone)]
pub struct RequestParams {
    pub input: Vec<AIAgentInput>,
    pub conversation_token: Option<ServerConversationToken>,
    pub forked_from_conversation_token: Option<ServerConversationToken>,
    pub ambient_agent_task_id: Option<AmbientAgentTaskId>,
    pub tasks: Vec<warp_multi_agent_api::Task>,
    pub existing_suggestions: Option<Suggestions>,
    pub metadata: Option<RequestMetadata>,
    pub session_context: SessionContext,
    pub model: LLMId,
    #[allow(unused)]
    pub coding_model: LLMId,
    pub cli_agent_model: LLMId,
    pub computer_use_model: LLMId,
    pub is_memory_enabled: bool,
    pub machine_memory: Option<MachineMemoryContext>,
    /// 会话显式绑定的项目上下文,
    /// 由 `chat_stream::render_project_context_block` 渲染进 system prompt。
    pub project_context: Option<ProjectAgentContext>,
    pub machine_index: Option<String>,
    /// Zap BYOP 专用:用户在 设置 → Agents → Rules 创建的全局 Rules
    /// (`AIFact::Memory`)的快照,在 `new()` 中从 `ObjectStoreModel` 一次性拉取
    /// 后随请求 plumb 到 `chat_stream::build_chat_request` → `prompt_renderer`,
    /// 由 `partials/user_rules.j2` 渲染进 system prompt。
    /// 仅在 `is_memory_enabled` 为 true 且未被 trash 时收集;为了 prompt cache
    /// 稳定性,按 `(name, content)` 字典序排序。
    pub user_rules: Vec<(Option<String>, String)>,
    pub warp_drive_context_enabled: bool,
    pub context_window_limit: Option<u32>,
    pub mcp_context: Option<MCPContext>,
    pub planning_enabled: bool,
    should_redact_secrets: bool,

    /// User-provided API keys for AI providers (BYO API Key).
    pub api_keys: Option<warp_multi_agent_api::request::settings::ApiKeys>,
    /// User-provided custom model providers (BYOK endpoints).
    pub custom_model_providers:
        Option<warp_multi_agent_api::request::settings::CustomModelProviders>,
    /// User-defined custom model routers referenced by the current selection. Mirrors
    /// `custom_model_providers`: the selected model's `config_key` indexes into this
    /// registry. `None` when no custom router is selected.
    pub custom_model_routers: Option<warp_multi_agent_api::request::settings::CustomModelRouters>,
    pub allow_use_of_warp_credits: bool,
    pub autonomy_level: warp_multi_agent_api::AutonomyLevel,
    pub isolation_level: warp_multi_agent_api::IsolationLevel,
    pub web_search_enabled: bool,
    pub computer_use_enabled: bool,
    pub ask_user_question_enabled: bool,
    pub research_agent_enabled: bool,
    pub supported_tools_override: Option<Vec<warp_multi_agent_api::ToolType>>,
    /// Zap BYOP 专用:本地会话 id,只用于 request-readiness 诊断日志。
    pub byop_conversation_id: Option<AIConversationId>,
    /// Zap BYOP 专用:单次请求内的非持久诊断关联 id。
    pub byop_readiness_attempt_id: Option<String>,
    /// The conversation ID of the parent agent that spawned this child agent, if any.
    pub parent_agent_id: Option<String>,
    /// The display name for this agent (e.g. "Agent 1"), assigned by the orchestrator.
    pub agent_name: Option<String>,
    /// Zap BYOP 专用:发起本请求时,关联的 LRC(Long Running Command)block id。
    /// tag-in 首轮和已进入 agent control 的 CLI subagent 后续轮都会填充,用于
    /// 让 BYOP prompt / tools 继续绑定到当前 PTY,避免模型另起 shell 操作同一个 TUI。
    pub lrc_command_id: Option<String>,
    /// Zap BYOP 专用:LRC 当前快照。`UserQuery.running_command` 只覆盖用户输入轮,
    /// auto-resume / tool result 后续轮需要通过这里继续携带最新 PTY 内容。
    pub lrc_running_command: Option<RunningCommand>,
    /// Zap BYOP 本地会话压缩 sidecar 快照(controller 把 conversation.compaction_state.clone() 塞进来)。
    /// `chat_stream::build_chat_request` 据此:
    ///   1. 过滤 [`crate::ai::byop_compaction::state::CompactionState::hidden_message_ids`] 里的 messages
    ///   2. 在被隐去区间的位置插入"摘要 user/assistant 对"
    ///   3. 把 `tool_output_compacted_at` 不为空的 ToolCallResult 替换为占位符
    ///   4. 在 `AIAgentInput::SummarizeConversation` 路径切 head + 拼 SUMMARY_TEMPLATE 作 user message
    ///
    /// 默认 `None` = 兼容路径(无压缩)。
    pub compaction_state: Option<crate::ai::byop_compaction::state::CompactionState>,
    /// 发出摘要前固定的覆盖范围，响应提交必须复用本计划。
    pub compaction_plan: Option<crate::ai::byop_compaction::plan::CompactionPlan>,
    /// Zap BYOP repair sidecar 快照。serializer 只读使用,不在请求构造中反序列化持久化 JSON。
    pub byop_repair_state: crate::ai::byop_readiness::RepairStateStatus,
    /// Zap BYOP 专用:本轮是否需要模拟上游 CreateTask 流程来升级 optimistic CLI subtask。
    /// 只有用户刚 tag-in 的首轮需要;已存在 CLI subagent 的后续轮只复用 task,不能重复 spawn。
    pub lrc_should_spawn_subagent: bool,
    /// Zap BYOP 专用:本轮响应应该写入的 task。普通对话是 root task;
    /// CLI subagent 后续轮则是对应 subtask。
    pub byop_target_task_id: Option<String>,
}

/// 收集用户在 设置 → Agents → Rules 创建的全局 Rules(`AIFact::Memory`)快照,
/// 用于注入 BYOP system prompt(Issue #116)。
///
/// - 过滤 trashed 条目
/// - 按 `(name, content)` 字典序排序,避免 HashMap 迭代导致的请求间顺序漂移
///   (否则会击穿上游 Anthropic / OpenAI 的 prompt cache)
///
/// 不在内部判断 `is_memory_enabled`,gate 由调用方控制;这样函数可作为纯
/// 集合逻辑独立测试,不依赖 `AISettings` 等 singleton。
pub(crate) fn collect_user_rules(
    object_store_model: &ObjectStoreModel,
) -> Vec<(Option<String>, String)> {
    let mut rules: Vec<(Option<String>, String)> = object_store_model
        .get_all_objects_of_type::<GenericStringObjectId, AIFactObjectModel>()
        .filter(|ai_fact| !ai_fact.is_trashed(object_store_model))
        .map(|ai_fact| match &ai_fact.model().string_model {
            AIFact::Memory(memory) => (memory.name.clone(), memory.content.clone()),
        })
        .collect();
    rules.sort();
    rules
}

pub type Event = Result<warp_multi_agent_api::ResponseEvent, Arc<AIApiError>>;

#[cfg(not(target_family = "wasm"))]
pub type ResponseStream = Pin<Box<dyn Stream<Item = Event> + Send + 'static>>;

// The WASM version of this type has no bound on `Send`, which is an unnecessary bound when
// targeting wasm because the browser is single-threaded (and we don't leverage WebWorkers for async
// execution in WoW).
#[cfg(target_family = "wasm")]
pub type ResponseStream = Pin<Box<dyn Stream<Item = Event>>>;

#[derive(Debug, Clone)]
pub struct ConversationData {
    pub id: AIConversationId,
    pub project_id: Option<String>,
    pub tasks: Vec<warp_multi_agent_api::Task>,
    pub server_conversation_token: Option<ServerConversationToken>,
    pub forked_from_conversation_token: Option<ServerConversationToken>,
    pub ambient_agent_task_id: Option<AmbientAgentTaskId>,
    pub existing_suggestions: Option<Suggestions>,
}

impl RequestParams {
    #[cfg(test)]
    pub(crate) fn new_for_test(
        input: Vec<AIAgentInput>,
        tasks: Vec<warp_multi_agent_api::Task>,
    ) -> Self {
        Self {
            input,
            conversation_token: None,
            forked_from_conversation_token: None,
            ambient_agent_task_id: None,
            byop_target_task_id: tasks.first().map(|task| task.id.clone()),
            tasks,
            existing_suggestions: None,
            metadata: None,
            session_context: SessionContext::new_for_test(),
            model: LLMId::from("byop:test"),
            coding_model: LLMId::from("byop:test"),
            cli_agent_model: LLMId::from("byop:test"),
            computer_use_model: LLMId::from("byop:test"),
            is_memory_enabled: false,
            machine_memory: None,
            project_context: None,
            machine_index: None,
            user_rules: Vec::new(),
            warp_drive_context_enabled: false,
            context_window_limit: None,
            mcp_context: None,
            planning_enabled: true,
            should_redact_secrets: false,
            api_keys: None,
            custom_model_providers: None,
            custom_model_routers: None,
            allow_use_of_warp_credits: false,
            autonomy_level: warp_multi_agent_api::AutonomyLevel::Supervised,
            isolation_level: warp_multi_agent_api::IsolationLevel::None,
            web_search_enabled: false,
            computer_use_enabled: false,
            ask_user_question_enabled: false,
            research_agent_enabled: false,
            supported_tools_override: None,
            byop_conversation_id: Some(AIConversationId::new()),
            byop_readiness_attempt_id: None,
            parent_agent_id: None,
            agent_name: None,
            lrc_command_id: None,
            lrc_running_command: None,
            compaction_state: None,
            compaction_plan: None,
            byop_repair_state: Default::default(),
            lrc_should_spawn_subagent: false,
        }
    }

    pub fn new(
        terminal_view_id: Option<EntityId>,
        session_context: SessionContext,
        request_input: &RequestInput,
        conversation: ConversationData,
        metadata: Option<RequestMetadata>,
        app: &AppContext,
    ) -> Self {
        let ai_settings = AISettings::as_ref(app);
        let is_memory_enabled = ai_settings.is_memory_enabled(app);
        let machine_memory = machine_memory::load_for_session(&session_context, app);
        let project_context =
            project_agent_context::load_for_conversation(conversation.project_id.as_deref());
        let machine_index = machine_memory::load_index_for_session(&session_context, app);
        let warp_drive_context_enabled = ai_settings.is_warp_drive_context_enabled(app);

        // Zap BYOP 修复 Issue #116:gate 在 `is_memory_enabled`,具体收集逻辑
        // 抽到 `collect_user_rules` 纯函数,只接 `&ObjectStoreModel` 入参以便测试,
        // 不依赖完整 AppContext singleton 集合。
        let user_rules = if is_memory_enabled {
            collect_user_rules(ObjectStoreModel::as_ref(app))
        } else {
            Vec::new()
        };

        // Build MCP context - either grouped by server or flat lists based on feature flag
        let mcp_context = if FeatureFlag::MCPGroupedServerContext.is_enabled() {
            // Group MCP tools and resources by server
            let templatable_manager = TemplatableMCPServerManager::as_ref(app);

            let mut active_servers: Vec<&TemplatableMCPServerInfo> = templatable_manager
                .get_active_templatable_servers()
                .values()
                .copied()
                .collect();

            // If file-based MCP servers are enabled, add active servers in scope of
            // the user's current working directory
            if let Some(cwd) = session_context.current_working_directory() {
                active_servers.extend(
                    templatable_manager
                        .get_active_file_based_servers(Path::new(cwd), app)
                        .values(),
                );
            }

            // Include any ephemeral MCP servers started via the Oz CLI.
            active_servers.extend(
                templatable_manager
                    .get_active_cli_spawned_servers()
                    .values(),
            );

            // Include built-in Warp-hosted servers (e.g. the Factory MCP).
            active_servers.extend(templatable_manager.get_active_builtin_servers().values());

            let servers: Vec<MCPServer> = active_servers
                .into_iter()
                .map(|server| MCPServer {
                    name: server.name().to_string(),
                    description: server.description().unwrap_or_default().to_string(),
                    id: server.installation_id().to_string(),
                    resources: server.resources().to_vec(),
                    tools: server.tools().to_vec(),
                })
                .collect();

            if servers.is_empty() {
                None
            } else {
                #[allow(deprecated)]
                Some(MCPContext {
                    resources: vec![],
                    tools: vec![],
                    servers,
                })
            }
        } else {
            // Flat lists of resources and tools
            let templatable_mcp_manager = TemplatableMCPServerManager::as_ref(app);
            let resources = templatable_mcp_manager
                .resources()
                .cloned()
                .collect::<Vec<_>>();
            let tools = templatable_mcp_manager.tools().cloned().collect::<Vec<_>>();

            #[allow(deprecated)]
            (!resources.is_empty() || !tools.is_empty()).then_some(MCPContext {
                resources,
                tools,
                servers: vec![],
            })
        };

        let should_redact_secrets = get_secret_obfuscation_mode(app).should_redact_secret();

        let user_workspaces = UserWorkspaces::as_ref(app);
        let api_key_manager = ApiKeyManager::as_ref(app);
        let is_byo_enabled = user_workspaces.is_byo_api_key_enabled(app);
        #[cfg(not(target_family = "wasm"))]
        let geap_binding = crate::ai::geap_credentials::current_geap_policy(app).mint_binding();
        #[cfg(target_family = "wasm")]
        let geap_binding: Option<::ai::api_keys::GeapMintBinding> = None;
        let api_keys = api_key_manager.api_keys_for_request(
            is_byo_enabled,
            user_workspaces.is_aws_bedrock_credentials_enabled(app),
            geap_binding,
        );
        let is_custom_inference_enabled = user_workspaces.is_custom_inference_enabled(app);
        let custom_model_providers =
            api_key_manager.custom_model_providers_for_request(is_custom_inference_enabled);
        let custom_model_routers = FeatureFlag::CustomModelRouters.is_enabled().then(|| {
            LLMPreferences::as_ref(app).custom_model_routers_for_request(
                &request_input.model_id,
                &request_input.coding_model_id,
            )
        });
        let allow_use_of_warp_credits = *AISettings::as_ref(app).can_use_warp_credits_for_fallback;

        let app_execution_mode = AppExecutionMode::as_ref(app);
        let autonomy_level = if app_execution_mode.is_autonomous() {
            warp_multi_agent_api::AutonomyLevel::Unsupervised
        } else {
            warp_multi_agent_api::AutonomyLevel::Supervised
        };

        let isolation_level = if app_execution_mode.is_sandboxed() {
            warp_multi_agent_api::IsolationLevel::Sandbox
        } else {
            warp_multi_agent_api::IsolationLevel::None
        };

        let web_search_enabled =
            BlocklistAIPermissions::as_ref(app).get_web_search_enabled(app, terminal_view_id);
        let research_agent_enabled = app
            .private_user_preferences()
            .read_value("ResearchAgentEnabled")
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();
        let is_ambient_agent = conversation.ambient_agent_task_id.is_some();
        let computer_use_enabled = FeatureFlag::AgentModeComputerUse.is_enabled()
            && BlocklistAIPermissions::as_ref(app)
                .get_computer_use_setting(app, terminal_view_id)
                .is_enabled()
            && computer_use::is_supported_on_current_platform()
            && (FeatureFlag::LocalComputerUse.is_enabled() || is_ambient_agent);
        let ask_user_question_enabled = BlocklistAIPermissions::as_ref(app)
            .get_ask_user_question_setting(app, terminal_view_id)
            != crate::ai::execution_profiles::AskUserQuestionPermission::Never;

        let byop_target_task_id = if request_input.input_messages.len() == 1 {
            request_input
                .input_messages
                .keys()
                .next()
                .map(ToString::to_string)
        } else {
            None
        };

        // Reconcile the persisted override against the active base model's
        // current `LLMContextWindow` instead of trusting whatever was stored
        // last. If the active model isn't configurable or has been removed
        // server-side, drop the override; otherwise clamp it to the model's
        // current `[min, max]` range. This closes the window between an
        // in-flight model metadata refresh and the next request.
        let context_window_limit = AIExecutionProfilesModel::as_ref(app)
            .active_profile(terminal_view_id, app)
            .data()
            .context_window_limit_for_request(app);

        Self {
            input: request_input.all_inputs().cloned().collect(),
            conversation_token: conversation.server_conversation_token,
            forked_from_conversation_token: conversation.forked_from_conversation_token,
            ambient_agent_task_id: conversation.ambient_agent_task_id,
            tasks: conversation.tasks,
            existing_suggestions: conversation.existing_suggestions,
            context_window_limit,
            metadata,
            session_context,
            model: request_input.model_id.clone(),
            coding_model: request_input.coding_model_id.clone(),
            cli_agent_model: request_input.cli_agent_model_id.clone(),
            computer_use_model: request_input.computer_use_model_id.clone(),
            is_memory_enabled,
            machine_memory,
            project_context,
            machine_index,
            user_rules,
            warp_drive_context_enabled,
            mcp_context,
            planning_enabled: true,
            should_redact_secrets,
            api_keys,
            custom_model_providers,
            custom_model_routers,
            allow_use_of_warp_credits,
            autonomy_level,
            isolation_level,
            web_search_enabled,
            computer_use_enabled,
            ask_user_question_enabled,
            research_agent_enabled,
            supported_tools_override: request_input.supported_tools_override.clone(),
            byop_conversation_id: Some(conversation.id),
            byop_readiness_attempt_id: None,
            parent_agent_id: None,
            agent_name: None,
            lrc_command_id: None,
            lrc_running_command: None,
            lrc_should_spawn_subagent: false,
            byop_target_task_id,
            // BYOP-only:由 controller 在 dispatch 到 BYOP exec 前回填(setter 风格,
            // 避免穿过 ConversationRequestData / 非 BYOP 路径)。
            compaction_state: None,
            compaction_plan: None,
            byop_repair_state: Default::default(),
        }
    }
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
