//! 仪表盘：GPU 路径绘制的 270° 圆弧 + 指针 + 阈值色区 + 中心数值。
//!
//! ```ignore
//! GaugeChart::new(72.).max(100.).title("CPU 使用率")
//!     .zones(vec![(0.6, theme::success()), (0.85, theme::warning()), (1.0, theme::danger())])
//! ```

use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, Hsla, InspectorElementId, IntoElement,
    LayoutId, PathBuilder, Pixels, Role, SharedString, Style, Window, accesskit, point, px,
};

use crate::a11y;
use crate::chart::chart_id;
use crate::chart_util::{arc, circle, format_value, paint_text_centered, polar};
use crate::theme;

/// 仪表盘。默认量程 0~100，270° 圆弧（缺口朝下）。
pub struct GaugeChart {
    id: ElementId,
    value: f32,
    max: f32,
    size: Pixels,
    thickness: Pixels,
    color: Option<Hsla>,
    label: Option<SharedString>,
    title: Option<SharedString>,
    zones: Vec<(f32, Hsla)>,
}

impl GaugeChart {
    pub fn new(value: f32) -> Self {
        let id = chart_id("gauge", (value, 100.));
        Self {
            id,
            value,
            max: 100.,
            size: px(180.),
            thickness: px(14.),
            color: None,
            label: None,
            title: None,
            zones: Vec::new(),
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    fn refresh_id(&mut self) {
        self.id = chart_id(
            "gauge",
            (
                self.value,
                self.max,
                self.label.as_ref().map(|label| label.as_ref()),
                self.title.as_ref().map(|title| title.as_ref()),
            ),
        );
    }

    /// 量程上限（默认 100）。
    pub fn max(mut self, max: f32) -> Self {
        self.max = max.max(1e-6);
        self.refresh_id();
        self
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// 圆弧粗细。
    pub fn thickness(mut self, thickness: Pixels) -> Self {
        self.thickness = thickness;
        self
    }

    /// 进度弧颜色（默认按阈值色区，无色区时用主题强调色）。
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// 中心数值文字（默认显示当前值）。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self.refresh_id();
        self
    }

    /// 数值下方的说明文字（如 "CPU 使用率"）。
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self.refresh_id();
        self
    }

    /// 阈值色区：`(比例上限, 颜色)`，按比例升序，命中第一个上限 >= 当前比例的色区。
    pub fn zones(mut self, zones: Vec<(f32, Hsla)>) -> Self {
        self.zones = zones;
        self
    }

    fn resolve_color(&self, ratio: f32) -> Hsla {
        if let Some(c) = self.color {
            return c;
        }
        for (bound, color) in &self.zones {
            if ratio <= *bound {
                return *color;
            }
        }
        theme::accent()
    }
}

impl IntoElement for GaugeChart {
    type Element = GaugeElement;

    fn into_element(self) -> Self::Element {
        let ratio = (self.value / self.max).clamp(0., 1.);
        let color = self.resolve_color(ratio);
        GaugeElement {
            id: self.id,
            value: self.value,
            max: self.max,
            ratio,
            size: self.size,
            thickness: self.thickness,
            color,
            label: self.label,
            title: self.title,
        }
    }
}

/// 仪表盘内部绘制元素。
pub struct GaugeElement {
    id: ElementId,
    value: f32,
    max: f32,
    ratio: f32,
    size: Pixels,
    thickness: Pixels,
    color: Hsla,
    label: Option<SharedString>,
    title: Option<SharedString>,
}

impl GaugeElement {
    fn semantics(&self) -> a11y::Semantics {
        let title = self.title.as_deref().unwrap_or("仪表盘");
        let display = self
            .label
            .clone()
            .unwrap_or_else(|| format_value(self.value).into());
        a11y::Semantics::new(
            Role::Figure,
            format!(
                "{title}：当前 {display}，数值 {}，量程 0 至 {}，占比 {:.0}%",
                format_value(self.value),
                format_value(self.max),
                self.ratio * 100.
            ),
        )
        .id(format!("{:?}", self.id))
        .numeric(self.value as f64)
        .numeric_range(0., self.max as f64)
    }
}

impl Element for GaugeElement {
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
        let radius = (self.size - self.thickness) / 2. - px(2.);
        if radius <= px(0.) {
            return;
        }

        // 270° 圆弧：起点 135°（左下），顺时针扫到 45°（右下），缺口朝下
        let start = std::f32::consts::PI * 0.75;
        let sweep = std::f32::consts::PI * 1.5;
        let end = start + sweep;
        let value_angle = start + sweep * self.ratio;

        // 轨道弧
        if let Ok(path) = arc(center, radius, start, end, self.thickness).build() {
            window.paint_path(path, theme::track());
        }
        // 进度弧
        if self.ratio > 0. {
            if let Ok(path) = arc(center, radius, start, value_angle, self.thickness).build() {
                window.paint_path(path, self.color);
            }
        }

        // 刻度（每 10% 一小刻度）
        for i in 0..=10 {
            let a = start + sweep * (i as f32 / 10.);
            let outer = polar(center, radius - self.thickness / 2. - px(3.), a);
            let inner = polar(center, radius - self.thickness / 2. - px(8.), a);
            let mut tick = PathBuilder::stroke(px(1.));
            tick.move_to(outer);
            tick.line_to(inner);
            if let Ok(path) = tick.build() {
                window.paint_path(path, theme::border());
            }
        }

        // 指针
        let tip = polar(center, radius * 0.68, value_angle);
        let mut needle = PathBuilder::stroke(px(3.));
        needle.move_to(center);
        needle.line_to(tip);
        if let Ok(path) = needle.build() {
            window.paint_path(path, theme::text());
        }
        // 轴心
        if let Ok(path) = circle(center, px(5.)).build() {
            window.paint_path(path, theme::text());
        }

        // 中心数值（落在下方缺口区）
        let value_text = self
            .label
            .clone()
            .unwrap_or_else(|| format_value(self.value).into());
        let value_size = px((f32::from(self.size) * 0.18).max(15.));
        let value_center = point(center.x, center.y + radius * 0.5);
        paint_text_centered(
            window,
            cx,
            value_text,
            value_center,
            value_size,
            theme::text(),
        );

        if let Some(title) = &self.title {
            paint_text_centered(
                window,
                cx,
                title.clone(),
                point(center.x, value_center.y + value_size * 0.9),
                px(11.),
                theme::text_muted(),
            );
        }

        // 量程两端标签
        let label_r = radius + self.thickness / 2. + px(2.);
        let min_pos = polar(center, label_r, start);
        let max_pos = polar(center, label_r, end);
        paint_text_centered(
            window,
            cx,
            "0",
            point(min_pos.x, min_pos.y + px(6.)),
            px(10.),
            theme::text_muted(),
        );
        paint_text_centered(
            window,
            cx,
            format_value(self.max),
            point(max_pos.x, max_pos.y + px(6.)),
            px(10.),
            theme::text_muted(),
        );
    }
}
