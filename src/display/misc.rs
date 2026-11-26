//! 布局辅助元素：分隔线、卡片容器、章节标题。

use gpui::{
    App, Div, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    SharedString, Stateful, Styled, Window, div, px,
};

use crate::{
    a11y::Semantics,
    theme::{
        self, Tokens,
        metric::{DIVIDER_TEXT_WEIGHT, DividerMetric},
    },
};

/// 分隔线配色：线条 `colorSplit`；带文字标题色 `colorTextHeading`，
/// plain 时退回 `colorText`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DividerStyle {
    pub line: Hsla,
    pub text: Hsla,
    pub plain_text: Hsla,
}

pub fn divider_style(tokens: &Tokens) -> DividerStyle {
    DividerStyle {
        line: tokens.color_split,
        text: tokens.color_text_heading,
        plain_text: tokens.color_text,
    }
}

/// 文字位置（对应 antd `orientation`）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerPlacement {
    #[default]
    Center,
    Start,
    End,
}

/// 分隔线。
///
/// ```ignore
/// Divider::new()
/// Divider::new().text("标题")
/// Divider::new().vertical()
/// ```
#[derive(Default, IntoElement)]
pub struct Divider {
    text: Option<SharedString>,
    placement: DividerPlacement,
    plain: bool,
    dashed: bool,
    vertical: bool,
}

impl Divider {
    pub fn new() -> Self {
        Self::default()
    }

    /// 线中嵌入文字
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn start(mut self) -> Self {
        self.placement = DividerPlacement::Start;
        self
    }

    pub fn end(mut self) -> Self {
        self.placement = DividerPlacement::End;
        self
    }

    /// 文字用正文色与常规字重
    pub fn plain(mut self) -> Self {
        self.plain = true;
        self
    }

    pub fn dashed(mut self) -> Self {
        self.dashed = true;
        self
    }

    /// 竖线（用于行内元素之间）
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
}

fn rail(look: &Look, grow: f32) -> Div {
    let mut rail = div().h(px(look.m.width)).flex_basis(px(0.));
    rail.style().flex_grow = Some(grow);
    if look.dashed {
        rail.border_t(px(look.m.width))
            .border_dashed()
            .border_color(look.style.line)
    } else {
        rail.bg(look.style.line)
    }
}

struct Look {
    style: DividerStyle,
    m: DividerMetric,
    dashed: bool,
}

impl RenderOnce for Divider {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let look = theme::with(|tokens| Look {
            style: divider_style(tokens),
            m: DividerMetric::resolve(tokens),
            dashed: self.dashed,
        });

        if self.vertical {
            let line = div()
                .flex_shrink_0()
                .w(px(look.m.width))
                .h(px(look.m.vertical_height))
                .mx(px(look.m.vertical_margin_inline));
            let line = if look.dashed {
                line.border_l(px(look.m.width))
                    .border_dashed()
                    .border_color(look.style.line)
            } else {
                line.bg(look.style.line)
            };
            return line.into_any_element();
        }

        let mut row = div().flex().flex_row().items_center().w_full();
        if let Some(text) = self.text {
            row = row.my(px(look.m.margin_with_text));
            let (start, end) = match self.placement {
                DividerPlacement::Center => (1.0, 1.0),
                DividerPlacement::Start => {
                    (look.m.orientation_margin, 1.0 - look.m.orientation_margin)
                }
                DividerPlacement::End => {
                    (1.0 - look.m.orientation_margin, look.m.orientation_margin)
                }
            };
            let (size, color, pad) = if self.plain {
                (
                    look.m.plain_text_size,
                    look.style.plain_text,
                    look.m.plain_text_padding_inline,
                )
            } else {
                (
                    look.m.text_size,
                    look.style.text,
                    look.m.text_padding_inline,
                )
            };
            let semantics =
                Semantics::new(Role::Label, text.clone()).id(format!("divider-label-{text}"));
            let element_id = semantics.element_id();
            let label = semantics.apply(
                div()
                    .id(element_id)
                    .flex_shrink_0()
                    .px(px(pad))
                    .text_size(px(size))
                    .text_color(color)
                    .child(text),
            );
            let label = if self.plain {
                label
            } else {
                label.font_weight(FontWeight(DIVIDER_TEXT_WEIGHT))
            };
            row = row
                .child(rail(&look, start))
                .child(label)
                .child(rail(&look, end));
            row.into_any_element()
        } else {
            let line = div().w_full().my(px(look.m.margin)).flex_shrink_0();
            let line = if look.dashed {
                line.border_t(px(look.m.width))
                    .border_dashed()
                    .border_color(look.style.line)
            } else {
                line.h(px(look.m.width)).bg(look.style.line)
            };
            line.into_any_element()
        }
    }
}

/// 白色圆角卡片容器（带边框与内边距）
pub fn card() -> Div {
    div()
        .flex()
        .flex_col()
        .gap_4()
        .p_5()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border())
        .rounded_lg()
}

/// 章节标题
pub fn section_title(title: impl Into<SharedString>) -> Stateful<Div> {
    let title = title.into();
    let semantics = Semantics::new(Role::Heading, title.clone())
        .id(format!("section-title-{title}"))
        .level(2);
    let element_id = semantics.element_id();
    semantics.apply(
        div()
            .id(element_id)
            .text_base()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(theme::text())
            .child(title),
    )
}

/// 次要说明文字
pub fn caption(text: impl Into<SharedString>) -> Div {
    div()
        .text_sm()
        .text_color(theme::text_muted())
        .child(text.into())
}

/// 表单行：左侧标签 + 右侧控件
pub fn field_row(label: impl Into<SharedString>) -> Div {
    div().flex().flex_row().items_center().gap_3().child(
        div()
            .w(px(90.))
            .flex_shrink_0()
            .text_sm()
            .text_color(theme::text_muted())
            .child(label.into()),
    )
}
