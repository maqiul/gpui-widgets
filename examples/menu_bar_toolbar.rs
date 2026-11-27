//! MenuBar / Toolbar 示例：顶层菜单、子菜单、工具栏选中态与真实动作回调。

use gpui::{
    App, Bounds, Context, Entity, KeyBinding, Render, SharedString, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::{MenuBar, MenuBarItem, MenuItem, Theme, Toolbar, ToolbarItem, theme};

actions!(menu_bar_toolbar, [Quit]);

struct Panel {
    open_menu: Option<usize>,
    selected_menu: Option<usize>,
    bold: bool,
    italic: bool,
    status: String,
}

impl Panel {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            open_menu: None,
            selected_menu: None,
            bold: false,
            italic: false,
            status: "就绪".into(),
        }
    }
}

fn menu_action(entity: Entity<Panel>) -> impl Fn(usize, &SharedString, &mut Window, &mut App) {
    move |top, label, _, cx| {
        entity.update(cx, |this, cx| {
            this.selected_menu = Some(top);
            this.status = format!("菜单动作：{label}");
            cx.notify();
        });
    }
}

fn toolbar_action(entity: Entity<Panel>) -> impl Fn(&SharedString, &mut Window, &mut App) {
    move |id, _, cx| {
        entity.update(cx, |this, cx| {
            match id.as_str() {
                "bold" => {
                    this.bold = !this.bold;
                    this.status = if this.bold {
                        "粗体：开"
                    } else {
                        "粗体：关"
                    }
                    .into();
                }
                "italic" => {
                    this.italic = !this.italic;
                    this.status = if this.italic {
                        "斜体：开"
                    } else {
                        "斜体：关"
                    }
                    .into();
                }
                "save" => this.status = "已保存".into(),
                "disabled" => {}
                _ => {}
            }
            cx.notify();
        });
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let menu = MenuBar::new(
            "app-menu",
            vec![
                MenuBarItem::new("file", "文件").children(vec![
                    MenuItem::new("新建"),
                    MenuItem::new("打开"),
                    MenuItem::separator(),
                    MenuItem::new("退出").danger(),
                ]),
                MenuBarItem::new("edit", "编辑").children(vec![
                    MenuItem::new("撤销"),
                    MenuItem::new("重做"),
                    MenuItem::separator(),
                    MenuItem::submenu(
                        "更多",
                        vec![
                            MenuItem::new("首选项"),
                            MenuItem::new("禁用项").disabled(true),
                        ],
                    ),
                ]),
                MenuBarItem::new("view", "视图").children(vec![
                    MenuItem::new("显示工具栏"),
                    MenuItem::new("显示状态栏"),
                ]),
                MenuBarItem::new("help", "帮助"),
            ],
        )
        .open(self.open_menu)
        .selected(self.selected_menu)
        .menu_width(px(190.))
        .on_open_change(cx.listener(|this, open, _, cx| {
            this.open_menu = *open;
            if let Some(index) = open {
                this.selected_menu = Some(*index);
            }
            cx.notify();
        }))
        .on_select(menu_action(entity.clone()));

        let toolbar = Toolbar::new(
            "editor-toolbar",
            vec![
                ToolbarItem::new("bold", "粗体")
                    .icon(div().child("B"))
                    .icon_only()
                    .selected(self.bold),
                ToolbarItem::new("italic", "斜体")
                    .icon(div().child("I"))
                    .icon_only()
                    .selected(self.italic),
                ToolbarItem::separator(),
                ToolbarItem::new("save", "保存"),
                ToolbarItem::spacer(),
                ToolbarItem::new("disabled", "禁用工具").disabled(true),
            ],
        )
        .on_action(toolbar_action(entity));

        let vertical_toolbar = Toolbar::new(
            "side-toolbar",
            vec![
                ToolbarItem::new("select", "选择")
                    .icon_only()
                    .selected(true),
                ToolbarItem::new("pan", "平移").icon_only(),
                ToolbarItem::separator(),
                ToolbarItem::new("zoom", "缩放").icon_only(),
            ],
        )
        .vertical(true);

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::bg_layout())
            .child(menu)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .child(
                        div()
                            .w(px(64.))
                            .flex_shrink_0()
                            .flex()
                            .justify_center()
                            .pt_4()
                            .bg(theme::bg_container())
                            .border_r_1()
                            .border_color(theme::border())
                            .child(vertical_toolbar),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap_4()
                            .child(toolbar)
                            .child(
                                div()
                                    .w(px(520.))
                                    .p_6()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .bg(theme::bg_container())
                                    .border_1()
                                    .border_color(theme::border())
                                    .rounded(px(10.))
                                    .child("点击顶层菜单，或打开后横向移动切换菜单。")
                                    .child("工具栏支持方向键、Home / End 与选中态。")
                                    .child(
                                        div()
                                            .mt_2()
                                            .p_3()
                                            .rounded(px(6.))
                                            .bg(theme::fill_tertiary())
                                            .child(format!("最近动作：{}", self.status)),
                                    ),
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
        let bounds = Bounds::centered(None, size(px(820.), px(560.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("MenuBar / Toolbar".into()),
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
