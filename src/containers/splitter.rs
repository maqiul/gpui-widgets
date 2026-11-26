//! 可拖拽分割面板：左右 / 上下两栏，拖动分隔条调整比例（受控）。

use std::{cell::Cell, rc::Rc};

use gpui::{
    AccessibleAction, AnyElement, App, Bounds, CursorStyle, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, MouseButton, MouseMoveEvent, Orientation, Pixels,
    Position, RenderOnce, Role, SharedString, Style, Window, div, prelude::*, px, relative,
};

use crate::a11y;
use crate::theme;

const MIN_RATIO: f32 = 0.05;
const MAX_RATIO: f32 = 0.95;
const RATIO_STEP: f32 = 0.05;

fn keyboard_ratio(current: f32, key: &str, vertical: bool) -> Option<f32> {
    let key = key.strip_prefix("arrow").unwrap_or(key);
    let next = match key {
        "home" => MIN_RATIO,
        "end" => MAX_RATIO,
        "right" if !vertical => current + RATIO_STEP,
        "left" if !vertical => current - RATIO_STEP,
        "up" if vertical => current + RATIO_STEP,
        "down" if vertical => current - RATIO_STEP,
        _ => return None,
    };
    Some(next.clamp(MIN_RATIO, MAX_RATIO))
}

fn finite_f32(value: f64) -> Option<f32> {
    let value = value as f32;
    value.is_finite().then_some(value)
}

fn pointer_ratio(position: f32, origin: f32, extent: f32) -> Option<f32> {
    if position.is_finite() && origin.is_finite() && extent.is_finite() && extent > 0. {
        Some(((position - origin) / extent).clamp(MIN_RATIO, MAX_RATIO))
    } else {
        None
    }
}

/// 拖拽状态（全局共享，同一时刻只有一个分割条被拖动）
#[derive(Default)]
struct SplitterDrag(Option<SharedString>);

impl gpui::Global for SplitterDrag {}

/// 可拖拽分割面板（受控）。
///
/// ```ignore
/// Splitter::new("editor", left_pane, right_pane)
///     .ratio(self.split_ratio)
///     .on_change(cx.listener(|this, ratio: &f32, _, cx| {
///         this.split_ratio = *ratio; cx.notify();
///     }))
/// ```
#[derive(IntoElement)]
pub struct Splitter {
    id: SharedString,
    left: AnyElement,
    right: AnyElement,
    ratio: f32,
    vertical: bool,
    size: Pixels,
    on_change: Option<Rc<dyn Fn(&f32, &mut Window, &mut App)>>,
}

impl Splitter {
    pub fn new(
        id: impl Into<SharedString>,
        left: impl IntoElement,
        right: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            left: left.into_any_element(),
            right: right.into_any_element(),
            ratio: 0.5,
            vertical: false,
            size: px(240.),
            on_change: None,
        }
    }

    /// 左栏 / 上栏占比（0.0 ~ 1.0）
    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.clamp(MIN_RATIO, MAX_RATIO);
        self
    }

    /// 上下分割（默认左右）
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }

    /// 容器尺寸（左右分割时为高度，上下分割时为宽度）
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for Splitter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let vertical = self.vertical;
        let ratio = self.ratio;
        let dragging = cx.default_global::<SplitterDrag>().0.as_ref() == Some(&id);
        let bounds: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
        let on_change = self.on_change.clone();
        let (m, idle, hover, knob) = theme::with(|t| {
            let m = crate::theme::metric::splitter_metric(t);
            (
                m,
                crate::theme::metric::splitter_bar_state(t, false, false, false),
                crate::theme::metric::splitter_bar_state(t, true, false, false),
                crate::theme::metric::splitter_knob_color(t),
            )
        });
        let active =
            theme::with(|t| crate::theme::metric::splitter_bar_state(t, false, true, false));
        let half = m.trigger_size / 2.0;
        let group = format!("{id}-bar-group");

        // antd 分割容器无描边无圆角，面板自带 1px 横向内边距
        let mut root = a11y::Semantics::new(Role::Region, "分割面板")
            .id(format!("{id}-region"))
            .apply(div().id(id.clone()))
            .relative()
            .flex()
            .w_full()
            .overflow_hidden();
        root = if vertical {
            root.flex_col().h_full().w(self.size)
        } else {
            root.h(self.size)
        };

        // 边界探测层（绝对定位铺满，不参与交互或无障碍树）
        root = root.child(SplitterProbe {
            bounds: bounds.clone(),
            id: ElementId::Name(format!("{id}-bounds-probe").into()),
        });

        // 两栏：用 flex_grow 按比例分配剩余空间
        let left_grow = self.ratio.max(MIN_RATIO);
        let right_grow = (1. - self.ratio).max(MIN_RATIO);
        let pane_pad = px(m.panel_padding_inline);
        root = root
            .child({
                let mut pane = div().overflow_hidden().px(pane_pad).child(self.left);
                // basis 归零：否则初始宽度跟随内容（First/Second 字宽不同），
                // 50/50 会被带偏
                pane.style().flex_basis = Some(relative(0.).into());
                pane.style().flex_grow = Some(left_grow);
                pane.style().flex_shrink = Some(0.);
                pane
            })
            .child({
                // 命中区 6px 但负外边距抵消占位（antd 条宽 0），可见条 2px 居中
                let mut divider = div()
                    .id(SharedString::from(format!("{id}-divider")))
                    .group(group.clone())
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .on_mouse_down(MouseButton::Left, {
                        let id = id.clone();
                        move |_, _, cx| {
                            cx.default_global::<SplitterDrag>().0 = Some(id.clone());
                        }
                    });
                let strip = div().absolute().bg(if dragging { active } else { idle });
                let strip = strip.group_hover(group, move |style| style.bg(hover));
                divider = if vertical {
                    divider
                        .h(px(m.trigger_size))
                        .w_full()
                        .mt(px(-half))
                        .mb(px(-half))
                        .cursor(CursorStyle::ResizeUpDown)
                        .child(
                            strip
                                .left_0()
                                .right_0()
                                .top(px(half - m.bar_size / 2.0))
                                .h(px(m.bar_size)),
                        )
                        .child(div().w(px(m.draggable_size)).h(px(m.bar_size)).bg(knob))
                } else {
                    divider
                        .w(px(m.trigger_size))
                        .h_full()
                        .ml(px(-half))
                        .mr(px(-half))
                        .cursor(CursorStyle::ResizeLeftRight)
                        .child(
                            strip
                                .top_0()
                                .bottom_0()
                                .left(px(half - m.bar_size / 2.0))
                                .w(px(m.bar_size)),
                        )
                        .child(div().w(px(m.bar_size)).h(px(m.draggable_size)).bg(knob))
                };
                divider = a11y::Semantics::new(Role::Slider, "调整面板分割比例")
                    .id(format!("{id}-slider"))
                    .numeric(f64::from(ratio))
                    .numeric_range(f64::from(MIN_RATIO), f64::from(MAX_RATIO))
                    .numeric_step(f64::from(RATIO_STEP))
                    .orientation(if vertical {
                        Orientation::Vertical
                    } else {
                        Orientation::Horizontal
                    })
                    .read_only(on_change.is_none())
                    .focusable(on_change.is_some())
                    .apply(divider);
                if let Some(on_change) = on_change.clone() {
                    divider = divider
                        .on_key_down({
                            let on_change = on_change.clone();
                            move |event, window, cx| {
                                if let Some(next) =
                                    keyboard_ratio(ratio, event.keystroke.key.as_str(), vertical)
                                {
                                    on_change(&next, window, cx);
                                    cx.stop_propagation();
                                }
                            }
                        })
                        .on_a11y_action(AccessibleAction::SetValue, {
                            let on_change = on_change.clone();
                            move |data, window, cx| {
                                if let Some(gpui::accesskit::ActionData::NumericValue(value)) = data
                                    && let Some(value) = finite_f32(*value)
                                {
                                    on_change(&value.clamp(MIN_RATIO, MAX_RATIO), window, cx);
                                    cx.stop_propagation();
                                }
                            }
                        })
                        .on_a11y_action(AccessibleAction::Increment, {
                            let on_change = on_change.clone();
                            move |_, window, cx| {
                                on_change(
                                    &(ratio + RATIO_STEP).clamp(MIN_RATIO, MAX_RATIO),
                                    window,
                                    cx,
                                );
                                cx.stop_propagation();
                            }
                        })
                        .on_a11y_action(AccessibleAction::Decrement, {
                            let on_change = on_change.clone();
                            move |_, window, cx| {
                                on_change(
                                    &(ratio - RATIO_STEP).clamp(MIN_RATIO, MAX_RATIO),
                                    window,
                                    cx,
                                );
                                cx.stop_propagation();
                            }
                        });
                }
                divider
            })
            .child({
                let mut pane = div().overflow_hidden().px(pane_pad).child(self.right);
                pane.style().flex_basis = Some(relative(0.).into());
                pane.style().flex_grow = Some(right_grow);
                pane.style().flex_shrink = Some(0.);
                pane
            });

        // 拖拽中：根据鼠标位置更新比例
        root = root
            .on_mouse_move({
                let id = id.clone();
                let bounds = bounds.clone();
                move |event: &MouseMoveEvent, window, cx| {
                    if cx.default_global::<SplitterDrag>().0.as_ref() != Some(&id) {
                        return;
                    }
                    let Some(b) = bounds.get() else { return };
                    let Some(ratio) = (if vertical {
                        pointer_ratio(
                            f32::from(event.position.y),
                            f32::from(b.top()),
                            f32::from(b.size.height),
                        )
                    } else {
                        pointer_ratio(
                            f32::from(event.position.x),
                            f32::from(b.left()),
                            f32::from(b.size.width),
                        )
                    }) else {
                        return;
                    };
                    if let Some(on_change) = &on_change {
                        on_change(&ratio, window, cx);
                    }
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let id = id.clone();
                move |_, _, cx| {
                    let drag = cx.default_global::<SplitterDrag>();
                    if drag.0.as_ref() == Some(&id) {
                        drag.0 = None;
                    }
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let id = id.clone();
                move |_, _, cx| {
                    let drag = cx.default_global::<SplitterDrag>();
                    if drag.0.as_ref() == Some(&id) {
                        drag.0 = None;
                    }
                }
            });

        root
    }
}

/// 记录容器边界的探测层（不暴露无障碍节点）
struct SplitterProbe {
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    id: ElementId,
}

impl Element for SplitterProbe {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn write_a11y_info(&self, _node: &mut gpui::accesskit::Node) {}

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
        style.position = Position::Absolute;
        style.inset.top = relative(0.).into();
        style.inset.left = relative(0.).into();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        self.bounds.set(Some(bounds));
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_RATIO, MIN_RATIO, finite_f32, keyboard_ratio, pointer_ratio};

    #[test]
    fn divider_keyboard_actions_are_orientation_aware_and_clamped() {
        assert_eq!(keyboard_ratio(0.5, "right", false), Some(0.55));
        assert_eq!(keyboard_ratio(0.5, "left", false), Some(0.45));
        assert_eq!(keyboard_ratio(0.5, "arrowright", false), Some(0.55));
        assert_eq!(keyboard_ratio(0.5, "arrowleft", false), Some(0.45));
        assert_eq!(keyboard_ratio(0.5, "arrowup", true), Some(0.55));
        assert_eq!(keyboard_ratio(0.5, "arrowdown", true), Some(0.45));
        assert_eq!(keyboard_ratio(0.5, "arrowup", false), None);
        assert_eq!(keyboard_ratio(0.5, "arrowdown", false), None);
        assert_eq!(keyboard_ratio(0.5, "arrowright", true), None);
        assert_eq!(keyboard_ratio(0.5, "arrowleft", true), None);
        assert_eq!(keyboard_ratio(0.5, "home", false), Some(MIN_RATIO));
        assert_eq!(keyboard_ratio(0.5, "end", true), Some(MAX_RATIO));
        assert_eq!(
            keyboard_ratio(MAX_RATIO, "arrowright", false),
            Some(MAX_RATIO)
        );
        assert_eq!(
            keyboard_ratio(MIN_RATIO, "arrowleft", false),
            Some(MIN_RATIO)
        );
        assert_eq!(keyboard_ratio(0.5, "enter", false), None);
    }

    #[test]
    fn divider_pointer_and_set_value_reject_invalid_geometry() {
        assert_eq!(pointer_ratio(75., 25., 100.), Some(0.5));
        assert_eq!(pointer_ratio(-25., 0., 100.), Some(MIN_RATIO));
        assert_eq!(pointer_ratio(125., 0., 100.), Some(MAX_RATIO));
        assert_eq!(pointer_ratio(0., 0., 0.), None);
        assert_eq!(pointer_ratio(f32::NAN, 0., 100.), None);
        assert_eq!(finite_f32(f64::NAN), None);
        assert_eq!(finite_f32(f64::INFINITY), None);
        assert_eq!(finite_f32(0.5), Some(0.5));
    }
}
