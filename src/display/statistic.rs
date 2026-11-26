//! 数值统计卡：大数字 + 标题 + 可选趋势。

use gpui::{
    App, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    Styled, Window, div, px,
};

use crate::a11y::Semantics;
use crate::theme::{self, Tokens, metric::StatisticMetric};

/// 趋势方向
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    Up,
    Down,
}

/// 统计卡的纯数据外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatisticStyle {
    pub title: Hsla,
    pub content: Hsla,
    pub trend_up: Hsla,
    pub trend_down: Hsla,
}

/// 标题 `colorTextDescription`、内容与前缀/后缀 `colorTextHeading`；
/// 趋势沿用功能色（`colorSuccess` / `colorError`，antd 无此特性）。
pub fn statistic_style(tokens: &Tokens) -> StatisticStyle {
    StatisticStyle {
        title: tokens.color_text_description,
        content: tokens.color_text_heading,
        trend_up: tokens.color_success,
        trend_down: tokens.color_error,
    }
}

/// 数值统计卡。
///
/// ```ignore
/// Statistic::new("月活跃用户", "28,431")
///     .suffix("人")
///     .trend(Trend::Up, "12.5%")
/// ```
#[derive(IntoElement)]
pub struct Statistic {
    title: SharedString,
    value: SharedString,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
    trend: Option<(Trend, SharedString)>,
}

impl Statistic {
    pub fn new(title: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            value: value.into(),
            prefix: None,
            suffix: None,
            trend: None,
        }
    }

    /// 数值前缀（如 "¥"）
    pub fn prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// 数值后缀（如 "人" / "%"）
    pub fn suffix(mut self, suffix: impl Into<SharedString>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// 趋势（上升绿色 / 下降红色）
    pub fn trend(mut self, trend: Trend, text: impl Into<SharedString>) -> Self {
        self.trend = Some((trend, text.into()));
        self
    }
}

impl RenderOnce for Statistic {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (style, m) =
            theme::with(|tokens| (statistic_style(tokens), StatisticMetric::resolve(tokens)));
        let mut display_value = String::new();
        if let Some(prefix) = &self.prefix {
            display_value.push_str(prefix);
        }
        display_value.push_str(&self.value);
        if let Some(suffix) = &self.suffix {
            display_value.push_str(suffix);
        }
        let mut semantics = Semantics::new(Role::Status, self.title.clone())
            .id(format!("statistic-{}-{}", self.title, self.value))
            .value(SharedString::from(display_value));
        if let Some((trend, text)) = &self.trend {
            semantics = semantics.description(SharedString::from(match trend {
                Trend::Up => format!("上升 {text}"),
                Trend::Down => format!("下降 {text}"),
            }));
        }
        let element_id = semantics.element_id();

        let mut value_row = div()
            .flex()
            .items_baseline()
            .gap(px(m.affix_gap))
            .text_size(px(m.content_size))
            .text_color(style.content);
        if let Some(prefix) = &self.prefix {
            value_row = value_row.child(div().child(prefix.clone()));
        }
        value_row = value_row.child(div().child(self.value.clone()));
        if let Some(suffix) = &self.suffix {
            value_row = value_row.child(div().child(suffix.clone()));
        }

        let mut root = semantics.apply(
            div()
                .id(element_id)
                .flex()
                .flex_col()
                .child(
                    div()
                        .pb(px(m.header_gap))
                        .text_size(px(m.title_size))
                        .text_color(style.title)
                        .child(self.title.clone()),
                )
                .child(value_row),
        );

        if let Some((trend, text)) = &self.trend {
            let (arrow, color) = match trend {
                Trend::Up => ("▲", style.trend_up),
                Trend::Down => ("▼", style.trend_down),
            };
            root = root.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(m.trend_gap))
                    .text_size(px(m.trend_size))
                    .text_color(color)
                    .child(arrow)
                    .child(text.clone()),
            );
        }

        root
    }
}
