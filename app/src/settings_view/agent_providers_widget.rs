//! 自定义 Agent Provider 设置面板 widget。
//!
//! UI 形态:
//! - Sub-header (标题左 + 右上角 `+ 添加提供商` 小按钮) + 简短说明
//! - 每条 provider 一张卡片,卡片内含:
//!   · `Name` / `Base URL` / `API Key` 三个输入框(仅编辑,不自动保存)
//!   · 模型列表区: 表头 `显示名 | 模型 ID`,每行两个输入框 + `×` 删除按钮
//!   · 底部按钮行: `+ 添加模型` `Fetch from API` `保存` `Remove` (provider)
//!
//! **保存行为**: 点"保存"按钮会把表单状态一次性下发到 `AISettings`
//! 与 `AgentProviderSecrets`。输入框失焦/按 Enter 不会保存 —— 这是为了
//! 避免用户边改边被“隐式提交”。会重建页面的结构性操作(添加/删除模型行、
//! 添加/删除 header 行、API 协议 chip、模型能力 chip)会先提交当前卡片草稿,
//! 再执行原操作,避免重建时丢失未保存输入。
//!
//! 当 provider 列表大小或某条 provider 的 models 数量变化时,
//! `AISettingsPageView::rebuild_current_page` 会被触发以重建整个 widget,
//! 从而让新增/删除的条目获得自己的 EditorView handle。
//! `rebuild_current_page` 内部会复用旧 PageType 的 vertical scroll handle,
//! 滚动位置不会被重置。
//!
//! provider 元数据(name/base_url/models) 走 `settings.toml`,
//! `api_key` 走 OS keychain (`AgentProviderSecrets`)。

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use settings::Setting;
use strum::IntoEnumIterator;
use warp_core::features::FeatureFlag;
use warpui::elements::{
    ChildView, Container, CornerRadius, CrossAxisAlignment, Expanded, Flex, MainAxisAlignment,
    MouseStateHandle, ParentElement, Radius, Text, Wrap,
};
use warpui::ui_components::button::ButtonVariant;
use warpui::ui_components::components::{Coords, UiComponent, UiComponentStyles};
use warpui::{AppContext, Element, SingletonEntity, ViewContext, ViewHandle};

use super::ai_page::{AISettingsPageAction, AISettingsPageView, ModelCapabilityKind};
use super::settings_page::{HEADER_PADDING, SettingsWidget, build_sub_header};
use crate::ai::agent_providers::AgentProviderSecrets;
use crate::appearance::Appearance;
use crate::editor::{
    EditorView, Event as EditorEvent, SingleLineEditorOptions, TextColors, TextOptions,
};
use crate::settings::{
    AISettings, AgentProvider, AgentProviderApiType, AgentProviderModel, ResponsesStateModeSetting,
    ResponsesTransportSetting,
};

const CARD_BUTTON_PADDING: f32 = 6.0;
const FIELD_LABEL_MARGIN_TOP: f32 = 6.0;
const FIELD_LABEL_MARGIN_BOTTOM: f32 = 2.0;
const MODEL_ROW_GAP: f32 = 6.0;

// ---------------------------------------------------------------------------
// 模型行展开状态(process-local,thread_local 单线程 UI 安全;不持久化)
// ---------------------------------------------------------------------------

std::thread_local! {
    /// {provider_id => Set<model_index>} 当前展开的模型条目。
    static EXPANDED_MODELS: RefCell<HashMap<String, HashSet<usize>>> = RefCell::new(HashMap::new());
}

pub(super) fn is_model_expanded(provider_id: &str, model_index: usize) -> bool {
    EXPANDED_MODELS.with(|m| {
        m.borrow()
            .get(provider_id)
            .is_some_and(|set| set.contains(&model_index))
    })
}

pub(super) fn toggle_model_expanded(provider_id: &str, model_index: usize) {
    EXPANDED_MODELS.with(|m| {
        let mut map = m.borrow_mut();
        let set = map.entry(provider_id.to_string()).or_default();
        if !set.insert(model_index) {
            set.remove(&model_index);
        }
    });
}

/// 删除 provider 时连带清掉它的展开记录,避免索引漂移。
pub(super) fn clear_expanded_models_for_provider(provider_id: &str) {
    EXPANDED_MODELS.with(|m| {
        m.borrow_mut().remove(provider_id);
    });
}

/// 一条模型条目(name + id + context + output)的可编辑 view handle。
struct ModelRow {
    name_editor: ViewHandle<EditorView>,
    id_editor: ViewHandle<EditorView>,
    context_editor: ViewHandle<EditorView>,
    output_editor: ViewHandle<EditorView>,
    /// detail panel 内的删除按钮。
    remove_button_state: MouseStateHandle,
    /// row 末尾 chevron 右侧的快速删除按钮。
    quick_remove_button_state: MouseStateHandle,
    /// row 末尾的展开/折叠 chevron。
    expand_button_state: MouseStateHandle,
    /// detail panel 内 image/pdf/audio 三态 chip 的鼠标状态。
    image_chip_state: MouseStateHandle,
    pdf_chip_state: MouseStateHandle,
    audio_chip_state: MouseStateHandle,
    /// detail panel 内 reasoning / tool_call 两个 bool toggle 的状态。
    reasoning_chip_state: MouseStateHandle,
    tool_call_chip_state: MouseStateHandle,
}

struct HeaderRow {
    key_editor: ViewHandle<EditorView>,
    val_editor: ViewHandle<EditorView>,
    remove_button_state: MouseStateHandle,
}

/// 一条 provider 行的所有可编辑 view handle。
struct ProviderRow {
    name_editor: ViewHandle<EditorView>,
    base_url_editor: ViewHandle<EditorView>,
    api_key_editor: ViewHandle<EditorView>,
    fetch_button_state: MouseStateHandle,
    sync_models_dev_button_state: MouseStateHandle,
    save_button_state: MouseStateHandle,
    remove_button_state: MouseStateHandle,
    add_model_button_state: MouseStateHandle,
    header_rows: Vec<HeaderRow>,
    add_header_button_state: MouseStateHandle,
    /// 5 个 ApiType chip 各自的鼠标状态。HashMap 由 chip 显示名映射。
    api_type_chip_states: RefCell<HashMap<AgentProviderApiType, MouseStateHandle>>,
    responses_state_chip_states: RefCell<HashMap<ResponsesStateModeSetting, MouseStateHandle>>,
    responses_transport_chip_states: RefCell<HashMap<ResponsesTransportSetting, MouseStateHandle>>,
    responses_compaction_chip_states: RefCell<HashMap<u32, MouseStateHandle>>,
    responses_background_chip_state: MouseStateHandle,
    responses_ptc_chip_state: MouseStateHandle,
    responses_reasoning_pro_chip_state: MouseStateHandle,
    responses_reasoning_context_chip_state: MouseStateHandle,
    responses_multi_agent_chip_state: MouseStateHandle,
    model_rows: Vec<ModelRow>,
}

type ModelDraftEditorHandles = (
    usize,
    ViewHandle<EditorView>,
    ViewHandle<EditorView>,
    ViewHandle<EditorView>,
    ViewHandle<EditorView>,
);

#[derive(Clone)]
struct ProviderDraftEditors {
    provider_id: String,
    name_editor: ViewHandle<EditorView>,
    base_url_editor: ViewHandle<EditorView>,
    api_key_editor: ViewHandle<EditorView>,
    header_editors: Vec<(ViewHandle<EditorView>, ViewHandle<EditorView>)>,
    model_editors: Vec<ModelDraftEditorHandles>,
}

impl ProviderDraftEditors {
    fn from_row(provider_id: String, row: &ProviderRow) -> Self {
        Self {
            provider_id,
            name_editor: row.name_editor.clone(),
            base_url_editor: row.base_url_editor.clone(),
            api_key_editor: row.api_key_editor.clone(),
            header_editors: row
                .header_rows
                .iter()
                .map(|h| (h.key_editor.clone(), h.val_editor.clone()))
                .collect(),
            model_editors: row
                .model_rows
                .iter()
                .enumerate()
                .map(|(idx, m)| {
                    (
                        idx,
                        m.name_editor.clone(),
                        m.id_editor.clone(),
                        m.context_editor.clone(),
                        m.output_editor.clone(),
                    )
                })
                .collect(),
        }
    }

    fn to_save_action(&self, app: &AppContext) -> AISettingsPageAction {
        self.to_save_action_with(
            app,
            |provider_id, name, base_url, api_key, headers, models| {
                AISettingsPageAction::SaveAgentProviderEdits {
                    provider_id,
                    name,
                    base_url,
                    api_key,
                    headers,
                    models,
                }
            },
        )
    }

    fn to_save_then_action(
        &self,
        app: &AppContext,
        action: AISettingsPageAction,
    ) -> AISettingsPageAction {
        self.to_save_action_with(
            app,
            |provider_id, name, base_url, api_key, headers, models| {
                AISettingsPageAction::SaveAgentProviderEditsThen {
                    provider_id,
                    name,
                    base_url,
                    api_key,
                    headers,
                    models,
                    action: Box::new(action),
                }
            },
        )
    }

    fn to_save_action_with(
        &self,
        app: &AppContext,
        build: impl FnOnce(
            String,
            String,
            String,
            String,
            Vec<(String, String)>,
            Vec<(usize, String, String, u32, u32)>,
        ) -> AISettingsPageAction,
    ) -> AISettingsPageAction {
        let name = self.name_editor.as_ref(app).buffer_text(app);
        let base_url = self.base_url_editor.as_ref(app).buffer_text(app);
        let api_key = self.api_key_editor.as_ref(app).buffer_text(app);
        let headers: Vec<(String, String)> = self
            .header_editors
            .iter()
            .map(|(k, v)| {
                (
                    k.as_ref(app).buffer_text(app),
                    v.as_ref(app).buffer_text(app),
                )
            })
            .collect();
        let models: Vec<(usize, String, String, u32, u32)> = self
            .model_editors
            .iter()
            .map(|(idx, name_e, id_e, ctx_e, out_e)| {
                let m_name = name_e.as_ref(app).buffer_text(app);
                let m_id = id_e.as_ref(app).buffer_text(app);
                let context_window = parse_token_count(&ctx_e.as_ref(app).buffer_text(app));
                let max_output_tokens = parse_token_count(&out_e.as_ref(app).buffer_text(app));
                (*idx, m_name, m_id, context_window, max_output_tokens)
            })
            .collect();

        build(
            self.provider_id.clone(),
            name,
            base_url,
            api_key,
            headers,
            models,
        )
    }
}

/// 自定义 Agent Provider 设置 widget。
pub(super) struct AgentProvidersWidget {
    add_button_state: MouseStateHandle,
    rows: RefCell<HashMap<String, ProviderRow>>,
}

impl AgentProvidersWidget {
    pub(super) fn new(ctx: &mut ViewContext<AISettingsPageView>) -> Self {
        let providers = AISettings::as_ref(ctx).agent_providers.value().clone();
        let mut rows = HashMap::with_capacity(providers.len());
        for provider in &providers {
            let row = Self::build_row(provider, ctx);
            rows.insert(provider.id.clone(), row);
        }

        // 进入页面即触发一次目录加载(磁盘缓存 + 必要时网络)。
        ctx.dispatch_typed_action_deferred(AISettingsPageAction::EnsureModelsDevLoaded);

        Self {
            add_button_state: MouseStateHandle::default(),
            rows: RefCell::new(rows),
        }
    }

    /// 构造单条模型行的 EditorView 与订阅。
    fn build_model_row(
        model: &AgentProviderModel,
        ctx: &mut ViewContext<AISettingsPageView>,
    ) -> ModelRow {
        // ---- name 编辑器 ----
        let initial_name = model.name.clone();
        let name_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::handle(ctx).as_ref(ctx);
            let options = single_line_editor_options(appearance, false);
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text(
                crate::t!("settings-agent-providers-model-name-placeholder"),
                ctx,
            );
            if !initial_name.is_empty() {
                editor.set_buffer_text(&initial_name, ctx);
            }
            editor
        });
        // 仅负责失焦时收拢选区；不再隐式保存，保存走底部“保存”按钮。
        ctx.subscribe_to_view(&name_editor, move |_, editor, event, ctx| {
            collapse_selection_if_blurred(&editor, event, ctx);
        });

        // ---- id 编辑器 ----
        let initial_id = model.id.clone();
        let id_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::handle(ctx).as_ref(ctx);
            let options = single_line_editor_options(appearance, false);
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text(
                crate::t!("settings-agent-providers-model-id-placeholder"),
                ctx,
            );
            if !initial_id.is_empty() {
                editor.set_buffer_text(&initial_id, ctx);
            }
            editor
        });
        ctx.subscribe_to_view(&id_editor, move |_, editor, event, ctx| {
            collapse_selection_if_blurred(&editor, event, ctx);
        });

        // ---- context_window 编辑器(数字,空 = 0 = 未指定) ----
        let initial_context = if model.context_window == 0 {
            String::new()
        } else {
            model.context_window.to_string()
        };
        let context_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::handle(ctx).as_ref(ctx);
            let options = single_line_editor_options(appearance, false);
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text(
                crate::t!("settings-agent-providers-model-context-placeholder"),
                ctx,
            );
            if !initial_context.is_empty() {
                editor.set_buffer_text(&initial_context, ctx);
            }
            editor
        });
        ctx.subscribe_to_view(&context_editor, move |_, editor, event, ctx| {
            collapse_selection_if_blurred(&editor, event, ctx);
        });

        // ---- max_output_tokens 编辑器 ----
        let initial_output = if model.max_output_tokens == 0 {
            String::new()
        } else {
            model.max_output_tokens.to_string()
        };
        let output_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::handle(ctx).as_ref(ctx);
            let options = single_line_editor_options(appearance, false);
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text(
                crate::t!("settings-agent-providers-model-output-placeholder"),
                ctx,
            );
            if !initial_output.is_empty() {
                editor.set_buffer_text(&initial_output, ctx);
            }
            editor
        });
        ctx.subscribe_to_view(&output_editor, move |_, editor, event, ctx| {
            collapse_selection_if_blurred(&editor, event, ctx);
        });

        ModelRow {
            name_editor,
            id_editor,
            context_editor,
            output_editor,
            remove_button_state: MouseStateHandle::default(),
            quick_remove_button_state: MouseStateHandle::default(),
            expand_button_state: MouseStateHandle::default(),
            image_chip_state: MouseStateHandle::default(),
            pdf_chip_state: MouseStateHandle::default(),
            audio_chip_state: MouseStateHandle::default(),
            reasoning_chip_state: MouseStateHandle::default(),
            tool_call_chip_state: MouseStateHandle::default(),
        }
    }

    fn build_header_row(
        key: &str,
        value: &str,
        ctx: &mut ViewContext<AISettingsPageView>,
    ) -> HeaderRow {
        let initial_key = key.to_owned();
        let key_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::handle(ctx).as_ref(ctx);
            let options = single_line_editor_options(appearance, false);
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text("x-portkey-provider", ctx);
            if !initial_key.is_empty() {
                editor.set_buffer_text(&initial_key, ctx);
            }
            editor
        });

        let initial_value = value.to_owned();
        let val_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::handle(ctx).as_ref(ctx);
            let options = single_line_editor_options(appearance, false);
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text("openai", ctx);
            if !initial_value.is_empty() {
                editor.set_buffer_text(&initial_value, ctx);
            }
            editor
        });

        // header 行的保存同样走底部"保存"按钮；这里仅负责失焦选区收拢。
        // （header_index / provider_id / val_editor 仍会在 build_row 里作为 `HeaderRow` 现场读取。）
        ctx.subscribe_to_view(&key_editor, move |_, editor, event, ctx| {
            collapse_selection_if_blurred(&editor, event, ctx);
        });

        ctx.subscribe_to_view(&val_editor, move |_, editor, event, ctx| {
            collapse_selection_if_blurred(&editor, event, ctx);
        });

        HeaderRow {
            key_editor,
            val_editor,
            remove_button_state: MouseStateHandle::default(),
        }
    }

    /// 为一条 provider 构造它的所有 view handle 与按钮 mouse state。
    fn build_row(
        provider: &AgentProvider,
        ctx: &mut ViewContext<AISettingsPageView>,
    ) -> ProviderRow {
        let provider_id = provider.id.clone();

        // ---- Name 编辑器 ----
        let initial_name = provider.name.clone();
        let name_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::handle(ctx).as_ref(ctx);
            let options = single_line_editor_options(appearance, false);
            let mut editor = EditorView::single_line(options, ctx);
            editor
                .set_placeholder_text(crate::t!("settings-agent-providers-name-placeholder"), ctx);
            if !initial_name.is_empty() {
                editor.set_buffer_text(&initial_name, ctx);
            }
            editor
        });
        // 仅负责失焦选区收拢；保存走底部“保存”按钮。
        ctx.subscribe_to_view(&name_editor, move |_, editor, event, ctx| {
            collapse_selection_if_blurred(&editor, event, ctx);
        });

        // ---- Base URL 编辑器 ----
        let initial_base_url = provider.base_url.clone();
        let base_url_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::handle(ctx).as_ref(ctx);
            let options = single_line_editor_options(appearance, false);
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text(
                crate::t!("settings-agent-providers-base-url-placeholder"),
                ctx,
            );
            if !initial_base_url.is_empty() {
                editor.set_buffer_text(&initial_base_url, ctx);
            }
            editor
        });
        ctx.subscribe_to_view(&base_url_editor, move |_, editor, event, ctx| {
            collapse_selection_if_blurred(&editor, event, ctx);
        });

        // ---- API Key 编辑器(密码模式) ----
        let initial_api_key = AgentProviderSecrets::as_ref(ctx)
            .get(&provider_id)
            .map(str::to_owned)
            .unwrap_or_default();
        let api_key_editor = ctx.add_typed_action_view(move |ctx| {
            let appearance = Appearance::handle(ctx).as_ref(ctx);
            let options = single_line_editor_options(appearance, true);
            let mut editor = EditorView::single_line(options, ctx);
            editor.set_placeholder_text(
                crate::t!("settings-agent-providers-api-key-placeholder"),
                ctx,
            );
            if !initial_api_key.is_empty() {
                editor.set_buffer_text(&initial_api_key, ctx);
            }
            editor
        });
        ctx.subscribe_to_view(&api_key_editor, move |_, editor, event, ctx| {
            collapse_selection_if_blurred(&editor, event, ctx);
        });

        // ---- 模型行 ----
        let model_rows: Vec<ModelRow> = provider
            .models
            .iter()
            .map(|m| Self::build_model_row(m, ctx))
            .collect();

        let header_rows: Vec<HeaderRow> = provider
            .extra_headers
            .iter()
            .map(|(k, v)| Self::build_header_row(k, v, ctx))
            .collect();
        let add_header_button_state = MouseStateHandle::default();

        ProviderRow {
            name_editor,
            base_url_editor,
            api_key_editor,
            fetch_button_state: MouseStateHandle::default(),
            sync_models_dev_button_state: MouseStateHandle::default(),
            save_button_state: MouseStateHandle::default(),
            remove_button_state: MouseStateHandle::default(),
            add_model_button_state: MouseStateHandle::default(),
            header_rows,
            add_header_button_state,
            api_type_chip_states: RefCell::new(HashMap::new()),
            responses_state_chip_states: RefCell::new(HashMap::new()),
            responses_transport_chip_states: RefCell::new(HashMap::new()),
            responses_compaction_chip_states: RefCell::new(HashMap::new()),
            responses_background_chip_state: MouseStateHandle::default(),
            responses_ptc_chip_state: MouseStateHandle::default(),
            responses_reasoning_pro_chip_state: MouseStateHandle::default(),
            responses_reasoning_context_chip_state: MouseStateHandle::default(),
            responses_multi_agent_chip_state: MouseStateHandle::default(),
            model_rows,
        }
    }

    /// 渲染 "API Type" 行:5 个 chip 横排,当前选中的高亮显示。
    /// 点击 chip 即 dispatch `SetAgentProviderApiType`,后端会顺手填默认 endpoint。
    fn render_api_type_field(
        &self,
        provider: &AgentProvider,
        row: &ProviderRow,
        draft_editors: ProviderDraftEditors,
        label_color: warp_core::ui::theme::Fill,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let label_text = Container::new(
            Text::new(
                crate::t!("settings-agent-providers-field-api-type"),
                appearance.ui_font_family(),
                appearance.ui_font_size(),
            )
            .with_color(label_color.into())
            .finish(),
        )
        .with_margin_top(FIELD_LABEL_MARGIN_TOP)
        .with_margin_bottom(FIELD_LABEL_MARGIN_BOTTOM)
        .finish();

        let mut chip_row = Flex::row().with_cross_axis_alignment(CrossAxisAlignment::Center);
        {
            let mut states = row.api_type_chip_states.borrow_mut();
            for variant in AgentProviderApiType::iter() {
                let state = states.entry(variant).or_default().clone();
                let is_selected = provider.api_type == variant;
                let label = if is_selected {
                    format!("● {}", variant.display_name())
                } else {
                    variant.display_name().to_owned()
                };
                let chip = Self::render_card_button_preserving_draft(
                    label,
                    state,
                    draft_editors.clone(),
                    AISettingsPageAction::SetAgentProviderApiType {
                        provider_id: provider.id.clone(),
                        api_type: variant,
                    },
                    appearance,
                );
                chip_row = chip_row.with_child(Container::new(chip).with_margin_right(6.).finish());
            }
        }

        let hint_text = Container::new(
            Text::new(
                crate::t!(
                    "settings-agent-providers-api-type-hint",
                    url = provider.api_type.default_base_url()
                ),
                appearance.ui_font_family(),
                appearance.ui_font_size(),
            )
            .with_color(appearance.theme().disabled_ui_text_color().into())
            .soft_wrap(true)
            .finish(),
        )
        .with_margin_top(2.)
        .finish();

        Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
            .with_child(label_text)
            .with_child(chip_row.finish())
            .with_child(hint_text)
            .finish()
    }

    fn render_card_button(
        label: impl Into<String>,
        mouse_state: MouseStateHandle,
        action: AISettingsPageAction,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        appearance
            .ui_builder()
            .button(ButtonVariant::Secondary, mouse_state)
            .with_style(UiComponentStyles {
                font_size: Some(appearance.ui_font_body()),
                padding: Some(Coords::uniform(CARD_BUTTON_PADDING)),
                ..Default::default()
            })
            .with_centered_text_label(label.into())
            .build()
            .on_click(move |ctx, _, _| {
                ctx.dispatch_typed_action(action.clone());
            })
            .finish()
    }

    fn render_responses_options(
        &self,
        provider: &AgentProvider,
        row: &ProviderRow,
        draft_editors: ProviderDraftEditors,
        label_color: warp_core::ui::theme::Fill,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let label = |text: &str| {
            Container::new(
                Text::new(
                    text.to_owned(),
                    appearance.ui_font_family(),
                    appearance.ui_font_size(),
                )
                .with_color(label_color.into())
                .finish(),
            )
            .with_margin_top(FIELD_LABEL_MARGIN_TOP)
            .with_margin_bottom(FIELD_LABEL_MARGIN_BOTTOM)
            .finish()
        };
        let selected_label = |selected: bool, text: &str| {
            if selected {
                format!("● {text}")
            } else {
                text.to_owned()
            }
        };

        let mut state_row = Flex::row().with_cross_axis_alignment(CrossAxisAlignment::Center);
        {
            let mut states = row.responses_state_chip_states.borrow_mut();
            for state_mode in ResponsesStateModeSetting::iter() {
                let display = match state_mode {
                    ResponsesStateModeSetting::LocalReplay => {
                        crate::t!("settings-agent-providers-responses-state-local").to_string()
                    }
                    ResponsesStateModeSetting::PreviousResponse => {
                        crate::t!("settings-agent-providers-responses-state-provider-chain")
                            .to_string()
                    }
                    ResponsesStateModeSetting::Conversation => {
                        crate::t!("settings-agent-providers-responses-state-cloud-conversation")
                            .to_string()
                    }
                };
                let button = Self::render_card_button_preserving_draft(
                    selected_label(provider.responses.state_mode == state_mode, &display),
                    states.entry(state_mode).or_default().clone(),
                    draft_editors.clone(),
                    AISettingsPageAction::SetAgentProviderResponsesStateMode {
                        provider_id: provider.id.clone(),
                        state_mode,
                    },
                    appearance,
                );
                state_row =
                    state_row.with_child(Container::new(button).with_margin_right(6.).finish());
            }
        }

        let mut transport_row = Flex::row().with_cross_axis_alignment(CrossAxisAlignment::Center);
        {
            let mut states = row.responses_transport_chip_states.borrow_mut();
            for transport in ResponsesTransportSetting::iter() {
                let display = match transport {
                    ResponsesTransportSetting::Http => "HTTP + SSE",
                    ResponsesTransportSetting::WebSocket => "WebSocket",
                };
                let button = Self::render_card_button_preserving_draft(
                    selected_label(provider.responses.transport == transport, display),
                    states.entry(transport).or_default().clone(),
                    draft_editors.clone(),
                    AISettingsPageAction::SetAgentProviderResponsesTransport {
                        provider_id: provider.id.clone(),
                        transport,
                    },
                    appearance,
                );
                transport_row =
                    transport_row.with_child(Container::new(button).with_margin_right(6.).finish());
            }
        }

        let mut compaction_row = Flex::row().with_cross_axis_alignment(CrossAxisAlignment::Center);
        {
            let mut states = row.responses_compaction_chip_states.borrow_mut();
            for (threshold, display) in [
                (0, crate::t!("common-off")),
                (32_000, "32k".to_string()),
                (64_000, "64k".to_string()),
                (128_000, "128k".to_string()),
            ] {
                let button = Self::render_card_button_preserving_draft(
                    selected_label(provider.responses.compact_threshold == threshold, &display),
                    states.entry(threshold).or_default().clone(),
                    draft_editors.clone(),
                    AISettingsPageAction::SetAgentProviderResponsesCompactThreshold {
                        provider_id: provider.id.clone(),
                        compact_threshold: threshold,
                    },
                    appearance,
                );
                compaction_row = compaction_row
                    .with_child(Container::new(button).with_margin_right(6.).finish());
            }
        }

        let background_enabled = provider.responses.state_mode
            != ResponsesStateModeSetting::LocalReplay
            && provider.responses.transport == ResponsesTransportSetting::Http;
        let background_text = if background_enabled {
            selected_label(
                provider.responses.background,
                &crate::t!("settings-agent-providers-responses-background"),
            )
        } else {
            crate::t!("settings-agent-providers-responses-background-requires-cloud").to_string()
        };
        let background = Self::render_card_button_preserving_draft(
            background_text,
            row.responses_background_chip_state.clone(),
            draft_editors.clone(),
            AISettingsPageAction::ToggleAgentProviderResponsesBackground {
                provider_id: provider.id.clone(),
            },
            appearance,
        );
        let ptc = Self::render_card_button_preserving_draft(
            selected_label(
                provider.responses.programmatic_tool_calling,
                &crate::t!("settings-agent-providers-responses-ptc"),
            ),
            row.responses_ptc_chip_state.clone(),
            draft_editors.clone(),
            AISettingsPageAction::ToggleAgentProviderResponsesProgrammaticToolCalling {
                provider_id: provider.id.clone(),
            },
            appearance,
        );
        let mut capabilities_row = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(Container::new(background).with_margin_right(6.).finish())
            .with_child(Container::new(ptc).with_margin_right(6.).finish());
        if FeatureFlag::ResponsesMultiAgentBeta.is_enabled() {
            capabilities_row =
                capabilities_row.with_child(Self::render_card_button_preserving_draft(
                    selected_label(
                        provider.responses.multi_agent_beta,
                        &crate::t!("settings-agent-providers-responses-multi-agent"),
                    ),
                    row.responses_multi_agent_chip_state.clone(),
                    draft_editors.clone(),
                    AISettingsPageAction::ToggleAgentProviderResponsesMultiAgentBeta {
                        provider_id: provider.id.clone(),
                    },
                    appearance,
                ));
        }
        let reasoning_pro = Self::render_card_button_preserving_draft(
            selected_label(
                provider.responses.reasoning_pro_mode,
                &crate::t!("settings-agent-providers-responses-reasoning-pro"),
            ),
            row.responses_reasoning_pro_chip_state.clone(),
            draft_editors.clone(),
            AISettingsPageAction::ToggleAgentProviderResponsesReasoningProMode {
                provider_id: provider.id.clone(),
            },
            appearance,
        );
        let reasoning_context = Self::render_card_button_preserving_draft(
            selected_label(
                provider.responses.reasoning_all_turns,
                &crate::t!("settings-agent-providers-responses-reasoning-all-turns"),
            ),
            row.responses_reasoning_context_chip_state.clone(),
            draft_editors,
            AISettingsPageAction::ToggleAgentProviderResponsesReasoningAllTurns {
                provider_id: provider.id.clone(),
            },
            appearance,
        );
        let reasoning_row = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(Container::new(reasoning_pro).with_margin_right(6.).finish())
            .with_child(reasoning_context)
            .finish();

        let hint = Container::new(
            Text::new(
                crate::t!("settings-agent-providers-responses-privacy-hint"),
                appearance.ui_font_family(),
                appearance.ui_font_size(),
            )
            .with_color(appearance.theme().disabled_ui_text_color().into())
            .soft_wrap(true)
            .finish(),
        )
        .with_margin_top(4.)
        .finish();

        Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
            .with_child(label(&crate::t!(
                "settings-agent-providers-responses-state-label"
            )))
            .with_child(state_row.finish())
            .with_child(label(&crate::t!(
                "settings-agent-providers-responses-transport-label"
            )))
            .with_child(transport_row.finish())
            .with_child(label(&crate::t!(
                "settings-agent-providers-responses-compaction-label"
            )))
            .with_child(compaction_row.finish())
            .with_child(label(&crate::t!(
                "settings-agent-providers-responses-reasoning-label"
            )))
            .with_child(reasoning_row)
            .with_child(label(&crate::t!(
                "settings-agent-providers-responses-capabilities-label"
            )))
            .with_child(capabilities_row.finish())
            .with_child(hint)
            .finish()
    }

    fn render_card_button_preserving_draft(
        label: impl Into<String>,
        mouse_state: MouseStateHandle,
        draft_editors: ProviderDraftEditors,
        action: AISettingsPageAction,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        appearance
            .ui_builder()
            .button(ButtonVariant::Secondary, mouse_state)
            .with_style(UiComponentStyles {
                font_size: Some(appearance.ui_font_body()),
                padding: Some(Coords::uniform(CARD_BUTTON_PADDING)),
                ..Default::default()
            })
            .with_centered_text_label(label.into())
            .build()
            .on_click(move |ctx, app, _| {
                ctx.dispatch_typed_action(draft_editors.to_save_then_action(app, action.clone()));
            })
            .finish()
    }

    fn render_model_row(
        provider: &AgentProvider,
        index: usize,
        model: &AgentProviderModel,
        row: &ModelRow,
        draft_editors: ProviderDraftEditors,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let provider_id = provider.id.as_str();
        let is_expanded = is_model_expanded(provider_id, index);

        // chevron:展开 ▾ / 折叠 ▸。复用 render_card_button 的视觉风格。
        let chevron_label = if is_expanded { "▾" } else { "▸" };
        let chevron_button = Self::render_card_button_preserving_draft(
            chevron_label,
            row.expand_button_state.clone(),
            draft_editors.clone(),
            AISettingsPageAction::ToggleAgentProviderModelExpanded {
                provider_id: provider.id.clone(),
                model_index: index,
            },
            appearance,
        );
        let quick_remove_button = Self::render_card_button_preserving_draft(
            "×",
            row.quick_remove_button_state.clone(),
            draft_editors.clone(),
            AISettingsPageAction::RemoveAgentProviderModel {
                provider_id: provider.id.clone(),
                model_index: index,
            },
            appearance,
        );
        let row_controls = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                Container::new(chevron_button)
                    .with_margin_right(MODEL_ROW_GAP)
                    .finish(),
            )
            .with_child(quick_remove_button)
            .finish();

        let cell = |flex: f32, view: &ViewHandle<EditorView>| -> Box<dyn Element> {
            Expanded::new(
                flex,
                Container::new(ChildView::new(view).finish())
                    .with_margin_right(MODEL_ROW_GAP)
                    .finish(),
            )
            .finish()
        };

        let header_row = Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(cell(2., &row.name_editor))
            .with_child(cell(2., &row.id_editor))
            .with_child(cell(1., &row.context_editor))
            .with_child(cell(1., &row.output_editor))
            .with_child(row_controls)
            .finish();

        let mut col = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
            .with_child(header_row);

        if is_expanded {
            col = col.with_child(Self::render_model_detail_panel(
                provider,
                index,
                model,
                row,
                draft_editors,
                appearance,
            ));
        }

        Container::new(col.finish())
            .with_margin_bottom(MODEL_ROW_GAP)
            .finish()
    }

    /// 单条模型的展开 detail 面板:
    /// - Modalities: image / pdf / audio 三态 chip(Auto / On / Off)
    /// - Capabilities: reasoning / tool_call 两个 bool chip
    /// - 底部 Remove 按钮
    fn render_model_detail_panel(
        provider: &AgentProvider,
        index: usize,
        model: &AgentProviderModel,
        row: &ModelRow,
        draft_editors: ProviderDraftEditors,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let label_color = theme.active_ui_text_color();

        // ---- Modalities 区 ----
        let modalities_label = Container::new(
            Text::new(
                crate::t!("settings-agent-providers-model-modalities"),
                appearance.ui_font_family(),
                appearance.ui_font_size(),
            )
            .with_color(label_color.into())
            .finish(),
        )
        .with_margin_top(FIELD_LABEL_MARGIN_TOP)
        .with_margin_bottom(FIELD_LABEL_MARGIN_BOTTOM)
        .finish();

        let modality_chip = |label: &str,
                             slot: Option<bool>,
                             state: MouseStateHandle,
                             kind: ModelCapabilityKind|
         -> Box<dyn Element> {
            // 三态视觉:Auto = 裸标签 / On = `● label` / Off = `○ label`。
            // 沿用现有 ApiType / ReasoningEffort chip 的 `● {label}` selected 风格,
            // Off 用空心圆 ○ 跟实心 ● 对照,Auto 不带前缀(跟未选中态一致)。
            let chip_label = match slot {
                None => label.to_string(),
                Some(true) => format!("● {label}"),
                Some(false) => format!("○ {label}"),
            };
            Self::render_card_button_preserving_draft(
                chip_label,
                state,
                draft_editors.clone(),
                AISettingsPageAction::CycleAgentProviderModelCapability {
                    provider_id: provider.id.clone(),
                    model_index: index,
                    kind,
                },
                appearance,
            )
        };

        let modalities_row = Wrap::row()
            .with_spacing(6.)
            .with_run_spacing(4.)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(modality_chip(
                &crate::t!("settings-agent-providers-model-modality-image"),
                model.image,
                row.image_chip_state.clone(),
                ModelCapabilityKind::Image,
            ))
            .with_child(modality_chip(
                &crate::t!("settings-agent-providers-model-modality-pdf"),
                model.pdf,
                row.pdf_chip_state.clone(),
                ModelCapabilityKind::Pdf,
            ))
            .with_child(modality_chip(
                &crate::t!("settings-agent-providers-model-modality-audio"),
                model.audio,
                row.audio_chip_state.clone(),
                ModelCapabilityKind::Audio,
            ))
            .finish();

        // ---- Capabilities 区(reasoning / tool_call) ----
        let capabilities_label = Container::new(
            Text::new(
                crate::t!("settings-agent-providers-model-capabilities"),
                appearance.ui_font_family(),
                appearance.ui_font_size(),
            )
            .with_color(label_color.into())
            .finish(),
        )
        .with_margin_top(FIELD_LABEL_MARGIN_TOP)
        .with_margin_bottom(FIELD_LABEL_MARGIN_BOTTOM)
        .finish();

        let bool_chip = |label: &str,
                         on: bool,
                         state: MouseStateHandle,
                         action: AISettingsPageAction|
         -> Box<dyn Element> {
            let chip_label = if on {
                format!("● {label}")
            } else {
                format!("○ {label}")
            };
            Self::render_card_button_preserving_draft(
                chip_label,
                state,
                draft_editors.clone(),
                action,
                appearance,
            )
        };

        let capabilities_row = Wrap::row()
            .with_spacing(6.)
            .with_run_spacing(4.)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(bool_chip(
                &crate::t!("settings-agent-providers-model-capability-reasoning"),
                model.reasoning,
                row.reasoning_chip_state.clone(),
                AISettingsPageAction::ToggleAgentProviderModelReasoning {
                    provider_id: provider.id.clone(),
                    model_index: index,
                },
            ))
            .with_child(bool_chip(
                &crate::t!("settings-agent-providers-model-capability-tool-calling"),
                model.tool_call,
                row.tool_call_chip_state.clone(),
                AISettingsPageAction::ToggleAgentProviderModelToolCall {
                    provider_id: provider.id.clone(),
                    model_index: index,
                },
            ))
            .finish();

        // ---- Remove 按钮(展开后才出现,避免折叠态误删)----
        let remove_button = Self::render_card_button_preserving_draft(
            crate::t!("settings-agent-providers-remove-model"),
            row.remove_button_state.clone(),
            draft_editors,
            AISettingsPageAction::RemoveAgentProviderModel {
                provider_id: provider.id.clone(),
                model_index: index,
            },
            appearance,
        );

        let remove_row = Container::new(
            Flex::row()
                .with_main_axis_alignment(MainAxisAlignment::End)
                .with_child(remove_button)
                .finish(),
        )
        .with_margin_top(FIELD_LABEL_MARGIN_TOP)
        .finish();

        // 整体 detail panel 用一个稍内缩 + 边框样式,跟主 row 拉开层级。
        Container::new(
            Flex::column()
                .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
                .with_child(modalities_label)
                .with_child(modalities_row)
                .with_child(capabilities_label)
                .with_child(capabilities_row)
                .with_child(remove_row)
                .finish(),
        )
        .with_margin_top(4.)
        .with_margin_left(12.)
        .with_margin_bottom(8.)
        .finish()
    }

    fn render_provider_card(
        &self,
        provider: &AgentProvider,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let is_any_ai_enabled = AISettings::as_ref(app).is_any_ai_enabled(app);
        let label_color = if is_any_ai_enabled {
            appearance.theme().active_ui_text_color()
        } else {
            appearance.theme().disabled_ui_text_color()
        };
        let detail_color = if is_any_ai_enabled {
            appearance.theme().foreground()
        } else {
            appearance.theme().disabled_ui_text_color()
        };

        let rows = self.rows.borrow();
        let row = match rows.get(&provider.id) {
            Some(row) => row,
            None => {
                return Container::new(
                    Text::new(
                        crate::t!(
                            "settings-agent-providers-row-missing",
                            id = provider.id.as_str()
                        ),
                        appearance.ui_font_family(),
                        appearance.ui_font_size(),
                    )
                    .with_color(detail_color.into())
                    .finish(),
                )
                .with_margin_bottom(8.)
                .finish();
            }
        };
        let draft_editors = ProviderDraftEditors::from_row(provider.id.clone(), row);

        let name_field = field_block(
            &crate::t!("settings-agent-providers-field-name"),
            ChildView::new(&row.name_editor).finish(),
            label_color,
            appearance,
        );
        let api_type_field = self.render_api_type_field(
            provider,
            row,
            draft_editors.clone(),
            label_color,
            appearance,
        );
        let responses_options =
            (provider.api_type == AgentProviderApiType::OpenAiResp).then(|| {
                self.render_responses_options(
                    provider,
                    row,
                    draft_editors.clone(),
                    label_color,
                    appearance,
                )
            });
        let base_url_field = field_block(
            &crate::t!("settings-agent-providers-field-base-url"),
            ChildView::new(&row.base_url_editor).finish(),
            label_color,
            appearance,
        );
        let api_key_field = field_block(
            &crate::t!("settings-agent-providers-field-api-key"),
            ChildView::new(&row.api_key_editor).finish(),
            label_color,
            appearance,
        );

        let headers_label = Container::new(
            Text::new(
                crate::t!("settings-agent-providers-extra-headers"),
                appearance.ui_font_family(),
                appearance.ui_font_size(),
            )
            .with_color(label_color.into())
            .finish(),
        )
        .with_margin_top(FIELD_LABEL_MARGIN_TOP)
        .with_margin_bottom(FIELD_LABEL_MARGIN_BOTTOM)
        .finish();
        let mut headers_column = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
            .with_child(headers_label);

        for (idx, h_row) in row.header_rows.iter().enumerate() {
            let remove_header_button = Self::render_card_button_preserving_draft(
                "×",
                h_row.remove_button_state.clone(),
                draft_editors.clone(),
                AISettingsPageAction::RemoveAgentProviderHeader {
                    provider_id: provider.id.clone(),
                    header_index: idx,
                },
                appearance,
            );
            let header_row = Flex::row()
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(
                    Expanded::new(
                        1.,
                        Container::new(ChildView::new(&h_row.key_editor).finish())
                            .with_margin_right(MODEL_ROW_GAP)
                            .finish(),
                    )
                    .finish(),
                )
                .with_child(
                    Expanded::new(
                        1.,
                        Container::new(ChildView::new(&h_row.val_editor).finish())
                            .with_margin_right(MODEL_ROW_GAP)
                            .finish(),
                    )
                    .finish(),
                )
                .with_child(remove_header_button)
                .finish();
            headers_column.add_child(
                Container::new(header_row)
                    .with_margin_bottom(MODEL_ROW_GAP)
                    .finish(),
            );
        }

        let add_header_button = Self::render_card_button_preserving_draft(
            crate::t!("settings-agent-providers-add-header"),
            row.add_header_button_state.clone(),
            draft_editors.clone(),
            AISettingsPageAction::AddAgentProviderHeader {
                provider_id: provider.id.clone(),
            },
            appearance,
        );
        headers_column.add_child(add_header_button);

        // ---- 模型列表区 ----
        let models_label = Container::new(
            Text::new(
                crate::t!(
                    "settings-agent-providers-models-label",
                    count = provider.models.len()
                ),
                appearance.ui_font_family(),
                appearance.ui_font_size(),
            )
            .with_color(label_color.into())
            .finish(),
        )
        .with_margin_top(FIELD_LABEL_MARGIN_TOP)
        .with_margin_bottom(FIELD_LABEL_MARGIN_BOTTOM)
        .finish();

        let mut models_column = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
            .with_child(models_label);

        if provider.models.is_empty() {
            let empty_hint = Container::new(
                Text::new(
                    crate::t!("settings-agent-providers-models-empty-hint"),
                    appearance.ui_font_family(),
                    appearance.ui_font_size(),
                )
                .with_color(appearance.theme().disabled_ui_text_color().into())
                .soft_wrap(true)
                .finish(),
            )
            .with_margin_bottom(MODEL_ROW_GAP)
            .finish();
            models_column.add_child(empty_hint);
        } else {
            // 表头: 显示名 | 模型 ID | 上下文 | 输出
            let dim = appearance.theme().disabled_ui_text_color();
            let header_cell = |flex: f32, label: &str| -> Box<dyn Element> {
                Expanded::new(
                    flex,
                    Container::new(
                        Text::new(
                            label.to_string(),
                            appearance.ui_font_family(),
                            appearance.ui_font_size(),
                        )
                        .with_color(dim.into())
                        .finish(),
                    )
                    .with_margin_right(MODEL_ROW_GAP)
                    .finish(),
                )
                .finish()
            };
            let header = Container::new(
                Flex::row()
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(header_cell(
                        2.,
                        &crate::t!("settings-agent-providers-models-header-name"),
                    ))
                    .with_child(header_cell(
                        2.,
                        &crate::t!("settings-agent-providers-models-header-id"),
                    ))
                    .with_child(header_cell(
                        1.,
                        &crate::t!("settings-agent-providers-models-header-context"),
                    ))
                    .with_child(header_cell(
                        1.,
                        &crate::t!("settings-agent-providers-models-header-output"),
                    ))
                    // 占位,与下方展开/删除两个按钮对齐。
                    .with_child(
                        Flex::row()
                            .with_cross_axis_alignment(CrossAxisAlignment::Center)
                            .with_child(
                                Container::new(
                                    Text::new(
                                        "  ".to_string(),
                                        appearance.ui_font_family(),
                                        appearance.ui_font_size(),
                                    )
                                    .with_color(dim.into())
                                    .finish(),
                                )
                                .with_margin_right(MODEL_ROW_GAP)
                                .finish(),
                            )
                            .with_child(
                                Text::new(
                                    "  ".to_string(),
                                    appearance.ui_font_family(),
                                    appearance.ui_font_size(),
                                )
                                .with_color(dim.into())
                                .finish(),
                            )
                            .finish(),
                    )
                    .finish(),
            )
            .with_margin_bottom(2.)
            .finish();
            models_column.add_child(header);

            for (idx, m_row) in row.model_rows.iter().enumerate() {
                let model = match provider.models.get(idx) {
                    Some(m) => m,
                    // 极端情况:rebuild 间隙 settings 又被改了,model_rows 与 provider.models
                    // 长度暂时不一致;跳过避免 panic,下一帧会自然修正。
                    None => continue,
                };
                models_column.add_child(Self::render_model_row(
                    provider,
                    idx,
                    model,
                    m_row,
                    draft_editors.clone(),
                    appearance,
                ));
            }
        }

        // ---- 底部按钮行 ----
        let add_model_button = Self::render_card_button_preserving_draft(
            crate::t!("settings-agent-providers-add-model"),
            row.add_model_button_state.clone(),
            draft_editors.clone(),
            AISettingsPageAction::AddAgentProviderModel {
                provider_id: provider.id.clone(),
            },
            appearance,
        );
        let fetch_button = Self::render_card_button_preserving_draft(
            crate::t!("settings-agent-providers-fetch-from-api"),
            row.fetch_button_state.clone(),
            draft_editors.clone(),
            AISettingsPageAction::FetchAgentProviderModels {
                provider_id: provider.id.clone(),
            },
            appearance,
        );
        let sync_models_dev_button = Self::render_card_button_preserving_draft(
            crate::t!("settings-agent-providers-sync-models-dev"),
            row.sync_models_dev_button_state.clone(),
            draft_editors.clone(),
            AISettingsPageAction::SyncProviderModelsFromModelsDev {
                provider_id: provider.id.clone(),
            },
            appearance,
        );
        let remove_button = Self::render_card_button(
            crate::t!("settings-agent-providers-remove"),
            row.remove_button_state.clone(),
            AISettingsPageAction::RemoveAgentProvider {
                provider_id: provider.id.clone(),
            },
            appearance,
        );

        // ---- 保存按钮:在 on_click 闭包里现场读取所有表单 buffer。
        // 这里不能预先 build action(表单值随输入变化),所以过 draft editor handle
        // 随闭包走,点击时一起 dispatch SaveAgentProviderEdits。
        let save_button = {
            let draft_editors = draft_editors.clone();

            appearance
                .ui_builder()
                .button(ButtonVariant::Accent, row.save_button_state.clone())
                .with_style(UiComponentStyles {
                    font_size: Some(appearance.ui_font_body()),
                    padding: Some(Coords::uniform(CARD_BUTTON_PADDING)),
                    ..Default::default()
                })
                .with_centered_text_label(crate::t!("settings-agent-providers-save"))
                .build()
                .on_click(move |ctx, app, _| {
                    ctx.dispatch_typed_action(draft_editors.to_save_action(app));
                })
                .finish()
        };

        let bottom_row = Flex::row()
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                Flex::row()
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(
                        Container::new(add_model_button)
                            .with_margin_right(8.)
                            .finish(),
                    )
                    .with_child(Container::new(fetch_button).with_margin_right(8.).finish())
                    .with_child(sync_models_dev_button)
                    .finish(),
            )
            .with_child(
                Container::new(
                    Flex::row()
                        .with_cross_axis_alignment(CrossAxisAlignment::Center)
                        .with_child(Container::new(save_button).with_margin_right(8.).finish())
                        .with_child(remove_button)
                        .finish(),
                )
                // 与左侧主操作组（添加模型 / 抓取 / 同步）拉开明显间隔，
                // 避免 SpaceBetween 在卡片宽不够时两组贴在一起。
                .with_margin_left(16.)
                .finish(),
            )
            .finish();

        // 用透明 detail_color 触发它被读取(避免 unused 警告);仅用于潜在配色。
        let _ = detail_color;

        let mut content = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
            .with_child(name_field)
            .with_child(api_type_field);
        if let Some(responses_options) = responses_options {
            content.add_child(responses_options);
        }
        content.add_child(base_url_field);
        content.add_child(api_key_field);
        content.add_child(
            Container::new(headers_column.finish())
                .with_margin_top(8.)
                .finish(),
        );
        content.add_child(
            Container::new(models_column.finish())
                .with_margin_top(8.)
                .finish(),
        );
        content.add_child(Container::new(bottom_row).with_margin_top(10.).finish());

        Container::new(content.finish())
            .with_background(appearance.theme().surface_1())
            .with_uniform_padding(12.)
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(6.)))
            .with_margin_bottom(8.)
            .finish()
    }
}

/// 把用户输入解析成 token 数。容忍 `128k` / `128K` / `128 000` / `128,000` / 空白,
/// 解析失败一律返回 0(语义:未指定)。
fn parse_token_count(input: &str) -> u32 {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ',' && *c != '_')
        .collect();
    if cleaned.is_empty() {
        return 0;
    }
    let lower = cleaned.to_lowercase();
    let (num_part, multiplier): (&str, u64) = if let Some(stripped) = lower.strip_suffix('k') {
        (stripped, 1_000)
    } else if let Some(stripped) = lower.strip_suffix('m') {
        (stripped, 1_000_000)
    } else {
        (lower.as_str(), 1)
    };
    num_part
        .parse::<f64>()
        .ok()
        .map(|n| (n * multiplier as f64).round() as u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0)
}

/// 失焦时把编辑器选区折叠到末尾。
///
/// 每个输入框是一个独立的 `EditorView`,各自维护自己的 selection range。
/// 选区高亮的绘制不受焦点状态影响(见 `app/src/editor/view/element.rs:1091`),
/// 所以双击/三击/拖选后失焦,旧选区会一直留在 buffer 上,与其它编辑器的选区
/// 同时显示,看起来像"多个 select 状态"。这里在 Blurred 时把 head/tail 都
/// 收到末尾,视觉上释放选中。
fn collapse_selection_if_blurred(
    editor: &ViewHandle<EditorView>,
    event: &EditorEvent,
    ctx: &mut ViewContext<AISettingsPageView>,
) {
    if matches!(event, EditorEvent::Blurred) {
        editor.update(ctx, |editor, ctx| editor.move_to_buffer_end(ctx));
    }
}

fn single_line_editor_options(
    appearance: &Appearance,
    is_password: bool,
) -> SingleLineEditorOptions {
    SingleLineEditorOptions {
        is_password,
        clear_selections_on_blur: true,
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
    }
}

fn field_block(
    label: &str,
    editor_element: Box<dyn Element>,
    label_color: warp_core::ui::theme::Fill,
    appearance: &Appearance,
) -> Box<dyn Element> {
    let label_text = Container::new(
        Text::new(
            label.to_string(),
            appearance.ui_font_family(),
            appearance.ui_font_size(),
        )
        .with_color(label_color.into())
        .finish(),
    )
    .with_margin_top(FIELD_LABEL_MARGIN_TOP)
    .with_margin_bottom(FIELD_LABEL_MARGIN_BOTTOM)
    .finish();

    Flex::column()
        .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
        .with_child(label_text)
        .with_child(editor_element)
        .finish()
}

impl SettingsWidget for AgentProvidersWidget {
    type View = AISettingsPageView;

    fn search_terms(&self) -> &str {
        "agent provider providers custom openai responses compatible deepseek glm moonshot dashscope qwen ollama base url api key models save state websocket background compaction programmatic multi-agent ZDR 提供商 自定义 模型 保存 状态 后台 压缩"
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let is_any_ai_enabled = AISettings::as_ref(app).is_any_ai_enabled(app);
        let providers = AISettings::as_ref(app).agent_providers.value().clone();

        let title_node = build_sub_header(
            appearance,
            crate::t!("settings-agent-providers-title"),
            Some(if is_any_ai_enabled {
                appearance.theme().active_ui_text_color()
            } else {
                appearance.theme().disabled_ui_text_color()
            }),
        )
        .finish();

        let header_add_button = Self::render_card_button(
            crate::t!("settings-agent-providers-add-button"),
            self.add_button_state.clone(),
            AISettingsPageAction::AddAgentProvider,
            appearance,
        );

        let header = Container::new(
            Flex::row()
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(Expanded::new(1., title_node).finish())
                .with_child(header_add_button)
                .finish(),
        )
        .with_padding_bottom(HEADER_PADDING)
        .finish();

        let description_text = crate::t!("settings-agent-providers-description");
        let description = Container::new(
            Text::new(
                description_text,
                appearance.ui_font_family(),
                appearance.ui_font_size(),
            )
            .with_color(if is_any_ai_enabled {
                appearance.theme().foreground().into()
            } else {
                appearance.theme().disabled_ui_text_color().into()
            })
            .soft_wrap(true)
            .finish(),
        )
        .with_margin_bottom(12.)
        .finish();

        let mut column = Flex::column().with_child(header).with_child(description);

        if providers.is_empty() {
            let empty = Container::new(
                Text::new(
                    crate::t!("settings-agent-providers-empty"),
                    appearance.ui_font_family(),
                    appearance.ui_font_size(),
                )
                .with_color(appearance.theme().disabled_ui_text_color().into())
                .finish(),
            )
            .with_margin_bottom(12.)
            .finish();
            column.add_child(empty);
        } else {
            for provider in &providers {
                column.add_child(self.render_provider_card(provider, appearance, app));
            }
        }

        Container::new(column.finish())
            .with_margin_bottom(HEADER_PADDING)
            .finish()
    }
}
