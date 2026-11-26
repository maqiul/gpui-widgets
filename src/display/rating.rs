//! 星级评分：antd Rate 令牌与半星几何（StarFilled 路径 + 左半裁剪）。

use std::rc::Rc;

use gpui::{
    AccessibleAction, App, ContentMask, CursorStyle, Div, ElementId, IntoElement, Orientation,
    Role, SharedString, Stateful, Window, accesskit, canvas, div, prelude::*, px,
};

use crate::theme::metric::{rating_metric, rating_style};
use crate::{
    a11y,
    svg_path::{paint_svg, svg_segments},
    theme,
};

const STAR_D: &str = "M908.1 353.1l-253.9-36.9L540.7 86.1c-3.1-6.3-8.2-11.4-14.5-14.5-15.8-7.8-35-1.3-42.9 14.5L369.8 316.2l-253.9 36.9c-7 1-13.4 4.3-18.3 9.3a32.05 32.05 0 00.6 45.3l183.7 179.1-43.4 252.9a31.95 31.95 0 0046.4 33.7L512 754l227.1 119.4c6.2 3.3 13.4 4.4 20.3 3.2 17.4-3 29.1-19.5 26.1-36.9l-43.4-252.9 183.7-179.1c5-4.9 8.3-11.3 9.3-18.3 2.7-17.5-9.5-33.7-27-36.3z";
const STAR_VIEW: (f32, f32, f32, f32) = (64., 64., 896., 896.);

fn snap_rating_value(value: f32, max: usize, step: f32) -> f32 {
    if !value.is_finite() {
        return 0.;
    }
    ((value / step).round() * step).clamp(0., max as f32)
}

fn rating_keyboard_value(current: f32, key: &str, max: usize, step: f32) -> Option<f32> {
    let key = key.strip_prefix("arrow").unwrap_or(key);
    let value = match key {
        "home" => 0.,
        "end" => max as f32,
        "left" | "down" => current - step,
        "right" | "up" => current + step,
        _ => return None,
    };
    Some(snap_rating_value(value, max, step))
}

/// 星级评分（默认 5 星）。
///
/// ```ignore
/// Rating::new("score", self.score)
///     .on_change(cx.listener(|this, value, _, cx| {
///         this.score = *value;
///         cx.notify();
///     }))
/// ```
pub struct Rating {
    id: ElementId,
    value: f32,
    max: usize,
    readonly: bool,
    allow_half: bool,
    small: bool,
    large: bool,
    on_change: Option<Rc<dyn Fn(&f32, &mut Window, &mut App)>>,
}

impl Rating {
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value,
            max: 5,
            readonly: false,
            allow_half: false,
            small: false,
            large: false,
            on_change: None,
        }
    }

    /// 星星总数，默认 5
    pub fn max(mut self, max: usize) -> Self {
        self.max = max.max(1);
        self
    }

    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// 允许半星（antd allowHalf）
    pub fn allow_half(mut self, allow_half: bool) -> Self {
        self.allow_half = allow_half;
        self
    }

    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    pub fn large(mut self) -> Self {
        self.large = true;
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Rating {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let m = theme::with(rating_metric);
        let style = theme::with(rating_style);
        let size = if self.small {
            m.star_size_sm
        } else if self.large {
            m.star_size_lg
        } else {
            m.star_size
        };
        let segs = svg_segments(STAR_D);
        let value = self.value.clamp(0., self.max as f32);
        let id = format!("{:?}", self.id);
        let interactive = !self.readonly && self.on_change.is_some();
        let step = if self.allow_half { 0.5 } else { 1. };
        let mut semantics = if interactive {
            a11y::Semantics::new(Role::Slider, "评分")
                .id(format!("{id}-rating"))
                .numeric(value as f64)
                .numeric_range(0., self.max as f64)
                .numeric_step(step as f64)
                .orientation(Orientation::Horizontal)
                .focusable(true)
        } else {
            a11y::Semantics::new(Role::Meter, "评分")
                .id(format!("{id}-rating"))
                .numeric(value as f64)
                .numeric_range(0., self.max as f64)
                .numeric_step(step as f64)
                .value(SharedString::from(format!("{value:.1} / {} 星", self.max)))
        };
        if self.readonly {
            semantics = semantics.read_only(true);
        }
        let element_id = semantics.element_id();
        let mut root = semantics.apply(
            div()
                .id(element_id)
                .flex()
                .flex_row()
                .items_center()
                .gap(px(m.gap)),
        );

        if interactive {
            let on_change = self.on_change.clone().expect("interactive");
            let max = self.max;
            let keyboard_on_change = on_change.clone();
            root = root.on_key_down(move |event, window, cx| {
                if let Some(value) =
                    rating_keyboard_value(value, event.keystroke.key.as_str(), max, step)
                {
                    keyboard_on_change(&value, window, cx);
                    cx.stop_propagation();
                }
            });
            let value_on_change = on_change.clone();
            root = root
                .on_a11y_action(AccessibleAction::SetValue, move |data, window, cx| {
                    if let Some(accesskit::ActionData::NumericValue(value)) = data {
                        let value = *value as f32;
                        if value.is_finite() {
                            value_on_change(&snap_rating_value(value, max, step), window, cx);
                        }
                    }
                })
                .on_a11y_action(AccessibleAction::Increment, {
                    let on_change = on_change.clone();
                    move |_, window, cx| {
                        on_change(&snap_rating_value(value + step, max, step), window, cx);
                    }
                })
                .on_a11y_action(AccessibleAction::Decrement, move |_, window, cx| {
                    on_change(&snap_rating_value(value - step, max, step), window, cx);
                });
        }

        for ix in 0..self.max {
            let fill = (value - ix as f32).clamp(0., 1.);
            let segs = segs.clone();
            let star = canvas(
                |bounds, _, _| bounds,
                move |bounds, _, window, _| {
                    let base = if fill >= 1. {
                        style.star
                    } else {
                        style.star_bg
                    };
                    paint_svg(bounds, STAR_VIEW, &segs, base, window);
                    if fill > 0. && fill < 1. {
                        let half = gpui::Bounds::new(
                            bounds.origin,
                            gpui::size(bounds.size.width / 2., bounds.size.height),
                        );
                        window.with_content_mask(Some(ContentMask { bounds: half }), |window| {
                            paint_svg(bounds, STAR_VIEW, &segs, style.star, window)
                        });
                    }
                },
            )
            .w(px(size))
            .h(px(size));

            let mut cell = div()
                .id(ElementId::Name(format!("{:?}-star-{ix}", self.id).into()))
                .relative()
                .w(px(size))
                .h(px(size))
                .child(star);
            let semantics =
                a11y::Semantics::new(Role::Button, SharedString::from(format!("{} 星", ix + 1)))
                    .id(format!("{id}-star-{ix}-a11y"))
                    .position(ix + 1, self.max)
                    .disabled(!interactive && !self.readonly)
                    .read_only(self.readonly)
                    .focusable(interactive);
            cell = semantics.apply(cell);

            if interactive {
                let on_change = self.on_change.clone().expect("interactive");
                // 点击当前分值清零，否则设为 ix + 1（半星时左半为 ix + 0.5）
                let next_full = if ix as f32 + 1. == value {
                    0.
                } else {
                    ix as f32 + 1.
                };
                let next_half = if ix as f32 + 0.5 == value {
                    0.
                } else {
                    ix as f32 + 0.5
                };
                cell = cell.cursor(CursorStyle::PointingHand);
                cell = a11y::click_void(cell, move |window, cx| {
                    on_change(&next_full, window, cx);
                });
                if self.allow_half {
                    let on_change = self.on_change.clone().expect("interactive");
                    cell = cell.child(a11y::click_void(
                        div()
                            .id(ElementId::Name(format!("{id}-star-{ix}-half").into()))
                            .absolute()
                            .top_0()
                            .left_0()
                            .bottom_0()
                            .w(px(size / 2.))
                            .cursor(CursorStyle::PointingHand),
                        move |window, cx| {
                            on_change(&next_half, window, cx);
                            // 半星覆盖层只处理左半，避免再触发父星的整星点击。
                            cx.stop_propagation();
                        },
                    ));
                }
            }

            root = root.child(cell);
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::{rating_keyboard_value, snap_rating_value};

    #[test]
    fn values_snap_to_the_configured_step_and_clamp() {
        assert_eq!(snap_rating_value(2.3, 5, 0.5), 2.5);
        assert_eq!(snap_rating_value(2.3, 5, 1.), 2.);
        assert_eq!(snap_rating_value(-1., 5, 0.5), 0.);
        assert_eq!(snap_rating_value(8., 5, 0.5), 5.);
        assert_eq!(snap_rating_value(f32::NAN, 5, 0.5), 0.);
    }

    #[test]
    fn keyboard_values_support_half_steps_and_boundaries() {
        assert_eq!(rating_keyboard_value(2., "right", 5, 0.5), Some(2.5));
        assert_eq!(rating_keyboard_value(2.5, "left", 5, 0.5), Some(2.));
        assert_eq!(rating_keyboard_value(2., "arrowright", 5, 0.5), Some(2.5));
        assert_eq!(rating_keyboard_value(2.5, "arrowleft", 5, 0.5), Some(2.));
        assert_eq!(rating_keyboard_value(5., "arrowup", 5, 0.5), Some(5.));
        assert_eq!(rating_keyboard_value(0., "arrowdown", 5, 0.5), Some(0.));
        assert_eq!(rating_keyboard_value(2., "home", 5, 0.5), Some(0.));
        assert_eq!(rating_keyboard_value(2., "end", 5, 0.5), Some(5.));
        assert_eq!(rating_keyboard_value(2., "enter", 5, 0.5), None);
    }
}
