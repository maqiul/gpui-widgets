//! 简单图表：柱状图（BarChart）、折线图（LineChart）、迷你走势（Sparkline）。
//!
//! 全部由 GPUI 原生绘制（div / Path），无第三方图表依赖。

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, Hsla, InspectorElementId, IntoElement,
    LayoutId, PathBuilder, Pixels, Role, SharedString, Style, Window, accesskit, div, point,
    prelude::*, px, relative,
};

use crate::{a11y, theme};

/// 内容派生的稳定图表 ID；宿主可通过各图表的 `id` 方法覆盖以区分同数据实例。
pub(crate) fn chart_id(kind: &str, signature: impl std::fmt::Debug) -> ElementId {
    let mut hasher = DefaultHasher::new();
    hasher.write(kind.as_bytes());
    hasher.write(format!("{signature:?}").as_bytes());
    ElementId::Name(format!("chart-{kind}-{:016x}", hasher.finish()).into())
}

fn id_text(id: &ElementId) -> String {
    format!("{id:?}")
}

fn values_summary(values: &[f32]) -> String {
    let mut summary = values
        .iter()
        .take(8)
        .map(|value| format_value(*value))
        .collect::<Vec<_>>()
        .join("、");
    if values.len() > 8 {
        summary.push_str(&format!("等 {} 项", values.len()));
    }
    summary
}

// ---------------------------------------------------------------------------
// BarChart
// ---------------------------------------------------------------------------

/// 柱状图数据项
pub struct BarDatum {
    label: SharedString,
    value: f32,
    color: Option<Hsla>,
}

impl BarDatum {
    pub fn new(label: impl Into<SharedString>, value: f32) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// 柱状图。
///
/// ```ignore
/// BarChart::new(vec![
///     BarDatum::new("一月", 120.),
///     BarDatum::new("二月", 180.),
/// ])
/// ```
#[derive(IntoElement)]
pub struct BarChart {
    id: ElementId,
    data: Vec<BarDatum>,
    height: Pixels,
    max: Option<f32>,
}

impl BarChart {
    pub fn new(data: Vec<BarDatum>) -> Self {
        let id = chart_id(
            "bar",
            data.iter()
                .map(|datum| (datum.label.as_ref(), datum.value))
                .collect::<Vec<_>>(),
        );
        Self {
            id,
            data,
            height: px(160.),
            max: None,
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 图表区域高度（不含底部标签）
    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    /// 数值上限（默认取数据最大值）
    pub fn max(mut self, max: f32) -> Self {
        self.max = Some(max);
        self
    }
}

impl gpui::RenderOnce for BarChart {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let max = self
            .max
            .unwrap_or_else(|| self.data.iter().map(|d| d.value).fold(0., f32::max))
            .max(1e-6);
        // 顶部留出数值文字的空间
        let bar_area = self.height - px(18.);
        let root_a11y_id = id_text(&self.id);
        let summary = self
            .data
            .iter()
            .take(8)
            .map(|datum| format!("{} {}", datum.label, format_value(datum.value)))
            .collect::<Vec<_>>()
            .join("、");
        let summary = if self.data.len() > 8 {
            format!("{summary}等 {} 项", self.data.len())
        } else {
            summary
        };
        let count = self.data.len();

        let mut bars = div()
            .id(ElementId::Name(format!("{root_a11y_id}-bars").into()))
            .flex()
            .items_end()
            .gap_2()
            .h(self.height)
            .w_full();
        let mut labels = div().flex().gap_2().w_full().mt_1();

        for (index, datum) in self.data.iter().enumerate() {
            let ratio = (datum.value / max).clamp(0., 1.);
            let bar_height = (f32::from(bar_area) * ratio).max(2.);
            let color = datum.color.unwrap_or_else(theme::accent);
            let item_id = format!("{root_a11y_id}-bar-{index}");
            let semantics = a11y::Semantics::new(
                Role::ListItem,
                format!("{}：{}", datum.label, format_value(datum.value)),
            )
            .id(item_id.clone())
            .numeric(datum.value as f64)
            .position(index + 1, count);

            bars = bars.child(
                semantics.apply(
                    div()
                        .id(ElementId::Name(item_id.into()))
                        .flex_1()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_end()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child(format_value(datum.value)),
                        )
                        .child(div().w_full().h(px(bar_height)).rounded_t(px(3.)).bg(color)),
                ),
            );
            labels = labels.child(
                div()
                    .flex_1()
                    .text_center()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child(datum.label.clone()),
            );
        }

        let semantics =
            a11y::Semantics::new(Role::Figure, format!("柱状图：{count} 项；{summary}"))
                .id(root_a11y_id.clone());
        let bars = a11y::Semantics::new(Role::List, "柱状图数据项")
            .id(format!("{root_a11y_id}-bars"))
            .apply(bars);

        semantics
            .apply(div().id(self.id).w_full())
            .child(bars)
            .child(labels)
    }
}

// ---------------------------------------------------------------------------
// LineChart / Sparkline
// ---------------------------------------------------------------------------

/// 折线图。
///
/// ```ignore
/// LineChart::new(vec![3., 7., 5., 9., 6.])
///     .labels(vec!["周一".into(), "周二".into(), /* ... */])
///     .area(true)
/// ```
#[derive(IntoElement)]
pub struct LineChart {
    id: ElementId,
    data: Vec<f32>,
    labels: Vec<SharedString>,
    height: Pixels,
    color: Hsla,
    area: bool,
    dots: bool,
    max: Option<f32>,
}

impl LineChart {
    pub fn new(data: Vec<f32>) -> Self {
        let id = chart_id("line", &data);
        Self {
            id,
            data,
            labels: Vec::new(),
            height: px(160.),
            color: theme::accent(),
            area: false,
            dots: true,
            max: None,
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 底部 X 轴标签（与数据点一一对应，数量不足时只显示前几个）
    pub fn labels(mut self, labels: Vec<SharedString>) -> Self {
        self.id = chart_id(
            "line",
            (
                &self.data,
                labels
                    .iter()
                    .map(|label| label.as_ref())
                    .collect::<Vec<_>>(),
            ),
        );
        self.labels = labels;
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = color.into();
        self
    }

    /// 折线下方填充半透明面积
    pub fn area(mut self, area: bool) -> Self {
        self.area = area;
        self
    }

    /// 数据点圆点（默认开）
    pub fn dots(mut self, dots: bool) -> Self {
        self.dots = dots;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = Some(max);
        self
    }
}

impl gpui::RenderOnce for LineChart {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        LineElement {
            id: self.id,
            data: self.data,
            labels: self.labels,
            height: self.height,
            color: self.color,
            area: self.area,
            dots: self.dots,
            max: self.max,
            grid: true,
            sparkline: false,
        }
    }
}

/// 迷你走势图：无坐标轴的小折线，适合嵌入表格 / 卡片。
///
/// ```ignore
/// Sparkline::new(vec![1., 4., 2., 8., 5.]).w(px(120.)).h(px(32.))
/// ```
#[derive(IntoElement)]
pub struct Sparkline {
    id: ElementId,
    data: Vec<f32>,
    width: Pixels,
    height: Pixels,
    color: Hsla,
    area: bool,
}

impl Sparkline {
    pub fn new(data: Vec<f32>) -> Self {
        let id = chart_id("sparkline", &data);
        Self {
            id,
            data,
            width: px(120.),
            height: px(32.),
            color: theme::accent(),
            area: true,
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn w(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    pub fn h(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = color.into();
        self
    }

    pub fn area(mut self, area: bool) -> Self {
        self.area = area;
        self
    }
}

impl gpui::RenderOnce for Sparkline {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().w(self.width).h(self.height).child(LineElement {
            id: self.id,
            data: self.data,
            labels: Vec::new(),
            height: self.height,
            color: self.color,
            area: self.area,
            dots: false,
            max: None,
            grid: false,
            sparkline: true,
        })
    }
}

/// 折线绘制的内部元素
struct LineElement {
    id: ElementId,
    data: Vec<f32>,
    labels: Vec<SharedString>,
    height: Pixels,
    color: Hsla,
    area: bool,
    dots: bool,
    max: Option<f32>,
    grid: bool,
    sparkline: bool,
}

impl LineElement {
    fn semantics(&self) -> a11y::Semantics {
        let name = if self.sparkline {
            "迷你走势图"
        } else {
            "折线图"
        };
        let (min, max) = self
            .data
            .iter()
            .copied()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                (min.min(value), max.max(value))
            });
        let mut semantics = a11y::Semantics::new(
            Role::Figure,
            if self.data.is_empty() {
                format!("{name}：无数据")
            } else {
                format!(
                    "{name}：{} 个数据点，最小 {}，最大 {}，最新 {}；{}",
                    self.data.len(),
                    format_value(min),
                    format_value(max),
                    format_value(self.data[self.data.len() - 1]),
                    values_summary(&self.data)
                )
            },
        )
        .id(id_text(&self.id));
        if let Some(latest) = self.data.last() {
            semantics = semantics.numeric(*latest as f64);
        }
        semantics
    }
}

impl Element for LineElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn a11y_role(&self) -> Option<Role> {
        Some(self.semantics().role())
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        self.semantics().write(node);
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = self.height.into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.data.is_empty() {
            return;
        }

        let label_h = if self.labels.is_empty() {
            px(0.)
        } else {
            px(18.)
        };
        let pad = px(4.);
        let chart_left = bounds.left() + pad;
        let chart_right = bounds.right() - pad;
        let chart_top = bounds.top() + pad;
        let chart_bottom = bounds.bottom() - label_h - pad;
        let chart_w = chart_right - chart_left;
        let chart_h = chart_bottom - chart_top;
        if chart_w <= px(0.) || chart_h <= px(0.) {
            return;
        }

        let max = self
            .max
            .unwrap_or_else(|| self.data.iter().copied().fold(0., f32::max))
            .max(1e-6);

        // 网格线
        if self.grid {
            for i in 0..=3 {
                let y = chart_top + chart_h * (i as f32 / 3.);
                let mut grid = PathBuilder::stroke(px(1.));
                grid.move_to(point(chart_left, y));
                grid.line_to(point(chart_right, y));
                if let Ok(path) = grid.build() {
                    window.paint_path(path, theme::border());
                }
            }
        }

        // 数据点坐标
        let n = self.data.len();
        let points: Vec<_> = self
            .data
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let x = if n == 1 {
                    chart_left + chart_w / 2.
                } else {
                    chart_left + chart_w * (i as f32 / (n - 1) as f32)
                };
                let y = chart_bottom - chart_h * (v / max).clamp(0., 1.);
                point(x, y)
            })
            .collect();

        // 面积填充
        if self.area && n >= 2 {
            let mut fill = PathBuilder::fill();
            fill.move_to(point(points[0].x, chart_bottom));
            for p in &points {
                fill.line_to(*p);
            }
            fill.line_to(point(points[n - 1].x, chart_bottom));
            fill.close();
            if let Ok(path) = fill.build() {
                let mut soft = self.color;
                soft.a = 0.15;
                window.paint_path(path, soft);
            }
        }

        // 折线
        if n >= 2 {
            let mut line = PathBuilder::stroke(px(2.));
            line.move_to(points[0]);
            for p in &points[1..] {
                line.line_to(*p);
            }
            if let Ok(path) = line.build() {
                window.paint_path(path, self.color);
            }
        }

        // 圆点
        if self.dots {
            for p in &points {
                let mut dot = PathBuilder::fill();
                let r = px(3.);
                let segments = 12;
                for i in 0..=segments {
                    let angle = std::f32::consts::TAU * i as f32 / segments as f32;
                    let q = point(p.x + r * angle.cos(), p.y + r * angle.sin());
                    if i == 0 {
                        dot.move_to(q);
                    } else {
                        dot.line_to(q);
                    }
                }
                dot.close();
                if let Ok(path) = dot.build() {
                    window.paint_path(path, self.color);
                }
            }
        }

        // X 轴标签
        if !self.labels.is_empty() {
            let style = window.text_style();
            let font_size = px(11.);
            for (i, label) in self.labels.iter().enumerate().take(n) {
                let run = gpui::TextRun {
                    len: label.len(),
                    font: style.font(),
                    color: theme::text_muted(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let line = window
                    .text_system()
                    .shape_line(label.clone(), font_size, &[run], None);
                let origin = point(points[i].x - line.width / 2., bounds.bottom() - px(14.));
                let _ = line.paint(
                    origin,
                    px(14.),
                    gpui::TextAlign::default(),
                    None,
                    window,
                    cx,
                );
            }
        }
    }
}

/// 数值格式化：整数不带小数点
fn format_value(value: f32) -> String {
    if (value - value.round()).abs() < 1e-6 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.1}")
    }
}
