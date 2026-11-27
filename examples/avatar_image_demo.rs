//! Avatar 图片、加载失败回退与受控错误态运行时示例。

use std::sync::Arc;

use gpui::{
    App, Bounds, Context, ImageSource, IntoElement, KeyBinding, ParentElement, Render, RenderImage,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::{Avatar, Button, Theme, theme};

actions!(avatar_image_demo, [Quit]);

fn sample_image() -> ImageSource {
    let mut image = image::RgbaImage::new(64, 64);
    for y in 0..64 {
        for x in 0..64 {
            let stripe = ((x / 8 + y / 8) % 2) as u8;
            image.put_pixel(
                x,
                y,
                image::Rgba([
                    40 + stripe * 120,
                    90 + (y as u8 / 2),
                    180 + (x as u8 / 2),
                    255,
                ]),
            );
        }
    }
    ImageSource::Render(Arc::new(RenderImage::new(vec![image::Frame::new(image)])))
}

struct Panel {
    image: ImageSource,
    use_image: bool,
    image_error: bool,
    status: String,
}

impl Panel {
    fn new(_: &mut Context<Self>) -> Self {
        Self {
            image: sample_image(),
            use_image: true,
            image_error: false,
            status: "图片头像已加载".into(),
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let source = if self.use_image {
            self.image.clone()
        } else {
            ImageSource::from("avatar-image-does-not-exist.png")
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .bg(theme::bg_layout())
            .child(
                div()
                    .text_lg()
                    .text_color(theme::text())
                    .child("Avatar Image"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child("图片按 cover 裁剪；加载中或失败时回退首字符，失败态可外部受控。"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .child(
                        Button::new("切换图片")
                            .id("toggle-image")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.use_image = !this.use_image;
                                this.image_error = false;
                                this.status = if this.use_image {
                                    "图片头像已加载".into()
                                } else {
                                    "已切换到缺失图片".into()
                                };
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("模拟失败")
                            .id("simulate-error")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.use_image = false;
                                this.image_error = false;
                                this.status = "等待图片加载失败回调".into();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("清除失败")
                            .id("clear-error")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.use_image = true;
                                this.image_error = false;
                                this.status = "已恢复图片头像".into();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .p_3()
                    .rounded(px(8.))
                    .bg(theme::bg_container())
                    .border_1()
                    .border_color(theme::border())
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child(self.status.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_6()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(
                                Avatar::new("张")
                                    .id("avatar-image")
                                    .size(px(64.))
                                    .image(source)
                                    .image_error(self.image_error)
                                    .on_image_error(cx.listener(|this, error, _, cx| {
                                        this.image_error = true;
                                        this.status =
                                            format!("图片加载失败，已回退首字符：{error}");
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_secondary())
                                    .child("受控图片头像"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(
                                Avatar::new("李")
                                    .id("avatar-forced-fallback")
                                    .size(px(64.))
                                    .image(self.image.clone())
                                    .image_error(true),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_secondary())
                                    .child("强制失败态"),
                            ),
                    ),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());
        let bounds = Bounds::centered(None, size(px(560.), px(360.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Avatar Image".into()),
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
