//! 提示条控件：整行的信息 / 成功 / 警告 / 错误通知，可带描述、图标与关闭按钮。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, IntoElement, Role, SharedString, Stateful, Window, div,
    prelude::*, px,
};

use crate::a11y::{self, Semantics};
use crate::theme::{self, Tokens, metric};

/// 提示条变体
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    Info,
    Success,
    Warning,
    Danger,
}

impl AlertVariant {
    fn index(self) -> usize {
        match self {
            AlertVariant::Info => 0,
            AlertVariant::Success => 1,
            AlertVariant::Warning => 2,
            AlertVariant::Danger => 3,
        }
    }
}

/// 提示条外观（antd Alert token）：底色 / 描边 / 图标色按变体，
/// 标题 `colorTextHeading`，描述 `colorText`，关闭图标 `colorIcon`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlertStyle {
    pub bg: Hsla,
    pub border: Hsla,
    pub icon: Hsla,
    pub title: Hsla,
    pub description: Hsla,
    pub close: Hsla,
    pub close_hover: Hsla,
    pub radius: f32,
    pub padding_v: f32,
    pub padding_h: f32,
    pub icon_gap: f32,
    pub icon_size: f32,
    pub title_size: f32,
    pub title_margin: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub close_size: f32,
    pub border_width: f32,
}

pub fn alert_state(
    tokens: &Tokens,
    variant: AlertVariant,
    with_description: bool,
    banner: bool,
) -> AlertStyle {
    let m = metric::AlertMetric::resolve(tokens);
    let (bg, border, icon) = match variant.index() {
        0 => (
            tokens.color_info_bg,
            tokens.color_info_border,
            tokens.color_info,
        ),
        1 => (
            tokens.color_success_bg,
            tokens.color_success_border,
            tokens.color_success,
        ),
        2 => (
            tokens.color_warning_bg,
            tokens.color_warning_border,
            tokens.color_warning,
        ),
        _ => (
            tokens.color_error_bg,
            tokens.color_error_border,
            tokens.color_error,
        ),
    };
    let state = usize::from(with_description);
    AlertStyle {
        bg,
        border,
        icon,
        title: tokens.color_text_heading,
        description: tokens.color_text,
        close: tokens.color_icon,
        close_hover: tokens.color_icon_hover,
        radius: if banner { 0.0 } else { m.radius },
        padding_v: m.padding[state].0,
        padding_h: m.padding[state].1,
        icon_gap: m.icon_gap[state],
        icon_size: m.icon_size[state],
        title_size: if with_description {
            m.description_title_size
        } else {
            m.title_size
        },
        title_margin: m.title_margin,
        font_size: m.font_size,
        line_height: m.line_height,
        close_size: m.close_size,
        border_width: if banner { 0.0 } else { tokens.line_width },
    }
}

fn variant_icon(variant: AlertVariant) -> &'static str {
    match variant {
        AlertVariant::Info => "ℹ",
        AlertVariant::Success => "✓",
        AlertVariant::Warning => "⚠",
        AlertVariant::Danger => "✕",
    }
}

/// 通用提示条。
///
/// ```ignore
/// Alert::new("保存成功").success().description("所有更改已同步到云端")
/// ```
pub struct Alert {
    title: SharedString,
    id: Option<SharedString>,
    description: Option<SharedString>,
    variant: AlertVariant,
    banner: bool,
    show_icon: Option<bool>,
    closable: bool,
    on_close: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl Alert {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            id: None,
            description: None,
            variant: AlertVariant::Info,
            banner: false,
            show_icon: None,
            closable: false,
            on_close: None,
        }
    }

    /// 实例唯一 id；不传则回退标题，同标题叠层需显式传 id。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn info(mut self) -> Self {
        self.variant = AlertVariant::Info;
        self
    }

    pub fn success(mut self) -> Self {
        self.variant = AlertVariant::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.variant = AlertVariant::Warning;
        self
    }

    pub fn danger(mut self) -> Self {
        self.variant = AlertVariant::Danger;
        self
    }

    /// 通栏模式：去圆角 / 边框，占满整行（antd banner）
    pub fn banner(mut self) -> Self {
        self.banner = true;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 是否显示变体图标；antd 默认非 banner 不显示、banner 显示
    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = Some(show);
        self
    }

    /// 显示关闭按钮
    pub fn on_close(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.closable = true;
        self.on_close = Some(Rc::new(listener));
        self
    }

    /// 只显示关闭按钮，不绑定回调
    pub fn closable(mut self) -> Self {
        self.closable = true;
        self
    }
}

/// 派生关闭钮 id：实例 id 优先，未传则回退标题。
fn close_id(id: Option<&str>, title: &str) -> String {
    format!("alert-close-{}", id.unwrap_or(title))
}

impl IntoElement for Alert {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let close_id = close_id(self.id.as_deref(), &self.title);
        let alert_id = format!(
            "alert-{}",
            self.id.as_deref().unwrap_or(self.title.as_ref())
        );
        let with_description = self.description.is_some();
        let style = theme::with(|t| alert_state(t, self.variant, with_description, self.banner));
        let mut semantics = Semantics::new(Role::Alert, self.title.clone()).id(alert_id.clone());
        if let Some(description) = &self.description {
            semantics = semantics.description(description.clone());
        }

        let mut row = div()
            .id(ElementId::Name(alert_id.into()))
            .flex()
            .flex_row()
            .w_full()
            .pt(px(style.padding_v))
            .pb(px(style.padding_v))
            .px(px(style.padding_h))
            .bg(style.bg)
            .rounded(px(style.radius))
            .text_size(px(style.font_size))
            .line_height(px(style.line_height));
        if with_description {
            row = row.items_start();
        } else {
            row = row.items_center();
        }
        if style.border_width > 0.0 {
            row = row
                .border(px(style.border_width))
                .border_color(style.border);
        }

        let show_icon = self.show_icon.unwrap_or(self.banner);
        if show_icon {
            row = row.child(
                div()
                    .flex_shrink_0()
                    .mr(px(style.icon_gap))
                    .text_size(px(style.icon_size))
                    .line_height(px(style.icon_size))
                    .text_color(style.icon)
                    .child(variant_icon(self.variant)),
            );
        }
        let mut body = div().flex().flex_col().flex_1().min_w_0().child(
            div()
                .text_size(px(style.title_size))
                .line_height(px(style.line_height))
                .text_color(style.title)
                .child(self.title),
        );
        if let Some(description) = self.description {
            body = body.child(
                div()
                    .mt(px(style.title_margin))
                    .text_size(px(style.font_size))
                    .line_height(px(style.line_height))
                    .text_color(style.description)
                    .child(description),
            );
        }
        row = row.child(body);

        if self.closable {
            let mut close = div()
                .id(ElementId::Name(close_id.clone().into()))
                .flex_shrink_0()
                .ml(px(style.icon_gap))
                .cursor(CursorStyle::PointingHand)
                .text_size(px(style.close_size))
                .line_height(px(style.close_size))
                .text_color(style.close)
                .hover(move |s| s.text_color(style.close_hover))
                .child("✕");
            let close_semantics = Semantics::new(Role::Button, "关闭提示")
                .id(close_id)
                .disabled(self.on_close.is_none())
                .focusable(self.on_close.is_some());
            if let Some(on_close) = self.on_close {
                close = a11y::click_void(close, move |window, cx| on_close(window, cx));
            }
            row = row.child(close_semantics.apply(close));
        }

        semantics.apply(row)
    }
}

#[cfg(test)]
mod tests {
    use super::close_id;

    #[test]
    fn alert_id_default_derives_from_title() {
        assert_eq!(close_id(None, "保存成功"), "alert-close-保存成功");
    }

    #[test]
    fn alert_id_stable_across_repeated_calls() {
        for _ in 0..3 {
            assert_eq!(close_id(None, "保存成功"), close_id(None, "保存成功"));
            assert_eq!(
                close_id(Some("save-ok"), "保存成功"),
                close_id(Some("save-ok"), "保存成功")
            );
        }
    }

    #[test]
    fn alert_id_distinct_titles_do_not_collide() {
        assert_ne!(close_id(None, "保存成功"), close_id(None, "保存警告"));
    }

    #[test]
    fn alert_id_explicit_overrides_title() {
        assert_eq!(
            close_id(Some("save-ok-7"), "保存成功"),
            "alert-close-save-ok-7"
        );
    }

    #[test]
    fn alert_id_same_title_still_collides() {
        assert_eq!(close_id(None, "保存成功"), close_id(None, "保存成功"));
        assert_ne!(
            close_id(Some("a"), "保存成功"),
            close_id(Some("b"), "保存成功")
        );
    }

    #[test]
    fn alert_id_empty_title_keeps_prefix() {
        assert_eq!(close_id(None, ""), "alert-close-");
    }
}
