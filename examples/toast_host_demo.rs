//! ToastHost / NotificationHost 自动到期与稳定 key 的运行时验收示例。

use std::time::Duration;

use gpui::{
    App, Bounds, Context, Div, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::{
    Button, ButtonVariant, NotificationDismissed, NotificationHost, NotificationItem, Theme,
    ToastDismissed, ToastHost, ToastItem, theme,
};

actions!(toast_host_demo, [Quit]);

fn caption(text: &str) -> Div {
    div()
        .text_sm()
        .text_color(theme::text_secondary())
        .child(text.to_string())
}

fn action(
    label: &'static str,
    id: &'static str,
    selected: bool,
    _cx: &mut Context<ToastHostDemo>,
) -> Button {
    Button::new(label).id(id).small().variant(if selected {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outlined
    })
}

struct ToastHostDemo {
    toasts: gpui::Entity<ToastHost>,
    notifications: gpui::Entity<NotificationHost>,
    sequence: u64,
    status: String,
}

impl ToastHostDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        let view = cx.weak_entity();
        let toast_dismiss = move |event: &ToastDismissed, app: &mut App| {
            let _ = view.update(app, |this, cx| {
                this.status = format!(
                    "Toast {} ({})",
                    event.key.as_deref().unwrap_or("none"),
                    if event.by_timer { "到期" } else { "手动" }
                );
                cx.notify();
            });
        };
        let view = cx.weak_entity();
        let notification_dismiss = move |event: &NotificationDismissed, app: &mut App| {
            let _ = view.update(app, |this, cx| {
                this.status = format!(
                    "Notification {} ({})",
                    event.key.as_deref().unwrap_or("none"),
                    if event.by_timer { "到期" } else { "手动" }
                );
                cx.notify();
            });
        };
        let toasts = cx.new(|cx| {
            ToastHost::new(cx)
                .config(gpui_widgets::ToastConfig {
                    max_count: 6,
                    pause_on_hover: true,
                })
                .on_dismiss(toast_dismiss)
        });
        let notifications = cx.new(|cx| {
            NotificationHost::new(cx)
                .config(gpui_widgets::NotificationConfig {
                    max_count: 4,
                    pause_on_hover: true,
                })
                .on_dismiss(notification_dismiss)
        });
        Self {
            toasts,
            notifications,
            sequence: 1,
            status: "等待弹出提示".into(),
        }
    }

    fn show_toast(&mut self, variant: gpui_widgets::ToastVariant, cx: &mut Context<Self>) {
        let key = format!("toast-demo-{}", self.sequence);
        self.sequence += 1;
        let item = ToastItem::new(format!("{variant:?} 提示 · {key}")).key(key);
        let item = match variant {
            gpui_widgets::ToastVariant::Info => item,
            gpui_widgets::ToastVariant::Success => item.success(),
            gpui_widgets::ToastVariant::Warning => item.warning(),
            gpui_widgets::ToastVariant::Danger => item.danger(),
        };
        self.toasts.update(cx, |host, cx| host.show(item, cx));
        self.status = format!("已弹出 {variant:?}");
        cx.notify();
    }

    fn show_race(&mut self, cx: &mut Context<Self>) {
        for (suffix, duration) in [("short", 1), ("middle", 10), ("top", 10)] {
            let key = format!("race-{suffix}");
            self.toasts.update(cx, |host, cx| {
                host.show_with(
                    ToastItem::new(format!("竞态验证 · {suffix}")).key(key),
                    Duration::from_secs(duration),
                    cx,
                );
            });
        }
        self.status = "1 秒后 short 到期；随后关闭 middle，top 必须保留".into();
        cx.notify();
    }

    fn show_notification(&mut self, cx: &mut Context<Self>) {
        let key = format!("notification-demo-{}", self.sequence);
        self.sequence += 1;
        self.notifications.update(cx, |host, cx| {
            host.show(
                NotificationItem::new("登录成功")
                    .description("会话已保存，可在账号区继续操作。")
                    .key(key)
                    .success(),
                cx,
            );
        });
        self.status = "已弹出 Notification，默认 4.5 秒到期".into();
        cx.notify();
    }
}

impl Render for ToastHostDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let toast_count = self.toasts.read(cx).len();
        let notification_count = self.notifications.read(cx).len();
        div()
            .id("toast-host-demo-scroll")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .pt(px(110.))
            .bg(theme::bg_layout())
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme::text())
                    .child("ToastHost / NotificationHost"),
            )
            .child(caption(
                "默认自动到期；悬停暂停；key 稳定；最新在上，超限丢最旧。",
            ))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        action("Success 3s", "show-success", false, cx).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.show_toast(gpui_widgets::ToastVariant::Success, cx)
                            },
                        )),
                    )
                    .child(
                        action("Warning 4.5s", "show-warning", false, cx).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.show_toast(gpui_widgets::ToastVariant::Warning, cx)
                            },
                        )),
                    )
                    .child(
                        action("Danger 6s", "show-danger", false, cx).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.show_toast(gpui_widgets::ToastVariant::Danger, cx)
                            },
                        )),
                    )
                    .child(
                        action("常驻", "show-persistent", false, cx).on_click(cx.listener(
                            |this, _, _, cx| {
                                let key = format!("persistent-{}", this.sequence);
                                this.sequence += 1;
                                this.toasts.update(cx, |host, cx| {
                                    host.show_with(
                                        ToastItem::new("常驻提示").key(key),
                                        Duration::ZERO,
                                        cx,
                                    );
                                });
                                this.status = "常驻提示不会自动到期".into();
                                cx.notify();
                            },
                        )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        action("竞态序列", "show-race", false, cx)
                            .on_click(cx.listener(|this, _, _, cx| this.show_race(cx))),
                    )
                    .child(
                        action("Notification 4.5s", "show-notification", false, cx)
                            .on_click(cx.listener(|this, _, _, cx| this.show_notification(cx))),
                    )
                    .child(
                        action("全部清除", "clear-all", false, cx).on_click(cx.listener(
                            |this, _, _, cx| {
                                this.toasts.update(cx, |host, cx| host.clear(cx));
                                this.notifications.update(cx, |host, cx| host.clear(cx));
                                this.status = "已清除全部提示".into();
                                cx.notify();
                            },
                        )),
                    ),
            )
            .child(caption(&format!(
                "Toast {toast_count} 条 · Notification {notification_count} 条"
            )))
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child(self.status.clone()),
            )
            .child(self.toasts.clone())
            .child(self.notifications.clone())
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());
        let bounds = Bounds::centered(None, size(px(960.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Toast Host Acceptance".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(ToastHostDemo::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
