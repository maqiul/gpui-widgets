//! 滑杆控件：自定义 `Element` 实现，支持鼠标点击定位与拖拽。
//!
//! 拖拽状态通过 `Window::use_keyed_state` 按控件 id 持久化，
//! 因此每个 `Slider` 必须提供唯一 id。

use std::rc::Rc;

use gpui::{
    AccessibleAction, App, Bounds, DispatchPhase, Div, Element, ElementId, Entity, GlobalElementId,
    Hitbox, HitboxBehavior, Hsla, InspectorElementId, IntoElement, LayoutId, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Orientation, Pixels, Role, SharedString,
    Stateful, Style, TextRun, Window, accesskit, div, fill, point, prelude::*, px, relative, size,
};

use crate::{
    a11y,
    theme::{self, Tokens, metric::SliderMetric},
};

/// 跨帧持久化的拖拽状态
struct SliderState {
    dragging: bool,
}

/// 滑杆交互状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderStateKind {
    Normal,
    Hover,
    Active,
    Disabled,
}

/// 滑杆的纯数据外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderStyle {
    pub rail: Hsla,
    pub track: Hsla,
    pub handle: Hsla,
    pub handle_ring: Hsla,
    pub handle_outline: Hsla,
    pub dot_border: Hsla,
    pub dot_active: Hsla,
}

/// 逐实例覆写的滑杆取色：字段为 `None` 时保留令牌派生值。
///
/// 只开放视觉权重最高的四项；`handle_outline`（按下外圈）与刻度点始终跟令牌，
/// 免得把 antd 的 alpha 语义拆成手工配色。整体换主色仍建议 `theme::set_accent`。
///
/// ```ignore
/// Slider::new("volume", 0.4).colors(SliderColors::new().track(c).handle_ring(c))
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderColors {
    /// 底槽
    pub rail: Option<Hsla>,
    /// 已选段
    pub track: Option<Hsla>,
    /// 滑块内芯
    pub handle: Option<Hsla>,
    /// 滑块环（antd 下滑块的视觉主色）
    pub handle_ring: Option<Hsla>,
}

impl SliderColors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rail(mut self, color: impl Into<Hsla>) -> Self {
        self.rail = Some(color.into());
        self
    }

    pub fn track(mut self, color: impl Into<Hsla>) -> Self {
        self.track = Some(color.into());
        self
    }

    pub fn handle(mut self, color: impl Into<Hsla>) -> Self {
        self.handle = Some(color.into());
        self
    }

    pub fn handle_ring(mut self, color: impl Into<Hsla>) -> Self {
        self.handle_ring = Some(color.into());
        self
    }

    /// 把覆写套到令牌派生样式上，纯函数。
    pub fn apply(self, base: SliderStyle) -> SliderStyle {
        SliderStyle {
            rail: self.rail.unwrap_or(base.rail),
            track: self.track.unwrap_or(base.track),
            handle: self.handle.unwrap_or(base.handle),
            handle_ring: self.handle_ring.unwrap_or(base.handle_ring),
            ..base
        }
    }
}

/// 滑杆状态：轨道 `colorFillTertiary` → 悬停 `colorFillSecondary`；
/// 已选轨道与滑块环 `colorPrimaryBorder` → 悬停 `colorPrimaryBorderHover`，
/// 悬停/按下滑块环改 `colorPrimary` 并加 `colorPrimaryOutline` 外圈；
/// 禁用轨道 `colorBgContainerDisabled`，滑块环为 `colorTextDisabled` 叠在
/// `colorBgContainer` 上的实色；刻度点边框 `colorBorderSecondary`
/// （悬停 `colorFillContentHover`），已过刻度点 `colorPrimaryBorder`。
pub fn slider_state(tokens: &Tokens, state: SliderStateKind) -> SliderStyle {
    match state {
        SliderStateKind::Normal => SliderStyle {
            rail: tokens.color_fill_tertiary,
            track: tokens.color_primary_border,
            handle: tokens.color_bg_elevated,
            handle_ring: tokens.color_primary_border,
            handle_outline: tokens.color_primary_outline,
            dot_border: tokens.color_border_secondary,
            dot_active: tokens.color_primary_border,
        },
        SliderStateKind::Hover | SliderStateKind::Active => SliderStyle {
            rail: tokens.color_fill_secondary,
            track: tokens.color_primary_border_hover,
            handle: tokens.color_bg_elevated,
            handle_ring: tokens.color_primary,
            handle_outline: tokens.color_primary_outline,
            dot_border: tokens.color_fill_content_hover,
            dot_active: tokens.color_primary_border,
        },
        SliderStateKind::Disabled => SliderStyle {
            rail: tokens.color_fill_tertiary,
            track: tokens.color_bg_container_disabled,
            handle: tokens.color_bg_elevated,
            handle_ring: theme::blend(tokens.color_text_disabled, tokens.color_bg_container),
            handle_outline: tokens.color_primary_outline,
            dot_border: tokens.color_bg_container_disabled,
            dot_active: tokens.color_bg_container_disabled,
        },
    }
}

/// 通用滑杆。
///
/// ```ignore
/// Slider::new("volume", self.volume)
///     .range(0., 100.)
///     .step(1.)
///     .on_change(cx.listener(|this, value, _window, cx| {
///         this.volume = *value;
///         cx.notify();
///     }))
/// ```
pub struct Slider {
    id: ElementId,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    label: Option<SharedString>,
    disabled: bool,
    vertical: bool,
    marks: Vec<(f32, SharedString)>,
    colors: SliderColors,
    on_change: Option<Rc<dyn Fn(&f32, &mut Window, &mut App)>>,
}

impl Slider {
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value,
            min: 0.,
            max: 1.,
            step: None,
            label: None,
            disabled: false,
            vertical: false,
            marks: Vec::new(),
            colors: SliderColors::default(),
            on_change: None,
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    /// 读屏名称；默认使用 id，marks 会附加到名称末尾。
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 垂直方向（自下而上取值，需父容器给定高度）
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }

    /// 刻度标记：(取值, 标签)，在轨道上画圆点并显示文字
    pub fn marks(
        mut self,
        marks: impl IntoIterator<Item = (f32, impl Into<SharedString>)>,
    ) -> Self {
        self.marks = marks.into_iter().map(|(v, l)| (v, l.into())).collect();
        self
    }

    /// 覆写取色（底槽 / 已选段 / 滑块内芯 / 滑块环）；未覆写的字段保持令牌派生值。
    pub fn colors(mut self, colors: SliderColors) -> Self {
        self.colors = colors;
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    fn semantics(&self) -> a11y::Semantics {
        let id = self.id.to_string();
        let name = self.label.as_deref().unwrap_or(&id);
        let label = if self.marks.is_empty() {
            name.to_owned()
        } else {
            let marks = self
                .marks
                .iter()
                .map(|(_, label)| label.as_ref())
                .collect::<Vec<_>>()
                .join("、");
            format!("{name}：{marks}")
        };
        let mut semantics = a11y::Semantics::new(Role::Slider, label)
            .id(format!("slider-{id}"))
            .numeric(self.value as f64)
            .numeric_range(self.min as f64, self.max as f64)
            .orientation(if self.vertical {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            })
            .disabled(self.disabled);
        if let Some(step) = self.step {
            semantics = semantics.numeric_step(step as f64);
        }
        semantics
    }
}

fn snap_value(value: f32, min: f32, max: f32, step: Option<f32>) -> f32 {
    let mut value = value.clamp(min, max);
    if let Some(step) = step.filter(|step| *step > 0.) {
        value = (value / step).round() * step;
    }
    value.clamp(min, max)
}

fn keyboard_value(
    current: f32,
    key: &str,
    vertical: bool,
    min: f32,
    max: f32,
    step: Option<f32>,
) -> Option<f32> {
    if max <= min {
        return None;
    }
    let key = key.strip_prefix("arrow").unwrap_or(key);
    let step = step.filter(|step| *step > 0.).unwrap_or((max - min) / 100.);
    let value = match key {
        "home" => min,
        "end" => max,
        "pageup" => current + step * 10.,
        "pagedown" => current - step * 10.,
        "up" | "right" if !vertical || key == "up" => current + step,
        "down" | "left" if vertical || key == "left" => current - step,
        _ => return None,
    };
    Some(snap_value(value, min, max, Some(step)))
}

impl IntoElement for Slider {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let id = self.id.clone();
        let semantics = self.semantics();
        let element_id = semantics.element_id();
        let mut wrapper = semantics
            .focusable(!self.disabled)
            .apply(div().id(element_id))
            .when(self.vertical, |element| element.h_full())
            .when(!self.vertical, |element| element.w_full());

        if !self.disabled {
            let on_change = self.on_change.clone();
            let keyboard_on_change = on_change.clone();
            let vertical = self.vertical;
            let current = self.value;
            let (min, max, step) = (self.min, self.max, self.step);
            wrapper = wrapper.on_key_down(move |event, window, cx| {
                if let Some(value) = keyboard_value(
                    current,
                    event.keystroke.key.as_str(),
                    vertical,
                    min,
                    max,
                    step,
                ) {
                    if let Some(on_change) = &keyboard_on_change {
                        on_change(&value, window, cx);
                    }
                    cx.stop_propagation();
                }
            });
            if let Some(on_change) = on_change {
                let current = self.value;
                let (min, max, step) = (self.min, self.max, self.step);
                let increment = step.filter(|step| *step > 0.).unwrap_or(if max > min {
                    (max - min) / 100.
                } else {
                    0.
                });
                let value_on_change = on_change.clone();
                wrapper = wrapper
                    .on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
                        if let Some(accesskit::ActionData::NumericValue(value)) = data
                            && let Some(value) = value_to_f32(*value)
                        {
                            let value = snap_value(value, min, max, step);
                            value_on_change(&value, window, cx);
                        }
                    })
                    .on_a11y_action(AccessibleAction::Increment, {
                        let on_change = on_change.clone();
                        move |_, window, cx| {
                            let value = snap_value(current + increment, min, max, step);
                            on_change(&value, window, cx);
                        }
                    })
                    .on_a11y_action(AccessibleAction::Decrement, {
                        let on_change = on_change.clone();
                        move |_, window, cx| {
                            let value = snap_value(current - increment, min, max, step);
                            on_change(&value, window, cx);
                        }
                    });
            }
        }

        wrapper.child(SliderVisual {
            id,
            value: self.value,
            min: self.min,
            max: self.max,
            step: self.step,
            disabled: self.disabled,
            vertical: self.vertical,
            marks: self.marks,
            colors: self.colors,
            on_change: self.on_change,
        })
    }
}

fn value_to_f32(value: f64) -> Option<f32> {
    let value = value as f32;
    value.is_finite().then_some(value)
}

/// 内容区（去掉 antd 外边距后的可交互矩形）：水平方向只占顶部
/// `controlSize` 高的一条，刻度文字用剩余空间；垂直方向取满高。
fn content_bounds(bounds: Bounds<Pixels>, m: &SliderMetric, vertical: bool) -> Bounds<Pixels> {
    Bounds::new(
        point(
            bounds.left() + px(m.margin_full),
            bounds.top() + px(m.margin_part),
        ),
        size(
            bounds.size.width - px(m.margin_full * 2.),
            if vertical {
                bounds.size.height - px(m.margin_part * 2.)
            } else {
                px(m.control_size)
            },
        ),
    )
}

/// 由鼠标坐标换算取值（含步进吸附；垂直时自下而上）
fn value_from_pos(
    pos: gpui::Point<Pixels>,
    bounds: Bounds<Pixels>,
    vertical: bool,
    min: f32,
    max: f32,
    step: Option<f32>,
) -> f32 {
    let fraction = if vertical {
        if bounds.size.height > px(0.) {
            ((bounds.bottom() - pos.y) / bounds.size.height).clamp(0., 1.)
        } else {
            0.
        }
    } else if bounds.size.width > px(0.) {
        ((pos.x - bounds.left()) / bounds.size.width).clamp(0., 1.)
    } else {
        0.
    };
    let mut value = min + fraction * (max - min);
    if let Some(step) = step.filter(|step| *step > 0.) {
        value = (value / step).round() * step;
    }
    value.clamp(min, max)
}

pub struct PrepaintState {
    hitbox: Hitbox,
    state: Entity<SliderState>,
}

struct SliderVisual {
    id: ElementId,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    disabled: bool,
    vertical: bool,
    marks: Vec<(f32, SharedString)>,
    colors: SliderColors,
    on_change: Option<Rc<dyn Fn(&f32, &mut Window, &mut App)>>,
}

impl SliderVisual {
    fn fraction(&self) -> f32 {
        if self.max > self.min {
            ((self.value - self.min) / (self.max - self.min)).clamp(0., 1.)
        } else {
            0.
        }
    }
}

impl Element for SliderVisual {
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
        let m = theme::with(SliderMetric::resolve);
        let mut style = Style::default();
        if self.vertical {
            style.size.width = if self.marks.is_empty() {
                px(m.control_size + m.margin_full * 2.)
            } else {
                px(m.control_size + m.margin_full * 2. + 44.)
            }
            .into();
            style.size.height = relative(1.).into();
        } else {
            style.size.width = relative(1.).into();
            style.size.height = px(m.control_size
                + m.margin_part * 2.
                + if self.marks.is_empty() {
                    0.
                } else {
                    m.margin_part_with_mark
                })
            .into();
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
        let state =
            window.use_keyed_state(self.id.clone(), cx, |_, _| SliderState { dragging: false });
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
        cx: &mut App,
    ) {
        let m = theme::with(SliderMetric::resolve);
        let fraction = self.fraction();
        let has_marks = !self.marks.is_empty();
        let dragging = prepaint.state.read(cx).dragging;
        let hovered = !self.disabled && prepaint.hitbox.is_hovered(window);
        let kind = if self.disabled {
            SliderStateKind::Disabled
        } else if hovered && dragging {
            SliderStateKind::Active
        } else if hovered {
            SliderStateKind::Hover
        } else {
            SliderStateKind::Normal
        };
        let (style, radius, mark_active_text, mark_text) = theme::with(|tokens| {
            (
                slider_state(tokens, kind),
                px(tokens.border_radius_xs),
                tokens.color_text,
                tokens.color_text_description,
            )
        });
        let style = self.colors.apply(style);

        let content = content_bounds(bounds, &m, self.vertical);
        let rail_size = px(m.rail_size);
        let track_bounds = if self.vertical {
            let center_x = if has_marks {
                content.left() + px(m.margin_full)
            } else {
                content.center().x
            };
            Bounds::new(
                point(center_x - rail_size / 2., content.top()),
                size(rail_size, content.size.height),
            )
        } else {
            Bounds::new(
                point(content.left(), content.center().y - rail_size / 2.),
                size(content.size.width, rail_size),
            )
        };
        let mut track = fill(track_bounds, style.rail);
        track.corner_radii = radius.into();
        window.paint_quad(track);

        // 已选部分（垂直时自下而上）
        let filled_bounds = if self.vertical {
            Bounds::new(
                point(
                    track_bounds.left(),
                    track_bounds.bottom() - track_bounds.size.height * fraction,
                ),
                size(rail_size, track_bounds.size.height * fraction),
            )
        } else {
            Bounds::new(
                track_bounds.origin,
                size(track_bounds.size.width * fraction, rail_size),
            )
        };
        let mut filled = fill(filled_bounds, style.track);
        filled.corner_radii = radius.into();
        window.paint_quad(filled);

        // 刻度圆点与标签
        if has_marks {
            let dot_size = px(m.dot_size);
            let text_style = window.text_style();
            let font_size = px(m.mark_font);
            for (mark_value, label) in &self.marks {
                let mark_fraction = if self.max > self.min {
                    ((mark_value - self.min) / (self.max - self.min)).clamp(0., 1.)
                } else {
                    0.
                };
                let dot_center = if self.vertical {
                    point(
                        track_bounds.center().x,
                        content.bottom() - content.size.height * mark_fraction,
                    )
                } else {
                    point(
                        content.left() + content.size.width * mark_fraction,
                        track_bounds.center().y,
                    )
                };
                let mut dot = fill(
                    Bounds::new(
                        point(dot_center.x - dot_size / 2., dot_center.y - dot_size / 2.),
                        size(dot_size, dot_size),
                    ),
                    style.handle,
                );
                dot.corner_radii = (dot_size / 2.).into();
                dot.border_widths = px(m.ring_width).into();
                dot.border_color = if mark_fraction <= fraction {
                    style.dot_active
                } else {
                    style.dot_border
                };
                window.paint_quad(dot);

                let run = TextRun {
                    len: label.len(),
                    font: text_style.font(),
                    color: if mark_fraction <= fraction {
                        mark_active_text
                    } else {
                        mark_text
                    },
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let line = window
                    .text_system()
                    .shape_line(label.clone(), font_size, &[run], None);
                let origin = if self.vertical {
                    point(
                        track_bounds.right() + px(10.),
                        dot_center.y - font_size * 0.55,
                    )
                } else {
                    point(
                        dot_center.x - line.width / 2.,
                        content.top() + px(m.mark_top),
                    )
                };
                let _ = line.paint(
                    origin,
                    font_size * 1.4,
                    gpui::TextAlign::default(),
                    None,
                    window,
                    cx,
                );
            }
        }

        // 滑块：外圈（描边 + 可选外发光）→ 内圆
        let handle_center = if self.vertical {
            point(
                track_bounds.center().x,
                content.bottom() - content.size.height * fraction,
            )
        } else {
            point(
                content.left() + content.size.width * fraction,
                track_bounds.center().y,
            )
        };
        let hovered_look = matches!(kind, SliderStateKind::Hover | SliderStateKind::Active);
        let handle_size = if hovered_look {
            m.handle_size_hover
        } else {
            m.handle_size
        };
        let ring = if hovered_look {
            m.ring_width_hover
        } else {
            m.ring_width
        };
        if hovered_look {
            let halo = px(handle_size + (ring + m.outline_width) * 2.);
            let mut outer = fill(
                Bounds::new(
                    point(handle_center.x - halo / 2., handle_center.y - halo / 2.),
                    size(halo, halo),
                ),
                style.handle_outline,
            );
            outer.corner_radii = (halo / 2.).into();
            window.paint_quad(outer);
        }
        let ringed = px(handle_size + ring * 2.);
        let mut ring_quad = fill(
            Bounds::new(
                point(handle_center.x - ringed / 2., handle_center.y - ringed / 2.),
                size(ringed, ringed),
            ),
            style.handle_ring,
        );
        ring_quad.corner_radii = (ringed / 2.).into();
        window.paint_quad(ring_quad);
        let inner = px(handle_size);
        let mut handle = fill(
            Bounds::new(
                point(handle_center.x - inner / 2., handle_center.y - inner / 2.),
                size(inner, inner),
            ),
            style.handle,
        );
        handle.corner_radii = (inner / 2.).into();
        window.paint_quad(handle);

        if self.disabled {
            return;
        }

        let vertical = self.vertical;
        let (min, max, step) = (self.min, self.max, self.step);
        let on_change = self.on_change.clone();
        let hitbox = prepaint.hitbox.clone();
        let state = prepaint.state.clone();

        // 按下：开始拖拽并立即定位
        window.on_mouse_event({
            let hitbox = hitbox.clone();
            let state = state.clone();
            let on_change = on_change.clone();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
                    return;
                }
                if !hitbox.is_hovered(window) {
                    return;
                }
                state.update(cx, |state, _| state.dragging = true);
                if let Some(on_change) = &on_change {
                    on_change(
                        &value_from_pos(event.position, content, vertical, min, max, step),
                        window,
                        cx,
                    );
                }
            }
        });

        // 拖动：按住左键移动时持续更新（允许移出轨道）
        window.on_mouse_event({
            let state = state.clone();
            let on_change = on_change.clone();
            move |event: &MouseMoveEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !state.read(cx).dragging {
                    return;
                }
                if event.pressed_button != Some(MouseButton::Left) {
                    state.update(cx, |state, _| state.dragging = false);
                    return;
                }
                if let Some(on_change) = &on_change {
                    on_change(
                        &value_from_pos(event.position, content, vertical, min, max, step),
                        window,
                        cx,
                    );
                }
            }
        });

        // 松开：结束拖拽
        window.on_mouse_event(move |event: &MouseUpEvent, phase, _window, cx| {
            if phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
                return;
            }
            state.update(cx, |state, _| state.dragging = false);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_value_steps_and_clamps() {
        assert!(
            (keyboard_value(0.4, "right", false, 0., 1., Some(0.1)).unwrap() - 0.5).abs() < 1e-6
        );
        assert!(
            (keyboard_value(0.4, "arrowright", false, 0., 1., Some(0.1)).unwrap() - 0.5).abs()
                < 1e-6
        );
        assert_eq!(
            keyboard_value(0.5, "arrowright", true, 0., 1., Some(0.1)),
            None
        );
        assert_eq!(
            keyboard_value(0.95, "pageup", false, 0., 1., Some(0.1)),
            Some(1.)
        );
        assert_eq!(
            keyboard_value(0.5, "home", true, 0., 1., Some(0.1)),
            Some(0.)
        );
        assert_eq!(
            keyboard_value(0.5, "end", false, 0., 1., Some(0.1)),
            Some(1.)
        );
        assert_eq!(keyboard_value(0.5, "enter", false, 0., 1., Some(0.1)), None);
    }

    #[test]
    fn snap_value_handles_default_step_and_bounds() {
        assert_eq!(snap_value(2., 0., 1., None), 1.);
        assert_eq!(snap_value(-1., 0., 1., None), 0.);
        assert!((snap_value(0.26, 0., 1., Some(0.1)) - 0.3).abs() < 1e-6);
        assert_eq!(snap_value(0.5, 1., 1., Some(0.1)), 1.);
    }
}
