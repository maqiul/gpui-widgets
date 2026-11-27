//! DropdownMenu SubMenu 示例：一级、二级子菜单与真实动作回调。

use gpui::{
    App, Bounds, Context, Entity, KeyBinding, Render, TitlebarOptions, Window, WindowBounds,
    WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::{Button, DropdownMenu, MenuItem, Theme, theme};

actions!(dropdown_submenu, [Quit]);

struct Panel {
    open: bool,
    status: String,
}

impl Panel {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            open: false,
            status: "就绪".into(),
        }
    }
}

fn status_action(entity: Entity<Panel>, label: &'static str) -> impl Fn(&mut Window, &mut App) {
    move |_, cx| {
        entity.update(cx, |this, cx| {
            this.status = label.into();
            cx.notify();
        });
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let set_status = |label| status_action(entity.clone(), label);

        let menu = DropdownMenu::new(
            "file-actions",
            Button::new("文件操作").secondary(),
            vec![
                MenuItem::submenu(
                    "新建",
                    vec![
                        MenuItem::new("文本文档").on_click(set_status("新建文本文档")),
                        MenuItem::new("Markdown").on_click(set_status("新建 Markdown")),
                        MenuItem::separator(),
                        MenuItem::new("文件夹").on_click(set_status("新建文件夹")),
                    ],
                ),
                MenuItem::new("编辑").on_click(set_status("编辑")),
                MenuItem::submenu(
                    "导出",
                    vec![
                        MenuItem::submenu(
                            "图片",
                            vec![
                                MenuItem::new("PNG").on_click(set_status("导出 PNG")),
                                MenuItem::new("JPG").on_click(set_status("导出 JPG")),
                            ],
                        ),
                        MenuItem::new("PDF").on_click(set_status("导出 PDF")),
                    ],
                ),
                MenuItem::new("禁用项").disabled(true),
            ],
        )
        .width(px(200.))
        .open(self.open)
        .on_open_change(cx.listener(|this, open, _, cx| {
            this.open = *open;
            cx.notify();
        }));

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::bg_layout())
            .child(
                div()
                    .h(px(56.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px_4()
                    .bg(theme::bg_container())
                    .border_b_1()
                    .border_color(theme::border())
                    .child("DropdownMenu · SubMenu"),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .child(
                        div()
                            .w(px(520.))
                            .p_6()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .bg(theme::bg_container())
                            .border_1()
                            .border_color(theme::border())
                            .rounded(px(10.))
                            .child("点击“文件操作”，然后移入“新建”或“导出”展开子菜单。")
                            .child(menu)
                            .child(
                                div()
                                    .mt_2()
                                    .p_3()
                                    .rounded(px(6.))
                                    .bg(theme::fill_tertiary())
                                    .child(format!("最近动作：{}", self.status)),
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
        let bounds = Bounds::centered(None, size(px(760.), px(520.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("DropdownMenu SubMenu".into()),
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
