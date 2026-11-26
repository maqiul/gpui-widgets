//! 进度条控件：水平进度 / 分段进度 + 指示文字。
//!
//! 对齐 antdv-next Progress（`.ant-progress-body` 行盒、轨道/分段 + 指示器）。

use gpui::{Div, ElementId, Hsla, IntoElement, Role, Stateful, div, prelude::*, px, relative};

use crate::a11y::Semantics;
use crate::theme::{self, Tokens, metric};

/// 进度状态：正常 / 进行中 / 成功 / 异常。
/// antdv-next 的 `active` 只加类名，无独立颜色。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressStatus {
    Normal,
    Active,
    Success,
    Exception,
}

/// 进度条外观：轨道色随状态，指示器在成功 / 异常时换成同色图标。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressStyle {
    pub track: Hsla,
    pub rail: Hsla,
    pub text: Hsla,
    pub glyph: Hsla,
    pub stroke: f32,
    pub radius: f32,
    pub gap: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub steps_item_width: f32,
    pub steps_gap: f32,
}

pub fn progress_state(
    tokens: &Tokens,
    status: ProgressStatus,
    small: bool,
    show_info: bool,
) -> ProgressStyle {
    let m = metric::ProgressMetric::resolve(tokens);
    let pick = |pair: [f32; 2]| if small { pair[1] } else { pair[0] };
    let track = match status {
        ProgressStatus::Normal | ProgressStatus::Active => tokens.color_info,
        ProgressStatus::Success => tokens.color_success,
        ProgressStatus::Exception => tokens.color_error,
    };
    let icon = matches!(status, ProgressStatus::Success | ProgressStatus::Exception);
    let font_size = pick(m.font_size);
    ProgressStyle {
        track,
        rail: tokens.color_fill_secondary,
        text: if icon { track } else { tokens.color_text },
        glyph: tokens.color_text_light_solid,
        stroke: pick(m.stroke),
        radius: m.radius,
        gap: m.gap,
        font_size,
        line_height: if show_info { font_size } else { pick(m.stroke) },
        steps_item_width: pick(m.steps_item),
        steps_gap: m.steps_gap,
    }
}

/// 通用进度条，`value` 取值范围 0.0 ~ 1.0。
///
/// ```ignore
/// ProgressBar::new(self.volume / 100.)
///     .status(ProgressStatus::Success)
/// ```
pub struct ProgressBar {
    value: f32,
    id: Option<String>,
    origin: String,
    height: Option<f32>,
    steps: Option<usize>,
    status: ProgressStatus,
    show_info: bool,
    small: bool,
}

impl ProgressBar {
    #[track_caller]
    pub fn new(value: f32) -> Self {
        let location = core::panic::Location::caller();
        Self {
            value: value.clamp(0., 1.),
            id: None,
            origin: format!(
                "progress-bar-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            ),
            height: None,
            steps: None,
            status: ProgressStatus::Normal,
            show_info: true,
            small: false,
        }
    }

    /// 无障碍稳定 id；同页多个进度条应显式传入不同值。
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 自定义轨道高度（antd `strokeWidth`）
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// 分段进度：将进度条均分为 n 段，按进度点亮前几段（antd steps）
    pub fn steps(mut self, steps: usize) -> Self {
        self.steps = Some(steps.max(1));
        self
    }

    pub fn status(mut self, status: ProgressStatus) -> Self {
        self.status = status;
        self
    }

    pub fn show_info(mut self, show_info: bool) -> Self {
        self.show_info = show_info;
        self
    }

    pub fn small(mut self, small: bool) -> Self {
        self.small = small;
        self
    }
}

fn status_description(status: ProgressStatus) -> &'static str {
    match status {
        ProgressStatus::Normal => "正常",
        ProgressStatus::Active => "进行中",
        ProgressStatus::Success => "已完成",
        ProgressStatus::Exception => "异常",
    }
}

fn progress_semantics(value: f32, status: ProgressStatus, id: String) -> Semantics {
    Semantics::new(Role::ProgressIndicator, "进度")
        .id(id)
        .description(status_description(status))
        .value(format!("{}%", (value * 100.0).round() as u32))
        .numeric(f64::from(value))
        .numeric_range(0.0, 1.0)
}

fn indicator(style: &ProgressStyle, status: ProgressStatus, value: f32) -> Div {
    let mut text = div()
        .flex_shrink_0()
        .text_size(px(style.font_size))
        .line_height(px(style.line_height))
        .text_color(style.text);

    match status {
        // 成功 / 异常：antd 用 14px 圆形图标替换百分比
        ProgressStatus::Success | ProgressStatus::Exception => {
            text = text.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(style.font_size))
                    .rounded_full()
                    .bg(style.track)
                    .text_size(px(style.font_size * 0.64))
                    .line_height(px(style.font_size))
                    .text_color(style.glyph)
                    .child(if status == ProgressStatus::Success {
                        "✓"
                    } else {
                        "✕"
                    }),
            );
        }
        _ => text = text.child(format!("{}%", (value * 100.0).round() as u32)),
    }

    text
}

impl IntoElement for ProgressBar {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let style = theme::with(|t| progress_state(t, self.status, self.small, self.show_info));
        let value = self.value;
        let stroke = self.height.unwrap_or(style.stroke);
        let id = self.id.clone().unwrap_or_else(|| {
            format!("{}-{}", self.origin, (value * 100.0).round() as u32).into()
        });
        let semantics = progress_semantics(value, self.status, id.clone());

        if let Some(steps) = self.steps {
            let filled = (value * steps as f32).round() as usize;
            let mut body = div()
                .id(ElementId::Name(id.into()))
                .flex()
                .flex_row()
                .items_center()
                .h(px(style.line_height))
                .gap(px(style.steps_gap));
            for ix in 0..steps {
                body = body.child(
                    div()
                        .w(px(style.steps_item_width))
                        .h(px(stroke))
                        .bg(if ix < filled { style.track } else { style.rail }),
                );
            }
            if self.show_info {
                body = body.child(div().ml(px(style.gap)).child(indicator(
                    &style,
                    self.status,
                    value,
                )));
            }
            return semantics.apply(body);
        }

        let mut row = div()
            .id(ElementId::Name(id.into()))
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .gap(px(style.gap))
            .h(px(style.line_height))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .h(px(stroke))
                    .rounded(px(style.radius))
                    .overflow_hidden()
                    .bg(style.rail)
                    .child(
                        div()
                            .h_full()
                            .w(relative(value))
                            .rounded(px(style.radius))
                            .bg(style.track),
                    ),
            );

        if self.show_info {
            row = row.child(indicator(&style, self.status, value));
        }

        semantics.apply(row)
    }
}
