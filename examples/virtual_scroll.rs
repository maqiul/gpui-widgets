//! 虚拟滚动示例：万级 Table / List 只渲染视口内的行。

use gpui::{
    App, Bounds, Context, KeyBinding, Render, TitlebarOptions, Window, WindowBounds, WindowOptions,
    actions, div, prelude::*, px, size,
};
use gpui_widgets::{List, ListItem, Table, Theme, theme};

actions!(virtual_scroll, [Quit]);

const ROWS: usize = 10_000;

struct Panel {
    headers: Vec<&'static str>,
    table_rows: Vec<Vec<gpui::SharedString>>,
    table_selected: Option<usize>,
    table_selected_rows: Vec<usize>,
    list_selected: Option<usize>,
}

impl Panel {
    fn new() -> Self {
        Self {
            headers: vec!["#", "会话", "主机", "状态", "延迟"],
            table_rows: (0..ROWS)
                .map(|ix| {
                    vec![
                        format!("{}", ix + 1).into(),
                        format!("session-{ix:05}").into(),
                        format!("host-{:03}.internal", ix % 500).into(),
                        if ix % 7 == 0 { "断开" } else { "已连接" }.into(),
                        format!("{} ms", 12 + ix % 90).into(),
                    ]
                })
                .collect(),
            table_selected: Some(0),
            table_selected_rows: vec![0],
            list_selected: Some(0),
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let table = Table::new(
            "virtual-table",
            self.headers.clone(),
            self.table_rows.clone(),
        )
        .striped(true)
        .selectable(true)
        .range_select(true)
        .selected(self.table_selected)
        .selected_rows(self.table_selected_rows.clone())
        .on_selected_rows(cx.listener(|this, rows: &Vec<usize>, _, cx| {
            this.table_selected_rows = rows.clone();
            cx.notify();
        }))
        .on_row_click(cx.listener(|this, row: &usize, _, cx| {
            this.table_selected = Some(*row);
            cx.notify();
        }))
        .column_widths(vec![px(48.), px(150.), px(170.), px(76.), px(82.)])
        .virtualized(px(520.))
        .bordered();

        let list = List::virtualized(px(520.), px(56.), ROWS, |ix| {
            ListItem::new(format!("会话 {ix:05}"))
                .description(format!(
                    "host-{:03}.internal · {} ms",
                    ix % 500,
                    12 + ix % 90
                ))
                .avatar("●")
        })
        .id("virtual-list")
        .selected(self.list_selected)
        .on_select(cx.listener(|this, ix: &usize, _, cx| {
            this.list_selected = Some(*ix);
            cx.notify();
        }))
        .header("会话列表")
        .footer(format!(
            "共 {ROWS} 条，仅渲染视口内的行；当前选中 {:?}",
            self.list_selected
        ));

        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .bg(theme::bg_container())
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child(format!(
                        "虚拟滚动 · 表格 {ROWS} 行 + 列表 {ROWS} 行；点击或用 ↑ / ↓ / Home / End 导航，Shift / Ctrl + 方向键调整表格多选"
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .child(div().w(px(580.)).flex_shrink_0().child(table))
                    .child(div().w(px(420.)).flex_shrink_0().child(list)),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());

        let bounds = Bounds::centered(None, size(px(1060.0), px(700.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("virtual_scroll".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Panel::new()),
        )
        .unwrap();
        cx.activate(true);
    });
}
