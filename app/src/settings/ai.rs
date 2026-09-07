//! Settings for Blocklist AI.
//!
//! These settings are currently used to configure the underlying model/API used to power the AI
//! UX, as well as small UX configurations.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
pub use cloud_object_models::{
    AgentModeCommandExecutionPredicate, DEFAULT_COMMAND_EXECUTION_ALLOWLIST,
    DEFAULT_COMMAND_EXECUTION_DENYLIST,
};
use indexmap::IndexMap;
use regex::Regex;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use settings::{
    RespectUserSyncSetting, Setting, SupportedPlatforms, SyncToCloud, define_settings_group,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use warp_core::execution_mode::AppExecutionMode;
use warp_core::features::FeatureFlag;
use warp_errors::report_if_error;
use warpui::platform::OperatingSystem;
use warpui::platform::keyboard::KeyCode;
use warpui::{AppContext, Entity, ModelContext, SingletonEntity, UpdateModel};

use crate::ai::execution_profiles::ExecutionProfilesConfig;
use crate::ai::request_usage_model::RequestLimitInfo;
use crate::terminal::CLIAgent;
use crate::workspaces::user_workspaces::UserWorkspaces;

pub enum FocusedTerminalInfoEvent {
    TerminalInfoUpdated,
}

/// Singleton model that is used to track the remote sessions in the terminal.
/// Useful for organizations that have restrictions on using AI in sessions in
/// remote sessions.
#[derive(Default, Clone, Debug)]
pub struct FocusedTerminalInfo {
    contains_any_remote_blocks: bool,
    contains_any_restored_remote_blocks: bool,
}

impl FocusedTerminalInfo {
    pub fn new(_: &mut ModelContext<Self>) -> Self {
        Self {
            contains_any_remote_blocks: false,
            contains_any_restored_remote_blocks: false,
        }
    }

    pub fn contains_any_remote_blocks(&self) -> bool {
        self.contains_any_remote_blocks
    }

    pub fn contains_any_restored_remote_blocks(&self) -> bool {
        self.contains_any_restored_remote_blocks
    }

    /// Updates both remote blocks and restored blocks status in a single atomic operation.
    /// Only emits a TerminalInfoUpdated event if either value changes.
    /// Returns true if the event was emitted.
    pub fn update(
        &mut self,
        contains_any_remote_blocks: bool,
        contains_any_restored_remote_blocks: bool,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        let remote_changed = self.contains_any_remote_blocks != contains_any_remote_blocks;
        let restored_changed =
            self.contains_any_restored_remote_blocks != contains_any_restored_remote_blocks;

        if remote_changed || restored_changed {
            self.contains_any_remote_blocks = contains_any_remote_blocks;
            self.contains_any_restored_remote_blocks = contains_any_restored_remote_blocks;
            ctx.emit(FocusedTerminalInfoEvent::TerminalInfoUpdated);
            return true;
        }

        false
    }
}

impl Entity for FocusedTerminalInfo {
    type Event = FocusedTerminalInfoEvent;
}

impl SingletonEntity for FocusedTerminalInfo {}

#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Physical key used to toggle voice input.",
    rename_all = "snake_case"
)]
pub enum VoiceInputToggleKey {
    #[default]
    #[schemars(description = "No toggle key assigned.")]
    None,
    /// Fn key is default toggle key for Mac, when the feature is toggled on.
    #[schemars(description = "Fn key.")]
    Fn,
    /// Alt or Option key (left side).
    #[schemars(description = "Alt or Option key (left side).")]
    AltLeft,
    /// Alt or Option key (right side). Used as default toggle
    /// key for Windows and Linux, , when the feature is toggled on.
    #[schemars(description = "Alt or Option key (right side).")]
    AltRight,
    #[schemars(description = "Control key (left side).")]
    ControlLeft,
    #[schemars(description = "Control key (right side).")]
    ControlRight,
    /// The Windows, ⌘, Command, or other OS symbol key.
    #[schemars(description = "Super, Windows, or Command key (left side).")]
    SuperLeft,
    /// The Windows, ⌘, Command, or other OS symbol key.
    #[schemars(description = "Super, Windows, or Command key (right side).")]
    SuperRight,
    #[schemars(description = "Shift key (left side).")]
    ShiftLeft,
    #[schemars(description = "Shift key (right side).")]
    ShiftRight,
}

settings::macros::implement_setting_for_enum!(
    VoiceInputToggleKey,
    AISettings,
    SupportedPlatforms::DESKTOP,
    // Never sync to cloud to allow users to use different toggle keys on different devices,
    // especially given platform differences.
    SyncToCloud::Never,
    surface: settings::SettingSurfaces::GUI,
    private: false,
    toml_path: "agents.voice.voice_input_toggle_key",
    description: "The key used to toggle voice input.",
);

impl VoiceInputToggleKey {
    pub fn all_possible_values() -> Vec<VoiceInputToggleKey> {
        let all_keys = VoiceInputToggleKey::iter().collect();
        match OperatingSystem::get() {
            OperatingSystem::Mac => all_keys,
            // For non-Mac platforms, we exclude the `Fn` key since it may not be correctly identified by winit.
            // In particular, we saw it is unidentified for a ThinkPad with a standard keyboard.
            OperatingSystem::Windows | OperatingSystem::Linux | OperatingSystem::Other(_) => {
                all_keys
                    .into_iter()
                    .filter(|key| *key != VoiceInputToggleKey::Fn)
                    .collect()
            }
        }
    }

    /// Display name for choosing key from the AI settings page.
    pub fn display_name(&self) -> String {
        // We use the underlying host OS to determine the correct key name to display.
        let (super_key_name, alt_key_name): (&'static str, &'static str) =
            match OperatingSystem::get() {
                OperatingSystem::Mac => ("Command", "Option"),
                OperatingSystem::Windows => ("Windows", "Alt"),
                OperatingSystem::Linux | OperatingSystem::Other(_) => ("Super", "Alt"),
            };

        match self {
            VoiceInputToggleKey::None => crate::t!("common-none"),
            VoiceInputToggleKey::Fn => "Fn".to_string(),
            VoiceInputToggleKey::AltLeft => {
                crate::t!("settings-key-side-left", key = alt_key_name)
            }
            VoiceInputToggleKey::AltRight => {
                crate::t!("settings-key-side-right", key = alt_key_name)
            }
            VoiceInputToggleKey::ControlLeft => {
                crate::t!("settings-key-side-left", key = "Control")
            }
            VoiceInputToggleKey::ControlRight => {
                crate::t!("settings-key-side-right", key = "Control")
            }
            VoiceInputToggleKey::SuperLeft => {
                crate::t!("settings-key-side-left", key = super_key_name)
            }
            VoiceInputToggleKey::SuperRight => {
                crate::t!("settings-key-side-right", key = super_key_name)
            }
            VoiceInputToggleKey::ShiftLeft => {
                crate::t!("settings-key-side-left", key = "Shift")
            }
            VoiceInputToggleKey::ShiftRight => {
                crate::t!("settings-key-side-right", key = "Shift")
            }
        }
    }

    pub fn to_key_code(&self) -> Option<KeyCode> {
        match self {
            VoiceInputToggleKey::None => None,
            VoiceInputToggleKey::Fn => Some(KeyCode::Fn),
            VoiceInputToggleKey::AltLeft => Some(KeyCode::AltLeft),
            VoiceInputToggleKey::AltRight => Some(KeyCode::AltRight),
            VoiceInputToggleKey::ControlLeft => Some(KeyCode::ControlLeft),
            VoiceInputToggleKey::ControlRight => Some(KeyCode::ControlRight),
            VoiceInputToggleKey::SuperLeft => Some(KeyCode::SuperLeft),
            VoiceInputToggleKey::SuperRight => Some(KeyCode::SuperRight),
            VoiceInputToggleKey::ShiftLeft => Some(KeyCode::ShiftLeft),
            VoiceInputToggleKey::ShiftRight => Some(KeyCode::ShiftRight),
        }
    }

    /// Converts the voice input toggle key to a Keystroke representation.
    /// Since these are standalone modifier keys, we construct the Keystroke directly
    /// rather than using `parse()` (which always requires a non-modifier key to be included).
    pub fn keystroke(&self) -> Option<warpui::keymap::Keystroke> {
        use warpui::keymap::Keystroke;

        let keystroke = match self {
            VoiceInputToggleKey::None => return None,
            VoiceInputToggleKey::Fn => Keystroke {
                key: "fn".to_string(),
                ..Default::default()
            },
            VoiceInputToggleKey::AltLeft | VoiceInputToggleKey::AltRight => Keystroke {
                alt: true,
                ..Default::default()
            },
            VoiceInputToggleKey::ControlLeft | VoiceInputToggleKey::ControlRight => Keystroke {
                ctrl: true,
                ..Default::default()
            },
            VoiceInputToggleKey::SuperLeft | VoiceInputToggleKey::SuperRight => Keystroke {
                cmd: true,
                ..Default::default()
            },
            VoiceInputToggleKey::ShiftLeft | VoiceInputToggleKey::ShiftRight => Keystroke {
                shift: true,
                ..Default::default()
            },
        };
        Some(keystroke)
    }

    pub fn tooltip_message(&self) -> String {
        match self.keystroke() {
            Some(keystroke) => {
                let symbol = keystroke.displayed();
                let key_name = match self {
                    VoiceInputToggleKey::AltLeft
                    | VoiceInputToggleKey::ControlLeft
                    | VoiceInputToggleKey::SuperLeft
                    | VoiceInputToggleKey::ShiftLeft => {
                        crate::t!("settings-key-side-left", key = symbol.as_str())
                    }
                    VoiceInputToggleKey::AltRight
                    | VoiceInputToggleKey::ControlRight
                    | VoiceInputToggleKey::SuperRight
                    | VoiceInputToggleKey::ShiftRight => {
                        crate::t!("settings-key-side-right", key = symbol.as_str())
                    }
                    VoiceInputToggleKey::None | VoiceInputToggleKey::Fn => symbol,
                };
                crate::t!("settings-voice-input-hold-key", key = key_name.as_str())
            }
            None => crate::t!("settings-ai-voice-input-label"),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, VoiceInputToggleKey::None)
    }
}

/// The full ISO-639-1 language catalog offered in the voice-input Speech
/// Language picker, as `(code, display_name)` pairs. The empty code is the
/// `Auto-detect` sentinel: when it is the stored value, no language is forced
/// and the transcription provider (Wispr Flow) auto-detects the language.
/// Kept as a data table (rather than an enum) so the entire list stays easy to
/// scan and extend; the picker is filterable, so the length is not a problem.
pub const VOICE_INPUT_LANGUAGES: &[(&str, &str)] = &[
    ("", "Auto-detect"),
    ("ab", "Abkhaz"),
    ("aa", "Afar"),
    ("af", "Afrikaans"),
    ("ak", "Akan"),
    ("sq", "Albanian"),
    ("am", "Amharic"),
    ("ar", "Arabic"),
    ("an", "Aragonese"),
    ("hy", "Armenian"),
    ("as", "Assamese"),
    ("av", "Avaric"),
    ("ae", "Avestan"),
    ("ay", "Aymara"),
    ("az", "Azerbaijani"),
    ("bm", "Bambara"),
    ("ba", "Bashkir"),
    ("eu", "Basque"),
    ("be", "Belarusian"),
    ("bn", "Bengali"),
    ("bh", "Bihari"),
    ("bi", "Bislama"),
    ("bs", "Bosnian"),
    ("br", "Breton"),
    ("bg", "Bulgarian"),
    ("my", "Burmese"),
    ("ca", "Catalan"),
    ("ch", "Chamorro"),
    ("ce", "Chechen"),
    ("ny", "Chichewa"),
    ("zh", "Chinese"),
    ("cv", "Chuvash"),
    ("kw", "Cornish"),
    ("co", "Corsican"),
    ("cr", "Cree"),
    ("hr", "Croatian"),
    ("cs", "Czech"),
    ("da", "Danish"),
    ("dv", "Divehi"),
    ("nl", "Dutch"),
    ("dz", "Dzongkha"),
    ("en", "English"),
    ("eo", "Esperanto"),
    ("et", "Estonian"),
    ("ee", "Ewe"),
    ("fo", "Faroese"),
    ("fj", "Fijian"),
    ("fi", "Finnish"),
    ("fr", "French"),
    ("ff", "Fulah"),
    ("gl", "Galician"),
    ("lg", "Ganda"),
    ("ka", "Georgian"),
    ("de", "German"),
    ("el", "Greek"),
    ("gn", "Guarani"),
    ("gu", "Gujarati"),
    ("ht", "Haitian Creole"),
    ("ha", "Hausa"),
    ("he", "Hebrew"),
    ("hz", "Herero"),
    ("hi", "Hindi"),
    ("ho", "Hiri Motu"),
    ("hu", "Hungarian"),
    ("is", "Icelandic"),
    ("io", "Ido"),
    ("ig", "Igbo"),
    ("id", "Indonesian"),
    ("ia", "Interlingua"),
    ("ie", "Interlingue"),
    ("iu", "Inuktitut"),
    ("ik", "Inupiaq"),
    ("ga", "Irish"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("jv", "Javanese"),
    ("kl", "Kalaallisut"),
    ("kn", "Kannada"),
    ("kr", "Kanuri"),
    ("ks", "Kashmiri"),
    ("kk", "Kazakh"),
    ("km", "Khmer"),
    ("ki", "Kikuyu"),
    ("rw", "Kinyarwanda"),
    ("kv", "Komi"),
    ("kg", "Kongo"),
    ("ko", "Korean"),
    ("ku", "Kurdish"),
    ("kj", "Kwanyama"),
    ("ky", "Kyrgyz"),
    ("lo", "Lao"),
    ("la", "Latin"),
    ("lv", "Latvian"),
    ("li", "Limburgish"),
    ("ln", "Lingala"),
    ("lt", "Lithuanian"),
    ("lu", "Luba-Katanga"),
    ("lb", "Luxembourgish"),
    ("mk", "Macedonian"),
    ("mg", "Malagasy"),
    ("ms", "Malay"),
    ("ml", "Malayalam"),
    ("mt", "Maltese"),
    ("gv", "Manx"),
    ("mi", "Maori"),
    ("mr", "Marathi"),
    ("mh", "Marshallese"),
    ("mn", "Mongolian"),
    ("na", "Nauru"),
    ("nv", "Navajo"),
    ("ng", "Ndonga"),
    ("ne", "Nepali"),
    ("nd", "North Ndebele"),
    ("se", "Northern Sami"),
    ("no", "Norwegian"),
    ("nb", "Norwegian Bokmal"),
    ("nn", "Norwegian Nynorsk"),
    ("ii", "Nuosu"),
    ("oc", "Occitan"),
    ("oj", "Ojibwe"),
    ("cu", "Old Church Slavonic"),
    ("or", "Oriya"),
    ("om", "Oromo"),
    ("os", "Ossetian"),
    ("pi", "Pali"),
    ("ps", "Pashto"),
    ("fa", "Persian"),
    ("pl", "Polish"),
    ("pt", "Portuguese"),
    ("pa", "Punjabi"),
    ("qu", "Quechua"),
    ("ro", "Romanian"),
    ("rm", "Romansh"),
    ("rn", "Rundi"),
    ("ru", "Russian"),
    ("sm", "Samoan"),
    ("sg", "Sango"),
    ("sa", "Sanskrit"),
    ("sc", "Sardinian"),
    ("gd", "Scottish Gaelic"),
    ("sr", "Serbian"),
    ("sn", "Shona"),
    ("sd", "Sindhi"),
    ("si", "Sinhala"),
    ("sk", "Slovak"),
    ("sl", "Slovenian"),
    ("so", "Somali"),
    ("nr", "South Ndebele"),
    ("st", "Southern Sotho"),
    ("es", "Spanish"),
    ("su", "Sundanese"),
    ("sw", "Swahili"),
    ("ss", "Swati"),
    ("sv", "Swedish"),
    ("tl", "Tagalog"),
    ("ty", "Tahitian"),
    ("tg", "Tajik"),
    ("ta", "Tamil"),
    ("tt", "Tatar"),
    ("te", "Telugu"),
    ("th", "Thai"),
    ("bo", "Tibetan"),
    ("ti", "Tigrinya"),
    ("to", "Tongan"),
    ("ts", "Tsonga"),
    ("tn", "Tswana"),
    ("tr", "Turkish"),
    ("tk", "Turkmen"),
    ("tw", "Twi"),
    ("uk", "Ukrainian"),
    ("ur", "Urdu"),
    ("ug", "Uyghur"),
    ("uz", "Uzbek"),
    ("ve", "Venda"),
    ("vi", "Vietnamese"),
    ("vo", "Volapuk"),
    ("wa", "Walloon"),
    ("cy", "Welsh"),
    ("fy", "Western Frisian"),
    ("wo", "Wolof"),
    ("xh", "Xhosa"),
    ("yi", "Yiddish"),
    ("yo", "Yoruba"),
    ("za", "Zhuang"),
    ("zu", "Zulu"),
];

/// The default mode for new terminal sessions.
#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Default mode for new sessions.",
    rename_all = "snake_case"
)]
pub enum DefaultSessionMode {
    /// New sessions start in the terminal mode (default).
    #[default]
    Terminal,
    /// New sessions start in agent view.
    Agent,
    /// New sessions start in cloud (ambient) agent mode.
    AmbientAgent,
    /// New sessions open a user-defined tab config.
    /// The specific config is identified by the companion `default_tab_config_path` setting.
    TabConfig,
    /// New sessions open in a local Docker sandbox.
    /// Requires the `LocalDockerSandbox` feature flag; falls back to `Terminal` when disabled.
    DockerSandbox,
}

settings::macros::implement_setting_for_enum!(
    DefaultSessionMode,
    AISettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    surface: settings::SettingSurfaces::GUI,
    private: false,
    toml_path: "general.default_session_mode",
    description: "The default mode for new terminal sessions.",
);

impl DefaultSessionMode {
    /// Display name for the settings dropdown.
    pub fn display_name(&self) -> String {
        match self {
            DefaultSessionMode::Terminal => crate::t!("default-session-terminal"),
            DefaultSessionMode::Agent => crate::t!("default-session-agent"),
            DefaultSessionMode::AmbientAgent => crate::t!("default-session-ambient-agent"),
            DefaultSessionMode::TabConfig => crate::t!("default-session-tab-config"),
            DefaultSessionMode::DockerSandbox => crate::t!("default-session-local-docker-sandbox"),
        }
    }
}

/// Controls how agent thinking/reasoning traces are displayed after streaming.
#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Controls how agent thinking is displayed after streaming.",
    rename_all = "snake_case"
)]
pub enum ThinkingDisplayMode {
    /// Show reasoning blocks while streaming, then collapse them when complete (default).
    #[default]
    ShowAndCollapse,
    /// Always keep reasoning blocks expanded, even after streaming finishes.
    AlwaysShow,
    /// Never show reasoning blocks.
    NeverShow,
}

settings::macros::implement_setting_for_enum!(
    ThinkingDisplayMode,
    AISettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    surface: settings::SettingSurfaces::GUI,
    private: false,
    toml_path: "agents.warp_agent.other.thinking_display_mode",
    description: "Controls how agent thinking traces are displayed after streaming.",
);

impl ThinkingDisplayMode {
    /// Display name for the settings dropdown.
    pub fn display_name(&self) -> String {
        match self {
            ThinkingDisplayMode::ShowAndCollapse => {
                crate::t!("thinking-display-show-collapse-label")
            }
            ThinkingDisplayMode::AlwaysShow => crate::t!("thinking-display-always-show-label"),
            ThinkingDisplayMode::NeverShow => crate::t!("thinking-display-never-show-label"),
        }
    }

    pub fn command_palette_description(&self) -> String {
        match self {
            ThinkingDisplayMode::ShowAndCollapse => {
                crate::t!("agent-thinking-display-show-collapse")
            }
            ThinkingDisplayMode::AlwaysShow => crate::t!("agent-thinking-display-always-show"),
            ThinkingDisplayMode::NeverShow => crate::t!("agent-thinking-display-never-show"),
        }
    }

    pub fn should_render(&self) -> bool {
        !matches!(self, ThinkingDisplayMode::NeverShow)
    }

    pub fn should_keep_expanded(&self) -> bool {
        matches!(self, ThinkingDisplayMode::AlwaysShow)
    }
}

/// Controls how child-agent message bodies are displayed.
#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Controls how child-agent messages are displayed.",
    rename_all = "snake_case"
)]
pub enum OrchestrationMessageDisplayMode {
    /// Show child-agent messages while streaming, then collapse them.
    ShowAndCollapse,
    /// Keep child-agent message bodies expanded.
    AlwaysShow,
    /// Keep child-agent message bodies collapsed.
    #[default]
    AlwaysCollapse,
}

settings::macros::implement_setting_for_enum!(
    OrchestrationMessageDisplayMode,
    AISettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    surface: settings::SettingSurfaces::GUI,
    private: false,
    toml_path: "agents.warp_agent.other.orchestration_message_display_mode",
    description: "Controls how child-agent messages are displayed.",
);

/// Which unit the usage entry displays in InfiniShell TUI.
#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Which unit the usage entry displays in InfiniShell TUI.",
    rename_all = "snake_case"
)]
pub enum TuiUsageDisplayMode {
    /// Credits spent — the same number the GUI's usage footer shows (default).
    #[default]
    Credits,
    /// Provider dollar cost.
    Cost,
}

settings::macros::implement_setting_for_enum!(
    TuiUsageDisplayMode,
    AISettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Never,
    surface: settings::SettingSurfaces::TUI,
    private: false,
    toml_path: "agents.usage_display_mode",
    description: "Which unit the usage entry displays in InfiniShell TUI: credits or provider cost.",
);
/// One configurable item in the InfiniShell TUI statusline.
#[derive(
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Copy,
    Clone,
    Hash,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "A configurable item in the InfiniShell TUI statusline.",
    rename_all = "snake_case"
)]
#[serde(rename_all = "snake_case")]
pub enum TuiStatuslineItem {
    AutoApprove,
    /// Vim mode indicator (NOR/INS/VIS/V-L/REP); hidden when vim mode is disabled.
    VimModeIndicator,
    Model,
    WorkingDirectory,
    GitBranch,
    GitBranchStatus,
    GitDiffStatus,
    GitHubPullRequest,
    CreditUsage,
    ContextWindowUsage,
    Date,
    #[schemars(rename = "time_12_hour")]
    Time12Hour,
    #[schemars(rename = "time_24_hour")]
    Time24Hour,
    AgentTodoList,
    VoiceInput,
}

impl TuiStatuslineItem {
    pub const ALL: [Self; 15] = [
        Self::AutoApprove,
        Self::VimModeIndicator,
        Self::Model,
        Self::WorkingDirectory,
        Self::GitBranch,
        Self::GitBranchStatus,
        Self::GitDiffStatus,
        Self::GitHubPullRequest,
        Self::CreditUsage,
        Self::ContextWindowUsage,
        Self::Date,
        Self::Time12Hour,
        Self::Time24Hour,
        Self::AgentTodoList,
        Self::VoiceInput,
    ];

    pub fn label(self) -> String {
        match self {
            Self::AutoApprove => crate::t!("settings-tui-statusline-auto-approve"),
            Self::VimModeIndicator => crate::t!("settings-tui-statusline-vim-mode"),
            Self::Model => crate::t!("settings-tui-statusline-model"),
            Self::WorkingDirectory => crate::t!("settings-tui-statusline-working-directory"),
            Self::GitBranch => crate::t!("settings-tui-statusline-git-branch"),
            Self::GitBranchStatus => crate::t!("settings-tui-statusline-git-branch-status"),
            Self::GitDiffStatus => crate::t!("settings-tui-statusline-git-diff-status"),
            Self::GitHubPullRequest => crate::t!("settings-tui-statusline-github-pull-request"),
            Self::CreditUsage => crate::t!("settings-tui-statusline-credit-usage"),
            Self::ContextWindowUsage => crate::t!("settings-tui-statusline-context-window-usage"),
            Self::Date => crate::t!("settings-tui-statusline-date"),
            Self::Time12Hour => crate::t!("settings-tui-statusline-time-12-hour"),
            Self::Time24Hour => crate::t!("settings-tui-statusline-time-24-hour"),
            Self::AgentTodoList => crate::t!("settings-tui-statusline-agent-todo-list"),
            Self::VoiceInput => crate::t!("settings-tui-statusline-voice-input"),
        }
    }
}

/// Ordered and enabled items in the InfiniShell TUI statusline.
#[derive(
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Clone,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
pub struct TuiStatuslineConfig {
    pub order: Vec<TuiStatuslineItem>,
    pub enabled: Vec<TuiStatuslineItem>,
}

impl Default for TuiStatuslineConfig {
    fn default() -> Self {
        Self {
            order: TuiStatuslineItem::ALL.to_vec(),
            enabled: vec![
                TuiStatuslineItem::AutoApprove,
                TuiStatuslineItem::VimModeIndicator,
                TuiStatuslineItem::Model,
                TuiStatuslineItem::WorkingDirectory,
                TuiStatuslineItem::GitBranch,
                TuiStatuslineItem::GitDiffStatus,
            ],
        }
    }
}

impl TuiStatuslineConfig {
    /// Returns a complete, duplicate-free catalog and a valid enabled subset.
    pub fn normalized(&self) -> Self {
        let is_legacy_config = !self.order.contains(&TuiStatuslineItem::VimModeIndicator);
        let mut order = Vec::with_capacity(TuiStatuslineItem::ALL.len());
        for item in self.order.iter().copied().chain(TuiStatuslineItem::ALL) {
            if TuiStatuslineItem::ALL.contains(&item) && !order.contains(&item) {
                order.push(item);
            }
        }

        let mut enabled = Vec::with_capacity(self.enabled.len());
        for item in self.enabled.iter().copied() {
            if order.contains(&item) && !enabled.contains(&item) {
                enabled.push(item);
            }
        }
        if is_legacy_config {
            enabled.insert(0, TuiStatuslineItem::VimModeIndicator);
        }

        Self { order, enabled }
    }

    pub fn is_enabled(&self, item: TuiStatuslineItem) -> bool {
        self.enabled.contains(&item)
    }
}

impl TuiUsageDisplayMode {
    /// The other unit — clicking the usage entry flips to this.
    pub fn toggled(self) -> Self {
        match self {
            TuiUsageDisplayMode::Credits => TuiUsageDisplayMode::Cost,
            TuiUsageDisplayMode::Cost => TuiUsageDisplayMode::Credits,
        }
    }
}

impl OrchestrationMessageDisplayMode {
    /// Display name for the settings dropdown.
    pub fn display_name(&self) -> String {
        match self {
            OrchestrationMessageDisplayMode::ShowAndCollapse => {
                crate::t!("orchestration-display-show-collapse-label")
            }
            OrchestrationMessageDisplayMode::AlwaysShow => {
                crate::t!("orchestration-display-always-show-label")
            }
            OrchestrationMessageDisplayMode::AlwaysCollapse => {
                crate::t!("orchestration-display-always-collapse-label")
            }
        }
    }

    pub fn command_palette_description(&self) -> String {
        match self {
            OrchestrationMessageDisplayMode::ShowAndCollapse => {
                crate::t!("orchestration-display-show-collapse-command")
            }
            OrchestrationMessageDisplayMode::AlwaysShow => {
                crate::t!("orchestration-display-always-show-command")
            }
            OrchestrationMessageDisplayMode::AlwaysCollapse => {
                crate::t!("orchestration-display-always-collapse-command")
            }
        }
    }

    /// Whether child-agent message bodies should expand while streaming.
    pub fn should_expand_agent_message_body(&self) -> bool {
        matches!(
            self,
            OrchestrationMessageDisplayMode::ShowAndCollapse
                | OrchestrationMessageDisplayMode::AlwaysShow
        )
    }

    /// Whether child-agent message bodies should collapse after streaming.
    pub fn should_collapse_agent_message_body_on_finish(&self) -> bool {
        matches!(self, OrchestrationMessageDisplayMode::ShowAndCollapse)
    }
}

/// Controls what happens when a user submits a new prompt while the agent is
/// still responding to an earlier prompt.
///
/// This is the *default* used when a conversation has no explicit auto-queue
/// override. Per-conversation overrides live on `QueuedQueryModel` and take
/// precedence over this setting.
#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Default behavior when submitting a new prompt while the agent is still responding.",
    rename_all = "snake_case"
)]
pub enum PromptSubmissionMode {
    /// Cancel the in-flight response and submit the new prompt immediately
    /// (default).
    #[default]
    Interrupt,
    /// Hold the new prompt until the in-flight response finishes, then submit.
    Queue,
}

settings::macros::implement_setting_for_enum!(
    PromptSubmissionMode,
    AISettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    surface: settings::SettingSurfaces::GUI,
    private: false,
    toml_path: "agents.warp_agent.other.default_prompt_submission_mode",
    description: "Default behavior when submitting a new prompt while the agent is still responding.",
    feature_flag: FeatureFlag::QueueSlashCommand,
);

impl PromptSubmissionMode {
    /// Display name for the settings dropdown.
    pub fn display_name(&self) -> String {
        match self {
            PromptSubmissionMode::Interrupt => crate::t!("prompt-submission-interrupt-label"),
            PromptSubmissionMode::Queue => crate::t!("prompt-submission-queue-label"),
        }
    }

    pub fn command_palette_description(&self) -> String {
        match self {
            PromptSubmissionMode::Interrupt => crate::t!("prompt-submission-interrupt-command"),
            PromptSubmissionMode::Queue => {
                crate::t!("prompt-submission-queue-command")
            }
        }
    }
}

/// What happens when a prompt is submitted while an agent controls an agent-requested
/// long-running command (LRC).
///
/// Only consulted when [`PromptSubmissionMode`] is `Interrupt`: in `Queue` mode
/// prompts always queue until the full response finishes, so this setting is
/// hidden and ignored.
#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "What happens when a prompt is submitted while an agent controls an agent-requested long-running command.",
    rename_all = "snake_case"
)]
pub enum LongRunningCommandSubmissionMode {
    /// Send the prompt to the agent immediately, steering it mid-command.
    SendImmediately,
    /// Queue the prompt and send it to the agent when the command finishes
    /// (default).
    #[default]
    QueueUntilCommandCompletes,
}

settings::macros::implement_setting_for_enum!(
    LongRunningCommandSubmissionMode,
    AISettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    surface: settings::SettingSurfaces::GUI,
    private: false,
    toml_path: "agents.warp_agent.other.long_running_command_submission_mode",
    description: "What happens when a prompt is submitted while an agent controls an agent-requested long-running command.",
    feature_flag: FeatureFlag::QueueSlashCommand,
);

impl LongRunningCommandSubmissionMode {
    /// Display name for the settings dropdown.
    pub fn display_name(&self) -> String {
        match self {
            LongRunningCommandSubmissionMode::SendImmediately => {
                crate::t!("lrc-submission-send-immediately-label")
            }
            LongRunningCommandSubmissionMode::QueueUntilCommandCompletes => {
                crate::t!("lrc-submission-queue-label")
            }
        }
    }

    pub fn command_palette_description(&self) -> String {
        match self {
            LongRunningCommandSubmissionMode::SendImmediately => {
                crate::t!("lrc-submission-send-immediately-command")
            }
            LongRunningCommandSubmissionMode::QueueUntilCommandCompletes => {
                crate::t!("lrc-submission-queue-command")
            }
        }
    }
}

/// Tracks the state of the quota reset banner
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Default,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(description = "State of the quota reset banner.")]
pub struct BannerState {
    #[serde(default)]
    #[schemars(description = "Whether the banner has been dismissed.")]
    pub dismissed: bool,
}

/// Tracks information about a single billing cycle for AI request usage
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(description = "Information about a single billing cycle.")]
pub struct CycleInfo {
    /// End date of the billing cycle
    #[schemars(description = "End date of the billing cycle.")]
    pub end_date: DateTime<Utc>,
    /// Whether the quota was exceeded in this cycle
    #[schemars(description = "Whether the usage quota was exceeded in this cycle.")]
    pub was_quota_exceeded: bool,
    /// State of the quota reset banner
    #[schemars(description = "State of the quota reset banner for this cycle.")]
    pub banner_state: BannerState,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Default,
    PartialEq,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(description = "AI usage quota information across billing cycles.")]
pub struct AIRequestQuotaInfo {
    /// History of billing cycles and their usage.
    ///
    /// Note that these are only populated going forward from when this setting
    /// was introduced.
    #[schemars(description = "History of billing cycles and their quota usage.")]
    pub cycle_history: Vec<CycleInfo>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Default,
    PartialEq,
    EnumIter,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "File read permission level for the agent.",
    rename_all = "snake_case"
)]
pub enum AgentModeCodingPermissionsType {
    /// Agent Mode must ask for explicit permission for any type of file read.
    #[default]
    AlwaysAskBeforeReading,
    /// Agent Mode can always read files without explicit consent.
    AlwaysAllowReading,
    /// Agent Mode can only read certain files without explicit consent.
    ///
    /// The specific filepaths are backed by the
    /// [`AISettings::agent_mode_coding_file_read_allowlist`] setting.
    AllowReadingSpecificFiles,
}

// ---------------------------------------------------------------------------
// 自定义 Agent 提供商配置(进程内 Provider)
// ---------------------------------------------------------------------------

/// Agent 提供商支持的协议类型。
///
/// 第一阶段仅支持 OpenAI 兼容协议(适用于 OpenAI、DeepSeek、智谱 GLM、
/// Moonshot、通义千问 DashScope-OpenAI 兼容端点、SiliconFlow、OpenRouter、
/// 任何 OpenAI 兼容的本地服务等)。后续可在此扩展 Anthropic、Google、Bedrock。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AgentProviderKind {
    /// OpenAI 兼容的 Chat Completions / `/v1/models` 协议。
    #[default]
    OpenAiCompatible,
}

/// BYOP provider 实际使用的 API 协议类型 — 显式指定,
/// 由 chat_stream 通过 genai `ServiceTargetResolver` 一对一映射到对应的
/// `AdapterKind`,完全绕过"按模型名识别"的默认行为,避免误识别。
///
/// **注意**:这是相对 [`AgentProviderKind`] 的更细粒度维度。
/// `AgentProviderKind` 目前只有 `OpenAiCompatible`(语义"用户自管 endpoint"),
/// `AgentProviderApiType` 决定 genai 用哪种原生协议序列化请求 / 解析响应。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AgentProviderApiType {
    /// OpenAI Chat Completions(`POST /v1/chat/completions`)。
    /// 适用于:OpenAI 官方、DeepSeek、SiliconFlow、OpenRouter、智谱 GLM、
    /// Moonshot、DashScope-OpenAI 兼容、本地 vLLM/llama.cpp 等。
    #[default]
    OpenAi,
    /// OpenAI Responses API(`POST /v1/responses`)。
    /// 适用于:GPT-5 / Codex / Pro 等较新模型。
    OpenAiResp,
    /// Google Gemini 原生协议(generativelanguage.googleapis.com)。
    Gemini,
    /// Anthropic Messages API 原生协议(`POST /v1/messages`,默认 `api.anthropic.com/v1/`)。
    Anthropic,
    /// Ollama 原生协议(本地或自建 Ollama)。
    Ollama,
    /// DeepSeek 原生协议。与 OpenAI 兼容相比:多轮 thinking 模式必须把
    /// `reasoning_content` 字段带回服务端(否则 400),仅 genai DeepSeek
    /// adapter 处理这个非标字段。`deepseek-reasoner / deepseek-v4-flash` 等
    /// thinking-mode 模型必须选这个类型,普通 chat 模型(`deepseek-chat`)
    /// 选 OpenAI 也可以工作。
    DeepSeek,
}

/// OpenAI Responses API 的会话状态策略。
///
/// 默认使用本地回放，保证 `store:false`，也兼容不实现 provider 持久状态的
/// OpenAI-compatible endpoint。其余两种模式会把会话内容持久化到 provider，
/// 因此必须由用户显式选择。
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    schemars::JsonSchema,
    EnumIter,
)]
#[serde(rename_all = "snake_case")]
pub enum ResponsesStateModeSetting {
    #[default]
    LocalReplay,
    PreviousResponse,
    Conversation,
}

impl ResponsesStateModeSetting {
    pub fn display_name(self) -> String {
        match self {
            Self::LocalReplay => crate::t!("settings-responses-state-local-zdr"),
            Self::PreviousResponse => crate::t!("settings-responses-state-provider-chain"),
            Self::Conversation => crate::t!("settings-responses-state-cloud-conversation"),
        }
    }
}

/// Responses 主请求使用的传输。
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    schemars::JsonSchema,
    EnumIter,
)]
#[serde(rename_all = "snake_case")]
pub enum ResponsesTransportSetting {
    #[default]
    Http,
    WebSocket,
}

impl ResponsesTransportSetting {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Http => "HTTP + SSE",
            Self::WebSocket => "WebSocket",
        }
    }
}

/// 单个自定义 provider 的 Responses 专属能力配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AgentProviderResponsesOptions {
    #[serde(default)]
    pub state_mode: ResponsesStateModeSetting,
    #[serde(default)]
    pub transport: ResponsesTransportSetting,
    /// 后台响应可在断线后按 sequence number 恢复；本地/ZDR 模式不允许启用。
    #[serde(default)]
    pub background: bool,
    /// 服务端自动 compaction 阈值；0 表示不发送 `context_management`。
    #[serde(default)]
    pub compact_threshold: u32,
    /// 启用 Programmatic Tool Calling。写操作仍由本地审批和沙箱执行。
    #[serde(default)]
    pub programmatic_tool_calling: bool,
    /// GPT-5.6 Pro reasoning mode，以更高延迟和 token 消耗换取复杂任务可靠性。
    #[serde(default)]
    pub reasoning_pro_mode: bool,
    /// GPT-5.6 跨轮复用可用 reasoning items，而非只使用当前轮 reasoning。
    #[serde(default)]
    pub reasoning_all_turns: bool,
    /// OpenAI Responses Multi-agent Beta。还会受独立 feature flag 保护。
    #[serde(default)]
    pub multi_agent_beta: bool,
    #[serde(default = "default_responses_max_concurrent_subagents")]
    pub max_concurrent_subagents: u8,
}

impl Default for AgentProviderResponsesOptions {
    fn default() -> Self {
        Self {
            state_mode: ResponsesStateModeSetting::default(),
            transport: ResponsesTransportSetting::default(),
            background: false,
            compact_threshold: 0,
            programmatic_tool_calling: false,
            reasoning_pro_mode: false,
            reasoning_all_turns: false,
            multi_agent_beta: false,
            max_concurrent_subagents: default_responses_max_concurrent_subagents(),
        }
    }
}

fn default_responses_max_concurrent_subagents() -> u8 {
    3
}

/// Provider 级别的 reasoning effort(思考深度)偏好。
///
/// 语义说明:
/// - `Auto`(默认):不向 genai 传 effort。OpenAI / Anthropic adapter 内部会按
///   模型名后缀(`-low` / `-high` / `-zero` 等)自动推断;Gemini / DeepSeek 不推断。
/// - `Off`:对支持 reasoning 的模型显式发送 `none`,关闭思考链。
/// - 其他档位:client 端先用 `reasoning::model_supports_reasoning` 判定,**仅在该
///   模型支持时**注入,避免向 claude-3-5-haiku / gpt-4o / gemini-1.5-pro 等老模型
///   注入 thinking 参数被上游 400 拒绝。
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    schemars::JsonSchema,
    EnumIter,
)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffortSetting {
    #[default]
    Auto,
    Off,
    Minimal,
    Low,
    Medium,
    High,
    XHigh,
    Max,
}

impl ReasoningEffortSetting {
    pub fn display_name(&self) -> String {
        match self {
            Self::Auto => crate::t!("reasoning-effort-auto"),
            Self::Off => crate::t!("reasoning-effort-off"),
            Self::Minimal => crate::t!("reasoning-effort-minimal"),
            Self::Low => crate::t!("reasoning-effort-low"),
            Self::Medium => crate::t!("reasoning-effort-medium"),
            Self::High => crate::t!("reasoning-effort-high"),
            Self::XHigh => crate::t!("reasoning-effort-xhigh"),
            Self::Max => crate::t!("reasoning-effort-max"),
        }
    }

    /// 转成 genai `ReasoningEffort`。`Auto` 返回 None(调用方不要 set)。
    pub fn to_genai(self) -> Option<genai::chat::ReasoningEffort> {
        use genai::chat::ReasoningEffort as GE;
        Some(match self {
            Self::Auto => return None,
            Self::Off => GE::None,
            Self::Minimal => GE::Minimal,
            Self::Low => GE::Low,
            Self::Medium => GE::Medium,
            Self::High => GE::High,
            Self::XHigh => GE::XHigh,
            Self::Max => GE::Max,
        })
    }
}

impl AgentProviderApiType {
    /// 设置 UI dropdown 显示文字。
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::OpenAiResp => "OpenAI-Response",
            Self::Gemini => "Gemini",
            Self::Anthropic => "Anthropic",
            Self::Ollama => "Ollama",
            Self::DeepSeek => "DeepSeek",
        }
    }

    /// 反向解析 Debug 格式名(`OpenAi` / `DeepSeek` 等),用于 BYOPLastUsedReasoningMap
    /// 这种 `<api_type>:<model_id>` 复合 key 的 hydrate 场景。未知字符串返回 None。
    pub fn from_debug_str(s: &str) -> Option<Self> {
        Some(match s {
            "OpenAi" => Self::OpenAi,
            "OpenAiResp" => Self::OpenAiResp,
            "Gemini" => Self::Gemini,
            "Anthropic" => Self::Anthropic,
            "Ollama" => Self::Ollama,
            "DeepSeek" => Self::DeepSeek,
            _ => return None,
        })
    }

    /// 当用户没填 base_url 时使用的默认 endpoint。新建 provider / 切换 ApiType
    /// 时,UI 可调用此方法预填,便于新手。
    ///
    /// **必须以 `/` 结尾**:genai 0.6.x 的 adapter 内部用 `format!("{base_url}messages")` /
    /// `Url::join` 拼接 service path,缺尾随 `/` 会拼出乱地址(Anthropic 是 `.devmessages` 直接连)
    /// 或被 `Url::join` 吃掉 path 最后一段。client 端 `build_client` 也会兜底补 `/`,
    /// 这里依然要求显式尾随 `/`,保证 UI 预填值落到 settings.toml 后即使绕过 client 兜底也是对的。
    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::OpenAi => "https://api.openai.com/v1/",
            Self::OpenAiResp => "https://api.openai.com/v1/",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta/",
            Self::Anthropic => "https://api.anthropic.com/v1/",
            Self::Ollama => "http://localhost:11434/",
            Self::DeepSeek => "https://api.deepseek.com/v1/",
        }
    }
}

/// 一条用户自定义的 Agent 提供商配置。
///
/// `api_key` **不**保存在这里,而是保存在 `AgentProviderSecrets` 单例(secure storage),
/// 通过 `id` 关联。这样设置文件 (settings.toml) 不会泄漏敏感信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AgentProvider {
    /// 提供商唯一 ID,首次创建时生成,持久化到设置中作为 secret 的关联键。
    #[serde(default = "AgentProvider::default_id")]
    pub id: String,

    /// 用户给这个提供商起的显示名(例如 "DeepSeek 官方"、"本地 Ollama")。
    pub name: String,

    /// 协议类型,目前固定为 OpenAI 兼容(语义"用户自管 endpoint")。
    /// 实际请求/响应序列化协议由 [`AgentProvider::api_type`] 决定。
    #[serde(default)]
    pub kind: AgentProviderKind,

    /// 显式指定的 API 协议类型(OpenAI / OpenAI-Response / Gemini / Anthropic / Ollama)。
    /// 老配置(无此字段)反序列化为 `OpenAi` 兼容老语义。
    #[serde(default)]
    pub api_type: AgentProviderApiType,

    /// API base URL,例如 `https://api.deepseek.com/v1`、`http://localhost:11434`。
    /// 不要带尾随斜杠,但代码侧会做容错。
    pub base_url: String,

    /// 用户配置的、希望暴露给 Agent 选择的模型列表。
    /// 每条同时含 `name`(显示名) 与 `id`(发送给上游 API 的 model 字段值)。
    #[serde(default)]
    pub models: Vec<AgentProviderModel>,

    /// 附加 HTTP 请求头,发给上游 provider 时逐条 merge 进请求。
    /// 用于需要额外路由头的 gateway(如 Portkey 的 `x-portkey-provider`)。
    /// `api_key` 仍走 `Authorization: Bearer` 标准路径。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_headers: Vec<(String, String)>,

    /// 仅在 `api_type = open_ai_resp` 时生效。
    #[serde(default)]
    pub responses: AgentProviderResponsesOptions,
}

impl AgentProvider {
    fn default_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// 构造一个新的、空的提供商。
    pub fn new_empty() -> Self {
        Self {
            id: Self::default_id(),
            name: String::new(),
            kind: AgentProviderKind::default(),
            api_type: AgentProviderApiType::default(),
            base_url: String::new(),
            models: Vec::new(),
            extra_headers: Vec::new(),
            responses: AgentProviderResponsesOptions::default(),
        }
    }
}

impl settings_value::SettingsValue for AgentProvider {}

/// 单条模型条目: `name` 是用户在 model picker 中看到的显示名,
/// `id` 是真正发给上游 OpenAI 兼容 API 的 `model` 字段值。
///
/// 序列化为 toml 时形如:
/// ```toml
/// [[agent_providers.models]]
/// name = "DS V3 通用"
/// id   = "deepseek-chat"
/// ```
///
/// 反序列化兼容老格式 `models = ["deepseek-chat", "deepseek-coder"]`
/// (每个字符串视为 `{ name = id, id = id }`),便于现有用户无痛升级。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct AgentProviderModel {
    pub name: String,
    pub id: String,

    /// 上下文窗口(tokens)。来源:用户填或 models.dev 自动带入。
    /// 0 表示未知 — 请求使用默认安全预算，仍限制单条和累计工具输出。
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub context_window: u32,

    /// 单次最大输出 tokens。0 表示未指定。
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub max_output_tokens: u32,

    /// 是否支持 reasoning(思考/CoT)输出。
    #[serde(default, skip_serializing_if = "is_false")]
    pub reasoning: bool,

    /// 是否支持 function/tool calling。
    /// 默认 `true` — 老配置升级 + 用户手填新 model 时不要默认禁工具,
    /// 不支持工具调用的模型由 models.dev 数据带入显式 false。
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub tool_call: bool,

    // ----- 多模态附件 capability,三态语义:
    // - `None`(toml 字段缺省)= Auto: 运行时按 models.dev catalog → substring fallback 推断
    // - `Some(true)` = Force-On: 用户强制开,绕过推断
    // - `Some(false)` = Force-Off: 用户强制关
    //
    // 字段命名故意用 `image` 而非 `vision`,跟 models.dev `modalities.input: ["image"]`
    // 字面对应,语义最窄不歧义(避免用户误以为 vision = image+pdf+...)。
    /// 是否支持图像输入(image/* MIME)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<bool>,
    /// 是否支持 PDF 文档输入(application/pdf)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pdf: Option<bool>,
    /// 是否支持音频输入(audio/* MIME)。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}
fn is_false(v: &bool) -> bool {
    !*v
}
fn is_true(v: &bool) -> bool {
    *v
}
fn default_true() -> bool {
    true
}

impl AgentProviderModel {
    pub fn from_id(id: String) -> Self {
        Self {
            name: id.clone(),
            id,
            context_window: 0,
            max_output_tokens: 0,
            reasoning: false,
            tool_call: true,
            image: None,
            pdf: None,
            audio: None,
        }
    }
}

impl<'de> Deserialize<'de> for AgentProviderModel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Either {
            Plain(String),
            Full {
                #[serde(default)]
                name: String,
                id: String,
                #[serde(default)]
                context_window: u32,
                #[serde(default)]
                max_output_tokens: u32,
                #[serde(default)]
                reasoning: bool,
                #[serde(default = "default_true")]
                tool_call: bool,
                #[serde(default)]
                image: Option<bool>,
                #[serde(default)]
                pdf: Option<bool>,
                #[serde(default)]
                audio: Option<bool>,
            },
        }
        match Either::deserialize(deserializer)? {
            Either::Plain(id) => Ok(AgentProviderModel::from_id(id)),
            Either::Full {
                name,
                id,
                context_window,
                max_output_tokens,
                reasoning,
                tool_call,
                image,
                pdf,
                audio,
            } => {
                let name = if name.is_empty() { id.clone() } else { name };
                Ok(AgentProviderModel {
                    name,
                    id,
                    context_window,
                    max_output_tokens,
                    reasoning,
                    tool_call,
                    image,
                    pdf,
                    audio,
                })
            }
        }
    }
}

impl settings_value::SettingsValue for AgentProviderModel {}

/// Maps custom toolbar command regex patterns to CLI agent names.
/// Keys are regex patterns (insertion-ordered), values are serialized CLIAgent names (e.g. "Claude").
/// An empty string value means "Any CLI Agent" (CLIAgent::Unknown).
///
/// Uses `IndexMap` to preserve insertion order so the settings UI list is deterministic.
/// Supports backward-compatible deserialization from the legacy `Vec<String>` format,
/// where each string is converted to a key with an empty agent value.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ToolbarCommandMap(IndexMap<String, String>);

impl ToolbarCommandMap {
    pub(crate) fn new(map: IndexMap<String, String>) -> Self {
        Self(map)
    }
}

impl<'de> Deserialize<'de> for ToolbarCommandMap {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum MapOrVec {
            Map(IndexMap<String, String>),
            Vec(Vec<String>),
        }

        match MapOrVec::deserialize(deserializer) {
            Ok(MapOrVec::Map(map)) => Ok(ToolbarCommandMap::new(map)),
            Ok(MapOrVec::Vec(vec)) => {
                let map = vec
                    .into_iter()
                    .map(|pattern| (pattern, String::new()))
                    .collect();
                Ok(ToolbarCommandMap::new(map))
            }
            Err(e) => Err(e),
        }
    }
}

impl schemars::JsonSchema for ToolbarCommandMap {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("ToolbarCommandMap")
    }

    fn json_schema(r#gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        r#gen.subschema_for::<HashMap<String, String>>()
    }
}

impl std::ops::Deref for ToolbarCommandMap {
    type Target = IndexMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl settings_value::SettingsValue for ToolbarCommandMap {
    fn to_file_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.0).unwrap_or_default()
    }

    fn from_file_value(value: &serde_json::Value) -> Option<Self> {
        // Try map format first (using from_value to preserve insertion order), then legacy array format.
        if value.is_object()
            && let Ok(map) = serde_json::from_value::<IndexMap<String, String>>(value.clone())
        {
            return Some(ToolbarCommandMap::new(map));
        }
        if let Some(arr) = value.as_array() {
            let result: IndexMap<String, String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| (s.to_string(), String::new())))
                .collect();
            return Some(ToolbarCommandMap::new(result));
        }
        None
    }
}

/// 持久化记忆"上次某 (api_type, model) 用过的 reasoning effort 档位"。
/// key 形式:`<api_type>:<model_id>`,例如 `DeepSeek:deepseek-v4-pro`。
/// value 是 `ReasoningEffortSetting` 枚举(serde_json snake_case)。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BYOPLastUsedReasoningMap(pub IndexMap<String, ReasoningEffortSetting>);

impl BYOPLastUsedReasoningMap {
    pub fn new(map: IndexMap<String, ReasoningEffortSetting>) -> Self {
        Self(map)
    }

    /// 拼 key:`<api_type>:<model_id>`。api_type 用 Debug 拼出 `DeepSeek` 等驼峰名,
    /// 跟 ReasoningEffortSetting 的 serde 形式无关。
    pub fn make_key(api_type: AgentProviderApiType, model_id: &str) -> String {
        format!("{api_type:?}:{model_id}")
    }
}

impl std::ops::Deref for BYOPLastUsedReasoningMap {
    type Target = IndexMap<String, ReasoningEffortSetting>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl schemars::JsonSchema for BYOPLastUsedReasoningMap {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "BYOPLastUsedReasoningMap".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        generator.subschema_for::<HashMap<String, String>>()
    }
}

impl settings_value::SettingsValue for BYOPLastUsedReasoningMap {
    fn to_file_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.0).unwrap_or_default()
    }

    fn from_file_value(value: &serde_json::Value) -> Option<Self> {
        if value.is_object() {
            if let Ok(map) =
                serde_json::from_value::<IndexMap<String, ReasoningEffortSetting>>(value.clone())
            {
                return Some(Self::new(map));
            }
        }
        None
    }
}

/// Per-agent 设置：控制单个 CLI agent 的工具栏、标签页菜单和标题栏可见性。
/// key 是 CLIAgent 序列化名（例如 "Claude", "Gemini"）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PerAgentSettings {
    /// 是否在终端输入底部展示编码智能体工具栏。
    #[serde(default = "default_true_bool")]
    pub toolbar: bool,
    /// 是否在新建标签页菜单中展示该 agent 的快速启动入口。
    #[serde(default = "default_true_bool", alias = "tab_menu")]
    pub tabmenu: bool,
    /// 是否在标题栏右侧展示该 agent 的快捷启动按钮。
    #[serde(default)]
    pub titlebar: bool,
}

fn default_true_bool() -> bool {
    true
}

impl PerAgentSettings {
    /// 返回指定 agent 的默认值。titlebar 对 Claude/Codex/Gemini/Antigravity 默认开启。
    pub fn default_for(agent: CLIAgent) -> Self {
        let titlebar = matches!(
            agent,
            CLIAgent::Claude | CLIAgent::Codex | CLIAgent::Gemini | CLIAgent::Antigravity
        );
        Self {
            toolbar: true,
            tabmenu: true,
            titlebar,
        }
    }
}

impl Default for PerAgentSettings {
    fn default() -> Self {
        Self {
            toolbar: true,
            tabmenu: true,
            titlebar: false,
        }
    }
}

impl settings_value::SettingsValue for PerAgentSettings {}

define_settings_group!(AISettings, settings: [
    // 历史遗留设置。Zap 的 Zap 智能体现在固定开启,不要用这个字段判断启用状态。
    is_any_ai_enabled: IsAnyAIEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::No),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.is_any_ai_enabled",
        description: "Controls whether all AI features are enabled.",
    },
    // This field should not be referenced directly to lookup active AI enablement -- use the
    // `is_active_ai_enabled()` getter.
    is_active_ai_enabled_internal: IsActiveAIEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::No),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.active_ai.enabled",
        description: "Controls whether proactive AI features like suggestions are enabled.",
    },
    // This field should not be referenced directly to lookup autodetection enablement -- use the
    // `is_ai_autodetection_enabled()` getter.
    ai_autodetection_enabled_internal: AIAutoDetectionEnabled {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::ALL,
        private: false,
        toml_path: "agents.warp_agent.input.ai_auto_detection_enabled",
        description: "Controls whether AI automatically detects natural language input.",
    },
    // This field should not be referenced directly -- use the
    // `is_nld_in_terminal_enabled()` getter.
    // Controls whether natural language detection is enabled in the terminal input.
    //
    // This is only used when `FeatureFlag::AgentView` is enabled.
    nld_in_terminal_enabled_internal: NLDInTerminalEnabled {
        // openWarp:NLD in terminal 默认开。HeuristicClassifier 命中 CJK / 自然语言时
        // 自动切到 AI 输入,这是 openWarp 中文用户能直接在终端写中文当 prompt 的前提。
        // 上游默认 false 是因为 cloud 路线下用户先进 AgentView 全屏,在 terminal mode
        // 不期望自动切换;openWarp 没有 cloud 全屏入口,terminal 即主输入区。
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.input.nld_in_terminal_enabled",
        description: "Controls whether natural language detection is enabled in the terminal input.",
    },
    autodetection_command_denylist: AICommandDenylist {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.input.ai_command_denylist",
        description: "Commands to exclude from AI natural language autodetection.",
    },
    // This field should not be referenced directly to lookup intelligent autosuggestion enablement
    // -- use the `is_intelligent_autosuggestions_enabled()` getter.
    intelligent_autosuggestions_enabled_internal: IntelligentAutosuggestionsEnabled {
        type: bool,
        default: true, // TODO(roland): revisit this when launched to stable
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.active_ai.intelligent_autosuggestions_enabled",
        description: "Controls whether AI-powered intelligent autosuggestions are enabled.",
    }
    // This field should not be referenced directly to lookup Prompt Suggestions
    // enablement -- use the `is_prompt_suggestions_enabled()` getter.
    // Note that AgentModeQuerySuggestionsEnabled is a legacy name (the feature was initially named Agent
    // Mode Query Suggestions), however, we do not want to change the name of the setting key to avoid
    // breaking existing user settings.
    prompt_suggestions_enabled_internal: AgentModeQuerySuggestionsEnabled {
        type: bool,
        default: true, // TODO(advait): revisit this when launched to stable
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.active_ai.agent_mode_query_suggestions_enabled",
        description: "Controls whether prompt suggestions are shown in agent mode.",
    }

    // This field should not be referenced directly to lookup Code Suggestions
    // enablement -- use the `is_code_suggestions_enabled()` getter.
    code_suggestions_enabled_internal: CodeSuggestionsEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.active_ai.code_suggestions_enabled",
        description: "Controls whether AI code suggestions are enabled.",
    }
    // This field should not be referenced directly to lookup natural language autosuggestions
    // enablement -- use the `is_natural_language_autosuggestions_enabled()` getter.
    // This feature refers to ghosted text for AI input queries.
    natural_language_autosuggestions_enabled_internal: NaturalLanguageAutosuggestionsEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.active_ai.natural_language_autosuggestions_enabled",
        description: "Controls whether ghosted text autosuggestions are shown for AI input queries.",
        feature_flag: FeatureFlag::PredictAMQueries,
    }
    // This field should not be referenced directly to lookup git operations AI autogen
    // enablement -- use the `is_git_operations_autogen_enabled()` getter.
    git_operations_autogen_enabled_internal: GitOperationsAutogenEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.active_ai.git_operations_autogen_enabled",
        description: "Controls whether AI auto-generates commit messages and PR title/body in the code review dialogs.",
    }
    // This field should not be referenced directly to lookup Rule Suggestions
    // enablement -- use the `is_rule_suggestions_enabled()` getter.
    rule_suggestions_enabled_internal: RuleSuggestionsEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.active_ai.rule_suggestions_enabled",
        description: "Controls whether the agent suggests rules to save after responses.",
        feature_flag: FeatureFlag::SuggestedRules,
    }
    // This field should not be referenced directly to lookup Voice AI enablement -- use the
    // `is_voice_input_enabled()` getter.
    voice_input_enabled_internal: VoiceInputEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.voice.voice_input_enabled",
        description: "Controls whether voice input is enabled for AI interactions.",
    },
    // The number of times the user has entered Agent Mode.
    // Not a user-visible setting. We model it so we can show the voice input new feature popup
    // the correct number of times.
    entered_agent_mode_num_times: EnteredAgentModeNumTimes {
        type: usize,
        default: 0,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    },
    // Whether or not the user has manually dismissed the voice input new feature popup.
    dismissed_voice_input_new_feature_popup: DismissedVoiceInputNewFeaturePopup {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    },
    // This field is used to store the key used for voice input toggling.
    // Note this is not the named key, but rather corresponds to the physical key.
    voice_input_toggle_key: VoiceInputToggleKey,
    // Preferred spoken language for voice transcription. Stored as an ISO-639-1
    // code (e.g. "es"); an empty string means Auto-detect. Options come from
    // VOICE_INPUT_LANGUAGES.
    voice_input_language: VoiceInputLanguage {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.voice.voice_input_language",
        description: "Preferred spoken language for voice input transcription.",
    },
    // This is not a user-visible setting - it's merely a one-time flag to track if the user has
    // explicitly interacted with voice input. We use this to determine whether we should show a toast
    // to inform the user about voice input and auto-set the keybinding.
    explicitly_interacted_with_voice: ExplicitlyInteractedWithVoice {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::DESKTOP,
        // Never sync to cloud to keep state separate across devices, since microphone access is per-device.
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: true,
    },
    // Predicates that Agent Mode can use to decide if it can execute
    // a command without explicit user consent.
    //
    // Prefer [`BlocklistAIPermissions::can_autoexecute_command`] to
    // interpret this allowlist.
    agent_mode_command_execution_allowlist: AgentModeCommandExecutionAllowlist {
        type: Vec<AgentModeCommandExecutionPredicate>,
        default: DEFAULT_COMMAND_EXECUTION_ALLOWLIST.clone(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::ALL,
        private: false,
        toml_path: "agents.profiles.agent_mode_command_execution_allowlist",
        description: "Commands that the agent can execute without explicit permission.",
    },
    // Predicates that Agent Mode can use to decide if a command must
    // be executed by the user.
    //
    // Prefer [`BlocklistAIPermissions::can_autoexecute_command`] to
    // interpret this denylist.
    agent_mode_command_execution_denylist: AgentModeCommandExecutionDenylist {
        type: Vec<AgentModeCommandExecutionPredicate>,
        default: DEFAULT_COMMAND_EXECUTION_DENYLIST.clone(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::ALL,
        private: false,
        toml_path: "agents.profiles.agent_mode_command_execution_denylist",
        description: "Commands that the agent must always ask before executing.",
    },
    // Enabled iff Agent Mode can execute readonly commands without explicit user consent.
    //
    // Prefer [`BlocklistAIPermissions::can_autoexecute_command`] to
    // interpret this setting.
    agent_mode_execute_read_only_commands: AgentModeExecuteReadonlyCommands {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::ALL,
        private: false,
        toml_path: "agents.profiles.agent_mode_execute_readonly_commands",
        description: "Whether the agent can auto-execute read-only commands without asking.",
    },
    // Determines coding permissions that Agent Mode has.
    // Note that if Agent Mode has permissions to execute readonly commands,
    // that automatically gives Agent Mode the ability to also _read_ files for coding
    // tasks, including codebase search.
    //
    // Prefer [`BlocklistAIPermissions::can_read_file`] to interpret this setting.
    agent_mode_coding_permissions: AgentModeCodingPermissions {
        type: AgentModeCodingPermissionsType,
        default: AgentModeCodingPermissionsType::default(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::ALL,
        private: false,
        toml_path: "agents.profiles.agent_mode_coding_permissions",
        description: "The file read permission level for the agent.",
    }
    // Specific filepaths that Agent Mode can read without asking for additional permissions.
    // These should be persisted as absolute filepaths to avoid ambiguity.
    //
    // This is used in conjunction with [`AgentModeCodingPermissionsType::AllowReadingSpecificFiles`]
    // but modelled as a separate setting because it is not cloud-synced.
    //
    // Prefer [`BlocklistAIPermissions::can_read_file`] to interpret this setting.
    agent_mode_coding_file_read_allowlist: AgentModeCodingFileReadAllowlist {
        type: Vec<PathBuf>,
        default: vec![],
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::ALL,
        private: false,
        toml_path: "agents.profiles.agent_mode_coding_file_read_allowlist",
        description: "File paths the agent can read without asking for permission.",
    }
    // The complete execution-profile collection shared by GUI and TUI.
    // GUI cloud synchronization respects the user's settings-sync preference;
    // TUI settings mode keeps this value local.
    execution_profiles: ExecutionProfiles {
        type: ExecutionProfilesConfig,
        default: ExecutionProfilesConfig::default(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::ALL,
        private: false,
        toml_path: "agents.execution_profiles",
        max_table_depth: 2,
        description: "AI execution profiles and their permissions.",
    }
    // Which unit the TUI footer's usage entry displays (credits or provider
    // cost), flipped by clicking the entry.
    //
    // TUI-only and file-backed so the choice persists across TUI sessions.
    usage_display_mode: TuiUsageDisplayMode,
    // Ordered visibility configuration for the TUI's bottom statusline.
    // TUI-only and local so separate devices can use different terminal layouts.
    tui_statusline: TuiStatusline {
        type: TuiStatuslineConfig,
        default: TuiStatuslineConfig::default(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::TUI,
        private: false,
        toml_path: "agents.statusline",
        description: "Controls the order and visibility of InfiniShell TUI statusline items.",
    },
    // Whether or not the profile-level command autoexecution speedbump has been shown.
    //
    // Not a user-visible setting - we model it as a setting so we can track how often
    // it's shown across devices.
    has_shown_agent_mode_profile_command_autoexecution_speedbump: HasShownAgentModeProfileCommandAutoexecutionSpeedbump {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }
    // Whether or not we should show the speedbump for auto-executing readonly cmds.
    //
    // Not a user-visible settings - we model it as a setting so we can track how often
    // it's shown across devices.
    should_show_agent_mode_autoexecute_readonly_commands_speedbump: ShouldShowAgentModeModelExecuteReadonlyCommandsSpeedbump {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }
    // Whether or not we should show the speedbump for auto-writing to the PTY.
    //
    // Not a user-visible settings - we model it as a setting so we can track how often
    // it's shown across devices.
    should_show_agent_mode_write_to_pty_speedbump: ShouldShowAgentModeWriteToPtySpeedbump {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }
    // Whether or not we should show the speedbump for auto-reading files.
    //
    // Not a user-visible settings - we model it as a setting so we can track how often
    // it's shown across devices.
    should_show_agent_mode_autoread_files_speedbump: ShouldShowAgentModeCodingReadPermissionsNudge {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }
    // Whether or not we should show the one-shot speedbump on Ask-User-Question cards.
    //
    // Not a user-visible setting - we model it as a setting so we can track state.
    // Intentionally NOT cloud-synced: we want users to see the first-time nudge on
    // each fresh device, and we avoid a cloud-sync race that would make the flag
    // silently stay `false` on new devices after being consumed once elsewhere.
    should_show_agent_mode_ask_user_question_speedbump: ShouldShowAgentModeAskUserQuestionSpeedbump {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }
    // Whether to use locally loaded AWS credentials for Bedrock-enabled requests.
    aws_bedrock_credentials_enabled: AwsBedrockCredentialsEnabled {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "cloud_platform.third_party_api_keys.aws_bedrock_credentials_enabled",
        description: "Whether InfiniShell should use your local AWS credentials for Bedrock-enabled requests.",
    }
    // Whether to automatically run the AWS login command when Bedrock credentials are expired.
    //
    // When true, the configured login command will be run automatically without asking.
    // When false (default), a prompt will be shown asking for permission.
    aws_bedrock_auto_login: AwsBedrockAutoLogin {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "cloud_platform.third_party_api_keys.aws_bedrock_auto_login",
        description: "Whether to automatically run the AWS login command when Bedrock credentials expire.",
    }
    // Command to run to refresh AWS credentials when using Bedrock auto-login.
    aws_bedrock_auth_refresh_command: AwsBedrockAuthRefreshCommand {
        type: String,
        default: "aws login".to_string(),
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "cloud_platform.third_party_api_keys.aws_bedrock_auth_refresh_command",
        description: "The command to run to refresh AWS credentials for Bedrock.",
    }
    // AWS profile name to use when loading credentials from the local AWS credential/config chain.
    aws_bedrock_profile: AwsBedrockProfile {
        type: String,
        default: "default".to_string(),
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "cloud_platform.third_party_api_keys.aws_bedrock_profile",
        description: "The AWS profile name to use for Bedrock credentials.",
    }
    // Whether the AWS Bedrock login banner has been permanently dismissed.
    //
    // Not a user-visible setting - we model it as a setting so we can track state.
    aws_bedrock_login_banner_dismissed: AwsBedrockLoginBannerDismissed {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }
    // Whether to mint and attach Gemini Enterprise (GEAP) credentials to eligible agent
    // requests, routing them through the workspace's Google Cloud project. Only consulted
    // when the admin sets the GEAP host to RESPECT_USER_SETTING; ENFORCE bypasses it.
    // Prefer [`UserWorkspaces::is_gemini_enterprise_credentials_enabled`] to interpret
    // this setting.
    gemini_enterprise_credentials_enabled: GeminiEnterpriseCredentialsEnabled {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "cloud_platform.third_party_api_keys.gemini_enterprise_credentials_enabled",
        description: "Whether Warp should route eligible requests through your workspace's Gemini Enterprise Google Cloud project.",
    }
    // Whether or not the user wants agent mode requests to use their saved rules.
    memory_enabled: MemoryEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.knowledge.rules_enabled",
        description: "Whether the agent uses your saved rules during requests.",
    }
    // legacy SSH 会话是否使用每机器记忆。
    ssh_machine_memory_enabled: SshMachineMemoryEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.knowledge.ssh_machine_memory_enabled",
        description: "Whether the agent uses per-machine memory in legacy SSH sessions.",
    }
    // Whether zap drive context should be included in AI requests
    warp_drive_context_enabled: WarpDriveContextEnabled {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.knowledge.warp_drive_context_enabled",
        description: "Whether InfiniShell Drive context is included in AI requests.",
    }

    // Whether the agent mode setup banner has been shown for a given repo path.
    // Once shown, it will not be shown again for that repo.
    //
    // Not a user-visible settings - we model it as a setting so we can track state.
    agent_mode_setup_banner_shown_for_repo_paths: AgentModeSetupBannerShownForRepoPaths {
        type: Vec<PathBuf>,
        default: vec![],
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    // Information about AI request quotas and usage across billing cycles
    ai_request_quota_info: AIRequestQuotaInfoSetting {
        type: AIRequestQuotaInfo,
        default: AIRequestQuotaInfo::default(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    },

    // Whether or not we should show the speedbump for showing code suggestion banners.
    // This includes both passive code diffs and suggested prompts (passive unit tests).
    //
    // Not a user-visible settings - we model it as a setting so we can track if the speedbump has already been shown or not.
    show_code_suggestion_speedbump: ShouldShowCodeSuggestionSpeedbump {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    mcp_execution_path: MCPExecutionPath {
        type: Option<String>,
        default: None,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: true,
    },

    // This is not a user-visible setting - its merely a one-time flag to track if the agents 3 launch modal
    // has been shown to the user.
    //
    // We model it as a setting so it's only shown once to a given user regardless of the number of
    // devices they use.
    did_check_to_trigger_agents_3_launch_modal: DidShowAgents3LaunchModal {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::No),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    // This is not a user-visible setting - it's merely a one-time flag to track if the
    // orchestration launch modal has been shown to the user.
    //
    // We model it as a setting so it's only shown once to a given user regardless of the number of
    // devices they use.
    did_check_to_trigger_orchestration_launch_modal: DidShowOrchestrationLaunchModal {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::No),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    // This is not a user-visible setting - it's merely a one-time flag to track if the
    // InfiniShell TUI launch modal has been shown to the user.
    //
    // We model it as a setting so it's only shown once to a given user regardless of the number of
    // devices they use.
    did_check_to_trigger_agent_cli_launch_modal: DidShowAgentCliLaunchModal {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::No),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    // This is not a user-visible setting - it's merely a one-time flag to track if the
    // free-AI-removal notice modal has been shown to (or silently marked as seen for) the user.
    //
    // We model it as a setting so it's only shown once to a given user regardless of the number of
    // devices they use.
    did_check_to_trigger_free_ai_removal_modal: DidShowFreeAiRemovalModal {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::No),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    // Whether or not the user has enabled fallback to Zap credits for user-provided models.
    can_use_warp_credits_for_fallback: CanUseWarpCreditsForFallback {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::ALL,
        private: false,
        storage_key: "CanUseWarpCreditsWithByok",
        toml_path: "cloud_platform.third_party_api_keys.can_use_warp_credits_with_byok",
        description: "Whether InfiniShell credits can be used as a fallback for user-provided models.",
    }

    should_render_use_agent_footer_for_user_commands: ShouldRenderUseAgentToolbarForUserCommands {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.other.should_render_use_agent_toolbar_for_user_commands",
        description: "Whether to show the \"Use Agent\" footer for terminal commands.",
    }

    // Whether to render the CLI agent footer for commands like Claude, Codex, Gemini, etc.
    // This is independent of the "Use Agent" footer setting.
    should_render_cli_agent_footer: ShouldRenderCLIAgentToolbar {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.third_party.should_render_cli_agent_toolbar",
        description: "Whether to show the CLI agent footer for coding agent commands.",
    }
    // When enabled and a CLI agent session has a plugin listener, rich input
    // auto-closes when the session enters a Blocked state (the agent requires
    // direct keyboard interaction) and auto-opens when it leaves Blocked.
    auto_toggle_rich_input: AutoToggleRichInput {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.third_party.auto_toggle_composer",
        description: "Whether CLI agent Rich Input automatically closes and reopens based on the agent's blocked state.",
    }

    // When enabled and a CLI agent session has a plugin listener, rich input
    // auto-opens once when the session starts or when the listener is registered.
    auto_open_rich_input_on_cli_agent_start: AutoOpenRichInputOnCLIAgentStart {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.third_party.auto_open_composer_on_cli_agent_start",
        description: "Whether CLI agent Rich Input automatically opens when a CLI agent session starts.",
    }

    // When enabled and a CLI agent session does NOT have a plugin listener,
    // rich input auto-closes after the user submits a prompt.
    // When the plugin IS present, this setting has no effect (auto-show/hide
    // from auto_toggle_rich_input handles rich input lifecycle).
    auto_dismiss_rich_input_after_submit: AutoDismissRichInputAfterSubmit {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.third_party.auto_dismiss_composer_after_submit",
        description: "Whether CLI agent Rich Input automatically closes after the user submits a prompt.",
    }

    // When enabled, the Rich Input editor submits on Ctrl+Enter instead of Enter.
    // Enter inserts a newline; Ctrl+Enter submits.
    submit_on_ctrl_enter: SubmitRichInputOnCtrlEnter {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.third_party.submit_on_ctrl_enter",
        description: "When enabled, the Rich Input editor submits on Ctrl+Enter instead of Enter. Enter inserts a newline.",
    }

    // Maps custom toolbar command regex patterns to specific CLI agents.
    // Keys are regex patterns matched against the full command string.
    // Values are serialized CLIAgent names (empty string = any agent).
    // Supports migration from the legacy Vec<String> format.
    cli_agent_footer_enabled_commands: CLIAgentToolbarEnabledCommands {
        type: ToolbarCommandMap,
        default: ToolbarCommandMap::default(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.third_party.cli_agent_toolbar_enabled_commands",
        max_table_depth: 1,
        description: "Maps custom toolbar command patterns to specific CLI agents.",
    }

    // This is not a user-visible setting - it tracks whether a paid user has dismissed the
    // agent management help page by clicking "View Agents".
    //
    // When false and user is on a paid plan, the help page is shown.
    // When true, the help page is hidden (user dismissed it).
    // Free users never see the help page by default regardless of this setting.
    did_dismiss_cloud_setup_guide: DidDismissAgentManagementHelpPage {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    // Whether the ambient agent trial widget has been dismissed by the user.
    //
    // Not a user-visible setting - we model it as a setting so we can track state.
    ambient_agent_trial_widget_dismissed: AmbientAgentTrialWidgetDismissed {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    // The raw stored default mode for new sessions. Use `default_session_mode()` to retrieve the
    // effective value, which is gated on AI availability.
    default_session_mode_internal: DefaultSessionMode,

    // The file path of the tab config used when default_session_mode_internal is TabConfig.
    // Only read when mode is TabConfig; ignored for all other modes.
    // Machine-local (tab config paths vary per machine), so never synced to cloud.
    default_tab_config_path: DefaultTabConfigPath {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "general.default_tab_config_path",
    }

    // Whether computer use is enabled for cloud agent conversations started from the Zap app.
    // This setting is only used when the AI autonomy setting is AlwaysAsk or not set.
    cloud_agent_computer_use_enabled: CloudAgentComputerUseEnabled {
        type: bool,
        default: warp_core::channel::ChannelState::channel().is_dogfood(),
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.other.cloud_agent_computer_use_enabled",
        description: "Whether computer use is enabled for cloud agent conversations.",
    }

    // Whether file-based MCP servers from third-party AI tools (e.g. Claude, Codex) should
    // be automatically detected and spawned. Zap-native config files (.warp/.mcp.json) are
    // always detected and spawned, regardless of this setting.
    file_based_mcp_enabled: FileBasedMcpEnabled {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.mcp_servers.file_based_mcp_enabled",
        description: "Whether third-party file-based MCP servers are automatically detected.",
    }

    // Controls how agent thinking/reasoning traces are displayed.
    thinking_display_mode: ThinkingDisplayMode,

    // Controls how orchestration message bodies are expanded by default.
    orchestration_message_display_mode: OrchestrationMessageDisplayMode,

    // Default behavior when the user submits a new prompt while the agent is still
    // responding. Per-conversation overrides live on `QueuedQueryModel`; this
    // setting is the fallback used when a conversation has no explicit override.
    default_prompt_submission_mode: PromptSubmissionMode,

    // What happens when a prompt is submitted while an agent controls an agent-requested
    // long-running command. Only consulted when `default_prompt_submission_mode` is `Interrupt`;
    // per-LRC manual overrides live on `QueuedQueryModel`.
    long_running_command_submission_mode: LongRunningCommandSubmissionMode,

    // Whether agent-executed shell commands should be included in command history
    // (up-arrow, Ctrl-R search, inline history menu).
    // When false, commands run by the AI agent are excluded from history.
    include_agent_commands_in_history: IncludeAgentCommandsInHistory {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.input.include_agent_commands_in_history",
        description: "Whether agent-executed commands are included in command history.",
    }

    // 控制高权限审批模式是否可运行命中本地命令拒绝列表的命令。
    auto_approve_bypasses_command_denylist: AutoApproveBypassesCommandDenylist {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::ALL,
        private: false,
        toml_path: "agents.warp_agent.other.auto_approve_bypasses_command_denylist",
        description: "Whether Full Access (or legacy auto-approve) bypasses the local command denylist.",
    }

    // Controls whether the conversation history view appears in the tools panel.
    show_conversation_history: ShowConversationHistory {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.other.show_conversation_history",
        description: "Whether conversation history appears in the tools panel.",
    }


    // Controls whether agent notifications (mailbox button, toasts, notification items) are shown.
    show_agent_notifications: ShowAgentNotifications {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.other.show_agent_notifications",
        description: "Whether agent notifications are shown.",
    }

    // Zap T1-2:已完成工具卡默认隐藏(对齐 opencode TUI showDetails 行为)。
    // true → 默认隐藏 status.is_done() 的 RequestCommandOutput / ReadFiles /
    // Grep / FileGlob / RequestFileEdits 等卡片,只保留 in-progress + error,
    // 长 session 不被历史卡片堆积淹没新内容。folded 状态可由外观设置面板切换。
    hide_completed_tool_cards: HideCompletedToolCards {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.appearance.hide_completed_tool_cards",
        description: "When true, completed tool action cards (read files, grep, search codebase, requested commands, etc.) are hidden after they finish. In-progress and errored cards are always shown. Useful for long sessions to keep focus on the latest activity.",
    }

    // Per-agent, per-host tracking of whether the user dismissed the plugin install chip.
    // Keys are "<agent_prefix>" for local sessions or "<agent_prefix>@<host>" for remote.
    // Local-only so dismissal doesn't sync across devices.
    plugin_install_chip_dismissed_map: PluginInstallChipDismissedMap {
        type: HashMap<String, bool>,
        default: HashMap::default(),
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    // Per-agent, per-host tracking of the MINIMUM_PLUGIN_VERSION for which the user
    // dismissed the plugin update chip. Empty/absent means not dismissed.
    // Keys are "<agent_prefix>" for local sessions or "<agent_prefix>@<host>" for remote.
    // Local-only so dismissal doesn't sync across devices.
    plugin_update_chip_dismissed_for_version_map: PluginUpdateChipDismissedForVersionMap {
        type: HashMap<String, String>,
        default: HashMap::default(),
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    // 用户自定义 Agent 提供商列表。第一阶段仅支持 OpenAI 兼容协议。
    //
    // 注意: 提供商的 `api_key` 不在这里持久化,见 `AgentProviderSecrets`。
    agent_providers: AgentProviders {
        type: Vec<AgentProvider>,
        default: Vec::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.providers",
        description: "User-configured custom Agent providers (OpenAI-compatible).",
    }

    // Zap BYOP 本地会话压缩 — 1:1 对齐 opencode `Config.compaction.auto`。
    // true 时按 token-overflow 自动触发摘要;false 仅手动 /compact /compact-and 触发。
    byop_compaction_auto: ByopCompactionAuto {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.byop_compaction.auto",
        description: "Enable BYOP automatic conversation compaction on context overflow.",
    }

    // Zap BYOP 本地会话压缩 — 1:1 对齐 opencode `Config.compaction.prune`。
    // true 时每次 LLM 请求前清旧 tool output(替换为占位符)。
    byop_compaction_prune: ByopCompactionPrune {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.byop_compaction.prune",
        description: "Auto-prune older tool outputs to free BYOP context.",
    }

    // Zap BYOP 本地会话压缩 — 1:1 对齐 opencode `Config.compaction.tail_turns`(默认 2)。
    // 保留最近 N 个 user turn 作 tail,前面的进入 head 给摘要 LLM。0 关闭压缩。
    byop_compaction_tail_turns: ByopCompactionTailTurns {
        type: u32,
        default: 2u32,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.byop_compaction.tail_turns",
        description: "Number of recent user turns to keep verbatim during compaction.",
    }

    // Zap BYOP 本地会话压缩 — 1:1 对齐 `Config.compaction.preserve_recent_tokens`。
    // 0 = 自动按公式算(min(MAX=8000, max(MIN=2000, usable * 0.25)));> 0 强制覆盖。
    byop_compaction_preserve_recent_tokens: ByopCompactionPreserveRecentTokens {
        type: u32,
        default: 0u32,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.byop_compaction.preserve_recent_tokens",
        description: "Override the recent-tokens preservation budget (0 = auto).",
    }

    // Zap BYOP 本地会话压缩 — 1:1 对齐 `Config.compaction.reserved`。
    // overflow 判定时 usable = input_limit - reserved。0 = 自动按 min(20_000, max_output) 算。
    byop_compaction_reserved: ByopCompactionReserved {
        type: u32,
        default: 0u32,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.byop_compaction.reserved",
        description: "Reserved buffer tokens for compaction overflow check (0 = auto).",
    }

    // Zap BYOP 本地会话压缩 — 摘要专用模型(可选)。
    // 设置后:摘要 LLM 调用走这个 provider+model 而非当前 conversation 模型。
    // 留空两个字段 = 用 conversation 当前模型。
    byop_compaction_model_provider_id: ByopCompactionModelProviderId {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.byop_compaction.model.provider_id",
        description: "Optional dedicated provider id for compaction LLM calls.",
    }

    byop_compaction_model_id: ByopCompactionModelId {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.byop_compaction.model.model_id",
        description: "Optional dedicated model id for compaction LLM calls.",
    }

    // Zap BYOP 模型 + 思考深度持久化(picker 切换后立即写入,新 tab/重启沿用)。
    // 模型用 LLMId 字符串形式;空串 = 没有 last_used,落回 profile 默认。
    byop_last_used_model_id: ByopLastUsedModelId {
        type: String,
        default: String::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.byop.last_used_model_id",
        description: "Last selected BYOP model id (picker hydrates new tabs/sessions from this).",
    }

    // Zap BYOP per-(api_type, model) 思考深度记忆。
    // key = `<api_type>:<model_id>`,value = ReasoningEffortSetting。picker 切换写入。
    byop_last_used_reasoning: ByopLastUsedReasoning {
        type: BYOPLastUsedReasoningMap,
        default: BYOPLastUsedReasoningMap::default(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.byop.last_used_reasoning",
        max_table_depth: 1,
        description: "Per-(api_type, model) reasoning effort memory for BYOP picker.",
    }

    // Per-agent 设置：控制单个 CLI agent 的工具栏和标签页菜单可见性。
    // key 是 CLIAgent::to_serialized_name() 的结果。
    cli_agent_per_agent_settings: CLIAgentPerAgentSettings {
        type: HashMap<String, PerAgentSettings>,
        default: HashMap::new(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.third_party.per_agent",
        max_table_depth: 1,
        description: "Per-agent visibility settings for toolbar and tab menu.",
    }

    // 是否已完成至少一次 CLI agent 安装扫描。
    // 首次打开第三方智能体设置页时,若该标记为 false 则自动触发一次同步。
    cli_agent_scan_completed: CLIAgentScanCompleted {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    should_force_disable_cloud_handoff: ShouldForceDisableCloudHandoff {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.other.should_force_disable_cloud_handoff",
        description: "Whether to force-disable local-to-cloud handoff.",
    }

    should_force_disable_ampersand_handoff: ShouldForceDisableAmpersandHandoff {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::DESKTOP,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.other.should_force_disable_ampersand_handoff",
        description: "Whether to force-disable the & prefix for cloud handoff compose mode.",
    }

    auto_handoff_on_sleep_enabled: AutoHandoffOnSleepEnabled {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::MAC,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        surface: settings::SettingSurfaces::GUI,
        private: false,
        toml_path: "agents.warp_agent.other.auto_handoff_on_sleep_enabled",
        description: "Whether Warp automatically hands off local agent conversations to cloud when the computer is about to sleep.",
    }

    // This is not a user-visible setting - it's merely a one-time flag to track if the
    // auto-handoff sleep modal has been shown to the user.
    //
    // We model it as a setting so it's only shown once to a given user regardless of the number of
    // devices they use.
    did_show_auto_handoff_sleep_modal: DidShowAutoHandoffSleepModal {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::No),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }

    // Not a user-visible setting - it tracks which one-time feature-intro popups the
    // user has already seen, keyed by the feature-intro id (see `FEATURE_INTROS`).
    //
    // We model it as a globally-synced setting (not respecting the user's sync setting)
    // so each feature is announced at most once per user, regardless of how many devices
    // they use. A feature is considered seen when its id is present and mapped to `true`.
    seen_feature_intro_ids: SeenFeatureIntroIds {
        type: HashMap<String, bool>,
        default: HashMap::default(),
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::No),
        surface: settings::SettingSurfaces::GUI,
        private: true,
    }
]);

impl AISettings {
    pub fn register_and_subscribe_to_events(app: &mut AppContext) {
        Self::register(app);
        app.add_singleton_model(FocusedTerminalInfo::new);
        CompiledCommandsForCodingAgentToolbar::register(app);

        app.update_model(&Self::handle(app), |_me, ctx| {
            ctx.subscribe_to_model(&FocusedTerminalInfo::handle(ctx), |_me, _, event, ctx| {
                if matches!(event, FocusedTerminalInfoEvent::TerminalInfoUpdated) {
                    // Pipe the event so that any view that listens for settings changes will be notified.
                    ctx.emit(AISettingsChangedEvent::IsAnyAIEnabled {
                        change_event_reason: ChangeEventReason::LocalChange,
                    });
                }
            });
        });
    }

    pub fn is_any_ai_enabled(&self, _app: &AppContext) -> bool {
        // Zap 不再允许通过设置关闭 Zap 智能体。旧配置文件里持久化的
        // `agents.warp_agent.is_any_ai_enabled = false` 会被忽略。
        true
    }

    /// Returns whether conversation history is available for the current
    /// account and AI state.
    ///
    /// The stored `show_conversation_history` preference is kept separately so
    /// an onboarding choice can take effect automatically after signup and AI
    /// enablement without asking the user to toggle the setting again.
    pub fn is_conversation_history_available(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    /// Returns whether conversation history should currently appear in the
    /// tools panel.
    pub fn is_conversation_history_enabled(&self, app: &AppContext) -> bool {
        *self.show_conversation_history && self.is_conversation_history_available(app)
    }

    pub fn default_session_mode(&self, app: &AppContext) -> DefaultSessionMode {
        let mode = *self.default_session_mode_internal.value();
        match mode {
            // Terminal and TabConfig don't require AI.
            DefaultSessionMode::Terminal | DefaultSessionMode::TabConfig => mode,
            // Agent and AmbientAgent require AI to be enabled.
            DefaultSessionMode::Agent | DefaultSessionMode::AmbientAgent => {
                if self.is_any_ai_enabled(app) {
                    mode
                } else {
                    DefaultSessionMode::Terminal
                }
            }
            // DockerSandbox is gated on its feature flag; fall back to Terminal
            // when disabled so a stale stored value doesn't wedge the user.
            DefaultSessionMode::DockerSandbox => {
                if FeatureFlag::LocalDockerSandbox.is_enabled() {
                    mode
                } else {
                    DefaultSessionMode::Terminal
                }
            }
        }
    }

    /// Returns the stored default tab config path (only meaningful when mode is `TabConfig`).
    pub fn default_tab_config_path(&self) -> &str {
        &self.default_tab_config_path
    }

    /// Looks up the `TabConfig` matching the stored `default_tab_config_path`.
    /// Returns `None` if the path is empty or no loaded config matches.
    pub fn resolved_default_tab_config(
        &self,
        app: &AppContext,
    ) -> Option<crate::tab_configs::TabConfig> {
        let path_str = self.default_tab_config_path.as_str();
        if path_str.is_empty() {
            return None;
        }
        let path = std::path::Path::new(path_str);
        crate::user_config::WarpConfig::as_ref(app)
            .tab_configs()
            .iter()
            .find(|config| config.source_path.as_deref().is_some_and(|p| p == path))
            .cloned()
    }

    pub fn is_active_ai_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_any_ai_enabled(app)
            && *self.is_active_ai_enabled_internal
            && AppExecutionMode::as_ref(app).allows_active_ai()
    }

    pub fn is_prompt_suggestions_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_active_ai_enabled(app) && *self.prompt_suggestions_enabled_internal
    }

    pub fn is_rule_suggestions_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_active_ai_enabled(app) && *self.rule_suggestions_enabled_internal
    }

    pub fn is_code_suggestions_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_active_ai_enabled(app) && *self.code_suggestions_enabled_internal
    }

    pub fn is_natural_language_autosuggestions_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_active_ai_enabled(app) && *self.natural_language_autosuggestions_enabled_internal
    }

    pub fn is_git_operations_autogen_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_active_ai_enabled(app) && *self.git_operations_autogen_enabled_internal
    }

    pub fn is_intelligent_autosuggestions_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_active_ai_enabled(app) && *self.intelligent_autosuggestions_enabled_internal
    }

    pub fn is_voice_input_enabled(&self, app: &warpui::AppContext) -> bool {
        // Voice input is conditionally-compiled because it requires additional dependencies on some platforms.
        cfg!(feature = "voice_input")
            && self.is_any_ai_enabled(app)
            && *self.voice_input_enabled_internal
    }

    /// Preferred spoken language for voice transcription, or `None` for auto-detect.
    pub fn voice_input_language_code(&self) -> Option<&str> {
        let code = self.voice_input_language.as_str();
        if code.is_empty() { None } else { Some(code) }
    }

    /// Returns `true` if input autodetection is enabled.
    ///
    /// If `FeatureFlag::AgentView` is enabled, this specifically gates NLD enablement in the agent
    /// view only.
    pub fn is_ai_autodetection_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_any_ai_enabled(app) && *self.ai_autodetection_enabled_internal
    }

    /// Returns `true` if NLD is enabled in the terminal.
    ///
    /// This is only used when `FeatureFlag::AgentView` is enabled.
    /// If the user has not explicitly set this setting, it defaults to the value of
    /// `ai_autodetection_enabled_internal`.
    pub fn is_nld_in_terminal_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_any_ai_enabled(app) && *self.nld_in_terminal_enabled_internal
    }

    pub fn is_memory_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_any_ai_enabled(app) && *self.memory_enabled
    }

    pub fn is_ssh_machine_memory_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_memory_enabled(app) && *self.ssh_machine_memory_enabled
    }

    pub fn is_warp_drive_context_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_any_ai_enabled(app) && *self.warp_drive_context_enabled
    }

    pub fn is_file_based_mcp_enabled(&self, app: &warpui::AppContext) -> bool {
        if !FeatureFlag::FileBasedMcp.is_enabled() || !self.is_any_ai_enabled(app) {
            return false;
        }
        // NOTE: we intentionally do not force-enable this in autonomous agent runs. Previously
        // we auto-spawned file-based MCPs in autonomous execution, but that bypassed
        // the user's explicit opt-in and let any MCP config checked into a repo run
        // arbitrary commands as part of an agent run. Respecting the toggle
        // closes that attack surface; agents that need project-scoped MCP
        // servers should surface an explicit, auditable opt-in. A more robust
        // solution (e.g. per-environment allowlisting, signed configs) should be
        // explored in the future.
        *self.file_based_mcp_enabled
    }

    pub fn is_orchestration_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    /// Returns true when local-to-cloud handoff is effectively enabled.
    /// False when the user has disabled it, or AI is globally off.
    ///
    /// Zap:上游此处还会检查 `PrivacySettings::is_cloud_conversation_storage_enabled`
    /// 以及组织管理员的 `cloud_conversation_storage` 策略。云端对话存储链路在 Zap 中
    /// 已物理切断,且 Zap 没有托管组织策略,这两项检查一并去掉,只保留用户侧开关
    /// 与 feature flag 门控。
    pub fn is_cloud_handoff_enabled(&self, app: &warpui::AppContext) -> bool {
        if !self.is_any_ai_enabled(app) || *self.should_force_disable_cloud_handoff {
            return false;
        }
        FeatureFlag::HandoffLocalCloud.is_enabled()
            && cfg!(all(feature = "local_fs", not(target_family = "wasm")))
    }

    pub fn is_ampersand_handoff_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_cloud_handoff_enabled(app) && !*self.should_force_disable_ampersand_handoff
    }

    pub fn is_auto_handoff_on_sleep_enabled(&self, app: &warpui::AppContext) -> bool {
        self.is_cloud_handoff_enabled(app)
            && self
                .auto_handoff_on_sleep_enabled
                .is_supported_on_current_platform()
            && *self.auto_handoff_on_sleep_enabled
    }

    /// Determines whether a quota reset banner should be displayed to the user.
    ///
    /// The banner should be shown if the most recent completed billing cycle had
    /// quota exceeded and the banner was not manually dismissed.
    pub fn should_display_quota_reset_banner(&self) -> bool {
        let quota_info = &self.ai_request_quota_info;

        let most_recent_completed_cycle = quota_info
            .cycle_history
            .iter()
            .rev()
            .find(|cycle| cycle.end_date < Utc::now());

        if let Some(cycle) = most_recent_completed_cycle
            && cycle.was_quota_exceeded
            && !cycle.banner_state.dismissed
        {
            return true;
        }

        false
    }

    /// Marks the banner as dismissed for all completed cycles.
    pub fn mark_quota_banner_as_dismissed(&mut self, ctx: &mut ModelContext<Self>) {
        let mut cycle_history = self.ai_request_quota_info.cycle_history.clone();

        for cycle in cycle_history.iter_mut() {
            if cycle.end_date < Utc::now() {
                cycle.banner_state.dismissed = true;
            }
        }

        report_if_error!(
            self.ai_request_quota_info
                .set_value(AIRequestQuotaInfo { cycle_history }, ctx)
        );
    }

    /// Updates the quota info based on the latest RequestLimitInfo.
    ///
    /// This method finds or creates the appropriate CycleInfo based on the
    /// request_limit_info's next refresh time and updates its fields accordingly.
    pub fn update_quota_info(
        &mut self,
        request_limit_info: &RequestLimitInfo,
        ctx: &mut ModelContext<Self>,
    ) {
        // Convert ServerTimestamp to DateTime<Utc>
        let next_refresh_time = request_limit_info.next_refresh_time.utc();
        let now = Utc::now();

        // Check if request_limit_info has unlimited requests
        let is_quota_exceeded = !request_limit_info.is_unlimited
            && request_limit_info.num_requests_used_since_refresh >= request_limit_info.limit;

        let mut cycle_history = self.ai_request_quota_info.cycle_history.clone();

        // Track if we updated an existing cycle
        let mut updated_existing_cycle = false;

        // Find or create a cycle that matches the current period
        if let Some(current_cycle) = cycle_history.last_mut()
            && now <= current_cycle.end_date
        {
            // Update existing cycle
            current_cycle.was_quota_exceeded = is_quota_exceeded;
            updated_existing_cycle = true;
        }

        // Only create a new cycle if we didn't update an existing one
        if !updated_existing_cycle {
            // Create a new cycle
            let new_cycle = CycleInfo {
                end_date: next_refresh_time,
                was_quota_exceeded: is_quota_exceeded,
                banner_state: BannerState::default(),
            };

            cycle_history.push(new_cycle);
        }

        report_if_error!(
            self.ai_request_quota_info
                .set_value(AIRequestQuotaInfo { cycle_history }, ctx)
        );
    }

    pub fn is_command_denylist_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_command_allowlist_editable(&self, app: &AppContext) -> bool {
        let set_by_workspace = UserWorkspaces::as_ref(app)
            .ai_autonomy_settings()
            .has_override_for_execute_commands_allowlist();

        self.is_any_ai_enabled(app) && !set_by_workspace
    }

    pub fn is_directory_allowlist_editable(&self, app: &AppContext) -> bool {
        let set_by_workspace = UserWorkspaces::as_ref(app)
            .ai_autonomy_settings()
            .has_override_for_read_files_allowlist();

        self.is_any_ai_enabled(app) && !set_by_workspace
    }

    pub fn is_execute_commands_permissions_editable(&self, app: &AppContext) -> bool {
        let set_by_workspace = UserWorkspaces::as_ref(app)
            .ai_autonomy_settings()
            .has_override_for_execute_commands();

        self.is_any_ai_enabled(app) && !set_by_workspace
    }

    pub fn is_write_to_pty_permissions_editable(&self, app: &AppContext) -> bool {
        let set_by_workspace = UserWorkspaces::as_ref(app)
            .ai_autonomy_settings()
            .has_override_for_write_to_pty();
        self.is_any_ai_enabled(app) && !set_by_workspace
    }

    pub fn is_computer_use_permissions_editable(&self, app: &AppContext) -> bool {
        let set_by_workspace = UserWorkspaces::as_ref(app)
            .ai_autonomy_settings()
            .has_override_for_computer_use();
        self.is_any_ai_enabled(app) && !set_by_workspace
    }

    pub fn is_read_files_permissions_editable(&self, app: &AppContext) -> bool {
        let set_by_workspace = UserWorkspaces::as_ref(app)
            .ai_autonomy_settings()
            .has_override_for_read_files();

        self.is_any_ai_enabled(app) && !set_by_workspace
    }

    pub fn is_code_diffs_permissions_editable(&self, app: &AppContext) -> bool {
        let set_by_workspace = UserWorkspaces::as_ref(app)
            .ai_autonomy_settings()
            .has_override_for_code_diffs();

        self.is_any_ai_enabled(app) && !set_by_workspace
    }

    pub fn is_ask_user_question_permissions_editable(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app)
    }

    pub fn is_mcp_permission_editable(&self, app: &AppContext) -> bool {
        // TODO: Allow workspace overrides on MCP permissions.
        self.is_any_ai_enabled(app)
    }

    pub fn is_run_agents_permissions_editable(&self, app: &AppContext) -> bool {
        self.is_orchestration_enabled(app)
    }

    pub fn show_code_suggestion_speedbump(&self, app: &AppContext) -> bool {
        self.is_any_ai_enabled(app) && *self.show_code_suggestion_speedbump
    }

    /// Handles first-time voice input setup when user clicks the voice button.
    ///
    /// If the user hasn't explicitly interacted with voice yet:
    /// - Sets the default voice input toggle key based on the OS
    /// - Marks `explicitly_interacted_with_voice` as true
    /// - Returns `Some(toggle_key)` so the caller can show a toast
    ///
    /// If the user has already interacted with voice, returns `None`.
    pub fn maybe_setup_first_time_voice(
        &mut self,
        ctx: &mut ModelContext<Self>,
    ) -> Option<VoiceInputToggleKey> {
        if *self.explicitly_interacted_with_voice.value() {
            return None;
        }

        let voice_input_toggle_key = match OperatingSystem::get() {
            OperatingSystem::Mac => VoiceInputToggleKey::Fn,
            OperatingSystem::Windows | OperatingSystem::Linux | OperatingSystem::Other(_) => {
                VoiceInputToggleKey::AltRight
            }
        };

        report_if_error!(
            self.voice_input_toggle_key
                .set_value(voice_input_toggle_key, ctx)
        );

        report_if_error!(self.explicitly_interacted_with_voice.set_value(true, ctx));

        Some(voice_input_toggle_key)
    }

    pub fn add_cli_agent_footer_enabled_command(
        &mut self,
        command: &str,
        ctx: &mut ModelContext<Self>,
    ) {
        let command = command.trim();
        if command.is_empty() {
            return;
        }
        if self
            .cli_agent_footer_enabled_commands
            .value()
            .contains_key(command)
        {
            return;
        }

        let mut map = self.cli_agent_footer_enabled_commands.value().0.clone();
        map.insert(command.to_string(), String::new());
        report_if_error!(
            self.cli_agent_footer_enabled_commands
                .set_value(ToolbarCommandMap::new(map), ctx)
        );
    }

    pub fn remove_cli_agent_footer_enabled_command(
        &mut self,
        command: &str,
        ctx: &mut ModelContext<Self>,
    ) {
        let command = command.trim();
        let mut map = self.cli_agent_footer_enabled_commands.value().0.clone();
        map.shift_remove(command);
        report_if_error!(
            self.cli_agent_footer_enabled_commands
                .set_value(ToolbarCommandMap::new(map), ctx)
        );
    }

    pub fn set_cli_agent_for_command(
        &mut self,
        pattern: &str,
        agent: Option<CLIAgent>,
        ctx: &mut ModelContext<Self>,
    ) {
        let mut map = self.cli_agent_footer_enabled_commands.value().0.clone();
        if !map.contains_key(pattern) {
            return;
        }
        let value = agent.map(|a| a.to_serialized_name()).unwrap_or_default();
        map.insert(pattern.to_string(), value);
        report_if_error!(
            self.cli_agent_footer_enabled_commands
                .set_value(ToolbarCommandMap::new(map), ctx)
        );
    }

    /// Whether the feature-intro popover with the given id key has been seen.
    pub fn is_feature_intro_seen(&self, key: &str) -> bool {
        self.seen_feature_intro_ids
            .get(key)
            .copied()
            .unwrap_or(false)
    }

    /// Records that the feature-intro popover with the given id key has been seen,
    /// so it is never shown again. No-op if already recorded.
    pub fn mark_feature_intro_seen(&mut self, key: &str, ctx: &mut ModelContext<Self>) {
        if self.is_feature_intro_seen(key) {
            return;
        }
        let mut map = self.seen_feature_intro_ids.clone();
        map.insert(key.to_owned(), true);
        report_if_error!(self.seen_feature_intro_ids.set_value(map, ctx));
    }

    /// Whether the plugin install chip was dismissed for the given agent/host.
    pub fn is_plugin_install_chip_dismissed(&self, key: &str) -> bool {
        self.plugin_install_chip_dismissed_map
            .get(key)
            .copied()
            .unwrap_or(false)
    }

    /// Mark the plugin install chip as dismissed for the given agent/host.
    pub fn dismiss_plugin_install_chip(&mut self, key: &str, ctx: &mut ModelContext<Self>) {
        let mut map = self.plugin_install_chip_dismissed_map.clone();
        map.insert(key.to_owned(), true);
        report_if_error!(self.plugin_install_chip_dismissed_map.set_value(map, ctx));
    }

    /// Returns the minimum plugin version for which the update chip was dismissed
    /// for the given agent/host, or an empty string if not dismissed.
    pub fn plugin_update_chip_dismissed_version(&self, key: &str) -> &str {
        self.plugin_update_chip_dismissed_for_version_map
            .get(key)
            .map(String::as_str)
            .unwrap_or("")
    }

    /// Record that the user dismissed the update chip for the given agent/host at
    /// the specified minimum version.
    pub fn dismiss_plugin_update_chip(
        &mut self,
        key: &str,
        version: String,
        ctx: &mut ModelContext<Self>,
    ) {
        let mut map = self.plugin_update_chip_dismissed_for_version_map.clone();
        map.insert(key.to_owned(), version);
        report_if_error!(
            self.plugin_update_chip_dismissed_for_version_map
                .set_value(map, ctx)
        );
    }

    // ── Per-agent settings ──

    /// 查询某个 CLI agent 的工具栏是否启用。未在 per-agent 设置中出现时取 agent 默认值。
    pub fn is_cli_agent_toolbar_enabled(&self, agent: CLIAgent) -> bool {
        if matches!(agent, CLIAgent::Unknown) {
            return true;
        }
        self.cli_agent_per_agent_settings
            .get(agent.to_serialized_name().as_str())
            .map(|s| s.toolbar)
            .unwrap_or_else(|| PerAgentSettings::default_for(agent).toolbar)
    }

    /// 查询某个 CLI agent 是否在新建标签页菜单中显示。未在 per-agent 设置中出现时取 agent 默认值。
    pub fn is_cli_agent_tab_menu_enabled(&self, agent: CLIAgent) -> bool {
        if matches!(agent, CLIAgent::Unknown) {
            return false;
        }
        self.cli_agent_per_agent_settings
            .get(agent.to_serialized_name().as_str())
            .map(|s| s.tabmenu)
            .unwrap_or_else(|| PerAgentSettings::default_for(agent).tabmenu)
    }

    /// 查询某个 CLI agent 的标题栏按钮是否启用。未在 per-agent 设置中出现时取 agent 默认值。
    pub fn is_cli_agent_titlebar_enabled(&self, agent: CLIAgent) -> bool {
        if matches!(agent, CLIAgent::Unknown) {
            return false;
        }
        self.cli_agent_per_agent_settings
            .get(agent.to_serialized_name().as_str())
            .map(|s| s.titlebar)
            .unwrap_or_else(|| PerAgentSettings::default_for(agent).titlebar)
    }

    /// 设置单个 agent 的工具栏启用状态。
    pub fn set_cli_agent_toolbar(
        &mut self,
        agent: CLIAgent,
        enabled: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let key = agent.to_serialized_name();
        let mut map = self.cli_agent_per_agent_settings.clone();
        map.entry(key)
            .and_modify(|s| s.toolbar = enabled)
            .or_insert_with(|| PerAgentSettings {
                toolbar: enabled,
                ..PerAgentSettings::default_for(agent)
            });
        report_if_error!(self.cli_agent_per_agent_settings.set_value(map, ctx));
    }

    /// 设置单个 agent 的标签页菜单启用状态。
    pub fn set_cli_agent_tab_menu(
        &mut self,
        agent: CLIAgent,
        enabled: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let key = agent.to_serialized_name();
        let mut map = self.cli_agent_per_agent_settings.clone();
        map.entry(key)
            .and_modify(|s| s.tabmenu = enabled)
            .or_insert_with(|| PerAgentSettings {
                tabmenu: enabled,
                ..PerAgentSettings::default_for(agent)
            });
        report_if_error!(self.cli_agent_per_agent_settings.set_value(map, ctx));
    }

    /// 设置单个 agent 的标题栏按钮启用状态。
    pub fn set_cli_agent_titlebar(
        &mut self,
        agent: CLIAgent,
        enabled: bool,
        ctx: &mut ModelContext<Self>,
    ) {
        let key = agent.to_serialized_name();
        let mut map = self.cli_agent_per_agent_settings.clone();
        map.entry(key)
            .and_modify(|s| s.titlebar = enabled)
            .or_insert_with(|| PerAgentSettings {
                titlebar: enabled,
                ..PerAgentSettings::default_for(agent)
            });
        report_if_error!(self.cli_agent_per_agent_settings.set_value(map, ctx));
    }

    /// 根据安装扫描结果同步 per-agent 设置。
    /// - 新检测到的 agent 写入默认值(toolbar=true, tabmenu=true)
    /// - 已卸载的 agent 从设置中移除
    /// - 标注扫描完成
    pub fn sync_per_agent_from_scan(
        &mut self,
        installed: &HashMap<CLIAgent, bool>,
        ctx: &mut ModelContext<Self>,
    ) {
        let installed_agents: Vec<CLIAgent> = installed
            .iter()
            .filter(|(a, v)| **v && !matches!(a, CLIAgent::Unknown))
            .map(|(a, _)| *a)
            .collect();
        let installed_names: std::collections::HashSet<String> = installed_agents
            .iter()
            .map(|a| a.to_serialized_name())
            .collect();

        let mut per_agent = self.cli_agent_per_agent_settings.clone();

        for agent in &installed_agents {
            per_agent
                .entry(agent.to_serialized_name())
                .or_insert_with(|| PerAgentSettings::default_for(*agent));
        }

        // 已卸载的 agent → 移除
        per_agent.retain(|name, _| installed_names.contains(name.as_str()));

        let changed = &per_agent != self.cli_agent_per_agent_settings.value();
        if changed {
            report_if_error!(self.cli_agent_per_agent_settings.set_value(per_agent, ctx));
        }

        if !*self.cli_agent_scan_completed.value() {
            report_if_error!(self.cli_agent_scan_completed.set_value(true, ctx));
        }
    }

    /// 返回是否已完成至少一次 CLI agent 安装扫描。
    pub fn is_cli_agent_scan_completed(&self) -> bool {
        *self.cli_agent_scan_completed.value()
    }
}

/// Singleton model that caches compiled regexes for the `cli_agent_footer_enabled_commands`
/// setting. Each entry pairs a compiled regex with the CLI agent it maps to.
pub struct CompiledCommandsForCodingAgentToolbar {
    regexes: Vec<(Regex, CLIAgent)>,
}

impl CompiledCommandsForCodingAgentToolbar {
    fn parse(app: &AppContext) -> Vec<(Regex, CLIAgent)> {
        AISettings::as_ref(app)
            .cli_agent_footer_enabled_commands
            .value()
            .iter()
            .filter_map(|(pattern, agent_name)| {
                let regex = Regex::new(pattern).ok()?;
                let agent = CLIAgent::from_serialized_name(agent_name);
                Some((regex, agent))
            })
            .collect()
    }

    fn register(app: &mut AppContext) {
        let handle = app.add_singleton_model(|ctx| Self {
            regexes: Self::parse(ctx),
        });
        let ai_settings = AISettings::handle(app);
        app.subscribe_to_model(&ai_settings, move |_, event, ctx| {
            if matches!(
                event,
                AISettingsChangedEvent::CLIAgentToolbarEnabledCommands { .. }
            ) {
                let regexes = Self::parse(ctx);
                handle.update(ctx, |me, _| {
                    me.regexes = regexes;
                });
            }
        });
    }

    /// Returns the CLI agent assigned to the first matching pattern, or `None`
    /// if no pattern matches the command.
    pub fn matched_agent(app: &AppContext, command: &str) -> Option<CLIAgent> {
        Self::as_ref(app)
            .regexes
            .iter()
            .find(|(regex, _)| regex.is_match(command))
            .map(|(_, agent)| *agent)
    }
}

impl Entity for CompiledCommandsForCodingAgentToolbar {
    type Event = ();
}

impl SingletonEntity for CompiledCommandsForCodingAgentToolbar {}

#[cfg(test)]
#[path = "ai_tests.rs"]
mod tests;
