//! 雷达图：GPU 路径绘制的多轴网格 + 轴线 + 多系列半透明多边形。
//!
//! ```ignore
//! RadarChart::new(
//!     vec!["攻击".into(), "防御".into(), "速度".into(), "技巧".into(), "耐力".into()],
//!     vec![
//!         RadarSeries::new("选手 A", vec![80., 65., 90., 70., 60.]),
//!         RadarSeries::new("选手 B", vec![60., 85., 55., 80., 75.]),
//!     ],
//! ).max(100.)
//! ```

use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, Hsla, InspectorElementId, IntoElement,
    LayoutId, PathBuilder, Pixels, Role, SharedString, Style, Window, accesskit, div, point,
    prelude::*, px,
};

use crate::a11y;
use crate::chart::chart_id;
use crate::chart_util::{circle, format_value, paint_text_centered, polar};
use crate::theme;

/// 雷达图系列。
pub struct RadarSeries {
    name: SharedString,
    values: Vec<f32>,
    color: Option<Hsla>,
}

impl RadarSeries {
    pub fn new(name: impl Into<SharedString>, values: Vec<f32>) -> Self {
        Self {
            name: name.into(),
            values,
            color: None,
        }
    }

    /// 自定义系列颜色（默认取主题分类色板）。
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// 雷达图。
#[derive(IntoElement)]
pub struct RadarChart {
    id: ElementId,
    axes: Vec<SharedString>,
    series: Vec<RadarSeries>,
    max: Option<f32>,
    size: Pixels,
    levels: usize,
    legend: bool,
}

impl RadarChart {
    pub fn new(axes: Vec<SharedString>, series: Vec<RadarSeries>) -> Self {
        let id = chart_id(
            "radar",
            (
                axes.iter().map(|axis| axis.as_ref()).collect::<Vec<_>>(),
                series
                    .iter()
                    .map(|series| (series.name.as_ref(), &series.values))
                    .collect::<Vec<_>>(),
            ),
        );
        Self {
            id,
            axes,
            series,
            max: None,
            size: px(240.),
            levels: 4,
            legend: true,
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 数值上限（默认取数据最大值）。
    pub fn max(mut self, max: f32) -> Self {
        self.max = Some(max);
        self
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// 网格环数（默认 4）。
    pub fn levels(mut self, levels: usize) -> Self {
        self.levels = levels.max(1);
        self
    }

    /// 是否显示底部图例（默认开）。
    pub fn legend(mut self, on: bool) -> Self {
        self.legend = on;
        self
    }
}

struct ResolvedSeries {
    name: SharedString,
    color: Hsla,
    values: Vec<f32>,
}

impl gpui::RenderOnce for RadarChart {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let max = self
            .max
            .unwrap_or_else(|| {
                self.series
                    .iter()
                    .flat_map(|s| s.values.iter())
                    .copied()
                    .fold(0., f32::max)
            })
            .max(1e-6);

        let resolved: Vec<ResolvedSeries> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| ResolvedSeries {
                name: s.name.clone(),
                color: s.color.unwrap_or_else(|| theme::chart_color(i)),
                values: s.values.clone(),
            })
            .collect();

        let chart = RadarElement {
            id: self.id.clone(),
            axes: self.axes.clone(),
            series: resolved,
            max,
            size: self.size,
            levels: self.levels,
        };

        let mut root = div().flex().flex_col().items_center().gap_2().child(chart);

        if self.legend && !self.series.is_empty() {
            let legend = div()
                .flex()
                .flex_row()
                .flex_wrap()
                .justify_center()
                .gap_4()
                .children(self.series.iter().enumerate().map(|(i, s)| {
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
                                .child(div().w(px(9.)).h(px(9.)).rounded(px(2.)).bg(color))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(s.name.clone()),
                                ),
                        )
                }));
            let legend = a11y::Semantics::new(Role::List, "雷达图图例")
                .id(format!("{:?}-legend", self.id))
                .apply(legend.id(ElementId::Name(format!("{:?}-legend", self.id).into())));
            root = root.child(legend);
        }

        root
    }
}

/// 雷达图内部绘制元素。
struct RadarElement {
    id: ElementId,
    axes: Vec<SharedString>,
    series: Vec<ResolvedSeries>,
    max: f32,
    size: Pixels,
    levels: usize,
}

impl RadarElement {
    fn semantics(&self) -> a11y::Semantics {
        let axes = self
            .axes
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("、");
        let series = self
            .series
            .iter()
            .take(8)
            .map(|series| {
                let values = series
                    .values
                    .iter()
                    .take(8)
                    .map(|value| format_value(*value))
                    .collect::<Vec<_>>()
                    .join("、");
                format!("{}：{values}", series.name)
            })
            .collect::<Vec<_>>()
            .join("；");
        a11y::Semantics::new(
            Role::Figure,
            format!(
                "雷达图：{} 轴（{axes}），{} 系列，量程上限 {}；{series}",
                self.axes.len(),
                self.series.len(),
                format_value(self.max)
            ),
        )
        .id(format!("{:?}", self.id))
    }
}

impl Element for RadarElement {
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
        style.size.width = self.size.into();
        style.size.height = self.size.into();
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
        let n = self.axes.len();
        if n < 3 {
            return;
        }
        let center = point(
            bounds.left() + self.size / 2.,
            bounds.top() + self.size / 2.,
        );
        let radius = self.size / 2. - px(28.);
        if radius <= px(0.) {
            return;
        }

        let angle =
            |i: usize| -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * i as f32 / n as f32;

        // 网格环
        for level in 1..=self.levels {
            let r = radius * (level as f32 / self.levels as f32);
            let mut ring = PathBuilder::stroke(px(1.));
            for i in 0..=n {
                let p = polar(center, r, angle(i % n));
                if i == 0 {
                    ring.move_to(p);
                } else {
                    ring.line_to(p);
                }
            }
            if let Ok(path) = ring.build() {
                window.paint_path(path, theme::border());
            }
        }

        // 轴线 + 轴标签
        for i in 0..n {
            let vertex = polar(center, radius, angle(i));
            let mut spoke = PathBuilder::stroke(px(1.));
            spoke.move_to(center);
            spoke.line_to(vertex);
            if let Ok(path) = spoke.build() {
                window.paint_path(path, theme::border());
            }
            let label_pos = polar(center, radius + px(16.), angle(i));
            paint_text_centered(
                window,
                cx,
                self.axes[i].clone(),
                label_pos,
                px(11.),
                theme::text_muted(),
            );
        }

        // 系列多边形
        for s in &self.series {
            let points: Vec<_> = (0..n)
                .map(|i| {
                    let v = s.values.get(i).copied().unwrap_or(0.);
                    let r = radius * (v / self.max).clamp(0., 1.);
                    polar(center, r, angle(i))
                })
                .collect();

            // 半透明填充
            let mut fill = PathBuilder::fill();
            for (i, p) in points.iter().enumerate() {
                if i == 0 {
                    fill.move_to(*p);
                } else {
                    fill.line_to(*p);
                }
            }
            fill.close();
            if let Ok(path) = fill.build() {
                let mut soft = s.color;
                soft.a = 0.18;
                window.paint_path(path, soft);
            }

            // 描边
            let mut outline = PathBuilder::stroke(px(2.));
            for (i, p) in points.iter().enumerate() {
                if i == 0 {
                    outline.move_to(*p);
                } else {
                    outline.line_to(*p);
                }
            }
            outline.close();
            if let Ok(path) = outline.build() {
                window.paint_path(path, s.color);
            }

            // 顶点圆点
            for p in &points {
                if let Ok(path) = circle(*p, px(3.)).build() {
                    window.paint_path(path, s.color);
                }
            }
        }
    }
}
