//! 数值步进器：减 / 加按钮 + 当前值显示，受控组件。

use std::rc::Rc;

use gpui::{
    AccessibleAction, App, CursorStyle, Div, ElementId, IntoElement, Role, Stateful, Window, div,
    prelude::*, px,
};

use crate::a11y;
use crate::theme;

fn stepped_value(value: f32, delta: f32, min: f32, max: f32) -> f32 {
    (value + delta).clamp(min, max)
}

fn keyboard_value(current: f32, key: &str, min: f32, max: f32, step: f32) -> Option<f32> {
    let key = key.strip_prefix("arrow").unwrap_or(key);
    let next = match key {
        "home" => min,
        "end" => max,
        "up" | "right" => stepped_value(current, step, min, max),
        "down" | "left" => stepped_value(current, -step, min, max),
        _ => return None,
    };
    Some(next)
}

fn finite_f32(value: f64) -> Option<f32> {
    let value = value as f32;
    value.is_finite().then_some(value)
}

/// 数值步进器。
///
/// ```ignore
/// Stepper::new("count", self.count)
///     .min(0.).max(10.).step(1.)
///     .on_change(cx.listener(|this, value, _, cx| {
///         this.count = *value;
///         cx.notify();
///     }))
/// ```
pub struct Stepper {
    id: ElementId,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    disabled: bool,
    on_change: Option<Rc<dyn Fn(&f32, &mut Window, &mut App)>>,
}

impl Stepper {
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value,
            min: f32::MIN,
            max: f32::MAX,
            step: 1.,
            disabled: false,
            on_change: None,
        }
    }

    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Stepper {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let m = theme::with(crate::theme::metric::stepper_metric);
        let mut row = div()
            .id(ElementId::Name(format!("{:?}-stepper", self.id).into()))
            .flex()
            .flex_row()
            .items_center()
            .h(px(m.height))
            .rounded(px(m.radius))
            .border(px(m.border_width))
            .border_color(m.border_color)
            .bg(if self.disabled { m.disabled_bg } else { m.bg })
            .overflow_hidden();

        let buttons: [(&str, f32); 2] = [("minus", -self.step), ("plus", self.step)];
        for (name, delta) in buttons {
            let next = stepped_value(self.value, delta, self.min, self.max);
            let at_bound = next == self.value;
            let idle = crate::theme::metric::stepper_button_state(
                &theme::with(|t| t.clone()),
                self.disabled,
                at_bound,
                false,
                false,
            );
            let button_id = ElementId::Name(format!("{:?}-{name}", self.id).into());
            let mut button = div()
                .id(button_id.clone())
                .w(px(m.button_size))
                .h(px(m.button_size))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(m.font_size))
                .bg(idle.bg)
                .text_color(if self.disabled {
                    m.disabled_fg
                } else {
                    idle.fg
                })
                .child(if delta < 0. { "−" } else { "+" });

            let semantics = a11y::Semantics::new(
                Role::Button,
                if delta < 0. {
                    "Decrease value"
                } else {
                    "Increase value"
                },
            )
            .id(format!("{:?}", button_id))
            .disabled(self.disabled || at_bound)
            .focusable(self.on_change.is_some() && !self.disabled && !at_bound);
            button = semantics.apply(button);

            if !self.disabled && !at_bound {
                let hover = crate::theme::metric::stepper_button_state(
                    &theme::with(|t| t.clone()),
                    false,
                    false,
                    true,
                    false,
                );
                let active = crate::theme::metric::stepper_button_state(
                    &theme::with(|t| t.clone()),
                    false,
                    false,
                    true,
                    true,
                );
                let on_change = self.on_change.clone();
                button = button
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |style| style.bg(hover.bg).text_color(hover.fg))
                    .active(move |style| style.bg(active.bg).text_color(active.fg));
                if let Some(on_change) = on_change {
                    button = a11y::click(button, move |_, window, cx| {
                        on_change(&next, window, cx);
                    });
                }
            }

            row = row.child(button);
            if name == "minus" {
                let mut value = div()
                    .id(ElementId::Name(
                        format!("{:?}-stepper-value-element", self.id).into(),
                    ))
                    .w(px(m.value_width))
                    .h(px(m.button_size))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(m.font_size))
                    .border_x(px(m.border_width))
                    .border_color(m.border_color)
                    .text_color(if self.disabled { m.disabled_fg } else { m.fg })
                    .child(format!("{}", self.value));
                value = a11y::Semantics::new(Role::SpinButton, "Value")
                    .id(format!("{:?}-stepper-value", self.id))
                    .value(format!("{}", self.value))
                    .numeric(f64::from(self.value))
                    .numeric_range(f64::from(self.min), f64::from(self.max))
                    .numeric_step(f64::from(self.step))
                    .disabled(self.disabled)
                    .read_only(self.on_change.is_none())
                    .focusable(!self.disabled && self.on_change.is_some())
                    .apply(value);
                if !self.disabled {
                    if let Some(on_change) = self.on_change.clone() {
                        let current = self.value;
                        let (min, max, step) = (self.min, self.max, self.step);
                        value = value
                            .on_key_down({
                                let on_change = on_change.clone();
                                move |event, window, cx| {
                                    if let Some(next) = keyboard_value(
                                        current,
                                        event.keystroke.key.as_str(),
                                        min,
                                        max,
                                        step,
                                    ) {
                                        on_change(&next, window, cx);
                                        cx.stop_propagation();
                                    }
                                }
                            })
                            .on_a11y_action(AccessibleAction::SetValue, {
                                let on_change = on_change.clone();
                                move |data, window, cx| {
                                    if let Some(gpui::accesskit::ActionData::NumericValue(value)) =
                                        data
                                        && let Some(value) = finite_f32(*value)
                                    {
                                        on_change(&value.clamp(min, max), window, cx);
                                        cx.stop_propagation();
                                    }
                                }
                            })
                            .on_a11y_action(AccessibleAction::Increment, {
                                let on_change = on_change.clone();
                                move |_, window, cx| {
                                    on_change(&stepped_value(current, step, min, max), window, cx);
                                    cx.stop_propagation();
                                }
                            })
                            .on_a11y_action(AccessibleAction::Decrement, {
                                let on_change = on_change.clone();
                                move |_, window, cx| {
                                    on_change(&stepped_value(current, -step, min, max), window, cx);
                                    cx.stop_propagation();
                                }
                            });
                    }
                }
                row = row.child(value);
            }
        }

        a11y::Semantics::new(Role::Group, "Stepper")
            .id(format!("{:?}-stepper-group", self.id))
            .disabled(self.disabled)
            .apply(row)
    }
}

#[cfg(test)]
mod tests {
    use super::{finite_f32, keyboard_value, stepped_value};

    #[test]
    fn value_actions_step_and_clamp() {
        assert_eq!(stepped_value(5., 1., 0., 10.), 6.);
        assert_eq!(stepped_value(5., -1., 0., 10.), 4.);
        assert_eq!(stepped_value(10., 1., 0., 10.), 10.);
        assert_eq!(stepped_value(0., -1., 0., 10.), 0.);
        assert_eq!(keyboard_value(5., "up", 0., 10., 2.), Some(7.));
        assert_eq!(keyboard_value(5., "left", 0., 10., 2.), Some(3.));
        assert_eq!(keyboard_value(5., "arrowup", 0., 10., 2.), Some(7.));
        assert_eq!(keyboard_value(5., "arrowleft", 0., 10., 2.), Some(3.));
        assert_eq!(keyboard_value(5., "home", 0., 10., 2.), Some(0.));
        assert_eq!(keyboard_value(5., "end", 0., 10., 2.), Some(10.));
        assert_eq!(keyboard_value(5., "enter", 0., 10., 2.), None);
        assert_eq!(finite_f32(f64::NAN), None);
        assert_eq!(finite_f32(5.), Some(5.));
    }
}
