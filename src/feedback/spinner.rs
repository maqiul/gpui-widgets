//! 加载指示器：antd Spin 的四点旋转几何（antRotate / antSpinMove 帧函数见 metric::spin_frame）。

use std::time::Duration;

use gpui::{
    Animation, AnimationExt, App, Bounds, Div, Element, ElementId, GlobalElementId, Hsla,
    InspectorElementId, IntoElement, LayoutId, Pixels, Role, SharedString, Stateful, Style, Window,
    div, fill, point, prelude::*, px, size,
};

use crate::a11y::Semantics;
use crate::theme;
use crate::theme::metric::{spin_dot_centers, spin_frame, spin_metric, spin_style};

/// 旋转加载指示器（antd Spin 四点样式）。
///
/// ```ignore
/// Spinner::new()               // 默认 20px
/// Spinner::new().size(px(32.))
/// ```
pub struct Spinner {
    size: Pixels,
    delta: f32,
    base: Option<Hsla>,
    tip: Option<SharedString>,
    animated: bool,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: px(20.),
            delta: 0.,
            base: None,
            tip: None,
            animated: true,
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// 圆点基色（默认主题强调色；深色按钮上可传 white()）
    pub fn color(mut self, color: Hsla) -> Self {
        self.base = Some(color);
        self
    }

    /// 加载文案（显示在圆点环下方）
    pub fn tip(mut self, tip: impl Into<SharedString>) -> Self {
        self.tip = Some(tip.into());
        self
    }

    /// 关闭动画，停在 antd 首帧（rotate 45°、四点 opacity 0.3），供截图比对
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoElement for Spinner {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let m = theme::with(spin_metric);
        let style = theme::with(spin_style);
        let base = self.base.unwrap_or(style.dot);
        // 注意：动画必须包在内部类型上。若对 Spinner 自身调 with_animation，
        // 每帧 animator 返回的 Spinner 会再次 into_element 包装，导致无限递归
        let dots = SpinnerDots {
            size: self.size,
            delta: self.delta,
            base,
        };
        let dots = if self.animated {
            // 旋转周期 1.2s、亮度周期 2s，公倍数 6s 覆盖两者
            dots.with_animation(
                ElementId::Name("spinner".into()),
                Animation::new(Duration::from_millis(6000)).repeat(),
                |mut this, delta| {
                    this.delta = delta;
                    this
                },
            )
            .into_any_element()
        } else {
            dots.into_any_element()
        };
        let tip_id = self.tip.as_ref().map(|tip| format!("spinner-{tip}"));
        let mut root = div()
            .id(ElementId::Name(
                tip_id.clone().unwrap_or_else(|| "spinner".into()).into(),
            ))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(m.gap))
            .child(dots);
        if let Some(tip) = self.tip {
            let id = tip_id.unwrap_or_else(|| "spinner".into());
            root = root.child(
                div()
                    .text_size(px(m.desc_size))
                    .line_height(px(m.desc_line_height))
                    .text_color(base)
                    .child(tip.clone()),
            );
            return Semantics::new(Role::Status, tip).id(id).apply(root);
        }
        root
    }
}

/// 实际绘制的四点环（被 AnimationElement 包裹，避免递归包装）
pub struct SpinnerDots {
    size: Pixels,
    delta: f32,
    base: Hsla,
}

impl Element for SpinnerDots {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
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
        _cx: &mut App,
    ) {
        let holder = f32::from(self.size);
        let margin_xxs = theme::with(|t| t.margin_xxs);
        let m = theme::with(spin_metric);
        let item = (holder - margin_xxs / 2.) / 2.;
        let (angle, opacities) = spin_frame(self.delta * 6.);
        let radius = item * m.dot_scale / 2.;
        for ((cx, cy), opacity) in spin_dot_centers(holder, item, angle)
            .into_iter()
            .zip(opacities)
        {
            let center = point(bounds.origin.x + px(cx), bounds.origin.y + px(cy));
            let mut dot = fill(
                Bounds::new(
                    point(center.x - px(radius), center.y - px(radius)),
                    size(px(radius * 2.), px(radius * 2.)),
                ),
                self.base.opacity(opacity),
            );
            dot.corner_radii = px(radius).into();
            window.paint_quad(dot);
        }
    }
}
