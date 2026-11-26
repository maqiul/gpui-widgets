//! 卡片容器：标题 / 额外操作 / 封面 / 内容 / 底部操作栏（对应 antd Card）。

use gpui::{
    AnyElement, BoxShadow, ElementId, Hsla, IntoElement, RenderOnce, Role, SharedString, div,
    prelude::*, px,
};

use crate::a11y;
use crate::theme::{self, Tokens, metric};

/// 卡片外观（antd Card token）：容器底 `colorBgContainer`、描边 `colorBorderSecondary`、
/// 圆角 `borderRadiusLG`；标题 `colorTextHeading` + strong，正文 `colorText`，
/// 操作栏 `colorTextDescription`、悬停主色。
#[derive(Clone, Debug, PartialEq)]
pub struct CardStyle {
    pub bg: Hsla,
    pub border: Hsla,
    pub title: Hsla,
    pub extra: Hsla,
    pub text: Hsla,
    pub actions: Hsla,
    pub action_hover: Hsla,
    pub radius: f32,
    pub header_height: f32,
    pub header_padding: f32,
    pub header_font_size: f32,
    pub body_padding: f32,
    pub actions_padding: f32,
    pub actions_font_size: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub border_width: f32,
    pub divider_width: f32,
    pub shadow: Vec<BoxShadow>,
}

pub fn card_state(tokens: &Tokens, bordered: bool) -> CardStyle {
    let m = metric::CardMetric::resolve(tokens);
    CardStyle {
        bg: tokens.color_bg_container,
        border: tokens.color_border_secondary,
        title: tokens.color_text_heading,
        extra: tokens.color_text,
        text: tokens.color_text,
        actions: tokens.color_text_description,
        action_hover: tokens.color_primary,
        radius: m.radius,
        header_height: m.header_height,
        header_padding: m.header_padding,
        header_font_size: m.header_font_size,
        body_padding: m.body_padding,
        actions_padding: m.actions_padding,
        actions_font_size: m.actions_font_size,
        font_size: m.font_size,
        line_height: m.line_height,
        border_width: if bordered { tokens.line_width } else { 0.0 },
        divider_width: tokens.line_width,
        shadow: if bordered {
            Vec::new()
        } else {
            tokens.shadow_tertiary.0.clone()
        },
    }
}

/// 通用卡片。
///
/// ```ignore
/// Card::new("卡片标题")
///     .extra(Link::new("更多", "#"))
///     .child("卡片内容")
///     .action(Button::new("操作一"))
///     .action(Button::new("操作二"))
/// ```
#[derive(IntoElement)]
pub struct Card {
    title: SharedString,
    extra: Option<AnyElement>,
    cover: Option<AnyElement>,
    actions: Vec<AnyElement>,
    bordered: bool,
    hoverable: bool,
    children: Vec<AnyElement>,
}

impl Card {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            extra: None,
            cover: None,
            actions: Vec::new(),
            bordered: true,
            hoverable: false,
            children: Vec::new(),
        }
    }

    /// 标题栏右侧的额外元素
    pub fn extra(mut self, extra: impl IntoElement) -> Self {
        self.extra = Some(extra.into_any_element());
        self
    }

    /// 封面（标题栏之上，如图片）
    pub fn cover(mut self, cover: impl IntoElement) -> Self {
        self.cover = Some(cover.into_any_element());
        self
    }

    /// 底部操作栏追加一项（均分宽度）
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    /// 是否显示边框（默认 true）
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// 悬停时浮起阴影（antd hoverable）
    pub fn hoverable(mut self) -> Self {
        self.hoverable = true;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for Card {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let style = theme::with(|t| card_state(t, self.bordered));
        let card_id = format!("card-{}", self.title);
        let mut card = a11y::Semantics::new(Role::Region, self.title.clone())
            .id(card_id.clone())
            .apply(div().id(ElementId::Name(card_id.into())))
            .flex()
            .flex_col()
            .w_full()
            .bg(style.bg)
            .rounded(px(style.radius))
            .overflow_hidden()
            .text_size(px(style.font_size))
            .line_height(px(style.line_height))
            .shadow(style.shadow.clone());
        if style.border_width > 0.0 {
            card = card
                .border(px(style.border_width))
                .border_color(style.border);
        }
        if self.hoverable {
            let shadow = style.shadow.clone();
            let hover_bg = style.bg;
            card = card.hover(move |s| s.shadow(shadow.clone()).border_color(hover_bg));
        }

        // 标题栏
        let mut header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap_3()
            .min_h(px(style.header_height))
            .px(px(style.header_padding))
            .child(
                a11y::Semantics::new(Role::Heading, self.title.clone())
                    .id(format!("card-{}-heading", self.title))
                    .level(2)
                    .apply(div().id(ElementId::Name(
                        format!("card-{}-heading", self.title).into(),
                    )))
                    .text_size(px(style.header_font_size))
                    .line_height(px(style.line_height))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(style.title)
                    .child(self.title.clone()),
            );
        header = header
            .border_b(px(style.divider_width))
            .border_color(style.border);
        if let Some(extra) = self.extra {
            header = header.child(extra);
        }
        card = card.child(header);

        // 封面（antd 中位于标题栏之后）
        if let Some(cover) = self.cover {
            let mut cover = div().w_full().child(cover);
            if style.border_width > 0.0 {
                // antd 封面用 -1px 外边距压住卡片描边
                cover = cover
                    .mt(px(-style.border_width))
                    .mx(px(-style.border_width));
            }
            card = card.child(cover);
        }

        // 内容区
        card = card.child(
            div()
                .flex()
                .flex_col()
                .p(px(style.body_padding))
                .text_size(px(style.font_size))
                .line_height(px(style.line_height))
                .text_color(style.text)
                .children(self.children),
        );

        // 底部操作栏
        if !self.actions.is_empty() {
            let border = style.border;
            let divider_width = style.divider_width;
            let action_hover = style.action_hover;
            let mut actions = div()
                .flex()
                .flex_row()
                .border_t(px(divider_width))
                .border_color(border);
            for (ix, action) in self.actions.into_iter().enumerate() {
                let mut cell = div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .my(px(style.actions_padding))
                    .text_size(px(style.actions_font_size))
                    .line_height(px(style.line_height))
                    .text_color(style.actions)
                    .hover(move |s| s.text_color(action_hover));
                if ix > 0 {
                    cell = cell.border_l(px(divider_width)).border_color(border);
                }
                actions = actions.child(cell.child(action));
            }
            card = card.child(actions);
        }

        card
    }
}
