//! 通知中心示例：未读、已读、删除、全部已读、键盘导航与虚拟滚动。

use gpui::{
    App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::theme::{self, Theme};
use gpui_widgets::{Button, NotificationCenter, NotificationItem, NotificationVariant};

actions!(notification_center, [Quit]);

fn sample_items() -> Vec<NotificationItem> {
    let titles = [
        "构建完成",
        "有新评论",
        "磁盘空间不足",
        "同步失败",
        "欢迎使用",
        "安全提醒",
        "版本已发布",
        "共享文件更新",
        "任务已分配",
        "登录成功",
    ];
    let variants = [
        NotificationVariant::Success,
        NotificationVariant::Info,
        NotificationVariant::Warning,
        NotificationVariant::Danger,
        NotificationVariant::Info,
    ];
    (0..60)
        .map(|index| {
            NotificationItem::new(format!("{} {:02}", titles[index % titles.len()], index + 1))
                .description(format!(
                    "这是第 {} 条通知的说明文字，长文本会在行内省略。",
                    index + 1
                ))
                .timestamp(format!("{:02}:{:02}", 9 + index / 60, index % 60))
                .read(index % 4 == 0)
        })
        .enumerate()
        .map(|(index, item)| match variants[index % variants.len()] {
            NotificationVariant::Success => item.success(),
            NotificationVariant::Warning => item.warning(),
            NotificationVariant::Danger => item.danger(),
            NotificationVariant::Info => item,
        })
        .collect()
}

struct Panel {
    items: Vec<NotificationItem>,
    open: bool,
    selected: Option<usize>,
    status: String,
}

impl Panel {
    fn new(_: &mut Context<Self>) -> Self {
        Self {
            items: sample_items(),
            open: true,
            selected: Some(0),
            status: "就绪：点击条目或使用 ↑ / ↓ / Home / End".into(),
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .bg(theme::bg_layout())
            .child(div().text_lg().text_color(theme::text()).child("通知中心"))
            .child(div().text_sm().text_color(theme::text_secondary()).child(
                "右侧覆盖层包含 60 条通知，使用固定行高虚拟列表；可点击已读、删除或全部已读。",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .child(
                        Button::new("打开 / 关闭")
                            .id("toggle-center")
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.open = !this.open;
                                this.status = if this.open {
                                    "通知中心已打开".into()
                                } else {
                                    "通知中心已关闭".into()
                                };
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("新增未读")
                            .id("add-notification")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.items.insert(
                                    0,
                                    NotificationItem::new("新的系统通知")
                                        .success()
                                        .description("这条通知尚未阅读")
                                        .timestamp("现在"),
                                );
                                this.open = true;
                                this.selected = Some(0);
                                this.status = "已新增 1 条未读".into();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("清空列表")
                            .id("clear-notifications")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.items.clear();
                                this.selected = None;
                                this.status = "已进入空状态".into();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .mt_2()
                    .p_3()
                    .rounded(px(8.))
                    .bg(theme::bg_container())
                    .border_1()
                    .border_color(theme::border())
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child(self.status.clone()),
            );

        if !self.items.is_empty() {
            let items = self.items.clone();
            root = root.child(
                NotificationCenter::new("demo-notification-center", items)
                    .open(self.open)
                    .selected(self.selected)
                    .on_select(cx.listener(|this, index, _, cx| {
                        this.selected = Some(*index);
                        this.status =
                            format!("已选择第 {} 条：{}", index + 1, this.items[*index].title());
                        cx.notify();
                    }))
                    .on_read(cx.listener(|this, index: &usize, _, cx| {
                        if *index < this.items.len() {
                            this.items[*index].set_read(true);
                            this.status = format!("已读第 {} 条", index + 1);
                            cx.notify();
                        }
                    }))
                    .on_remove(cx.listener(|this, index, _, cx| {
                        if *index < this.items.len() {
                            this.items.remove(*index);
                            this.selected = this
                                .selected
                                .map(|selected| selected.min(this.items.len().saturating_sub(1)));
                            this.status = format!("已删除第 {} 条", index + 1);
                            cx.notify();
                        }
                    }))
                    .on_mark_all_read({
                        let entity = entity.clone();
                        move |_, cx| {
                            entity.update(cx, |this, cx| {
                                for item in &mut this.items {
                                    item.set_read(true);
                                }
                                this.status = "全部通知已读".into();
                                cx.notify();
                            });
                        }
                    })
                    .on_dismiss({
                        let entity = entity.clone();
                        move |_, cx| {
                            entity.update(cx, |this, cx| {
                                this.open = false;
                                this.status = "通知中心已关闭".into();
                                cx.notify();
                            });
                        }
                    }),
            );
        } else {
            root = root.child(
                NotificationCenter::new("demo-notification-center", Vec::new())
                    .open(self.open)
                    .empty_text("暂无通知，去喝杯咖啡吧")
                    .on_dismiss({
                        let entity = entity.clone();
                        move |_, cx| {
                            entity.update(cx, |this, cx| {
                                this.open = false;
                                this.status = "通知中心已关闭".into();
                                cx.notify();
                            });
                        }
                    }),
            );
        }
        root
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());
        let bounds = Bounds::centered(None, size(px(900.), px(680.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Notification Center".into()),
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
