//! 结果页：操作结果的大图标反馈（成功 / 失败 / 警告 / 信息）。

use gpui::{
    AnyElement, App, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    Role, SharedString, Styled, Window, div, px,
};

use crate::a11y::Semantics;
use crate::theme::{self, metric::ResultMetric};

/// 结果状态
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResultStatus {
    Success,
    Error,
    Warning,
    Info,
}

/// Result 的取色：状态色 + 文字层级 + 正文块底色。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResultStyle {
    pub success: Hsla,
    pub error: Hsla,
    pub warning: Hsla,
    pub info: Hsla,
    pub icon_fg: Hsla,
    pub title: Hsla,
    pub subtitle: Hsla,
    pub body_bg: Hsla,
}

pub fn result_style(tokens: &theme::Tokens) -> ResultStyle {
    ResultStyle {
        success: tokens.color_success,
        error: tokens.color_error,
        warning: tokens.color_warning,
        info: tokens.color_info,
        icon_fg: tokens.color_bg_container,
        title: tokens.color_text_heading,
        subtitle: tokens.color_text_description,
        body_bg: tokens.color_fill_alter,
    }
}

fn status_style(status: ResultStatus, style: &ResultStyle) -> (&'static str, Hsla) {
    match status {
        ResultStatus::Success => ("✓", style.success),
        ResultStatus::Error => ("✕", style.error),
        ResultStatus::Warning => ("!", style.warning),
        ResultStatus::Info => ("i", style.info),
    }
}

/// 结果页。
///
/// ```ignore
/// Result::new(ResultStatus::Success, "提交成功")
///     .description("订单号 20261112-001，预计 3 天内发货")
///     .action(Button::new("返回首页"))
/// ```
#[derive(IntoElement)]
pub struct Result {
    status: ResultStatus,
    title: SharedString,
    description: Option<SharedString>,
    action: Option<AnyElement>,
}

impl Result {
    pub fn new(status: ResultStatus, title: impl Into<SharedString>) -> Self {
        Self {
            status,
            title: title.into(),
            description: None,
            action: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 底部操作区（按钮等）
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl RenderOnce for Result {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let style = theme::with(result_style);
        let (icon, color) = status_style(self.status, &style);
        let m = theme::with(ResultMetric::resolve);
        let result_id = format!("result-{}", self.title);
        let mut semantics = Semantics::new(Role::Status, self.title.clone()).id(result_id.clone());
        if let Some(description) = &self.description {
            semantics = semantics.description(description.clone());
        }

        let icon_el = div()
            .w(px(m.icon_size))
            .h(px(m.icon_size))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(color)
            .text_color(style.icon_fg)
            .text_size(px(m.icon_size * m.icon_glyph_ratio))
            .font_weight(gpui::FontWeight::BOLD)
            .child(icon);

        let mut root = div()
            .id(ElementId::Name(result_id.into()))
            .flex()
            .flex_col()
            .items_center()
            .py(px(m.root_padding_block))
            .px(px(m.root_padding_inline))
            .child(div().mb(px(m.icon_margin_bottom)).child(icon_el))
            .child(
                div()
                    .text_size(px(m.title_size))
                    .line_height(px(m.title_line_height))
                    .text_color(style.title)
                    .my(px(m.title_margin))
                    .child(self.title.clone()),
            );

        if let Some(description) = &self.description {
            root = root.child(
                div()
                    .text_size(px(m.subtitle_size))
                    .line_height(px(m.subtitle_line_height))
                    .text_color(style.subtitle)
                    .child(description.clone()),
            );
        }
        if let Some(action) = self.action {
            root = root.child(div().mt(px(m.extra_margin)).child(action));
        }
        semantics.apply(root)
    }
}
