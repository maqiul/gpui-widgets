//! OTP 输入示例：逐格录入、方向键、整串粘贴、掩码与完成回调。

use gpui::{
    App, Bounds, Context, Entity, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::text_input::{InputSize, InputStatus, InputVariant};
use gpui_widgets::{Button, OtpInput, Theme, theme};

actions!(otp_input, [Quit]);

struct Panel {
    code: Entity<OtpInput>,
    pin: Entity<OtpInput>,
    disabled: Entity<OtpInput>,
    status: String,
    completed: u32,
}

impl Panel {
    fn new(cx: &mut Context<Self>) -> Self {
        let entity = cx.entity();
        let code = cx.new(|cx| {
            OtpInput::new(cx, 6)
                .id("demo-code")
                .label("六位验证码")
                .variant(InputVariant::Outlined)
                .on_change({
                    let entity = entity.clone();
                    move |value, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.status = format!("验证码变化：{value}");
                            cx.notify();
                        });
                    }
                })
                .on_complete({
                    let entity = entity.clone();
                    move |value, _, cx| {
                        entity.update(cx, |this, cx| {
                            this.completed += 1;
                            this.status = format!("验证码完成：{value}");
                            cx.notify();
                        });
                    }
                })
        });
        let pin = cx.new(|cx| {
            OtpInput::new(cx, 4)
                .id("demo-pin")
                .label("四位密码")
                .mask(true)
                .separator(true)
                .size(InputSize::Large)
        });
        let disabled = cx.new(|cx| {
            OtpInput::new(cx, 6)
                .id("demo-disabled")
                .label("禁用验证码")
                .status(InputStatus::Error)
                .size(InputSize::Small)
                .disabled(true)
        });
        disabled.update(cx, |input, cx| input.set_value("123456", cx));
        Self {
            code,
            pin,
            disabled,
            status: "就绪".into(),
            completed: 0,
        }
    }
}

fn action_button(
    id: &'static str,
    label: &'static str,
    cx: &mut Context<Panel>,
    action: impl Fn(&mut Panel, &mut Window, &mut Context<Panel>) + 'static,
) -> impl IntoElement {
    Button::new(label)
        .id(id)
        .secondary()
        .on_click(cx.listener(move |this, _, window, cx| action(this, window, cx)))
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .bg(theme::bg_layout())
            .child(
                div()
                    .w(px(620.))
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_5()
                    .bg(theme::bg_container())
                    .border_1()
                    .border_color(theme::border())
                    .rounded(px(10.))
                    .child(div().text_lg().text_color(theme::text()).child("OTP 输入"))
                    .child(div().text_sm().text_color(theme::text_secondary()).child(
                        "点击格子后逐位输入；支持 Left / Right / Home / End、Backspace、整串粘贴。",
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(div().text_sm().child("六位验证码"))
                            .child(self.code.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(div().text_sm().child("四位掩码密码（大号 / 分隔符）"))
                            .child(self.pin.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(div().text_sm().child("错误态 / 小号 / 禁用"))
                            .child(self.disabled.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .child(action_button(
                                "fill-code",
                                "填入 123456",
                                cx,
                                |this, _, cx| {
                                    this.code.update(cx, |input, cx| {
                                        input.set_value("123456", cx);
                                    });
                                    this.status = "已程序化填入".into();
                                    cx.notify();
                                },
                            ))
                            .child(action_button("clear-code", "清空", cx, |this, _, cx| {
                                this.code.update(cx, |input, cx| input.clear(cx));
                                this.status = "已清空".into();
                                cx.notify();
                            }))
                            .child(action_button(
                                "focus-code",
                                "聚焦",
                                cx,
                                |this, window, cx| {
                                    this.code.update(cx, |input, cx| input.focus(window, cx));
                                },
                            )),
                    )
                    .child(
                        div()
                            .p_3()
                            .rounded(px(6.))
                            .bg(theme::fill_tertiary())
                            .text_color(theme::text_secondary())
                            .child(format!("{}；完成次数：{}", self.status, self.completed)),
                    ),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());
        let bounds = Bounds::centered(None, size(px(760.), px(640.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("OTP Input".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Panel::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
