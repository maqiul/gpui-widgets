//! 工具提示：悬停时由 GPUI 原生 tooltip 机制显示气泡（自动延迟与定位）。

use gpui::{
    AnyElement, Context, Div, ElementId, Hsla, IntoElement, Render, Role, SharedString, Stateful,
    Window, div, prelude::*, px,
};

use crate::a11y::Semantics;
use crate::theme::{self, Tokens, metric};

/// 气泡外观（antdv-next `.ant-tooltip-container` 实测）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooltipStyle {
    pub bg: Hsla,
    pub fg: Hsla,
    pub padding_block: f32,
    pub padding_inline: f32,
    pub radius: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub font_size: f32,
    pub line_height: f32,
}

/// `colorBgSpotlight` 底 + `colorTextLightSolid` 字；`paddingSM / 2` 纵向、`paddingXS` 横向。
pub fn tooltip_state(tokens: &Tokens) -> TooltipStyle {
    let m = metric::TooltipMetric::resolve(tokens);
    TooltipStyle {
        bg: tokens.color_bg_spotlight,
        fg: tokens.color_text_light_solid,
        padding_block: m.padding_block,
        padding_inline: m.padding_inline,
        radius: m.radius,
        min_height: m.min_height,
        max_width: m.max_width,
        font_size: m.font_size,
        line_height: m.line_height,
    }
}

/// 气泡表面：原生 tooltip 与截图面板共用同一套渲染。
pub fn tooltip_bubble(text: impl Into<SharedString>) -> Stateful<Div> {
    let (style, shadow) = theme::with(|t| (tooltip_state(t), t.shadow_secondary.0.clone()));
    let text = text.into();
    let id = format!("tooltip-bubble-{text}");
    Semantics::new(Role::Tooltip, text.clone())
        .id(id.clone())
        .read_only(true)
        .apply(
            div()
                .id(ElementId::Name(id.into()))
                .max_w(px(style.max_width))
                .min_h(px(style.min_height))
                .px(px(style.padding_inline))
                .py(px(style.padding_block))
                .rounded(px(style.radius))
                .bg(style.bg)
                .text_color(style.fg)
                .text_size(px(style.font_size))
                .line_height(px(style.line_height))
                .shadow(shadow)
                .child(text),
        )
}

/// 通用工具提示包装器。
///
/// ```ignore
/// Tooltip::new("删除此项", Button::new("删除").danger())
/// ```
pub struct Tooltip {
    text: SharedString,
    child: AnyElement,
}

impl Tooltip {
    pub fn new(text: impl Into<SharedString>, child: impl IntoElement) -> Self {
        Self {
            text: text.into(),
            child: child.into_any_element(),
        }
    }
}

impl IntoElement for Tooltip {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let text = self.text;
        div()
            .id(ElementId::Name(format!("tooltip-{text}").into()))
            .tooltip(move |_, cx| cx.new(|_| TooltipView { text: text.clone() }).into())
            .child(self.child)
    }
}

/// 气泡视图：深色底 + 浅色文字
struct TooltipView {
    text: SharedString,
}

impl Render for TooltipView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        tooltip_bubble(self.text.clone())
    }
}
