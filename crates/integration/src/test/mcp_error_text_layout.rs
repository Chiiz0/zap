//! 使用原生字体排版检查 MCP 错误文案；不启动 MCP 服务。

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use pathfinder_geometry::vector::vec2f;
use warp::appearance::Appearance;
use warp::i18n;
use warp::settings_view::mcp_servers::ServerCardItemId;
use warp::settings_view::mcp_servers::server_card::{ServerCardStatus, ServerCardView};
use warpui_core::elements::{
    ChildView, ConstrainedBox, Container, DEFAULT_LINE_HEIGHT_RATIO, Flex, MainAxisSize, Padding,
    ParentElement, Text,
};
use warpui_core::integration::{AssertionOutcome, TestStep};
use warpui_core::platform::{LineStyle, WindowBounds, WindowStyle};
use warpui_core::text_layout::{
    DEFAULT_TOP_BOTTOM_RATIO, LayoutCache, StyleAndFont, TextAlignment, TextStyle,
};
use warpui_core::{
    AppContext, Element, Entity, SingletonEntity, TypedActionView, View, ViewHandle, async_assert,
};

use super::{Builder, new_builder};

fn check_error_text_layout(locale: &str, ctx: &AppContext) -> Result<(), String> {
    if i18n::current_languages()
        .first()
        .map(ToString::to_string)
        .as_deref()
        != Some(locale)
    {
        return Err(format!("未成功切换到 {locale}"));
    }
    let appearance = Appearance::as_ref(ctx);
    let cache = LayoutCache::new();
    for (key, text) in localized_error_messages()? {
        if text.is_empty() || text.contains(key) || text.contains('{') {
            return Err(format!("{locale}/{key} 未完整解析：{text:?}"));
        }
        let styles = [(
            0..text.chars().count(),
            StyleAndFont::new(
                appearance.ui_font_family(),
                Default::default(),
                TextStyle::new().with_foreground_color(appearance.theme().ui_error_color()),
            ),
        )];

        // 与 ServerCard 的 FormattedTextElement 相同：UI 字体、默认行高、软换行、无截断。
        // 两档宽度代表卡片中扣除图标与操作按钮后可供错误信息使用的文字区域。
        for width in [220.0, 360.0] {
            let frame = cache.layout_text(
                &text,
                LineStyle {
                    font_size: appearance.ui_builder().ui_font_size(),
                    line_height_ratio: DEFAULT_LINE_HEIGHT_RATIO,
                    baseline_ratio: DEFAULT_TOP_BOTTOM_RATIO,
                    fixed_width_tab_size: None,
                },
                &styles,
                width,
                f32::MAX,
                TextAlignment::Left,
                None,
                &ctx.font_cache().text_layout_system(),
            );
            if frame.lines().last().map(|line| line.end_index()) != Some(text.chars().count()) {
                return Err(format!(
                    "{locale}/{key} 在 {width}px 下未排版到最后一个字符"
                ));
            }
            if !frame.height().is_finite() || frame.height() <= 0.0 {
                return Err(format!("{locale}/{key} 在 {width}px 下高度无效"));
            }
            if width == 220.0 && frame.lines().len() < 2 {
                return Err(format!("{locale}/{key} 在窄文字区域未产生预期软换行"));
            }
            for line in frame.lines() {
                // 与 Line::paint_internal 一致：软换行处的尾部空白不产生可见溢出。
                // macOS 总会给末行附加 clip_config；只有可见宽度超限才实际触发裁剪。
                let visible_width = line.width - line.trailing_whitespace_width;
                if visible_width > width + 0.1 {
                    return Err(format!(
                        "{locale}/{key} 在 {width}px 下溢出或截断：可见 {visible_width}px，尾部空白 {}px",
                        line.trailing_whitespace_width
                    ));
                }
                if !line.chars_with_missing_glyphs.is_empty() {
                    return Err(format!(
                        "{locale}/{key} 缺少字形：{:?}",
                        line.chars_with_missing_glyphs
                    ));
                }
            }
        }
    }
    Ok(())
}

fn localized_error_messages() -> Result<Vec<(&'static str, String)>, String> {
    let loader = i18n::loader().ok_or("本地化加载器未初始化")?;
    let messages = [
        ("mcp-credentials-rejected", HashMap::new()),
        (
            "mcp-header-unresolved-secrets",
            HashMap::from([
                ("header", "Authorization"),
                ("secrets", "MCP_ACCESS_TOKEN, MCP_PROJECT_SECRET"),
            ]),
        ),
        ("mcp-oauth-unavailable", HashMap::new()),
        ("mcp-headless-oauth-required", HashMap::new()),
        (
            "mcp-startup-missing-secrets",
            HashMap::from([("secrets", "MCP_ACCESS_TOKEN, MCP_PROJECT_SECRET")]),
        ),
    ];

    Ok(messages
        .into_iter()
        .map(|(key, args)| (key, loader.get_args(key, args)))
        .collect())
}

struct McpCardsPreview {
    locale: &'static str,
    width: f32,
    cards: Vec<ViewHandle<ServerCardView>>,
}

impl Entity for McpCardsPreview {
    type Event = ();
}

impl TypedActionView for McpCardsPreview {
    type Action = ();
}

impl View for McpCardsPreview {
    fn ui_name() -> &'static str {
        "McpCardsPreview"
    }

    fn render(&self, ctx: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(ctx);
        let mut column = Flex::column()
            .with_main_axis_size(MainAxisSize::Min)
            .with_spacing(12.);
        column = column.with_child(
            Text::new(
                format!("MCP · {} · {} px", self.locale, self.width),
                appearance.ui_font_family(),
                appearance.ui_font_size(),
            )
            .with_color(appearance.theme().active_ui_text_color().into_solid())
            .finish(),
        );
        for card in &self.cards {
            column = column.with_child(
                ConstrainedBox::new(ChildView::new(card).finish())
                    .with_width(self.width)
                    .finish(),
            );
        }
        Container::new(column.finish())
            .with_background(appearance.theme().surface_2())
            .with_padding(Padding::uniform(24.))
            .finish()
    }
}

/// 仅供手动运行：输出生产 MCP 错误卡片的四张原生窗口截图。
/// 运行时需显式设置 `WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1` 以启用 Metal。
pub fn test_mcp_error_cards_visual_preview() -> Builder {
    warp::features::FeatureFlag::McpDebuggingIds.set_enabled(false);
    let first_frame_drawn = Rc::new(Cell::new(false));
    let first_frame_callback = first_frame_drawn.clone();
    let mut builder = new_builder().with_real_display().with_step(
        TestStep::new("等待原应用完成首帧初始化")
            .with_action(move |app, window_id, _| {
                let first_frame_callback = first_frame_callback.clone();
                app.update(|ctx| {
                    ctx.on_next_frame_drawn(window_id, move || first_frame_callback.set(true));
                    ctx.windows()
                        .platform_window(window_id)
                        .expect("应有原应用窗口")
                        .as_ctx()
                        .request_redraw();
                });
            })
            .add_named_assertion("原应用首帧已绘制", move |_, _| {
                async_assert!(first_frame_drawn.get(), "等待原应用首帧")
            }),
    );
    for (locale, width) in [("en", 480.), ("en", 680.), ("zh-CN", 480.), ("zh-CN", 680.)] {
        builder = builder
            .with_step(TestStep::new("打开隔离的 MCP 错误卡片预览").with_action(
                move |app, _, _| {
                    i18n::init(None);
                    i18n::set_locale(locale);
                    let (preview_window, preview) = app.add_window_with_bounds(
                        WindowStyle::Normal,
                        WindowBounds::ExactSize(vec2f(width + 48., 1000.)),
                        |ctx| {
                            let cards = localized_error_messages()
                                .expect("应能读取错误文案")
                                .into_iter()
                                .enumerate()
                                .map(|(index, (_, text))| {
                                    ctx.add_typed_action_view(|_| {
                                        ServerCardView::new(
                                            ServerCardItemId::FileBasedMCP(
                                                format!("00000000-0000-0000-0000-{index:012}")
                                                    .parse()
                                                    .expect("测试 UUID 应有效"),
                                            ),
                                            format!("MCP {}", index + 1),
                                            None,
                                            None,
                                            Some(text),
                                            Vec::new(),
                                            ServerCardStatus::Error.into(),
                                        )
                                    })
                                })
                                .collect();
                            McpCardsPreview {
                                locale,
                                width,
                                cards,
                            }
                        },
                    );
                    preview.update(app, |_, ctx| ctx.notify());
                    app.update(|ctx| ctx.windows().show_window_and_focus_app(preview_window));
                },
            ))
            .with_step(
                TestStep::new("确认预览窗口并截图")
                    .add_named_assertion(
                        "截图目标是当前语言及宽度的预览窗口",
                        move |app, _| {
                            let window_id = app
                                .read(|ctx| ctx.windows().active_window())
                                .expect("应有活动窗口");
                            let preview = app.root_view::<McpCardsPreview>(window_id);
                            if !preview.is_some_and(|view| {
                                view.read(app, |view, _| {
                                    view.locale == locale && view.width == width
                                })
                            }) {
                                return async_assert!(false, "预览窗口尚未获得焦点");
                            }
                            AssertionOutcome::Success
                        },
                    )
                    .with_take_screenshot(format!("mcp-error-cards-{locale}-{width:.0}.png")),
            );
    }
    builder
}

pub fn test_mcp_error_text_layout_in_english_and_chinese() -> Builder {
    new_builder().with_step(
        TestStep::new("检查 MCP 错误文案的中英文原生字体布局").add_named_assertion(
            "错误文案在窄卡片及正常卡片文字区域完整换行",
            |app, _| {
                i18n::init(None);
                let previous_locale = i18n::current_languages().first().map(ToString::to_string);
                let result = ["en", "zh-CN"].into_iter().try_for_each(|locale| {
                    i18n::set_locale(locale);
                    app.update(|ctx| check_error_text_layout(locale, ctx))
                });
                if let Some(previous_locale) = previous_locale {
                    i18n::set_locale(&previous_locale);
                }
                match result {
                    Ok(()) => AssertionOutcome::Success,
                    Err(message) => AssertionOutcome::immediate_failure(message),
                }
            },
        ),
    )
}
