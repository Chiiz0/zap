//! 使用原生字体检查上下文预算错误及其外层提示，不发起模型请求。

use std::collections::HashMap;

use warp::appearance::Appearance;
use warp::i18n;
use warpui_core::elements::DEFAULT_LINE_HEIGHT_RATIO;
use warpui_core::integration::{AssertionOutcome, TestStep};
use warpui_core::platform::LineStyle;
use warpui_core::text_layout::{
    DEFAULT_TOP_BOTTOM_RATIO, LayoutCache, StyleAndFont, TextAlignment, TextStyle,
};
use warpui_core::{AppContext, SingletonEntity};

use super::{Builder, new_builder};

fn check_context_error_text_layout(locale: &str, ctx: &AppContext) -> Result<(), String> {
    if i18n::current_languages()
        .first()
        .map(ToString::to_string)
        .as_deref()
        != Some(locale)
    {
        return Err(format!("未成功切换到 {locale}"));
    }
    let loader = i18n::loader().ok_or("本地化加载器未初始化")?;
    let message = loader.get("ai-error-context-too-large");
    let wrapped = loader.get_args(
        "ai-error-context-window-exceeded",
        HashMap::from([("message", message.as_str())]),
    );
    let appearance = Appearance::as_ref(ctx);
    let cache = LayoutCache::new();
    for (key, text) in [
        ("ai-error-context-too-large", message),
        (
            "ai-error-compaction-invalid",
            loader.get("ai-error-compaction-invalid"),
        ),
        (
            "ai-error-compaction-save-failed",
            loader.get("ai-error-compaction-save-failed"),
        ),
        ("ai-error-context-window-exceeded", wrapped),
    ] {
        if text.is_empty() || text.contains("ai-error-") || text.contains('{') {
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

        // 对齐 RenderableAction 正文的 UI 字体、终端字号、默认行高及无截断软换行。
        // 宽度表示扣除卡片内边距和错误图标后的实际文字区域。
        for width in [220.0, 360.0] {
            let frame = cache.layout_text(
                &text,
                LineStyle {
                    font_size: appearance.monospace_font_size(),
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
                let visible_width = line.width - line.trailing_whitespace_width;
                if visible_width > width + 0.1 {
                    return Err(format!(
                        "{locale}/{key} 在 {width}px 下溢出或截断：可见 {visible_width}px"
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

pub fn test_context_error_text_layout_in_english_and_chinese() -> Builder {
    new_builder().with_step(
        TestStep::new("检查上下文错误文案的中英文原生字体布局").add_named_assertion(
            "上下文提示及外层消息在窄文字区域完整换行",
            |app, _| {
                i18n::init(None);
                let previous_locale = i18n::current_languages().first().map(ToString::to_string);
                let result = ["en", "zh-CN"].into_iter().try_for_each(|locale| {
                    i18n::set_locale(locale);
                    app.update(|ctx| check_context_error_text_layout(locale, ctx))
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
