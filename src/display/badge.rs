//! 徽章控件：小型状态标签（Badge）与右上角计数角标（CountBadge）。

use gpui::{
    AnyElement, BoxShadow, Context, Div, Hsla, IntoElement, Render, Role, SharedString, Stateful,
    Window, div, prelude::*, px,
};

use crate::{
    a11y::Semantics,
    overlay::tooltip::tooltip_bubble,
    theme::{self, Tokens, metric},
};

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

/// antd Badge 的预设状态点颜色。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BadgeStatus {
    Default,
    Processing,
    Success,
    Error,
    Warning,
}

/// 计数角标尺寸；状态点文本始终使用正文字号。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BadgeSize {
    #[default]
    Default,
    Small,
}

pub fn badge_variant_colors(
    tokens: &Tokens,
    variant: BadgeVariant,
    custom: Option<Hsla>,
) -> (Hsla, Hsla) {
    if let Some(color) = custom {
        return (Hsla { a: 0.12, ..color }, color);
    }
    match variant {
        BadgeVariant::Neutral => (tokens.control_item_bg_hover, tokens.color_text_secondary),
        BadgeVariant::Info => (tokens.color_primary_bg, tokens.color_primary),
        BadgeVariant::Success => (tokens.color_success_bg, tokens.color_success),
        BadgeVariant::Warning => (tokens.color_warning_bg, tokens.color_warning),
        BadgeVariant::Danger => (tokens.color_error_bg, tokens.color_error),
    }
}

pub fn badge_status_color(tokens: &Tokens, status: BadgeStatus, custom: Option<Hsla>) -> Hsla {
    if let Some(color) = custom {
        return color;
    }
    match status {
        BadgeStatus::Default => tokens.color_text_placeholder,
        BadgeStatus::Processing => tokens.color_info,
        BadgeStatus::Success => tokens.color_success,
        BadgeStatus::Error => tokens.color_error,
        BadgeStatus::Warning => tokens.color_warning,
    }
}

/// 通用徽章。
///
/// ```ignore
/// Badge::new("运行中").success()
/// Badge::new("成功").status(BadgeStatus::Success)
/// ```
pub struct Badge {
    id: Option<SharedString>,
    origin: SharedString,
    label: SharedString,
    variant: BadgeVariant,
    custom: Option<Hsla>,
    status: Option<BadgeStatus>,
    title: Option<SharedString>,
    aria_label: Option<SharedString>,
}

impl Badge {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "badge-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            label: label.into(),
            variant: BadgeVariant::default(),
            custom: None,
            status: None,
            title: None,
            aria_label: None,
        }
    }

    /// 实例 ID；同页多个同文本徽章应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 更新显示文本。
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.label = text.into();
        self
    }

    /// 状态点模式；设置后显示「色点 + 文本」，不再显示胶囊底色。
    pub fn status(mut self, status: BadgeStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// 自定义颜色（胶囊模式文字用该色、背景用其 12% 透明度；状态点模式用于色点）。
    pub fn color(mut self, color: Hsla) -> Self {
        self.custom = Some(color);
        self
    }

    /// 悬停提示，同时写入无障碍描述。
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 无障碍名称；不设置时使用显示文本。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
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

struct BadgeTitleView {
    text: SharedString,
}

impl Render for BadgeTitleView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        tooltip_bubble(self.text.clone())
    }
}

fn with_badge_title(element: Stateful<Div>, title: Option<SharedString>) -> Stateful<Div> {
    match title {
        Some(text) => {
            element.tooltip(move |_, cx| cx.new(|_| BadgeTitleView { text: text.clone() }).into())
        }
        None => element,
    }
}

impl IntoElement for Badge {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let id = self.id.clone().unwrap_or_else(|| self.origin.clone());
        let accessible_label = self
            .aria_label
            .clone()
            .unwrap_or_else(|| self.label.clone());
        let mut semantics = Semantics::new(Role::Status, accessible_label).id(id);
        if let Some(title) = &self.title {
            semantics = semantics.description(title.clone());
        }
        let element_id = semantics.element_id();

        let element = if let Some(status) = self.status {
            let (dot_color, text_color, font_size, gap) = theme::with(|tokens| {
                (
                    badge_status_color(tokens, status, self.custom),
                    tokens.color_text,
                    tokens.font_size,
                    tokens.margin_xs,
                )
            });
            div()
                .id(element_id)
                .flex()
                .items_center()
                .child(
                    div()
                        .relative()
                        .top(px(-1.))
                        .size(px(6.))
                        .rounded_full()
                        .bg(dot_color),
                )
                .child(
                    div()
                        .ml(px(gap))
                        .text_size(px(font_size))
                        .text_color(text_color)
                        .child(self.label),
                )
        } else {
            let (bg, fg) =
                theme::with(|tokens| badge_variant_colors(tokens, self.variant, self.custom));
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
                .child(self.label)
        };

        with_badge_title(semantics.apply(element), self.title)
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

pub fn badge_state_sized(
    tokens: &Tokens,
    dot: bool,
    color: Option<Hsla>,
    size: BadgeSize,
) -> BadgeStyle {
    let mut style = badge_state(tokens, dot, color);
    if size == BadgeSize::Small {
        style.height = 14.;
        style.min_width = 14.;
        style.radius = 7.;
        style.line_height = 14.;
    }
    style
}

fn badge_count_width(style: &BadgeStyle, chars: usize) -> f32 {
    if chars <= 1 {
        style.min_width
    } else {
        style.min_width.max(16. + chars as f32 * 7.)
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
    size: BadgeSize,
    title: Option<SharedString>,
    aria_label: Option<SharedString>,
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
            size: BadgeSize::Default,
            title: None,
            aria_label: None,
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

    /// 计数角标尺寸；状态点文本不受影响。
    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    /// 悬停提示，同时写入无障碍描述。
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 无障碍名称；不设置时使用「N 条消息」。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
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
            let style = theme::with(|t| badge_state_sized(t, self.dot, self.color, self.size));
            let (dx, dy) = self.offset;
            let badge_label = if self.dot {
                SharedString::from("有新消息")
            } else if self.count > self.max {
                SharedString::from(format!("{} 条以上消息", self.max))
            } else {
                SharedString::from(format!("{} 条消息", self.count))
            };
            let accessible_label = self
                .aria_label
                .clone()
                .unwrap_or_else(|| badge_label.clone());
            let mut semantics = Semantics::new(Role::Status, accessible_label).id(format!(
                "{}-{}-{}",
                self.id.as_deref().unwrap_or(&self.origin),
                self.count,
                self.dot
            ));
            if let Some(title) = &self.title {
                semantics = semantics.description(title.clone());
            }
            let element_id = semantics.element_id();
            let title = self.title.clone();
            let badge = if self.dot {
                let half = style.dot_size / 2.0;
                let element = semantics.apply(
                    div()
                        .id(element_id)
                        .absolute()
                        .top(px(-half + dy))
                        .right(px(-half - dx))
                        .size(px(style.dot_size))
                        .rounded_full()
                        .bg(style.bg)
                        .shadow(vec![style.ring]),
                );
                with_badge_title(element, title)
            } else {
                let label = if self.count > self.max {
                    format!("{}+", self.max)
                } else {
                    self.count.to_string()
                };
                let width = badge_count_width(&style, label.chars().count());
                let element = semantics.apply(
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
                );
                with_badge_title(element, title)
            };
            wrapper = wrapper.child(badge);
        }

        wrapper
    }
}
