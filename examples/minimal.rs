//! 最小接入示例：展示其他项目如何依赖 gpui-widgets。
//!
//! 运行：cargo run --example minimal

use gpui::{
    App, Bounds, Context, Entity, FocusHandle, Focusable, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, size,
};
use gpui_widgets::{Button, Checkbox, TextInput, misc::card, theme};

struct Demo {
    agreed: bool,
    name_input: Entity<TextInput>,
    focus_handle: FocusHandle,
}

impl Focusable for Demo {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(theme::bg())
            .text_color(theme::text())
            .track_focus(&self.focus_handle(cx))
            .flex()
            .items_center()
            .justify_center()
            .child(
                card().w(px(360.)).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(div().text_lg().child("gpui-widgets 最小示例"))
                        .child(
                            Checkbox::new("同意条款", self.agreed).on_change(cx.listener(
                                |this, checked, _, cx| {
                                    this.agreed = *checked;
                                    cx.notify();
                                },
                            )),
                        )
                        .child(self.name_input.clone())
                        .child(Button::new("提交").primary().on_click(cx.listener(
                            |this, _, _, cx| {
                                let name = this.name_input.read(cx).text().to_string();
                                println!("提交：agreed={}, name={name}", this.agreed);
                            },
                        ))),
                ),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        // 注册 TextInput 的编辑快捷键（应用生命周期内调用一次）
        gpui_widgets::text_input::bind_keys(cx);

        let bounds = Bounds::centered(None, size(px(480.), px(360.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("gpui-widgets minimal".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| Demo {
                    agreed: false,
                    name_input: cx.new(|cx| TextInput::new(cx).placeholder("请输入姓名")),
                    focus_handle: cx.focus_handle(),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
