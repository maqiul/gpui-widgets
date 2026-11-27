//! 一次性验证码输入：逐格显示、单焦点导航、整串粘贴与完成回调。

use std::rc::Rc;

use gpui::{
    AccessibleAction, App, Context, CursorStyle, ElementId, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Render, Role, SharedString,
    StatefulInteractiveElement, Styled, Window, accesskit, div, px,
};

use crate::a11y::{self, Semantics};
use crate::input::text_input::{self, InputSize, InputStatus, InputVariant};
use crate::theme::{self, Tokens};

const MAX_LENGTH: usize = 32;

pub fn otp_state(
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    tokens: &Tokens,
    hover: bool,
    focused: bool,
    disabled: bool,
) -> text_input::InputStyle {
    text_input::state(variant, status, size, tokens, hover, focused, disabled)
}

pub fn normalize_otp_char(value: char) -> Option<char> {
    value.is_alphanumeric().then_some(value)
}

pub fn normalize_otp_text(value: &str) -> Vec<char> {
    value.chars().filter_map(normalize_otp_char).collect()
}

pub fn otp_target(current: usize, key: &str, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key {
        "left" => Some(current.saturating_sub(1)),
        "right" => Some((current + 1).min(len - 1)),
        "home" => Some(0),
        "end" => Some(len - 1),
        _ => None,
    }
}

pub fn insert_otp_text(chars: &mut [char], active: &mut usize, text: &str) -> bool {
    let input = normalize_otp_text(text);
    if input.is_empty() || chars.is_empty() {
        return false;
    }
    let mut changed = false;
    let start = *active;
    let input_len = input.len();
    for (offset, value) in input.into_iter().enumerate() {
        let index = start + offset;
        if index >= chars.len() {
            break;
        }
        if chars[index] != value {
            chars[index] = value;
            changed = true;
        }
    }
    *active = (start + input_len).min(chars.len() - 1);
    changed
}

pub struct OtpInput {
    focus_handle: FocusHandle,
    id: ElementId,
    chars: Vec<char>,
    active: usize,
    label: SharedString,
    placeholder: SharedString,
    disabled: bool,
    mask: bool,
    separator: bool,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    on_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    on_complete: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
}

impl OtpInput {
    pub fn new(cx: &mut Context<Self>, length: usize) -> Self {
        let length = length.clamp(1, MAX_LENGTH);
        Self {
            focus_handle: cx.focus_handle(),
            id: ElementId::Name(format!("otp-input-{:?}", cx.entity_id()).into()),
            chars: vec!['\0'; length],
            active: 0,
            label: "验证码".into(),
            placeholder: "请输入验证码".into(),
            disabled: false,
            mask: false,
            separator: false,
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            on_change: None,
            on_complete: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn length(&self) -> usize {
        self.chars.len()
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn mask(mut self, mask: bool) -> Self {
        self.mask = mask;
        self
    }

    pub fn separator(mut self, separator: bool) -> Self {
        self.separator = separator;
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

    pub fn size(mut self, size: impl Into<InputSize>) -> Self {
        self.size = size.into();
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    pub fn on_complete(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_complete = Some(Rc::new(listener));
        self
    }

    pub fn value(&self) -> String {
        self.chars.iter().copied().filter(|c| *c != '\0').collect()
    }

    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars.iter().copied()
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn is_complete(&self) -> bool {
        self.chars.iter().all(|c| *c != '\0')
    }

    pub fn is_empty(&self) -> bool {
        self.chars.iter().all(|c| *c == '\0')
    }

    pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
        self.chars.fill('\0');
        for (slot, value) in self.chars.iter_mut().zip(normalize_otp_text(value)) {
            *slot = value;
        }
        self.active = 0;
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        if self.is_empty() {
            return;
        }
        self.chars.fill('\0');
        self.active = 0;
        cx.notify();
    }

    pub fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.disabled {
            self.focus_handle.focus(window, cx);
        }
    }

    fn notify_change(&self, was_complete: bool, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.value();
        if let Some(listener) = &self.on_change {
            let listener = listener.clone();
            let value = value.clone();
            window.defer(cx, move |window, cx| listener(&value, window, cx));
        }
        if !was_complete && self.is_complete() {
            if let Some(listener) = &self.on_complete {
                let listener = listener.clone();
                let value = value.clone();
                window.defer(cx, move |window, cx| listener(&value, window, cx));
            }
        }
    }

    fn insert_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let was_complete = self.is_complete();
        if insert_otp_text(&mut self.chars, &mut self.active, text) {
            self.notify_change(was_complete, window, cx);
            cx.notify();
        }
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.insert_text(&text, window, cx);
        }
    }

    fn set_value_from_a11y(
        &mut self,
        data: Option<&accesskit::ActionData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let Some(accesskit::ActionData::Value(value)) = data else {
            return;
        };
        let next = normalize_otp_text(value);
        let next: String = next.into_iter().collect();
        if next == self.value() {
            return;
        }
        let was_complete = self.is_complete();
        self.set_value(&next, cx);
        self.notify_change(was_complete, window, cx);
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let key = event.keystroke.key.as_str();
        let mods = event.keystroke.modifiers;
        if (mods.control || mods.platform) && !mods.alt && key == "v" {
            cx.stop_propagation();
            self.paste(window, cx);
            return;
        }
        if let Some(target) = otp_target(self.active, key, self.chars.len()) {
            cx.stop_propagation();
            self.active = target;
            cx.notify();
            return;
        }
        match key {
            "backspace" | "delete" => {
                cx.stop_propagation();
                let target = if key == "backspace" && self.chars[self.active] == '\0' {
                    self.active.saturating_sub(1)
                } else {
                    self.active
                };
                if self.chars[target] != '\0' {
                    let was_complete = self.is_complete();
                    self.chars[target] = '\0';
                    self.active = target;
                    self.notify_change(was_complete, window, cx);
                    cx.notify();
                } else {
                    self.active = target;
                    cx.notify();
                }
            }
            _ => {
                if !mods.control && !mods.alt && !mods.platform {
                    if let Some(value) = key.chars().next().filter(|_| key.chars().count() == 1) {
                        if let Some(value) = normalize_otp_char(value) {
                            cx.stop_propagation();
                            self.insert_text(&value.to_string(), window, cx);
                        }
                    }
                }
            }
        }
    }
}

impl Render for OtpInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let style = theme::with(|t| {
            otp_state(
                self.variant,
                self.status,
                self.size,
                t,
                false,
                false,
                self.disabled,
            )
        });
        let focus_style = theme::with(|t| {
            otp_state(
                self.variant,
                self.status,
                self.size,
                t,
                false,
                true,
                self.disabled,
            )
        });
        let hover_style = theme::with(|t| {
            otp_state(
                self.variant,
                self.status,
                self.size,
                t,
                true,
                false,
                self.disabled,
            )
        });
        let gap = theme::with(|t| t.padding_xs);
        let hover_bg = hover_style.bg;
        let hover_border = hover_style.border_color();
        let mut root = div().flex().flex_row().items_center().gap(px(gap)).w_full();
        if !self.disabled {
            root = root.track_focus(&self.focus_handle);
        }

        for (index, value) in self.chars.iter().copied().enumerate() {
            let active = focused && self.active == index;
            let cell_style = if active { &focus_style } else { &style };
            let mut cell = div()
                .id(ElementId::Name(
                    format!("{:?}-cell-{index}", self.id).into(),
                ))
                .w(px(cell_style.height))
                .h(px(cell_style.height))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(cell_style.font_size))
                .line_height(px(cell_style.line_height))
                .text_color(cell_style.fg)
                .cursor(if self.disabled {
                    CursorStyle::Arrow
                } else {
                    CursorStyle::PointingHand
                });
            cell = text_input::apply_frame(cell, cell_style).p_0();
            if !self.disabled && !active {
                cell = cell.hover(move |style| style.bg(hover_bg).border_color(hover_border));
            }
            if value != '\0' {
                cell = cell.child(if self.mask {
                    "•".to_string()
                } else {
                    value.to_string()
                });
            }
            if !self.disabled {
                let entity = cx.entity();
                cell = a11y::click_void(cell, move |window, cx| {
                    let _ = entity.update(cx, |this, cx| {
                        this.active = index;
                        this.focus_handle.focus(window, cx);
                        cx.notify();
                    });
                });
            }
            root = root.child(cell);
            if self.separator && index + 1 < self.chars.len() {
                root = root.child(
                    div()
                        .px(px(gap / 2.0))
                        .text_color(style.placeholder)
                        .child("-"),
                );
            }
        }

        let root = root
            .id(ElementId::Name(format!("{:?}-root", self.id).into()))
            .on_key_down(cx.listener(Self::on_key_down))
            .on_a11y_action(AccessibleAction::SetValue, {
                let entity = cx.entity();
                move |data, window, cx| {
                    let _ =
                        entity.update(cx, |this, cx| this.set_value_from_a11y(data, window, cx));
                }
            });

        Semantics::new(Role::TextInput, self.label.clone())
            .id(format!("otp-input-{:?}", self.id))
            .value(self.value())
            .placeholder(self.placeholder.clone())
            .disabled(self.disabled)
            .invalid(self.status == InputStatus::Error)
            .focusable(!self.disabled)
            .apply(root)
    }
}

impl Focusable for OtpInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otp_text_normalization_and_insertion_are_unicode_safe() {
        assert_eq!(normalize_otp_char('7'), Some('7'));
        assert_eq!(normalize_otp_char('中'), Some('中'));
        assert_eq!(normalize_otp_char('-'), None);
        assert_eq!(
            normalize_otp_text(" 12-3a中!\n"),
            vec!['1', '2', '3', 'a', '中']
        );

        let mut chars = ['\0'; 4];
        let mut active = 0;
        assert!(insert_otp_text(&mut chars, &mut active, "12-3"));
        assert_eq!(chars, ['1', '2', '3', '\0']);
        assert_eq!(active, 3);
        assert!(!insert_otp_text(&mut chars, &mut active, "---"));
    }

    #[test]
    fn otp_keyboard_targets_clamp_and_jump() {
        assert_eq!(otp_target(2, "left", 6), Some(1));
        assert_eq!(otp_target(0, "left", 6), Some(0));
        assert_eq!(otp_target(4, "right", 6), Some(5));
        assert_eq!(otp_target(5, "right", 6), Some(5));
        assert_eq!(otp_target(3, "home", 6), Some(0));
        assert_eq!(otp_target(3, "end", 6), Some(5));
        assert_eq!(otp_target(3, "up", 6), None);
        assert_eq!(otp_target(0, "left", 0), None);
    }
}
