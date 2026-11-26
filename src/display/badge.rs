//! 徽章控件：小型状态标签（Badge）与右上角计数角标（CountBadge）。

use gpui::{
    AnyElement, BoxShadow, Div, Hsla, IntoElement, Role, SharedString, Stateful, div, prelude::*,
    px,
};

use crate::a11y::Semantics;
use crate::theme::{self, Tokens, metric};

/// 徽章配色变体
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

/// 通用徽章。
///
/// ```ignore
/// Badge::new("运行中").success()
/// ```
pub struct Badge {
    label: SharedString,
    variant: BadgeVariant,
    custom: Option<Hsla>,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::default(),
            custom: None,
        }
    }

    /// 自定义颜色（文字用该色，背景用其 12% 透明度）
    pub fn color(mut self, color: Hsla) -> Self {
        self.custom = Some(color);
        self
    }

    pub fn info(mut self) -> Self {
        self.variant = BadgeVariant::Info;
        self
    }

    pub fn success(mut self) -> Self {
        self.variant = BadgeVariant::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.variant = BadgeVariant::Warning;
        self
    }

    pub fn danger(mut self) -> Self {
        self.variant = BadgeVariant::Danger;
        self
    }
}

impl IntoElement for Badge {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let (bg, fg): (Hsla, Hsla) = match self.custom {
            Some(c) => (Hsla { a: 0.12, ..c }, c),
            None => match self.variant {
                BadgeVariant::Neutral => (theme::hover_bg(), theme::text_muted()),
                BadgeVariant::Info => (theme::accent_soft(), theme::accent()),
                BadgeVariant::Success => (theme::success_soft(), theme::success()),
                BadgeVariant::Warning => (theme::warning_soft(), theme::warning()),
                BadgeVariant::Danger => (theme::danger_soft(), theme::danger()),
            },
        };

        let semantics =
            Semantics::new(Role::Status, self.label.clone()).id(format!("badge-{}", self.label));
        let element_id = semantics.element_id();
        semantics.apply(
            div()
                .id(element_id)
                .flex()
                .items_center()
                .px_2()
                .py(px(2.))
                .rounded_full()
                .bg(bg)
                .text_color(fg)
                .text_xs()
                .child(self.label),
        )
    }
}

/// 计数角标 / 圆点外观（antdv-next Badge 实测）：底 `colorError`、文字白色、
/// 高 20 / 圆角 10 / 字号 12；圆点 6×6；外圈 1px 容器色。
#[derive(Clone, Debug)]
pub struct BadgeStyle {
    pub bg: Hsla,
    pub text: Hsla,
    pub height: f32,
    pub min_width: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub dot_size: f32,
    pub ring: BoxShadow,
}

pub fn badge_state(tokens: &Tokens, _dot: bool, color: Option<Hsla>) -> BadgeStyle {
    let m = metric::BadgeMetric::resolve(tokens);
    BadgeStyle {
        bg: color.unwrap_or(tokens.color_error),
        text: tokens.color_text_light_solid,
        height: m.height,
        min_width: m.min_width,
        radius: m.radius,
        font_size: m.font_size,
        line_height: m.height,
        dot_size: m.dot,
        ring: metric::badge_ring_shadow(tokens.color_bg_container),
    }
}

/// 计数角标：包裹任意元素，在其右上角显示红色计数 / 圆点（对应 antd Badge）。
///
/// ```ignore
/// CountBadge::new(5).child(Button::new("消息"))
/// CountBadge::new(0).dot().child(Avatar::new("张"))
/// ```
#[derive(IntoElement)]
pub struct CountBadge {
    id: Option<SharedString>,
    origin: SharedString,
    count: u32,
    max: u32,
    dot: bool,
    show_zero: bool,
    color: Option<Hsla>,
    offset: (f32, f32),
    child: Option<AnyElement>,
}

impl CountBadge {
    #[track_caller]
    pub fn new(count: u32) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "count-badge-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            count,
            max: 99,
            dot: false,
            show_zero: false,
            color: None,
            offset: (0., 0.),
            child: None,
        }
    }

    /// 实例 ID；同页多个角标应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 自定义角标底色（默认危险红）
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// 角标偏移（x 向右、y 向下，像素）
    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = (x, y);
        self
    }

    /// 封顶值，超出显示 "{max}+"（默认 99）
    pub fn max(mut self, max: u32) -> Self {
        self.max = max;
        self
    }

    /// 只显示小红点（不显示数字）
    pub fn dot(mut self) -> Self {
        self.dot = true;
        self
    }

    /// count 为 0 时仍显示角标
    pub fn show_zero(mut self) -> Self {
        self.show_zero = true;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
}

impl RenderOnce for CountBadge {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let mut wrapper = div().relative().flex();
        if let Some(child) = self.child {
            wrapper = wrapper.child(child);
        }

        let visible = self.dot || self.count > 0 || self.show_zero;
        if visible {
            let style = theme::with(|t| badge_state(t, self.dot, self.color));
            let (dx, dy) = self.offset;
            let badge_label = if self.dot {
                SharedString::from("有新消息")
            } else if self.count > self.max {
                SharedString::from(format!("{} 条以上消息", self.max))
            } else {
                SharedString::from(format!("{} 条消息", self.count))
            };
            let semantics = Semantics::new(Role::Status, badge_label).id(format!(
                "{}-{}-{}",
                self.id.as_deref().unwrap_or(&self.origin),
                self.count,
                self.dot
            ));
            let element_id = semantics.element_id();
            let badge = if self.dot {
                let half = style.dot_size / 2.0;
                semantics.apply(
                    div()
                        .id(element_id)
                        .absolute()
                        .top(px(-half + dy))
                        .right(px(-half - dx))
                        .size(px(style.dot_size))
                        .rounded_full()
                        .bg(style.bg)
                        .shadow(vec![style.ring]),
                )
            } else {
                let label = if self.count > self.max {
                    format!("{}+", self.max)
                } else {
                    self.count.to_string()
                };
                let m = theme::with(|t| metric::BadgeMetric::resolve(t));
                let width = m.count_width(label.chars().count());
                semantics.apply(
                    div()
                        .id(element_id)
                        .absolute()
                        .top(px(-style.height / 2.0 + dy))
                        .right(px(-width / 2.0 - dx))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(width))
                        .h(px(style.height))
                        .rounded(px(style.radius))
                        .bg(style.bg)
                        .text_color(style.text)
                        .text_size(px(style.font_size))
                        .line_height(px(style.line_height))
                        .shadow(vec![style.ring])
                        .child(label),
                )
            };
            wrapper = wrapper.child(badge);
        }

        wrapper
    }
}
