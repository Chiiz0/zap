use ::ai::api_keys::{ApiKeyManager, ApiKeyManagerEvent, ApiKeys, CustomEndpointParams};
#[cfg(not(target_family = "wasm"))]
use ::ai::grok_subscription::oauth::{self, ManualCodeExchange};
use enum_iterator::all;
use itertools::Itertools;
use regex::Regex;
use settings::{Setting, ToggleableSetting};
use strum::IntoEnumIterator;
use warp_core::context_flag::ContextFlag;
use warp_core::features::FeatureFlag;
use warp_core::ui::theme::color::internal_colors;
use warpui::elements::{
    Border, ChildView, ConstrainedBox, Container, CornerRadius, CrossAxisAlignment, Dismiss, Empty,
    Expanded, Fill, Flex, FormattedTextElement, HighlightedHyperlink, Hoverable, HyperlinkLens,
    HyperlinkUrl, MainAxisAlignment, MainAxisSize, MouseStateHandle, ParentElement, Radius,
    Shrinkable, Text,
};
use warpui::fonts::{Properties, Weight};
use warpui::keymap::{ContextPredicate, Keystroke};
use warpui::platform::Cursor;
use warpui::text_layout::TextAlignment;
use warpui::ui_components::components::{Coords, UiComponent, UiComponentStyles};
use warpui::ui_components::slider::SliderStateHandle;
use warpui::ui_components::switch::SwitchStateHandle;
use warpui::{
    Action, AppContext, Element, Entity, SingletonEntity, TypedActionView, View, ViewContext,
    ViewHandle, WeakViewHandle, id,
};

use super::agent_providers_widget::AgentProvidersWidget;
use super::custom_inference_modal::{
    CustomEndpointModal, CustomEndpointModalEvent, CustomEndpointModalViewState,
};
use super::execution_profile_view::{ExecutionProfileView, ExecutionProfileViewEvent};
use super::remove_custom_endpoint_confirmation_dialog::{
    RemoveCustomEndpointConfirmationDialog, RemoveCustomEndpointConfirmationDialogEvent,
};
use super::set_default_model_modal::{SetDefaultModelModalBody, SetDefaultModelModalBodyEvent};
use super::settings_page::{
    HEADER_PADDING, InputListItem, LocalOnlyIconState, MatchData, PageType, SettingsPageMeta,
    SettingsPageViewHandle, SettingsWidget, TOGGLE_BUTTON_RIGHT_PADDING, ToggleState,
    build_sub_header, build_toggle_element, render_body_item_label,
    render_body_item_label_with_icon, render_custom_size_header, render_dropdown_item,
    render_dropdown_item_label, render_filterable_dropdown_item, render_full_pane_width_ai_button,
    render_input_list, render_separator, render_settings_info_banner,
};
use super::{
    SettingActionPairContexts, SettingActionPairDescriptions, SettingsAction, SettingsSection,
    ToggleSettingActionPair, editor_text_colors, flags,
};
#[cfg(not(target_family = "wasm"))]
use crate::ai::aws_credentials::refresh_aws_credentials;
use crate::ai::blocklist::BlocklistAIPermissions;
use crate::ai::blocklist::agent_view::agent_input_footer::editor::{
    AgentToolbarEditorMode, AgentToolbarInlineEditor,
};
use crate::ai::execution_profiles::model_menu_items::available_model_menu_items;
use crate::ai::execution_profiles::profiles::{
    AIExecutionProfilesModel, AIExecutionProfilesModelEvent,
};
use crate::ai::execution_profiles::{
    AIExecutionProfile, AIExecutionProfileAppExt, ActionPermission, ExecutionProfileId,
    WriteToPtyPermission, long_context_pricing_warning_title,
};
#[cfg(not(target_family = "wasm"))]
use crate::ai::geap_credentials::force_refresh_geap_credentials;
use crate::ai::llms::{
    LLMContextWindow, LLMId, LLMPreferences, LLMPreferencesEvent, LLMProvider,
    is_using_api_key_for_provider,
};
use crate::ai::mcp::TemplatableMCPServerManager;
use crate::ai::paths::host_native_absolute_path;
use crate::cloud_object::GenericStringObjectFormat::Json;
use crate::cloud_object::model::persistence::{ObjectStoreEvent, ObjectStoreModel};
use crate::cloud_object::{JsonObjectType, ObjectType};
use crate::editor::{EditorOptions, InteractionState, SingleLineEditorOptions, TextColors};
use crate::modal::{Modal, ModalEvent, ModalViewState};
use crate::settings::{
    AIAutoDetectionEnabled, AICommandDenylist, AISettingsChangedEvent,
    AgentModeCodingPermissionsType, AgentModeCommandExecutionDenylist,
    AgentModeCommandExecutionPredicate, AgentModeQuerySuggestionsEnabled,
    AutoApproveBypassesCommandDenylist, AwsBedrockAutoLogin, AwsBedrockCredentialsEnabled,
    FileBasedMcpEnabled, GeminiEnterpriseCredentialsEnabled, GitOperationsAutogenEnabled,
    IncludeAgentCommandsInHistory, InputSettings, IntelligentAutosuggestionsEnabled,
    LongRunningCommandSubmissionMode, MemoryEnabled, NLDInTerminalEnabled,
    NaturalLanguageAutosuggestionsEnabled, OrchestrationMessageDisplayMode, PromptSubmissionMode,
    RuleSuggestionsEnabled, ShouldRenderCLIAgentToolbar,
    ShouldRenderUseAgentToolbarForUserCommands, ShowAgentTips, ShowAgentZeroStateHints,
    ShowConversationHistory, ShowHintText, ThinkingDisplayMode, VoiceInputEnabled,
    WarpDriveContextEnabled,
};
use crate::terminal::CLIAgent;
use crate::terminal::cli_agent::{CLIAgentInstallEvent, CLIAgentInstallModel};
use crate::terminal::session_settings::{SessionSettings, SessionSettingsChangedEvent};
use crate::view_components::action_button::{ActionButton, ButtonSize, SecondaryTheme};
use crate::view_components::{
    FilterableDropdown, SubmittableTextInput, SubmittableTextInputEvent, WarningBoxConfig,
    render_warning_box,
};
use crate::workspaces::user_workspaces::UserWorkspacesEvent;

/// Identifies which subpage of the AI settings the user is viewing.
/// When `None`, the page shows all widgets (legacy/full view).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AISubpage {
    /// The main "WarpAgent" page: global AI toggle + Active AI + Input + Other sections.
    WarpAgent,
    /// Agent profiles and permissions.
    Profiles,
    /// 自定义 AI 提供商(BYOP) 配置子页。
    Providers,
    /// Knowledge / Rules settings.
    Knowledge,
    /// Third-party CLI agent settings.
    ThirdPartyCLIAgents,
}

impl AISubpage {
    pub fn from_section(section: SettingsSection) -> Option<Self> {
        match section {
            SettingsSection::WarpAgent => Some(Self::WarpAgent),
            SettingsSection::AgentProfiles => Some(Self::Profiles),
            SettingsSection::AgentProviders => Some(Self::Providers),
            SettingsSection::Knowledge => Some(Self::Knowledge),
            SettingsSection::ThirdPartyCLIAgents => Some(Self::ThirdPartyCLIAgents),
            // AgentMCPServers renders the standalone MCPServers page, not an AI subpage.
            _ => None,
        }
    }
}
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use markdown_parser::{FormattedText, FormattedTextFragment, FormattedTextLine};
use warp_errors::{report_error, report_if_error};

use crate::ai::{AIRequestUsageModel, AIRequestUsageModelEvent};
use crate::appearance::{Appearance, AppearanceEvent};
use crate::editor::{EditorView, Event as EditorEvent, TextOptions};
use crate::menu::{MenuItem, MenuItemFields};
use crate::server::telemetry::{
    AgentModeAutoDetectionSettingOrigin, AutonomySettingToggleSource,
    ToggleCodeSuggestionsSettingSource,
};
use crate::settings::{AISettings, VOICE_INPUT_LANGUAGES, VoiceInputLanguage, VoiceInputToggleKey};
use crate::ui_components::blended_colors;
use crate::ui_components::icons::Icon;
use crate::util::bindings;
use crate::view_components::dropdown::DropdownAction;
use crate::view_components::{Dropdown, DropdownItem};
use crate::{TelemetryEvent, UserWorkspaces, send_telemetry_from_ctx};

const CONTENT_FONT_SIZE: f32 = 12.;

// AI 设置页描述文本走 i18n,key 见 app/i18n/{en,zh-CN}/warp.ftl 的 settings-ai-* 段。
const AI_SETTINGS_DROPDOWN_WIDTH: f32 = 250.;
const AI_SETTINGS_DROPDOWN_MAX_HEIGHT: f32 = 250.;
const CONTEXT_WINDOW_SLIDER_WIDTH: f32 = 220.;
const CONTEXT_WINDOW_INPUT_BOX_WIDTH: f32 = 120.;
const WISPR_FLOW_URL: &str = "https://wisprflow.ai/";
const CUSTOM_ENDPOINT_MODAL_MAX_HEIGHT_PERCENTAGE: f32 = 0.8;

pub fn init_actions_from_parent_view<T: Action + Clone>(
    app: &mut AppContext,
    context: &ContextPredicate,
    builder: fn(SettingsAction) -> T,
) {
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-active-ai"),
                builder(SettingsAction::AI(AISettingsPageAction::ToggleActiveAI)),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::IS_ACTIVE_AI_ENABLED,
            )
            .with_group(bindings::BindingGroup::WarpAi),
        ],
        app,
    );

    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &if FeatureFlag::AgentView.is_enabled() {
                    crate::t!("toggle-suffix-ai-input-autodetect-agent")
                } else {
                    crate::t!("toggle-suffix-ai-input-autodetect-nld")
                },
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleAIInputAutoDetection,
                )),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::AI_INPUT_AUTODETECTION_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| FeatureFlag::AgentMode.is_enabled()),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-nld-in-terminal"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleNLDInTerminal,
                )),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::NLD_IN_TERMINAL_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| FeatureFlag::AgentView.is_enabled()),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-next-command"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleIntelligentAutosuggestions,
                )),
                &(context.clone() & id!(flags::IS_ACTIVE_AI_ENABLED)),
                flags::INTELLIGENT_AUTOSUGGESTIONS_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-prompt-suggestions"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::TogglePromptSuggestions,
                )),
                &(context.clone() & id!(flags::IS_ACTIVE_AI_ENABLED)),
                flags::PROMPT_SUGGESTIONS_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-code-suggestions"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleCodeSuggestions,
                )),
                &(context.clone()
                    & id!(flags::IS_ACTIVE_AI_ENABLED)
                    & id!(flags::PROMPT_SUGGESTIONS_FLAG)),
                flags::CODE_SUGGESTIONS_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::custom(
                SettingActionPairDescriptions::new(
                    &crate::t!("settings-command-show-agent-tips"),
                    &crate::t!("settings-command-hide-agent-tips"),
                ),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleShowAgentTips,
                )),
                SettingActionPairContexts::new(
                    context.clone()
                        & id!(flags::IS_ANY_AI_ENABLED)
                        & !id!(flags::SHOW_AGENT_TIPS_FLAG),
                    context.clone()
                        & id!(flags::IS_ANY_AI_ENABLED)
                        & id!(flags::SHOW_AGENT_TIPS_FLAG),
                ),
                None,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| FeatureFlag::AgentTips.is_enabled()),
        ],
        app,
    );
    {
        use warpui::keymap::FixedBinding;

        use crate::settings::ThinkingDisplayMode;

        let ai_context = context.clone() & id!(flags::IS_ANY_AI_ENABLED);
        let mode_bindings: Vec<FixedBinding> = ThinkingDisplayMode::iter()
            .map(|mode| {
                let context_flag = match mode {
                    ThinkingDisplayMode::ShowAndCollapse => {
                        flags::THINKING_DISPLAY_SHOW_AND_COLLAPSE
                    }
                    ThinkingDisplayMode::AlwaysShow => flags::THINKING_DISPLAY_ALWAYS_SHOW,
                    ThinkingDisplayMode::NeverShow => flags::THINKING_DISPLAY_NEVER_SHOW,
                };
                FixedBinding::empty(
                    mode.command_palette_description(),
                    builder(SettingsAction::AI(
                        AISettingsPageAction::SetThinkingDisplayMode(mode),
                    )),
                    ai_context.clone() & !id!(context_flag),
                )
                .with_group(bindings::BindingGroup::WarpAi.as_str())
            })
            .collect();
        app.register_fixed_bindings(mode_bindings);
    }
    {
        use warpui::keymap::FixedBinding;

        let ai_context = context.clone() & id!(flags::IS_ANY_AI_ENABLED);
        let mode_bindings: Vec<FixedBinding> = OrchestrationMessageDisplayMode::iter()
            .map(|mode| {
                let context_flag = match mode {
                    OrchestrationMessageDisplayMode::ShowAndCollapse => {
                        flags::ORCHESTRATION_MESSAGE_DISPLAY_SHOW_AND_COLLAPSE
                    }
                    OrchestrationMessageDisplayMode::AlwaysShow => {
                        flags::ORCHESTRATION_MESSAGE_DISPLAY_ALWAYS_SHOW
                    }
                    OrchestrationMessageDisplayMode::AlwaysCollapse => {
                        flags::ORCHESTRATION_MESSAGE_DISPLAY_ALWAYS_COLLAPSE
                    }
                };
                FixedBinding::empty(
                    mode.command_palette_description(),
                    builder(SettingsAction::AI(
                        AISettingsPageAction::SetOrchestrationMessageDisplayMode(mode),
                    )),
                    ai_context.clone() & !id!(context_flag),
                )
                .with_group(bindings::BindingGroup::WarpAi.as_str())
            })
            .collect();
        app.register_fixed_bindings(mode_bindings);
    }
    if FeatureFlag::QueueSlashCommand.is_enabled() {
        use warpui::keymap::FixedBinding;

        let ai_context = context.clone() & id!(flags::IS_ANY_AI_ENABLED);
        let mode_bindings: Vec<FixedBinding> = PromptSubmissionMode::iter()
            .map(|mode| {
                let context_flag = match mode {
                    PromptSubmissionMode::Interrupt => flags::PROMPT_SUBMISSION_INTERRUPT,
                    PromptSubmissionMode::Queue => flags::PROMPT_SUBMISSION_QUEUE,
                };
                FixedBinding::empty(
                    mode.command_palette_description(),
                    builder(SettingsAction::AI(
                        AISettingsPageAction::SetPromptSubmissionMode(mode),
                    )),
                    ai_context.clone() & !id!(context_flag),
                )
                .with_group(bindings::BindingGroup::WarpAi.as_str())
            })
            .collect();
        app.register_fixed_bindings(mode_bindings);

        // The LRC submission mode only applies (and is only shown) when the default
        // prompt submission mode is Interrupt, so its palette entries are gated on it.
        let lrc_mode_bindings: Vec<FixedBinding> = LongRunningCommandSubmissionMode::iter()
            .map(|mode| {
                let context_flag = match mode {
                    LongRunningCommandSubmissionMode::SendImmediately => {
                        flags::LRC_SUBMISSION_SEND_IMMEDIATELY
                    }
                    LongRunningCommandSubmissionMode::QueueUntilCommandCompletes => {
                        flags::LRC_SUBMISSION_QUEUE_UNTIL_COMMAND_COMPLETES
                    }
                };
                FixedBinding::empty(
                    mode.command_palette_description(),
                    builder(SettingsAction::AI(
                        AISettingsPageAction::SetLongRunningCommandSubmissionMode(mode),
                    )),
                    ai_context.clone()
                        & id!(flags::PROMPT_SUBMISSION_INTERRUPT)
                        & !id!(context_flag),
                )
                .with_group(bindings::BindingGroup::WarpAi.as_str())
            })
            .collect();
        app.register_fixed_bindings(lrc_mode_bindings);
    }
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-nl-autosuggestions"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleNaturalLanguageAutosuggestions,
                )),
                &(context.clone() & id!(flags::IS_ACTIVE_AI_ENABLED)),
                flags::NATURAL_LANGUAGE_AUTOSUGGESTIONS_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| FeatureFlag::PredictAMQueries.is_enabled()),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-git-operations"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleGitOperationsAutogen,
                )),
                &(context.clone() & id!(flags::IS_ACTIVE_AI_ENABLED)),
                flags::GIT_OPERATIONS_AUTOGEN_FLAG,
            )
            .with_enabled(|| FeatureFlag::GitOperationsInCodeReview.is_enabled())
            .is_supported_on_current_platform(
                AISettings::as_ref(app)
                    .git_operations_autogen_enabled_internal
                    .is_supported_on_current_platform()
                    && UserWorkspaces::as_ref(app).is_git_operations_ai_enabled(),
            ),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-voice-input"),
                builder(SettingsAction::AI(AISettingsPageAction::ToggleVoiceInput)),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::IS_VOICE_INPUT_ENABLED,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| cfg!(feature = "voice_input")),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::custom(
                SettingActionPairDescriptions::new(
                    &crate::t!("settings-command-show-use-agent-footer"),
                    &crate::t!("settings-command-hide-use-agent-footer"),
                ),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleUseAgentToolbar,
                )),
                SettingActionPairContexts::new(
                    context.clone()
                        & id!(flags::IS_ANY_AI_ENABLED)
                        & !id!(flags::USE_AGENT_FOOTER_FLAG),
                    context.clone()
                        & id!(flags::IS_ANY_AI_ENABLED)
                        & id!(flags::USE_AGENT_FOOTER_FLAG),
                ),
                None,
            )
            .with_group(bindings::BindingGroup::WarpAi),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-agent-command-history"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleIncludeAgentCommandsInHistory,
                )),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::INCLUDE_AGENT_COMMANDS_IN_HISTORY_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi),
            ToggleSettingActionPair::custom(
                if FeatureFlag::AgentApprovalModes.is_enabled() {
                    SettingActionPairDescriptions::new(
                        &crate::t!("settings-command-allow-full-access-denylist-bypass"),
                        &crate::t!("settings-command-require-full-access-denylist-approval"),
                    )
                } else {
                    SettingActionPairDescriptions::new(
                        &crate::t!("settings-command-allow-auto-approve-denylist-bypass"),
                        &crate::t!("settings-command-require-auto-approve-denylist-approval"),
                    )
                },
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleAutoApproveBypassesCommandDenylist,
                )),
                SettingActionPairContexts::new(
                    context.clone()
                        & id!(flags::IS_ANY_AI_ENABLED)
                        & !id!(flags::AUTO_APPROVE_BYPASSES_COMMAND_DENYLIST_FLAG),
                    context.clone()
                        & id!(flags::IS_ANY_AI_ENABLED)
                        & id!(flags::AUTO_APPROVE_BYPASSES_COMMAND_DENYLIST_FLAG),
                ),
                None,
            )
            .with_group(bindings::BindingGroup::WarpAi),
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-conversation-history"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleShowConversationHistory,
                )),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::SHOW_CONVERSATION_HISTORY,
            )
            .with_group(bindings::BindingGroup::WarpAi),
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-model-picker"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleShowBaseModelPickerInPrompt,
                )),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::SHOW_BASE_MODEL_PICKER_IN_PROMPT_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi),
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-coding-agent-toolbar"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleCLIAgentToolbar,
                )),
                context,
                flags::CLI_AGENT_FOOTER_ENABLED,
            )
            .with_group(bindings::BindingGroup::WarpAi),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("settings-ai-rules-label"),
                builder(SettingsAction::AI(AISettingsPageAction::ToggleRules)),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::AI_RULES_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| FeatureFlag::AIRules.is_enabled()),
            ToggleSettingActionPair::new(
                &crate::t!("settings-ai-suggested-rules-label"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleRuleSuggestions,
                )),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::SUGGESTED_RULES_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| {
                FeatureFlag::AIRules.is_enabled() && FeatureFlag::SuggestedRules.is_enabled()
            }),
            ToggleSettingActionPair::new(
                &crate::t!("settings-ai-drive-context-label"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleWarpDriveContext,
                )),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::WARP_DRIVE_CONTEXT_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| FeatureFlag::AIRules.is_enabled()),
            ToggleSettingActionPair::new(
                &crate::t!("settings-ai-file-based-mcp-toggle"),
                builder(SettingsAction::AI(AISettingsPageAction::ToggleFileBasedMcp)),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::FILE_BASED_MCP_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| {
                FeatureFlag::McpServer.is_enabled()
                    && FeatureFlag::FileBasedMcp.is_enabled()
                    && ContextFlag::ShowMCPServers.is_enabled()
            }),
        ],
        app,
    );
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-warp-credit-fallback"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleCanUseWarpCreditsForFallback,
                )),
                &(context.clone() & id!(flags::IS_ANY_AI_ENABLED)),
                flags::WARP_CREDIT_FALLBACK_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .is_supported_on_current_platform(
                UserWorkspaces::as_ref(app).is_byo_api_key_enabled(app)
                    || UserWorkspaces::as_ref(app).is_custom_inference_enabled(app),
            ),
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-auto-toggle-rich-input"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleAutoToggleRichInput,
                )),
                &(context.clone() & id!(flags::CLI_AGENT_FOOTER_ENABLED)),
                flags::AUTO_TOGGLE_RICH_INPUT_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| FeatureFlag::CLIAgentRichInput.is_enabled()),
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-auto-open-rich-input"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleAutoOpenRichInputOnCLIAgentStart,
                )),
                &(context.clone() & id!(flags::CLI_AGENT_FOOTER_ENABLED)),
                flags::AUTO_OPEN_RICH_INPUT_ON_CLI_AGENT_START_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| FeatureFlag::CLIAgentRichInput.is_enabled()),
            ToggleSettingActionPair::new(
                &crate::t!("toggle-suffix-auto-dismiss-rich-input"),
                builder(SettingsAction::AI(
                    AISettingsPageAction::ToggleAutoDismissRichInputAfterSubmit,
                )),
                &(context.clone() & id!(flags::CLI_AGENT_FOOTER_ENABLED)),
                flags::AUTO_DISMISS_RICH_INPUT_AFTER_SUBMIT_FLAG,
            )
            .with_group(bindings::BindingGroup::WarpAi)
            .with_enabled(|| FeatureFlag::CLIAgentRichInput.is_enabled()),
        ],
        app,
    );
}

pub struct AISettingsPageView {
    page: PageType<Self>,
    active_subpage: Option<AISubpage>,
    voice_input_toggle_key_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,
    voice_input_language_dropdown: ViewHandle<FilterableDropdown<AISettingsPageAction>>,
    local_only_icon_tooltip_states: RefCell<HashMap<String, MouseStateHandle>>,
    autodetection_denylist_editor: ViewHandle<EditorView>,
    autonomy_dropdown_menu: ViewHandle<Dropdown<AISettingsPageAction>>,

    code_read_autonomy_dropdown_menu: ViewHandle<Dropdown<AISettingsPageAction>>,

    code_read_allowlist_editor: ViewHandle<SubmittableTextInput>,
    code_read_allowlist_mouse_state_handles: Vec<MouseStateHandle>,

    command_execution_allowlist_editor: ViewHandle<SubmittableTextInput>,
    command_execution_allowlist_mouse_state_handles: Vec<MouseStateHandle>,
    command_execution_denylist_editor: ViewHandle<SubmittableTextInput>,
    command_execution_denylist_mouse_state_handles: Vec<MouseStateHandle>,
    cli_agent_footer_command_editor: ViewHandle<SubmittableTextInput>,
    cli_agent_footer_command_mouse_state_handles: Vec<MouseStateHandle>,
    cli_agent_footer_command_agent_dropdowns: Vec<ViewHandle<Dropdown<AISettingsPageAction>>>,
    agent_toolbar_inline_editor: ViewHandle<AgentToolbarInlineEditor>,
    cli_agent_toolbar_inline_editor: ViewHandle<AgentToolbarInlineEditor>,

    apply_code_diffs_dropdown_menu: ViewHandle<Dropdown<AISettingsPageAction>>,
    read_files_dropdown_menu: ViewHandle<Dropdown<AISettingsPageAction>>,
    execute_commands_dropdown_menu: ViewHandle<Dropdown<AISettingsPageAction>>,
    write_to_pty_autonomy_dropdown_menu: ViewHandle<Dropdown<AISettingsPageAction>>,
    mcp_permissions_dropdown_menu: ViewHandle<Dropdown<AISettingsPageAction>>,

    // Allowlisting directories (default profile)
    directory_allowlist_mouse_state_handles: Vec<MouseStateHandle>,
    directory_allowlist_editor: ViewHandle<SubmittableTextInput>,

    // Allowlisting commands (default profile)
    command_allowlist_mouse_state_handles: Vec<MouseStateHandle>,
    command_allowlist_editor: ViewHandle<SubmittableTextInput>,

    // Denylisting commands (default profile)
    command_denylist_mouse_state_handles: Vec<MouseStateHandle>,
    command_denylist_tooltip_mouse_state_handles: Vec<MouseStateHandle>,
    command_denylist_editor: ViewHandle<SubmittableTextInput>,

    mcp_allowlist_mouse_state_handles: Vec<MouseStateHandle>,
    mcp_allowlist_dropdown: ViewHandle<FilterableDropdown<AISettingsPageAction>>,

    mcp_denylist_mouse_state_handles: Vec<MouseStateHandle>,
    mcp_denylist_dropdown: ViewHandle<FilterableDropdown<AISettingsPageAction>>,

    base_model_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,
    coding_model_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,

    context_window_slider_state: SliderStateHandle,
    context_window_editor: ViewHandle<EditorView>,
    last_synced_context_window_editor_value: Option<u32>,
    dragged_context_window_value: Option<u32>,

    thinking_display_mode_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,
    orchestration_message_display_mode_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,
    default_prompt_submission_mode_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,
    lrc_submission_mode_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,
    #[cfg(feature = "local_fs")]
    conversation_layout_dropdown: ViewHandle<Dropdown<AISettingsPageAction>>,

    // Profile views
    profile_views: Vec<ViewHandle<ExecutionProfileView>>,
    add_profile_button: ViewHandle<ActionButton>,

    // Custom model router views (gated on FeatureFlag::CustomModelRouters)
    #[cfg(feature = "local_fs")]
    router_views: Vec<ViewHandle<super::custom_router_view::CustomRouterView>>,
    #[cfg(feature = "local_fs")]
    add_router_button: ViewHandle<ActionButton>,

    // Custom inference (custom endpoints)
    custom_endpoint_modal_state: CustomEndpointModalViewState,
    remove_custom_endpoint_confirmation_dialog: ViewHandle<RemoveCustomEndpointConfirmationDialog>,
    pending_remove_custom_endpoint_index: Option<usize>,
    custom_inference_add_button: ViewHandle<ActionButton>,
    custom_endpoint_edit_buttons: Vec<ViewHandle<ActionButton>>,

    // Prompt offering to switch the default Agent Mode model after a BYO key or
    // custom endpoint is saved while the default isn't backed by a credential.
    set_default_model_modal: ModalViewState<Modal<SetDefaultModelModalBody>>,
    // Snapshot of the provider keys from the last `KeysUpdated`, used to detect a
    // newly added key and prompt the user to switch their default model.
    last_seen_provider_keys: ApiKeys,

    // In-flight fallback exchange for a pasted SuperGrok authorization code.
    // This stores only the PKCE verifier clone needed by the manual path while
    // `OauthAttempt::finish` owns the full loopback attempt.
    #[cfg(not(target_family = "wasm"))]
    grok_oauth_attempt: Option<ManualCodeExchange>,
    #[cfg(not(target_family = "wasm"))]
    grok_code_editor: ViewHandle<EditorView>,
}

impl AISettingsPageView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let is_any_ai_enabled = AISettings::as_ref(ctx).is_any_ai_enabled(ctx);

        let workspace = UserWorkspaces::handle(ctx);
        let ai_autonomy_settings = workspace.as_ref(ctx).ai_autonomy_settings();
        ctx.subscribe_to_model(&workspace, |me, workspace, event, ctx| {
            if let UserWorkspacesEvent::TeamsChanged = event {
                me.refresh_all_execution_profile_ui(ctx);
                me.reset_execution_profile_mouse_state_handles(ctx);

                let is_any_ai_enabled = AISettings::as_ref(ctx).is_any_ai_enabled(ctx);
                let ai_autonomy_settings = workspace.as_ref(ctx).ai_autonomy_settings();

                Self::update_editor_interaction_state(
                    me.command_denylist_editor.as_ref(ctx).editor().clone(),
                    is_any_ai_enabled,
                    ctx,
                );

                Self::update_editor_interaction_state(
                    me.command_allowlist_editor.as_ref(ctx).editor().clone(),
                    is_any_ai_enabled
                        && !ai_autonomy_settings.has_override_for_execute_commands_allowlist(),
                    ctx,
                );

                Self::update_editor_interaction_state(
                    me.directory_allowlist_editor.as_ref(ctx).editor().clone(),
                    is_any_ai_enabled
                        && !ai_autonomy_settings.has_override_for_read_files_allowlist(),
                    ctx,
                );

                me.sync_custom_endpoint_buttons(ctx);
                ctx.notify();
            }
        });

        let voice_input_toggle_key_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            if !AISettings::as_ref(ctx).is_voice_input_enabled(ctx) {
                dropdown.set_disabled(ctx);
            }

            let values = VoiceInputToggleKey::all_possible_values();
            let current_value = AISettings::as_ref(ctx).voice_input_toggle_key.value();
            let selected_index = values
                .iter()
                .position(|val| val == current_value)
                .unwrap_or_else(|| {
                    log::warn!(
                        "Could not find current VoiceInputToggleKey value in dropdown option list"
                    );
                    0
                });

            dropdown.add_items(
                values
                    .into_iter()
                    .map(|val| {
                        DropdownItem::new(
                            val.display_name(),
                            AISettingsPageAction::SetVoiceInputToggleKey(val),
                        )
                    })
                    .collect(),
                ctx,
            );
            dropdown.set_selected_by_index(selected_index, ctx);

            dropdown
        });

        let voice_input_language_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = FilterableDropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            if !AISettings::as_ref(ctx).is_voice_input_enabled(ctx) {
                dropdown.set_disabled(ctx);
            }

            dropdown.add_items(
                VOICE_INPUT_LANGUAGES
                    .iter()
                    .map(|&(code, name)| {
                        DropdownItem::new(
                            name,
                            AISettingsPageAction::SetVoiceInputLanguage(code.to_string()),
                        )
                    })
                    .collect(),
                ctx,
            );
            let current_code = AISettings::as_ref(ctx)
                .voice_input_language_code()
                .unwrap_or("")
                .to_string();
            dropdown.set_selected_by_action(
                AISettingsPageAction::SetVoiceInputLanguage(current_code),
                ctx,
            );

            dropdown
        });

        let coding_model_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_menu_max_height(AI_SETTINGS_DROPDOWN_MAX_HEIGHT, ctx);
            dropdown
        });
        Self::refresh_coding_model_menu(&coding_model_dropdown, ctx);

        let base_model_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_menu_max_height(AI_SETTINGS_DROPDOWN_MAX_HEIGHT, ctx);

            dropdown
        });
        Self::refresh_base_model_menu(&base_model_dropdown, ctx);

        let initial_context_window_value = Self::initial_context_window_value(ctx);
        let clamped_initial = Self::configurable_context_window(ctx)
            .map(|cw| initial_context_window_value.clamp(cw.min, cw.max))
            .unwrap_or(initial_context_window_value);
        let context_window_slider_state = SliderStateHandle::default();

        let context_window_editor = ctx.add_typed_action_view(|ctx| {
            let options = SingleLineEditorOptions {
                text: TextOptions {
                    font_size_override: Some(Appearance::as_ref(ctx).ui_font_size()),
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_buffer_text(&clamped_initial.to_string(), ctx);
            editor
        });
        ctx.subscribe_to_view(&context_window_editor, |me, _, event, ctx| {
            me.handle_context_window_editor_event(event, ctx);
        });
        let last_synced_context_window_editor_value = Some(clamped_initial);

        let thinking_display_mode_dropdown =
            OtherAIWidget::create_thinking_display_mode_dropdown(ctx);
        // Set initial selection based on current setting value.
        {
            let current_mode = AISettings::as_ref(ctx).thinking_display_mode;
            thinking_display_mode_dropdown.update(ctx, |dropdown, ctx| {
                dropdown.set_selected_by_action(
                    AISettingsPageAction::SetThinkingDisplayMode(current_mode),
                    ctx,
                );
            });
        }
        let orchestration_message_display_mode_dropdown =
            OtherAIWidget::create_orchestration_message_display_mode_dropdown(ctx);
        {
            let current_mode = AISettings::as_ref(ctx).orchestration_message_display_mode;
            orchestration_message_display_mode_dropdown.update(ctx, |dropdown, ctx| {
                dropdown.set_selected_by_action(
                    AISettingsPageAction::SetOrchestrationMessageDisplayMode(current_mode),
                    ctx,
                );
            });
        }

        let default_prompt_submission_mode_dropdown =
            OtherAIWidget::create_default_prompt_submission_mode_dropdown(ctx);
        {
            let current_mode = AISettings::as_ref(ctx).default_prompt_submission_mode;
            default_prompt_submission_mode_dropdown.update(ctx, |dropdown, ctx| {
                dropdown.set_selected_by_action(
                    AISettingsPageAction::SetPromptSubmissionMode(current_mode),
                    ctx,
                );
            });
        }

        let lrc_submission_mode_dropdown = OtherAIWidget::create_lrc_submission_mode_dropdown(ctx);
        {
            let current_mode = AISettings::as_ref(ctx).long_running_command_submission_mode;
            lrc_submission_mode_dropdown.update(ctx, |dropdown, ctx| {
                dropdown.set_selected_by_action(
                    AISettingsPageAction::SetLongRunningCommandSubmissionMode(current_mode),
                    ctx,
                );
            });
        }

        let autonomy_dropdown_menu = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown
        });
        Self::refresh_autonomy_dropdown_menu(&autonomy_dropdown_menu, ctx);

        let code_read_autonomy_dropdown_menu = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown
        });
        Self::refresh_code_read_autonomy_dropdown_menu(&code_read_autonomy_dropdown_menu, ctx);

        // While the data model supports arbitrary files in the allowlist,
        // it's most intuitive to allowlist whole directories.
        let code_read_allowlist_editor = ctx.add_typed_action_view(|ctx| {
            let mut input = SubmittableTextInput::new(ctx).validate_on_submit(|s| {
                let expanded = host_native_absolute_path(s, &None, &None);
                Path::new(&expanded).is_dir()
            });
            input.set_placeholder_text(crate::t!("settings-ai-repo-placeholder"), ctx);
            input
        });
        Self::update_editor_interaction_state(
            code_read_allowlist_editor.as_ref(ctx).editor().clone(),
            is_any_ai_enabled,
            ctx,
        );

        ctx.subscribe_to_view(&code_read_allowlist_editor, |_, _, event, ctx| {
            if let SubmittableTextInputEvent::Submit(s) = event {
                let expanded = host_native_absolute_path(s, &None, &None);
                BlocklistAIPermissions::handle(ctx).update(ctx, |model, ctx| {
                    report_if_error!(
                        model.add_filepath_to_code_read_allowlist(PathBuf::from(expanded), ctx)
                    );
                });
            }
        });

        let autodetection_denylist_editor = ctx.add_typed_action_view(|ctx| {
            let appearance = Appearance::as_ref(ctx);
            let options = EditorOptions {
                autogrow: true,
                soft_wrap: true,
                text: TextOptions {
                    font_size_override: Some(appearance.ui_font_size()),
                    font_family_override: Some(appearance.monospace_font_family()),
                    text_colors_override: Some(TextColors {
                        default_color: appearance.theme().active_ui_text_color(),
                        disabled_color: appearance.theme().disabled_ui_text_color(),
                        hint_color: appearance.theme().disabled_ui_text_color(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut editor = EditorView::new(options, ctx);

            editor.set_placeholder_text(
                crate::t!("settings-ai-commands-comma-separated-placeholder"),
                ctx,
            );

            let current_value = AISettings::as_ref(ctx)
                .autodetection_command_denylist
                .value()
                .clone();
            editor.set_buffer_text(current_value.as_str(), ctx);
            editor
        });
        Self::update_editor_interaction_state(
            autodetection_denylist_editor.clone(),
            is_any_ai_enabled,
            ctx,
        );

        ctx.subscribe_to_view(&autodetection_denylist_editor, move |me, _, event, ctx| {
            me.handle_detection_denylist_editor_event(event, ctx);
        });

        let command_execution_allowlist_editor = ctx.add_typed_action_view(|ctx| {
            let mut input =
                SubmittableTextInput::new(ctx).validate_on_edit(|s| Regex::new(s).is_ok());
            input.set_placeholder_text(crate::t!("settings-ai-regex-example-placeholder"), ctx);
            input
        });
        Self::update_editor_interaction_state(
            command_execution_allowlist_editor
                .as_ref(ctx)
                .editor()
                .clone(),
            is_any_ai_enabled,
            ctx,
        );

        ctx.subscribe_to_view(&command_execution_allowlist_editor, |_, _, event, ctx| {
            if let SubmittableTextInputEvent::Submit(s) = event {
                let predicate = match AgentModeCommandExecutionPredicate::new_regex(s) {
                    Ok(regex) => regex,
                    Err(e) => {
                        log::warn!(
                            "Failed to convert string to regex for cmd execution allowlist: {e}"
                        );
                        return;
                    }
                };
                BlocklistAIPermissions::handle(ctx).update(ctx, |model, ctx| {
                    report_if_error!(model.add_command_to_autoexecution_allowlist(predicate, ctx));
                })
            }
        });

        let command_execution_denylist_editor = ctx.add_typed_action_view(|ctx| {
            let mut input =
                SubmittableTextInput::new(ctx).validate_on_edit(|s| Regex::new(s).is_ok());
            input.set_placeholder_text(crate::t!("settings-ai-regex-example-placeholder"), ctx);
            input
        });
        Self::update_editor_interaction_state(
            command_execution_denylist_editor
                .as_ref(ctx)
                .editor()
                .clone(),
            is_any_ai_enabled,
            ctx,
        );

        ctx.subscribe_to_view(&command_execution_denylist_editor, |_, _, event, ctx| {
            if let SubmittableTextInputEvent::Submit(s) = event {
                let predicate = match AgentModeCommandExecutionPredicate::new_regex(s) {
                    Ok(regex) => regex,
                    Err(e) => {
                        log::warn!(
                            "Failed to convert string to regex for cmd execution denylist: {e}"
                        );
                        return;
                    }
                };
                BlocklistAIPermissions::handle(ctx).update(ctx, |model, ctx| {
                    report_if_error!(model.add_command_to_autoexecution_denylist(predicate, ctx));
                })
            }
        });

        let cli_agent_footer_command_editor = ctx.add_typed_action_view(|ctx| {
            let mut input =
                SubmittableTextInput::new(ctx).validate_on_edit(|s| Regex::new(s).is_ok());
            input.set_placeholder_text(
                crate::t!("settings-ai-command-supports-regex-placeholder"),
                ctx,
            );
            input
        });
        // The coding agent footer command editor is always enabled,
        // independent of the global AI toggle, because it controls
        // third-party coding agents rather than Zap's own AI.
        Self::update_editor_interaction_state(
            cli_agent_footer_command_editor.as_ref(ctx).editor().clone(),
            true,
            ctx,
        );
        ctx.subscribe_to_view(
            &cli_agent_footer_command_editor,
            |_, _, event, ctx| match event {
                SubmittableTextInputEvent::Submit(command) => {
                    AISettings::handle(ctx).update(ctx, |settings, ctx| {
                        settings.add_cli_agent_footer_enabled_command(command, ctx);
                    });
                }
                SubmittableTextInputEvent::Escape => ctx.emit(AISettingsPageEvent::FocusModal),
            },
        );

        let request_usage_model = AIRequestUsageModel::handle(ctx);
        ctx.subscribe_to_model(&request_usage_model, |_, _, _, ctx| {
            ctx.notify();
        });

        ctx.subscribe_to_model(&UserWorkspaces::handle(ctx), |me, _handle, _event, ctx| {
            // Re-render if teams-related data changed that may affect whether features such as voice input are enabled.
            Self::refresh_base_model_menu(&me.base_model_dropdown, ctx);
            Self::refresh_coding_model_menu(&me.coding_model_dropdown, ctx);
            me.sync_custom_endpoint_buttons(ctx);
            ctx.notify();
        });

        ctx.subscribe_to_model(
            &AIExecutionProfilesModel::handle(ctx),
            |me, _, event, ctx| {
                match event {
                    AIExecutionProfilesModelEvent::ProfileCreated
                    | AIExecutionProfilesModelEvent::ProfileDeleted => {
                        me.refresh_profile_views(ctx);
                    }
                    AIExecutionProfilesModelEvent::ProfileUpdated(_) => {
                        me.refresh_all_execution_profile_ui(ctx);
                        me.reset_execution_profile_mouse_state_handles(ctx);
                        me.sync_context_window_editor(ctx, false);
                    }
                    AIExecutionProfilesModelEvent::UpdatedActiveProfile { .. } => (),
                }
                ctx.notify();
            },
        );

        let object_store_model = ObjectStoreModel::handle(ctx);
        ctx.subscribe_to_model(&object_store_model, |me, _, event, ctx| {
            let added_or_deleted_mcp_servers = matches!(
                event,
                ObjectStoreEvent::ObjectCreated { type_and_id } | ObjectStoreEvent::ObjectDeleted { type_and_id, .. }
                if matches!(
                    type_and_id.object_type(),
                    ObjectType::GenericStringObject(Json(JsonObjectType::MCPServer))
                )
            );

            if added_or_deleted_mcp_servers {
                Self::refresh_mcp_allowlist_dropdown(&me.mcp_allowlist_dropdown, ctx);
                Self::refresh_mcp_denylist_dropdown(&me.mcp_denylist_dropdown, ctx);
                ctx.notify();
            }
        });

        let templatable_manager = TemplatableMCPServerManager::handle(ctx);
        ctx.subscribe_to_model(&templatable_manager, |me, _, _event, ctx| {
            Self::refresh_mcp_allowlist_dropdown(&me.mcp_allowlist_dropdown, ctx);
            Self::refresh_mcp_denylist_dropdown(&me.mcp_denylist_dropdown, ctx);
            ctx.notify();
        });

        ctx.subscribe_to_model(
            &LLMPreferences::handle(ctx),
            |me, _, event, ctx| match event {
                LLMPreferencesEvent::UpdatedAvailableLLMs => {
                    Self::refresh_base_model_menu(&me.base_model_dropdown, ctx);
                    Self::refresh_coding_model_menu(&me.coding_model_dropdown, ctx);
                    me.sync_context_window_editor(ctx, false);
                }
                LLMPreferencesEvent::UpdatedActiveAgentModeLLM => {
                    Self::refresh_base_model_menu(&me.base_model_dropdown, ctx);
                    me.sync_context_window_editor(ctx, false);
                }
                LLMPreferencesEvent::UpdatedActiveCodingLLM => {
                    Self::refresh_coding_model_menu(&me.coding_model_dropdown, ctx);
                }
                LLMPreferencesEvent::UpdatedReasoningEffort => {}
            },
        );

        // Refresh model dropdowns when BYO API keys update so key icons reflect latest state.
        ctx.subscribe_to_model(&ApiKeyManager::handle(ctx), |me, _model, _event, ctx| {
            Self::refresh_base_model_menu(&me.base_model_dropdown, ctx);
            Self::refresh_coding_model_menu(&me.coding_model_dropdown, ctx);
            me.sync_context_window_editor(ctx, false);
            me.sync_custom_endpoint_buttons(ctx);
            // Driving the prompt off the key-store update (rather than the editor's
            // blur/Enter) means it fires reliably however the key was committed —
            // clicking outside the field, pressing Enter, or tabbing away.
            me.maybe_prompt_for_newly_added_provider_key(ctx);
            ctx.notify();
        });

        ctx.subscribe_to_model(&AISettings::handle(ctx), |me, _, event, ctx| {
            match event {
                AISettingsChangedEvent::AICommandDenylist { .. } => {
                    me.autodetection_denylist_editor.update(ctx, |editor, ctx| {
                        let denylist_value = &AISettings::as_ref(ctx)
                            .autodetection_command_denylist
                            .value()
                            .clone();
                        editor.set_buffer_text(denylist_value, ctx);
                    });
                }
                AISettingsChangedEvent::IsAnyAIEnabled { .. } => {
                    let is_enabled = AISettings::as_ref(ctx).is_any_ai_enabled(ctx);
                    let ai_autonomy_settings = UserWorkspaces::as_ref(ctx).ai_autonomy_settings();

                    Self::update_editor_interaction_state(
                        me.autodetection_denylist_editor.clone(),
                        is_enabled,
                        ctx,
                    );
                    Self::update_editor_interaction_state(
                        me.command_execution_allowlist_editor
                            .as_ref(ctx)
                            .editor()
                            .clone(),
                        is_enabled,
                        ctx,
                    );
                    Self::update_editor_interaction_state(
                        me.command_execution_denylist_editor
                            .as_ref(ctx)
                            .editor()
                            .clone(),
                        is_enabled,
                        ctx,
                    );
                    Self::update_editor_interaction_state(
                        me.code_read_allowlist_editor.as_ref(ctx).editor().clone(),
                        is_enabled,
                        ctx,
                    );

                    Self::update_editor_interaction_state(
                        me.directory_allowlist_editor.as_ref(ctx).editor().clone(),
                        is_enabled && !ai_autonomy_settings.has_override_for_read_files_allowlist(),
                        ctx,
                    );

                    Self::update_editor_interaction_state(
                        me.command_denylist_editor.as_ref(ctx).editor().clone(),
                        is_enabled,
                        ctx,
                    );

                    Self::update_editor_interaction_state(
                        me.command_allowlist_editor.as_ref(ctx).editor().clone(),
                        is_enabled
                            && !ai_autonomy_settings.has_override_for_execute_commands_allowlist(),
                        ctx,
                    );

                    me.update_voice_input_dropdown_enablement(ctx);
                    Self::refresh_autonomy_dropdown_menu(&me.autonomy_dropdown_menu, ctx);

                    me.refresh_all_execution_profile_ui(ctx);

                    Self::refresh_code_read_autonomy_dropdown_menu(
                        &me.code_read_autonomy_dropdown_menu,
                        ctx,
                    );
                    Self::refresh_base_model_menu(&me.base_model_dropdown, ctx);
                    Self::refresh_coding_model_menu(&me.coding_model_dropdown, ctx);
                    Self::refresh_mcp_allowlist_dropdown(&me.mcp_allowlist_dropdown, ctx);
                    Self::refresh_mcp_denylist_dropdown(&me.mcp_denylist_dropdown, ctx);
                    me.sync_context_window_editor(ctx, true);
                    me.sync_custom_endpoint_buttons(ctx);
                }
                AISettingsChangedEvent::VoiceInputEnabled { .. } => {
                    me.update_voice_input_dropdown_enablement(ctx);
                }
                AISettingsChangedEvent::AgentModeExecuteReadonlyCommands { .. } => {
                    Self::refresh_autonomy_dropdown_menu(&me.autonomy_dropdown_menu, ctx);
                    Self::refresh_code_read_autonomy_dropdown_menu(
                        &me.code_read_autonomy_dropdown_menu,
                        ctx,
                    );
                }
                AISettingsChangedEvent::AgentModeCodingPermissions { .. } => {
                    Self::refresh_code_read_autonomy_dropdown_menu(
                        &me.code_read_autonomy_dropdown_menu,
                        ctx,
                    );
                }
                AISettingsChangedEvent::VoiceInputToggleKey { .. } => {
                    let current_value = AISettings::as_ref(ctx)
                        .voice_input_toggle_key
                        .value()
                        .display_name();
                    me.voice_input_toggle_key_dropdown
                        .update(ctx, |dropdown, ctx| {
                            dropdown.set_selected_by_name(current_value, ctx)
                        });
                }
                AISettingsChangedEvent::VoiceInputLanguage { .. } => {
                    let current_code = AISettings::as_ref(ctx)
                        .voice_input_language_code()
                        .unwrap_or("")
                        .to_string();
                    me.voice_input_language_dropdown
                        .update(ctx, |dropdown, ctx| {
                            dropdown.set_selected_by_action(
                                AISettingsPageAction::SetVoiceInputLanguage(current_code),
                                ctx,
                            )
                        });
                }
                AISettingsChangedEvent::AgentModeCommandExecutionAllowlist { .. } => {
                    me.command_execution_allowlist_mouse_state_handles = AISettings::as_ref(ctx)
                        .agent_mode_command_execution_allowlist
                        .value()
                        .iter()
                        .map(|_| Default::default())
                        .collect();
                }
                AISettingsChangedEvent::AgentModeCommandExecutionDenylist { .. } => {
                    me.command_execution_denylist_mouse_state_handles = AISettings::as_ref(ctx)
                        .agent_mode_command_execution_denylist
                        .value()
                        .iter()
                        .map(|_| Default::default())
                        .collect();
                }
                AISettingsChangedEvent::AgentModeCodingFileReadAllowlist { .. } => {
                    me.code_read_allowlist_mouse_state_handles = AISettings::as_ref(ctx)
                        .agent_mode_coding_file_read_allowlist
                        .value()
                        .iter()
                        .map(|_| Default::default())
                        .collect();
                }
                AISettingsChangedEvent::CLIAgentToolbarEnabledCommands { .. } => {
                    me.cli_agent_footer_command_mouse_state_handles = AISettings::as_ref(ctx)
                        .cli_agent_footer_enabled_commands
                        .value()
                        .keys()
                        .map(|_| Default::default())
                        .collect();
                    me.cli_agent_footer_command_agent_dropdowns =
                        Self::create_cli_agent_dropdowns(ctx);
                }
                AISettingsChangedEvent::ThinkingDisplayMode { .. } => {
                    let current_mode = *AISettings::as_ref(ctx).thinking_display_mode.value();
                    me.thinking_display_mode_dropdown
                        .update(ctx, |dropdown, ctx| {
                            dropdown.set_selected_by_action(
                                AISettingsPageAction::SetThinkingDisplayMode(current_mode),
                                ctx,
                            );
                        });
                }
                AISettingsChangedEvent::OrchestrationMessageDisplayMode { .. } => {
                    let current_mode = AISettings::as_ref(ctx).orchestration_message_display_mode;
                    me.orchestration_message_display_mode_dropdown
                        .update(ctx, |dropdown, ctx| {
                            dropdown.set_selected_by_action(
                                AISettingsPageAction::SetOrchestrationMessageDisplayMode(
                                    current_mode,
                                ),
                                ctx,
                            );
                        });
                }
                AISettingsChangedEvent::PromptSubmissionMode { .. } => {
                    let current_mode = AISettings::as_ref(ctx).default_prompt_submission_mode;
                    me.default_prompt_submission_mode_dropdown
                        .update(ctx, |dropdown, ctx| {
                            dropdown.set_selected_by_action(
                                AISettingsPageAction::SetPromptSubmissionMode(current_mode),
                                ctx,
                            );
                        });
                }
                AISettingsChangedEvent::LongRunningCommandSubmissionMode { .. } => {
                    let current_mode = AISettings::as_ref(ctx).long_running_command_submission_mode;
                    me.lrc_submission_mode_dropdown
                        .update(ctx, |dropdown, ctx| {
                            dropdown.set_selected_by_action(
                                AISettingsPageAction::SetLongRunningCommandSubmissionMode(
                                    current_mode,
                                ),
                                ctx,
                            );
                        });
                }
                _ => (),
            }
            ctx.notify();
        });

        ctx.subscribe_to_model(&SessionSettings::handle(ctx), |_, _, event, ctx| {
            if let SessionSettingsChangedEvent::ShowModelSelectorsInPrompt { .. } = event {
                ctx.notify();
            }
        });

        ctx.subscribe_to_model(&InputSettings::handle(ctx), |_, _, _, ctx| {
            ctx.notify();
        });

        // CLI agent 安装扫描完成后刷新设置页（per-agent 表格出现）
        ctx.subscribe_to_model(
            &CLIAgentInstallModel::handle(ctx),
            |_, _, CLIAgentInstallEvent::ScanComplete, ctx| {
                ctx.notify();
            },
        );

        let current_permission =
            BlocklistAIPermissions::as_ref(ctx).active_permissions_profile(ctx, None);

        let apply_code_diffs_dropdown_menu = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);

            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-agent-decides"),
                        AISettingsPageAction::SetApplyCodeDiffs(ActionPermission::AgentDecides),
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-always-allow"),
                        AISettingsPageAction::SetApplyCodeDiffs(ActionPermission::AlwaysAllow),
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-always-ask"),
                        AISettingsPageAction::SetApplyCodeDiffs(ActionPermission::AlwaysAsk),
                    ),
                ],
                ctx,
            );
            dropdown
        });
        Self::refresh_execution_profile_dropdown_menu(
            &apply_code_diffs_dropdown_menu,
            current_permission.apply_code_diffs,
            !AISettings::as_ref(ctx).is_code_diffs_permissions_editable(ctx),
            ctx,
        );

        let read_files_dropdown_menu = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-agent-decides"),
                        AISettingsPageAction::SetReadFiles(ActionPermission::AgentDecides),
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-always-allow"),
                        AISettingsPageAction::SetReadFiles(ActionPermission::AlwaysAllow),
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-always-ask"),
                        AISettingsPageAction::SetReadFiles(ActionPermission::AlwaysAsk),
                    ),
                ],
                ctx,
            );
            dropdown
        });
        Self::refresh_execution_profile_dropdown_menu(
            &read_files_dropdown_menu,
            current_permission.read_files,
            !AISettings::as_ref(ctx).is_read_files_permissions_editable(ctx),
            ctx,
        );

        let execute_commands_dropdown_menu = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-agent-decides"),
                        AISettingsPageAction::SetExecuteCommands(ActionPermission::AgentDecides),
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-always-allow"),
                        AISettingsPageAction::SetExecuteCommands(ActionPermission::AlwaysAllow),
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-always-ask"),
                        AISettingsPageAction::SetExecuteCommands(ActionPermission::AlwaysAsk),
                    ),
                ],
                ctx,
            );
            dropdown
        });
        Self::refresh_execution_profile_dropdown_menu(
            &execute_commands_dropdown_menu,
            current_permission.execute_commands,
            !AISettings::as_ref(ctx).is_execute_commands_permissions_editable(ctx),
            ctx,
        );

        let write_to_pty_autonomy_dropdown_menu = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-always-allow"),
                        AISettingsPageAction::SetWriteToPty(WriteToPtyPermission::AlwaysAllow),
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-always-ask"),
                        AISettingsPageAction::SetWriteToPty(WriteToPtyPermission::AlwaysAsk),
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-ask-on-first-write"),
                        AISettingsPageAction::SetWriteToPty(WriteToPtyPermission::AskOnFirstWrite),
                    ),
                ],
                ctx,
            );
            dropdown
        });
        Self::refresh_write_to_pty_dropdown_menu(
            &write_to_pty_autonomy_dropdown_menu,
            current_permission.write_to_pty,
            !AISettings::as_ref(ctx).is_write_to_pty_permissions_editable(ctx),
            ctx,
        );

        let mcp_permissions_dropdown_menu = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_items(
                vec![
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-agent-decides"),
                        AISettingsPageAction::SetMCPPermissions(ActionPermission::AgentDecides),
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-always-allow"),
                        AISettingsPageAction::SetMCPPermissions(ActionPermission::AlwaysAllow),
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-always-ask"),
                        AISettingsPageAction::SetMCPPermissions(ActionPermission::AlwaysAsk),
                    ),
                ],
                ctx,
            );
            dropdown
        });
        Self::refresh_execution_profile_dropdown_menu(
            &mcp_permissions_dropdown_menu,
            current_permission.mcp_permissions,
            !AISettings::as_ref(ctx).is_mcp_permission_editable(ctx),
            ctx,
        );

        let mcp_allowlist_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = FilterableDropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_menu_header_to_static(Box::leak(
                crate::t!("settings-ai-mcp-dropdown-header").into_boxed_str(),
            ));
            dropdown
        });
        Self::refresh_mcp_allowlist_dropdown(&mcp_allowlist_dropdown, ctx);
        let mcp_allowlist_mouse_state_handles = BlocklistAIPermissions::as_ref(ctx)
            .get_mcp_allowlist(ctx, None)
            .iter()
            .map(|_| Default::default())
            .collect();

        let mcp_denylist_dropdown = ctx.add_typed_action_view(|ctx| {
            let mut dropdown = FilterableDropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_menu_header_to_static(Box::leak(
                crate::t!("settings-ai-mcp-dropdown-header").into_boxed_str(),
            ));
            dropdown
        });
        Self::refresh_mcp_denylist_dropdown(&mcp_denylist_dropdown, ctx);
        let mcp_denylist_mouse_state_handles = BlocklistAIPermissions::as_ref(ctx)
            .get_mcp_denylist(ctx, None)
            .iter()
            .map(|_| Default::default())
            .collect();

        let command_execution_allowlist_mouse_state_handles = AISettings::as_ref(ctx)
            .agent_mode_command_execution_allowlist
            .value()
            .iter()
            .map(|_| Default::default())
            .collect();

        let command_execution_denylist_mouse_state_handles = AISettings::as_ref(ctx)
            .agent_mode_command_execution_denylist
            .value()
            .iter()
            .map(|_| Default::default())
            .collect();
        let cli_agent_footer_command_mouse_state_handles = AISettings::as_ref(ctx)
            .cli_agent_footer_enabled_commands
            .value()
            .keys()
            .map(|_| Default::default())
            .collect();

        let code_read_allowlist_mouse_state_handles = AISettings::as_ref(ctx)
            .agent_mode_coding_file_read_allowlist
            .value()
            .iter()
            .map(|_| Default::default())
            .collect();

        let directory_allowlist_mouse_state_handles = current_permission
            .directory_allowlist
            .iter()
            .map(|_| Default::default())
            .collect();

        let directory_allowlist_editor = ctx.add_typed_action_view(|ctx| {
            let mut input = SubmittableTextInput::new(ctx).validate_on_submit(|s| {
                let expanded = host_native_absolute_path(s, &None, &None);
                Path::new(&expanded).is_dir()
            });
            input.set_placeholder_text(crate::t!("settings-ai-repo-placeholder"), ctx);
            input
        });

        Self::update_editor_interaction_state(
            directory_allowlist_editor.as_ref(ctx).editor().clone(),
            is_any_ai_enabled,
            ctx,
        );

        ctx.subscribe_to_view(&directory_allowlist_editor, |_, _, event, ctx| {
            if let SubmittableTextInputEvent::Submit(s) = event {
                let expanded = host_native_absolute_path(s, &None, &None);
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    let profile_id = profile.id();

                    model.add_to_directory_allowlist(profile_id, &PathBuf::from(expanded), ctx);
                });
                ctx.notify();
            }
        });

        let org_denylist = BlocklistAIPermissions::get_org_execute_commands_denylist(ctx);
        let command_denylist_mouse_state_handles = current_permission
            .command_denylist
            .iter()
            .map(|_| Default::default())
            .collect();
        let command_denylist_tooltip_mouse_state_handles: Vec<MouseStateHandle> =
            org_denylist.iter().map(|_| Default::default()).collect();

        let command_denylist_editor = ctx.add_typed_action_view(|ctx| {
            let mut input =
                SubmittableTextInput::new(ctx).validate_on_edit(|s| Regex::new(s).is_ok());
            input.set_placeholder_text(crate::t!("settings-ai-regex-example-placeholder"), ctx);
            input
        });
        Self::update_editor_interaction_state(
            command_denylist_editor.as_ref(ctx).editor().clone(),
            is_any_ai_enabled && !ai_autonomy_settings.has_override_for_execute_commands_denylist(),
            ctx,
        );

        ctx.subscribe_to_view(&command_denylist_editor, |_, _, event, ctx| {
            if let SubmittableTextInputEvent::Submit(s) = event {
                let predicate = match AgentModeCommandExecutionPredicate::new_regex(s) {
                    Ok(regex) => regex,
                    Err(e) => {
                        log::warn!(
                            "Failed to convert string to regex for cmd execution denylist: {e}"
                        );
                        return;
                    }
                };
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    let profile_id = profile.id();
                    model.add_to_command_denylist(profile_id, &predicate, ctx);
                });
                ctx.notify();
            }
        });

        let command_allowlist_mouse_state_handles = current_permission
            .command_allowlist
            .iter()
            .map(|_| Default::default())
            .collect();

        let command_allowlist_editor = ctx.add_typed_action_view(|ctx| {
            let mut input =
                SubmittableTextInput::new(ctx).validate_on_edit(|s| Regex::new(s).is_ok());
            input.set_placeholder_text(crate::t!("settings-ai-regex-example-placeholder"), ctx);
            input
        });
        Self::update_editor_interaction_state(
            command_allowlist_editor.as_ref(ctx).editor().clone(),
            is_any_ai_enabled
                && !ai_autonomy_settings.has_override_for_execute_commands_allowlist(),
            ctx,
        );

        ctx.subscribe_to_view(&command_allowlist_editor, |_, _, event, ctx| {
            if let SubmittableTextInputEvent::Submit(s) = event {
                let predicate = match AgentModeCommandExecutionPredicate::new_regex(s) {
                    Ok(regex) => regex,
                    Err(e) => {
                        log::warn!(
                            "Failed to convert string to regex for cmd execution allowlist: {e}"
                        );
                        return;
                    }
                };
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    let profile_id = profile.id();
                    model.add_to_command_allowlist(profile_id, &predicate, ctx);
                });
                ctx.notify();
            }
        });

        let ai_request_model = AIRequestUsageModel::handle(ctx);
        ctx.subscribe_to_model(&ai_request_model, |me, _, event, ctx| {
            match event {
                AIRequestUsageModelEvent::RequestUsageUpdated => ctx.notify(),
                AIRequestUsageModelEvent::CreditAvailabilityUpdated => ctx.notify(),
                AIRequestUsageModelEvent::RequestBonusRefunded { .. } => ctx.notify(),
                AIRequestUsageModelEvent::AmbientCreditsBannerDismissed => {}
            }
            Self::refresh_base_model_menu(&me.base_model_dropdown, ctx);
            Self::refresh_coding_model_menu(&me.coding_model_dropdown, ctx);
        });

        let profile_views = Self::create_profile_views(ctx);

        // Custom model router views
        #[cfg(feature = "local_fs")]
        let router_views = Self::create_router_views(ctx);
        #[cfg(feature = "local_fs")]
        let add_router_button = ctx.add_typed_action_view(|_| {
            ActionButton::new(crate::t!("settings-ai-add-router"), SecondaryTheme)
                .with_size(ButtonSize::Small)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(AISettingsPageAction::OpenAddCustomRouter);
                })
        });
        #[cfg(feature = "local_fs")]
        {
            let is_enabled = warp_core::features::FeatureFlag::CustomModelRouters.is_enabled()
                && is_any_ai_enabled;
            add_router_button.update(ctx, |button, ctx| {
                button.set_disabled(!is_enabled, ctx);
            });
        }

        let add_profile_button = ctx.add_typed_action_view(|_| {
            ActionButton::new(crate::t!("settings-ai-add-profile"), SecondaryTheme)
                .with_icon(Icon::Plus)
                .with_size(ButtonSize::Small)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(AISettingsPageAction::CreateProfile);
                })
        });

        add_profile_button.update(ctx, |button, ctx| {
            button.set_disabled(!is_any_ai_enabled, ctx);
        });

        // Custom inference
        let custom_inference_controls_enabled = is_any_ai_enabled
            && UserWorkspaces::as_ref(ctx).is_custom_inference_enabled(ctx)
            && UserWorkspaces::as_ref(ctx).are_member_byo_endpoints_allowed();
        let custom_inference_add_button = ctx.add_typed_action_view(|_| {
            ActionButton::new(
                crate::t!("settings-agent-providers-add-custom-endpoint"),
                SecondaryTheme,
            )
            .with_size(ButtonSize::Small)
            .on_click(|ctx| {
                ctx.dispatch_typed_action(AISettingsPageAction::OpenAddCustomEndpointModal);
            })
        });
        custom_inference_add_button.update(ctx, |button, ctx| {
            button.set_disabled(!custom_inference_controls_enabled, ctx);
        });

        let custom_endpoint_modal_body =
            ctx.add_typed_action_view(|ctx| CustomEndpointModal::new(None, None, ctx));
        ctx.subscribe_to_view(&custom_endpoint_modal_body, |me, _, event, ctx| {
            me.handle_custom_endpoint_modal_event(event, ctx);
        });

        let custom_endpoint_modal_view = ctx.add_typed_action_view(|ctx| {
            Modal::new(
                Some(crate::t!(
                    "settings-agent-providers-add-custom-endpoint-title"
                )),
                custom_endpoint_modal_body.clone(),
                ctx,
            )
            .with_modal_style(UiComponentStyles {
                width: Some(560.),
                ..Default::default()
            })
            .with_header_style(UiComponentStyles {
                padding: Some(Coords {
                    top: 24.,
                    bottom: 0.,
                    left: 24.,
                    right: 24.,
                }),
                font_size: Some(16.),
                font_weight: Some(Weight::Bold),
                ..Default::default()
            })
            .with_body_style(UiComponentStyles {
                padding: Some(Coords {
                    top: 0.,
                    bottom: 24.,
                    left: 24.,
                    right: 0.,
                }),
                ..Default::default()
            })
            .with_background_opacity(100)
            .with_max_height_percentage(CUSTOM_ENDPOINT_MODAL_MAX_HEIGHT_PERCENTAGE)
            .with_dismiss_on_click()
            .with_dismiss_keystroke(Keystroke::parse("escape").unwrap())
        });
        ctx.subscribe_to_view(&custom_endpoint_modal_view, |me, _, event, ctx| {
            me.handle_custom_endpoint_modal_close_event(event, ctx);
        });

        let custom_endpoint_modal_state =
            CustomEndpointModalViewState::new(ModalViewState::new(custom_endpoint_modal_view));

        let set_default_model_modal_body = ctx.add_typed_action_view(SetDefaultModelModalBody::new);
        ctx.subscribe_to_view(&set_default_model_modal_body, |me, _, event, ctx| {
            me.handle_set_default_model_modal_event(event, ctx);
        });
        let set_default_model_modal_view = ctx.add_typed_action_view(|ctx| {
            Modal::new(
                Some(crate::t!("settings-agent-providers-change-default-title")),
                set_default_model_modal_body.clone(),
                ctx,
            )
            .with_modal_style(UiComponentStyles {
                width: Some(480.),
                height: Some(380.),
                ..Default::default()
            })
            .with_body_style(UiComponentStyles {
                height: Some(300.),
                ..Default::default()
            })
            .with_background_opacity(100)
            .with_dismiss_on_click()
            .with_dismiss_keystroke(Keystroke::parse("escape").unwrap())
        });
        ctx.subscribe_to_view(
            &set_default_model_modal_view,
            |me, _, event, ctx| match event {
                ModalEvent::Close => me.hide_set_default_model_modal(ctx),
            },
        );
        let set_default_model_modal = ModalViewState::new(set_default_model_modal_view);
        let last_seen_provider_keys = ApiKeyManager::as_ref(ctx).keys().clone();

        let remove_custom_endpoint_confirmation_dialog =
            ctx.add_typed_action_view(RemoveCustomEndpointConfirmationDialog::new);
        ctx.subscribe_to_view(
            &remove_custom_endpoint_confirmation_dialog,
            |me, _, event, ctx| {
                me.handle_remove_custom_endpoint_confirmation_dialog_event(event, ctx);
            },
        );

        let custom_endpoint_edit_buttons = Self::create_custom_endpoint_edit_buttons(
            ApiKeyManager::as_ref(ctx).keys().custom_endpoints.len(),
            custom_inference_controls_enabled,
            ctx,
        );

        let agent_toolbar_inline_editor = ctx.add_typed_action_view(|ctx| {
            AgentToolbarInlineEditor::new(AgentToolbarEditorMode::AgentView, ctx)
        });
        let cli_agent_toolbar_inline_editor = ctx.add_typed_action_view(|ctx| {
            AgentToolbarInlineEditor::new(AgentToolbarEditorMode::CLIAgent, ctx)
        });

        #[cfg(feature = "local_fs")]
        let conversation_layout_dropdown = ctx.add_typed_action_view(|ctx| {
            use crate::util::file::external_editor::settings::OpenConversationPreference;

            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);

            let items = vec![
                DropdownItem::new(
                    crate::t!("settings-ai-conversation-layout-newtab"),
                    AISettingsPageAction::SetConversationLayout(OpenConversationPreference::NewTab),
                ),
                DropdownItem::new(
                    crate::t!("settings-ai-conversation-layout-splitpane"),
                    AISettingsPageAction::SetConversationLayout(
                        OpenConversationPreference::SplitPane,
                    ),
                ),
            ];
            dropdown.set_items(items, ctx);

            let current = *crate::util::file::external_editor::EditorSettings::as_ref(ctx)
                .open_conversation_layout_preference;
            match current {
                OpenConversationPreference::NewTab => dropdown
                    .set_selected_by_name(crate::t!("settings-ai-conversation-layout-newtab"), ctx),
                OpenConversationPreference::SplitPane => dropdown.set_selected_by_name(
                    crate::t!("settings-ai-conversation-layout-splitpane"),
                    ctx,
                ),
            };
            dropdown
        });

        #[cfg(not(target_family = "wasm"))]
        let grok_code_editor = Self::create_grok_code_editor(ctx);
        #[cfg(not(target_family = "wasm"))]
        ctx.subscribe_to_view(&grok_code_editor, |me, _, event, ctx| {
            if matches!(event, EditorEvent::Enter | EditorEvent::Paste) {
                let code = me.grok_code_editor.as_ref(ctx).buffer_text(ctx);
                me.submit_grok_code(code, ctx);
            }
        });
        // Keep the snapshotted editor text colors in sync with theme changes,
        // like the API key editors above.
        #[cfg(not(target_family = "wasm"))]
        {
            let grok_code_editor = grok_code_editor.clone();
            ctx.subscribe_to_model(&Appearance::handle(ctx), move |_, _, event, ctx| {
                if let AppearanceEvent::ThemeChanged = event {
                    let colors = editor_text_colors(Appearance::as_ref(ctx));
                    grok_code_editor.update(ctx, move |editor, ctx| {
                        editor.set_text_colors(colors, ctx);
                    });
                }
            });
        }
        // Subscribe to WarpConfig to refresh router views when files change.
        #[cfg(feature = "local_fs")]
        ctx.subscribe_to_model(
            &crate::user_config::WarpConfig::handle(ctx),
            |me, _, event, ctx| {
                use crate::user_config::WarpConfigUpdateEvent;
                if matches!(event, WarpConfigUpdateEvent::ModelConfigs) {
                    me.router_views = Self::create_router_views(ctx);
                    ctx.notify();
                }
            },
        );

        Self {
            page: Self::build_page(None, ctx),
            active_subpage: None,
            voice_input_toggle_key_dropdown,
            voice_input_language_dropdown,
            autodetection_denylist_editor,
            local_only_icon_tooltip_states: Default::default(),
            command_execution_allowlist_editor,
            command_execution_denylist_editor,
            command_execution_allowlist_mouse_state_handles,
            command_execution_denylist_mouse_state_handles,
            cli_agent_footer_command_editor,
            cli_agent_footer_command_mouse_state_handles,
            cli_agent_footer_command_agent_dropdowns: Self::create_cli_agent_dropdowns(ctx),
            agent_toolbar_inline_editor,
            cli_agent_toolbar_inline_editor,
            base_model_dropdown,
            coding_model_dropdown,
            context_window_slider_state,
            context_window_editor,
            last_synced_context_window_editor_value,
            dragged_context_window_value: None,
            autonomy_dropdown_menu,
            code_read_allowlist_editor,
            code_read_autonomy_dropdown_menu,
            code_read_allowlist_mouse_state_handles,
            apply_code_diffs_dropdown_menu,
            read_files_dropdown_menu,
            execute_commands_dropdown_menu,
            write_to_pty_autonomy_dropdown_menu,
            mcp_permissions_dropdown_menu,
            directory_allowlist_mouse_state_handles,
            directory_allowlist_editor,
            command_denylist_mouse_state_handles,
            command_denylist_tooltip_mouse_state_handles,
            command_denylist_editor,
            command_allowlist_mouse_state_handles,
            command_allowlist_editor,
            mcp_allowlist_dropdown,
            mcp_allowlist_mouse_state_handles,
            mcp_denylist_dropdown,
            mcp_denylist_mouse_state_handles,
            thinking_display_mode_dropdown,
            orchestration_message_display_mode_dropdown,
            default_prompt_submission_mode_dropdown,
            lrc_submission_mode_dropdown,
            #[cfg(feature = "local_fs")]
            conversation_layout_dropdown,
            profile_views,
            add_profile_button,
            #[cfg(feature = "local_fs")]
            router_views,
            #[cfg(feature = "local_fs")]
            add_router_button,
            custom_endpoint_modal_state,
            remove_custom_endpoint_confirmation_dialog,
            pending_remove_custom_endpoint_index: None,
            custom_inference_add_button,
            custom_endpoint_edit_buttons,
            set_default_model_modal,
            last_seen_provider_keys,
            #[cfg(not(target_family = "wasm"))]
            grok_oauth_attempt: None,
            #[cfg(not(target_family = "wasm"))]
            grok_code_editor,
        }
    }

    fn update_voice_input_dropdown_enablement(&mut self, ctx: &mut ViewContext<Self>) {
        let is_voice_enabled = AISettings::as_ref(ctx).is_voice_input_enabled(ctx);
        self.voice_input_toggle_key_dropdown
            .update(ctx, |dropdown, ctx| {
                if is_voice_enabled {
                    dropdown.set_enabled(ctx);
                } else {
                    dropdown.set_disabled(ctx);
                }
            });
        self.voice_input_language_dropdown
            .update(ctx, |dropdown, ctx| {
                if is_voice_enabled {
                    dropdown.set_enabled(ctx);
                } else {
                    dropdown.set_disabled(ctx);
                }
            });
        ctx.notify();
    }

    pub fn get_modal_content(&self, app: &AppContext) -> Option<Box<dyn Element>> {
        if self.custom_endpoint_modal_state.is_open() {
            Some(self.custom_endpoint_modal_state.render())
        } else if self.set_default_model_modal.is_open() {
            Some(self.set_default_model_modal.render())
        } else if self
            .remove_custom_endpoint_confirmation_dialog
            .as_ref(app)
            .is_visible()
        {
            Some(ChildView::new(&self.remove_custom_endpoint_confirmation_dialog).finish())
        } else {
            None
        }
    }

    fn handle_set_default_model_modal_event(
        &mut self,
        event: &SetDefaultModelModalBodyEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            SetDefaultModelModalBodyEvent::Close => self.hide_set_default_model_modal(ctx),
            SetDefaultModelModalBodyEvent::SetDefault(id) => {
                // Mirror `AISettingsPageAction::SetBaseModel`: set the active
                // profile's base model and clear any stale context-window limit.
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    let profile_id = profiles_model.active_profile(None, ctx).id().clone();
                    profiles_model.set_base_model(&profile_id, Some(id.clone()), ctx);
                    profiles_model.set_context_window_limit(&profile_id, None, ctx);
                });
                self.sync_context_window_editor(ctx, true);
                self.hide_set_default_model_modal(ctx);

                let window_id = ctx.window_id();
                crate::ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    let toast = crate::view_components::DismissibleToast::success(crate::t!(
                        "settings-agent-providers-default-model-updated"
                    ));
                    toast_stack.add_ephemeral_toast(toast, window_id, ctx);
                });
                ctx.notify();
            }
        }
    }

    fn hide_set_default_model_modal(&mut self, ctx: &mut ViewContext<Self>) {
        self.set_default_model_modal.close();
        ctx.emit(AISettingsPageEvent::HideModal);
        ctx.notify();
    }

    fn show_set_default_model_modal(
        &mut self,
        description: String,
        choices: Vec<(LLMId, String)>,
        ctx: &mut ViewContext<Self>,
    ) {
        self.set_default_model_modal.view.update(ctx, |modal, ctx| {
            modal.body().update(ctx, |body, ctx| {
                body.set_choices(description, choices, ctx);
            });
        });
        self.set_default_model_modal.open();
        // Focus the modal so Escape closes it (the modal's escape binding only
        // fires while something inside the modal holds focus).
        ctx.focus(&self.set_default_model_modal.view);
        ctx.emit(AISettingsPageEvent::ShowModal);
        ctx.notify();
    }

    /// Returns `true` when the active Agent Mode default model is already served
    /// by a credential the user has: a BYO key/subscription for its provider, or
    /// one of their custom-endpoint models. `auto` models report `false` since
    /// they always consume Warp credits.
    fn active_base_model_is_byo_covered(ctx: &AppContext) -> bool {
        let (active_id, active_provider) = {
            let prefs = LLMPreferences::as_ref(ctx);
            let active = prefs.get_active_base_model(ctx, None);
            (active.id.clone(), active.provider)
        };
        if LLMPreferences::as_ref(ctx)
            .custom_llm_info_for_id(&active_id)
            .is_some()
        {
            return true;
        }
        is_using_api_key_for_provider(&active_provider, ctx)
    }

    /// The display name of the user's current default Agent Mode model, used in
    /// the prompt copy (e.g. "auto (cost-efficient)").
    fn active_base_model_display_name(ctx: &AppContext) -> String {
        LLMPreferences::as_ref(ctx)
            .get_active_base_model(ctx, None)
            .display_name
            .clone()
    }

    /// Whether to offer switching the default model. Scoped to free-plan users
    /// who are out of monthly (base-plan) credits, since only they hit the
    /// "no credits" error with an `auto` model. Also skips when the current
    /// default is already served by a BYO credential.
    fn should_offer_default_model_switch(ctx: &AppContext) -> bool {
        // Exclude only confirmed paid plans. Solo/individual users have no
        // `current_workspace`, and billing may not have loaded yet (Unknown), so
        // treat both as eligible and rely on the out-of-credits check below to
        // filter anyone who can still run Warp-hosted models. (A strict
        // `is_free_plan()` check here meant solo free users — the common case —
        // never saw the prompt.)
        let on_paid_plan = UserWorkspaces::as_ref(ctx)
            .current_workspace()
            .is_some_and(|workspace| workspace.billing_metadata.is_user_on_paid_plan());
        // Zap:上游用 `has_base_plan_requests_remaining()` 区分"月度基础额度"与
        // "附加额度"。我方 `AIRequestUsageModel` 是无云端计量的本地 stub,只有
        // `has_requests_remaining()`(恒为 true),因此这里恒为 false ——
        // BYOP 本地运行不会因额度耗尽去劝用户换默认模型。
        let out_of_monthly_credits = !AIRequestUsageModel::as_ref(ctx).has_requests_remaining();
        !on_paid_plan && out_of_monthly_credits && !Self::active_base_model_is_byo_covered(ctx)
    }

    /// Detects a provider key that was just added (absent -> present) by diffing
    /// against the last-seen keys, then offers to switch the default model. Run
    /// from `ApiKeyManagerEvent::KeysUpdated` so it fires regardless of how the
    /// key editor was committed.
    fn maybe_prompt_for_newly_added_provider_key(&mut self, ctx: &mut ViewContext<Self>) {
        let current = ApiKeyManager::as_ref(ctx).keys().clone();
        let newly_added = LLMProvider::API_KEY_PROVIDERS.into_iter().find(|provider| {
            let was_present = provider
                .api_key(&self.last_seen_provider_keys)
                .is_some_and(|key| !key.trim().is_empty());
            let now_present = provider
                .api_key(&current)
                .is_some_and(|key| !key.trim().is_empty());
            !was_present && now_present
        });
        self.last_seen_provider_keys = current;
        if let Some(provider) = newly_added {
            self.maybe_prompt_set_default_model_for_provider(provider, ctx);
        }
    }

    /// After a BYO provider key is added, offer to switch the default Agent Mode
    /// model to one from that provider.
    fn maybe_prompt_set_default_model_for_provider(
        &mut self,
        provider: LLMProvider,
        ctx: &mut ViewContext<Self>,
    ) {
        // Only prompt when the key is actually usable for requests (BYO enabled).
        if !is_using_api_key_for_provider(&provider, ctx) {
            return;
        }
        if !Self::should_offer_default_model_switch(ctx) {
            return;
        }
        let choices: Vec<(LLMId, String)> = LLMPreferences::as_ref(ctx)
            .get_base_llm_choices_for_agent_mode(ctx)
            .filter(|llm| llm.provider == provider)
            .map(|llm| (llm.id.clone(), llm.menu_display_name()))
            .collect();
        if choices.is_empty() {
            return;
        }
        let provider_name = provider.display_name();
        let current_default = Self::active_base_model_display_name(ctx);
        let description = crate::t!(
            "settings-agent-providers-change-default-provider-description",
            provider = provider_name,
            model = current_default.as_str()
        );
        self.show_set_default_model_modal(description, choices, ctx);
    }

    /// After a custom endpoint is added or saved, offer to switch the default
    /// Agent Mode model to one of its models.
    fn maybe_prompt_set_default_model_for_custom_endpoint(
        &mut self,
        endpoint_index: usize,
        ctx: &mut ViewContext<Self>,
    ) {
        if !Self::can_use_custom_inference_controls(ctx) {
            return;
        }
        if !Self::should_offer_default_model_switch(ctx) {
            return;
        }
        let Some(endpoint) = ApiKeyManager::as_ref(ctx)
            .keys()
            .custom_endpoints
            .get(endpoint_index)
            .cloned()
        else {
            return;
        };
        // Build directly from the endpoint's models rather than the synthetic
        // `custom_llms`, which are rebuilt asynchronously on `KeysUpdated`.
        let choices: Vec<(LLMId, String)> = endpoint
            .models
            .iter()
            .filter(|m| !m.name.trim().is_empty() && !m.config_key.is_empty())
            .map(|m| {
                (
                    LLMId::from(m.config_key.clone()),
                    m.display_label().to_string(),
                )
            })
            .collect();
        if choices.is_empty() {
            return;
        }
        let current_default = Self::active_base_model_display_name(ctx);
        let description = crate::t!(
            "settings-agent-providers-change-default-endpoint-description",
            endpoint = endpoint.name.as_str(),
            model = current_default.as_str()
        );
        self.show_set_default_model_modal(description, choices, ctx);
    }

    fn sync_custom_endpoint_buttons(&mut self, ctx: &mut ViewContext<Self>) {
        let enabled = Self::can_use_custom_inference_controls(ctx);

        self.custom_inference_add_button.update(ctx, |button, ctx| {
            button.set_disabled(!enabled, ctx);
        });

        let endpoint_count = ApiKeyManager::as_ref(ctx).keys().custom_endpoints.len();
        if self.custom_endpoint_edit_buttons.len() != endpoint_count {
            self.custom_endpoint_edit_buttons =
                Self::create_custom_endpoint_edit_buttons(endpoint_count, enabled, ctx);
        } else {
            for button in &self.custom_endpoint_edit_buttons {
                button.update(ctx, |button, ctx| {
                    button.set_disabled(!enabled, ctx);
                });
            }
        }
    }

    fn create_custom_endpoint_edit_buttons(
        count: usize,
        enabled: bool,
        ctx: &mut ViewContext<Self>,
    ) -> Vec<ViewHandle<ActionButton>> {
        (0..count)
            .map(|index| {
                let button = ctx.add_typed_action_view(move |_| {
                    ActionButton::new(crate::t!("common-edit"), SecondaryTheme)
                        .with_icon(Icon::Pencil)
                        .with_size(ButtonSize::Small)
                        .on_click(move |ctx| {
                            ctx.dispatch_typed_action(
                                AISettingsPageAction::OpenEditCustomEndpointModal(index),
                            );
                        })
                });
                button.update(ctx, |button, ctx| {
                    button.set_disabled(!enabled, ctx);
                });
                button
            })
            .collect()
    }
    fn can_use_custom_inference_controls(app: &AppContext) -> bool {
        AISettings::as_ref(app).is_any_ai_enabled(app)
            && UserWorkspaces::as_ref(app).is_custom_inference_enabled(app)
            && UserWorkspaces::as_ref(app).are_member_byo_endpoints_allowed()
    }

    fn show_add_custom_endpoint_modal(&mut self, ctx: &mut ViewContext<Self>) {
        if !Self::can_use_custom_inference_controls(ctx) {
            return;
        }
        self.remove_custom_endpoint_confirmation_dialog
            .update(ctx, |dialog, ctx| {
                dialog.hide(ctx);
            });
        self.pending_remove_custom_endpoint_index = None;

        self.custom_endpoint_modal_state.set_title(
            Some(crate::t!(
                "settings-agent-providers-add-custom-endpoint-title"
            )),
            ctx,
        );
        self.custom_endpoint_modal_state.prefill(None, None, ctx);
        self.custom_endpoint_modal_state.open(ctx);
        ctx.emit(AISettingsPageEvent::ShowModal);
        ctx.notify();
    }

    fn show_edit_custom_endpoint_modal(&mut self, index: usize, ctx: &mut ViewContext<Self>) {
        if !Self::can_use_custom_inference_controls(ctx) {
            return;
        }
        let endpoint = ApiKeyManager::as_ref(ctx)
            .keys()
            .custom_endpoints
            .get(index)
            .cloned();
        if endpoint.is_none() {
            return;
        }

        self.remove_custom_endpoint_confirmation_dialog
            .update(ctx, |dialog, ctx| {
                dialog.hide(ctx);
            });
        self.pending_remove_custom_endpoint_index = None;

        self.custom_endpoint_modal_state.set_title(
            Some(crate::t!(
                "settings-agent-providers-edit-custom-endpoint-title"
            )),
            ctx,
        );
        self.custom_endpoint_modal_state
            .prefill(endpoint.as_ref(), Some(index), ctx);
        self.custom_endpoint_modal_state.open(ctx);
        ctx.emit(AISettingsPageEvent::ShowModal);
        ctx.notify();
    }

    fn hide_custom_endpoint_modal(&mut self, ctx: &mut ViewContext<Self>) {
        self.custom_endpoint_modal_state.close(ctx);
        ctx.emit(AISettingsPageEvent::HideModal);
        ctx.notify();
    }

    fn handle_custom_endpoint_modal_close_event(
        &mut self,
        event: &ModalEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            ModalEvent::Close => {
                self.hide_custom_endpoint_modal(ctx);
            }
        }
    }

    fn handle_custom_endpoint_modal_event(
        &mut self,
        event: &CustomEndpointModalEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            CustomEndpointModalEvent::Close => {
                self.hide_custom_endpoint_modal(ctx);
            }
            CustomEndpointModalEvent::AddEndpoint {
                name,
                url,
                api_key,
                schema,
                models,
            } => {
                if !Self::can_use_custom_inference_controls(ctx) {
                    self.hide_custom_endpoint_modal(ctx);
                    return;
                }
                ApiKeyManager::handle(ctx).update(ctx, |manager, ctx| {
                    manager.add_custom_endpoint(
                        CustomEndpointParams {
                            name: name.clone(),
                            url: url.clone(),
                            api_key: api_key.clone(),
                            models: models.clone(),
                            schema: *schema,
                        },
                        ctx,
                    );
                });
                self.hide_custom_endpoint_modal(ctx);

                let window_id = ctx.window_id();
                crate::ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    let toast = crate::view_components::DismissibleToast::success(crate::t!(
                        "settings-agent-providers-endpoint-added"
                    ));
                    toast_stack.add_ephemeral_toast(toast, window_id, ctx);
                });

                // The new endpoint is appended last.
                let new_index = ApiKeyManager::as_ref(ctx)
                    .keys()
                    .custom_endpoints
                    .len()
                    .saturating_sub(1);
                self.maybe_prompt_set_default_model_for_custom_endpoint(new_index, ctx);
                ctx.notify();
            }
            CustomEndpointModalEvent::SaveEndpoint {
                index,
                name,
                url,
                api_key,
                schema,
                models,
            } => {
                if !Self::can_use_custom_inference_controls(ctx) {
                    self.hide_custom_endpoint_modal(ctx);
                    return;
                }
                ApiKeyManager::handle(ctx).update(ctx, |manager, ctx| {
                    manager.save_custom_endpoint(
                        *index,
                        CustomEndpointParams {
                            name: name.clone(),
                            url: url.clone(),
                            api_key: api_key.clone(),
                            models: models.clone(),
                            schema: *schema,
                        },
                        ctx,
                    );
                });
                self.hide_custom_endpoint_modal(ctx);

                let window_id = ctx.window_id();
                crate::ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    let toast = crate::view_components::DismissibleToast::success(crate::t!(
                        "settings-agent-providers-endpoint-saved"
                    ));
                    toast_stack.add_ephemeral_toast(toast, window_id, ctx);
                });
                self.maybe_prompt_set_default_model_for_custom_endpoint(*index, ctx);
                ctx.notify();
            }
            CustomEndpointModalEvent::RemoveEndpoint { index } => {
                if !Self::can_use_custom_inference_controls(ctx) {
                    self.hide_custom_endpoint_modal(ctx);
                    return;
                }
                self.hide_custom_endpoint_modal(ctx);
                self.show_remove_custom_endpoint_confirmation_dialog(*index, ctx);
            }
        }
    }

    fn show_remove_custom_endpoint_confirmation_dialog(
        &mut self,
        index: usize,
        ctx: &mut ViewContext<Self>,
    ) {
        if !Self::can_use_custom_inference_controls(ctx) {
            return;
        }
        let endpoint = ApiKeyManager::as_ref(ctx)
            .keys()
            .custom_endpoints
            .get(index)
            .cloned();
        let Some(endpoint) = endpoint else {
            return;
        };

        let model_labels = endpoint
            .models
            .iter()
            .map(|model| model.alias.clone().unwrap_or_else(|| model.name.clone()))
            .filter(|s| !s.trim().is_empty())
            .collect();

        self.pending_remove_custom_endpoint_index = Some(index);
        self.remove_custom_endpoint_confirmation_dialog
            .update(ctx, |dialog, ctx| {
                dialog.show(index, endpoint.name.clone(), model_labels, ctx);
            });
        ctx.notify();
    }

    fn handle_remove_custom_endpoint_confirmation_dialog_event(
        &mut self,
        event: &RemoveCustomEndpointConfirmationDialogEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            RemoveCustomEndpointConfirmationDialogEvent::Cancel => {
                self.pending_remove_custom_endpoint_index = None;
                self.remove_custom_endpoint_confirmation_dialog
                    .update(ctx, |dialog, ctx| {
                        dialog.hide(ctx);
                    });
                ctx.notify();
            }
            RemoveCustomEndpointConfirmationDialogEvent::Confirm(index) => {
                if !Self::can_use_custom_inference_controls(ctx) {
                    self.pending_remove_custom_endpoint_index = None;
                    self.remove_custom_endpoint_confirmation_dialog
                        .update(ctx, |dialog, ctx| {
                            dialog.hide(ctx);
                        });
                    ctx.notify();
                    return;
                }
                ApiKeyManager::handle(ctx).update(ctx, |manager, ctx| {
                    manager.remove_custom_endpoint(*index, ctx);
                });
                self.pending_remove_custom_endpoint_index = None;
                self.remove_custom_endpoint_confirmation_dialog
                    .update(ctx, |dialog, ctx| {
                        dialog.hide(ctx);
                    });
                self.sync_custom_endpoint_buttons(ctx);

                let window_id = ctx.window_id();
                crate::ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    let toast = crate::view_components::DismissibleToast::success(crate::t!(
                        "settings-agent-providers-endpoint-removed"
                    ));
                    toast_stack.add_ephemeral_toast(toast, window_id, ctx);
                });
                ctx.notify();
            }
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn create_grok_code_editor(ctx: &mut ViewContext<Self>) -> ViewHandle<EditorView> {
        ctx.add_typed_action_view(|ctx| {
            let appearance = Appearance::handle(ctx).as_ref(ctx);
            let options = SingleLineEditorOptions {
                text: TextOptions {
                    font_size_override: Some(appearance.ui_font_size()),
                    font_family_override: Some(appearance.monospace_font_family()),
                    text_colors_override: Some(editor_text_colors(appearance)),
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text(crate::t!("settings-ai-grok-code-placeholder"), ctx);
            editor
        })
    }

    /// Kicks off the xAI (Grok) subscription OAuth flow: opens the consent
    /// screen in the browser, runs a loopback PKCE callback server, exchanges
    /// the resulting authorization code for OAuth tokens, and persists them via
    /// `ApiKeyManager` (which then proactively refreshes them before expiry).
    ///
    /// In parallel, this reveals the manual code-entry row so the user can
    /// paste the code xAI displays when the browser can't reach the loopback
    /// callback. Whichever path completes first connects the subscription; the
    /// other completion is ignored once the view-owned attempt state is cleared.
    #[cfg(not(target_family = "wasm"))]
    fn start_grok_oauth(&mut self, ctx: &mut ViewContext<Self>) {
        use warp_core::safe_error;

        use crate::ToastStack;
        use crate::view_components::{DismissibleToast, ToastLink};
        use crate::workspace::WorkspaceAction;

        /// Object id shared by the connect-flow toasts so the completion toast
        /// (success or error) automatically replaces the in-progress one.
        const CONNECT_TOAST_OBJECT_ID: &str = "grok_oauth_connect_toast";

        // Record attempt initiation on click (before we attempt to bind the
        // loopback server). This ensures every terminal SuperGrokSubscriptionConnectFinished
        // (including immediate bind failures) is paired with a preceding Initiated
        // for funnel/drop-off analysis.
        send_telemetry_from_ctx!(TelemetryEvent::SuperGrokSubscriptionConnectInitiated, ctx);

        // Starting the attempt binds the loopback callback server before the
        // browser opens, so a bind failure surfaces immediately, without a
        // dangling browser tab.
        let attempt = match oauth::OauthAttempt::start() {
            Ok(attempt) => attempt,
            Err(err) => {
                safe_error!(
                    safe: ("Failed to start Grok OAuth callback server"),
                    full: ("Failed to start Grok OAuth callback server: {err:#}")
                );
                send_telemetry_from_ctx!(
                    TelemetryEvent::SuperGrokSubscriptionConnectFinished {
                        error: Some("bind_failed".to_string()),
                    },
                    ctx
                );
                let window_id = ctx.window_id();
                ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    let error = err.to_string();
                    let toast = DismissibleToast::error(crate::t!(
                        "settings-ai-grok-start-failed",
                        error = error.as_str()
                    ));
                    toast_stack.add_ephemeral_toast(toast, window_id, ctx);
                });
                return;
            }
        };

        // Capture the PKCE verifier so the fallback is ready if xAI shows a
        // code instead of redirecting.
        self.grok_oauth_attempt = Some(attempt.manual_code_exchange());
        self.grok_code_editor.update(ctx, |editor, ctx| {
            editor.clear_buffer(ctx);
        });
        ctx.notify();
        // Open xAI's consent screen in the user's default browser.
        let authorize_url = attempt.authorize_url();
        ctx.open_url(&authorize_url);

        let window_id = ctx.window_id();
        ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
            // Persistent rather than ephemeral so the copy-URL fallback stays
            // available when the browser fails to open. It can't linger
            // forever: the completion toast below replaces it (shared object
            // id), and the OAuth attempt itself times out when the callback
            // never arrives.
            let toast = DismissibleToast::default(crate::t!("settings-ai-grok-opening-browser"))
                .with_object_id(CONNECT_TOAST_OBJECT_ID.to_string())
                .with_link(
                    ToastLink::new(crate::t!("settings-ai-grok-copy-url"))
                        .with_onclick_action(WorkspaceAction::CopyTextToClipboard(authorize_url)),
                );
            toast_stack.add_persistent_toast(toast, window_id, ctx);
        });

        ctx.spawn(async move { attempt.finish().await }, |me, result, ctx| {
            // Ignore loopback completion after a successful pasted-code path.
            if me.grok_oauth_attempt.is_none() {
                return;
            }
            let window_id = ctx.window_id();
            let toast = match result {
                Ok(tokens) => {
                    me.grok_oauth_attempt = None;
                    me.grok_code_editor.update(ctx, |editor, ctx| {
                        editor.clear_buffer(ctx);
                    });
                    send_telemetry_from_ctx!(
                        TelemetryEvent::SuperGrokSubscriptionConnectFinished { error: None },
                        ctx
                    );
                    // Persist the tokens to secure storage and kick off the
                    // proactive refresh loop so subsequent requests can
                    // authenticate with the connected subscription.
                    ApiKeyManager::handle(ctx).update(ctx, move |manager, ctx| {
                        manager.store_grok_tokens(tokens, ctx);
                    });
                    DismissibleToast::success(crate::t!("settings-ai-grok-connected"))
                }
                Err(err) => {
                    me.grok_oauth_attempt = None;
                    me.grok_code_editor.update(ctx, |editor, ctx| {
                        editor.clear_buffer(ctx);
                    });
                    safe_error!(
                        safe: ("Grok OAuth loopback callback failed"),
                        full: ("Grok OAuth loopback callback failed: {err:#}")
                    );
                    send_telemetry_from_ctx!(
                        TelemetryEvent::SuperGrokSubscriptionConnectFinished {
                            error: Some("loopback_failed".to_string()),
                        },
                        ctx
                    );
                    let error = err.to_string();
                    DismissibleToast::error(crate::t!(
                        "settings-ai-grok-connect-failed",
                        error = error.as_str()
                    ))
                }
            };
            ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                toast_stack.add_ephemeral_toast(
                    toast.with_object_id(CONNECT_TOAST_OBJECT_ID.to_string()),
                    window_id,
                    ctx,
                );
            });
            ctx.notify();
        });
    }

    /// Exchanges a pasted SuperGrok authorization code using the current
    /// attempt's PKCE verifier.
    #[cfg(not(target_family = "wasm"))]
    fn submit_grok_code(&mut self, code: String, ctx: &mut ViewContext<Self>) {
        use warp_core::safe_error;

        use crate::ToastStack;
        use crate::view_components::DismissibleToast;

        // Shared with the browser connect-flow toasts.
        const CONNECT_TOAST_OBJECT_ID: &str = "grok_oauth_connect_toast";
        let Some(exchange) = self.grok_oauth_attempt.clone() else {
            return;
        };
        if code.trim().is_empty() {
            return;
        }

        ctx.spawn(
            async move { exchange.exchange(&code).await },
            |me, result, ctx| {
                if me.grok_oauth_attempt.is_none() {
                    return;
                }
                let window_id = ctx.window_id();
                let toast = match result {
                    Ok(tokens) => {
                        me.grok_oauth_attempt = None;
                        me.grok_code_editor.update(ctx, |editor, ctx| {
                            editor.clear_buffer(ctx);
                        });
                        send_telemetry_from_ctx!(
                            TelemetryEvent::SuperGrokSubscriptionConnectFinished { error: None },
                            ctx
                        );
                        ApiKeyManager::handle(ctx).update(ctx, move |manager, ctx| {
                            manager.store_grok_tokens(tokens, ctx);
                        });
                        DismissibleToast::success(crate::t!("settings-ai-grok-connected"))
                    }
                    Err(err) => {
                        // Keep the row open so the user can correct the code.
                        safe_error!(
                            safe: ("Grok manual code exchange failed"),
                            full: ("Grok manual code exchange failed: {err:#}")
                        );
                        send_telemetry_from_ctx!(
                            TelemetryEvent::SuperGrokSubscriptionConnectFinished {
                                error: Some("manual_code_failed".to_string()),
                            },
                            ctx
                        );
                        let error = err.to_string();
                        DismissibleToast::error(crate::t!(
                            "settings-ai-grok-connect-failed",
                            error = error.as_str()
                        ))
                    }
                };
                ToastStack::handle(ctx).update(ctx, |toast_stack, ctx| {
                    toast_stack.add_ephemeral_toast(
                        toast.with_object_id(CONNECT_TOAST_OBJECT_ID.to_string()),
                        window_id,
                        ctx,
                    );
                });
                ctx.notify();
            },
        );
    }

    /// Set the active subpage and rebuild the widget list to show only relevant widgets.
    pub fn set_active_subpage(&mut self, subpage: Option<AISubpage>, ctx: &mut ViewContext<Self>) {
        if self.active_subpage != subpage {
            self.active_subpage = subpage;
            self.page = Self::build_page(subpage, ctx);
            ctx.notify();
        }
    }
    fn knowledge_widgets() -> Vec<Box<dyn SettingsWidget<View = Self>>> {
        let mut widgets: Vec<Box<dyn SettingsWidget<View = Self>>> =
            vec![Box::new(RulesWidget::default())];
        if FeatureFlag::SuggestedRules.is_enabled() {
            widgets.push(Box::new(SuggestedRulesWidget::default()));
        }
        // 去中心化分支:不再渲染 "InfiniShell Drive as agent context" 开关。
        widgets.extend([
            Box::new(ManageRulesWidget::default()) as Box<dyn SettingsWidget<View = Self>>
        ]);
        widgets
    }

    /// 重建当前 subpage 的 widget 列表。
    /// 用于 widget 的内部状态依赖 `AISettings` 中的复杂集合(例如自定义 Agent
    /// Provider 列表),在集合大小变化时需要重新创建 widget 持有的 ViewHandle。
    pub fn rebuild_current_page(&mut self, ctx: &mut ViewContext<Self>) {
        // 复用旧 page 的滚动 handle,避免重建后跳回顶部。
        let preserved_scroll = self.page.scroll_states();
        self.page = Self::build_page(self.active_subpage, ctx);
        if let Some((v, h)) = preserved_scroll {
            self.page.replace_scroll_states(v, h);
        }
        ctx.notify();
    }

    fn build_page(subpage: Option<AISubpage>, ctx: &mut ViewContext<Self>) -> PageType<Self> {
        let ai_settings = AISettings::as_ref(ctx);
        let should_show_usage_widget = !UserWorkspaces::as_ref(ctx).is_byo_api_key_enabled(ctx);

        let mut widgets: Vec<Box<dyn SettingsWidget<View = AISettingsPageView>>> = Vec::new();

        // When viewing a specific subpage, only include its widgets.
        // When subpage is None (legacy/backward-compat), show all widgets.
        match subpage {
            None => {
                // Full page: all widgets (legacy behavior)
                widgets.push(Box::new(WarpAgentHeaderWidget));
                if should_show_usage_widget {
                    widgets.push(Box::new(UsageWidget::new(ctx)));
                }
                if ai_settings
                    .intelligent_autosuggestions_enabled_internal
                    .is_supported_on_current_platform()
                    || ai_settings
                        .prompt_suggestions_enabled_internal
                        .is_supported_on_current_platform()
                    || (FeatureFlag::PredictAMQueries.is_enabled()
                        && ai_settings
                            .natural_language_autosuggestions_enabled_internal
                            .is_supported_on_current_platform())
                    || (FeatureFlag::GitOperationsInCodeReview.is_enabled()
                        && ai_settings
                            .git_operations_autogen_enabled_internal
                            .is_supported_on_current_platform())
                {
                    widgets.push(Box::new(ActiveAIWidget::new(ctx)));
                }
                widgets.push(Box::new(AgentsWidget::default()));
                widgets.push(Box::new(AIInputWidget::default()));
                if MCPServersWidget::should_show_mcp() {
                    widgets.push(Box::new(MCPServersWidget::default()));
                }
                if FeatureFlag::AIRules.is_enabled() {
                    widgets.extend(Self::knowledge_widgets());
                }
                if cfg!(feature = "voice_input")
                    && ai_settings
                        .voice_input_enabled_internal
                        .is_supported_on_current_platform()
                {
                    widgets.push(Box::new(VoiceWidget::default()));
                }
                widgets.extend(cli_agent_widgets());
                widgets.push(Box::new(AwsBedrockWidget::new(ctx)));
                widgets.push(Box::new(GeminiEnterpriseWidget::new(ctx)));
                widgets.push(Box::new(AgentProvidersWidget::new(ctx)));
                widgets.push(Box::new(OtherAIWidget::default()));
            }
            Some(AISubpage::WarpAgent) => {
                // Oz page: header + Active AI + Input + Other
                widgets.push(Box::new(WarpAgentHeaderWidget));
                if ai_settings
                    .intelligent_autosuggestions_enabled_internal
                    .is_supported_on_current_platform()
                    || ai_settings
                        .prompt_suggestions_enabled_internal
                        .is_supported_on_current_platform()
                    || (FeatureFlag::PredictAMQueries.is_enabled()
                        && ai_settings
                            .natural_language_autosuggestions_enabled_internal
                            .is_supported_on_current_platform())
                    || (FeatureFlag::GitOperationsInCodeReview.is_enabled()
                        && ai_settings
                            .git_operations_autogen_enabled_internal
                            .is_supported_on_current_platform())
                {
                    widgets.push(Box::new(ActiveAIWidget::new(ctx)));
                }
                widgets.push(Box::new(AIInputWidget::default()));
                let voice_supported = cfg!(feature = "voice_input")
                    && ai_settings
                        .voice_input_enabled_internal
                        .is_supported_on_current_platform();
                if voice_supported {
                    widgets.push(Box::new(VoiceWidget::default()));
                }
                widgets.push(Box::new(AwsBedrockWidget::new(ctx)));
                widgets.push(Box::new(GeminiEnterpriseWidget::new(ctx)));
                if FeatureFlag::CustomModelRouters.is_enabled() {
                    widgets.push(Box::new(CustomModelRoutersWidget));
                }
                widgets.push(Box::new(OtherAIWidget::default()));
            }
            Some(AISubpage::Providers) => {
                widgets.push(Box::new(AgentProvidersWidget::new(ctx)));
            }
            Some(AISubpage::Profiles) => {
                if should_show_usage_widget {
                    widgets.push(Box::new(UsageWidget::new(ctx)));
                }
                widgets.push(Box::new(AgentsWidget::default()));
            }
            Some(AISubpage::Knowledge) => {
                if FeatureFlag::AIRules.is_enabled() {
                    widgets.extend(Self::knowledge_widgets());
                }
            }
            Some(AISubpage::ThirdPartyCLIAgents) => {
                widgets.extend(cli_agent_widgets());
            }
        }

        // Multi-section subpages (Warp Agent, Profiles) render their own subheader-sized
        // section titles inside each widget, so they get no page-level title here.
        // Single-topic subpages follow the Account-page convention and render their title as
        // page chrome, so filtering their setting widgets never removes the title.
        let title = match subpage {
            Some(AISubpage::Knowledge) => Some(crate::t_static!("settings-ai-knowledge-section")),
            Some(AISubpage::ThirdPartyCLIAgents) => {
                Some(crate::t_static!("settings-ai-third-party-cli-section"))
            }
            // Zap:BYOP 提供商子页由 `AgentProvidersWidget` 自己渲染 sub-header 标题,
            // 与 Warp Agent / Profiles 一样不需要页面级标题。
            None
            | Some(AISubpage::WarpAgent)
            | Some(AISubpage::Profiles)
            | Some(AISubpage::Providers) => None,
        };
        PageType::new_uncategorized(widgets, title)
    }

    fn handle_context_window_editor_event(
        &mut self,
        event: &EditorEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            EditorEvent::Blurred | EditorEvent::Enter => {
                if !AISettings::as_ref(ctx).is_any_ai_enabled(ctx) {
                    self.sync_context_window_editor(ctx, true);
                    return;
                }
                if let Some(cw) = Self::configurable_context_window(ctx) {
                    let buffer_text = self.context_window_editor.as_ref(ctx).buffer_text(ctx);
                    let cleaned: String = buffer_text
                        .chars()
                        .filter(|c| !c.is_whitespace() && *c != ',')
                        .collect();
                    if let Ok(parsed) = cleaned.parse::<u32>() {
                        let clamped = parsed.clamp(cw.min, cw.max);
                        if Some(clamped) != Self::current_context_window_display_value(ctx) {
                            AIExecutionProfilesModel::handle(ctx).update(
                                ctx,
                                |profiles_model, ctx| {
                                    let profile_id =
                                        profiles_model.active_profile(None, ctx).id().clone();
                                    profiles_model.set_context_window_limit(
                                        &profile_id,
                                        Some(clamped),
                                        ctx,
                                    );
                                },
                            );
                        }
                    }
                }
                self.sync_context_window_editor(ctx, true);
                if let EditorEvent::Enter = event {
                    ctx.emit(AISettingsPageEvent::FocusModal);
                }
                ctx.notify();
            }
            EditorEvent::Escape => ctx.emit(AISettingsPageEvent::FocusModal),
            _ => {}
        }
    }

    fn active_profile_data(app: &AppContext) -> AIExecutionProfile {
        AIExecutionProfilesModel::as_ref(app)
            .active_profile(None, app)
            .data()
            .clone()
    }

    fn configurable_context_window(app: &AppContext) -> Option<LLMContextWindow> {
        Self::active_profile_data(app).configurable_context_window(app)
    }

    fn current_context_window_display_value(app: &AppContext) -> Option<u32> {
        Self::active_profile_data(app).context_window_display_value(app)
    }

    fn initial_context_window_value(app: &AppContext) -> u32 {
        Self::current_context_window_display_value(app).unwrap_or_else(|| {
            LLMPreferences::as_ref(app)
                .get_active_base_model(app, None)
                .context_window
                .default_max
        })
    }

    fn sync_context_window_editor(&mut self, ctx: &mut ViewContext<Self>, force: bool) {
        self.dragged_context_window_value = None;
        let Some(value) = Self::current_context_window_display_value(ctx) else {
            self.last_synced_context_window_editor_value = None;
            self.context_window_slider_state.reset_offset();
            ctx.notify();
            return;
        };

        let formatted = value.to_string();
        let should_update = if force {
            true
        } else {
            match self.last_synced_context_window_editor_value {
                Some(last_value) => {
                    self.context_window_editor.as_ref(ctx).buffer_text(ctx)
                        == last_value.to_string()
                }
                None => true,
            }
        };

        if should_update {
            self.context_window_editor.update(ctx, |editor, ctx| {
                if editor.buffer_text(ctx) != formatted {
                    editor.system_reset_buffer_text(&formatted, ctx);
                }
            });
            self.last_synced_context_window_editor_value = Some(value);
            self.context_window_slider_state.reset_offset();
            ctx.notify();
        }
    }

    fn handle_detection_denylist_editor_event(
        &mut self,
        event: &EditorEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        match event {
            EditorEvent::Blurred | EditorEvent::Enter => {
                let buffer_text = self
                    .autodetection_denylist_editor
                    .as_ref(ctx)
                    .buffer_text(ctx);
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(e) = settings
                        .autodetection_command_denylist
                        .set_value(buffer_text, ctx)
                    {
                        log::warn!("Failed to set AI autodetection blacklist commands: {e:?}");
                    }
                })
            }
            EditorEvent::Escape => ctx.emit(AISettingsPageEvent::FocusModal),
            _ => {}
        }
    }

    fn update_editor_interaction_state(
        editor: ViewHandle<EditorView>,
        is_enabled: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        editor.update(ctx, |editor, ctx| {
            let interaction_state = if is_enabled {
                InteractionState::Editable
            } else {
                InteractionState::Disabled
            };
            editor.set_interaction_state(interaction_state, ctx);
            ctx.notify();
        })
    }

    pub fn refresh_base_model_menu(
        menu: &ViewHandle<Dropdown<AISettingsPageAction>>,
        ctx: &mut ViewContext<Self>,
    ) {
        menu.update(ctx, |menu, ctx| {
            let disabled_by_ai_toggle = !AISettings::as_ref(ctx).is_any_ai_enabled(ctx);

            if disabled_by_ai_toggle {
                menu.set_disabled(ctx);
            } else {
                menu.set_enabled(ctx);
            }

            let choices = LLMPreferences::as_ref(ctx)
                .get_base_llm_choices_for_agent_mode(ctx)
                .collect_vec();

            let items = available_model_menu_items(
                choices,
                |llm| {
                    DropdownAction::select_action_and_close(AISettingsPageAction::SetBaseModel(
                        llm.id.clone(),
                    ))
                },
                None,
                None,
                false,
                false,
                ctx,
            );
            menu.set_rich_items(items, ctx);

            let active = LLMPreferences::as_ref(ctx).get_active_base_model(ctx, None);
            menu.set_selected_by_action(AISettingsPageAction::SetBaseModel(active.id.clone()), ctx);
            ctx.notify();
        });
        ctx.notify();
    }

    pub fn refresh_coding_model_menu(
        menu: &ViewHandle<Dropdown<AISettingsPageAction>>,
        ctx: &mut ViewContext<Self>,
    ) {
        menu.update(ctx, |menu, ctx| {
            let disabled_by_ai_toggle = !AISettings::as_ref(ctx).is_any_ai_enabled(ctx);

            if disabled_by_ai_toggle {
                menu.set_disabled(ctx);
            } else {
                menu.set_enabled(ctx);
            }

            let choices = LLMPreferences::as_ref(ctx)
                .get_coding_llm_choices(ctx)
                .collect_vec();

            let items = available_model_menu_items(
                choices,
                |llm| {
                    DropdownAction::select_action_and_close(AISettingsPageAction::SetCodingModel(
                        llm.id.clone(),
                    ))
                },
                None,
                None,
                false,
                false,
                ctx,
            );
            menu.set_rich_items(items, ctx);
            let active = LLMPreferences::as_ref(ctx).get_active_coding_model(ctx, None);

            menu.set_selected_by_action(
                AISettingsPageAction::SetCodingModel(active.id.clone()),
                ctx,
            );
            ctx.notify();
        });
        ctx.notify();
    }

    fn refresh_autonomy_dropdown_menu(
        menu: &ViewHandle<Dropdown<AISettingsPageAction>>,
        ctx: &mut ViewContext<Self>,
    ) {
        menu.update(ctx, |menu, ctx| {
            if AISettings::as_ref(ctx).is_any_ai_enabled(ctx) {
                menu.set_enabled(ctx);
            } else {
                menu.set_disabled(ctx);
            }

            menu.set_items(
                vec![
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-read-only"),
                        AISettingsPageAction::SetAutonomyReadonlyCommandsSetting,
                    ),
                    DropdownItem::new(
                        crate::t!("settings-ai-permission-supervised"),
                        AISettingsPageAction::SetAutonomySupervisedSetting,
                    ),
                ],
                ctx,
            );
            let active = if *AISettings::as_ref(ctx).agent_mode_execute_read_only_commands {
                0
            } else {
                1
            };
            menu.set_selected_by_index(active, ctx);
            ctx.notify();
        });
        ctx.notify();
    }

    fn refresh_all_execution_profile_ui(&self, ctx: &mut ViewContext<Self>) {
        let permissions = BlocklistAIPermissions::handle(ctx);

        let apply_code_diffs_setting = permissions
            .as_ref(ctx)
            .get_apply_code_diffs_setting(ctx, None);
        Self::refresh_execution_profile_dropdown_menu(
            &self.apply_code_diffs_dropdown_menu,
            apply_code_diffs_setting,
            !AISettings::as_ref(ctx).is_code_diffs_permissions_editable(ctx),
            ctx,
        );

        let read_files_setting = permissions.as_ref(ctx).get_read_files_setting(ctx, None);
        Self::refresh_execution_profile_dropdown_menu(
            &self.read_files_dropdown_menu,
            read_files_setting,
            !AISettings::as_ref(ctx).is_read_files_permissions_editable(ctx),
            ctx,
        );

        let execute_commands_setting: ActionPermission = permissions
            .as_ref(ctx)
            .get_execute_commands_setting(ctx, None);
        Self::refresh_execution_profile_dropdown_menu(
            &self.execute_commands_dropdown_menu,
            execute_commands_setting,
            !AISettings::as_ref(ctx).is_execute_commands_permissions_editable(ctx),
            ctx,
        );

        let write_to_pty_setting: WriteToPtyPermission =
            permissions.as_ref(ctx).get_write_to_pty_setting(ctx, None);
        Self::refresh_write_to_pty_dropdown_menu(
            &self.write_to_pty_autonomy_dropdown_menu,
            write_to_pty_setting,
            !AISettings::as_ref(ctx).is_write_to_pty_permissions_editable(ctx),
            ctx,
        );

        let mcp_permissions_setting = permissions
            .as_ref(ctx)
            .get_mcp_permissions_setting(ctx, None);
        Self::refresh_execution_profile_dropdown_menu(
            &self.mcp_permissions_dropdown_menu,
            mcp_permissions_setting,
            !AISettings::as_ref(ctx).is_mcp_permission_editable(ctx),
            ctx,
        );
        Self::refresh_mcp_allowlist_dropdown(&self.mcp_allowlist_dropdown, ctx);
        Self::refresh_mcp_denylist_dropdown(&self.mcp_denylist_dropdown, ctx);

        let is_any_ai_enabled = AISettings::as_ref(ctx).is_any_ai_enabled(ctx);
        self.add_profile_button.update(ctx, |button, ctx| {
            button.set_disabled(!is_any_ai_enabled, ctx);
        });
    }

    fn reset_execution_profile_mouse_state_handles(&mut self, ctx: &mut ViewContext<Self>) {
        let blocklist_permissions = BlocklistAIPermissions::as_ref(ctx);

        self.directory_allowlist_mouse_state_handles = blocklist_permissions
            .get_read_files_allowlist(ctx, None)
            .iter()
            .map(|_| Default::default())
            .collect();

        self.command_denylist_mouse_state_handles = blocklist_permissions
            .get_execute_commands_denylist(ctx, None)
            .iter()
            .map(|_| Default::default())
            .collect();

        let org_denylist = BlocklistAIPermissions::get_org_execute_commands_denylist(ctx);
        self.command_denylist_tooltip_mouse_state_handles =
            org_denylist.iter().map(|_| Default::default()).collect();

        self.command_allowlist_mouse_state_handles = blocklist_permissions
            .get_execute_commands_allowlist(ctx, None)
            .iter()
            .map(|_| Default::default())
            .collect();

        self.mcp_allowlist_mouse_state_handles = blocklist_permissions
            .get_mcp_allowlist(ctx, None)
            .iter()
            .map(|_| Default::default())
            .collect();

        self.mcp_denylist_mouse_state_handles = blocklist_permissions
            .get_mcp_denylist(ctx, None)
            .iter()
            .map(|_| Default::default())
            .collect();
    }

    fn refresh_execution_profile_dropdown_menu(
        menu: &ViewHandle<Dropdown<AISettingsPageAction>>,
        current_permission: ActionPermission,
        disabled: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        menu.update(ctx, |menu, ctx| {
            if !disabled {
                menu.set_enabled(ctx);
            } else {
                menu.set_disabled(ctx);
            }

            let active = match current_permission {
                ActionPermission::AgentDecides | ActionPermission::Unknown => 0,
                ActionPermission::AlwaysAllow => 1,
                ActionPermission::AlwaysAsk => 2,
            };

            menu.set_selected_by_index(active, ctx);
            ctx.notify();
        });
        ctx.notify();
    }

    fn refresh_write_to_pty_dropdown_menu(
        menu: &ViewHandle<Dropdown<AISettingsPageAction>>,
        current_permission: WriteToPtyPermission,
        disabled: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        menu.update(ctx, |menu, ctx| {
            if !disabled {
                menu.set_enabled(ctx);
            } else {
                menu.set_disabled(ctx);
            }

            let active = match current_permission {
                WriteToPtyPermission::AlwaysAllow => 0,
                WriteToPtyPermission::AlwaysAsk | WriteToPtyPermission::Unknown => 1,
                WriteToPtyPermission::AskOnFirstWrite => 2,
            };

            menu.set_selected_by_index(active, ctx);
            ctx.notify();
        });
        ctx.notify();
    }

    /// Currently, the coding permissions only support "read" access.
    fn refresh_code_read_autonomy_dropdown_menu(
        menu: &ViewHandle<Dropdown<AISettingsPageAction>>,
        ctx: &mut ViewContext<Self>,
    ) {
        menu.update(ctx, |menu, ctx| {
            if AISettings::as_ref(ctx).is_any_ai_enabled(ctx) {
                menu.set_enabled(ctx);
            } else {
                menu.set_disabled(ctx);
            }

            menu.set_items(
                AgentModeCodingPermissionsType::iter()
                    .map(|t| {
                        let display = match t {
                            AgentModeCodingPermissionsType::AlwaysAskBeforeReading => {
                                crate::t!("settings-ai-permission-always-ask")
                            }
                            AgentModeCodingPermissionsType::AlwaysAllowReading => {
                                crate::t!("settings-ai-permission-always-allow")
                            }
                            AgentModeCodingPermissionsType::AllowReadingSpecificFiles => {
                                crate::t!("settings-ai-permission-allow-specific-dirs")
                            }
                        };
                        DropdownItem::new(display, AISettingsPageAction::SetCodingPermission(t))
                    })
                    .collect(),
                ctx,
            );
            let ai_settings = AISettings::as_ref(ctx);

            let active = if *ai_settings.agent_mode_execute_read_only_commands {
                menu.set_disabled(ctx);
                AgentModeCodingPermissionsType::AlwaysAllowReading
            } else {
                *ai_settings.agent_mode_coding_permissions
            };
            menu.set_selected_by_action(AISettingsPageAction::SetCodingPermission(active), ctx);
            ctx.notify();
        });
        ctx.notify();
    }

    fn get_non_allowlisted_or_denylisted_mcp_servers(
        ctx: &mut ViewContext<Self>,
    ) -> Vec<(uuid::Uuid, String)> {
        let all_mcp_servers =
            TemplatableMCPServerManager::get_all_templatable_mcp_server_names(ctx);
        let already_allowlisted_mcp_servers =
            BlocklistAIPermissions::as_ref(ctx).get_mcp_allowlist(ctx, None);
        let already_denylisted_mcp_servers =
            BlocklistAIPermissions::as_ref(ctx).get_mcp_denylist(ctx, None);

        all_mcp_servers
            .into_iter()
            .filter(|(uuid, _)| {
                let is_allowlisted = already_allowlisted_mcp_servers.contains(uuid);
                let is_denylisted = already_denylisted_mcp_servers.contains(uuid);
                !is_allowlisted && !is_denylisted
            })
            .collect()
    }

    fn refresh_menu_dropdown<F>(
        menu: &ViewHandle<FilterableDropdown<AISettingsPageAction>>,
        action_fn: F,
        ctx: &mut ViewContext<Self>,
    ) where
        F: Fn(uuid::Uuid) -> AISettingsPageAction,
    {
        let mcps_in_dropdown = Self::get_non_allowlisted_or_denylisted_mcp_servers(ctx);
        menu.update(ctx, |menu, ctx| {
            if AISettings::as_ref(ctx).is_any_ai_enabled(ctx) {
                menu.set_enabled(ctx);
            } else {
                menu.set_disabled(ctx);
            }

            let items: Vec<DropdownItem<AISettingsPageAction>> = mcps_in_dropdown
                .iter()
                .map(|(uuid, server_name)| DropdownItem::new(server_name, action_fn(*uuid)))
                .collect();

            menu.set_items(items, ctx);
            ctx.notify()
        });
        ctx.notify();
    }

    fn refresh_mcp_allowlist_dropdown(
        menu: &ViewHandle<FilterableDropdown<AISettingsPageAction>>,
        ctx: &mut ViewContext<Self>,
    ) {
        Self::refresh_menu_dropdown(menu, AISettingsPageAction::AddToMCPAllowlist, ctx);
    }

    #[cfg(feature = "local_fs")]
    fn create_router_views(
        ctx: &mut ViewContext<Self>,
    ) -> Vec<ViewHandle<super::custom_router_view::CustomRouterView>> {
        use super::custom_router_view::{CustomRouterView, CustomRouterViewEvent};
        use crate::user_config::WarpConfig;
        if !warp_core::features::FeatureFlag::CustomModelRouters.is_enabled() {
            return Vec::new();
        }
        let routers: Vec<crate::ai::custom_model_routers::CustomModelRouter> =
            WarpConfig::as_ref(ctx).custom_model_routers().clone();
        routers
            .into_iter()
            .map(|router| {
                let router_clone = router.clone();
                let view = ctx.add_typed_action_view(|ctx| CustomRouterView::new(router, ctx));
                ctx.subscribe_to_view(&view, move |me, _, event, ctx| match event {
                    CustomRouterViewEvent::OpenFile(path) => {
                        ctx.emit(AISettingsPageEvent::OpenCustomRouterFile(path.clone()));
                    }
                    CustomRouterViewEvent::Edit => {
                        let r = router_clone.clone();
                        ctx.emit(AISettingsPageEvent::OpenCustomRouterEditor(Some(r)));
                    }
                    CustomRouterViewEvent::Delete => {
                        if let Some(path) = &router_clone.source_path {
                            #[cfg(feature = "local_fs")]
                            {
                                if let Err(e) =
                                    crate::user_config::WarpConfig::delete_custom_model_router(path)
                                {
                                    log::warn!("Failed to delete custom router: {e:?}");
                                }
                            }
                            me.router_views = Self::create_router_views(ctx);
                            ctx.notify();
                        }
                    }
                });
                view
            })
            .collect()
    }

    fn create_profile_views(ctx: &mut ViewContext<Self>) -> Vec<ViewHandle<ExecutionProfileView>> {
        let profiles_model = AIExecutionProfilesModel::as_ref(ctx);
        let profile_ids = profiles_model.get_all_profile_ids();

        profile_ids
            .iter()
            .map(|profile_id| {
                let profile_id = profile_id.clone();
                let profile_view = ctx.add_typed_action_view(|ctx| {
                    ExecutionProfileView::new(profile_id.clone(), ctx)
                });
                let profile_id_for_event = profile_id.clone();

                ctx.subscribe_to_view(&profile_view, move |_me, _, event, ctx| match event {
                    ExecutionProfileViewEvent::EditProfile => {
                        ctx.emit(AISettingsPageEvent::OpenExecutionProfileEditor(
                            profile_id_for_event.clone(),
                        ));
                    }
                });

                profile_view
            })
            .collect()
    }

    fn refresh_profile_views(&mut self, ctx: &mut ViewContext<Self>) {
        let new_profile_views = Self::create_profile_views(ctx);
        self.profile_views = new_profile_views;
    }

    fn refresh_mcp_denylist_dropdown(
        menu: &ViewHandle<FilterableDropdown<AISettingsPageAction>>,
        ctx: &mut ViewContext<Self>,
    ) {
        Self::refresh_menu_dropdown(menu, AISettingsPageAction::AddToMCPDenylist, ctx);
    }

    fn create_cli_agent_dropdowns(
        ctx: &mut ViewContext<Self>,
    ) -> Vec<ViewHandle<Dropdown<AISettingsPageAction>>> {
        let entries: Vec<(String, CLIAgent)> = AISettings::as_ref(ctx)
            .cli_agent_footer_enabled_commands
            .value()
            .iter()
            .map(|(pattern, agent_value)| {
                (pattern.clone(), CLIAgent::from_serialized_name(agent_value))
            })
            .collect();

        entries
            .into_iter()
            .map(|(pattern_clone, current_agent)| {
                ctx.add_typed_action_view(move |ctx| {
                    let mut dropdown = Dropdown::new(ctx);
                    dropdown.set_top_bar_max_width(160.);
                    dropdown.set_menu_width(180., ctx);
                    dropdown.set_main_axis_size(MainAxisSize::Min, ctx);

                    let mut items: Vec<MenuItem<DropdownAction>> = Vec::new();

                    for agent in all::<CLIAgent>() {
                        if matches!(agent, CLIAgent::Unknown) {
                            continue;
                        }
                        let icon = agent.icon();
                        let mut fields = MenuItemFields::new(agent.display_name())
                            .with_on_select_action(DropdownAction::select_action_and_close(
                                AISettingsPageAction::SetCLIAgentForCommand {
                                    pattern: pattern_clone.clone(),
                                    agent: Some(agent),
                                },
                            ));
                        if let Some(icon) = icon {
                            fields = fields.with_icon(icon);
                        }
                        items.push(fields.into_item());
                    }

                    let other_label = crate::t!("settings-ai-coding-agent-other");
                    items.push(
                        MenuItemFields::new(other_label.clone())
                            .with_on_select_action(DropdownAction::select_action_and_close(
                                AISettingsPageAction::SetCLIAgentForCommand {
                                    pattern: pattern_clone.clone(),
                                    agent: None,
                                },
                            ))
                            .into_item(),
                    );

                    dropdown.set_rich_items(items, ctx);

                    let other_label_for_override = other_label.clone();
                    dropdown.set_menu_header_text_override(move |label| {
                        if label == other_label_for_override {
                            crate::t!("settings-ai-coding-agent-select-header")
                        } else {
                            label.to_string()
                        }
                    });

                    let selected_name: String = if matches!(current_agent, CLIAgent::Unknown) {
                        other_label
                    } else {
                        current_agent.display_name().to_string()
                    };
                    dropdown.set_selected_by_name(selected_name, ctx);

                    dropdown
                })
            })
            .collect()
    }

    fn save_agent_provider_edits(
        provider_id: &str,
        name: &str,
        base_url: &str,
        api_key: &str,
        headers: &[(String, String)],
        models: &[(usize, String, String, u32, u32)],
        ctx: &mut ViewContext<Self>,
    ) {
        AISettings::handle(ctx).update(ctx, |settings, ctx| {
            let mut providers = settings.agent_providers.value().clone();
            if let Some(p) = providers.iter_mut().find(|p| p.id == provider_id) {
                p.name = name.to_owned();
                p.base_url = base_url.to_owned();
                p.extra_headers = headers.to_vec();
                // 按 model_index 更新，跳过越界索引（rebuild 中间表单与 settings 可能短暂不一致）。
                for (idx, m_name, m_id, ctx_window, max_out) in models {
                    if let Some(m) = p.models.get_mut(*idx) {
                        m.name = m_name.clone();
                        m.id = m_id.clone();
                        m.context_window = *ctx_window;
                        m.max_output_tokens = *max_out;
                    }
                }
            }
            let _ = settings.agent_providers.set_value(providers, ctx);
        });
        crate::ai::agent_providers::AgentProviderSecrets::handle(ctx).update(
            ctx,
            |secrets, ctx| {
                secrets.set(provider_id, api_key.to_owned(), ctx);
            },
        );
    }
}

impl View for AISettingsPageView {
    fn ui_name() -> &'static str {
        "AISettingsPage"
    }

    fn render(&self, app: &warpui::AppContext) -> Box<dyn warpui::Element> {
        self.page.render(self, app)
    }
}

#[allow(clippy::large_enum_variant)]
pub enum AISettingsPageEvent {
    FocusModal,
    OpenAIFactCollection,
    OpenMCPServerCollection,
    #[cfg(feature = "local_fs")]
    OpenCustomRouterEditor(Option<crate::ai::custom_model_routers::CustomModelRouter>),
    #[cfg(feature = "local_fs")]
    OpenCustomRouterFile(PathBuf),
    OpenExecutionProfileEditor(ExecutionProfileId),
    ShowModal,
    HideModal,
}

impl Entity for AISettingsPageView {
    type Event = AISettingsPageEvent;
}

/// Per-agent 可见性维度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PerAgentDimension {
    Toolbar,
    TabMenu,
    Titlebar,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AISettingsPageAction {
    OpenUrl(String),
    SetVoiceInputToggleKey(VoiceInputToggleKey),
    SetVoiceInputLanguage(String),
    ToggleActiveAI,
    ToggleIntelligentAutosuggestions,
    TogglePromptSuggestions,
    ToggleCodeSuggestions,
    ToggleNaturalLanguageAutosuggestions,
    ToggleGitOperationsAutogen,
    ToggleAIInputAutoDetection,
    ToggleNLDInTerminal,
    ToggleCLIAgentToolbar,
    /// 切换单个 CLI agent 的指定维度可见性。
    ToggleCLIAgentPerAgent(CLIAgent, PerAgentDimension),
    ToggleUseAgentToolbar,
    ToggleVoiceInput,
    ToggleCanUseWarpCreditsForFallback,
    HyperlinkClick(HyperlinkUrl),
    ToggleShowInputHintText,
    ToggleShowAgentTips,
    /// 切换「显示 Agent 快捷键提示」设置（零状态三件套 + message bar 底部 4 项 hint）。
    ToggleShowAgentZeroStateHints,
    SetThinkingDisplayMode(ThinkingDisplayMode),
    SetOrchestrationMessageDisplayMode(OrchestrationMessageDisplayMode),
    SetPromptSubmissionMode(PromptSubmissionMode),
    SetLongRunningCommandSubmissionMode(LongRunningCommandSubmissionMode),
    RemoveCLIAgentToolbarEnabledCommand(String),
    RemoveFromCommandExecutionAllowlist(AgentModeCommandExecutionPredicate),
    RemoveFromCommandExecutionDenylist(AgentModeCommandExecutionPredicate),
    OpenAIFactCollection,
    OpenMCPServerCollection,
    OpenExecutionProfileEditor(ExecutionProfileId),
    SetBaseModel(LLMId),
    SetCodingModel(LLMId),
    /// Called while the user is actively dragging the context window slider.
    ContextWindowSliderDragged(u32),
    /// Called when the user commits a new context window value (slider drop or
    /// input box commit).
    SetContextWindowSize(u32),
    SetAutonomyReadonlyCommandsSetting,
    SetAutonomySupervisedSetting,
    SetCodingPermission(AgentModeCodingPermissionsType),
    RemoveDirectoryFromCodeReadAllowlist(PathBuf),
    ToggleRules,
    ToggleRuleSuggestions,
    ToggleWarpDriveContext,
    SetApplyCodeDiffs(ActionPermission),
    SetReadFiles(ActionPermission),
    SetExecuteCommands(ActionPermission),
    SetWriteToPty(WriteToPtyPermission),
    SetMCPPermissions(ActionPermission),
    RemoveFromProfileDirectoryAllowlist(PathBuf),
    RemoveFromProfileCommandDenylist(AgentModeCommandExecutionPredicate),
    RemoveFromProfileCommandAllowlist(AgentModeCommandExecutionPredicate),
    ToggleShowBaseModelPickerInPrompt,
    AddToMCPAllowlist(uuid::Uuid),
    RemoveFromMCPAllowlist(uuid::Uuid),
    AddToMCPDenylist(uuid::Uuid),
    RemoveFromMCPDenylist(uuid::Uuid),
    CreateProfile,
    ToggleAwsBedrockAutoLogin,
    ToggleAwsBedrockCredentialsEnabled,
    RefreshAwsBedrockCredentials,
    RefreshGeminiEnterpriseCredentials,
    ToggleGeminiEnterpriseCredentialsEnabled,
    ToggleFileBasedMcp,
    ToggleIncludeAgentCommandsInHistory,
    ToggleAutoApproveBypassesCommandDenylist,

    // Custom model routers
    #[cfg(feature = "local_fs")]
    OpenAddCustomRouter,

    // Custom inference
    OpenAddCustomEndpointModal,
    OpenEditCustomEndpointModal(usize),

    #[cfg(feature = "local_fs")]
    SetConversationLayout(crate::util::file::external_editor::settings::OpenConversationPreference),
    ToggleCloudHandoff,
    ToggleAmpersandHandoff,
    ToggleAutoHandoffOnSleep,
    ToggleShowConversationHistory,
    ToggleAutoToggleRichInput,
    ToggleAutoOpenRichInputOnCLIAgentStart,
    ToggleAutoDismissRichInputAfterSubmit,
    ToggleSubmitRichInputOnCtrlEnter,
    SetCLIAgentForCommand {
        pattern: String,
        agent: Option<CLIAgent>,
    },
    // 自定义 Agent Provider 管理动作
    AddAgentProvider,
    RemoveAgentProvider {
        provider_id: String,
    },
    UpdateAgentProviderName {
        provider_id: String,
        name: String,
    },
    UpdateAgentProviderBaseUrl {
        provider_id: String,
        base_url: String,
    },
    /// 显式设置 provider 的 API 协议类型(OpenAI / OpenAI-Response / Gemini / Anthropic / Ollama)。
    /// chat_stream 据此显式绑定 genai AdapterKind,绕过模型名识别。
    SetAgentProviderApiType {
        provider_id: String,
        api_type: crate::settings::AgentProviderApiType,
    },
    SetAgentProviderResponsesStateMode {
        provider_id: String,
        state_mode: crate::settings::ResponsesStateModeSetting,
    },
    SetAgentProviderResponsesTransport {
        provider_id: String,
        transport: crate::settings::ResponsesTransportSetting,
    },
    ToggleAgentProviderResponsesBackground {
        provider_id: String,
    },
    SetAgentProviderResponsesCompactThreshold {
        provider_id: String,
        compact_threshold: u32,
    },
    ToggleAgentProviderResponsesProgrammaticToolCalling {
        provider_id: String,
    },
    ToggleAgentProviderResponsesReasoningProMode {
        provider_id: String,
    },
    ToggleAgentProviderResponsesReasoningAllTurns {
        provider_id: String,
    },
    ToggleAgentProviderResponsesMultiAgentBeta {
        provider_id: String,
    },
    UpdateAgentProviderApiKey {
        provider_id: String,
        api_key: String,
    },
    /// 一次性保存某个 provider 卡片上的全部可编辑字段(name / base_url / api_key /
    /// extra_headers / models)。取代原来"失焦/Enter 逐字段推入"的 UX —— 用户在
    /// settings_view 点"保存"按钮后一起下发。
    SaveAgentProviderEdits {
        provider_id: String,
        name: String,
        base_url: String,
        api_key: String,
        headers: Vec<(String, String)>,
        /// 只携带可编辑部分:`(model_index, name, id, context_window, max_output_tokens)`。
        /// reasoning / tool_call / image / pdf / audio 由独立的 chip 动作维护,不走这里。
        models: Vec<(usize, String, String, u32, u32)>,
    },
    SaveAgentProviderEditsThen {
        provider_id: String,
        name: String,
        base_url: String,
        api_key: String,
        headers: Vec<(String, String)>,
        /// 只携带可编辑部分:`(model_index, name, id, context_window, max_output_tokens)`。
        models: Vec<(usize, String, String, u32, u32)>,
        action: Box<AISettingsPageAction>,
    },
    UpdateAgentProviderModels {
        provider_id: String,
        models: Vec<crate::settings::AgentProviderModel>,
    },
    AddAgentProviderModel {
        provider_id: String,
    },
    RemoveAgentProviderModel {
        provider_id: String,
        model_index: usize,
    },
    UpdateAgentProviderModelName {
        provider_id: String,
        model_index: usize,
        name: String,
    },
    UpdateAgentProviderModelId {
        provider_id: String,
        model_index: usize,
        id: String,
    },
    /// 更新单条模型的 context_window(tokens),0 = 未指定。
    UpdateAgentProviderModelContextWindow {
        provider_id: String,
        model_index: usize,
        context_window: u32,
    },
    /// 更新单条模型的 max_output_tokens,0 = 未指定。
    UpdateAgentProviderModelMaxOutput {
        provider_id: String,
        model_index: usize,
        max_output_tokens: u32,
    },
    AddAgentProviderHeader {
        provider_id: String,
    },
    RemoveAgentProviderHeader {
        provider_id: String,
        header_index: usize,
    },
    UpdateAgentProviderHeader {
        provider_id: String,
        header_index: usize,
        key: String,
        value: String,
    },
    FetchAgentProviderModels {
        provider_id: String,
    },
    /// 触发一次 models.dev 目录加载(磁盘缓存 + 必要时网络刷新)。Providers 子页打开即触发。
    EnsureModelsDevLoaded,
    /// 把现有 provider 的模型列表与 models.dev 同步(按 base_url 匹配),
    /// 用 catalog 提供的 context_window / reasoning / tool_call 等元数据填充本地条目。
    SyncProviderModelsFromModelsDev {
        provider_id: String,
    },

    // ----- 单条模型条目 detail panel -----
    /// 切换单条模型的 detail panel 展开/折叠状态。
    ToggleAgentProviderModelExpanded {
        provider_id: String,
        model_index: usize,
    },
    /// 三态循环切换单条模型的某个多模态 capability(image/pdf/audio)。
    /// `None → Some(true) → Some(false) → None`。
    CycleAgentProviderModelCapability {
        provider_id: String,
        model_index: usize,
        kind: ModelCapabilityKind,
    },
    /// 切换单条模型的 reasoning 标志(普通 bool 字段,不是三态)。
    ToggleAgentProviderModelReasoning {
        provider_id: String,
        model_index: usize,
    },
    /// 切换单条模型的 tool_call 标志。
    ToggleAgentProviderModelToolCall {
        provider_id: String,
        model_index: usize,
    },
}

/// model detail panel 三态 capability chip 的种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelCapabilityKind {
    Image,
    Pdf,
    Audio,
}

impl TypedActionView for AISettingsPageView {
    type Action = AISettingsPageAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            AISettingsPageAction::OpenUrl(url) => {
                ctx.open_url(url.as_str());
            }
            AISettingsPageAction::SetVoiceInputToggleKey(key) => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.voice_input_toggle_key.set_value(*key, ctx));
                    report_if_error!(
                        settings
                            .explicitly_interacted_with_voice
                            .set_value(true, ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::SetVoiceInputLanguage(language) => {
                let language = language.clone();
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.voice_input_language.set_value(language, ctx));
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleActiveAI => {
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .is_active_ai_enabled_internal
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::ToggleActiveAI {
                                is_active_ai_enabled: new_value,
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!("Failed to set value for Active AI setting: {e:?}");
                    }
                }
                ctx.notify();
            }
            AISettingsPageAction::ToggleIntelligentAutosuggestions => {
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .intelligent_autosuggestions_enabled_internal
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::ToggleIntelligentAutosuggestionsSetting {
                                is_intelligent_autosuggestions_enabled: new_value,
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!("Failed to set value for Next Command setting: {e:?}");
                    }
                }
                ctx.notify();
            }
            AISettingsPageAction::TogglePromptSuggestions => {
                if !UserWorkspaces::as_ref(ctx).is_prompt_suggestions_toggleable() {
                    return;
                }
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .prompt_suggestions_enabled_internal
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::TogglePromptSuggestionsSetting {
                                is_prompt_suggestions_enabled: new_value,
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!("Failed to set value for Prompt Suggestions setting: {e:?}");
                    }
                }
                ctx.notify();
            }
            AISettingsPageAction::ToggleCodeSuggestions => {
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .code_suggestions_enabled_internal
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::ToggleCodeSuggestionsSetting {
                                source: ToggleCodeSuggestionsSettingSource::Settings,
                                is_code_suggestions_enabled: new_value,
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!("Failed to set value for Code Suggestions setting: {e:?}");
                    }
                }
                ctx.notify();
            }
            AISettingsPageAction::ToggleNaturalLanguageAutosuggestions => {
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .natural_language_autosuggestions_enabled_internal
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::ToggleNaturalLanguageAutosuggestionsSetting {
                                is_natural_language_autosuggestions_enabled: new_value,
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to set value for Natural Language Autosuggestions setting: {e:?}"
                        );
                    }
                }
                ctx.notify();
            }
            AISettingsPageAction::ToggleGitOperationsAutogen => {
                if !UserWorkspaces::as_ref(ctx).is_git_operations_ai_enabled() {
                    return;
                }
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .git_operations_autogen_enabled_internal
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::ToggleGitOperationsAutogenSetting {
                                is_git_operations_autogen_enabled: new_value,
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!("Failed to set value for Git Operations Autogen setting: {e:?}");
                    }
                }
                ctx.notify();
            }
            AISettingsPageAction::ToggleAIInputAutoDetection => {
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .ai_autodetection_enabled_internal
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::AgentModeToggleAutoDetectionSetting {
                                is_autodetection_enabled: new_value,
                                origin: AgentModeAutoDetectionSettingOrigin::SettingsPage
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!("Failed to set value for Input Auto-detection: {e:?}");
                    }
                }
                ctx.notify();
            }
            AISettingsPageAction::ToggleNLDInTerminal => {
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .nld_in_terminal_enabled_internal
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(_new_value) => {}
                    Err(e) => {
                        log::warn!("Failed to set value for NLD in Terminal: {e:?}");
                    }
                }
                ctx.notify();
            }
            AISettingsPageAction::ToggleCLIAgentToolbar => {
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .should_render_cli_agent_footer
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::ToggleCLIAgentToolbarSetting {
                                is_enabled: new_value,
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!("Failed to set value for CLI Agent Footer setting: {e:?}");
                    }
                }
                ctx.notify();
            }
            AISettingsPageAction::ToggleCLIAgentPerAgent(agent, dim) => {
                let settings = AISettings::as_ref(ctx);
                let current = match dim {
                    PerAgentDimension::Toolbar => settings.is_cli_agent_toolbar_enabled(*agent),
                    PerAgentDimension::TabMenu => settings.is_cli_agent_tab_menu_enabled(*agent),
                    PerAgentDimension::Titlebar => settings.is_cli_agent_titlebar_enabled(*agent),
                };
                AISettings::handle(ctx).update(ctx, |settings, ctx| match dim {
                    PerAgentDimension::Toolbar => {
                        settings.set_cli_agent_toolbar(*agent, !current, ctx);
                    }
                    PerAgentDimension::TabMenu => {
                        settings.set_cli_agent_tab_menu(*agent, !current, ctx);
                    }
                    PerAgentDimension::Titlebar => {
                        settings.set_cli_agent_titlebar(*agent, !current, ctx);
                    }
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleAutoToggleRichInput => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.auto_toggle_rich_input.toggle_and_save_value(ctx));
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleAutoOpenRichInputOnCLIAgentStart => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .auto_open_rich_input_on_cli_agent_start
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleAutoDismissRichInputAfterSubmit => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .auto_dismiss_rich_input_after_submit
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleSubmitRichInputOnCtrlEnter => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.submit_on_ctrl_enter.toggle_and_save_value(ctx));
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleUseAgentToolbar => {
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .should_render_use_agent_footer_for_user_commands
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::ToggleUseAgentToolbarSetting {
                                is_enabled: new_value,
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!("Failed to set value for Use Agent Footer setting: {e:?}");
                    }
                }
                ctx.notify();
            }

            AISettingsPageAction::ToggleVoiceInput => {
                match AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings
                        .voice_input_enabled_internal
                        .toggle_and_save_value(ctx)
                }) {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::ToggleVoiceInputSetting {
                                is_voice_input_enabled: new_value,
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!("Failed to set value for Voice Input: {e:?}");
                    }
                }
                ctx.notify();
            }
            AISettingsPageAction::ToggleCanUseWarpCreditsForFallback => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .can_use_warp_credits_for_fallback
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::HyperlinkClick(hyperlink) => {
                ctx.notify();
                ctx.open_url(&hyperlink.url);
            }
            AISettingsPageAction::ToggleShowInputHintText => {
                InputSettings::handle(ctx).update(ctx, |input_settings, ctx| {
                    report_if_error!(input_settings.show_hint_text.toggle_and_save_value(ctx));
                    send_telemetry_from_ctx!(
                        // We purposely keep the FeaturesPageAction event, even though we have moved the setting to AI settings.
                        TelemetryEvent::FeaturesPageAction {
                            action: "ToggleShowInputHintText".to_string(),
                            value: format!("{}", *input_settings.show_hint_text),
                        },
                        ctx
                    );
                });
            }
            AISettingsPageAction::ToggleShowAgentTips => {
                InputSettings::handle(ctx).update(ctx, |input_settings, ctx| match input_settings
                    .show_agent_tips
                    .toggle_and_save_value(ctx)
                {
                    Ok(new_value) => {
                        send_telemetry_from_ctx!(
                            TelemetryEvent::ToggleShowAgentTips {
                                is_enabled: new_value,
                            },
                            ctx
                        );
                    }
                    Err(e) => {
                        log::warn!("Failed to set value for Show Agent Tips setting: {e:?}");
                    }
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleShowAgentZeroStateHints => {
                InputSettings::handle(ctx).update(ctx, |input_settings, ctx| {
                    if let Err(e) = input_settings
                        .show_agent_zero_state_hints
                        .toggle_and_save_value(ctx)
                    {
                        log::warn!(
                            "Failed to set value for Show Agent Zero-State Hints setting: {e:?}"
                        );
                    }
                });
                ctx.notify();
            }
            AISettingsPageAction::SetThinkingDisplayMode(mode) => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.thinking_display_mode.set_value(*mode, ctx));
                });
                ctx.notify();
            }
            AISettingsPageAction::SetOrchestrationMessageDisplayMode(mode) => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .orchestration_message_display_mode
                            .set_value(*mode, ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::SetPromptSubmissionMode(mode) => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .default_prompt_submission_mode
                            .set_value(*mode, ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::SetLongRunningCommandSubmissionMode(mode) => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .long_running_command_submission_mode
                            .set_value(*mode, ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::RemoveCLIAgentToolbarEnabledCommand(command) => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings.remove_cli_agent_footer_enabled_command(command, ctx);
                });
            }
            AISettingsPageAction::SetCLIAgentForCommand { pattern, agent } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    settings.set_cli_agent_for_command(pattern, *agent, ctx);
                });
            }
            AISettingsPageAction::RemoveFromCommandExecutionAllowlist(cmd) => {
                BlocklistAIPermissions::handle(ctx).update(ctx, |model, ctx| {
                    report_if_error!(model.remove_command_from_autoexecution_allowlist(cmd, ctx));
                })
            }
            AISettingsPageAction::RemoveFromCommandExecutionDenylist(cmd) => {
                BlocklistAIPermissions::handle(ctx).update(ctx, |model, ctx| {
                    report_if_error!(model.remove_command_from_denylist(cmd, ctx));
                })
            }
            AISettingsPageAction::OpenAIFactCollection => {
                ctx.emit(AISettingsPageEvent::OpenAIFactCollection)
            }
            AISettingsPageAction::OpenMCPServerCollection => {
                ctx.emit(AISettingsPageEvent::OpenMCPServerCollection)
            }
            AISettingsPageAction::OpenExecutionProfileEditor(profile_id) => ctx.emit(
                AISettingsPageEvent::OpenExecutionProfileEditor(profile_id.clone()),
            ),
            AISettingsPageAction::SetBaseModel(id) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    let profile_id = profiles_model.active_profile(None, ctx).id().clone();
                    profiles_model.set_base_model(&profile_id, Some(id.clone()), ctx);
                    profiles_model.set_context_window_limit(&profile_id, None, ctx);
                });
                self.sync_context_window_editor(ctx, true);
                ctx.notify();
            }
            AISettingsPageAction::SetCodingModel(id) => {
                LLMPreferences::handle(ctx).update(ctx, |prefs, ctx| {
                    prefs.update_preferred_coding_llm(id, None, ctx);
                });
            }
            AISettingsPageAction::ContextWindowSliderDragged(value) => {
                if !AISettings::as_ref(ctx).is_any_ai_enabled(ctx) {
                    self.sync_context_window_editor(ctx, true);
                    return;
                }
                if Self::configurable_context_window(ctx).is_some() {
                    self.dragged_context_window_value = Some(*value);
                    let formatted = value.to_string();
                    self.context_window_editor.update(ctx, |editor, ctx| {
                        editor.system_reset_buffer_text(&formatted, ctx);
                    });
                    ctx.notify();
                }
            }
            AISettingsPageAction::SetContextWindowSize(value) => {
                self.dragged_context_window_value = None;
                if !AISettings::as_ref(ctx).is_any_ai_enabled(ctx) {
                    self.sync_context_window_editor(ctx, true);
                    return;
                }
                let Some(cw) = Self::configurable_context_window(ctx) else {
                    return;
                };
                let clamped = (*value).clamp(cw.min, cw.max);
                AIExecutionProfilesModel::handle(ctx).update(ctx, |profiles_model, ctx| {
                    let profile_id = profiles_model.active_profile(None, ctx).id().clone();
                    profiles_model.set_context_window_limit(&profile_id, Some(clamped), ctx);
                });
                self.sync_context_window_editor(ctx, true);
                ctx.notify();
            }
            AISettingsPageAction::SetAutonomyReadonlyCommandsSetting
            | AISettingsPageAction::SetAutonomySupervisedSetting => {
                let readonly_cmd_execution_enabled = matches!(
                    action,
                    AISettingsPageAction::SetAutonomyReadonlyCommandsSetting
                );
                BlocklistAIPermissions::handle(ctx).update(ctx, |model, ctx| {
                    match model.set_should_autoexecute_readonly_commands(
                        readonly_cmd_execution_enabled,
                        ctx,
                    ) {
                        Ok(_) => {
                            send_telemetry_from_ctx!(
                                TelemetryEvent::ToggledAgentModeAutoexecuteReadonlyCommandsSetting {
                                    src: AutonomySettingToggleSource::SettingsPage,
                                    enabled: readonly_cmd_execution_enabled,
                                },
                                ctx);
                        }
                        Err(e) => report_error!(e),
                    }
                });
            }
            AISettingsPageAction::SetCodingPermission(p) => {
                BlocklistAIPermissions::handle(ctx).update(ctx, |model, ctx| {
                    match model.set_coding_permissions(*p, ctx) {
                        Ok(_) => {
                            send_telemetry_from_ctx!(
                                TelemetryEvent::ChangedAgentModeCodingPermissions {
                                    src: AutonomySettingToggleSource::SettingsPage,
                                    new: *p,
                                },
                                ctx
                            );
                        }
                        Err(e) => report_error!(e),
                    }
                });
            }
            AISettingsPageAction::SetApplyCodeDiffs(permission) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    model.set_apply_code_diffs(profile.id(), permission, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::SetReadFiles(permission) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    model.set_read_files(profile.id(), permission, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::SetExecuteCommands(permission) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    model.set_execute_commands(profile.id(), permission, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::SetWriteToPty(permission) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    model.set_write_to_pty(profile.id(), permission, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::SetMCPPermissions(permission) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    model.set_mcp_permissions(profile.id(), permission, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::RemoveDirectoryFromCodeReadAllowlist(dir) => {
                BlocklistAIPermissions::handle(ctx).update(ctx, |model, ctx| {
                    report_if_error!(
                        model.remove_filepath_from_code_read_allowlist(dir.to_owned(), ctx)
                    );
                });
            }
            AISettingsPageAction::ToggleRules => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let _ = settings.memory_enabled.toggle_and_save_value(ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleRuleSuggestions => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let _ = settings
                        .rule_suggestions_enabled_internal
                        .toggle_and_save_value(ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleWarpDriveContext => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let _ = settings
                        .warp_drive_context_enabled
                        .toggle_and_save_value(ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::RemoveFromProfileDirectoryAllowlist(path_buf) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    let profile_id = profile.id();
                    model.remove_from_directory_allowlist(
                        profile_id,
                        &PathBuf::from(path_buf),
                        ctx,
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::RemoveFromProfileCommandDenylist(cmd) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    let profile_id = profile.id();

                    model.remove_from_command_denylist(profile_id, cmd, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::RemoveFromProfileCommandAllowlist(command) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    let profile_id = profile.id();

                    model.remove_from_command_allowlist(profile_id, command, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleShowBaseModelPickerInPrompt => {
                SessionSettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(e) = settings
                        .show_model_selectors_in_prompt
                        .toggle_and_save_value(ctx)
                    {
                        log::warn!(
                            "Failed to set value for Show Base Model Picker in Prompt: {e:?}"
                        );
                    }
                });
                ctx.notify();
            }
            AISettingsPageAction::AddToMCPAllowlist(id) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    let profile_id = profile.id();
                    model.add_to_mcp_allowlist(profile_id, id, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::RemoveFromMCPAllowlist(id) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    let profile_id = profile.id();
                    model.remove_from_mcp_allowlist(profile_id, id, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::AddToMCPDenylist(id) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    let profile_id = profile.id();
                    model.add_to_mcp_denylist(profile_id, id, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::RemoveFromMCPDenylist(id) => {
                AIExecutionProfilesModel::handle(ctx).update(ctx, |model, ctx| {
                    let profile = model.default_profile(ctx);
                    let profile_id = profile.id();
                    model.remove_from_mcp_denylist(profile_id, id, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::CreateProfile => {
                let new_profile_id = AIExecutionProfilesModel::handle(ctx)
                    .update(ctx, |model, ctx| model.create_profile(ctx));

                if let Some(profile_id) = new_profile_id {
                    ctx.emit(AISettingsPageEvent::OpenExecutionProfileEditor(profile_id));
                }
                ctx.notify();
            }
            AISettingsPageAction::ToggleAwsBedrockAutoLogin => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.aws_bedrock_auto_login.toggle_and_save_value(ctx));
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleAwsBedrockCredentialsEnabled => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .aws_bedrock_credentials_enabled
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::RefreshAwsBedrockCredentials => {
                #[cfg(not(target_family = "wasm"))]
                ApiKeyManager::handle(ctx).update(ctx, |manager, ctx| {
                    drop(refresh_aws_credentials(manager, ctx));
                });
                ctx.notify();
            }
            AISettingsPageAction::RefreshGeminiEnterpriseCredentials => {
                #[cfg(not(target_family = "wasm"))]
                ApiKeyManager::handle(ctx).update(ctx, |manager, ctx| {
                    force_refresh_geap_credentials(manager, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleGeminiEnterpriseCredentialsEnabled => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .gemini_enterprise_credentials_enabled
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleFileBasedMcp => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.file_based_mcp_enabled.toggle_and_save_value(ctx));
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleIncludeAgentCommandsInHistory => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .include_agent_commands_in_history
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleAutoApproveBypassesCommandDenylist => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .auto_approve_bypasses_command_denylist
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            #[cfg(feature = "local_fs")]
            AISettingsPageAction::SetConversationLayout(layout) => {
                crate::util::file::external_editor::EditorSettings::handle(ctx).update(
                    ctx,
                    |settings, ctx| {
                        report_if_error!(
                            settings
                                .open_conversation_layout_preference
                                .set_value(*layout, ctx)
                        );
                    },
                );
                send_telemetry_from_ctx!(
                    TelemetryEvent::FeaturesPageAction {
                        action: "SetConversationLayout".to_string(),
                        value: format!("{layout:?}")
                    },
                    ctx
                );
                ctx.notify();
            }
            AISettingsPageAction::ToggleShowConversationHistory => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .show_conversation_history
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            #[cfg(feature = "local_fs")]
            AISettingsPageAction::OpenAddCustomRouter => {
                ctx.emit(AISettingsPageEvent::OpenCustomRouterEditor(None));
            }
            AISettingsPageAction::OpenAddCustomEndpointModal => {
                self.show_add_custom_endpoint_modal(ctx);
            }
            AISettingsPageAction::OpenEditCustomEndpointModal(index) => {
                self.show_edit_custom_endpoint_modal(*index, ctx);
            }
            AISettingsPageAction::ToggleCloudHandoff => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .should_force_disable_cloud_handoff
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleAmpersandHandoff => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .should_force_disable_ampersand_handoff
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::ToggleAutoHandoffOnSleep => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(
                        settings
                            .auto_handoff_on_sleep_enabled
                            .toggle_and_save_value(ctx)
                    );
                });
                ctx.notify();
            }
            AISettingsPageAction::AddAgentProvider => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    providers.push(crate::settings::AgentProvider::new_empty());
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::RemoveAgentProvider { provider_id } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    providers.retain(|p| p.id != *provider_id);
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                crate::ai::agent_providers::AgentProviderSecrets::handle(ctx).update(
                    ctx,
                    |secrets, ctx| {
                        secrets.remove(provider_id, ctx);
                    },
                );
                super::agent_providers_widget::clear_expanded_models_for_provider(provider_id);
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::UpdateAgentProviderName { provider_id, name } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        p.name = name.clone();
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::UpdateAgentProviderBaseUrl {
                provider_id,
                base_url,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        p.base_url = base_url.clone();
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::SetAgentProviderApiType {
                provider_id,
                api_type,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        p.api_type = *api_type;
                        // 若 base_url 为空,顺手填该类型的默认 endpoint(便于新手)。
                        // 用户已自填 base_url 时不动。
                        if p.base_url.trim().is_empty() {
                            p.base_url = api_type.default_base_url().to_owned();
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::SetAgentProviderResponsesStateMode {
                provider_id,
                state_mode,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(provider) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        provider.responses.state_mode = *state_mode;
                        if *state_mode == crate::settings::ResponsesStateModeSetting::LocalReplay {
                            provider.responses.background = false;
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::SetAgentProviderResponsesTransport {
                provider_id,
                transport,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(provider) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        provider.responses.transport = *transport;
                        if *transport == crate::settings::ResponsesTransportSetting::WebSocket {
                            provider.responses.background = false;
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::ToggleAgentProviderResponsesBackground { provider_id } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(provider) = providers.iter_mut().find(|p| p.id == *provider_id)
                        && provider.responses.state_mode
                            != crate::settings::ResponsesStateModeSetting::LocalReplay
                        && provider.responses.transport
                            == crate::settings::ResponsesTransportSetting::Http
                    {
                        provider.responses.background = !provider.responses.background;
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::SetAgentProviderResponsesCompactThreshold {
                provider_id,
                compact_threshold,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(provider) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        provider.responses.compact_threshold = *compact_threshold;
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::ToggleAgentProviderResponsesProgrammaticToolCalling {
                provider_id,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(provider) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        provider.responses.programmatic_tool_calling =
                            !provider.responses.programmatic_tool_calling;
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::ToggleAgentProviderResponsesReasoningProMode { provider_id } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(provider) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        provider.responses.reasoning_pro_mode =
                            !provider.responses.reasoning_pro_mode;
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::ToggleAgentProviderResponsesReasoningAllTurns { provider_id } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(provider) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        provider.responses.reasoning_all_turns =
                            !provider.responses.reasoning_all_turns;
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::ToggleAgentProviderResponsesMultiAgentBeta { provider_id } => {
                if FeatureFlag::ResponsesMultiAgentBeta.is_enabled() {
                    AISettings::handle(ctx).update(ctx, |settings, ctx| {
                        let mut providers = settings.agent_providers.value().clone();
                        if let Some(provider) = providers.iter_mut().find(|p| p.id == *provider_id)
                        {
                            provider.responses.multi_agent_beta =
                                !provider.responses.multi_agent_beta;
                        }
                        let _ = settings.agent_providers.set_value(providers, ctx);
                    });
                }
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::UpdateAgentProviderApiKey {
                provider_id,
                api_key,
            } => {
                crate::ai::agent_providers::AgentProviderSecrets::handle(ctx).update(
                    ctx,
                    |secrets, ctx| {
                        secrets.set(provider_id, api_key.clone(), ctx);
                    },
                );
                ctx.notify();
            }
            AISettingsPageAction::SaveAgentProviderEdits {
                provider_id,
                name,
                base_url,
                api_key,
                headers,
                models,
            } => {
                Self::save_agent_provider_edits(
                    provider_id,
                    name,
                    base_url,
                    api_key,
                    headers,
                    models,
                    ctx,
                );
                ctx.notify();
            }
            AISettingsPageAction::SaveAgentProviderEditsThen {
                provider_id,
                name,
                base_url,
                api_key,
                headers,
                models,
                action,
            } => {
                Self::save_agent_provider_edits(
                    provider_id,
                    name,
                    base_url,
                    api_key,
                    headers,
                    models,
                    ctx,
                );
                self.handle_action(action.as_ref(), ctx);
            }
            AISettingsPageAction::UpdateAgentProviderModels {
                provider_id,
                models,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        p.models = models.clone();
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::AddAgentProviderModel { provider_id } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        p.models
                            .push(crate::settings::AgentProviderModel::from_id(String::new()));
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                // 行级 add 需要新建 EditorView,所以走 rebuild;rebuild_current_page 已保留滚动。
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::RemoveAgentProviderModel {
                provider_id,
                model_index,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        if *model_index < p.models.len() {
                            p.models.remove(*model_index);
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                // 删一条会让后续 index 漂移,清掉这个 provider 的全部展开记录避免误展开。
                super::agent_providers_widget::clear_expanded_models_for_provider(provider_id);
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::UpdateAgentProviderModelName {
                provider_id,
                model_index,
                name,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        if let Some(m) = p.models.get_mut(*model_index) {
                            m.name = name.clone();
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::UpdateAgentProviderModelId {
                provider_id,
                model_index,
                id,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        if let Some(m) = p.models.get_mut(*model_index) {
                            m.id = id.clone();
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::UpdateAgentProviderModelContextWindow {
                provider_id,
                model_index,
                context_window,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        if let Some(m) = p.models.get_mut(*model_index) {
                            m.context_window = *context_window;
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::UpdateAgentProviderModelMaxOutput {
                provider_id,
                model_index,
                max_output_tokens,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        if let Some(m) = p.models.get_mut(*model_index) {
                            m.max_output_tokens = *max_output_tokens;
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::FetchAgentProviderModels { provider_id } => {
                let provider_id = provider_id.clone();
                let providers = AISettings::as_ref(ctx).agent_providers.value().clone();
                let Some(provider) = providers.into_iter().find(|p| p.id == provider_id) else {
                    return;
                };
                let api_key = crate::ai::agent_providers::AgentProviderSecrets::as_ref(ctx)
                    .get(&provider_id)
                    .map(str::to_owned);
                let client = http_client::Client::new();
                let provider_id_for_handler = provider_id.clone();
                ctx.spawn(
                    async move {
                        crate::ai::agent_providers::fetch_openai_compatible_models(
                            client,
                            &provider.base_url,
                            api_key.as_deref(),
                        )
                        .await
                    },
                    move |view, result, ctx| match result {
                        Ok(fetched) => {
                            AISettings::handle(ctx).update(ctx, |settings, ctx| {
                                let mut providers = settings.agent_providers.value().clone();
                                if let Some(p) = providers
                                    .iter_mut()
                                    .find(|p| p.id == provider_id_for_handler)
                                {
                                    // 合并保留: 已存在的 id 保留用户改过的 name,新 id 追加,
                                    // 本地多余的 id 不删(用户手动 ×)。
                                    let existing: std::collections::HashSet<String> =
                                        p.models.iter().map(|m| m.id.clone()).collect();
                                    for m in fetched {
                                        if !existing.contains(&m.id) {
                                            p.models.push(
                                                crate::settings::AgentProviderModel::from_id(m.id),
                                            );
                                        }
                                    }
                                }
                                let _ = settings.agent_providers.set_value(providers, ctx);
                            });
                            // 模型行数可能变了,需要 rebuild widget rows。
                            view.rebuild_current_page(ctx);
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to fetch models for provider {provider_id_for_handler}: {e}"
                            );
                            ctx.notify();
                        }
                    },
                );
            }
            AISettingsPageAction::AddAgentProviderHeader { provider_id } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        p.extra_headers.push((String::new(), String::new()));
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                // header 行数量变化后需要新建/销毁 EditorView handle,仅 notify 不会刷新 rows。
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::RemoveAgentProviderHeader {
                provider_id,
                header_index,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        if *header_index < p.extra_headers.len() {
                            p.extra_headers.remove(*header_index);
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                // 删除同样会导致 index 与现有 HeaderRow handle 漂移,需要重建页面。
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::UpdateAgentProviderHeader {
                provider_id,
                header_index,
                key,
                value,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        if let Some(h) = p.extra_headers.get_mut(*header_index) {
                            *h = (key.clone(), value.clone());
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                ctx.notify();
            }
            AISettingsPageAction::EnsureModelsDevLoaded => {
                use crate::ai::agent_providers::models_dev;
                let had_disk = models_dev::load_from_disk();
                if !had_disk || models_dev::is_stale() {
                    let client = http_client::Client::new();
                    ctx.spawn(
                        async move { models_dev::fetch_and_cache(client).await },
                        |view, result, ctx| match result {
                            Ok(()) => {
                                view.rebuild_current_page(ctx);
                            }
                            Err(e) => {
                                log::warn!("[models.dev] 拉取失败: {e}");
                                ctx.notify();
                            }
                        },
                    );
                } else {
                    ctx.notify();
                }
            }
            AISettingsPageAction::SyncProviderModelsFromModelsDev { provider_id } => {
                use crate::ai::agent_providers::models_dev;
                let Some(catalog) = models_dev::cached() else {
                    log::warn!("[models.dev] 目录未加载,无法同步 {provider_id}");
                    return;
                };
                let providers_snapshot = AISettings::as_ref(ctx).agent_providers.value().clone();
                let Some(local) = providers_snapshot.iter().find(|p| p.id == *provider_id) else {
                    return;
                };
                // 匹配策略:先按 base_url 完全相等 / 包含;否则按 name (大小写无关) 匹配 catalog provider id 或 name。
                let target_url = local.base_url.trim().trim_end_matches('/').to_lowercase();
                let target_name = local.name.trim().to_lowercase();
                let cat_provider = catalog.iter().find(|(_, p)| {
                    if let Some(api) = &p.api {
                        let api_norm = api.trim().trim_end_matches('/').to_lowercase();
                        if !target_url.is_empty()
                            && (api_norm == target_url
                                || api_norm.contains(&target_url)
                                || target_url.contains(&api_norm))
                        {
                            return true;
                        }
                    }
                    !target_name.is_empty()
                        && (p.name.to_lowercase() == target_name
                            || p.id.to_lowercase() == target_name)
                });
                let Some((_, cat_provider)) = cat_provider else {
                    log::warn!(
                        "[models.dev] 未在目录中找到匹配 (base_url={}, name={})",
                        local.base_url,
                        local.name
                    );
                    return;
                };
                let cat_models = cat_provider.models.clone();
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        // 既有 id 用 catalog 元数据覆盖;catalog 多出的追加;本地多出的(用户自定义)保留。
                        for local_model in p.models.iter_mut() {
                            if let Some(cat_m) = cat_models.get(&local_model.id) {
                                let merged = models_dev::into_agent_provider_model(cat_m);
                                local_model.context_window = merged.context_window;
                                local_model.max_output_tokens = merged.max_output_tokens;
                                local_model.reasoning = merged.reasoning;
                                local_model.tool_call = merged.tool_call;
                                if local_model.name.trim().is_empty() {
                                    local_model.name = merged.name;
                                }
                                // 多模态 capability:**只填 None 槽位**,Some(_) 视为用户
                                // 已显式覆盖,sync 不动。这样:
                                // - 首次 sync(用户没碰过) → 全部写入 catalog 推断结果
                                // - 用户手动 cycle 到 Some(true/false) 后再 sync → 保留覆盖
                                // - 用户三态循环回 None(Auto) → 下次 sync 又会被填上
                                if local_model.image.is_none() {
                                    local_model.image = merged.image;
                                }
                                if local_model.pdf.is_none() {
                                    local_model.pdf = merged.pdf;
                                }
                                if local_model.audio.is_none() {
                                    local_model.audio = merged.audio;
                                }
                            }
                        }
                        let existing: std::collections::HashSet<String> =
                            p.models.iter().map(|m| m.id.clone()).collect();
                        for cat_m in cat_models.values() {
                            if !existing.contains(&cat_m.id) {
                                p.models.push(models_dev::into_agent_provider_model(cat_m));
                            }
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::ToggleAgentProviderModelExpanded {
                provider_id,
                model_index,
            } => {
                super::agent_providers_widget::toggle_model_expanded(provider_id, *model_index);
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::CycleAgentProviderModelCapability {
                provider_id,
                model_index,
                kind,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        if let Some(m) = p.models.get_mut(*model_index) {
                            let slot = match kind {
                                ModelCapabilityKind::Image => &mut m.image,
                                ModelCapabilityKind::Pdf => &mut m.pdf,
                                ModelCapabilityKind::Audio => &mut m.audio,
                            };
                            // 三态循环:None → Some(true) → Some(false) → None。
                            *slot = match *slot {
                                None => Some(true),
                                Some(true) => Some(false),
                                Some(false) => None,
                            };
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::ToggleAgentProviderModelReasoning {
                provider_id,
                model_index,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        if let Some(m) = p.models.get_mut(*model_index) {
                            m.reasoning = !m.reasoning;
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
            AISettingsPageAction::ToggleAgentProviderModelToolCall {
                provider_id,
                model_index,
            } => {
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let mut providers = settings.agent_providers.value().clone();
                    if let Some(p) = providers.iter_mut().find(|p| p.id == *provider_id) {
                        if let Some(m) = p.models.get_mut(*model_index) {
                            m.tool_call = !m.tool_call;
                        }
                    }
                    let _ = settings.agent_providers.set_value(providers, ctx);
                });
                self.rebuild_current_page(ctx);
            }
        }
    }
}

impl SettingsPageMeta for AISettingsPageView {
    fn section() -> SettingsSection {
        SettingsSection::AI
    }

    fn should_render(&self, _ctx: &AppContext) -> bool {
        FeatureFlag::AgentMode.is_enabled()
    }

    fn on_page_selected(&mut self, _: bool, ctx: &mut ViewContext<Self>) {
        if UserWorkspaces::as_ref(ctx).is_byo_api_key_enabled(ctx) {
            return;
        }

        AIRequestUsageModel::handle(ctx).update(ctx, |ai_request_usage_model, ctx| {
            ai_request_usage_model.refresh_request_usage_async(ctx)
        });
    }

    fn update_filter(&mut self, query: &str, ctx: &mut ViewContext<Self>) -> MatchData {
        self.page.update_filter(query, ctx)
    }

    fn scroll_to_widget(&mut self, widget_id: &'static str) {
        self.page.scroll_to_widget(widget_id)
    }

    fn clear_highlighted_widget(&mut self) {
        self.page.clear_highlighted_widget();
    }
}

impl From<ViewHandle<AISettingsPageView>> for SettingsPageViewHandle {
    fn from(view_handle: ViewHandle<AISettingsPageView>) -> Self {
        SettingsPageViewHandle::AI(view_handle)
    }
}

fn render_ai_setting_toggle<S: Setting>(
    label: impl Into<String>,
    action: AISettingsPageAction,
    is_setting_enabled: bool,
    is_setting_toggleable: bool,
    switch_state: SwitchStateHandle,
    tooltip_states: &RefCell<HashMap<String, MouseStateHandle>>,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    build_toggle_element(
        render_body_item_label::<AISettingsPageAction>(
            label.into(),
            Some(styles::header_font_color(is_setting_toggleable, app)),
            None,
            LocalOnlyIconState::for_setting(
                S::storage_key(),
                S::sync_to_cloud(),
                &mut tooltip_states.borrow_mut(),
                app,
            ),
            ToggleState::Enabled,
            appearance,
        ),
        render_ai_feature_switch(
            switch_state,
            is_setting_enabled,
            is_setting_toggleable,
            action,
            app,
        ),
        appearance,
        None,
    )
}

fn render_ai_setting_label<S: Setting>(
    label: impl Into<String>,
    is_setting_toggleable: bool,
    tooltip_states: &RefCell<HashMap<String, MouseStateHandle>>,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    Container::new(render_body_item_label::<AISettingsPageAction>(
        label.into(),
        Some(styles::header_font_color(is_setting_toggleable, app)),
        None,
        LocalOnlyIconState::for_setting(
            S::storage_key(),
            S::sync_to_cloud(),
            &mut tooltip_states.borrow_mut(),
            app,
        ),
        ToggleState::Enabled,
        appearance,
    ))
    .with_margin_bottom(HEADER_PADDING)
    .finish()
}

fn render_ai_setting_description(
    description: impl Into<Cow<'static, str>>,
    is_setting_toggleable: bool,
    app: &AppContext,
) -> Box<dyn Element> {
    let default_font_size = Appearance::as_ref(app).ui_font_size();
    render_ai_setting_description_with_font_size(
        description,
        default_font_size,
        is_setting_toggleable,
        app,
    )
}

fn render_ai_setting_description_with_font_size(
    description: impl Into<Cow<'static, str>>,
    font_size: f32,
    is_setting_toggleable: bool,
    app: &AppContext,
) -> Box<dyn Element> {
    let ui_builder = Appearance::as_ref(app).ui_builder();
    ui_builder
        .paragraph(description)
        .with_style(UiComponentStyles {
            font_size: Some(font_size),
            font_color: Some(styles::description_font_color(is_setting_toggleable, app).into()),
            margin: Some(
                Coords::default()
                    .top(styles::DESCRIPTION_NEGATIVE_MARGIN_OFFSET)
                    .bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
                    .right(styles::TOGGLE_WIDTH_MARGIN),
            ),
            ..Default::default()
        })
        .build()
        .finish()
}

fn render_ai_status_text(
    status: impl Into<Cow<'static, str>>,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    appearance
        .ui_builder()
        .paragraph(status)
        .with_style(UiComponentStyles {
            font_size: Some(appearance.ui_font_size()),
            font_color: Some(styles::description_font_color(true, app).into()),
            margin: Some(
                Coords::default()
                    .bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
                    .right(styles::TOGGLE_WIDTH_MARGIN),
            ),
            ..Default::default()
        })
        .build()
        .finish()
}

fn render_toolbar_layout_editor(
    editor: &ViewHandle<AgentToolbarInlineEditor>,
    appearance: &Appearance,
) -> Box<dyn Element> {
    let label = Container::new(
        appearance
            .ui_builder()
            .span(crate::t!("settings-ai-toolbar-layout"))
            .with_style(UiComponentStyles {
                font_size: Some(appearance.ui_font_body()),
                ..Default::default()
            })
            .build()
            .finish(),
    )
    .with_margin_bottom(4.)
    .finish();
    let editor = Container::new(ChildView::new(editor).finish())
        .with_margin_bottom(16.)
        .finish();

    Flex::column().with_child(label).with_child(editor).finish()
}

fn render_ai_feature_switch(
    state_handle: SwitchStateHandle,
    is_setting_enabled: bool,
    is_setting_toggleable: bool,
    toggle_action: AISettingsPageAction,
    app: &AppContext,
) -> Box<dyn Element> {
    let appearance = Appearance::as_ref(app);
    let ui_builder = appearance.ui_builder();
    ui_builder
        .switch(state_handle)
        .check(is_setting_enabled)
        .with_disabled(!is_setting_toggleable)
        .with_disabled_styles(UiComponentStyles {
            background: Some(Fill::Solid(internal_colors::neutral_4(appearance.theme()))),
            foreground: Some(Fill::Solid(internal_colors::neutral_5(appearance.theme()))),
            ..Default::default()
        })
        .build()
        .on_click(move |ctx, _, _| {
            if !is_setting_toggleable {
                return;
            }
            ctx.dispatch_typed_action(toggle_action.clone());
        })
        .finish()
}

fn render_ai_list(
    header: &str,
    description: &str,
    input_list: Box<dyn Element>,
    view: &AISettingsPageView,
    ai_settings: &AISettings,
    app: &AppContext,
) -> Box<dyn Element> {
    let setting_header = render_ai_setting_label::<AgentModeCommandExecutionDenylist>(
        header.to_string(),
        ai_settings.is_any_ai_enabled(app),
        &view.local_only_icon_tooltip_states,
        app,
    );

    let description = render_ai_setting_description(
        description.to_string(),
        ai_settings.is_any_ai_enabled(app),
        app,
    );

    Flex::column()
        .with_child(setting_header)
        .with_child(Container::new(description).with_margin_bottom(-8.).finish())
        .with_child(input_list)
        .finish()
}

struct WarpAgentHeaderWidget;

impl SettingsWidget for WarpAgentHeaderWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "oz warp agent ai a.i. active next command prompt code diffs suggestion suggested suggestions \
                agent mode natural language detection input hint api keys bring your own byo google anthropic openai"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        let row = Flex::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                Text::new_inline(
                    crate::t!("settings-ai-warp-agent-header"),
                    appearance.ui_font_family(),
                    appearance.ui_font_display(),
                )
                .with_style(Properties::default().weight(Weight::Bold))
                .with_color(appearance.theme().active_ui_text_color().into())
                .finish(),
            );

        Container::new(row.finish())
            .with_padding_bottom(15.)
            .finish()
    }
}

struct UsageWidget {
    view_handle: WeakViewHandle<AISettingsPageView>,
    requests_highlight_index: HighlightedHyperlink,
}

impl UsageWidget {
    fn new(ctx: &ViewContext<AISettingsPageView>) -> Self {
        Self {
            view_handle: ctx.handle(),
            requests_highlight_index: Default::default(),
        }
    }
    fn render_request_usage_count(
        &self,
        used: usize,
        limit: usize,
        is_unlimited: bool,
        workspace_is_delinquent_due_to_payment_issue: bool,
        appearance: &Appearance,
    ) -> Box<dyn warpui::Element> {
        let mut row = Flex::row();
        if used >= limit || workspace_is_delinquent_due_to_payment_issue {
            row.add_child(
                ConstrainedBox::new(
                    Icon::AlertTriangle
                        .to_warpui_icon(appearance.theme().ui_error_color().into())
                        .finish(),
                )
                .with_height(16.)
                .with_width(16.)
                .finish(),
            )
        }

        let request_count_label = if workspace_is_delinquent_due_to_payment_issue {
            crate::t!("settings-ai-restricted-billing")
        } else if is_unlimited {
            crate::t!("settings-ai-unlimited")
        } else {
            format!("{used}/{limit}")
        };

        row.add_child(
            appearance
                .ui_builder()
                .paragraph(request_count_label)
                .with_style(UiComponentStyles {
                    font_color: {
                        if used >= limit {
                            Some(appearance.theme().ui_error_color())
                        } else {
                            Some(blended_colors::text_sub(
                                appearance.theme(),
                                appearance.theme().surface_1(),
                            ))
                        }
                    },
                    font_size: Some(appearance.ui_font_heading_3()),
                    margin: Some(Coords {
                        top: 0.,
                        bottom: 0.,
                        left: 8.,
                        right: 0.,
                    }),
                    ..Default::default()
                })
                .build()
                .finish(),
        );

        row.finish()
    }

    /// Renders a row of what is being limited, along with the current used/limit.
    #[allow(clippy::too_many_arguments)]
    fn render_ai_usage_limit_row(
        &self,
        header: impl Into<Cow<'static, str>>,
        description: impl Into<Cow<'static, str>>,
        used: usize,
        limit: usize,
        is_unlimited: bool,
        workspace_is_delinquent_due_to_payment_issue: bool,
        appearance: &Appearance,
    ) -> Box<dyn warpui::Element> {
        let request_usage_details = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::End)
            .with_child(self.render_request_usage_count(
                used,
                limit,
                is_unlimited,
                workspace_is_delinquent_due_to_payment_issue,
                appearance,
            ));

        let request_usage_description = FormattedTextElement::from_str(
            description,
            appearance.ui_font_family(),
            appearance.ui_font_size(),
        )
        .with_color(blended_colors::text_sub(
            appearance.theme(),
            appearance.theme().surface_1(),
        ));

        Flex::row()
            .with_child(
                Shrinkable::new(
                    2.,
                    Container::new(
                        Flex::column()
                            .with_child(
                                appearance
                                    .ui_builder()
                                    .paragraph(header)
                                    .with_style(UiComponentStyles {
                                        font_color: Some(blended_colors::text_main(
                                            appearance.theme(),
                                            appearance.theme().surface_1(),
                                        )),
                                        margin: Some(Coords {
                                            top: 0.,
                                            bottom: 4.,
                                            left: 0.,
                                            right: 0.,
                                        }),
                                        ..Default::default()
                                    })
                                    .build()
                                    .finish(),
                            )
                            .with_child(request_usage_description.finish())
                            .finish(),
                    )
                    .with_margin_bottom(16.)
                    .finish(),
                )
                .finish(),
            )
            .with_child(
                Shrinkable::new(
                    1.,
                    Container::new(request_usage_details.finish())
                        .with_margin_bottom(16.)
                        .finish(),
                )
                .finish(),
            )
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_main_axis_size(MainAxisSize::Max)
            .finish()
    }
}

impl SettingsWidget for UsageWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "a.i. ai usage limit plan"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_request_usage_model = AIRequestUsageModel::as_ref(app);
        let next_refresh_time = ai_request_usage_model.next_refresh_time();
        let uses_chinese_date_format = crate::i18n::current_languages()
            .first()
            .is_some_and(|language| language.to_string().starts_with("zh"));
        let formatted_next_refresh_time = if uses_chinese_date_format {
            next_refresh_time.format("%Y-%m-%d").to_string()
        } else {
            next_refresh_time.format("%b %d").to_string()
        };
        let workspace_is_delinquent_due_to_payment_issue = UserWorkspaces::as_ref(app)
            .team_for_view_handle(&self.view_handle, app)
            .map(|team| team.billing_metadata.is_delinquent_due_to_payment_issue())
            .unwrap_or_default();

        let usage_header = Container::new(
            Flex::row()
                .with_main_axis_size(MainAxisSize::Max)
                .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(
                    build_sub_header(
                        appearance,
                        crate::t!("settings-ai-usage-header"),
                        Some(styles::header_font_color(true, app)),
                    )
                    .finish(),
                )
                .with_child(
                    appearance
                        .ui_builder()
                        .paragraph(crate::t!(
                            "settings-ai-usage-resets",
                            date = formatted_next_refresh_time.as_str()
                        ))
                        .with_style(UiComponentStyles {
                            font_color: Some(blended_colors::text_sub(
                                appearance.theme(),
                                appearance.theme().surface_1(),
                            )),
                            ..Default::default()
                        })
                        .build()
                        .finish(),
                )
                .finish(),
        )
        .with_padding_bottom(HEADER_PADDING)
        .finish();

        let request_limit_description = crate::t!(
            "settings-ai-usage-limit-description",
            duration = ai_request_usage_model.refresh_duration_to_string()
        );

        let request_usage_row = self.render_ai_usage_limit_row(
            crate::t!("settings-ai-credits-label"),
            request_limit_description,
            ai_request_usage_model.requests_used(),
            ai_request_usage_model.request_limit(),
            ai_request_usage_model.is_unlimited(),
            workspace_is_delinquent_due_to_payment_issue,
            appearance,
        );

        Flex::column()
            .with_children([
                render_separator(appearance),
                usage_header,
                request_usage_row,
            ])
            .finish()
    }
}

struct ActiveAIWidget {
    view_handle: WeakViewHandle<AISettingsPageView>,
    active_ai_toggle: SwitchStateHandle,
    intelligent_autosuggestions_toggle: SwitchStateHandle,
    prompt_suggestions_toggle: SwitchStateHandle,
    code_suggestions_toggle: SwitchStateHandle,
    natural_language_autosuggestions_toggle: SwitchStateHandle,
    git_operations_autogen_toggle: SwitchStateHandle,
}

impl ActiveAIWidget {
    fn new(ctx: &ViewContext<AISettingsPageView>) -> Self {
        Self {
            view_handle: ctx.handle(),
            active_ai_toggle: Default::default(),
            intelligent_autosuggestions_toggle: Default::default(),
            prompt_suggestions_toggle: Default::default(),
            code_suggestions_toggle: Default::default(),
            natural_language_autosuggestions_toggle: Default::default(),
            git_operations_autogen_toggle: Default::default(),
        }
    }
    fn is_next_command_toggleable(&self, app: &AppContext) -> bool {
        UserWorkspaces::as_ref(app).is_next_command_enabled()
            && AISettings::as_ref(app)
                .intelligent_autosuggestions_enabled_internal
                .is_supported_on_current_platform()
    }

    fn is_prompt_suggestions_toggleable(&self, app: &AppContext) -> bool {
        UserWorkspaces::as_ref(app).is_prompt_suggestions_toggleable()
            && AISettings::as_ref(app)
                .prompt_suggestions_enabled_internal
                .is_supported_on_current_platform()
    }

    fn is_suggested_code_banners_toggleable(&self, app: &AppContext) -> bool {
        (self.is_prompt_suggestions_toggleable(app)
            || UserWorkspaces::as_ref(app).is_code_suggestions_toggleable())
            && AISettings::as_ref(app)
                .code_suggestions_enabled_internal
                .is_supported_on_current_platform()
    }

    fn is_natural_language_autosuggestions_toggleable(&self, app: &AppContext) -> bool {
        FeatureFlag::PredictAMQueries.is_enabled()
            && AISettings::as_ref(app)
                .natural_language_autosuggestions_enabled_internal
                .is_supported_on_current_platform()
    }

    fn is_git_operations_autogen_toggleable(&self, app: &AppContext) -> bool {
        FeatureFlag::GitOperationsInCodeReview.is_enabled()
            && AISettings::as_ref(app)
                .git_operations_autogen_enabled_internal
                .is_supported_on_current_platform()
            && UserWorkspaces::as_ref(app).is_git_operations_ai_enabled()
    }

    fn render_next_command_section(
        &self,
        view: &AISettingsPageView,
        app: &warpui::AppContext,
    ) -> Box<dyn warpui::Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_toggleable = ai_settings.is_active_ai_enabled(app);

        Flex::column()
            .with_child(
                render_ai_setting_toggle::<IntelligentAutosuggestionsEnabled>(
                    crate::t!("settings-ai-next-command-label"),
                    AISettingsPageAction::ToggleIntelligentAutosuggestions,
                    *ai_settings.intelligent_autosuggestions_enabled_internal,
                    is_toggleable,
                    self.intelligent_autosuggestions_toggle.clone(),
                    &view.local_only_icon_tooltip_states,
                    app,
                ),
            )
            .with_child(render_ai_setting_description(
                crate::t!("settings-ai-next-command-description"),
                is_toggleable,
                app,
            ))
            .finish()
    }

    fn render_prompt_suggestions_section(
        &self,
        view: &AISettingsPageView,
        app: &warpui::AppContext,
    ) -> Box<dyn warpui::Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_toggleable = ai_settings.is_active_ai_enabled(app);
        Flex::column()
            .with_child(
                render_ai_setting_toggle::<AgentModeQuerySuggestionsEnabled>(
                    crate::t!("settings-ai-prompt-suggestions-label"),
                    AISettingsPageAction::TogglePromptSuggestions,
                    *ai_settings.prompt_suggestions_enabled_internal,
                    is_toggleable,
                    self.prompt_suggestions_toggle.clone(),
                    &view.local_only_icon_tooltip_states,
                    app,
                ),
            )
            .with_child(render_ai_setting_description(
                crate::t!("settings-ai-prompt-suggestions-description"),
                is_toggleable,
                app,
            ))
            .finish()
    }

    fn render_suggested_code_banners_section(
        &self,
        view: &AISettingsPageView,
        app: &warpui::AppContext,
    ) -> Box<dyn warpui::Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_toggleable = ai_settings.is_active_ai_enabled(app);
        Flex::column()
            .with_child(
                render_ai_setting_toggle::<AgentModeQuerySuggestionsEnabled>(
                    crate::t!("settings-ai-suggested-code-banners-label"),
                    AISettingsPageAction::ToggleCodeSuggestions,
                    *ai_settings.code_suggestions_enabled_internal,
                    is_toggleable,
                    self.code_suggestions_toggle.clone(),
                    &view.local_only_icon_tooltip_states,
                    app,
                ),
            )
            .with_child(render_ai_setting_description(
                crate::t!("settings-ai-suggested-code-banners-description"),
                is_toggleable,
                app,
            ))
            .finish()
    }

    fn render_natural_language_autosuggestions_section(
        &self,
        view: &AISettingsPageView,
        app: &warpui::AppContext,
    ) -> Box<dyn warpui::Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_toggleable = ai_settings.is_active_ai_enabled(app);
        Flex::column()
            .with_child(render_ai_setting_toggle::<
                NaturalLanguageAutosuggestionsEnabled,
            >(
                crate::t!("settings-ai-natural-language-autosuggestions-label"),
                AISettingsPageAction::ToggleNaturalLanguageAutosuggestions,
                *ai_settings.natural_language_autosuggestions_enabled_internal,
                is_toggleable,
                self.natural_language_autosuggestions_toggle.clone(),
                &view.local_only_icon_tooltip_states,
                app,
            ))
            .with_child(render_ai_setting_description(
                crate::t!("settings-ai-natural-language-autosuggestions"),
                is_toggleable,
                app,
            ))
            .finish()
    }

    fn render_git_operations_autogen_section(
        &self,
        view: &AISettingsPageView,
        app: &warpui::AppContext,
    ) -> Box<dyn warpui::Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_toggleable = ai_settings.is_active_ai_enabled(app);
        Flex::column()
            .with_child(render_ai_setting_toggle::<GitOperationsAutogenEnabled>(
                crate::t!("settings-ai-git-operations-autogen-label"),
                AISettingsPageAction::ToggleGitOperationsAutogen,
                *ai_settings.git_operations_autogen_enabled_internal,
                is_toggleable,
                self.git_operations_autogen_toggle.clone(),
                &view.local_only_icon_tooltip_states,
                app,
            ))
            .with_child(render_ai_setting_description(
                crate::t!("settings-ai-git-operations-autogen-description"),
                is_toggleable,
                app,
            ))
            .finish()
    }
}

impl SettingsWidget for ActiveAIWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "active ai a.i. next command prompt suggestions code diffs suggested banners passive unit tests commit pull request pr git code review autogen generate"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        self.is_next_command_toggleable(app)
            || self.is_prompt_suggestions_toggleable(app)
            || self.is_suggested_code_banners_toggleable(app)
            || self.is_natural_language_autosuggestions_toggleable(app)
            || self.is_git_operations_autogen_toggleable(app)
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(app);
        let mut column = Flex::column()
            .with_child(render_separator(appearance))
            .with_child(
                Container::new(
                    Flex::row()
                        .with_main_axis_size(MainAxisSize::Max)
                        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                        .with_child(
                            build_sub_header(
                                appearance,
                                crate::t!("settings-ai-active-ai-section"),
                                Some(styles::header_font_color(is_any_ai_enabled, app)),
                            )
                            .finish(),
                        )
                        .with_child(
                            Container::new(render_ai_feature_switch(
                                self.active_ai_toggle.clone(),
                                *ai_settings.is_active_ai_enabled_internal,
                                is_any_ai_enabled,
                                AISettingsPageAction::ToggleActiveAI,
                                app,
                            ))
                            .with_padding_right(TOGGLE_BUTTON_RIGHT_PADDING)
                            .finish(),
                        )
                        .finish(),
                )
                .with_padding_bottom(HEADER_PADDING)
                .finish(),
            );

        if self.is_next_command_toggleable(app) {
            column.add_child(self.render_next_command_section(view, app));
        }

        if self.is_prompt_suggestions_toggleable(app) {
            column.add_child(self.render_prompt_suggestions_section(view, app));
        }

        if self.is_suggested_code_banners_toggleable(app) {
            column.add_child(self.render_suggested_code_banners_section(view, app));
        }

        if self.is_natural_language_autosuggestions_toggleable(app) {
            column.add_child(self.render_natural_language_autosuggestions_section(view, app));
        }

        if self.is_git_operations_autogen_toggleable(app) {
            column.add_child(self.render_git_operations_autogen_section(view, app));
        }

        column.finish()
    }
}

#[derive(Default)]
struct AgentsWidget {
    show_in_prompt_checkbox: MouseStateHandle,
}

impl SettingsWidget for AgentsWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        if MCPServersWidget::should_show_mcp() {
            "ai a.i. agent autonomy profiles allowlist denylist autoexecute permissions models llms planning mcp server"
        } else {
            "ai a.i. agent autonomy profiles allowlist denylist autoexecute permissions models llms planning"
        }
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(app);

        let mut column = Flex::column();

        if FeatureFlag::ProfilesDesignRevamp.is_enabled() {
            column.add_child(
                Container::new(self.render_profiles_section(view, ai_settings, appearance, app))
                    .with_margin_bottom(8.)
                    .finish(),
            );
        } else {
            // Legacy layout: show Agents header + Models + Permissions
            let mut agents_header = Flex::column();
            agents_header.add_child(
                build_sub_header(
                    appearance,
                    crate::t!("settings-ai-agents-header"),
                    Some(styles::header_font_color(is_any_ai_enabled, app)),
                )
                .with_padding_bottom(HEADER_PADDING)
                .finish(),
            );
            agents_header.add_child(render_ai_setting_description(
                crate::t!("settings-ai-agents-description"),
                ai_settings.is_any_ai_enabled(app),
                app,
            ));
            let agents_header = agents_header.finish();
            column.add_children([
                render_separator(appearance),
                Container::new(agents_header)
                    .with_margin_bottom(8.)
                    .finish(),
            ]);
            column.add_children([
                Container::new(self.render_models_section(view, ai_settings, appearance, app))
                    .with_margin_bottom(8.)
                    .finish(),
                Container::new(self.render_permissions_section(view, ai_settings, appearance, app))
                    .with_margin_bottom(8.)
                    .finish(),
            ]);
        };

        column.finish()
    }
}

impl AgentsWidget {
    fn render_profiles_section(
        &self,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(app);

        let header_and_description = Flex::column()
            .with_child(
                build_sub_header(
                    appearance,
                    crate::t!("settings-ai-profiles-header"),
                    Some(styles::header_font_color(is_any_ai_enabled, app)),
                )
                .finish(),
            )
            .with_child(
                Container::new(render_ai_setting_description(
                    crate::t!("settings-ai-profiles-description"),
                    is_any_ai_enabled,
                    app,
                ))
                .with_margin_top(12.)
                .finish(),
            )
            .finish();

        let mut profiles_row = Flex::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(Shrinkable::new(1., header_and_description).finish());

        if FeatureFlag::MultiProfile.is_enabled() {
            profiles_row.add_child(
                Container::new(view.add_profile_button.as_ref(app).render(app))
                    .with_margin_left(16.)
                    .finish(),
            );
        }

        let profiles_header = Container::new(profiles_row.finish())
            .with_margin_bottom(12.0)
            .finish();

        let mut profile_elements = vec![profiles_header];

        for profile_view in &view.profile_views {
            profile_elements.push(
                Container::new(ChildView::new(profile_view).finish())
                    .with_margin_bottom(8.)
                    .finish(),
            );
        }

        Flex::column().with_children(profile_elements).finish()
    }

    fn render_models_section(
        &self,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(app);
        let model_subheader = Container::new(render_custom_size_header(
            appearance,
            crate::t!("settings-ai-models-subheader"),
            14.0,
            Some(styles::header_font_color(is_any_ai_enabled, app)),
        ))
        .with_margin_bottom(8.0)
        .finish();

        let base_model_setting =
            Container::new(self.render_base_model_setting(view, ai_settings, appearance, app))
                .with_margin_bottom(8.0)
                .finish();

        let mut children = vec![model_subheader, base_model_setting];
        if let Some(context_window_setting) =
            self.render_context_window_setting(view, ai_settings, appearance, app)
        {
            children.push(
                Container::new(context_window_setting)
                    .with_margin_bottom(8.0)
                    .finish(),
            );
        }

        Flex::column().with_children(children).finish()
    }

    /// Renders the context window slider + numeric input row shown below the
    /// base model dropdown. Returns `None` if the active base model does not
    /// advertise a configurable context window, global AI is disabled, or the
    /// [`FeatureFlag::ConfigurableContextWindow`] flag is disabled.
    fn render_context_window_setting(
        &self,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Option<Box<dyn Element>> {
        if !FeatureFlag::ConfigurableContextWindow.is_enabled() {
            return None;
        }
        if !ai_settings.is_any_ai_enabled(app) {
            return None;
        }
        let cw = AISettingsPageView::configurable_context_window(app)?;
        let min = cw.min;
        let max = cw.max;

        let label = Container::new(render_body_item_label::<AISettingsPageAction>(
            crate::t!("settings-ai-context-window-label"),
            None,
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
        ))
        .with_margin_bottom(4.0)
        .finish();

        let min_label = appearance
            .ui_builder()
            .span(format!("{min}"))
            .with_style(UiComponentStyles {
                font_size: Some(appearance.ui_font_body()),
                ..Default::default()
            })
            .build()
            .finish();

        let max_label = appearance
            .ui_builder()
            .span(format!("{max}"))
            .with_style(UiComponentStyles {
                font_size: Some(appearance.ui_font_body()),
                ..Default::default()
            })
            .build()
            .finish();

        let current_value = AISettingsPageView::current_context_window_display_value(app)
            .unwrap_or(cw.default_max)
            .clamp(min, max);
        let slider = appearance
            .ui_builder()
            .slider(view.context_window_slider_state.clone())
            .with_range(min as f32..max as f32)
            .with_default_value(current_value as f32)
            .with_style(UiComponentStyles {
                width: Some(CONTEXT_WINDOW_SLIDER_WIDTH),
                margin: Some(Coords::default().left(8.).right(8.)),
                ..Default::default()
            })
            .on_drag(|ctx, _, val| {
                ctx.dispatch_typed_action(AISettingsPageAction::ContextWindowSliderDragged(
                    val.round() as u32,
                ));
            })
            .on_change(|ctx, _, val| {
                ctx.dispatch_typed_action(AISettingsPageAction::SetContextWindowSize(
                    val.round() as u32
                ));
            })
            .build()
            .finish();

        let context_window_editor = view.context_window_editor.clone();
        let input_box = Dismiss::new(
            appearance
                .ui_builder()
                .text_input(view.context_window_editor.clone())
                .with_style(UiComponentStyles {
                    width: Some(CONTEXT_WINDOW_INPUT_BOX_WIDTH),
                    padding: Some(Coords {
                        top: appearance.ui_font_size() / 2.,
                        bottom: appearance.ui_font_size() / 2.,
                        left: appearance.ui_font_size() * 5. / 6.,
                        right: appearance.ui_font_size() * 5. / 6.,
                    }),
                    margin: Some(Coords::default().left(12.)),
                    background: Some(appearance.theme().surface_2().into()),
                    ..Default::default()
                })
                .build()
                .finish(),
        )
        .on_dismiss(move |ctx, app| {
            let buffer_text = context_window_editor.as_ref(app).buffer_text(app);
            let cleaned: String = buffer_text
                .chars()
                .filter(|c| !c.is_whitespace() && *c != ',')
                .collect();
            if let Ok(parsed) = cleaned.parse::<u32>() {
                ctx.dispatch_typed_action(AISettingsPageAction::SetContextWindowSize(parsed));
            }
        })
        .finish();

        let row = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(min_label)
            .with_child(Shrinkable::new(1., slider).finish())
            .with_child(max_label)
            .with_child(input_box)
            .finish();

        let mut column = Flex::column().with_child(label).with_child(row);
        if AISettingsPageView::active_profile_data(app)
            .should_show_long_context_pricing_warning(view.dragged_context_window_value, app)
        {
            column.add_child(render_warning_box(
                WarningBoxConfig::formatted_title(long_context_pricing_warning_title()),
                appearance,
            ));
        }

        Some(column.finish())
    }

    fn render_permissions_section(
        &self,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(app);
        let permissions_subheader = Container::new(render_custom_size_header(
            appearance,
            crate::t!("settings-ai-permissions-subheader"),
            14.0,
            Some(styles::header_font_color(is_any_ai_enabled, app)),
        ))
        .with_margin_bottom(4.0)
        .finish();

        let code_diff_setting =
            BlocklistAIPermissions::as_ref(app).get_apply_code_diffs_setting(app, None);
        let code_diffs = self.render_execution_profile_dropdown(
            &crate::t!("settings-ai-apply-code-diffs"),
            Icon::Code2,
            code_diff_setting.description(),
            &view.apply_code_diffs_dropdown_menu,
            ai_settings,
            appearance,
            app,
        );

        let read_files_setting =
            BlocklistAIPermissions::as_ref(app).get_read_files_setting(app, None);
        let mut read_files_flex = Flex::column().with_main_axis_size(MainAxisSize::Min);
        read_files_flex.add_child(self.render_execution_profile_dropdown(
            &crate::t!("settings-ai-read-files"),
            Icon::Notebook,
            read_files_setting.description(),
            &view.read_files_dropdown_menu,
            ai_settings,
            appearance,
            app,
        ));

        if read_files_setting == ActionPermission::AlwaysAsk {
            let directory_allowlist =
                BlocklistAIPermissions::as_ref(app).get_read_files_allowlist(app, None);
            read_files_flex.add_child(
                Container::new(Self::render_directory_allowlist(
                    directory_allowlist,
                    view,
                    ai_settings,
                    appearance,
                    app,
                ))
                .with_margin_bottom(HEADER_PADDING)
                .finish(),
            );
        }
        let read_files = read_files_flex.finish();

        let execute_commands_setting =
            BlocklistAIPermissions::as_ref(app).get_execute_commands_setting(app, None);
        let mut execute_commands_flex = Flex::column().with_main_axis_size(MainAxisSize::Min);
        execute_commands_flex.add_child(self.render_execution_profile_dropdown(
            &crate::t!("settings-ai-execute-commands"),
            Icon::Terminal,
            execute_commands_setting.description(),
            &view.execute_commands_dropdown_menu,
            ai_settings,
            appearance,
            app,
        ));

        if execute_commands_setting == ActionPermission::AlwaysAsk
            || execute_commands_setting == ActionPermission::AgentDecides
        {
            let command_allowlist =
                BlocklistAIPermissions::as_ref(app).get_execute_commands_allowlist(app, None);
            execute_commands_flex.add_child(
                Container::new(Self::render_command_allowlist(
                    command_allowlist,
                    view,
                    ai_settings,
                    appearance,
                    app,
                ))
                .with_margin_bottom(8.)
                .finish(),
            );
        }

        if execute_commands_setting != ActionPermission::AlwaysAsk {
            let command_denylist = Container::new(Self::render_command_denylist(
                BlocklistAIPermissions::as_ref(app).get_execute_commands_denylist(app, None),
                view,
                ai_settings,
                appearance,
                app,
            ))
            .with_margin_bottom(8.)
            .finish();
            execute_commands_flex.add_child(command_denylist);
        }
        let execute_commands = execute_commands_flex.finish();

        let mut widget_children = vec![permissions_subheader];

        if UserWorkspaces::as_ref(app)
            .ai_autonomy_settings()
            .has_any_overrides()
        {
            widget_children.push(
                Container::new(render_settings_info_banner(
                    &crate::t!("settings-ai-info-banner-managed-by-workspace"),
                    None,
                    appearance,
                ))
                .with_margin_bottom(12.0)
                .finish(),
            );
        }

        widget_children.extend([code_diffs, read_files, execute_commands]);

        let write_to_pty_setting =
            BlocklistAIPermissions::as_ref(app).get_write_to_pty_setting(app, None);
        let write_to_pty = self.render_execution_profile_dropdown(
            &crate::t!("settings-ai-interact-running-commands"),
            Icon::Workflow,
            write_to_pty_setting.description(),
            &view.write_to_pty_autonomy_dropdown_menu,
            ai_settings,
            appearance,
            app,
        );
        widget_children.push(write_to_pty);

        if MCPServersWidget::should_show_mcp() {
            let mcp_permissions = self.render_mcp_permissions(view, ai_settings, appearance, app);
            widget_children.push(mcp_permissions);
        }

        Flex::column().with_children(widget_children).finish()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_execution_profile_dropdown(
        &self,
        header_text: &str,
        header_icon: Icon,
        permission_description: &'static str,
        dropdown_menu: &ViewHandle<Dropdown<AISettingsPageAction>>,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &warpui::AppContext,
    ) -> Box<dyn Element> {
        let header = Container::new(render_body_item_label_with_icon::<AISettingsPageAction>(
            header_text.into(),
            header_icon,
            Some(styles::header_font_color(
                ai_settings.is_any_ai_enabled(app),
                app,
            )),
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
        ))
        .finish();

        let description_color = appearance.theme().disabled_ui_text_color();
        let alert_icon = Container::new(
            ConstrainedBox::new(
                Icon::AlertCircle
                    .to_warpui_icon(
                        appearance
                            .theme()
                            .sub_text_color(appearance.theme().surface_2()),
                    )
                    .finish(),
            )
            .with_width(14.)
            .with_height(14.)
            .finish(),
        )
        .with_margin_right(4.)
        .finish();
        let text = Text::new(
            permission_description,
            appearance.ui_font_family(),
            appearance.ui_font_size(),
        )
        .with_color(description_color.into())
        .finish();
        let description = Flex::row()
            .with_children([alert_icon, Shrinkable::new(1.0, text).finish()])
            .finish();

        Container::new(
            Flex::column()
                .with_child(header)
                .with_child(ChildView::new(dropdown_menu).finish())
                .with_child(description)
                .finish(),
        )
        .with_margin_bottom(12.)
        .finish()
    }

    fn render_command_denylist(
        command_denylist: Vec<AgentModeCommandExecutionPredicate>,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_disabled = !ai_settings.is_any_ai_enabled(app);
        let org_denylist = BlocklistAIPermissions::get_org_execute_commands_denylist(app);
        let mut tooltip_idx = 0usize;
        let list = render_input_list(
            None,
            command_denylist
                .into_iter()
                .zip(view.command_denylist_mouse_state_handles.clone())
                .rev()
                .map(|(cmd, mouse_state_handle)| {
                    let is_org = org_denylist.contains(&cmd);
                    let tooltip_mouse_state = if is_org {
                        let handle = view
                            .command_denylist_tooltip_mouse_state_handles
                            .get(tooltip_idx)
                            .cloned();
                        tooltip_idx += 1;
                        handle
                    } else {
                        None
                    };
                    InputListItem {
                        item: cmd.to_string(),
                        mouse_state_handle,
                        on_remove_action: AISettingsPageAction::RemoveFromProfileCommandDenylist(
                            cmd,
                        ),
                        is_disabled: is_org || ai_disabled,
                        tooltip_mouse_state,
                    }
                }),
            Some(&view.command_denylist_editor),
            appearance,
        );
        render_ai_list(
            &crate::t!("settings-ai-command-denylist"),
            &crate::t!("settings-ai-command-denylist-description"),
            list,
            view,
            ai_settings,
            app,
        )
    }

    fn render_command_allowlist(
        command_allowlist: Vec<AgentModeCommandExecutionPredicate>,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let disabled = !ai_settings.is_command_allowlist_editable(app);
        let list = render_input_list(
            None,
            command_allowlist
                .into_iter()
                .zip(view.command_allowlist_mouse_state_handles.clone())
                .rev()
                .map(move |(cmd, mouse_state_handle)| InputListItem {
                    item: cmd.to_string(),
                    mouse_state_handle,
                    on_remove_action: AISettingsPageAction::RemoveFromProfileCommandAllowlist(cmd),
                    is_disabled: disabled,
                    tooltip_mouse_state: None,
                }),
            Some(&view.command_allowlist_editor),
            appearance,
        );

        render_ai_list(
            &crate::t!("settings-ai-command-allowlist"),
            &crate::t!("settings-ai-command-allowlist-description"),
            list,
            view,
            ai_settings,
            app,
        )
    }

    fn render_directory_allowlist(
        directory_allowlist: Vec<PathBuf>,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let disabled = !ai_settings.is_directory_allowlist_editable(app);
        let list = render_input_list(
            None,
            directory_allowlist
                .clone()
                .into_iter()
                .zip(view.directory_allowlist_mouse_state_handles.clone())
                .rev()
                .map(move |(path, mouse_state_handle)| InputListItem {
                    item: path.display().to_string(),
                    mouse_state_handle,
                    on_remove_action: AISettingsPageAction::RemoveFromProfileDirectoryAllowlist(
                        path,
                    ),
                    is_disabled: disabled,
                    tooltip_mouse_state: None,
                }),
            Some(&view.directory_allowlist_editor),
            appearance,
        );

        render_ai_list(
            &crate::t!("settings-ai-directory-allowlist"),
            &crate::t!("settings-ai-directory-allowlist-description"),
            list,
            view,
            ai_settings,
            app,
        )
    }

    fn render_base_model_setting(
        &self,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let show_in_prompt_checkbox = {
            let is_checked = *SessionSettings::as_ref(app).show_model_selectors_in_prompt;

            let mut checkbox = appearance
                .ui_builder()
                .checkbox(self.show_in_prompt_checkbox.clone(), None)
                .check(is_checked);

            if !ai_settings.is_any_ai_enabled(app) {
                checkbox = checkbox.disabled();
            }

            Container::new(
                Flex::row()
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_children([
                        checkbox
                            .build()
                            .on_click(move |ctx, _, _| {
                                ctx.dispatch_typed_action(
                                    AISettingsPageAction::ToggleShowBaseModelPickerInPrompt,
                                );
                            })
                            .finish(),
                        appearance
                            .ui_builder()
                            .span(crate::t!("settings-ai-show-model-picker-in-prompt"))
                            .with_style(UiComponentStyles {
                                font_color: Some(
                                    theme.sub_text_color(theme.surface_2()).into_solid(),
                                ),
                                font_size: Some(appearance.ui_font_body()),
                                ..Default::default()
                            })
                            .build()
                            .finish(),
                    ])
                    .finish(),
            )
            .with_margin_top(-6.0)
            .with_margin_left(-4.0)
            .finish()
        };

        render_dropdown_item(
            appearance,
            &crate::t!("settings-ai-base-model"),
            Some(&crate::t!("settings-ai-base-model-description")),
            Some(show_in_prompt_checkbox),
            LocalOnlyIconState::Hidden,
            (!ai_settings.is_any_ai_enabled(app))
                .then(|| appearance.theme().disabled_ui_text_color()),
            &view.base_model_dropdown,
        )
    }

    fn render_mcp_permissions(
        &self,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let all_runnable_mcp_servers =
            TemplatableMCPServerManager::get_all_templatable_mcp_server_names(app);
        if all_runnable_mcp_servers.is_empty() {
            self.render_mcp_permissions_zero_state(ai_settings, appearance, app)
        } else {
            self.render_mcp_permissions_with_servers(view, ai_settings, appearance, app)
        }
    }

    fn render_mcp_permissions_zero_state(
        &self,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let header = Container::new(render_body_item_label_with_icon::<AISettingsPageAction>(
            crate::t!("settings-ai-call-mcp-servers"),
            Icon::Dataflow,
            Some(styles::header_font_color(
                ai_settings.is_any_ai_enabled(app),
                app,
            )),
            None,
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
        ))
        .with_margin_bottom(4.)
        .finish();

        let subtext = {
            let subtext_fragments = vec![
                FormattedTextFragment::plain_text(crate::t!("settings-ai-mcp-empty-description")),
                FormattedTextFragment::hyperlink_action(
                    crate::t!("settings-ai-add-server"),
                    AISettingsPageAction::OpenMCPServerCollection,
                ),
                FormattedTextFragment::plain_text(crate::t!("settings-ai-mcp-empty-or")),
                FormattedTextFragment::hyperlink(crate::t!("settings-ai-mcp-empty-learn-more"), ""),
            ];

            Container::new(
                FormattedTextElement::new(
                    FormattedText::new([FormattedTextLine::Line(subtext_fragments)]),
                    appearance.ui_font_body(),
                    appearance.ui_font_family(),
                    appearance.ui_font_family(),
                    styles::description_font_color(ai_settings.is_any_ai_enabled(app), app).into(),
                    HighlightedHyperlink::default(),
                )
                .with_heading_to_font_size_multipliers(
                    appearance.heading_font_size_multipliers().clone(),
                )
                .with_hyperlink_font_color(appearance.theme().accent().into_solid())
                .register_default_click_handlers_with_action_support(|hyperlink_lens, ctx, _app| {
                    match hyperlink_lens {
                        HyperlinkLens::Url(url) => {
                            ctx.dispatch_typed_action(AISettingsPageAction::HyperlinkClick(
                                HyperlinkUrl {
                                    url: url.to_owned(),
                                },
                            ));
                        }
                        HyperlinkLens::Action(action_ref) => {
                            if let Some(action) =
                                action_ref.as_any().downcast_ref::<AISettingsPageAction>()
                            {
                                ctx.dispatch_typed_action(action.clone());
                            }
                        }
                    }
                })
                .finish(),
            )
            .with_margin_bottom(4.0)
            .finish()
        };

        Flex::column()
            .with_child(header)
            .with_child(subtext)
            .finish()
    }

    fn render_mcp_permissions_with_servers(
        &self,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let mut column = Flex::column();

        let current_mcp_setting =
            BlocklistAIPermissions::as_ref(app).get_mcp_permissions_setting(app, None);

        let permission_setting = self.render_execution_profile_dropdown(
            &crate::t!("settings-ai-call-mcp-servers"),
            Icon::Dataflow,
            current_mcp_setting.description(),
            &view.mcp_permissions_dropdown_menu,
            ai_settings,
            appearance,
            app,
        );
        column.add_child(permission_setting);

        if current_mcp_setting == ActionPermission::AlwaysAsk
            || current_mcp_setting == ActionPermission::AgentDecides
        {
            let allowlist = self.render_mcp_list(
                &crate::t!("settings-ai-mcp-allowlist"),
                &crate::t!("settings-ai-mcp-allowlist-description"),
                &view.mcp_allowlist_dropdown,
                BlocklistAIPermissions::as_ref(app).get_mcp_allowlist(app, None),
                view.mcp_allowlist_mouse_state_handles.clone(),
                AISettingsPageAction::RemoveFromMCPAllowlist,
                ai_settings,
                appearance,
                app,
            );
            column.add_child(allowlist);
        }

        if current_mcp_setting == ActionPermission::AlwaysAllow
            || current_mcp_setting == ActionPermission::AgentDecides
        {
            let denylist = self.render_mcp_list(
                &crate::t!("settings-ai-mcp-denylist"),
                &crate::t!("settings-ai-mcp-denylist-description"),
                &view.mcp_denylist_dropdown,
                BlocklistAIPermissions::as_ref(app).get_mcp_denylist(app, None),
                view.mcp_denylist_mouse_state_handles.clone(),
                AISettingsPageAction::RemoveFromMCPDenylist,
                ai_settings,
                appearance,
                app,
            );
            column.add_child(denylist);
        }

        column.finish()
    }

    // Helper function to render the allow and denylists for mcp servers
    #[allow(clippy::too_many_arguments)]
    fn render_mcp_list(
        &self,
        title: &str,
        description: &str,
        dropdown: &ViewHandle<FilterableDropdown<AISettingsPageAction>>,
        items: Vec<uuid::Uuid>,
        mouse_state_handles: Vec<MouseStateHandle>,
        action: impl Fn(uuid::Uuid) -> AISettingsPageAction,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let selector = Container::new(
            Flex::row()
                .with_main_axis_size(MainAxisSize::Max)
                .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                .with_children(vec![
                    Shrinkable::new(
                        1.0,
                        Container::new(render_dropdown_item_label(
                            title.to_string(),
                            Some(description.to_string()),
                            LocalOnlyIconState::Hidden,
                            (!ai_settings.is_any_ai_enabled(app))
                                .then(|| appearance.theme().disabled_ui_text_color()),
                            appearance,
                        ))
                        .with_margin_right(4.)
                        .finish(),
                    )
                    .finish(),
                    ChildView::new(dropdown).finish(),
                ])
                .finish(),
        )
        .with_margin_bottom(2.)
        .finish();

        let disabled = !ai_settings.is_any_ai_enabled(app);
        let items = render_input_list(
            None,
            items
                .into_iter()
                .rev()
                .zip(mouse_state_handles.clone())
                .filter_map(move |(uuid, mouse_state_handle)| {
                    let server_name = TemplatableMCPServerManager::get_mcp_name(&uuid, app);
                    server_name.map(|server_name| InputListItem {
                        item: server_name,
                        mouse_state_handle,
                        on_remove_action: action(uuid),
                        is_disabled: disabled,
                        tooltip_mouse_state: None,
                    })
                }),
            None,
            appearance,
        );

        Container::new(Flex::column().with_children(vec![selector, items]).finish())
            .with_margin_bottom(8.)
            .finish()
    }
}

#[derive(Default)]
struct AIInputWidget {
    incorrect_autodetection_highlight_index: HighlightedHyperlink,
    autodetection_toggle: SwitchStateHandle,
    nld_in_terminal_toggle: SwitchStateHandle,
    show_input_hint_toggle: SwitchStateHandle,
    show_agent_tips_toggle: SwitchStateHandle,
    // 「显示 Agent 快捷键提示」开关对应的 switch 状态句柄。
    show_agent_zero_state_hints_toggle: SwitchStateHandle,
    include_agent_commands_in_history_toggle: SwitchStateHandle,
    auto_approve_bypasses_command_denylist_toggle: SwitchStateHandle,
}

impl SettingsWidget for AIInputWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "oz agent ai input natural language detection autodetection prompt terminal command commands history shell executed execution queue interrupt submission submit auto-queue response while responding default long-running long running lrc auto-approve fast forward full access denylist permissions"
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(app);

        let input_header = build_sub_header(
            appearance,
            crate::t!("settings-ai-input-section"),
            Some(styles::header_font_color(is_any_ai_enabled, app)),
        )
        .with_padding_bottom(HEADER_PADDING)
        .finish();

        let natural_language_detection_section = Self::render_natural_language_detection_section(
            self.incorrect_autodetection_highlight_index.clone(),
            self.autodetection_toggle.clone(),
            self.nld_in_terminal_toggle.clone(),
            view,
            ai_settings,
            appearance,
            app,
        );

        let show_input_hint_text = render_ai_setting_toggle::<ShowHintText>(
            crate::t!("settings-ai-show-input-hint-text"),
            AISettingsPageAction::ToggleShowInputHintText,
            *InputSettings::as_ref(app).show_hint_text,
            is_any_ai_enabled,
            self.show_input_hint_toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        );

        let mut widget_children = vec![
            render_separator(appearance),
            input_header,
            natural_language_detection_section,
            show_input_hint_text,
        ];

        if FeatureFlag::AgentTips.is_enabled() {
            let agent_tips_toggle = render_ai_setting_toggle::<ShowAgentTips>(
                crate::t!("settings-ai-show-agent-tips"),
                AISettingsPageAction::ToggleShowAgentTips,
                *InputSettings::as_ref(app).show_agent_tips,
                is_any_ai_enabled,
                self.show_agent_tips_toggle.clone(),
                &view.local_only_icon_tooltip_states,
                app,
            );
            widget_children.push(agent_tips_toggle);
        }

        // 「显示 Agent 快捷键提示」：控制零状态三件套与 message bar 底部 4 项 hint。
        widget_children.push(render_ai_setting_toggle::<ShowAgentZeroStateHints>(
            crate::t!("settings-ai-show-agent-zero-state-hints"),
            AISettingsPageAction::ToggleShowAgentZeroStateHints,
            *InputSettings::as_ref(app).show_agent_zero_state_hints,
            is_any_ai_enabled,
            self.show_agent_zero_state_hints_toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        ));

        widget_children.push(render_ai_setting_toggle::<IncludeAgentCommandsInHistory>(
            crate::t!("settings-ai-include-agent-commands-in-history"),
            AISettingsPageAction::ToggleIncludeAgentCommandsInHistory,
            *ai_settings.include_agent_commands_in_history,
            is_any_ai_enabled,
            self.include_agent_commands_in_history_toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        ));

        let (denylist_bypass_label, denylist_bypass_description) =
            if FeatureFlag::AgentApprovalModes.is_enabled() {
                (
                    crate::t!("settings-ai-full-access-bypass-command-denylist"),
                    crate::t!("settings-ai-full-access-bypass-command-denylist-description"),
                )
            } else {
                (
                    crate::t!("settings-ai-auto-approve-bypass-command-denylist"),
                    crate::t!("settings-ai-auto-approve-bypass-command-denylist-description"),
                )
            };
        widget_children.push(
            Flex::column()
                .with_child(
                    render_ai_setting_toggle::<AutoApproveBypassesCommandDenylist>(
                        denylist_bypass_label,
                        AISettingsPageAction::ToggleAutoApproveBypassesCommandDenylist,
                        *ai_settings.auto_approve_bypasses_command_denylist,
                        is_any_ai_enabled,
                        self.auto_approve_bypasses_command_denylist_toggle.clone(),
                        &view.local_only_icon_tooltip_states,
                        app,
                    ),
                )
                .with_child(render_ai_setting_description(
                    denylist_bypass_description,
                    is_any_ai_enabled,
                    app,
                ))
                .finish(),
        );

        if FeatureFlag::QueueSlashCommand.is_enabled() {
            widget_children.push(render_dropdown_item(
                appearance,
                &crate::t!("settings-ai-default-prompt-submission-mode"),
                Some(&crate::t!(
                    "settings-ai-default-prompt-submission-description"
                )),
                None,
                LocalOnlyIconState::for_setting(
                    PromptSubmissionMode::storage_key(),
                    PromptSubmissionMode::sync_to_cloud(),
                    &mut view.local_only_icon_tooltip_states.borrow_mut(),
                    app,
                ),
                (!is_any_ai_enabled).then(|| appearance.theme().disabled_ui_text_color()),
                &view.default_prompt_submission_mode_dropdown,
            ));

            // Only meaningful in Interrupt mode: with Queue selected, prompts already
            // queue until the end of the full response, so the LRC mode is hidden.
            if ai_settings.default_prompt_submission_mode == PromptSubmissionMode::Interrupt {
                widget_children.push(
                    Container::new(render_dropdown_item(
                        appearance,
                        &crate::t!("settings-ai-default-long-running-submission-mode"),
                        Some(&crate::t!(
                            "settings-ai-default-long-running-submission-description"
                        )),
                        None,
                        LocalOnlyIconState::for_setting(
                            LongRunningCommandSubmissionMode::storage_key(),
                            LongRunningCommandSubmissionMode::sync_to_cloud(),
                            &mut view.local_only_icon_tooltip_states.borrow_mut(),
                            app,
                        ),
                        (!is_any_ai_enabled).then(|| appearance.theme().disabled_ui_text_color()),
                        &view.lrc_submission_mode_dropdown,
                    ))
                    .with_margin_top(styles::DESCRIPTION_MARGIN_BOTTOM)
                    .finish(),
                );
            }
        }

        Flex::column().with_children(widget_children).finish()
    }
}

impl AIInputWidget {
    fn render_natural_language_detection_section(
        incorrect_autodetection_highlight_index: HighlightedHyperlink,
        autodetection_toggle: SwitchStateHandle,
        nld_in_terminal_toggle: SwitchStateHandle,
        view: &AISettingsPageView,
        ai_settings: &AISettings,
        appearance: &Appearance,
        app: &warpui::AppContext,
    ) -> Box<dyn warpui::Element> {
        let is_toggleable = ai_settings.is_any_ai_enabled(app);
        let is_nld_enabled = *ai_settings.ai_autodetection_enabled_internal.value();

        let autodetection_denylist_input_field = appearance
            .ui_builder()
            .text_input(view.autodetection_denylist_editor.clone())
            .with_style(UiComponentStyles {
                width: Some(280.),
                padding: Some(Coords {
                    top: appearance.ui_font_size() / 3.,
                    bottom: appearance.ui_font_size() / 3.,
                    left: appearance.ui_font_size() / 2.,
                    right: appearance.ui_font_size() / 2.,
                }),
                ..Default::default()
            })
            .build()
            .finish();

        let mut section = Flex::column();

        if FeatureFlag::AgentView.is_enabled() {
            static AUTODETECTION_DESCRIPTION_FRAGMENTS: LazyLock<Vec<FormattedTextFragment>> =
                LazyLock::new(|| {
                    vec![
                        FormattedTextFragment::plain_text(crate::t!(
                            "settings-ai-incorrect-detection"
                        )),
                        FormattedTextFragment::hyperlink(
                            crate::t!("settings-ai-report-detection"),
                            "https://warpdotdev.typeform.com/to/offrTIpq",
                        ),
                    ]
                });

            section.add_children([
                render_ai_setting_toggle::<NLDInTerminalEnabled>(
                    crate::t!("settings-ai-autodetect-agent-prompts"),
                    AISettingsPageAction::ToggleNLDInTerminal,
                    ai_settings.is_nld_in_terminal_enabled(app),
                    is_toggleable,
                    nld_in_terminal_toggle,
                    &view.local_only_icon_tooltip_states,
                    app,
                ),
                render_ai_setting_toggle::<AIAutoDetectionEnabled>(
                    crate::t!("settings-ai-autodetect-terminal-commands"),
                    AISettingsPageAction::ToggleAIInputAutoDetection,
                    is_nld_enabled,
                    is_toggleable,
                    autodetection_toggle,
                    &view.local_only_icon_tooltip_states,
                    app,
                ),
                Container::new(
                    FormattedTextElement::new(
                        FormattedText::new([FormattedTextLine::Line(
                            (*AUTODETECTION_DESCRIPTION_FRAGMENTS).clone(),
                        )]),
                        appearance.ui_font_body(),
                        appearance.ui_font_family(),
                        appearance.ui_font_family(),
                        styles::description_font_color(is_toggleable, app).into(),
                        incorrect_autodetection_highlight_index,
                    )
                    .with_heading_to_font_size_multipliers(
                        appearance.heading_font_size_multipliers().clone(),
                    )
                    .with_hyperlink_font_color(appearance.theme().accent().into_solid())
                    .register_default_click_handlers(|url, ctx, _| {
                        ctx.dispatch_typed_action(AISettingsPageAction::HyperlinkClick(url));
                    })
                    .finish(),
                )
                .with_margin_top(styles::DESCRIPTION_NEGATIVE_MARGIN_OFFSET)
                .with_margin_bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
                .with_margin_right(styles::TOGGLE_WIDTH_MARGIN)
                .finish(),
            ])
        } else {
            static NATURAL_LANGUAGE_DETECTION_DESCRIPTION_FRAGMENTS: LazyLock<
                Vec<FormattedTextFragment>,
            > = LazyLock::new(|| {
                vec![
                    FormattedTextFragment::plain_text(crate::t!(
                        "settings-ai-natural-language-detection-description"
                    )),
                    FormattedTextFragment::plain_text(crate::t!(
                        "settings-ai-incorrect-input-detection"
                    )),
                    FormattedTextFragment::hyperlink(
                        crate::t!("settings-ai-report-detection"),
                        "https://warpdotdev.typeform.com/to/offrTIpq",
                    ),
                ]
            });

            section.add_children([
                render_ai_setting_toggle::<AIAutoDetectionEnabled>(
                    crate::t!("settings-ai-natural-language-detection"),
                    AISettingsPageAction::ToggleAIInputAutoDetection,
                    is_nld_enabled,
                    is_toggleable,
                    autodetection_toggle,
                    &view.local_only_icon_tooltip_states,
                    app,
                ),
                Container::new(
                    FormattedTextElement::new(
                        FormattedText::new([FormattedTextLine::Line(
                            (*NATURAL_LANGUAGE_DETECTION_DESCRIPTION_FRAGMENTS).clone(),
                        )]),
                        appearance.ui_font_body(),
                        appearance.ui_font_family(),
                        appearance.ui_font_family(),
                        styles::description_font_color(is_toggleable, app).into(),
                        incorrect_autodetection_highlight_index,
                    )
                    .with_heading_to_font_size_multipliers(
                        appearance.heading_font_size_multipliers().clone(),
                    )
                    .with_hyperlink_font_color(appearance.theme().accent().into_solid())
                    .register_default_click_handlers(|url, ctx, _| {
                        ctx.dispatch_typed_action(AISettingsPageAction::HyperlinkClick(url));
                    })
                    .finish(),
                )
                .with_margin_top(styles::DESCRIPTION_NEGATIVE_MARGIN_OFFSET)
                .with_margin_bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
                .with_margin_right(styles::TOGGLE_WIDTH_MARGIN)
                .finish(),
            ]);
        }

        section
            .with_child(render_ai_setting_label::<AICommandDenylist>(
                crate::t!("settings-ai-natural-language-denylist"),
                is_toggleable,
                &view.local_only_icon_tooltip_states,
                app,
            ))
            .with_child(render_ai_setting_description(
                crate::t!("settings-ai-natural-language-denylist-description"),
                is_toggleable,
                app,
            ))
            .with_child(
                Container::new(autodetection_denylist_input_field)
                    .with_margin_bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
                    .finish(),
            )
            .finish()
    }
}

#[derive(Default)]
struct MCPServersWidget {
    manage_mcp_servers_button: MouseStateHandle,
    mcp_docs_link_index: HighlightedHyperlink,
    file_based_mcp_toggle: SwitchStateHandle,
    file_based_mcp_docs_link_index: HighlightedHyperlink,
}

impl MCPServersWidget {
    fn should_show_mcp() -> bool {
        FeatureFlag::McpServer.is_enabled() && ContextFlag::ShowMCPServers.is_enabled()
    }
}

impl SettingsWidget for MCPServersWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "oz agent mcp server servers model context protocol file-based file based project claude .mcp.json .claude/.mcp.json .codex config.toml .codex/config.toml"
    }

    fn should_render(&self, _app: &AppContext) -> bool {
        MCPServersWidget::should_show_mcp()
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let is_any_ai_enabled = AISettings::as_ref(app).is_any_ai_enabled(app);
        let ai_settings = AISettings::as_ref(app);

        let header = build_sub_header(
            appearance,
            crate::t!("settings-ai-mcp-servers-section"),
            Some(styles::header_font_color(is_any_ai_enabled, app)),
        )
        .with_padding_bottom(HEADER_PADDING)
        .finish();

        let mcp_description = vec![
            FormattedTextFragment::plain_text(crate::t!("settings-ai-mcp-description")),
            FormattedTextFragment::hyperlink(crate::t!("common-learn-more"), ""),
        ];

        let description = Container::new(
            FormattedTextElement::new(
                FormattedText::new([FormattedTextLine::Line(mcp_description)]),
                appearance.ui_font_body(),
                appearance.ui_font_family(),
                appearance.ui_font_family(),
                styles::description_font_color(is_any_ai_enabled, app).into(),
                self.mcp_docs_link_index.clone(),
            )
            .with_heading_to_font_size_multipliers(
                appearance.heading_font_size_multipliers().clone(),
            )
            .with_hyperlink_font_color(appearance.theme().accent().into_solid())
            .register_default_click_handlers(|url, ctx, _| {
                ctx.dispatch_typed_action(AISettingsPageAction::HyperlinkClick(url));
            })
            .finish(),
        )
        .with_margin_top(styles::DESCRIPTION_NEGATIVE_MARGIN_OFFSET)
        .with_margin_bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
        .with_margin_right(styles::TOGGLE_WIDTH_MARGIN)
        .finish();

        let file_based_mcp_toggle = if FeatureFlag::FileBasedMcp.is_enabled() {
            Some(
                Flex::column()
                    .with_child(render_ai_setting_toggle::<FileBasedMcpEnabled>(
                        crate::t!("settings-ai-file-based-mcp-toggle"),
                        AISettingsPageAction::ToggleFileBasedMcp,
                        *ai_settings.file_based_mcp_enabled,
                        is_any_ai_enabled,
                        self.file_based_mcp_toggle.clone(),
                        &view.local_only_icon_tooltip_states,
                        app,
                    ))
                    .with_child({
                        static FILE_BASED_MCP_DESCRIPTION_FRAGMENTS: LazyLock<
                            Vec<FormattedTextFragment>,
                        > = LazyLock::new(|| {
                            vec![
                                FormattedTextFragment::plain_text(crate::t!(
                                    "settings-ai-file-based-mcp-description"
                                )),
                                FormattedTextFragment::hyperlink(
                                    crate::t!("settings-ai-file-based-mcp-supported-providers"),
                                    "",
                                ),
                            ]
                        });
                        Container::new(
                            FormattedTextElement::new(
                                FormattedText::new([FormattedTextLine::Line(
                                    (*FILE_BASED_MCP_DESCRIPTION_FRAGMENTS).clone(),
                                )]),
                                appearance.ui_font_body(),
                                appearance.ui_font_family(),
                                appearance.ui_font_family(),
                                styles::description_font_color(is_any_ai_enabled, app).into(),
                                self.file_based_mcp_docs_link_index.clone(),
                            )
                            .with_heading_to_font_size_multipliers(
                                appearance.heading_font_size_multipliers().clone(),
                            )
                            .with_hyperlink_font_color(appearance.theme().accent().into_solid())
                            .register_default_click_handlers(|url, ctx, _| {
                                ctx.dispatch_typed_action(AISettingsPageAction::HyperlinkClick(
                                    url,
                                ));
                            })
                            .finish(),
                        )
                        .with_margin_top(styles::DESCRIPTION_NEGATIVE_MARGIN_OFFSET)
                        .with_margin_bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
                        .with_margin_right(styles::TOGGLE_WIDTH_MARGIN)
                        .finish()
                    })
                    .finish(),
            )
        } else {
            None
        };

        let button = render_full_pane_width_ai_button(
            &crate::t!("settings-ai-manage-mcp-servers"),
            is_any_ai_enabled,
            self.manage_mcp_servers_button.clone(),
            AISettingsPageAction::OpenMCPServerCollection,
            appearance,
        );

        let mut column = Flex::column()
            .with_child(header)
            .with_child(description)
            .with_child(button);

        if let Some(toggle) = file_based_mcp_toggle {
            column = column.with_child(toggle);
        }
        column.finish()
    }
}

#[derive(Default)]
struct RulesWidget {
    rules_toggle: SwitchStateHandle,
    rules_link_index: HighlightedHyperlink,
}

impl SettingsWidget for RulesWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "fact memory memories rules conventions"
    }

    fn should_render(&self, _app: &AppContext) -> bool {
        FeatureFlag::AIRules.is_enabled()
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);
        let toggle = render_ai_setting_toggle::<MemoryEnabled>(
            crate::t!("settings-ai-rules-label"),
            AISettingsPageAction::ToggleRules,
            *ai_settings.memory_enabled,
            ai_settings.is_any_ai_enabled(app),
            self.rules_toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        );

        let rules_description = vec![
            FormattedTextFragment::plain_text(format!(
                "{} ",
                crate::t!("settings-ai-rules-description")
            )),
            FormattedTextFragment::hyperlink(crate::t!("settings-ai-learn-more"), ""),
        ];
        let description = Container::new(
            FormattedTextElement::new(
                FormattedText::new([FormattedTextLine::Line(rules_description)]),
                appearance.ui_font_body(),
                appearance.ui_font_family(),
                appearance.ui_font_family(),
                styles::description_font_color(ai_settings.is_any_ai_enabled(app), app).into(),
                self.rules_link_index.clone(),
            )
            .with_heading_to_font_size_multipliers(
                appearance.heading_font_size_multipliers().clone(),
            )
            .with_hyperlink_font_color(appearance.theme().accent().into_solid())
            .register_default_click_handlers(|url, ctx, _| {
                ctx.dispatch_typed_action(AISettingsPageAction::HyperlinkClick(url));
            })
            .finish(),
        )
        .with_margin_top(styles::DESCRIPTION_NEGATIVE_MARGIN_OFFSET)
        .with_margin_bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
        .with_margin_right(styles::TOGGLE_WIDTH_MARGIN)
        .finish();

        Flex::column()
            .with_child(toggle)
            .with_child(description)
            .finish()
    }
}

#[derive(Default)]
struct SuggestedRulesWidget {
    rule_suggestions_toggle: SwitchStateHandle,
}

impl SettingsWidget for SuggestedRulesWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "suggested rules suggest save"
    }

    fn should_render(&self, _app: &AppContext) -> bool {
        FeatureFlag::AIRules.is_enabled() && FeatureFlag::SuggestedRules.is_enabled()
    }

    fn render(
        &self,
        view: &Self::View,
        _appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);
        let toggle = render_ai_setting_toggle::<RuleSuggestionsEnabled>(
            crate::t!("settings-ai-suggested-rules-label"),
            AISettingsPageAction::ToggleRuleSuggestions,
            *ai_settings.rule_suggestions_enabled_internal,
            ai_settings.is_any_ai_enabled(app),
            self.rule_suggestions_toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        );

        let description = render_ai_setting_description(
            crate::t!("settings-ai-suggested-rules-description"),
            ai_settings.is_any_ai_enabled(app),
            app,
        );

        Flex::column()
            .with_child(toggle)
            .with_child(description)
            .finish()
    }
}

#[derive(Default)]
struct ManageRulesWidget {
    manage_rules_button: MouseStateHandle,
}

impl SettingsWidget for ManageRulesWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "manage rules rule collection"
    }

    fn should_render(&self, _app: &AppContext) -> bool {
        FeatureFlag::AIRules.is_enabled()
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        render_full_pane_width_ai_button(
            &crate::t!("settings-ai-manage-rules"),
            AISettings::as_ref(app).is_any_ai_enabled(app),
            self.manage_rules_button.clone(),
            AISettingsPageAction::OpenAIFactCollection,
            appearance,
        )
    }
}

// 去中心化分支不再把该 widget 加入 knowledge_widgets(),保留定义以便后续按需恢复。
#[allow(dead_code)]
#[derive(Default)]
struct WarpDriveContextWidget {
    warp_drive_context_toggle: SwitchStateHandle,
}

impl SettingsWidget for WarpDriveContextWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "warp drive agent context contents personal team developer workflows environments notebooks environment variables"
    }

    fn should_render(&self, _app: &AppContext) -> bool {
        FeatureFlag::AIRules.is_enabled()
    }

    fn render(
        &self,
        view: &Self::View,
        _appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);
        let toggle = render_ai_setting_toggle::<WarpDriveContextEnabled>(
            &crate::t!("settings-ai-drive-context-label"),
            AISettingsPageAction::ToggleWarpDriveContext,
            *ai_settings.warp_drive_context_enabled,
            ai_settings.is_any_ai_enabled(app),
            self.warp_drive_context_toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        );

        let description = render_ai_setting_description(
            crate::t!("settings-ai-drive-context-description"),
            ai_settings.is_any_ai_enabled(app),
            app,
        );

        Flex::column()
            .with_child(toggle)
            .with_child(description)
            .finish()
    }
}

#[derive(Default)]
struct VoiceWidget {
    voice_input_toggle: SwitchStateHandle,
    wispr_highlight_index: HighlightedHyperlink,
}

impl VoiceWidget {
    fn render_voice_section(
        &self,
        view: &AISettingsPageView,
        appearance: &Appearance,
        app: &warpui::AppContext,
    ) -> Box<dyn warpui::Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_toggleable = ai_settings.is_any_ai_enabled(app);
        let mut column = Flex::column().with_child(render_ai_setting_toggle::<VoiceInputEnabled>(
            crate::t!("settings-ai-voice-input-label"),
            AISettingsPageAction::ToggleVoiceInput,
            *ai_settings.voice_input_enabled_internal,
            is_toggleable,
            self.voice_input_toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        ));

        let voice_input_description_text_fragments = vec![
            FormattedTextFragment::plain_text(crate::t!("settings-ai-voice-description-prefix")),
            FormattedTextFragment::hyperlink("Wispr Flow", WISPR_FLOW_URL),
            FormattedTextFragment::plain_text(crate::t!("settings-ai-voice-description-suffix")),
        ];

        let voice_input_description = FormattedTextElement::new(
            FormattedText::new([FormattedTextLine::Line(
                voice_input_description_text_fragments,
            )]),
            appearance.ui_font_size(),
            appearance.ui_font_family(),
            appearance.ui_font_family(),
            styles::description_font_color(is_toggleable, app).into(),
            self.wispr_highlight_index.clone(),
        )
        .with_heading_to_font_size_multipliers(appearance.heading_font_size_multipliers().clone())
        .with_hyperlink_font_color(appearance.theme().accent().into_solid())
        .register_default_click_handlers(|url, ctx, _| {
            ctx.dispatch_typed_action(AISettingsPageAction::HyperlinkClick(url));
        });

        column.add_child(
            Container::new(voice_input_description.finish())
                .with_margin_top(styles::DESCRIPTION_NEGATIVE_MARGIN_OFFSET)
                .with_margin_bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
                .with_margin_right(styles::TOGGLE_WIDTH_MARGIN)
                .finish(),
        );

        if ai_settings.is_voice_input_enabled(app) {
            column.add_child(render_dropdown_item(
                appearance,
                &crate::t!("settings-ai-voice-key"),
                Some(&crate::t!("settings-ai-voice-key-hint")),
                None,
                LocalOnlyIconState::for_setting(
                    VoiceInputToggleKey::storage_key(),
                    VoiceInputToggleKey::sync_to_cloud(),
                    &mut view.local_only_icon_tooltip_states.borrow_mut(),
                    app,
                ),
                None,
                &view.voice_input_toggle_key_dropdown,
            ));
            column.add_child(render_filterable_dropdown_item(
                appearance,
                &crate::t!("settings-ai-speech-language"),
                Some(&crate::t!("settings-ai-speech-language-description")),
                None,
                LocalOnlyIconState::for_setting(
                    VoiceInputLanguage::storage_key(),
                    VoiceInputLanguage::sync_to_cloud(),
                    &mut view.local_only_icon_tooltip_states.borrow_mut(),
                    app,
                ),
                None,
                &view.voice_input_language_dropdown,
            ));
        }

        column.finish()
    }
}

impl SettingsWidget for VoiceWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "voice agent oz ai a.i. speech input natural language talk english spanish french german estonian finnish"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        cfg!(feature = "voice_input") && UserWorkspaces::as_ref(app).is_voice_enabled()
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(app);
        Flex::column()
            .with_child(render_separator(appearance))
            .with_child(
                build_sub_header(
                    appearance,
                    crate::t!("settings-ai-voice-section"),
                    Some(styles::header_font_color(is_any_ai_enabled, app)),
                )
                .with_padding_bottom(HEADER_PADDING)
                .finish(),
            )
            .with_child(self.render_voice_section(view, appearance, app))
            .finish()
    }
}
#[derive(Default)]
struct OtherAIWidget {
    use_agent_footer_toggle: SwitchStateHandle,
    show_conversation_history_toggle: SwitchStateHandle,
}

impl OtherAIWidget {
    fn create_thinking_display_mode_dropdown(
        ctx: &mut ViewContext<AISettingsPageView>,
    ) -> ViewHandle<Dropdown<AISettingsPageAction>> {
        let items: Vec<DropdownItem<AISettingsPageAction>> = ThinkingDisplayMode::iter()
            .map(|mode| {
                DropdownItem::new(
                    mode.display_name(),
                    AISettingsPageAction::SetThinkingDisplayMode(mode),
                )
            })
            .collect();

        ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_menu_max_height(AI_SETTINGS_DROPDOWN_MAX_HEIGHT, ctx);
            dropdown.add_items(items, ctx);
            dropdown
        })
    }

    fn create_default_prompt_submission_mode_dropdown(
        ctx: &mut ViewContext<AISettingsPageView>,
    ) -> ViewHandle<Dropdown<AISettingsPageAction>> {
        let items: Vec<DropdownItem<AISettingsPageAction>> = PromptSubmissionMode::iter()
            .map(|mode| {
                DropdownItem::new(
                    mode.display_name(),
                    AISettingsPageAction::SetPromptSubmissionMode(mode),
                )
            })
            .collect();

        ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_menu_max_height(AI_SETTINGS_DROPDOWN_MAX_HEIGHT, ctx);
            dropdown.add_items(items, ctx);
            dropdown
        })
    }

    fn create_lrc_submission_mode_dropdown(
        ctx: &mut ViewContext<AISettingsPageView>,
    ) -> ViewHandle<Dropdown<AISettingsPageAction>> {
        let items: Vec<DropdownItem<AISettingsPageAction>> =
            LongRunningCommandSubmissionMode::iter()
                .map(|mode| {
                    DropdownItem::new(
                        mode.display_name(),
                        AISettingsPageAction::SetLongRunningCommandSubmissionMode(mode),
                    )
                })
                .collect();

        ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_menu_max_height(AI_SETTINGS_DROPDOWN_MAX_HEIGHT, ctx);
            dropdown.add_items(items, ctx);
            dropdown
        })
    }

    fn create_orchestration_message_display_mode_dropdown(
        ctx: &mut ViewContext<AISettingsPageView>,
    ) -> ViewHandle<Dropdown<AISettingsPageAction>> {
        let items: Vec<DropdownItem<AISettingsPageAction>> =
            OrchestrationMessageDisplayMode::iter()
                .map(|mode| {
                    DropdownItem::new(
                        mode.display_name(),
                        AISettingsPageAction::SetOrchestrationMessageDisplayMode(mode),
                    )
                })
                .collect();

        ctx.add_typed_action_view(|ctx| {
            let mut dropdown = Dropdown::new(ctx);
            dropdown.set_top_bar_max_width(AI_SETTINGS_DROPDOWN_WIDTH);
            dropdown.set_menu_width(AI_SETTINGS_DROPDOWN_WIDTH, ctx);
            dropdown.set_menu_max_height(AI_SETTINGS_DROPDOWN_MAX_HEIGHT, ctx);
            dropdown.add_items(items, ctx);
            dropdown
        })
    }
}

impl SettingsWidget for OtherAIWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "other use agent footer toolbar layout chip chips rearrange re-arrange thinking expanded reasoning collapse never show orchestration messages child agents collapse expand hide conversation history"
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(app);
        let is_toggleable = is_any_ai_enabled;

        let mut column = Flex::column()
            .with_child(render_separator(appearance))
            .with_child(
                build_sub_header(
                    appearance,
                    crate::t!("settings-ai-other-section"),
                    Some(styles::header_font_color(is_any_ai_enabled, app)),
                )
                .with_padding_bottom(HEADER_PADDING)
                .finish(),
            );

        if FeatureFlag::AgentView.is_enabled() {
            let mut agent_view_column = Flex::column()
                .with_child(render_ai_setting_toggle::<
                    ShouldRenderUseAgentToolbarForUserCommands,
                >(
                    crate::t!("settings-ai-show-use-agent-footer"),
                    AISettingsPageAction::ToggleUseAgentToolbar,
                    *ai_settings.should_render_use_agent_footer_for_user_commands,
                    is_toggleable,
                    self.use_agent_footer_toggle.clone(),
                    &view.local_only_icon_tooltip_states,
                    app,
                ))
                .with_child(render_ai_setting_description(
                    crate::t!("settings-ai-use-agent-footer-description"),
                    is_toggleable,
                    app,
                ));

            if is_toggleable && FeatureFlag::AgentToolbarEditor.is_enabled() {
                agent_view_column.add_child(render_toolbar_layout_editor(
                    &view.agent_toolbar_inline_editor,
                    appearance,
                ));
            }

            column.add_child(agent_view_column.finish());
        }

        column.add_child(render_ai_setting_toggle::<ShowConversationHistory>(
            crate::t!("settings-ai-show-conversation-history"),
            AISettingsPageAction::ToggleShowConversationHistory,
            *ai_settings.show_conversation_history,
            is_toggleable,
            self.show_conversation_history_toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        ));

        column.add_child(render_dropdown_item(
            appearance,
            &crate::t!("settings-ai-thinking-display"),
            Some(&crate::t!("settings-ai-thinking-display-description")),
            None,
            LocalOnlyIconState::for_setting(
                ThinkingDisplayMode::storage_key(),
                ThinkingDisplayMode::sync_to_cloud(),
                &mut view.local_only_icon_tooltip_states.borrow_mut(),
                app,
            ),
            (!is_any_ai_enabled).then(|| appearance.theme().disabled_ui_text_color()),
            &view.thinking_display_mode_dropdown,
        ));

        column.add_child(render_dropdown_item(
            appearance,
            &crate::t!("settings-ai-orchestration-message-display"),
            Some(&crate::t!(
                "settings-ai-orchestration-message-display-description"
            )),
            None,
            LocalOnlyIconState::for_setting(
                OrchestrationMessageDisplayMode::storage_key(),
                OrchestrationMessageDisplayMode::sync_to_cloud(),
                &mut view.local_only_icon_tooltip_states.borrow_mut(),
                app,
            ),
            (!is_any_ai_enabled).then(|| appearance.theme().disabled_ui_text_color()),
            &view.orchestration_message_display_mode_dropdown,
        ));

        // TODO: OpenConversationLayoutPreference should not depend on local_fs, but it lives under the external editor settings
        // which does require local_fs. It was a mistake to put it there, but now we keep it there for backward compatibility.
        #[cfg(feature = "local_fs")]
        if FeatureFlag::InfiniShellNewSettingsModes.is_enabled() {
            use crate::util::file::external_editor::settings::OpenConversationLayoutPreference;

            column.add_child(render_dropdown_item(
                appearance,
                &crate::t!("settings-ai-conversation-layout-label"),
                None,
                None,
                LocalOnlyIconState::for_setting(
                    OpenConversationLayoutPreference::storage_key(),
                    OpenConversationLayoutPreference::sync_to_cloud(),
                    &mut view.local_only_icon_tooltip_states.borrow_mut(),
                    app,
                ),
                (!is_any_ai_enabled).then(|| appearance.theme().disabled_ui_text_color()),
                &view.conversation_layout_dropdown,
            ));
        }

        column.finish()
    }
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn cli_agent_settings_widget_id() -> &'static str {
    CLIAgentWidget::static_widget_id()
}
fn cli_agent_widgets() -> Vec<Box<dyn SettingsWidget<View = AISettingsPageView>>> {
    vec![
        Box::new(CLIAgentWidget::default()),
        Box::new(CLIAgentAutoToggleRichInputWidget::default()),
        Box::new(CLIAgentAutoOpenRichInputWidget::default()),
        Box::new(CLIAgentAutoDismissRichInputWidget::default()),
        Box::new(CLIAgentSubmitRichInputWidget::default()),
        Box::new(CLIAgentCommandsWidget),
        Box::new(CLIAgentToolbarLayoutWidget),
    ]
}

// ── Per-agent chip 布局常量 ──
const CHIP_HEIGHT: f32 = 28.;
const CHIP_HORIZONTAL_PADDING: f32 = 10.;
const CHIP_CORNER_RADIUS: f32 = 4.;
const CHIP_GAP: f32 = 8.;
const CHIP_WIDTH: f32 = 92.;
const CHIP_CHECK_ICON_SIZE: f32 = 12.;
const CHIP_CHECK_ICON_GAP: f32 = 4.;
const PER_AGENT_ACTIONS_WIDTH: f32 = CHIP_WIDTH * 3. + CHIP_GAP * 2.;
const AGENT_ICON_SIZE: f32 = 16.;
const AGENT_ICON_MARGIN_RIGHT: f32 = 8.;
const ROW_HORIZONTAL_PADDING: f32 = 8.;
const ROW_VERTICAL_PADDING: f32 = 6.;
const ROW_CORNER_RADIUS: f32 = 4.;
const ROW_GAP: f32 = 4.;
const PER_AGENT_SECTION_MARGIN_TOP: f32 = 16.;
const PER_AGENT_SECTION_MARGIN_BOTTOM: f32 = 8.;
type PerAgentChipKey = (CLIAgent, PerAgentDimension);

#[derive(Default)]
struct CLIAgentWidget {
    cli_agent_footer_toggle: SwitchStateHandle,
    /// Per-agent chip hover state, keyed by agent and visibility dimension.
    per_agent_chip_states: RefCell<HashMap<PerAgentChipKey, MouseStateHandle>>,
}

impl SettingsWidget for CLIAgentWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "third party cli coding agent claude codex gemini toolbar footer quick actions show"
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);

        // The Coding Agents section is always enabled, independent of the
        // global AI toggle, because these settings control third-party coding
        // agents (Claude Code, Codex, Gemini CLI) rather than Zap's own AI.
        let cli_agent_footer_toggle = render_ai_setting_toggle::<ShouldRenderCLIAgentToolbar>(
            crate::t!("settings-ai-show-coding-agent-toolbar"),
            AISettingsPageAction::ToggleCLIAgentToolbar,
            *ai_settings.should_render_cli_agent_footer,
            true,
            self.cli_agent_footer_toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        );

        let description_fragments = vec![
            FormattedTextFragment::plain_text(crate::t!(
                "settings-ai-cli-toolbar-description-prefix"
            )),
            FormattedTextFragment::inline_code("claude"),
            FormattedTextFragment::plain_text(crate::t!(
                "settings-ai-cli-toolbar-description-separator"
            )),
            FormattedTextFragment::inline_code("codex"),
            FormattedTextFragment::plain_text(crate::t!(
                "settings-ai-cli-toolbar-description-last-separator"
            )),
            FormattedTextFragment::inline_code("gemini"),
            FormattedTextFragment::plain_text(crate::t!(
                "settings-ai-cli-toolbar-description-suffix"
            )),
        ];

        let description = FormattedTextElement::new(
            FormattedText::new([FormattedTextLine::Line(description_fragments)]),
            appearance.ui_font_size(),
            appearance.ui_font_family(),
            appearance.monospace_font_family(),
            styles::description_font_color(true, app).into(),
            HighlightedHyperlink::default(),
        )
        .with_heading_to_font_size_multipliers(appearance.heading_font_size_multipliers().clone());

        let is_footer_enabled = *ai_settings.should_render_cli_agent_footer;

        Flex::column()
            .with_child(cli_agent_footer_toggle)
            .with_child(
                Container::new(description.finish())
                    .with_margin_top(styles::DESCRIPTION_NEGATIVE_MARGIN_OFFSET)
                    .with_margin_bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
                    .with_margin_right(styles::TOGGLE_WIDTH_MARGIN)
                    .finish(),
            )
            .with_child(self.render_per_agent_settings_section(
                ai_settings,
                is_footer_enabled,
                appearance,
                app,
            ))
            .finish()
    }
}

fn should_render_cli_agent_detail(app: &AppContext) -> bool {
    *AISettings::as_ref(app).should_render_cli_agent_footer
}

fn should_render_cli_agent_rich_input(app: &AppContext) -> bool {
    should_render_cli_agent_detail(app) && FeatureFlag::CLIAgentRichInput.is_enabled()
}

#[derive(Default)]
struct CLIAgentAutoToggleRichInputWidget {
    toggle: SwitchStateHandle,
    info_tooltip: MouseStateHandle,
}

impl SettingsWidget for CLIAgentAutoToggleRichInputWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "third party cli coding agent rich input auto show hide status plugin"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        should_render_cli_agent_rich_input(app)
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        use super::settings_page::AdditionalInfo;
        use crate::settings::AutoToggleRichInput;

        if !self.should_render(app) {
            return Empty::new().finish();
        }

        let label = render_body_item_label::<AISettingsPageAction>(
            crate::t!("settings-ai-auto-show-rich-input"),
            Some(styles::header_font_color(true, app)),
            Some(AdditionalInfo {
                mouse_state: self.info_tooltip.clone(),
                on_click_action: None,
                secondary_text: None,
                tooltip_override_text: Some(crate::t!("settings-ai-auto-show-rich-input-tooltip")),
            }),
            LocalOnlyIconState::for_setting(
                AutoToggleRichInput::storage_key(),
                AutoToggleRichInput::sync_to_cloud(),
                &mut view.local_only_icon_tooltip_states.borrow_mut(),
                app,
            ),
            ToggleState::Enabled,
            appearance,
        );

        build_toggle_element(
            label,
            render_ai_feature_switch(
                self.toggle.clone(),
                *AISettings::as_ref(app).auto_toggle_rich_input,
                true,
                AISettingsPageAction::ToggleAutoToggleRichInput,
                app,
            ),
            appearance,
            None,
        )
    }
}

#[derive(Default)]
struct CLIAgentAutoOpenRichInputWidget {
    toggle: SwitchStateHandle,
}

impl SettingsWidget for CLIAgentAutoOpenRichInputWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "third party cli coding agent rich input auto open session start"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        should_render_cli_agent_rich_input(app)
    }

    fn render(
        &self,
        view: &Self::View,
        _appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        use crate::settings::AutoOpenRichInputOnCLIAgentStart;

        if !self.should_render(app) {
            return Empty::new().finish();
        }

        render_ai_setting_toggle::<AutoOpenRichInputOnCLIAgentStart>(
            crate::t!("settings-ai-auto-open-rich-input"),
            AISettingsPageAction::ToggleAutoOpenRichInputOnCLIAgentStart,
            *AISettings::as_ref(app).auto_open_rich_input_on_cli_agent_start,
            true,
            self.toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        )
    }
}

#[derive(Default)]
struct CLIAgentAutoDismissRichInputWidget {
    toggle: SwitchStateHandle,
}

impl SettingsWidget for CLIAgentAutoDismissRichInputWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "third party cli coding agent rich input auto dismiss prompt submission"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        should_render_cli_agent_rich_input(app)
    }

    fn render(
        &self,
        view: &Self::View,
        _appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        use crate::settings::AutoDismissRichInputAfterSubmit;

        if !self.should_render(app) {
            return Empty::new().finish();
        }

        render_ai_setting_toggle::<AutoDismissRichInputAfterSubmit>(
            crate::t!("settings-ai-auto-dismiss-rich-input"),
            AISettingsPageAction::ToggleAutoDismissRichInputAfterSubmit,
            *AISettings::as_ref(app).auto_dismiss_rich_input_after_submit,
            true,
            self.toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        )
    }
}

#[derive(Default)]
struct CLIAgentSubmitRichInputWidget {
    toggle: SwitchStateHandle,
}

impl SettingsWidget for CLIAgentSubmitRichInputWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "third party cli coding agent rich input submit ctrl enter newline"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        should_render_cli_agent_rich_input(app)
    }

    fn render(
        &self,
        view: &Self::View,
        _appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        use crate::settings::SubmitRichInputOnCtrlEnter;

        if !self.should_render(app) {
            return Empty::new().finish();
        }

        render_ai_setting_toggle::<SubmitRichInputOnCtrlEnter>(
            crate::t!("settings-ai-submit-rich-input"),
            AISettingsPageAction::ToggleSubmitRichInputOnCtrlEnter,
            *AISettings::as_ref(app).submit_on_ctrl_enter,
            true,
            self.toggle.clone(),
            &view.local_only_icon_tooltip_states,
            app,
        )
    }
}

struct CLIAgentCommandsWidget;

impl SettingsWidget for CLIAgentCommandsWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "third party cli coding agent claude codex gemini toolbar commands regex patterns"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        should_render_cli_agent_detail(app)
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        if !self.should_render(app) {
            return Empty::new().finish();
        }

        let mut list_column = Flex::column();
        list_column.add_child(
            appearance
                .ui_builder()
                .paragraph(crate::t!("settings-ai-toolbar-commands-description"))
                .with_style(UiComponentStyles {
                    font_size: Some(CONTENT_FONT_SIZE),
                    ..Default::default()
                })
                .build()
                .finish(),
        );
        list_column.add_child(ChildView::new(&view.cli_agent_footer_command_editor).finish());

        let background = appearance.theme().surface_1();
        let font_color = appearance.theme().foreground();
        let items: Vec<_> = AISettings::as_ref(app)
            .cli_agent_footer_enabled_commands
            .value()
            .keys()
            .cloned()
            .collect();
        let len = items.len();
        for (rev_i, pattern) in items.iter().rev().enumerate() {
            let original_i = len - 1 - rev_i;
            let remove_action =
                AISettingsPageAction::RemoveCLIAgentToolbarEnabledCommand(pattern.clone());
            let mouse_state = view
                .cli_agent_footer_command_mouse_state_handles
                .get(original_i)
                .cloned()
                .unwrap_or_default();

            let remove_button = appearance
                .ui_builder()
                .close_button(16., mouse_state)
                .build()
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(remove_action.clone());
                })
                .finish();

            let label = appearance
                .ui_builder()
                .wrappable_text(pattern.clone(), true)
                .with_style(UiComponentStyles {
                    font_color: Some(font_color.into_solid()),
                    font_family_id: Some(appearance.monospace_font_family()),
                    font_size: Some(appearance.ui_font_size()),
                    ..Default::default()
                })
                .build()
                .finish();

            let mut right_side = Flex::row().with_cross_axis_alignment(CrossAxisAlignment::Center);
            if let Some(dropdown_handle) = view
                .cli_agent_footer_command_agent_dropdowns
                .get(original_i)
            {
                right_side.add_child(
                    Container::new(ChildView::new(dropdown_handle).finish())
                        .with_margin_right(8.)
                        .finish(),
                );
            }
            right_side.add_child(remove_button);

            let row = Container::new(
                Flex::row()
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_main_axis_size(MainAxisSize::Max)
                    .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .with_children([Shrinkable::new(1., label).finish(), right_side.finish()])
                    .finish(),
            )
            .with_background(background)
            .with_horizontal_padding(8.)
            .with_vertical_padding(4.)
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.)))
            .with_margin_bottom(4.)
            .finish();

            list_column.add_child(row);
        }

        let description = appearance
            .ui_builder()
            .paragraph(crate::t!("settings-ai-toolbar-commands-description"))
            .with_style(UiComponentStyles {
                font_size: Some(appearance.ui_font_size()),
                font_color: Some(styles::description_font_color(true, app).into()),
                margin: Some(
                    Coords::default()
                        .top(4.)
                        .bottom(styles::DESCRIPTION_MARGIN_BOTTOM)
                        .right(styles::TOGGLE_WIDTH_MARGIN),
                ),
                ..Default::default()
            })
            .build()
            .finish();

        Flex::column()
            .with_child(list_column.finish())
            .with_child(description)
            .finish()
    }
}

struct CLIAgentToolbarLayoutWidget;

impl SettingsWidget for CLIAgentToolbarLayoutWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "third party cli coding agent toolbar layout chip chips rearrange re-arrange"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        should_render_cli_agent_detail(app) && FeatureFlag::AgentToolbarEditor.is_enabled()
    }

    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        if !self.should_render(app) {
            return Empty::new().finish();
        }

        render_toolbar_layout_editor(&view.cli_agent_toolbar_inline_editor, appearance)
    }
}

impl CLIAgentWidget {
    fn render_per_agent_settings_section(
        &self,
        ai_settings: &AISettings,
        is_footer_enabled: bool,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let ui_font_family = appearance.ui_font_family();
        let install_model = CLIAgentInstallModel::as_ref(app);
        let installed_agents: Vec<CLIAgent> = enum_iterator::all::<CLIAgent>()
            .filter(|a| !matches!(a, CLIAgent::Unknown) && install_model.is_cli_agent_installed(*a))
            .collect();

        let mut column = Flex::column().with_child(
            Container::new(
                appearance
                    .ui_builder()
                    .span(crate::t!("settings-ai-per-agent-section"))
                    .with_style(UiComponentStyles {
                        font_size: Some(appearance.ui_font_body()),
                        font_color: Some(styles::header_font_color(true, app).into()),
                        ..Default::default()
                    })
                    .build()
                    .finish(),
            )
            .with_margin_top(PER_AGENT_SECTION_MARGIN_TOP)
            .with_margin_bottom(PER_AGENT_SECTION_MARGIN_BOTTOM)
            .finish(),
        );

        if !install_model.is_scan_complete() {
            column.add_child(render_ai_status_text(
                crate::t!("settings-ai-per-agent-scanning"),
                app,
            ));
            return column.finish();
        }

        if installed_agents.is_empty() {
            column.add_child(render_ai_status_text(
                crate::t!("settings-ai-per-agent-empty"),
                app,
            ));
            return column.finish();
        }

        for agent in installed_agents {
            let icon = agent
                .icon()
                .unwrap_or(crate::ui_components::icons::Icon::LayoutAlt01);
            let agent_name_text = appearance
                .ui_builder()
                .wrappable_text(agent.display_name().to_string(), true)
                .with_style(UiComponentStyles {
                    font_color: Some(theme.foreground().into_solid()),
                    font_family_id: Some(ui_font_family),
                    font_size: Some(appearance.ui_font_size()),
                    ..Default::default()
                })
                .build()
                .finish();

            let toolbar_chip = self.render_cli_agent_visibility_chip(
                crate::t!("settings-ai-per-agent-toolbar-col").to_string(),
                ai_settings.is_cli_agent_toolbar_enabled(agent),
                is_footer_enabled,
                agent,
                PerAgentDimension::Toolbar,
                appearance,
            );
            let tab_menu_chip = self.render_cli_agent_visibility_chip(
                crate::t!("settings-ai-per-agent-tab-menu-col").to_string(),
                ai_settings.is_cli_agent_tab_menu_enabled(agent),
                true,
                agent,
                PerAgentDimension::TabMenu,
                appearance,
            );
            let titlebar_chip = self.render_cli_agent_visibility_chip(
                crate::t!("settings-ai-per-agent-titlebar-col").to_string(),
                ai_settings.is_cli_agent_titlebar_enabled(agent),
                true,
                agent,
                PerAgentDimension::Titlebar,
                appearance,
            );
            let actions = ConstrainedBox::new(
                Flex::row()
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_spacing(CHIP_GAP)
                    .with_children([toolbar_chip, tab_menu_chip, titlebar_chip])
                    .finish(),
            )
            .with_width(PER_AGENT_ACTIONS_WIDTH)
            .finish();

            let row = Container::new(
                Flex::row()
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_main_axis_size(MainAxisSize::Max)
                    .with_children([
                        Container::new(
                            ConstrainedBox::new(icon.to_warpui_icon(theme.foreground()).finish())
                                .with_width(AGENT_ICON_SIZE)
                                .with_height(AGENT_ICON_SIZE)
                                .finish(),
                        )
                        .with_margin_right(AGENT_ICON_MARGIN_RIGHT)
                        .finish(),
                        Expanded::new(1., agent_name_text).finish(),
                        actions,
                    ])
                    .finish(),
            )
            .with_background(theme.surface_1())
            .with_horizontal_padding(ROW_HORIZONTAL_PADDING)
            .with_vertical_padding(ROW_VERTICAL_PADDING)
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(ROW_CORNER_RADIUS)))
            .with_margin_bottom(ROW_GAP)
            .finish();

            column.add_child(row);
        }

        column.finish()
    }

    fn render_cli_agent_visibility_chip(
        &self,
        label: String,
        is_enabled: bool,
        is_clickable: bool,
        agent: CLIAgent,
        dimension: PerAgentDimension,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let background = if is_enabled {
            internal_colors::accent_overlay_2(theme)
        } else {
            internal_colors::fg_overlay_1(theme)
        };
        let border_fill = if is_enabled {
            Fill::Solid(theme.accent().into_solid())
        } else {
            Fill::Solid(pathfinder_color::ColorU::transparent_black())
        };
        let text_color = if is_enabled && is_clickable {
            internal_colors::text_main(theme, theme.background().into_solid())
        } else {
            internal_colors::text_sub(theme, theme.background().into_solid())
        };
        let icon_color = warp_core::ui::theme::Fill::Solid(text_color);
        let ui_font_family = appearance.ui_font_family();
        let mouse = self
            .per_agent_chip_states
            .borrow_mut()
            .entry((agent, dimension))
            .or_insert_with(MouseStateHandle::default)
            .clone();

        let mut chip = Hoverable::new(mouse, move |_| {
            let mut content = Flex::row()
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_main_axis_alignment(MainAxisAlignment::Center);
            if is_enabled {
                content.add_child(
                    ConstrainedBox::new(Icon::Check.to_warpui_icon(icon_color).finish())
                        .with_width(CHIP_CHECK_ICON_SIZE)
                        .with_height(CHIP_CHECK_ICON_SIZE)
                        .finish(),
                );
                content.add_child(
                    ConstrainedBox::new(Empty::new().finish())
                        .with_width(CHIP_CHECK_ICON_GAP)
                        .finish(),
                );
            }
            content.add_child(
                FormattedTextElement::from_str(label.clone(), ui_font_family, 13.)
                    .with_color(text_color)
                    .with_weight(Weight::Normal)
                    .with_alignment(TextAlignment::Center)
                    .with_line_height_ratio(1.0)
                    .finish(),
            );

            let container = Container::new(content.finish())
                .with_horizontal_padding(CHIP_HORIZONTAL_PADDING)
                .with_corner_radius(CornerRadius::with_all(Radius::Pixels(CHIP_CORNER_RADIUS)))
                .with_background(background)
                .with_border(Border::all(1.).with_border_fill(border_fill));

            ConstrainedBox::new(container.finish())
                .with_width(CHIP_WIDTH)
                .with_height(CHIP_HEIGHT)
                .finish()
        });

        if is_clickable {
            chip = chip
                .with_cursor(Cursor::PointingHand)
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(AISettingsPageAction::ToggleCLIAgentPerAgent(
                        agent, dimension,
                    ));
                });
        }

        chip.finish()
    }
}

struct AwsBedrockWidget {
    aws_auth_refresh_command_editor: ViewHandle<EditorView>,
    aws_auth_refresh_profile_editor: ViewHandle<EditorView>,
    credentials_enabled_toggle: SwitchStateHandle,
    auto_login_toggle: SwitchStateHandle,
    refresh_credentials_button: ViewHandle<ActionButton>,
}

impl AwsBedrockWidget {
    fn new(ctx: &mut ViewContext<<Self as SettingsWidget>::View>) -> Self {
        let ai_settings = AISettings::as_ref(ctx);
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(ctx);

        let aws_auth_refresh_command = ai_settings.aws_bedrock_auth_refresh_command.value().clone();
        let aws_auth_refresh_profile = ai_settings.aws_bedrock_profile.value().clone();
        let is_usage_enabled = is_any_ai_enabled
            && UserWorkspaces::as_ref(ctx).is_aws_bedrock_credentials_enabled(ctx);

        let aws_auth_refresh_command_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::as_ref(ctx);
            let options = SingleLineEditorOptions {
                is_password: false,
                text: TextOptions {
                    font_size_override: Some(appearance.ui_font_size()),
                    font_family_override: Some(appearance.monospace_font_family()),
                    text_colors_override: Some(TextColors {
                        default_color: appearance.theme().active_ui_text_color(),
                        disabled_color: appearance.theme().disabled_ui_text_color(),
                        hint_color: appearance.theme().disabled_ui_text_color(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text(crate::t!("settings-ai-aws-login-placeholder"), ctx);
            editor.set_buffer_text(&aws_auth_refresh_command, ctx);
            editor
        });
        AISettingsPageView::update_editor_interaction_state(
            aws_auth_refresh_command_editor.clone(),
            is_usage_enabled,
            ctx,
        );
        ctx.subscribe_to_view(&aws_auth_refresh_command_editor, |_, editor, event, ctx| {
            if matches!(event, EditorEvent::Blurred | EditorEvent::Enter) {
                let buffer_text = editor.as_ref(ctx).buffer_text(ctx);
                let should_reset = buffer_text.trim().is_empty();
                let value = if should_reset {
                    "aws login".to_string()
                } else {
                    buffer_text
                };
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let _ = settings
                        .aws_bedrock_auth_refresh_command
                        .set_value(value, ctx);
                });
                if should_reset {
                    editor.update(ctx, |editor, ctx| {
                        editor.set_buffer_text("aws login", ctx);
                    });
                }
            }
        });

        let aws_auth_refresh_profile_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::as_ref(ctx);
            let options = SingleLineEditorOptions {
                is_password: false,
                text: TextOptions {
                    font_size_override: Some(appearance.ui_font_size()),
                    font_family_override: Some(appearance.monospace_font_family()),
                    text_colors_override: Some(TextColors {
                        default_color: appearance.theme().active_ui_text_color(),
                        disabled_color: appearance.theme().disabled_ui_text_color(),
                        hint_color: appearance.theme().disabled_ui_text_color(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text(crate::t!("settings-ai-default-placeholder"), ctx);
            editor.set_buffer_text(&aws_auth_refresh_profile, ctx);
            editor
        });
        AISettingsPageView::update_editor_interaction_state(
            aws_auth_refresh_profile_editor.clone(),
            is_usage_enabled,
            ctx,
        );
        ctx.subscribe_to_view(&aws_auth_refresh_profile_editor, |_, editor, event, ctx| {
            if matches!(event, EditorEvent::Blurred | EditorEvent::Enter) {
                let buffer_text = editor.as_ref(ctx).buffer_text(ctx);
                let should_reset = buffer_text.trim().is_empty();
                let value = if should_reset {
                    "default".to_string()
                } else {
                    buffer_text
                };
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    let _ = settings.aws_bedrock_profile.set_value(value, ctx);
                });
                if should_reset {
                    editor.update(ctx, |editor, ctx| {
                        editor.set_buffer_text("default", ctx);
                    });
                }
            }
        });

        let refresh_credentials_button = ctx.add_typed_action_view(|_| {
            ActionButton::new(crate::t!("settings-ai-refresh"), SecondaryTheme)
                .with_icon(Icon::RefreshCw04)
                .with_size(ButtonSize::Small)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(AISettingsPageAction::RefreshAwsBedrockCredentials);
                })
        });
        refresh_credentials_button.update(ctx, |button, ctx| {
            button.set_disabled(!is_usage_enabled, ctx);
        });

        // Keep enablement in sync with the Global AI toggle.
        let aws_auth_refresh_command_editor_clone = aws_auth_refresh_command_editor.clone();
        let aws_auth_refresh_profile_editor_clone = aws_auth_refresh_profile_editor.clone();
        let refresh_credentials_button_clone = refresh_credentials_button.clone();
        ctx.subscribe_to_model(&AISettings::handle(ctx), move |_, _, event, ctx| {
            if matches!(
                event,
                AISettingsChangedEvent::IsAnyAIEnabled { .. }
                    | AISettingsChangedEvent::AwsBedrockCredentialsEnabled { .. }
            ) {
                let is_any_ai_enabled = AISettings::as_ref(ctx).is_any_ai_enabled(ctx);
                let is_usage_enabled = is_any_ai_enabled
                    && UserWorkspaces::as_ref(ctx).is_aws_bedrock_credentials_enabled(ctx);

                AISettingsPageView::update_editor_interaction_state(
                    aws_auth_refresh_command_editor_clone.clone(),
                    is_usage_enabled,
                    ctx,
                );
                AISettingsPageView::update_editor_interaction_state(
                    aws_auth_refresh_profile_editor_clone.clone(),
                    is_usage_enabled,
                    ctx,
                );
                refresh_credentials_button_clone.update(ctx, |button, ctx| {
                    button.set_disabled(!is_usage_enabled, ctx);
                });

                ctx.notify();
            }
        });

        let aws_auth_refresh_command_editor_clone = aws_auth_refresh_command_editor.clone();
        let aws_auth_refresh_profile_editor_clone = aws_auth_refresh_profile_editor.clone();
        let refresh_credentials_button_clone = refresh_credentials_button.clone();
        ctx.subscribe_to_model(
            &UserWorkspaces::handle(ctx),
            move |_, workspace, event, ctx| {
                if let UserWorkspacesEvent::TeamsChanged = event {
                    let is_any_ai_enabled = AISettings::as_ref(ctx).is_any_ai_enabled(ctx);
                    let is_usage_enabled = is_any_ai_enabled
                        && workspace
                            .as_ref(ctx)
                            .is_aws_bedrock_credentials_enabled(ctx);

                    AISettingsPageView::update_editor_interaction_state(
                        aws_auth_refresh_command_editor_clone.clone(),
                        is_usage_enabled,
                        ctx,
                    );
                    AISettingsPageView::update_editor_interaction_state(
                        aws_auth_refresh_profile_editor_clone.clone(),
                        is_usage_enabled,
                        ctx,
                    );
                    refresh_credentials_button_clone.update(ctx, |button, ctx| {
                        button.set_disabled(!is_usage_enabled, ctx);
                    });

                    ctx.notify();
                }
            },
        );

        Self {
            aws_auth_refresh_command_editor,
            aws_auth_refresh_profile_editor,
            credentials_enabled_toggle: SwitchStateHandle::default(),
            auto_login_toggle: SwitchStateHandle::default(),
            refresh_credentials_button,
        }
    }

    fn render_aws_bedrock_section(
        &self,
        appearance: &Appearance,
        app: &AppContext,
        is_bedrock_available: bool,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);
        let user_workspaces = UserWorkspaces::as_ref(app);
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(app);
        let is_section_enabled = is_any_ai_enabled && is_bedrock_available;
        let is_admin_enforced = matches!(
            user_workspaces.aws_bedrock_host_enablement_setting(),
            crate::workspaces::workspace::HostEnablementSetting::Enforce
        );
        let is_toggleable =
            is_section_enabled && user_workspaces.is_aws_bedrock_credentials_toggleable();
        let are_credentials_enabled = user_workspaces.is_aws_bedrock_credentials_enabled(app);
        let is_usage_enabled = is_section_enabled && are_credentials_enabled;
        let toggle_description = if is_admin_enforced {
            crate::t!("settings-ai-aws-bedrock-description-managed")
        } else {
            crate::t!("settings-ai-aws-bedrock-description")
        };

        let mut column = Flex::column().with_spacing(16.).with_child(
            Flex::column()
                .with_child(render_ai_setting_toggle::<AwsBedrockCredentialsEnabled>(
                    crate::t!("settings-ai-aws-bedrock-toggle"),
                    AISettingsPageAction::ToggleAwsBedrockCredentialsEnabled,
                    are_credentials_enabled,
                    is_toggleable,
                    self.credentials_enabled_toggle.clone(),
                    &RefCell::new(HashMap::new()),
                    app,
                ))
                .with_child(render_ai_setting_description(
                    toggle_description,
                    is_section_enabled,
                    app,
                ))
                .finish(),
        );

        /// Helper function to render the UI for an input field.
        fn render_input(
            appearance: &Appearance,
            label: &'static str,
            editor: ViewHandle<EditorView>,
            is_enabled: bool,
            app: &AppContext,
        ) -> Box<dyn Element> {
            let ui_font_size = appearance.ui_font_size();
            let padding = Some(Coords {
                top: ui_font_size * 5. / 6.,
                bottom: ui_font_size * 5. / 6.,
                left: ui_font_size * 4. / 3.,
                right: ui_font_size * 4. / 3.,
            });
            let editor_style = UiComponentStyles {
                padding,
                background: Some(appearance.theme().surface_2().into()),
                ..Default::default()
            };

            let label = Text::new_inline(
                label,
                appearance.ui_font_family(),
                appearance.ui_font_body(),
            )
            .with_color(styles::header_font_color(is_enabled, app).into())
            .finish();

            let input = appearance
                .ui_builder()
                .text_input(editor)
                .with_style(editor_style)
                .build()
                .finish();

            Flex::column()
                .with_spacing(8.)
                .with_child(label)
                .with_child(input)
                .finish()
        }

        fn render_credential_status_card(
            refresh_button: &ViewHandle<ActionButton>,
            appearance: &Appearance,
            are_credentials_enabled: bool,
            app: &AppContext,
        ) -> Box<dyn Element> {
            let (title_color, detail_color) = (
                styles::header_font_color(are_credentials_enabled, app),
                styles::description_font_color(are_credentials_enabled, app),
            );
            let (title_text, detail_text, icon) = ApiKeyManager::as_ref(app)
                .aws_credentials_state()
                .user_facing_components();

            let icon = Container::new(
                ConstrainedBox::new(icon.to_warpui_icon(title_color).finish())
                    .with_width(16.)
                    .with_height(16.)
                    .finish(),
            )
            .with_horizontal_padding(4.)
            .finish();

            let text_column = Flex::column()
                .with_cross_axis_alignment(CrossAxisAlignment::Start)
                .with_spacing(4.)
                .with_child(
                    Text::new_inline(
                        title_text,
                        appearance.ui_font_family(),
                        appearance.ui_font_body(),
                    )
                    .with_style(Properties::default().weight(Weight::Semibold))
                    .with_color(title_color.into())
                    .finish(),
                )
                .with_child(
                    Text::new(
                        detail_text,
                        appearance.ui_font_family(),
                        appearance.ui_font_body(),
                    )
                    .with_color(detail_color.into())
                    .soft_wrap(true)
                    .finish(),
                );

            Container::new(
                Flex::row()
                    .with_main_axis_size(MainAxisSize::Max)
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_spacing(12.)
                    .with_child(
                        Expanded::new(
                            1.,
                            Flex::row()
                                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                                .with_spacing(12.)
                                .with_child(icon)
                                .with_child(Expanded::new(1., text_column.finish()).finish())
                                .finish(),
                        )
                        .finish(),
                    )
                    .with_child(ChildView::new(refresh_button).finish())
                    .finish(),
            )
            .with_uniform_padding(12.)
            .with_background(appearance.theme().surface_2())
            .with_border(Border::all(1.).with_border_fill(appearance.theme().outline()))
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(6.)))
            .finish()
        }

        column.add_child(
            Container::new(render_credential_status_card(
                &self.refresh_credentials_button,
                appearance,
                are_credentials_enabled,
                app,
            ))
            .with_margin_top(-styles::DESCRIPTION_MARGIN_BOTTOM)
            .finish(),
        );
        column.add_child(render_input(
            appearance,
            Box::leak(crate::t!("settings-ai-aws-login-command").into_boxed_str()),
            self.aws_auth_refresh_command_editor.clone(),
            is_usage_enabled,
            app,
        ));
        column.add_child(render_input(
            appearance,
            Box::leak(crate::t!("settings-ai-aws-profile").into_boxed_str()),
            self.aws_auth_refresh_profile_editor.clone(),
            is_usage_enabled,
            app,
        ));

        let auto_login_enabled = *AISettings::as_ref(app).aws_bedrock_auto_login.value();

        let toggle = render_ai_setting_toggle::<AwsBedrockAutoLogin>(
            crate::t!("settings-ai-aws-auto-login"),
            AISettingsPageAction::ToggleAwsBedrockAutoLogin,
            auto_login_enabled,
            is_usage_enabled,
            self.auto_login_toggle.clone(),
            &RefCell::new(HashMap::new()),
            app,
        );
        let description = render_ai_setting_description(
            crate::t!("settings-ai-aws-auto-login-description"),
            is_usage_enabled,
            app,
        );
        column.add_child(
            Flex::column()
                .with_child(toggle)
                .with_child(description)
                .finish(),
        );

        column.finish()
    }
}

impl SettingsWidget for AwsBedrockWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "aws bedrock amazon credentials login profile"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        // Only show if admin has enabled AWS Bedrock for the workspace
        UserWorkspaces::as_ref(app).is_aws_bedrock_available_from_workspace()
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let ai_settings = AISettings::as_ref(app);
        let is_any_ai_enabled = ai_settings.is_any_ai_enabled(app);
        let is_bedrock_available =
            UserWorkspaces::as_ref(app).is_aws_bedrock_available_from_workspace();

        let column = Flex::column()
            .with_child(render_separator(appearance))
            .with_child(
                build_sub_header(
                    appearance,
                    crate::t!("settings-ai-aws-bedrock-section"),
                    Some(styles::header_font_color(is_any_ai_enabled, app)),
                )
                .with_padding_bottom(HEADER_PADDING)
                .finish(),
            )
            .with_child(self.render_aws_bedrock_section(appearance, app, is_bedrock_available));

        Container::new(column.finish())
            .with_margin_bottom(HEADER_PADDING)
            .finish()
    }
}

struct GeminiEnterpriseWidget {
    credentials_enabled_toggle: SwitchStateHandle,
    refresh_credentials_button: ViewHandle<ActionButton>,
}

impl GeminiEnterpriseWidget {
    fn is_refresh_enabled(app: &AppContext) -> bool {
        AISettings::as_ref(app).is_any_ai_enabled(app)
            && UserWorkspaces::as_ref(app).is_gemini_enterprise_credentials_enabled(app)
            && !ApiKeyManager::as_ref(app)
                .geap_credentials_state()
                .requires_admin_action()
    }

    fn new(ctx: &mut ViewContext<<Self as SettingsWidget>::View>) -> Self {
        let refresh_credentials_button = ctx.add_typed_action_view(|_| {
            ActionButton::new(crate::t!("common-refresh"), SecondaryTheme)
                .with_icon(Icon::RefreshCw04)
                .with_size(ButtonSize::Small)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(
                        AISettingsPageAction::RefreshGeminiEnterpriseCredentials,
                    );
                })
        });
        refresh_credentials_button.update(ctx, |button, ctx| {
            button.set_disabled(!Self::is_refresh_enabled(ctx), ctx);
        });

        let refresh_credentials_button_clone = refresh_credentials_button.clone();
        ctx.subscribe_to_model(&UserWorkspaces::handle(ctx), move |_, _, event, ctx| {
            if matches!(
                event,
                UserWorkspacesEvent::TeamsChanged
                    | UserWorkspacesEvent::UpdateWorkspaceSettingsSuccess
            ) {
                refresh_credentials_button_clone.update(ctx, |button, ctx| {
                    button.set_disabled(!Self::is_refresh_enabled(ctx), ctx);
                });
                ctx.notify();
            }
        });

        let refresh_credentials_button_clone = refresh_credentials_button.clone();
        ctx.subscribe_to_model(&AISettings::handle(ctx), move |_, _, event, ctx| {
            if matches!(
                event,
                AISettingsChangedEvent::GeminiEnterpriseCredentialsEnabled { .. }
                    | AISettingsChangedEvent::IsAnyAIEnabled { .. }
            ) {
                refresh_credentials_button_clone.update(ctx, |button, ctx| {
                    button.set_disabled(!Self::is_refresh_enabled(ctx), ctx);
                });
                ctx.notify();
            }
        });

        let refresh_credentials_button_clone = refresh_credentials_button.clone();
        ctx.subscribe_to_model(&ApiKeyManager::handle(ctx), move |_, _, event, ctx| {
            if matches!(event, ApiKeyManagerEvent::KeysUpdated) {
                refresh_credentials_button_clone.update(ctx, |button, ctx| {
                    button.set_disabled(!Self::is_refresh_enabled(ctx), ctx);
                });
            }
        });

        Self {
            credentials_enabled_toggle: SwitchStateHandle::default(),
            refresh_credentials_button,
        }
    }

    fn render_gemini_enterprise_section(
        &self,
        appearance: &Appearance,
        app: &AppContext,
        is_gemini_enterprise_available: bool,
    ) -> Box<dyn Element> {
        let user_workspaces = UserWorkspaces::as_ref(app);
        let is_any_ai_enabled = AISettings::as_ref(app).is_any_ai_enabled(app);
        let is_section_enabled = is_any_ai_enabled && is_gemini_enterprise_available;
        let is_admin_enforced = matches!(
            user_workspaces.gemini_enterprise_host_enablement_setting(),
            crate::workspaces::workspace::HostEnablementSetting::Enforce
        );
        let is_toggleable =
            is_section_enabled && user_workspaces.is_gemini_enterprise_credentials_toggleable();
        let are_credentials_enabled = user_workspaces.is_gemini_enterprise_credentials_enabled(app);
        let toggle_description = if is_admin_enforced {
            crate::t!("settings-ai-gemini-enterprise-description-managed")
        } else {
            crate::t!("settings-ai-gemini-enterprise-description")
        };

        let mut column = Flex::column().with_spacing(16.).with_child(
            Flex::column()
                .with_child(
                    render_ai_setting_toggle::<GeminiEnterpriseCredentialsEnabled>(
                        crate::t!("settings-ai-gemini-enterprise-toggle"),
                        AISettingsPageAction::ToggleGeminiEnterpriseCredentialsEnabled,
                        are_credentials_enabled,
                        is_toggleable,
                        self.credentials_enabled_toggle.clone(),
                        &RefCell::new(HashMap::new()),
                        app,
                    ),
                )
                .with_child(render_ai_setting_description(
                    toggle_description,
                    is_section_enabled,
                    app,
                ))
                .finish(),
        );

        column.add_child(
            Container::new(self.render_credential_status_card(
                appearance,
                are_credentials_enabled,
                app,
            ))
            .with_margin_top(-styles::DESCRIPTION_MARGIN_BOTTOM)
            .finish(),
        );

        column.finish()
    }

    fn render_credential_status_card(
        &self,
        appearance: &Appearance,
        are_credentials_enabled: bool,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let manager = ApiKeyManager::as_ref(app);
        let (title_text, detail_text, icon) =
            manager.geap_credentials_state().user_facing_components();

        let (title_color, detail_color) = (
            styles::header_font_color(are_credentials_enabled, app),
            styles::description_font_color(are_credentials_enabled, app),
        );

        let icon = Container::new(
            ConstrainedBox::new(icon.to_warpui_icon(title_color).finish())
                .with_width(16.)
                .with_height(16.)
                .finish(),
        )
        .with_horizontal_padding(4.)
        .finish();

        let text_column = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Start)
            .with_spacing(4.)
            .with_child(
                Text::new_inline(title_text, appearance.ui_font_family(), CONTENT_FONT_SIZE)
                    .with_style(Properties::default().weight(Weight::Semibold))
                    .with_color(title_color.into())
                    .finish(),
            )
            .with_child(
                Text::new(detail_text, appearance.ui_font_family(), CONTENT_FONT_SIZE)
                    .with_color(detail_color.into())
                    .soft_wrap(true)
                    .finish(),
            );

        let row = Flex::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_spacing(12.)
            .with_child(
                Expanded::new(
                    1.,
                    Flex::row()
                        .with_cross_axis_alignment(CrossAxisAlignment::Center)
                        .with_spacing(12.)
                        .with_child(icon)
                        .with_child(Expanded::new(1., text_column.finish()).finish())
                        .finish(),
                )
                .finish(),
            )
            .with_child(ChildView::new(&self.refresh_credentials_button).finish());

        Container::new(row.finish())
            .with_uniform_padding(12.)
            .with_background(appearance.theme().surface_2())
            .with_border(Border::all(1.).with_border_fill(appearance.theme().outline()))
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(6.)))
            .finish()
    }
}

impl SettingsWidget for GeminiEnterpriseWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "gemini enterprise geap google vertex credentials"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        FeatureFlag::GeminiEnterprise.is_enabled()
            && UserWorkspaces::as_ref(app).is_gemini_enterprise_available_from_workspace()
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let is_any_ai_enabled = AISettings::as_ref(app).is_any_ai_enabled(app);
        let is_gemini_enterprise_available =
            UserWorkspaces::as_ref(app).is_gemini_enterprise_available_from_workspace();
        let column = Flex::column()
            .with_child(render_separator(appearance))
            .with_child(
                build_sub_header(
                    appearance,
                    crate::t!("settings-ai-gemini-enterprise-section"),
                    Some(styles::header_font_color(is_any_ai_enabled, app)),
                )
                .with_padding_bottom(HEADER_PADDING)
                .finish(),
            )
            .with_child(self.render_gemini_enterprise_section(
                appearance,
                app,
                is_gemini_enterprise_available,
            ));

        Container::new(column.finish())
            .with_margin_bottom(HEADER_PADDING)
            .finish()
    }
}

/// Stable `&'static str` id for the custom model routers settings widget,
/// exposed for the `warp://settings?widget=custom_router` deeplink (see
/// `settings_widget_deeplink_target`).
pub(crate) fn custom_model_routers_widget_id() -> &'static str {
    CustomModelRoutersWidget::static_widget_id()
}

#[derive(Default)]
struct CustomModelRoutersWidget;

impl SettingsWidget for CustomModelRoutersWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "custom model router complexity prompt auto model routing"
    }

    fn should_render(&self, _app: &AppContext) -> bool {
        FeatureFlag::CustomModelRouters.is_enabled()
    }

    #[cfg_attr(not(feature = "local_fs"), allow(unused_variables))]
    fn render(
        &self,
        view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let is_any_ai_enabled = AISettings::as_ref(app).is_any_ai_enabled(app);
        let header_color = styles::header_font_color(is_any_ai_enabled, app);

        // Header row: "Custom Model Routers" + add button
        let header_row = Flex::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                build_sub_header(
                    appearance,
                    crate::t!("settings-ai-custom-routers-section"),
                    Some(header_color),
                )
                .finish(),
            )
            .with_child({
                #[cfg(feature = "local_fs")]
                {
                    warpui::elements::Container::new(view.add_router_button.as_ref(app).render(app))
                        .with_margin_bottom(4.)
                        .with_margin_top(-4.)
                        .finish()
                }
                #[cfg(not(feature = "local_fs"))]
                {
                    warpui::elements::Empty::new().finish()
                }
            })
            .finish();

        let column = Flex::column()
            .with_child(render_separator(appearance))
            .with_child(
                Container::new(header_row)
                    .with_padding_bottom(HEADER_PADDING)
                    .finish(),
            )
            .with_child(render_ai_setting_description(
                crate::t!("settings-ai-custom-routers-description"),
                is_any_ai_enabled,
                app,
            ));

        // Error cards and router summary cards (local_fs only)
        #[cfg(feature = "local_fs")]
        let column = {
            use super::custom_router_view::render_router_error_card;
            use crate::user_config::WarpConfig;
            let mut c = column;
            // Error cards (files that failed to parse) — shown first
            let errors = WarpConfig::as_ref(app).custom_model_router_errors();
            for error in errors.iter() {
                c.add_child(
                    Container::new(render_router_error_card(
                        &error.file_name,
                        &error.error_message,
                        appearance,
                    ))
                    .with_margin_top(8.)
                    .finish(),
                );
            }
            // Router summary cards
            for view_handle in &view.router_views {
                c.add_child(
                    Container::new(warpui::elements::ChildView::new(view_handle).finish())
                        .with_margin_top(8.)
                        .finish(),
                );
            }
            c
        };

        // Add trailing space beneath this section (matching sibling sections
        // like AWS Bedrock) so the following section's title isn't crowded
        // against the router cards.
        Container::new(column.finish())
            .with_margin_bottom(HEADER_PADDING)
            .finish()
    }
}

mod styles {
    use warp_core::ui::appearance::Appearance;
    use warp_core::ui::theme::Fill;
    use warpui::{AppContext, SingletonEntity};

    // Apply a negative margin to the description text so it appears closer to the main
    // settings option text.
    pub const DESCRIPTION_NEGATIVE_MARGIN_OFFSET: f32 = -12.;

    /// The space between a description and the next toggle.
    pub const DESCRIPTION_MARGIN_BOTTOM: f32 = 12.;

    /// Margin to leave for switch toggle to the right of the description subtext.
    pub const TOGGLE_WIDTH_MARGIN: f32 = 48.;

    pub fn header_font_color(is_enabled_setting: bool, app: &AppContext) -> Fill {
        let appearance = Appearance::as_ref(app);
        if is_enabled_setting {
            appearance
                .theme()
                .main_text_color(appearance.theme().surface_2())
        } else {
            appearance.theme().disabled_ui_text_color()
        }
    }

    pub fn description_font_color(is_enabled_setting: bool, app: &AppContext) -> Fill {
        let appearance = Appearance::as_ref(app);
        if is_enabled_setting {
            appearance
                .theme()
                .sub_text_color(appearance.theme().surface_1())
        } else {
            appearance.theme().disabled_ui_text_color()
        }
    }
}
