//! 集成指南配套示例（与 docs/integration-guide.md 中的代码一致）。

use gpui::{
    App, Bounds, Context, Entity, FocusHandle, Focusable, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, size,
};
use gpui_widgets::{Button, Checkbox, Modal, Select, TextInput, ToastItem, ToastStack, theme};

struct MyApp {
    name: Entity<TextInput>,
    city: usize,
    city_open: bool,
    agreed: bool,
    modal_open: bool,
    toasts: Vec<ToastItem>,
    focus_handle: FocusHandle,
}

impl Focusable for MyApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl MyApp {
    fn push_toast(&mut self, item: ToastItem, cx: &mut Context<Self>) {
        self.toasts.push(item);
        cx.notify();
    }
}

impl Render for MyApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div()
            .size_full()
            .bg(theme::bg())
            .text_color(theme::text())
            .track_focus(&self.focus_handle(cx))
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.name.clone())
            .child(
                Select::new("city", ["北京", "上海", "广州", "深圳"], self.city)
                    .open(self.city_open)
                    .on_open_change(cx.listener(|this, open, _, cx| {
                        this.city_open = *open;
                        cx.notify();
                    }))
                    .on_select(cx.listener(|this, ix, _, cx| {
                        this.city = *ix;
                        this.city_open = false;
                        cx.notify();
                    })),
            )
            .child(
                Checkbox::new("我已阅读并同意条款", self.agreed).on_change(cx.listener(
                    |this, on, _, cx| {
                        this.agreed = *on;
                        cx.notify();
                    },
                )),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(Button::new("打开对话框").secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.modal_open = true;
                            cx.notify();
                        },
                    )))
                    .child(Button::new("提交").primary().on_click(cx.listener(
                        |this, _, _, cx| {
                            let name = this.name.read(cx).text().to_string();
                            this.push_toast(
                                ToastItem::new(format!("已提交：{name}")).success(),
                                cx,
                            );
                        },
                    ))),
            );

        // 浮层放在根视图末尾、滚动容器之外
        if self.modal_open {
            let entity = cx.entity();
            root = root.child(
                Modal::new("确认")
                    .on_close(move |_, cx| {
                        let _ = entity.update(cx, |this, cx| {
                            this.modal_open = false;
                            cx.notify();
                        });
                    })
                    .child("确认提交当前表单吗？")
                    .footer(
                        div()
                            .flex()
                            .gap_2()
                            .justify_end()
                            .child(Button::new("取消").secondary().on_click({
                                let entity = cx.entity();
                                move |_, _, cx| {
                                    let _ = entity.update(cx, |this, cx| {
                                        this.modal_open = false;
                                        cx.notify();
                                    });
                                }
                            }))
                            .child(Button::new("确定").primary().on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.modal_open = false;
                                    this.push_toast(ToastItem::new("提交成功").success(), cx);
                                },
                            ))),
                    ),
            );
        }
        if !self.toasts.is_empty() {
            root = root.child(ToastStack::new(self.toasts.clone()).on_close(cx.listener(
                |this, ix, _, cx| {
                    this.toasts.remove(*ix);
                    cx.notify();
                },
            )));
        }
        root
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        // 必须：注册输入框编辑快捷键（整个应用调用一次）
        gpui_widgets::text_input::bind_keys(cx);

        let bounds = Bounds::centered(None, size(px(520.), px(420.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| MyApp {
                    name: cx.new(|cx| TextInput::new(cx).placeholder("请输入姓名")),
                    city: 0,
                    city_open: false,
                    agreed: false,
                    modal_open: false,
                    toasts: Vec::new(),
                    focus_handle: cx.focus_handle(),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
