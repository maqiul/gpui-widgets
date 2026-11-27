//! 树形表格示例：展开 / 选中 / 级联勾选、键盘导航与虚拟滚动。

use gpui::{
    App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::theme::{self, Theme};
use gpui_widgets::{Button, TreeTable, TreeTableNode, tree_table_toggle_expanded};

actions!(tree_table, [Quit]);

fn sample_nodes() -> Vec<TreeTableNode> {
    (0..12)
        .map(|group| {
            let children = (0..8).map(|row| {
                TreeTableNode::new(
                    format!("group-{group}-row-{row}"),
                    [
                        format!("节点 {group:02}-{row:02}"),
                        format!("{} MB", 12 + row * 7),
                        format!(
                            "{}",
                            if row % 3 == 0 {
                                "同步中"
                            } else {
                                "已就绪"
                            }
                        ),
                    ],
                )
            });
            TreeTableNode::new(
                format!("group-{group}"),
                [format!("分组 {group:02}"), "—".into(), "8 个节点".into()],
            )
            .children(children)
        })
        .collect()
}

struct Panel {
    nodes: Vec<TreeTableNode>,
    expanded: Vec<SharedString>,
    selected: Option<SharedString>,
    checked: Vec<SharedString>,
    status: String,
}

impl Panel {
    fn new(_: &mut Context<Self>) -> Self {
        let expanded = (0..12)
            .map(|group| SharedString::from(format!("group-{group}")))
            .collect();
        Self {
            nodes: sample_nodes(),
            expanded,
            selected: Some("group-0-row-0".into()),
            checked: Vec::new(),
            status: "就绪：点击行或箭头，使用 ↑ / ↓ / ← / → / Home / End".into(),
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let root = div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .bg(theme::bg_layout())
            .child(div().text_lg().text_color(theme::text()).child("树形表格"))
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child("108 个层级节点、固定行高虚拟滚动、稳定节点 ID 与父子级联勾选。"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .child(
                        Button::new("全部展开")
                            .id("expand-all")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.expanded = (0..12)
                                    .map(|group| SharedString::from(format!("group-{group}")))
                                    .collect();
                                this.status = "已展开全部分组".into();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("全部收起")
                            .id("collapse-all")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.expanded.clear();
                                this.status = "已收起全部分组".into();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("清空勾选")
                            .id("clear-checked")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.checked.clear();
                                this.status = "已清空勾选".into();
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
                TreeTable::new(
                    "demo-tree-table",
                    ["名称", "大小", "状态"],
                    self.nodes.clone(),
                )
                .expanded(self.expanded.clone())
                .selected(self.selected.clone())
                .checkable(true)
                .checked(self.checked.clone())
                .column_widths(vec![px(420.), px(140.), px(180.)])
                .striped(true)
                .bordered(true)
                .virtualized(px(390.))
                .on_toggle(cx.listener(|this, id: &SharedString, _, cx| {
                    this.expanded = tree_table_toggle_expanded(&this.expanded, id);
                    this.status = format!("切换展开：{id}");
                    cx.notify();
                }))
                .on_select(cx.listener(|this, id: &SharedString, _, cx| {
                    this.selected = Some(id.clone());
                    this.status = format!("已选中：{id}");
                    cx.notify();
                }))
                .on_check(cx.listener(
                    |this, checked: &Vec<SharedString>, _, cx| {
                        this.checked = checked.clone();
                        this.status = format!("已勾选 {} 个节点", checked.len());
                        cx.notify();
                    },
                )),
            );
        root
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());
        let bounds = Bounds::centered(None, size(px(980.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("TreeTable".into()),
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
