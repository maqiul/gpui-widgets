//! 主题化滚动条：绑定 `ScrollHandle`，自绘轨道 + 滑块，支持滑块拖拽 / 轨道点击跳转 / 滚轮。
//!
//! GPUI 默认不绘制滚动条，本控件以库自身主题色补上可见、可交互的滚动反馈。
//! 拖拽采用窗口级鼠标事件（与 `Slider` 同范式），按住后允许移出轨道继续拖动。
//!
//! 绑定的 `ScrollHandle` 必须跨帧持久（放在视图字段或 `window.use_keyed_state`）。
//! 普通 Div 滚动区通过 `.track_scroll(&handle)` 共享 offset；手写滚动内容可用
//! `vertical_with_metrics` 显式传入可视高度与最大滚动距离，offset 仍由同一 handle 驱动。

use std::rc::Rc;

use gpui::{
    AccessibleAction, AnyElement, App, Bounds, DispatchPhase, Div, Element, ElementId, Entity,
    GlobalElementId, Hitbox, HitboxBehavior, Hsla, InspectorElementId, IntoElement, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Orientation, Pixels, RenderOnce,
    Role, ScrollEvent, ScrollHandle, Stateful, Style, Window, accesskit, div, fill, point,
    prelude::*, px, relative, size,
};

use crate::{a11y, theme};

/// 跨帧持久化的拖拽状态
struct ScrollbarState {
    dragging: bool,
    /// 按下时鼠标沿轴坐标
    start_mouse: Pixels,
    /// 按下时滚动偏移（负值）
    start_offset: Pixels,
}

/// 主题化滚动条。`horizontal` 绑定横向滚动区，`vertical` 绑定纵向滚动区。
///
/// ```ignore
/// // handle 需跨帧持久（视图字段或 use_keyed_state）
/// let handle = ScrollHandle::new();
/// div().id("area").overflow_x_scroll().track_scroll(&handle).child(content);
/// // 滚动条放在滚动区下方或侧边：
/// Scrollbar::horizontal("area-sb", handle.clone())
/// ```
pub struct Scrollbar {
    id: ElementId,
    handle: ScrollHandle,
    horizontal: bool,
    visible: Option<Pixels>,
    max: Option<Pixels>,
    thumb_color: Option<Hsla>,
    track_color: Option<Hsla>,
}

impl Scrollbar {
    /// 横向滚动条（宽占满父容器，高 8px，rc-virtual-list 槽宽）
    pub fn horizontal(id: impl Into<ElementId>, handle: ScrollHandle) -> Self {
        Self {
            id: id.into(),
            handle,
            horizontal: true,
            visible: None,
            max: None,
            thumb_color: None,
            track_color: None,
        }
    }

    /// 纵向滚动条（高占满父容器，宽 8px，rc-virtual-list 槽宽）
    pub fn vertical(id: impl Into<ElementId>, handle: ScrollHandle) -> Self {
        Self {
            id: id.into(),
            handle,
            horizontal: false,
            visible: None,
            max: None,
            thumb_color: None,
            track_color: None,
        }
    }

    /// 手写滚动内容的纵向适配入口：显式提供可视高度与最大滚动距离，
    /// 滚动偏移仍由同一个 `ScrollHandle` 驱动。
    pub fn vertical_with_metrics(
        id: impl Into<ElementId>,
        handle: ScrollHandle,
        visible: Pixels,
        max: Pixels,
    ) -> Self {
        Self {
            id: id.into(),
            handle,
            horizontal: false,
            visible: Some(visible),
            max: Some(max),
            thumb_color: None,
            track_color: None,
        }
    }

    /// 覆盖本条滚动条的滑块颜色（含 alpha）。优先级：本方法 >
    /// `theme::set_scrollbar_thumb` > antd 实测回退 `rgba(0,0,0,0.5)`。
    pub fn thumb_color(mut self, color: impl Into<Hsla>) -> Self {
        self.thumb_color = Some(color.into());
        self
    }

    /// 覆盖本条滚动条的轨道底色（仅悬停时可见）。优先级：本方法 >
    /// `theme::set_scrollbar_track` > 主题令牌 `colorFillQuaternary`。
    pub fn track_color(mut self, color: impl Into<Hsla>) -> Self {
        self.track_color = Some(color.into());
        self
    }

    fn max_offset(&self) -> Pixels {
        if self.horizontal {
            self.max.unwrap_or_else(|| self.handle.max_offset().x)
        } else {
            self.max.unwrap_or_else(|| self.handle.max_offset().y)
        }
    }

    fn semantics(&self) -> a11y::Semantics {
        let max = self.max_offset();
        let offset = if self.horizontal {
            self.handle.offset().x
        } else {
            self.handle.offset().y
        };
        let progress = if max > px(0.) {
            (-offset / max).clamp(0., 1.)
        } else {
            0.
        };
        let orientation = if self.horizontal {
            Orientation::Horizontal
        } else {
            Orientation::Vertical
        };
        a11y::Semantics::new(
            Role::ScrollBar,
            format!(
                "{} {}滚动条",
                self.id,
                if self.horizontal { "横向" } else { "纵向" }
            ),
        )
        .id(format!("scrollbar-{}", self.id))
        .orientation(orientation)
        .numeric(progress as f64)
        .numeric_range(0., 1.)
    }
}

fn scroll_progress(key: &str, horizontal: bool, current: f32, page: f32) -> Option<f32> {
    let key = key.strip_prefix("arrow").unwrap_or(key);
    let delta = match key {
        "home" => return Some(0.),
        "end" => return Some(1.),
        "pageup" => -page,
        "pagedown" => page,
        "up" if !horizontal => -0.1,
        "left" if horizontal => -0.1,
        "down" if !horizontal => 0.1,
        "right" if horizontal => 0.1,
        _ => return None,
    };
    Some((current + delta).clamp(0., 1.))
}

fn action_scroll_delta(data: Option<&accesskit::ActionData>, page: f32) -> f32 {
    match data {
        Some(accesskit::ActionData::ScrollUnit(accesskit::ScrollUnit::Page)) => page,
        _ => 0.1,
    }
}

fn current_progress(handle: &ScrollHandle, horizontal: bool, max: Pixels) -> f32 {
    let offset = if horizontal {
        handle.offset().x
    } else {
        handle.offset().y
    };
    (-offset / max).clamp(0., 1.)
}

impl IntoElement for Scrollbar {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let max = self.max_offset();
        let semantics = self.semantics();
        let slot = theme::metric::SCROLLBAR_SLOT;
        let plain = div()
            .id(semantics.element_id())
            .when(self.horizontal, |element| element.w_full().h(px(slot)))
            .when(!self.horizontal, |element| element.w(px(slot)).h_full());
        let mut wrapper = if max > px(0.) {
            semantics.focusable(true).apply(plain)
        } else {
            plain
        };

        if max > px(0.) {
            let horizontal = self.horizontal;
            let handle = self.handle.clone();
            let visible = if horizontal {
                self.visible.unwrap_or_else(|| handle.bounds().size.width)
            } else {
                self.visible.unwrap_or_else(|| handle.bounds().size.height)
            };
            let page = (visible / max).clamp(0., 1.);
            let set_progress: Rc<dyn Fn(f32, &mut Window, &mut App)> = Rc::new({
                let handle = handle.clone();
                move |progress, window, cx| {
                    let offset = -(max * progress.clamp(0., 1.));
                    let point = if horizontal {
                        point(offset, px(0.))
                    } else {
                        point(px(0.), offset)
                    };
                    handle.set_offset(point);
                    cx.notify(window.current_view());
                }
            });
            let keyboard_progress = set_progress.clone();
            let keyboard_handle = handle.clone();
            wrapper = wrapper.on_key_down(move |event, window, cx| {
                if let Some(progress) = scroll_progress(
                    event.keystroke.key.as_str(),
                    horizontal,
                    current_progress(&keyboard_handle, horizontal, max),
                    page,
                ) {
                    keyboard_progress(progress, window, cx);
                    cx.stop_propagation();
                }
            });

            let progress_action = set_progress.clone();
            wrapper = wrapper
                .on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
                    if let Some(accesskit::ActionData::NumericValue(progress)) = data
                        && progress.is_finite()
                    {
                        progress_action(*progress as f32, window, cx);
                    }
                })
                .on_a11y_action(AccessibleAction::Increment, {
                    let set_progress = set_progress.clone();
                    let handle = handle.clone();
                    move |_, window, cx| {
                        set_progress(current_progress(&handle, horizontal, max) + 0.1, window, cx)
                    }
                })
                .on_a11y_action(AccessibleAction::Decrement, {
                    let set_progress = set_progress.clone();
                    let handle = handle.clone();
                    move |_, window, cx| {
                        set_progress(current_progress(&handle, horizontal, max) - 0.1, window, cx)
                    }
                })
                .on_a11y_action(AccessibleAction::ScrollUp, {
                    let set_progress = set_progress.clone();
                    let handle = handle.clone();
                    move |data, window, cx| {
                        let progress = current_progress(&handle, horizontal, max);
                        set_progress(progress - action_scroll_delta(data, page), window, cx)
                    }
                })
                .on_a11y_action(AccessibleAction::ScrollDown, {
                    let set_progress = set_progress.clone();
                    let handle = handle.clone();
                    move |data, window, cx| {
                        let progress = current_progress(&handle, horizontal, max);
                        set_progress(progress + action_scroll_delta(data, page), window, cx)
                    }
                })
                .on_a11y_action(AccessibleAction::ScrollLeft, {
                    let set_progress = set_progress.clone();
                    let handle = handle.clone();
                    move |data, window, cx| {
                        let progress = current_progress(&handle, horizontal, max);
                        set_progress(progress - action_scroll_delta(data, page), window, cx)
                    }
                })
                .on_a11y_action(AccessibleAction::ScrollRight, {
                    let set_progress = set_progress.clone();
                    let handle = handle.clone();
                    move |data, window, cx| {
                        let progress = current_progress(&handle, horizontal, max);
                        set_progress(progress + action_scroll_delta(data, page), window, cx)
                    }
                });
            let offset_handle = handle.clone();
            wrapper = wrapper.on_a11y_action(
                AccessibleAction::SetScrollOffset,
                move |data, window, cx| {
                    if let Some(accesskit::ActionData::SetScrollOffset(target)) = data {
                        let offset = if horizontal { target.x } else { target.y };
                        let offset = px((offset as f32).clamp(-f32::from(max), 0.));
                        let point = if horizontal {
                            point(offset, px(0.))
                        } else {
                            point(px(0.), offset)
                        };
                        offset_handle.set_offset(point);
                        cx.notify(window.current_view());
                    }
                },
            );
        }

        wrapper.child(ScrollbarVisual {
            id: self.id,
            handle: self.handle,
            horizontal: self.horizontal,
            visible: self.visible,
            max: self.max,
            thumb_color: self.thumb_color,
            track_color: self.track_color,
        })
    }
}

/// 取色优先级：元素级覆盖 > 全局覆盖 > 主题默认。纯函数，便于矩阵单测。
pub fn resolve_layered_color(element: Option<Hsla>, global: Option<Hsla>, default: Hsla) -> Hsla {
    element.or(global).unwrap_or(default)
}

#[derive(IntoElement)]
pub(crate) struct VerticalScroll {
    id: ElementId,
    max_height: Option<Pixels>,
    child: AnyElement,
}

impl VerticalScroll {
    pub(crate) fn new(id: ElementId, child: AnyElement) -> Self {
        Self {
            id,
            max_height: None,
            child,
        }
    }

    pub(crate) fn max_h(mut self, height: Pixels) -> Self {
        self.max_height = Some(height);
        self
    }
}

impl RenderOnce for VerticalScroll {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tag = format!("{:?}", self.id);
        let handle = window
            .use_keyed_state(
                ElementId::Name(format!("{tag}-handle").into()),
                cx,
                |_, _| ScrollHandle::new(),
            )
            .read(cx)
            .clone();
        let mut root = div().flex().flex_row().flex_1().min_h(px(0.)).w_full();
        if let Some(height) = self.max_height {
            root = root.max_h(height);
        }
        root.child(
            div()
                .id(ElementId::Name(format!("{tag}-scroll").into()))
                .flex_1()
                .min_w(px(0.))
                .min_h(px(0.))
                .overflow_y_scroll()
                .track_scroll(&handle)
                .child(self.child),
        )
        .child(Scrollbar::vertical(
            ElementId::Name(format!("{tag}-scrollbar").into()),
            handle,
        ))
    }
}

pub struct PrepaintState {
    hitbox: Hitbox,
    state: Entity<ScrollbarState>,
}

struct ScrollbarVisual {
    id: ElementId,
    handle: ScrollHandle,
    horizontal: bool,
    visible: Option<Pixels>,
    max: Option<Pixels>,
    thumb_color: Option<Hsla>,
    track_color: Option<Hsla>,
}

impl ScrollbarVisual {
    fn max_offset(&self) -> Pixels {
        if self.horizontal {
            self.max.unwrap_or_else(|| self.handle.max_offset().x)
        } else {
            self.max.unwrap_or_else(|| self.handle.max_offset().y)
        }
    }
}

impl Element for ScrollbarVisual {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
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
        let slot = crate::theme::metric::SCROLLBAR_SLOT;
        if self.horizontal {
            style.size.width = relative(1.).into();
            style.size.height = px(slot).into();
        } else {
            style.size.width = px(slot).into();
            style.size.height = relative(1.).into();
        }
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| ScrollbarState {
            dragging: false,
            start_mouse: px(0.),
            start_offset: px(0.),
        });
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
        PrepaintState { hitbox, state }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let horizontal = self.horizontal;
        let handle = self.handle.clone();

        let max = self.max_offset();
        // 内容未溢出：不绘制、不挂监听
        if max <= px(0.) {
            return;
        }

        let visible = if horizontal {
            self.visible.unwrap_or_else(|| handle.bounds().size.width)
        } else {
            self.visible.unwrap_or_else(|| handle.bounds().size.height)
        };
        let offset = if horizontal {
            handle.offset().x
        } else {
            handle.offset().y
        };
        let progress = (-offset / max).clamp(0., 1.);

        let m = theme::with(crate::theme::metric::scrollbar_metric);
        let track_len = if horizontal {
            bounds.size.width
        } else {
            bounds.size.height
        };
        let thumb_len = crate::theme::metric::scrollbar_thumb_len(
            f32::from(track_len),
            f32::from(visible),
            f32::from(max),
        );
        let thumb_len = px(thumb_len);
        let travel = (track_len - thumb_len).max(px(0.));
        let thumb_pos = travel * progress;

        // 轨道：常态透明，hover 才 colorFillQuaternary（rc-virtual-list 槽整宽 8）
        let hovered = prepaint.hitbox.is_hovered(window);
        let track_bg = resolve_layered_color(
            self.track_color,
            theme::scrollbar_track_override(),
            if hovered {
                m.track_hover_bg
            } else {
                gpui::transparent_black()
            },
        );
        if hovered {
            let mut track = fill(bounds, track_bg);
            track.corner_radii = px(0.).into();
            window.paint_quad(track);
        }

        // 滑块：整槽宽、药丸圆角（CSS 99px 会被浏览器夹到半槽宽，gpui 圆角 SDF 不夹，需手动夹）
        let thumb_bounds = if horizontal {
            Bounds::new(
                point(bounds.left() + thumb_pos, bounds.top()),
                size(thumb_len, bounds.size.height),
            )
        } else {
            Bounds::new(
                point(bounds.left(), bounds.top() + thumb_pos),
                size(bounds.size.width, thumb_len),
            )
        };
        let radius = m
            .thumb_radius
            .min(f32::from(thumb_bounds.size.width) / 2.)
            .min(f32::from(thumb_bounds.size.height) / 2.);
        let mut thumb = fill(
            thumb_bounds,
            resolve_layered_color(
                self.thumb_color,
                theme::scrollbar_thumb_override(),
                m.thumb_color,
            ),
        );
        thumb.corner_radii = px(radius).into();
        window.paint_quad(thumb);

        let current_view = window.current_view();
        let state = prepaint.state.clone();
        let hitbox = prepaint.hitbox.clone();

        // 按下：点在滑块上开始拖拽；点在轨道上跳转并接着拖拽
        window.on_mouse_event({
            let handle = handle.clone();
            let state = state.clone();
            let hitbox = hitbox.clone();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || event.button != MouseButton::Left
                    || !hitbox.is_hovered(window)
                {
                    return;
                }
                let mouse = if horizontal {
                    event.position.x
                } else {
                    event.position.y
                };
                let origin = if horizontal {
                    bounds.left()
                } else {
                    bounds.top()
                };
                let along = mouse - origin;
                let on_thumb = along >= thumb_pos && along <= thumb_pos + thumb_len;
                let start_offset = if on_thumb {
                    offset
                } else {
                    // 轨道点击：让滑块中心对准点击处
                    let target = ((along - thumb_len / 2.) / travel.max(px(1.))).clamp(0., 1.);
                    let jumped = -(max * target);
                    let p = if horizontal {
                        point(jumped, px(0.))
                    } else {
                        point(px(0.), jumped)
                    };
                    handle.set_offset(p);
                    cx.notify(current_view);
                    jumped
                };
                state.update(cx, |s, _| {
                    s.dragging = true;
                    s.start_mouse = mouse;
                    s.start_offset = start_offset;
                });
            }
        });

        // 拖动：按住左键移动时持续更新（允许移出轨道）
        window.on_mouse_event({
            let handle = handle.clone();
            let state = state.clone();
            move |event: &MouseMoveEvent, phase, _window, cx| {
                if phase != DispatchPhase::Bubble || !state.read(cx).dragging {
                    return;
                }
                if event.pressed_button != Some(MouseButton::Left) {
                    state.update(cx, |s, _| s.dragging = false);
                    return;
                }
                let (start_mouse, start_offset) = {
                    let s = state.read(cx);
                    (s.start_mouse, s.start_offset)
                };
                let mouse = if horizontal {
                    event.position.x
                } else {
                    event.position.y
                };
                let delta = mouse - start_mouse;
                let scale = if travel > px(0.) { max / travel } else { 0. };
                let new_offset = (start_offset - delta * scale).clamp(-max, px(0.));
                let p = if horizontal {
                    point(new_offset, px(0.))
                } else {
                    point(px(0.), new_offset)
                };
                handle.set_offset(p);
                cx.notify(current_view);
            }
        });

        // 松开：结束拖拽
        window.on_mouse_event({
            let state = state.clone();
            move |event: &MouseUpEvent, phase, _window, cx| {
                if phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
                    return;
                }
                state.update(cx, |s, _| s.dragging = false);
            }
        });

        // 滚轮：在滚动条上滚动同样驱动内容
        window.on_mouse_event({
            let handle = handle.clone();
            let hitbox = hitbox.clone();
            move |event: &ScrollEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !hitbox.is_hovered(window) {
                    return;
                }
                let delta = event.delta.pixel_delta(window.line_height());
                let d = if horizontal {
                    if delta.x != px(0.) { delta.x } else { delta.y }
                } else {
                    delta.y
                };
                let cur = if horizontal {
                    handle.offset().x
                } else {
                    handle.offset().y
                };
                let new_offset = (cur + d).clamp(-max, px(0.));
                let p = if horizontal {
                    point(new_offset, px(0.))
                } else {
                    point(px(0.), new_offset)
                };
                handle.set_offset(p);
                cx.notify(current_view);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_scroll_respects_orientation_and_bounds() {
        assert_eq!(scroll_progress("left", true, 0.5, 0.25), Some(0.4));
        assert_eq!(scroll_progress("right", true, 0.5, 0.25), Some(0.6));
        assert_eq!(scroll_progress("arrowleft", true, 0.5, 0.25), Some(0.4));
        assert_eq!(scroll_progress("arrowright", true, 0.5, 0.25), Some(0.6));
        assert_eq!(scroll_progress("up", false, 0.5, 0.25), Some(0.4));
        assert_eq!(scroll_progress("right", false, 0.5, 0.25), None);
        assert_eq!(scroll_progress("arrowright", false, 0.5, 0.25), None);
        assert_eq!(scroll_progress("arrowup", false, 0.5, 0.25), Some(0.4));
        assert_eq!(scroll_progress("pagedown", false, 0.9, 0.25), Some(1.));
        assert_eq!(scroll_progress("home", true, 0.5, 0.25), Some(0.));
        assert_eq!(scroll_progress("end", false, 0.5, 0.25), Some(1.));
    }

    #[test]
    fn accessible_scroll_uses_requested_unit() {
        let page = accesskit::ActionData::ScrollUnit(accesskit::ScrollUnit::Page);
        let item = accesskit::ActionData::ScrollUnit(accesskit::ScrollUnit::Item);
        assert_eq!(action_scroll_delta(Some(&page), 0.25), 0.25);
        assert_eq!(action_scroll_delta(Some(&item), 0.25), 0.1);
        assert_eq!(action_scroll_delta(None, 0.25), 0.1);
    }
}
