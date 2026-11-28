//! 全局动效运行时验收：控件状态、旋转、呼吸、流光、浮层进入和通知退出。

use std::time::Duration;

use gpui::{
    App, Bounds, Context, Div, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::motion::{LoopEffect, MotionLoop};
use gpui_widgets::progress::{ProgressBar, ProgressStatus};
use gpui_widgets::{
    Button, Checkbox, Drawer, DropdownMenu, MenuItem, Modal, Radio, Segmented, Skeleton, Spinner,
    Switch, Tabs, Theme, ToastHost, ToastItem, theme,
};

actions!(motion_demo, [Quit]);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Overlay {
    None,
    Modal,
    Drawer,
    Menu,
}

struct MotionDemo {
    checked: bool,
    selected: usize,
    segmented: usize,
    tab: usize,
    switch_on: bool,
    overlay: Overlay,
    toasts: gpui::Entity<ToastHost>,
}

impl MotionDemo {
    fn new(cx: &mut Context<Self>, overlay: Overlay, toast: bool) -> Self {
        let this = Self {
            checked: true,
            selected: 0,
            segmented: 1,
            tab: 0,
            switch_on: true,
            overlay,
            toasts: cx.new(ToastHost::new),
        };
        if toast {
            this.toasts.update(cx, |host, cx| {
                host.show_with(
                    ToastItem::new("5 秒后播放退出动效").key("motion-toast"),
                    Duration::from_secs(5),
                    cx,
                );
            });
        }
        this
    }

    fn caption(text: impl Into<std::string::String>) -> Div {
        div()
            .text_sm()
            .text_color(theme::text_secondary())
            .child(text.into())
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        self.overlay = Overlay::None;
        cx.notify();
    }
}

impl Render for MotionDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.weak_entity();
        let modal_close = view.clone();
        let modal =
            Modal::new("动效 Modal")
                .id("motion-modal")
                .width(px(360.0))
                .child(
                    div()
                        .p_2()
                        .child("淡入并向上移动 8px；尊重系统减少动态效果。"),
                )
                .footer(div().flex().justify_end().child(
                    Button::new("关闭").on_click(cx.listener(|this, _, _, cx| this.close(cx))),
                ))
                .on_close(move |_, cx| {
                    let _ = modal_close.update(cx, |this, cx| this.close(cx));
                });
        let view = cx.weak_entity();
        let drawer_close = view.clone();
        let drawer = Drawer::new("动效 Drawer")
            .id("motion-drawer")
            .width(px(320.0))
            .child(div().p_2().child("从右侧滑入并淡入；内容不会被轨道裁切。"))
            .on_close(move |_, cx| {
                let _ = drawer_close.update(cx, |this, cx| this.close(cx));
            });

        let mut root = div()
            .id("motion-demo-scroll")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .pt(px(110.0))
            .bg(theme::bg_layout())
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme::text())
                    .child("全局控件动效"),
            )
            .child(Self::caption(
                "100–200ms 控件过渡；200ms 浮层与通知；静止时不请求动画帧。",
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_3()
                    .child(Button::new("按压 / 悬停").id("motion-primary").primary())
                    .child(
                        Button::new("打开 Modal")
                            .id("motion-open-modal")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.overlay = Overlay::Modal;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("打开 Drawer")
                            .id("motion-open-drawer")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.overlay = Overlay::Drawer;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("Toast 1s")
                            .id("motion-toast")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toasts.update(cx, |host, cx| {
                                    host.show_with(
                                        ToastItem::new("1 秒后播放退出动效").key("motion-toast"),
                                        Duration::from_secs(1),
                                        cx,
                                    );
                                });
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_6()
                    .child(
                        Checkbox::new("选中过渡", self.checked)
                            .id("motion-checkbox")
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.checked = *checked;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                Radio::new("选项 A", self.selected == 0)
                                    .id("motion-radio-a")
                                    .on_select(cx.listener(|this, _, _, cx| {
                                        this.selected = 0;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Radio::new("选项 B", self.selected == 1)
                                    .id("motion-radio-b")
                                    .on_select(cx.listener(|this, _, _, cx| {
                                        this.selected = 1;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        Switch::new(self.switch_on)
                            .id("motion-switch")
                            .label("旋钮过渡")
                            .on_change(cx.listener(|this, on, _, cx| {
                                this.switch_on = *on;
                                cx.notify();
                            })),
                    )
                    .child(Spinner::new().tip("四点旋转")),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_6()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(Self::caption("Segmented 选中过渡"))
                                    .child(
                                        Segmented::new(["日", "周", "月"], self.segmented)
                                            .id("motion-segmented")
                                            .on_select(cx.listener(|this, ix, _, cx| {
                                                this.segmented = *ix;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(Self::caption("Tabs 指示条过渡"))
                                    .child(
                                        Tabs::new(["概览", "详情", "设置"], self.tab)
                                            .id("motion-tabs")
                                            .on_select(cx.listener(|this, ix, _, cx| {
                                                this.tab = *ix;
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        DropdownMenu::new(
                            "motion-menu",
                            Button::new("菜单悬停").secondary(),
                            vec![
                                MenuItem::new("编辑"),
                                MenuItem::separator(),
                                MenuItem::new("删除").danger(),
                            ],
                        )
                        .open(matches!(self.overlay, Overlay::Menu))
                        .on_open_change(cx.listener(
                            |this, open, _, cx| {
                                this.overlay = if *open { Overlay::Menu } else { Overlay::None };
                                cx.notify();
                            },
                        )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_6()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(Self::caption("Active 流光"))
                            .child(
                                div().w(px(220.0)).child(
                                    ProgressBar::new(0.6)
                                        .status(ProgressStatus::Active)
                                        .id("motion-progress"),
                                ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(Self::caption("Skeleton 呼吸"))
                            .child(Skeleton::new().w(px(220.0)).active()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(Self::caption("通用 Pulse"))
                            .child(MotionLoop::new(
                                "motion-pulse",
                                div()
                                    .w(px(220.0))
                                    .h(px(16.0))
                                    .rounded(px(4.0))
                                    .bg(theme::accent()),
                                LoopEffect::Pulse,
                            )),
                    ),
            );

        match self.overlay {
            Overlay::Modal => root = root.child(modal),
            Overlay::Drawer => root = root.child(drawer),
            Overlay::Menu => {}
            Overlay::None => {}
        }
        root.child(self.toasts.clone())
    }
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let overlay = match mode.as_str() {
        "modal" => Overlay::Modal,
        "drawer" => Overlay::Drawer,
        "menu" => Overlay::Menu,
        _ => Overlay::None,
    };
    let toast = mode == "toast";
    gpui_platform::application().run(move |cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());
        let bounds = Bounds::centered(None, size(px(900.0), px(760.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Global Motion Acceptance".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| MotionDemo::new(cx, overlay, toast)),
        )
        .unwrap();
        cx.activate(true);
    });
}
