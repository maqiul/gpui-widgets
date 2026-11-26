//! 数字输入框：文本输入 + 悬停/聚焦显示的上下步进列 + 方向键步进，可选范围限制。
//! 外观对齐 antdv-next InputNumber（mode-input）：默认宽 90，步进列宽 22，
//! 常态隐藏（antd `handleVisible: auto`），`.controls_visible(true)` 对应组件令牌
//! `handleVisible: true` 常显。

use std::rc::Rc;

use gpui::{
    AccessibleAction, App, Context, CursorStyle, Entity, FocusHandle, Focusable, Hsla, IntoElement,
    KeyDownEvent, ParentElement, Render, Role, SharedString, Styled, Window, canvas, div,
    prelude::*, px, relative,
};

use crate::a11y::{self, Semantics};

use crate::{
    TextInput,
    svg_path::{paint_svg, svg_segments},
    text_input::{InputSize, InputStatus, InputVariant, apply_frame, hover_frame, state},
    theme,
    theme::metric::{input_number_metric, number_actions_bg, number_handle_state},
};

const ICON_UP: &str = "M890.5 755.3L537.9 269.2c-12.8-17.6-39-17.6-51.7 0L133.5 755.3A8 8 0 00140 768h75c5.1 0 9.9-2.5 12.9-6.6L512 369.8l284.1 391.6c3 4.1 7.8 6.6 12.9 6.6h75c6.5 0 10.3-7.4 6.5-12.7z";
const ICON_DOWN: &str = "M884 256h-75c-5.1 0-9.9 2.5-12.9 6.6L512 654.2 227.9 262.6c-3-4.1-7.8-6.6-12.9-6.6h-75c-6.5 0-10.3 7.4-6.5 12.7l352.6 486.1c12.8 17.6 39 17.6 51.7 0l352.6-486.1c3.9-5.3.1-12.7-6.4-12.7z";
const ICON_VIEW: (f32, f32, f32, f32) = (64.0, 64.0, 896.0, 896.0);

fn handle_icon(d: &'static str, size: f32, color: Hsla) -> impl IntoElement {
    let segs = svg_segments(d);
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, ICON_VIEW, &segs, color, window),
    )
    .w(px(size))
    .h(px(size))
    .flex_shrink_0()
}

/// 数字输入框（Entity，用法同 TextInput）。
///
/// 输入内容实时解析：合法数字回调 `Some(f64)`，空串回调 `None`，
/// 非法输入不回调（保留用户正在编辑的文本）。
///
/// ```ignore
/// let age = cx.new(|cx| NumberInput::new(cx).min(0.).max(150.).step(1.));
/// ```
pub struct NumberInput {
    input: Entity<TextInput>,
    placeholder: SharedString,
    min: Option<f64>,
    max: Option<f64>,
    step: f64,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    disabled: bool,
    width: Option<f32>,
    controls_visible: bool,
    on_change: Option<Rc<dyn Fn(Option<f64>, &mut Window, &mut App)>>,
}

impl NumberInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let entity = cx.entity();
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .bare(true)
                .on_change(move |text, window, cx| {
                    let _ = entity.update(cx, |this, cx| {
                        let parsed = parse_number(text);
                        // 非法输入（正在编辑中的 "-" / "1." 等）不打断用户
                        if text.is_empty() || parsed.is_some() {
                            if let Some(on_change) = this.on_change.clone() {
                                on_change(parsed, window, cx);
                            }
                        }
                    });
                })
        });
        Self {
            input,
            placeholder: "".into(),
            min: None,
            max: None,
            step: 1.,
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            disabled: false,
            width: None,
            controls_visible: false,
            on_change: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn status(mut self, status: InputStatus) -> Self {
        self.status = status;
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 控件宽度（默认 antd controlWidth 90）
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// 步进列常显（antd 组件令牌 `handleVisible: true`；默认 auto = 悬停/聚焦才显示）
    pub fn controls_visible(mut self, visible: bool) -> Self {
        self.controls_visible = visible;
        self
    }

    /// 数值变化回调（空串为 None）
    pub fn on_change(
        mut self,
        listener: impl Fn(Option<f64>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    /// 当前数值（非法 / 空为 None）
    pub fn value(&self, cx: &App) -> Option<f64> {
        parse_number(self.input.read(cx).text())
    }

    /// 程序化设置数值（不触发 on_change）
    pub fn set_value(&mut self, value: Option<f64>, _window: &mut Window, cx: &mut Context<Self>) {
        let text = value.map(format_number).unwrap_or_default();
        let input = self.input.clone();
        input.update(cx, |input, cx| input.set_text_silent(text, cx));
    }

    fn clamp(&self, value: f64) -> f64 {
        let mut value = value;
        if let Some(min) = self.min {
            value = value.max(min);
        }
        if let Some(max) = self.max {
            value = value.min(max);
        }
        value
    }

    fn step_by(&mut self, delta: f64, window: &mut Window, cx: &mut Context<Self>) {
        let current = self.value(cx).unwrap_or_else(|| self.min.unwrap_or(0.));
        let next = self.clamp(current + delta);
        self.set_value(Some(next), window, cx);
        // 静默写入不触发内部转发，手动通知外层
        if let Some(on_change) = self.on_change.clone() {
            on_change(Some(next), window, cx);
        }
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "up" => {
                self.step_by(self.step, window, cx);
                cx.stop_propagation();
            }
            "down" => {
                self.step_by(-self.step, window, cx);
                cx.stop_propagation();
            }
            _ => {}
        }
    }
}

impl Render for NumberInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.input.focus_handle(cx).is_focused(window);
        let placeholder = self.placeholder.clone();
        let (variant, status, size, disabled) =
            (self.variant, self.status, self.size, self.disabled);
        self.input.update(cx, |input, cx| {
            input.set_placeholder(placeholder.clone());
            input.set_form_state(variant, status, size, disabled, cx);
        });
        let style = theme::with(|t| state(variant, status, size, t, false, focused, disabled));
        let hover_style = theme::with(|t| state(variant, status, size, t, true, false, disabled));
        let m = theme::with(input_number_metric);
        let width = self.width.unwrap_or(m.control_width);
        let group = SharedString::from(format!("number-input-{:?}", cx.entity_id()));

        let mut root = div()
            .relative()
            .flex()
            .items_center()
            .w(px(width))
            .line_height(px(style.line_height))
            .text_size(px(style.font_size))
            .text_color(style.fg)
            .overflow_hidden()
            .on_key_down(cx.listener(Self::on_key_down))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .px(px(style.padding_inline))
                    .py(px(style.padding_block))
                    .child(self.input.clone()),
            );

        // antd mode-input：步进列绝对定位覆盖右缘，常态 opacity 0 / 宽 0，
        // 悬停或聚焦展开 handleWidth；禁用整列不渲染（display:none）。
        if !disabled {
            let (handle_fg, handle_bg, active_bg, border, actions_bg) = theme::with(|t| {
                (
                    number_handle_state(t, false, false, false).fg,
                    number_handle_state(t, true, false, false).fg,
                    number_handle_state(t, true, true, false)
                        .bg
                        .unwrap_or(t.color_fill_alter),
                    t.color_border,
                    number_actions_bg(t, variant == InputVariant::Filled, focused),
                )
            });
            let visible = focused || self.controls_visible;
            let step = self.step;
            let mut column = div()
                .absolute()
                .top_0()
                .right_0()
                .bottom_0()
                .w(px(m.handle_width))
                .flex()
                .flex_col()
                .overflow_hidden()
                .bg(actions_bg)
                .opacity(if visible { 1. } else { 0. })
                .group_hover(group.clone(), |s| s.opacity(1.));
            for (name, d, delta) in [("up", ICON_UP, step), ("down", ICON_DOWN, -step)] {
                let action_group =
                    SharedString::from(format!("number-{name}-{:?}", cx.entity_id()));
                let mut action = div()
                    .id(action_group.clone())
                    .group(action_group.clone())
                    .relative()
                    .h(relative(0.5))
                    .flex_shrink_0()
                    .group_hover(action_group.clone(), |s| s.h(relative(0.6)))
                    .flex()
                    .items_center()
                    .justify_center()
                    .border_l_1()
                    .border_color(border)
                    .cursor(CursorStyle::PointingHand)
                    .active(move |s| s.bg(active_bg))
                    .child(
                        div()
                            .group_hover(action_group.clone(), |s| s.opacity(0.))
                            .child(handle_icon(d, m.handle_font, handle_fg)),
                    )
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .opacity(0.)
                            .group_hover(action_group, |s| s.opacity(1.))
                            .child(handle_icon(d, m.handle_font, handle_bg)),
                    );
                let label = if name == "up" {
                    "增加数值"
                } else {
                    "减少数值"
                };
                action = Semantics::new(Role::Button, label)
                    .id(format!("number-{name}-{:?}", cx.entity_id()))
                    .focusable(true)
                    .apply(action);
                action = a11y::click_void(action, {
                    let entity = cx.entity();
                    move |window, cx| {
                        entity.update(cx, |this, cx| this.step_by(delta, window, cx));
                    }
                });
                if name == "down" {
                    action = action.border_t_1().border_color(border);
                }
                column = column.child(action);
            }
            root = root.child(column);
        }

        let mut root = hover_frame(apply_frame(root, &style), &hover_style, focused)
            .id(group.clone())
            .group(group);
        if !disabled {
            root = root.track_focus(&self.input.focus_handle(cx));
            let step = self.step;
            let entity = cx.entity();
            root = root
                .on_a11y_action(AccessibleAction::Increment, {
                    let entity = entity.clone();
                    move |_, window, cx| {
                        let _ = entity.update(cx, |this, cx| this.step_by(step, window, cx));
                    }
                })
                .on_a11y_action(AccessibleAction::Decrement, {
                    let entity = entity.clone();
                    move |_, window, cx| {
                        let _ = entity.update(cx, |this, cx| this.step_by(-step, window, cx));
                    }
                });
        }
        let mut semantics = Semantics::new(Role::NumberInput, "数字输入")
            .id(format!("number-input-{:?}", cx.entity_id()))
            .value(self.input.read(cx).a11y_value())
            .placeholder(placeholder)
            .numeric_step(self.step)
            .disabled(disabled)
            .invalid(status == InputStatus::Error)
            .focusable(!disabled);
        if let Some(value) = self.value(cx) {
            semantics = semantics.numeric(value);
        }
        if let (Some(min), Some(max)) = (self.min, self.max) {
            semantics = semantics.numeric_range(min, max);
        }
        let mut root = semantics.apply(root);
        match (self.min, self.max) {
            (Some(min), None) => root = root.aria_min_numeric_value(min),
            (None, Some(max)) => root = root.aria_max_numeric_value(max),
            _ => {}
        }
        self.input
            .update(cx, |input, cx| input.with_a11y_actions(root, cx))
    }
}

impl Focusable for NumberInput {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}

fn parse_number(text: &str) -> Option<f64> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    text.parse::<f64>().ok().filter(|v| v.is_finite())
}

fn format_number(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value}")
    }
}
