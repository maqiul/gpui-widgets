//! 数据网格示例：列宽拖拽、筛选、固定左右列与单元格编辑。

use gpui::{
    App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::theme::{self, Theme};
use gpui_widgets::{Button, Table};

actions!(data_grid, [Quit]);

fn sample_rows() -> Vec<Vec<SharedString>> {
    (0..24)
        .map(|row| {
            vec![
                format!("任务 {row:02}").into(),
                format!("负责人 {row:02}").into(),
                format!("{}", 12 + row * 3).into(),
                if row % 3 == 0 {
                    "同步中"
                } else {
                    "已就绪"
                }
                .into(),
                format!("2026-11-{:02}", row % 27 + 1).into(),
            ]
        })
        .collect()
}

struct Panel {
    rows: Vec<Vec<SharedString>>,
    widths: Vec<gpui::Pixels>,
    filter: SharedString,
    editing: Option<(usize, usize)>,
    selected: Option<usize>,
    status: String,
}

impl Panel {
    fn new(_: &mut Context<Self>) -> Self {
        Self {
            rows: sample_rows(),
            widths: vec![px(190.), px(150.), px(110.), px(130.), px(170.)],
            filter: SharedString::default(),
            editing: None,
            selected: Some(0),
            status: "就绪：拖拽表头边缘调宽，双击单元格编辑，Enter 或失焦提交".into(),
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .bg(theme::bg_layout())
            .child(div().text_lg().text_color(theme::text()).child("DataGrid"))
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child("Table 的 P2 列能力：筛选、列宽拖拽、固定左/右列与受控单元格编辑。"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .child(
                        Button::new("重置列宽")
                            .id("reset-widths")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.widths =
                                    vec![px(190.), px(150.), px(110.), px(130.), px(170.)];
                                this.status = "已重置列宽".into();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("清空筛选")
                            .id("clear-filter")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.filter = SharedString::default();
                                this.status = "已清空筛选".into();
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
                Table::new(
                    "demo-data-grid",
                    ["任务", "负责人", "点数", "状态", "更新日期"],
                    self.rows.clone(),
                )
                .column_widths(self.widths.clone())
                .resizable_columns(true)
                .on_column_resize(cx.listener(|this, widths: &Vec<gpui::Pixels>, _, cx| {
                    this.widths = widths.clone();
                    this.status = format!("已调整列宽：{widths:?}");
                    cx.notify();
                }))
                .filterable(true)
                .filter(self.filter.clone())
                .on_filter_change(cx.listener(|this, filter: &str, _, cx| {
                    this.filter = filter.into();
                    let filtered = gpui_widgets::table_filtered_indices(&this.rows, filter).len();
                    this.status = format!("筛选：{filter}，命中 {filtered} 行");
                    cx.notify();
                }))
                .fixed_left()
                .fixed_right_columns(1)
                .striped(true)
                .bordered()
                .selected(self.selected)
                .editable_columns(vec![0, 1, 2, 4])
                .editing(self.editing)
                .on_row_click(cx.listener(|this, row: &usize, _, cx| {
                    this.selected = Some(*row);
                    cx.notify();
                }))
                .on_edit_start(cx.listener(|this, cell: &(usize, usize), _, cx| {
                    this.editing = Some(*cell);
                    this.status = format!("编辑单元格 {cell:?}");
                    cx.notify();
                }))
                .on_cell_change(cx.listener(
                    |this, change: &((usize, usize), SharedString), _, cx| {
                        let (cell, text) = change;
                        if let Some(value) = this
                            .rows
                            .get_mut(cell.0)
                            .and_then(|row| row.get_mut(cell.1))
                        {
                            *value = text.clone();
                        }
                        this.status = format!("已更新 {cell:?}：{text}");
                        cx.notify();
                    },
                ))
                .on_edit_end(cx.listener(|this, cell: &(usize, usize), _, cx| {
                    this.editing = None;
                    this.status = format!("已提交 {cell:?}");
                    cx.notify();
                })),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        gpui_widgets::text_input::bind_keys(cx);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());
        let bounds = Bounds::centered(None, size(px(1120.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("DataGrid".into()),
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
