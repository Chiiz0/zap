//! This module contains model, controller, and view logic for Blocklist AI.
mod action_model;
pub mod agent_view;
pub mod block;
mod child_agent_launch;
pub mod code_block;
mod context_model;
mod controller;
pub(crate) mod conversation_selection;
pub(crate) mod diff_storage;
pub(crate) mod diff_types;
// Zap:上游的 handoff / local_agent_task_sync_model / orchestration_event_streamer
// 依赖已删除的云端 server_api / warp_graphql,不再挂载(文件保留待后续本地化)。
pub(crate) mod orchestration_events;
pub(crate) mod orchestration_topology;
mod passive_suggestions;
pub(crate) mod queued_query;
pub(super) use controller::RequestInput;
pub mod history_model;
pub mod inline_action;
mod input_mode_policy;
mod input_model;
mod permissions;
mod persistence;
pub mod prompt;
pub mod suggested_agent_mode_workflow_modal;
pub mod suggested_rule_modal;
mod suggestion_chip_view;
pub mod summarization_cancel_dialog;
pub mod usage;

pub(crate) mod telemetry;
pub(crate) mod telemetry_banner;
pub(crate) mod view_util;

// Consumed by `tui_export` for the `warp_tui` frontend.
#[cfg_attr(not(feature = "tui"), allow(unused_imports))]
pub use action_model::AIActionStatus;
// Consumed by `tui_export` for the `warp_tui` frontend.
#[cfg(feature = "tui")]
pub use action_model::{
    AskUserQuestionExecutor, NewConversationDecision, RequestFileEditsExecutor,
};
pub use action_model::{
    BlocklistAIActionEvent, BlocklistAIActionModel, ShellCommandExecutor, ShellCommandExecutorEvent,
};
#[cfg_attr(target_family = "wasm", allow(unused_imports))]
pub(crate) use action_model::{
    FileReadResult, PromptSuggestionExecutor, PromptSuggestionExecutorEvent, ReadFileContextResult,
    RequestFileEditsFormatKind, agent_shell_command_block_output, apply_edits,
    read_local_file_context,
};
// Consumed by `tui_export` for the `warp_tui` frontend.
#[cfg(feature = "tui")]
pub use action_model::{RunAgentsExecutor, RunAgentsExecutorEvent, RunAgentsSpawningSnapshot};
// Consumed by `tui_export` for the `warp_tui` frontend's child-agent
// materializer, in addition to the GUI pane-group dispatch.
#[cfg_attr(
    any(target_family = "wasm", not(feature = "tui")),
    allow(unused_imports)
)]
pub use action_model::{
    StartAgentExecutor, StartAgentExecutorEvent, StartAgentOutcome, StartAgentRequest,
    StartAgentRequestId,
};
#[cfg(any(test, feature = "integration_tests"))]
pub(crate) use block::model::testing::FakeAIBlockModel;
pub(crate) use block::{AIBlock, AIBlockEvent, RequestedEditResolution, init, model};
pub use block::{keyboard_navigable_buttons, toggleable_items};
pub use child_agent_launch::inherit_child_agent_settings;
#[cfg(not(target_family = "wasm"))]
#[cfg_attr(not(feature = "tui"), allow(unused_imports))]
pub use child_agent_launch::{
    PreparedLocalOzChildLaunch, apply_child_agent_model_override, prepare_local_oz_child_launch,
};
#[cfg(feature = "tui")]
pub use context_model::PendingAttachmentSummary;
pub(crate) use context_model::PendingContextSnapshot;
#[cfg(not(feature = "tui"))]
pub(crate) use context_model::block_context_from_terminal_model;
#[cfg(feature = "tui")]
pub use context_model::block_context_from_terminal_model;
pub use context_model::{
    AttachmentType, BlocklistAIContextEvent, BlocklistAIContextModel, PendingAttachment,
    PendingFile,
};
pub use controller::input_context::{
    BLOCK_CONTEXT_ATTACHMENT_REGEX, DIFF_HUNK_ATTACHMENT_REGEX, DRIVE_OBJECT_ATTACHMENT_REGEX,
};
pub(crate) use controller::input_context::{
    drive_object_attachment_for_reference, plan_attachment_for_reference,
};
#[cfg(test)]
pub(crate) use controller::response_stream::ResponseStream;
pub(crate) use controller::response_stream::ResponseStreamId;
pub use controller::{BlocklistAIController, SlashCommandRequest};
pub(crate) use controller::{BlocklistAIControllerEvent, ClientIdentifiers, SessionContext};
pub(crate) use conversation_selection::{
    ConversationSelection, ConversationSelectionEvent, ConversationSelectionHandle,
    PendingQueryState,
};
pub(crate) use history_model::{
    AIQueryHistory, AIQueryHistoryOutputStatus, BeginConversationRenameError,
    BlocklistAIHistoryEvent, BlocklistAIHistoryModel, ConversationStatusUpdate, FORK_PREFIX,
    PRE_REWIND_PREFIX,
};
// The policy types are re-exported for the TUI frontend via `tui_export`.
#[cfg_attr(not(feature = "tui"), allow(unused_imports))]
pub use input_mode_policy::{InputModePolicy, InputModePolicyHandle, PolicyConfigUpdate};
pub(crate) use input_model::BlocklistAIInputEvent;
pub use input_model::{
    BlocklistAIInputModel, InputConfig, InputType, InputTypeAutoDetectionSource,
};
pub(crate) use passive_suggestions::{
    LegacyPassiveSuggestionsEvent, LegacyPassiveSuggestionsModel, MaaPassiveSuggestionsEvent,
    MaaPassiveSuggestionsModel, PassiveSuggestionsModels,
};
pub use permissions::{BlocklistAIPermissions, CommandExecutionPermissionAllowedReason};
#[cfg_attr(target_family = "wasm", allow(unused))]
pub(crate) use persistence::PersistedAIInputType;
#[cfg_attr(target_family = "wasm", allow(unused))]
pub use persistence::maybe_build_ai_query_upsert_event;
pub(crate) use persistence::{PersistedAIInput, SerializedBlockListItem};
pub(crate) use queued_query::{
    AutofireAction, QueuedQuery, QueuedQueryId, QueuedQueryOrigin, is_lrc_auto_queue_active,
};
#[cfg_attr(not(feature = "tui"), allow(unused_imports))]
pub use queued_query::{QueuedQueryEvent, QueuedQueryModel};
pub use suggestion_chip_view::*;
pub use view_util::error_color;
pub(crate) use view_util::{
    ATTACH_AS_AGENT_MODE_CONTEXT_TEXT, CLAUDE_ORANGE, ai_brand_color, ai_indicator_height,
    format_credits, get_ai_block_overflow_menu_element_position_id,
    get_attached_blocks_chip_element_position_id, render_ai_agent_mode_icon,
    render_ai_follow_up_icon,
};

// Zap:`AIBlockResponseRating`(thumbs up/down 反馈评分)随反馈遥测一并删除,
// 此处不再重导出。
pub use crate::ai::blocklist::block::{TextLocation, secret_redaction};
