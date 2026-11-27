//! Badge 状态点、尺寸、标题提示与无障碍标签运行时示例。

use gpui::{
    App, Bounds, Context, Div, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::display::avatar::AvatarSize;
use gpui_widgets::{Avatar, Badge, BadgeSize, BadgeStatus, CountBadge, Theme, theme};

actions!(badge_status_demo, [Quit]);

fn caption(text: &str) -> Div {
    div()
        .text_sm()
        .text_color(theme::text_secondary())
        .child(text.to_string())
}

fn avatar(id: &str, name: &str) -> Avatar {
    Avatar::new(name).id(id).preset(AvatarSize::Small)
}

struct Panel;

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .bg(theme::bg_layout())
            .child(
                div()
                    .text_lg()
                    .text_color(theme::text())
                    .child("Badge Status"),
            )
            .child(caption(
                "状态点使用 token 配色；计数角标支持 small、title 与 aria-label。",
            ))
            .child(caption("状态点"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap_4()
                    .child(
                        Badge::new("默认")
                            .id("status-default")
                            .status(BadgeStatus::Default),
                    )
                    .child(
                        Badge::new("进行中")
                            .id("status-processing")
                            .status(BadgeStatus::Processing)
                            .title("任务正在处理"),
                    )
                    .child(
                        Badge::new("成功")
                            .id("status-success")
                            .status(BadgeStatus::Success),
                    )
                    .child(
                        Badge::new("警告")
                            .id("status-warning")
                            .status(BadgeStatus::Warning),
                    )
                    .child(
                        Badge::new("错误")
                            .id("status-error")
                            .status(BadgeStatus::Error),
                    ),
            )
            .child(caption("计数与圆点"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap_6()
                    .child(
                        CountBadge::new(5)
                            .id("count-5")
                            .child(avatar("count-5-avatar", "A")),
                    )
                    .child(
                        CountBadge::new(100)
                            .id("count-max")
                            .max(99)
                            .child(avatar("count-max-avatar", "B")),
                    )
                    .child(
                        CountBadge::new(0)
                            .id("count-zero")
                            .show_zero()
                            .child(avatar("count-zero-avatar", "C")),
                    )
                    .child(
                        CountBadge::new(0)
                            .id("count-dot")
                            .dot()
                            .child(avatar("count-dot-avatar", "D")),
                    )
                    .child(
                        CountBadge::new(8)
                            .id("count-small")
                            .size(BadgeSize::Small)
                            .title("小尺寸计数")
                            .aria_label("八条未读消息")
                            .child(avatar("count-small-avatar", "E")),
                    )
                    .child(
                        CountBadge::new(3)
                            .id("count-custom")
                            .color(rgb(0x722ed1).into())
                            .offset(4., 4.)
                            .child(avatar("count-custom-avatar", "F")),
                    ),
            )
            .child(caption("无障碍名称覆盖"))
            .child(
                Badge::new("可见文本")
                    .id("aria-label-demo")
                    .aria_label("无障碍名称示例"),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());
        let bounds = Bounds::centered(None, size(px(720.), px(420.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Badge Status".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Panel),
        )
        .unwrap();
        cx.activate(true);
    });
}
