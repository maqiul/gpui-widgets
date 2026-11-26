//! 饼图 / 环形图：GPU 路径绘制扇区 + 百分比标签 + 图例。
//!
//! ```ignore
//! PieChart::new(vec![
//!     PieDatum::new("直接访问", 335.),
//!     PieDatum::new("邮件营销", 234.),
//! ]).donut(true)
//! ```

use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, Hsla, InspectorElementId, IntoElement,
    LayoutId, Pixels, Role, SharedString, Style, Window, accesskit, div, point, prelude::*, px,
    white,
};

use crate::a11y;
use crate::chart::chart_id;
use crate::chart_util::{format_value, paint_text_centered, polar, sector};
use crate::theme;

/// 饼图数据项。
pub struct PieDatum {
    label: SharedString,
    value: f32,
    color: Option<Hsla>,
}

impl PieDatum {
    pub fn new(label: impl Into<SharedString>, value: f32) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    /// 自定义扇区颜色（默认取主题分类色板）。
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// 饼图 / 环形图。`donut(true)` 或 `inner_ratio(0.6)` 切换为环图。
#[derive(IntoElement)]
pub struct PieChart {
    id: ElementId,
    data: Vec<PieDatum>,
    size: Pixels,
    inner_ratio: f32,
    legend: bool,
    labels: bool,
}

impl PieChart {
    pub fn new(data: Vec<PieDatum>) -> Self {
        let id = chart_id(
            "pie",
            data.iter()
                .map(|datum| (datum.label.as_ref(), datum.value))
                .collect::<Vec<_>>(),
        );
        Self {
            id,
            data,
            size: px(180.),
            inner_ratio: 0.,
            legend: true,
            labels: true,
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 饼图直径。
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// 是否为环形图（内半径默认 0.6）。
    pub fn donut(mut self, on: bool) -> Self {
        self.inner_ratio = if on { 0.6 } else { 0. };
        self
    }

    /// 内半径比例（0~0.85），0 为实心饼图。
    pub fn inner_ratio(mut self, ratio: f32) -> Self {
        self.inner_ratio = ratio.clamp(0., 0.85);
        self
    }

    /// 是否显示右侧图例（默认开）。
    pub fn legend(mut self, on: bool) -> Self {
        self.legend = on;
        self
    }

    /// 是否显示扇区百分比标签（默认开）。
    pub fn labels(mut self, on: bool) -> Self {
        self.labels = on;
        self
    }
}

#[derive(Clone)]
struct PieSlice {
    label: SharedString,
    value: f32,
    color: Hsla,
    percent: f32,
}

impl gpui::RenderOnce for PieChart {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let total = self
            .data
            .iter()
            .map(|d| d.value.max(0.))
            .sum::<f32>()
            .max(1e-6);
        let slices: Vec<PieSlice> = self
            .data
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let value = d.value.max(0.);
                PieSlice {
                    label: d.label.clone(),
                    value,
                    color: d.color.unwrap_or_else(|| theme::chart_color(i)),
                    percent: value / total,
                }
            })
            .collect();

        let pie = PieElement {
            id: self.id.clone(),
            slices: slices.clone(),
            size: self.size,
            inner_ratio: self.inner_ratio,
            labels: self.labels,
            total,
        };

        let mut root = div().flex().flex_row().items_center().gap_5().child(pie);

        if self.legend {
            let mut legend = div().flex().flex_col().gap_2();
            for (index, s) in slices.iter().enumerate() {
                let item_id = format!("{:?}-legend-{index}", self.id);
                legend =
                    legend.child(
                        a11y::Semantics::new(
                            Role::ListItem,
                            format!(
                                "{}：{}，{:.0}%",
                                s.label,
                                format_value(s.value),
                                s.percent * 100.
                            ),
                        )
                        .id(item_id.clone())
                        .position(index + 1, slices.len())
                        .apply(
                            div()
                                .id(ElementId::Name(item_id.into()))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(div().w(px(10.)).h(px(10.)).rounded(px(2.)).bg(s.color))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme::text())
                                        .child(s.label.clone()),
                                )
                                .child(div().text_xs().text_color(theme::text_muted()).child(
                                    format!("{} · {:.0}%", format_value(s.value), s.percent * 100.),
                                )),
                        ),
                    );
            }
            let legend = a11y::Semantics::new(Role::List, "饼图图例")
                .id(format!("{:?}-legend", self.id))
                .apply(legend.id(ElementId::Name(format!("{:?}-legend", self.id).into())));
            root = root.child(legend);
        }

        root
    }
}

/// 饼图内部绘制元素。
struct PieElement {
    id: ElementId,
    slices: Vec<PieSlice>,
    size: Pixels,
    inner_ratio: f32,
    labels: bool,
    total: f32,
}

impl PieElement {
    fn semantics(&self) -> a11y::Semantics {
        let name = if self.inner_ratio > 0. {
            "环形图"
        } else {
            "饼图"
        };
        let summary = self
            .slices
            .iter()
            .take(8)
            .map(|slice| {
                format!(
                    "{} {}（{:.0}%）",
                    slice.label,
                    format_value(slice.value),
                    slice.percent * 100.
                )
            })
            .collect::<Vec<_>>()
            .join("；");
        let summary = if self.slices.len() > 8 {
            format!("{summary}；共 {} 项", self.slices.len())
        } else {
            summary
        };
        a11y::Semantics::new(
            Role::Figure,
            format!(
                "{name}：{} 项，总计 {}；{summary}",
                self.slices.len(),
                format_value(self.total)
            ),
        )
        .id(format!("{:?}", self.id))
        .numeric(self.total as f64)
    }
}

impl Element for PieElement {
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
        let center = point(
            bounds.left() + self.size / 2.,
            bounds.top() + self.size / 2.,
        );
        let r_outer = (self.size - px(4.)) / 2.;
        if r_outer <= px(0.) {
            return;
        }
        let r_inner = r_outer * self.inner_ratio;

        let mut angle = -std::f32::consts::FRAC_PI_2;
        for s in &self.slices {
            if s.percent <= 0. {
                continue;
            }
            let sweep = std::f32::consts::TAU * s.percent;
            let a1 = angle + sweep;
            if let Ok(path) = sector(center, r_outer, r_inner, angle, a1).build() {
                window.paint_path(path, s.color);
            }
            if self.labels && s.percent >= 0.05 {
                let mid = angle + sweep / 2.;
                let label_r = if r_inner > px(0.) {
                    (r_outer + r_inner) / 2.
                } else {
                    r_outer * 0.62
                };
                let pos = polar(center, label_r, mid);
                paint_text_centered(
                    window,
                    cx,
                    format!("{:.0}%", s.percent * 100.),
                    pos,
                    px(11.),
                    white(),
                );
            }
            angle = a1;
        }

        // 环图中心显示总计
        if self.inner_ratio > 0. && self.labels {
            let big = px((f32::from(self.size) * 0.16).max(13.));
            paint_text_centered(
                window,
                cx,
                format_value(self.total),
                point(center.x, center.y - px(4.)),
                big,
                theme::text(),
            );
            paint_text_centered(
                window,
                cx,
                "总计",
                point(center.x, center.y + big * 0.7),
                px(10.),
                theme::text_muted(),
            );
        }
    }
}
