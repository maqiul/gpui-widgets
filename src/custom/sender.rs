//! 发送框：多行输入 + 发送按钮 + 回车发送（对标 antd X Sender）。
//!
//! ```ignore
//! let sender = cx.new(|cx| Sender::new(cx)
//!     .placeholder("输入消息…")
//!     .on_send(|text, _, cx| { /* 发送 text */ }));
//! // render 中：div().child(sender.clone())
//! ```
//!
//! Enter 发送、Shift+Enter 换行；`loading(true)` 时禁用发送。
//! 启动时需调用 [`crate::text_area::bind_keys`]。

use std::rc::Rc;

use gpui::{
    App, Context, CursorStyle, ElementId, Entity, FocusHandle, Focusable, Render, Role,
    SharedString, Window, div, prelude::*, px,
};

use crate::{TextArea, a11y, theme};

/// 发送框（Entity，用法同 TextArea）。
pub struct Sender {
    input: Entity<TextArea>,
    placeholder: SharedString,
    loading: bool,
    on_send: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    on_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
}

impl Sender {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let entity = cx.entity();
        let input_height = theme::with(crate::theme::metric::sender_metric).input_height;
        let input = cx.new(|cx| {
            let submit_entity = entity.clone();
            TextArea::new(cx)
                .height(px(input_height))
                .on_change(move |text, window, cx| {
                    let _ = entity.update(cx, |this, cx| {
                        if let Some(on_change) = this.on_change.clone() {
                            on_change(text, window, cx);
                        }
                    });
                })
                .on_submit(move |window, cx| {
                    let _ = submit_entity.update(cx, |this, cx| this.send(window, cx));
                })
        });
        Self {
            input,
            placeholder: "输入消息，Enter 发送，Shift+Enter 换行".into(),
            loading: false,
            on_send: None,
            on_change: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 加载态（禁用发送，按钮显示「发送中」）。
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// 发送回调（回车或点按钮触发，发送后自动清空）。
    pub fn on_send(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_send = Some(Rc::new(listener));
        self
    }

    /// 内容变化回调。
    pub fn on_change(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    pub fn text(&self, cx: &App) -> String {
        self.input.read(cx).text().to_string()
    }

    pub fn set_text(
        &mut self,
        text: impl Into<SharedString>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input = self.input.clone();
        input.update(cx, |area, cx| area.set_text_silent(text, cx));
    }

    fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        let text = self.input.read(cx).text().to_string();
        if text.trim().is_empty() {
            return;
        }
        if let Some(on_send) = self.on_send.clone() {
            on_send(&text, window, cx);
        }
        let input = self.input.clone();
        input.update(cx, |area, cx| area.set_text_silent("", cx));
        if let Some(on_change) = self.on_change.clone() {
            on_change("", window, cx);
        }
        cx.notify();
    }
}

impl Render for Sender {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity_tag = format!("{:?}", cx.entity_id());
        let focused = self.input.focus_handle(cx).is_focused(window);
        let placeholder = self.placeholder.clone();
        self.input
            .update(cx, |area, _| area.set_placeholder(placeholder));

        let send_style = theme::with(|t| theme::metric::sender_send_style(t, self.loading));
        let send_btn = if self.loading {
            div()
                .id(ElementId::Name(format!("sender-send-{entity_tag}").into()))
                .flex()
                .items_center()
                .justify_center()
                .px_3()
                .py_1()
                .rounded_md()
                .bg(send_style.bg)
                .text_color(send_style.fg)
                .text_sm()
                .child("发送中")
        } else {
            let hover_bg = send_style.hover_bg;
            div()
                .id(ElementId::Name(format!("sender-send-{entity_tag}").into()))
                .flex()
                .items_center()
                .justify_center()
                .px_3()
                .py_1()
                .rounded_md()
                .cursor(CursorStyle::PointingHand)
                .bg(send_style.bg)
                .text_color(send_style.fg)
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .child("发送")
                .hover(move |s| s.bg(hover_bg.unwrap()))
        };
        let send_id = format!("sender-send-{entity_tag}");
        let mut send_btn = a11y::Semantics::new(Role::Button, "发送")
            .id(send_id)
            .disabled(self.loading)
            .focusable(!self.loading)
            .apply(send_btn);
        if !self.loading {
            let entity = cx.entity();
            send_btn = a11y::click_void(send_btn, move |window, cx| {
                entity.update(cx, |this, cx| this.send(window, cx));
            });
        }

        let root_id = format!("sender-{entity_tag}");
        let root = div()
            .id(ElementId::Name(root_id.clone().into()))
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .p_2()
            .bg(theme::bg_container())
            .border_1()
            .border_color(if focused {
                theme::primary()
            } else {
                theme::border()
            })
            .rounded_lg()
            .child(div().w_full().child(self.input.clone()))
            .child(div().flex().flex_row().justify_end().child(send_btn));
        a11y::Semantics::new(Role::Group, "消息发送")
            .id(root_id)
            .apply(root)
    }
}

impl Focusable for Sender {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}
