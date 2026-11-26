//! 环形进度：GPU 路径绘制的圆环进度指示器。

use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, Hsla, InspectorElementId, IntoElement,
    LayoutId, PathBuilder, Pixels, Role, SharedString, Style, Window, point, px,
};

use crate::a11y::Semantics;
use crate::theme;

/// 环形进度。
///
/// ```ignore
/// RingProgress::new(0.72).size(px(96.)).label("72%")
/// ```
pub struct RingProgress {
    value: f32,
    id: Option<SharedString>,
    origin: SharedString,
    size: Pixels,
    thickness: Pixels,
    color: Hsla,
    track_color: Hsla,
    label: Option<SharedString>,
}

impl RingProgress {
    /// value 取值 0.0 ~ 1.0（越界自动夹取）
    #[track_caller]
    pub fn new(value: f32) -> Self {
        let location = core::panic::Location::caller();
        Self {
            value: value.clamp(0., 1.),
            id: None,
            origin: format!(
                "ring-progress-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            size: px(80.),
            thickness: px(8.),
            color: theme::accent(),
            track_color: theme::track(),
            label: None,
        }
    }

    /// 无障碍稳定 id；同页多个圆环应显式传入不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// 圆环粗细
    pub fn thickness(mut self, thickness: Pixels) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = color.into();
        self
    }

    pub fn track_color(mut self, color: impl Into<Hsla>) -> Self {
        self.track_color = color.into();
        self
    }

    /// 圆环中心的文字（如 "72%"）
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl IntoElement for RingProgress {
    type Element = RingElement;

    fn into_element(self) -> Self::Element {
        let key = self.id.as_deref().unwrap_or(&self.origin);
        let semantic_key = self
            .label
            .as_deref()
            .map(|label| format!("{key}-{label}"))
            .unwrap_or_else(|| key.to_string());
        let semantics = Semantics::new(
            Role::ProgressIndicator,
            self.label.clone().unwrap_or_else(|| "环形进度".into()),
        )
        .id(format!("ring-progress-{semantic_key}"))
        .value(format!("{}%", (self.value * 100.0).round() as u32))
        .numeric(f64::from(self.value))
        .numeric_range(0.0, 1.0);
        RingElement {
            value: self.value,
            size: self.size,
            thickness: self.thickness,
            color: self.color,
            track_color: self.track_color,
            label: self.label,
            semantics,
        }
    }
}

/// 环形进度的内部绘制元素
pub struct RingElement {
    value: f32,
    size: Pixels,
    thickness: Pixels,
    color: Hsla,
    track_color: Hsla,
    label: Option<SharedString>,
    semantics: Semantics,
}

impl Element for RingElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.semantics.element_id())
    }

    fn a11y_role(&self) -> Option<Role> {
        Some(self.semantics.role())
    }

    fn write_a11y_info(&self, node: &mut gpui::accesskit::Node) {
        self.semantics.write(node);
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
        let radius = (self.size - self.thickness) / 2.;
        if radius <= px(0.) {
            return;
        }

        // 轨道：完整圆（多段折线逼近）
        let segments = 64;
        let mut track = PathBuilder::stroke(self.thickness);
        for i in 0..=segments {
            let angle = std::f32::consts::TAU * i as f32 / segments as f32;
            let p = point(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            if i == 0 {
                track.move_to(p);
            } else {
                track.line_to(p);
            }
        }
        if let Ok(path) = track.build() {
            window.paint_path(path, self.track_color);
        }

        // 进度弧：从正上方顺时针
        if self.value > 0. {
            let sweep = std::f32::consts::TAU * self.value;
            let arc_segments = ((segments as f32 * self.value).ceil() as usize).max(2);
            let mut arc = PathBuilder::stroke(self.thickness);
            for i in 0..=arc_segments {
                let angle = -std::f32::consts::FRAC_PI_2 + sweep * i as f32 / arc_segments as f32;
                let p = point(
                    center.x + radius * angle.cos(),
                    center.y + radius * angle.sin(),
                );
                if i == 0 {
                    arc.move_to(p);
                } else {
                    arc.line_to(p);
                }
            }
            if let Ok(path) = arc.build() {
                window.paint_path(path, self.color);
            }
        }

        // 中心文字
        if let Some(label) = &self.label {
            let style = window.text_style();
            let font_size = (f32::from(self.size) * 0.22).max(12.);
            let run = gpui::TextRun {
                len: label.len(),
                font: style.font(),
                color: theme::text(),
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let line = window
                .text_system()
                .shape_line(label.clone(), px(font_size), &[run], None);
            let origin = point(center.x - line.width / 2., center.y - px(font_size * 0.7));
            let _ = line.paint(
                origin,
                px(font_size * 1.4),
                gpui::TextAlign::default(),
                None,
                window,
                cx,
            );
        }
    }
}
