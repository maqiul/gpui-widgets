//! 快捷键键帽：展示键盘按键组合，如 `Ctrl + K`（对应 antd Typography 的 kbd 重置样式）。

use gpui::{
    App, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role, SharedString, Styled,
    Window, div, px,
};

use crate::{a11y::Semantics, theme};

/// antd fontFamilyCode 在 Windows 的回退字体（'SFMono-Regular', Consolas, …）
const KBD_FONT: &str = "Consolas";

/// 快捷键键帽。
///
/// ```ignore
/// Kbd::combo("Ctrl+K")   // 渲染为 [Ctrl] + [K]
/// Kbd::new("Enter")      // 单个键帽
/// ```
#[derive(IntoElement)]
pub struct Kbd {
    keys: Vec<SharedString>,
}

impl Kbd {
    /// 单个键帽
    pub fn new(key: impl Into<SharedString>) -> Self {
        Self {
            keys: vec![key.into()],
        }
    }

    /// 组合键，按 `+` 拆分（如 "Ctrl+Shift+P"）
    pub fn combo(combo: impl AsRef<str>) -> Self {
        Self {
            keys: combo
                .as_ref()
                .split('+')
                .map(|k| k.trim().to_string().into())
                .filter(|k: &SharedString| !k.is_empty())
                .collect(),
        }
    }
}

impl RenderOnce for Kbd {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let m = theme::with(crate::theme::metric::kbd_metric);
        let label = self.keys.join(" + ");
        let semantics = Semantics::new(Role::Keyboard, label.clone()).id(format!("kbd-{label}"));
        let element_id = semantics.element_id();
        let mut row = semantics.apply(div().id(element_id).flex().items_center());
        let last = self.keys.len().saturating_sub(1);
        for (ix, key) in self.keys.into_iter().enumerate() {
            row = row.child(
                div()
                    .mx(px(m.margin_inline))
                    .px(px(m.padding_inline))
                    .pt(px(m.padding_block_start))
                    .pb(px(m.padding_block_end))
                    .flex()
                    .justify_center()
                    .rounded(px(m.radius))
                    .border(px(m.border_width))
                    .border_b(px(m.border_bottom_width))
                    .border_color(m.border_color)
                    .bg(m.bg)
                    .text_size(px(m.font_size))
                    .font_family(KBD_FONT)
                    .text_color(m.fg)
                    .child(key),
            );
            if ix != last {
                row = row.child(div().text_size(px(m.font_size)).text_color(m.fg).child("+"));
            }
        }
        row
    }
}
