//! 快捷键录入控件：点击聚焦后按下组合键即可录入，如 `Ctrl + Shift + K`。
//!
//! 实现要点（Windows 平台按键派发规律）：
//! - 纯修饰键（Ctrl/Alt/Shift/Win）只派发 [`ModifiersChangedEvent`]，用于录入时的实时预览；
//! - 主键派发 [`KeyDownEvent`]，其 `keystroke.modifiers` 已携带当前按住的修饰键，据此完成录入。
//!
//! 交互：点击开始录入 → 按下组合键完成；`Esc` 取消（保留原值），`Backspace`/`Delete` 清空，
//! 右侧 `✕` 一键清除。失焦自动结束录入。
//!
//! ```ignore
//! let hotkey = cx.new(|cx| {
//!     let mut h = HotkeyInput::new(cx)
//!         .placeholder("点击录入快捷键")
//!         .allow_clear(true)
//!         .on_change(|value, _window, _cx| println!("hotkey = {value}"));
//!     h.set_binding("ctrl-shift-k", cx); // 程序化设置初值（gpui 绑定串）
//!     h
//! });
//! // render 中：div().child(hotkey.clone())
//! // hotkey.read(cx).value()   -> "Ctrl+Shift+K"（显示串）
//! // hotkey.read(cx).binding() -> "ctrl-shift-k"（可直接用于 KeyBinding::new）
//! ```

use std::rc::Rc;

use gpui::{
    AccessibleAction, App, Context, CursorStyle, Div, ElementId, FocusHandle, Focusable,
    InteractiveElement, IntoElement, KeyDownEvent, Keystroke, Modifiers, ModifiersChangedEvent,
    ParentElement, Render, Role, SharedString, StatefulInteractiveElement, Styled, Window,
    accesskit, div, px,
};

use crate::a11y::{self, Semantics};

use crate::text_input::{self, InputSize, InputStatus, InputStyle, InputVariant};
use crate::theme::{self, Tokens};

/// HotkeyInput 外框状态矩阵，语义与 [`text_input::state`] 一致
///（antd 没有快捷键录入组件，外框取 Input 令牌）。
pub fn state(
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    tokens: &Tokens,
    hover: bool,
    focused: bool,
    disabled: bool,
) -> InputStyle {
    text_input::state(variant, status, size, tokens, hover, focused, disabled)
}

/// 键帽度量（antd 无对应组件，取间距 / 圆角令牌的邻近档）
#[derive(Clone, Copy)]
struct KeyCapMetrics {
    pad_x: f32,
    radius: f32,
    font_size: f32,
}

fn key_cap_metrics(tokens: &Tokens) -> KeyCapMetrics {
    KeyCapMetrics {
        pad_x: tokens.padding_xs,
        radius: tokens.border_radius_sm,
        font_size: tokens.font_size_sm,
    }
}

/// 平台键显示名（跨平台）。
fn platform_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Cmd"
    }
    #[cfg(target_os = "windows")]
    {
        "Win"
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        "Super"
    }
}

/// 由修饰键状态生成显示片段（顺序：Ctrl / Alt / Shift / 平台键）。
fn modifier_parts(mods: Modifiers) -> Vec<SharedString> {
    let mut parts = Vec::new();
    if mods.control {
        parts.push("Ctrl".into());
    }
    if mods.alt {
        parts.push("Alt".into());
    }
    if mods.shift {
        parts.push("Shift".into());
    }
    if mods.platform {
        parts.push(platform_name().into());
    }
    parts
}

/// 是否按住主修饰键（Ctrl/Alt/平台键；Shift 单独不计）。
fn has_primary_modifier(mods: Modifiers) -> bool {
    mods.control || mods.alt || mods.platform
}

/// 主键友好显示名：单字符大写、功能键大写、命名键映射为易读文本。
fn display_key(key: &str) -> SharedString {
    if key.chars().count() == 1 {
        return key.to_ascii_uppercase().into();
    }
    if let Some(rest) = key.strip_prefix('f') {
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            return key.to_ascii_uppercase().into();
        }
    }
    match key {
        "space" => "Space",
        "enter" => "Enter",
        "tab" => "Tab",
        "escape" => "Esc",
        "backspace" => "Backspace",
        "delete" => "Del",
        "insert" => "Ins",
        "up" => "↑",
        "down" => "↓",
        "left" => "←",
        "right" => "→",
        "home" => "Home",
        "end" => "End",
        "pageup" => "PgUp",
        "pagedown" => "PgDn",
        "menu" => "Menu",
        other => other,
    }
    .to_string()
    .into()
}

/// 生成 gpui 规范绑定串（小写、`-` 分隔，平台键统一为 `cmd`），可直接用于 `KeyBinding::new`。
fn canonical_binding(mods: Modifiers, key: &str) -> String {
    let mut b = String::new();
    if mods.control {
        b.push_str("ctrl-");
    }
    if mods.alt {
        b.push_str("alt-");
    }
    if mods.shift {
        b.push_str("shift-");
    }
    if mods.platform {
        b.push_str("cmd-");
    }
    b.push_str(&key.to_ascii_lowercase());
    b
}

/// 将辅助技术传入的显示串或 gpui 绑定串规范为可解析的 gpui 绑定串。
fn normalize_a11y_binding(value: &str) -> Option<String> {
    if value == "+" || value == "-" {
        return Some(value.to_string());
    }
    if let Some(prefix) = value
        .strip_suffix("++")
        .or_else(|| value.strip_suffix("+-"))
    {
        let modifiers = normalize_a11y_binding(prefix.trim_end_matches(['+', '-']))?;
        let key = value.chars().last()?;
        return Some(if modifiers.is_empty() {
            key.to_string()
        } else {
            format!("{modifiers}-{key}")
        });
    }
    let mut parts = value
        .split(['+', '-'])
        .map(str::trim)
        .filter(|part| !part.is_empty());
    let mut normalized = String::new();
    let mut count = 0;
    while let Some(part) = parts.next() {
        let lower = part.to_ascii_lowercase();
        let part = match lower.as_str() {
            "control" | "ctrl" => "ctrl",
            "alt" | "option" => "alt",
            "shift" => "shift",
            "cmd" | "command" | "win" | "super" => "cmd",
            "esc" => "escape",
            "del" => "delete",
            "ins" => "insert",
            "pgup" => "pageup",
            "pgdn" => "pagedown",
            "↑" => "up",
            "↓" => "down",
            "←" => "left",
            "→" => "right",
            _ => lower.as_str(),
        };
        if !normalized.is_empty() {
            normalized.push('-');
        }
        normalized.push_str(part);
        count += 1;
    }
    (count > 0).then_some(normalized)
}

/// 单个键帽（令牌化度量，与 [`crate::Kbd`] 视觉一致）。
fn key_cap(label: SharedString, m: KeyCapMetrics) -> Div {
    div()
        .px(px(m.pad_x))
        .flex()
        .justify_center()
        .rounded(px(m.radius))
        .border_1()
        .border_color(theme::border())
        .bg(theme::surface())
        .shadow_sm()
        .line_height(px(m.font_size))
        .text_size(px(m.font_size))
        .text_color(theme::text())
        .child(label)
}

/// 快捷键录入控件（有状态 `Entity`）。
pub struct HotkeyInput {
    focus_handle: FocusHandle,
    id: ElementId,
    /// 已录入组合的显示片段，如 ["Ctrl", "Shift", "K"]。
    combo: Vec<SharedString>,
    /// gpui 规范绑定串，如 "ctrl-shift-k"。
    binding: String,
    /// 录入中实时按住的修饰键预览。
    pending: Vec<SharedString>,
    recording: bool,
    disabled: bool,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    /// 是否要求至少一个主修饰键（Ctrl/Alt/平台键）。
    require_modifier: bool,
    allow_clear: bool,
    placeholder: SharedString,
    on_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
}

impl HotkeyInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            id: ElementId::Name(format!("hotkey-input-{:?}", cx.entity_id()).into()),
            combo: Vec::new(),
            binding: String::new(),
            pending: Vec::new(),
            recording: false,
            disabled: false,
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            require_modifier: false,
            allow_clear: true,
            placeholder: "点击录入快捷键".into(),
            on_change: None,
        }
    }

    /// 自定义元素 id（同页多个实例时务必区分，避免 dispatch 节点撞 id）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 空值占位文本。
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 禁用态：灰底、不可聚焦 / 录入。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 外观变体（默认 outlined）
    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 校验状态（默认 / 错误 / 警告）
    pub fn status(mut self, status: InputStatus) -> Self {
        self.status = status;
        self
    }

    /// 尺寸（small / middle / large）
    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    /// 要求至少一个主修饰键（Ctrl/Alt/平台键）；为 true 时忽略裸键与仅 Shift 的组合。
    pub fn require_modifier(mut self, require: bool) -> Self {
        self.require_modifier = require;
        self
    }

    /// 有值时显示 ✕ 一键清空（默认开启）。
    pub fn allow_clear(mut self, allow_clear: bool) -> Self {
        self.allow_clear = allow_clear;
        self
    }

    /// 值变化回调（参数为显示串，如 "Ctrl+Shift+K"）。
    ///
    /// 经 `window.defer` 延迟触发，回调内可安全 update 本实体或父视图，不会双重租约。
    pub fn on_change(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    /// 显示串，如 "Ctrl+Shift+K"（空值返回 ""）。
    pub fn value(&self) -> String {
        self.combo
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join("+")
    }

    /// gpui 规范绑定串，如 "ctrl-shift-k"（空值返回 ""）。
    pub fn binding(&self) -> &str {
        &self.binding
    }

    /// 显示片段。
    pub fn keys(&self) -> &[SharedString] {
        &self.combo
    }

    pub fn is_empty(&self) -> bool {
        self.combo.is_empty()
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// 程序化设置绑定串（gpui 格式，如 "ctrl-shift-k"）；解析失败则忽略。不触发 on_change。
    pub fn set_binding(&mut self, source: &str, cx: &mut Context<Self>) {
        if let Ok(k) = Keystroke::parse_with_os(source, cx.platform_info().os) {
            let mut display = modifier_parts(k.modifiers);
            display.push(display_key(&k.key));
            self.combo = display;
            self.binding = canonical_binding(k.modifiers, &k.key);
            self.pending.clear();
            self.recording = false;
            cx.notify();
        }
    }

    /// 程序化清空。不触发 on_change。
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.combo.clear();
        self.binding.clear();
        self.pending.clear();
        self.recording = false;
        cx.notify();
    }

    /// 主动进入录入态并聚焦。
    pub fn start_recording(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.focus_handle.focus(window, cx);
        self.recording = true;
        self.pending.clear();
        cx.notify();
    }

    /// 结束录入态（保留已录入值）。
    pub fn stop_recording(&mut self, cx: &mut Context<Self>) {
        self.recording = false;
        self.pending.clear();
        cx.notify();
    }

    fn notify_change(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(f) = self.on_change.clone() {
            let value = self.value();
            window.defer(cx, move |window, cx| f(&value, window, cx));
        }
    }

    /// 完成一次录入：由修饰键 + 主键构建显示片段与绑定串。
    fn capture(&mut self, mods: Modifiers, key: &str, window: &mut Window, cx: &mut Context<Self>) {
        let mut display = modifier_parts(mods);
        display.push(display_key(key));
        self.combo = display;
        self.binding = canonical_binding(mods, key);
        self.pending.clear();
        self.recording = false;
        self.notify_change(window, cx);
        cx.notify();
    }

    fn on_key_down(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || !self.recording {
            return;
        }
        // 拦截默认动作（Tab 焦点切换 / Enter 提交等），录入期间按键归本控件所有。
        cx.stop_propagation();
        let k = &ev.keystroke;
        let mods = k.modifiers;
        let key = k.key.as_str();

        // Esc 取消录入，保留原值。
        if key == "escape" && !mods.modified() {
            self.recording = false;
            self.pending.clear();
            cx.notify();
            return;
        }
        // Backspace / Delete 清空。
        if (key == "backspace" || key == "delete") && !mods.modified() {
            self.combo.clear();
            self.binding.clear();
            self.pending.clear();
            self.recording = false;
            self.notify_change(window, cx);
            cx.notify();
            return;
        }
        // 要求主修饰键但未按：忽略，保持录入。
        if self.require_modifier && !has_primary_modifier(mods) {
            return;
        }
        self.capture(mods, key, window, cx);
    }

    fn on_modifiers_changed(
        &mut self,
        ev: &ModifiersChangedEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || !self.recording {
            return;
        }
        self.pending = modifier_parts(ev.modifiers);
        cx.notify();
    }

    fn start_recording_clicked(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.focus_handle.focus(window, cx);
        self.recording = true;
        self.pending.clear();
        cx.notify();
    }

    fn clear_clicked(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.combo.clear();
        self.binding.clear();
        self.pending.clear();
        self.recording = false;
        self.notify_change(window, cx);
        cx.notify();
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
        let value = value.trim();
        let previous = self.binding.clone();
        if value.is_empty() {
            if self.is_empty() {
                return;
            }
            self.clear(cx);
        } else {
            let source = if Keystroke::parse_with_os(value, cx.platform_info().os).is_ok() {
                value.to_string()
            } else {
                let Some(source) = normalize_a11y_binding(value) else {
                    return;
                };
                source
            };
            let Ok(key) = Keystroke::parse_with_os(&source, cx.platform_info().os) else {
                return;
            };
            if self.require_modifier && !has_primary_modifier(key.modifiers) {
                return;
            }
            self.set_binding(&source, cx);
        }
        if self.binding != previous {
            self.notify_change(window, cx);
        }
    }
}

impl Render for HotkeyInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        // 失焦自动结束录入。
        if self.recording && !focused {
            self.recording = false;
            self.pending.clear();
        }
        let recording = self.recording;
        let (variant, status, size, disabled) =
            (self.variant, self.status, self.size, self.disabled);

        let style = theme::with(|t| state(variant, status, size, t, false, focused, disabled));
        let hover_style = theme::with(|t| state(variant, status, size, t, true, false, disabled));
        let (gap, cap) = theme::with(|t| (t.padding_xxs, key_cap_metrics(t)));

        let mut root = div()
            .key_context("HotkeyInput")
            .flex()
            .flex_row()
            .items_center()
            .gap(px(gap))
            .w_full()
            .overflow_hidden()
            .line_height(px(style.line_height))
            .text_size(px(style.font_size))
            .text_color(style.fg)
            .px(px(style.padding_inline))
            .py(px(style.padding_block))
            .cursor(if disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::PointingHand
            });
        root = text_input::apply_frame(root, &style);
        root = text_input::hover_frame(root, &hover_style, focused);
        let mut root = root.id(self.id.clone());
        if !disabled {
            root = root.track_focus(&self.focus_handle);
            let entity = cx.entity();
            root = a11y::click_void(root, {
                let entity = entity.clone();
                move |window, cx| {
                    let _ = entity.update(cx, |this, cx| this.start_recording_clicked(window, cx));
                }
            })
            .on_key_down(cx.listener(Self::on_key_down))
            .on_modifiers_changed(cx.listener(Self::on_modifiers_changed))
            .on_a11y_action(AccessibleAction::SetValue, {
                let entity = entity.clone();
                move |data, window, cx| {
                    let _ =
                        entity.update(cx, |this, cx| this.set_value_from_a11y(data, window, cx));
                }
            });
        }

        if recording {
            // 录入中：实时预览按住的修饰键 + 提示。
            if self.pending.is_empty() {
                root = root.child(
                    div()
                        .flex_1()
                        .text_color(theme::accent())
                        .child("请按下快捷键…"),
                );
            } else {
                let mut row = div().flex_1().flex().flex_row().items_center().gap(px(gap));
                for part in &self.pending {
                    row = row.child(key_cap(part.clone(), cap));
                }
                row = row.child(div().text_color(theme::text_muted()).child("…"));
                root = root.child(row);
            }
            root = root.child(
                div()
                    .text_size(px(cap.font_size))
                    .text_color(theme::text_muted())
                    .child("Esc 取消"),
            );
        } else if !self.combo.is_empty() {
            let mut row = div()
                .flex_1()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap(px(gap));
            let last = self.combo.len() - 1;
            for (i, part) in self.combo.iter().enumerate() {
                row = row.child(key_cap(part.clone(), cap));
                if i != last {
                    row = row.child(
                        div()
                            .text_size(px(cap.font_size))
                            .text_color(theme::text_muted())
                            .child("+"),
                    );
                }
            }
            root = root.child(row);
            if self.allow_clear && !disabled {
                let entity = cx.entity();
                let clear = div()
                    .id(ElementId::Name(format!("{:?}-clear", self.id).into()))
                    .cursor(CursorStyle::PointingHand)
                    .text_size(px(cap.font_size))
                    .text_color(theme::text_muted())
                    .hover(|style| style.text_color(theme::danger()))
                    .child("✕");
                let clear = Semantics::new(Role::Button, "清除快捷键")
                    .id(format!("hotkey-clear-{:?}", self.id))
                    .focusable(true)
                    .apply(clear);
                root = root.child(a11y::click_void(clear, move |window, cx| {
                    let _ = entity.update(cx, |this, cx| {
                        cx.stop_propagation();
                        this.clear_clicked(window, cx);
                    });
                }));
            }
        } else {
            root = root.child(
                div()
                    .flex_1()
                    .text_color(style.placeholder)
                    .child(self.placeholder.clone()),
            );
        }

        Semantics::new(Role::TextInput, "快捷键")
            .id(format!("hotkey-input-{:?}", self.id))
            .value(self.value())
            .placeholder(self.placeholder.clone())
            .disabled(disabled)
            .invalid(status == InputStatus::Error)
            .focusable(!disabled)
            .apply(root)
    }
}

impl Focusable for HotkeyInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
