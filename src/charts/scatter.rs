//! 散点图：GPU 路径绘制的多系列散点 + 网格 + 坐标轴 + 图例。
//!
//! ```ignore
//! ScatterChart::new(vec![
//!     ScatterSeries::new("甲组", vec![ScatterPoint::new(1., 2.), ScatterPoint::new(3., 5.)]),
//!     ScatterSeries::new("乙组", vec![ScatterPoint::new(2., 3.), ScatterPoint::new(4., 6.)]),
//! ]).height(px(200.))
//! ```

use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, Hsla, InspectorElementId, IntoElement,
    LayoutId, Pixels, Role, SharedString, Style, Window, accesskit, div, point, prelude::*, px,
    relative,
};

use crate::a11y;
use crate::chart::chart_id;
use crate::chart_util::{circle, format_value, paint_text};
use crate::theme;

/// 散点坐标（假设非负，原点 0）。
#[derive(Clone, Copy)]
pub struct ScatterPoint {
    pub x: f32,
    pub y: f32,
}

impl ScatterPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 散点系列。
pub struct ScatterSeries {
    name: SharedString,
    points: Vec<ScatterPoint>,
    color: Option<Hsla>,
}

impl ScatterSeries {
    pub fn new(name: impl Into<SharedString>, points: Vec<ScatterPoint>) -> Self {
        Self {
            name: name.into(),
            points,
            color: None,
        }
    }

    /// 自定义系列颜色（默认取主题分类色板）。
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// 散点图。
#[derive(IntoElement)]
pub struct ScatterChart {
    id: ElementId,
    series: Vec<ScatterSeries>,
    height: Pixels,
    x_max: Option<f32>,
    y_max: Option<f32>,
    legend: bool,
    dot_radius: Pixels,
}

impl ScatterChart {
    pub fn new(series: Vec<ScatterSeries>) -> Self {
        let id = chart_id(
            "scatter",
            series
                .iter()
                .map(|series| {
                    (
                        series.name.as_ref(),
                        series
                            .points
                            .iter()
                            .map(|point| (point.x, point.y))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
        );
        Self {
            id,
            series,
            height: px(200.),
            x_max: None,
            y_max: None,
            legend: true,
            dot_radius: px(4.),
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    /// X 轴上限（默认取数据最大值并留 5% 余量）。
    pub fn x_max(mut self, x_max: f32) -> Self {
        self.x_max = Some(x_max);
        self
    }

    /// Y 轴上限。
    pub fn y_max(mut self, y_max: f32) -> Self {
        self.y_max = Some(y_max);
        self
    }

    /// 是否显示底部图例（默认开）。
    pub fn legend(mut self, on: bool) -> Self {
        self.legend = on;
        self
    }

    /// 散点半径。
    pub fn dot_radius(mut self, radius: Pixels) -> Self {
        self.dot_radius = radius;
        self
    }
}

struct ResolvedSeries {
    name: SharedString,
    color: Hsla,
    points: Vec<ScatterPoint>,
}

impl gpui::RenderOnce for ScatterChart {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let x_max = self
            .x_max
            .unwrap_or_else(|| {
                self.series
                    .iter()
                    .flat_map(|s| s.points.iter())
                    .map(|p| p.x)
                    .fold(0., f32::max)
                    * 1.05
            })
            .max(1e-6);
        let y_max = self
            .y_max
            .unwrap_or_else(|| {
                self.series
                    .iter()
                    .flat_map(|s| s.points.iter())
                    .map(|p| p.y)
                    .fold(0., f32::max)
                    * 1.05
            })
            .max(1e-6);

        let resolved: Vec<ResolvedSeries> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| ResolvedSeries {
                name: s.name.clone(),
                color: s.color.unwrap_or_else(|| theme::chart_color(i)),
                points: s.points.clone(),
            })
            .collect();

        let chart = ScatterElement {
            id: self.id.clone(),
            series: resolved,
            height: self.height,
            x_max,
            y_max,
            dot_radius: self.dot_radius,
        };

        let mut root = div().w_full().flex().flex_col().gap_2().child(chart);

        if self.legend && !self.series.is_empty() {
            let legend = div().flex().flex_row().flex_wrap().gap_4().children(
                self.series.iter().enumerate().map(|(i, s)| {
                    let color = s.color.unwrap_or_else(|| theme::chart_color(i));
                    let item_id = format!("{:?}-legend-{i}", self.id);
                    a11y::Semantics::new(Role::ListItem, s.name.clone())
                        .id(item_id.clone())
                        .position(i + 1, self.series.len())
                        .apply(
                            div()
                                .id(ElementId::Name(item_id.into()))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(div().w(px(9.)).h(px(9.)).rounded_full().bg(color))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(s.name.clone()),
                                ),
                        )
                }),
            );
            let legend = a11y::Semantics::new(Role::List, "散点图图例")
                .id(format!("{:?}-legend", self.id))
                .apply(legend.id(ElementId::Name(format!("{:?}-legend", self.id).into())));
            root = root.child(legend);
        }

        root
    }
}

/// 散点图内部绘制元素。
struct ScatterElement {
    id: ElementId,
    series: Vec<ResolvedSeries>,
    height: Pixels,
    x_max: f32,
    y_max: f32,
    dot_radius: Pixels,
}

impl ScatterElement {
    fn semantics(&self) -> a11y::Semantics {
        let point_count = self
            .series
            .iter()
            .map(|series| series.points.len())
            .sum::<usize>();
        let summary = self
            .series
            .iter()
            .take(8)
            .map(|series| {
                let points = series
                    .points
                    .iter()
                    .take(5)
                    .map(|point| format!("({}, {})", format_value(point.x), format_value(point.y)))
                    .collect::<Vec<_>>()
                    .join("、");
                format!("{} {} 点：{points}", series.name, series.points.len())
            })
            .collect::<Vec<_>>()
            .join("；");
        let summary = if self.series.len() > 8 {
            format!("{summary}；共 {} 系列", self.series.len())
        } else {
            summary
        };
        a11y::Semantics::new(
            Role::Figure,
            format!(
                "散点图：{} 系列、{point_count} 点，X 上限 {}，Y 上限 {}；{summary}",
                self.series.len(),
                format_value(self.x_max),
                format_value(self.y_max)
            ),
        )
        .id(format!("{:?}", self.id))
    }
}

impl Element for ScatterElement {
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
        let left_pad = px(30.);
        let bottom_pad = px(18.);
        let pad = px(6.);
        let chart_left = bounds.left() + left_pad;
        let chart_right = bounds.right() - pad;
        let chart_top = bounds.top() + pad;
        let chart_bottom = bounds.bottom() - bottom_pad;
        let chart_w = chart_right - chart_left;
        let chart_h = chart_bottom - chart_top;
        if chart_w <= px(0.) || chart_h <= px(0.) {
            return;
        }

        // 网格（横向 + 纵向各 4 条）
        for i in 0..=4 {
            let t = i as f32 / 4.;
            let y = chart_bottom - chart_h * t;
            let mut h = gpui::PathBuilder::stroke(px(1.));
            h.move_to(point(chart_left, y));
            h.line_to(point(chart_right, y));
            if let Ok(path) = h.build() {
                window.paint_path(path, theme::border());
            }
            let x = chart_left + chart_w * t;
            let mut v = gpui::PathBuilder::stroke(px(1.));
            v.move_to(point(x, chart_top));
            v.line_to(point(x, chart_bottom));
            if let Ok(path) = v.build() {
                window.paint_path(path, theme::border());
            }
        }

        // 坐标轴标签
        let fs = px(10.);
        paint_text(
            window,
            cx,
            format_value(self.y_max),
            point(bounds.left(), chart_top - px(4.)),
            fs,
            theme::text_muted(),
        );
        paint_text(
            window,
            cx,
            "0",
            point(bounds.left(), chart_bottom - px(4.)),
            fs,
            theme::text_muted(),
        );
        paint_text(
            window,
            cx,
            format_value(self.x_max),
            point(chart_right - px(24.), chart_bottom + px(4.)),
            fs,
            theme::text_muted(),
        );

        // 散点
        for s in &self.series {
            for p in &s.points {
                let x = chart_left + chart_w * (p.x / self.x_max).clamp(0., 1.);
                let y = chart_bottom - chart_h * (p.y / self.y_max).clamp(0., 1.);
                if let Ok(path) = circle(point(x, y), self.dot_radius).build() {
                    window.paint_path(path, s.color);
                }
            }
        }
    }
}
