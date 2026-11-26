//! 数据表格：表头 + 文本行，支持斑马纹 / 行选中 / 排序 / 多选 / 分页 / 展开行 / 固定左列 / 全网格边框。

use std::rc::Rc;

use gpui::{
    AnyElement, App, CursorStyle, Div, ElementId, Entity, IntoElement, Pixels, RenderOnce, Role,
    ScrollHandle, SharedString, Stateful, StatefulInteractiveElement, Toggled, Window, div,
    prelude::*, px,
};

use crate::{
    a11y::{self, Semantics},
    pagination::Pagination,
    scrollbar::Scrollbar,
    theme::{
        self,
        metric::{TableMetric, table_style},
    },
};

/// 行高 = 文字行高 + 上下单元格内边距
fn row_height() -> f32 {
    theme::with(|t| {
        let m = TableMetric::resolve(t);
        m.line_height + m.cell_padding_block * 2.
    })
}

fn cell_padding_block() -> f32 {
    theme::with(|t| TableMetric::resolve(t).cell_padding_block)
}

fn cell_padding_inline() -> f32 {
    theme::with(|t| TableMetric::resolve(t).cell_padding_inline)
}

fn selection_column_width() -> f32 {
    theme::with(|t| TableMetric::resolve(t).selection_column_width)
}

fn expand_column_width() -> f32 {
    theme::with(|t| TableMetric::resolve(t).expand_column_width)
}

fn default_col_width() -> f32 {
    theme::with(|t| TableMetric::resolve(t).default_col_width)
}

fn header_bg() -> gpui::Hsla {
    theme::with(|t| table_style(t).header_bg)
}

fn header_color() -> gpui::Hsla {
    theme::with(|t| table_style(t).header_color)
}

fn border_color() -> gpui::Hsla {
    theme::with(|t| table_style(t).border_color)
}

fn row_hover_bg() -> gpui::Hsla {
    theme::with(|t| table_style(t).row_hover_bg)
}

fn row_selected_bg() -> gpui::Hsla {
    theme::with(|t| table_style(t).row_selected_bg)
}

/// 数据表格（单元格为文本）。
///
/// ```ignore
/// Table::new("members", ["姓名", "城市", "技能"], rows)
///     .striped(true)
///     .selected(self.table_row)
///     .on_row_click(cx.listener(|this, ix, _, cx| {
///         this.table_row = Some(*ix);
///         cx.notify();
///     }))
/// ```
#[derive(IntoElement)]
pub struct Table {
    id: ElementId,
    headers: Vec<SharedString>,
    rows: Vec<Vec<SharedString>>,
    striped: bool,
    selected: Option<usize>,
    widths: Option<Vec<Pixels>>,
    sortable: Vec<usize>,
    sort: Option<(usize, bool)>,
    selectable: bool,
    selected_rows: Vec<usize>,
    page: Option<(usize, usize)>,
    expanded_rows: Vec<usize>,
    row_detail: Option<Rc<dyn Fn(usize) -> AnyElement>>,
    fixed_left: bool,
    bordered: bool,
    range_select: bool,
    show_header: bool,
    on_row_click: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_sort: Option<Rc<dyn Fn(&(usize, bool), &mut Window, &mut App)>>,
    on_selected_rows: Option<Rc<dyn Fn(&Vec<usize>, &mut Window, &mut App)>>,
    on_page_change: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_expand: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl Table {
    pub fn new(
        id: impl Into<ElementId>,
        headers: impl IntoIterator<Item = impl Into<SharedString>>,
        rows: Vec<Vec<SharedString>>,
    ) -> Self {
        Self {
            id: id.into(),
            headers: headers.into_iter().map(Into::into).collect(),
            rows,
            striped: false,
            selected: None,
            widths: None,
            sortable: Vec::new(),
            sort: None,
            selectable: false,
            selected_rows: Vec::new(),
            page: None,
            expanded_rows: Vec::new(),
            row_detail: None,
            fixed_left: false,
            bordered: false,
            range_select: false,
            show_header: true,
            on_row_click: None,
            on_sort: None,
            on_selected_rows: None,
            on_page_change: None,
            on_expand: None,
        }
    }

    /// 斑马纹（隔行浅灰底）
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    /// 当前选中行下标（受控）
    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_row_click(
        mut self,
        listener: impl Fn(&usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_row_click = Some(Rc::new(listener));
        self
    }

    /// 自定义列宽（按列下标，未覆盖的列均分剩余宽度）
    pub fn column_widths(mut self, widths: Vec<Pixels>) -> Self {
        self.widths = Some(widths);
        self
    }

    /// 可排序列下标集合（表头显示箭头，点击触发 on_sort）
    pub fn sortable(mut self, cols: Vec<usize>) -> Self {
        self.sortable = cols;
        self
    }

    /// 当前排序（列下标, 是否降序）受控；排序逻辑由调用方对 rows 自行处理
    pub fn sort(mut self, sort: Option<(usize, bool)>) -> Self {
        self.sort = sort;
        self
    }

    pub fn on_sort(
        mut self,
        listener: impl Fn(&(usize, bool), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_sort = Some(Rc::new(listener));
        self
    }

    /// 显示勾选列（多选受控）
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn selected_rows(mut self, rows: Vec<usize>) -> Self {
        self.selected_rows = rows;
        self
    }

    pub fn on_selected_rows(
        mut self,
        listener: impl Fn(&Vec<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selected_rows = Some(Rc::new(listener));
        self
    }

    /// 内置分页（页码从 1 起，受控）；行回调仍使用全局下标，全选只作用当前页
    pub fn pagination(mut self, page: usize, page_size: usize) -> Self {
        self.page = Some((page.max(1), page_size.max(1)));
        self
    }

    pub fn on_page_change(
        mut self,
        listener: impl Fn(&usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_page_change = Some(Rc::new(listener));
        self
    }

    /// 展开行内容生成器（按全局行下标），设置后首列前出现 ▸/▾ 展开按钮。
    /// 注意：与 `fixed_left()` 同时设置时展开行不生效（固定列要求统一行高）。
    pub fn row_detail(mut self, detail: impl Fn(usize) -> AnyElement + 'static) -> Self {
        self.row_detail = Some(Rc::new(detail));
        self
    }

    /// 当前展开的行下标集合（受控）
    pub fn expanded_rows(mut self, rows: Vec<usize>) -> Self {
        self.expanded_rows = rows;
        self
    }

    pub fn on_expand(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_expand = Some(Rc::new(listener));
        self
    }

    /// 固定首数据列：其余列横向滚动，行高统一为 37px，
    /// 未指定列宽的列默认 120px（建议配合 column_widths 使用）
    pub fn fixed_left(mut self) -> Self {
        self.fixed_left = true;
        self
    }

    /// 全网格边框：列与列之间显示竖向分隔线
    pub fn bordered(mut self) -> Self {
        self.bordered = true;
        self
    }

    /// 开启行点击区间 / 切换多选：普通点击选单行、Shift 点击选中连续区间、Ctrl 点击切换该行。
    /// 结果通过 [`Table::on_selected_rows`] 回传（闭区间按全局行下标计算，与分页页内顺序一致）。
    pub fn range_select(mut self, on: bool) -> Self {
        self.range_select = on;
        self
    }

    /// 是否显示表头（默认 true）
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }
}

impl RenderOnce for Table {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let cols = self.headers.len().max(1);
        let expandable = self.row_detail.is_some() && !self.fixed_left;
        let total = self.rows.len();

        // 分页切片：visible 保存全局行下标，回调统一用全局下标
        let visible: Vec<usize> = match self.page {
            Some((page, size)) => {
                let offset = page.saturating_sub(1) * size;
                (offset..total.min(offset + size)).collect()
            }
            None => (0..total).collect(),
        };

        let radius = theme::with(|t| TableMetric::resolve(t).radius);
        let column_count = cols + usize::from(self.selectable) + usize::from(expandable);
        let semantics = Semantics::new(Role::Table, "数据表格")
            .id(format!("{:?}-table-a11y", self.id))
            .value(SharedString::from(format!(
                "共 {total} 行，{column_count} 列"
            )));
        let mut root = semantics.apply(
            div()
                .id(ElementId::Name(format!("{:?}-table", self.id).into()))
                .flex()
                .flex_col()
                .w_full()
                .rounded(px(radius))
                .bg(theme::bg_container())
                .overflow_hidden()
                .text_sm(),
        );
        if self.bordered {
            root = root.border_1().border_color(border_color());
        }

        // 空态
        if total == 0 {
            return root.child(
                div()
                    .py_6()
                    .flex()
                    .justify_center()
                    .text_color(header_color())
                    .child("暂无数据"),
            );
        }

        let all_checked =
            !visible.is_empty() && visible.iter().all(|ix| self.selected_rows.contains(ix));
        let some_checked = visible.iter().any(|ix| self.selected_rows.contains(ix));

        // 区间 / 切换多选：锚点跨帧持久于窗口，供 Shift 点击计算连续区间
        let anchor = self.range_select.then(|| {
            window.use_keyed_state(
                ElementId::Name(format!("{:?}-selanchor", self.id).into()),
                cx,
                |_, _| None::<usize>,
            )
        });
        if self.fixed_left {
            root = root.child(self.fixed_body(
                &visible,
                cols,
                all_checked,
                some_checked,
                anchor.as_ref(),
                window,
                cx,
            ));
        } else {
            root = self.normal_body(
                root,
                &visible,
                cols,
                all_checked,
                some_checked,
                expandable,
                anchor.as_ref(),
            );
        }

        // 分页栏
        if let Some((page, size)) = self.page {
            let pages = ((total + size - 1) / size).max(1);
            let mut pag = Pagination::new(
                ElementId::Name(format!("{:?}-page", self.id).into()),
                page,
                pages,
            );
            if let Some(on_page_change) = self.on_page_change.clone() {
                pag = pag.on_change(move |p, window, cx| on_page_change(p, window, cx));
            }
            root = root.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px(px(cell_padding_inline()))
                    .py(px(cell_padding_block()))
                    .border_t_1()
                    .border_color(border_color())
                    .child(
                        div()
                            .text_xs()
                            .text_color(header_color())
                            .child(format!("共 {total} 条")),
                    )
                    .child(pag),
            );
        }

        root
    }
}

impl Table {
    /// 列宽容器：普通模式指定宽或 flex_1；固定模式指定宽或默认宽
    fn cell_w(&self, col: usize, fixed: bool) -> Div {
        let w = self.widths.as_ref().and_then(|w| w.get(col)).copied();
        match (fixed, w) {
            (true, Some(w)) => div().w(w),
            (true, None) => div().w(px(default_col_width())),
            (false, Some(w)) => div().w(w),
            (false, None) => div().flex_1(),
        }
    }

    /// 表头单元格（可排序列带箭头与点击回调）
    fn header_cell(
        &self,
        col: usize,
        fixed: bool,
        column_index: usize,
        column_count: usize,
    ) -> Stateful<Div> {
        let mut cell = self
            .cell_w(col, fixed)
            .id(ElementId::Name(
                format!("{:?}-header-{col}", self.id).into(),
            ))
            .px(px(cell_padding_inline()))
            .overflow_hidden()
            .whitespace_nowrap()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(header_color());
        cell = if fixed {
            cell.h(px(row_height())).flex().items_center()
        } else {
            cell.py(px(cell_padding_block()))
        };
        let title = self.headers.get(col).cloned().unwrap_or_default();
        if self.sortable.contains(&col) {
            let arrow = match self.sort {
                Some((c, desc)) if c == col => {
                    if desc {
                        "↓"
                    } else {
                        "↑"
                    }
                }
                _ => "↕",
            };
            let on_sort = self.on_sort.clone();
            let current = self.sort;
            cell = cell
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .cursor(CursorStyle::PointingHand)
                .child(title.clone())
                .child(div().text_xs().child(arrow));
            if let Some(on_sort) = on_sort {
                cell = a11y::click_void(cell, move |window, cx| {
                    let next = next_sort(current, col);
                    on_sort(&next, window, cx);
                });
            }
        } else {
            cell = cell.child(title.clone());
        }
        let semantics = Semantics::new(Role::ColumnHeader, title)
            .id(format!("{:?}-header-{col}-a11y", self.id))
            .row(0, self.rows.len() + usize::from(self.show_header))
            .column(column_index, column_count)
            .focusable(self.sortable.contains(&col) && self.on_sort.is_some());
        let mut cell = semantics.apply(cell);
        if col > 0 && !(fixed && col == 1) {
            cell = cell.border_l_1().border_color(border_color());
        }
        cell
    }

    /// 表头全选单元格（只作用当前页可见行）
    fn check_all_cell(
        &self,
        all: bool,
        some: bool,
        visible: &[usize],
        fixed: bool,
        column_count: usize,
    ) -> Stateful<Div> {
        let on_selected_rows = self.on_selected_rows.clone();
        let mut next = self.selected_rows.clone();
        if all {
            next.retain(|i| !visible.contains(i));
        } else {
            for ix in visible {
                if !next.contains(ix) {
                    next.push(*ix);
                }
            }
            next.sort_unstable();
        }
        let mut cell = div()
            .id(ElementId::Name(
                format!("{:?}-check-all-cell", self.id).into(),
            ))
            .w(px(selection_column_width()))
            .flex()
            .items_center()
            .justify_center()
            .child(check_box(
                &format!("{:?}-check-all", self.id),
                all,
                some && !all,
            ))
            .cursor(CursorStyle::PointingHand);
        if let Some(on_selected_rows) = on_selected_rows {
            cell = a11y::click_void(cell, move |window, cx| {
                on_selected_rows(&next, window, cx);
            });
        }
        if fixed {
            cell = cell.h(px(row_height()));
        } else {
            cell = cell.py(px(cell_padding_block()));
        }
        if self.bordered {
            cell = cell.border_r_1().border_color(border_color());
        }
        let semantics = Semantics::new(Role::CheckBox, "全选")
            .id(format!("{:?}-check-all-a11y", self.id))
            .toggled(if all {
                Toggled::True
            } else if some {
                Toggled::Mixed
            } else {
                Toggled::False
            })
            .row(0, self.rows.len() + usize::from(self.show_header))
            .column(0, column_count)
            .focusable(self.on_selected_rows.is_some());
        semantics.apply(cell)
    }

    /// 行勾选单元格
    fn row_check_cell(&self, ix: usize, fixed: bool, column_count: usize) -> Stateful<Div> {
        let checked = self.selected_rows.contains(&ix);
        let on_selected_rows = self.on_selected_rows.clone();
        let mut next = self.selected_rows.clone();
        if checked {
            next.retain(|i| *i != ix);
        } else {
            next.push(ix);
            next.sort_unstable();
        }
        let mut cell = div()
            .id(ElementId::Name(
                format!("{:?}-check-{ix}-cell", self.id).into(),
            ))
            .w(px(selection_column_width()))
            .flex()
            .items_center()
            .justify_center()
            .child(check_box(
                &format!("{:?}-check-{ix}", self.id),
                checked,
                false,
            ))
            .cursor(CursorStyle::PointingHand);
        if let Some(on_selected_rows) = on_selected_rows {
            cell = a11y::click(cell, move |_, window, cx| {
                cx.stop_propagation();
                on_selected_rows(&next, window, cx);
            });
        }
        if fixed {
            cell = cell.h(px(row_height()));
        }
        if self.bordered {
            cell = cell.border_r_1().border_color(border_color());
        }
        let semantics = Semantics::new(
            Role::CheckBox,
            SharedString::from(format!("选择第 {} 行", ix + 1)),
        )
        .id(format!("{:?}-check-{ix}-a11y", self.id))
        .toggled(if checked {
            Toggled::True
        } else {
            Toggled::False
        })
        .row(ix + 1, self.rows.len() + usize::from(self.show_header))
        .column(0, column_count)
        .focusable(self.on_selected_rows.is_some());
        semantics.apply(cell)
    }

    /// 展开按钮单元格
    fn expand_cell(&self, ix: usize, column_index: usize, column_count: usize) -> Stateful<Div> {
        let expanded = self.expanded_rows.contains(&ix);
        let on_expand = self.on_expand.clone();
        let mut cell = div()
            .id(ElementId::Name(format!("{:?}-expand-{ix}", self.id).into()))
            .w(px(expand_column_width()))
            .flex()
            .items_center()
            .justify_center()
            .text_color(header_color())
            .cursor(CursorStyle::PointingHand)
            .child(if expanded { "▾" } else { "▸" });
        if let Some(on_expand) = on_expand {
            cell = a11y::click(cell, move |_, window, cx| {
                cx.stop_propagation();
                on_expand(&ix, window, cx);
            });
        }
        if self.bordered {
            cell = cell.border_r_1().border_color(border_color());
        }
        let semantics = Semantics::new(
            Role::Button,
            SharedString::from(format!(
                "{}第 {} 行",
                if expanded { "收起" } else { "展开" },
                ix + 1
            )),
        )
        .id(format!("{:?}-expand-{ix}-a11y", self.id))
        .expanded(expanded)
        .row(ix + 1, self.rows.len() + usize::from(self.show_header))
        .column(column_index, column_count)
        .focusable(self.on_expand.is_some());
        semantics.apply(cell)
    }

    /// 数据单元格（不足列数补空）
    fn data_cell(
        &self,
        row: &[SharedString],
        ix: usize,
        col: usize,
        fixed: bool,
        column_index: usize,
        column_count: usize,
    ) -> Stateful<Div> {
        let mut cell = self
            .cell_w(col, fixed)
            .px(px(cell_padding_inline()))
            .overflow_hidden()
            .whitespace_nowrap();
        cell = if fixed {
            cell.h(px(row_height())).flex().items_center()
        } else {
            cell.py(px(cell_padding_block()))
        };
        if self.bordered && col > 0 && !(fixed && col == 1) {
            cell = cell.border_l_1().border_color(border_color());
        }
        let text = row.get(col).cloned().unwrap_or_default();
        let semantics = Semantics::new(Role::Cell, text.clone())
            .id(format!("{:?}-cell-{ix}-{col}-a11y", self.id))
            .row(ix + 1, self.rows.len() + usize::from(self.show_header))
            .column(column_index, column_count);
        semantics.apply(
            cell.id(ElementId::Name(
                format!("{:?}-cell-{ix}-{col}", self.id).into(),
            ))
            .child(text),
        )
    }

    /// 行背景：选中高亮优先，其次斑马纹
    fn row_bg(&self, row: Stateful<Div>, local_ix: usize, global_ix: usize) -> Stateful<Div> {
        let checked = self.selected_rows.contains(&global_ix);
        if self.selected == Some(global_ix) || checked {
            row.bg(row_selected_bg())
        } else if self.striped && local_ix % 2 == 1 {
            row.bg(header_bg())
        } else {
            row
        }
    }

    /// 行点击回调：开启 range_select 时驱动区间 / 切换多选，否则回退到 on_row_click。
    fn with_row_click(
        &self,
        row: Stateful<Div>,
        ix: usize,
        side: &str,
        anchor: Option<&Entity<Option<usize>>>,
    ) -> Stateful<Div> {
        let checked = self.selected_rows.contains(&ix);
        let label = self
            .rows
            .get(ix)
            .map(|cells| cells.join("，"))
            .unwrap_or_else(|| format!("第 {} 行", ix + 1));
        let semantics = Semantics::new(Role::Row, label)
            .id(format!("{:?}-{side}-{ix}-a11y", self.id))
            .row(ix + 1, self.rows.len() + usize::from(self.show_header))
            .selected(self.selected == Some(ix) || checked)
            .focusable(
                self.on_row_click.is_some()
                    || (self.range_select && self.on_selected_rows.is_some()),
            );
        let mut row = row;
        if let (Some(anchor), Some(on_selected_rows)) = (anchor, self.on_selected_rows.clone()) {
            let anchor = anchor.clone();
            let current = self.selected_rows.clone();
            row = row.cursor(CursorStyle::PointingHand).hover(move |style| {
                // 已选中行保持选中底色，避免 hover 灰底遮住高亮
                if checked {
                    style
                } else {
                    style.bg(row_hover_bg())
                }
            });
            row = a11y::click(row, move |event, window, cx| {
                let modifiers = event.modifiers();
                let prev = *anchor.read(cx);
                let next = range_selection(
                    &current,
                    ix,
                    prev,
                    modifiers.shift,
                    modifiers.control || modifiers.platform,
                );
                anchor.update(cx, |slot, _| *slot = Some(ix));
                on_selected_rows(&next, window, cx);
            });
        }
        if let Some(on_row_click) = self.on_row_click.clone() {
            row = row
                .cursor(CursorStyle::PointingHand)
                .hover(|style| style.bg(row_hover_bg()));
            row = a11y::click_void(row, move |window, cx| {
                on_row_click(&ix, window, cx);
            });
        }
        semantics.apply(row)
    }

    /// 普通模式：单栏行堆叠 + 展开详情行
    fn normal_body(
        &self,
        mut root: Stateful<Div>,
        visible: &[usize],
        cols: usize,
        all_checked: bool,
        some_checked: bool,
        expandable: bool,
        anchor: Option<&Entity<Option<usize>>>,
    ) -> Stateful<Div> {
        let column_count = cols + usize::from(self.selectable) + usize::from(expandable);
        let column_offset = usize::from(self.selectable) + usize::from(expandable);
        if self.show_header {
            let mut header = div()
                .flex()
                .flex_row()
                .bg(header_bg())
                .border_b_1()
                .border_color(border_color());
            if self.selectable {
                header = header.child(self.check_all_cell(
                    all_checked,
                    some_checked,
                    visible,
                    false,
                    column_count,
                ));
            }
            if expandable {
                let mut spacer = div().w(px(expand_column_width()));
                if self.bordered {
                    spacer = spacer.border_r_1().border_color(border_color());
                }
                header = header.child(spacer);
            }
            for col in 0..cols {
                header =
                    header.child(self.header_cell(col, false, column_offset + col, column_count));
            }
            let semantics = Semantics::new(Role::Row, "表头")
                .id(format!("{:?}-header-row-a11y", self.id))
                .row(0, self.rows.len() + 1);
            root = root.child(
                semantics
                    .apply(header.id(ElementId::Name(format!("{:?}-header-row", self.id).into()))),
            );
        }

        for (local, &ix) in visible.iter().enumerate() {
            let row = &self.rows[ix];
            let mut row_div = div()
                .id(ElementId::Name(format!("{:?}-row-{ix}", self.id).into()))
                .flex()
                .flex_row()
                .border_b_1()
                .border_color(border_color());
            row_div = self.row_bg(row_div, local, ix);
            if self.selectable {
                row_div = row_div.child(self.row_check_cell(ix, false, column_count));
            }
            if expandable {
                row_div =
                    row_div.child(self.expand_cell(ix, usize::from(self.selectable), column_count));
            }
            for col in 0..cols {
                row_div = row_div.child(self.data_cell(
                    row,
                    ix,
                    col,
                    false,
                    column_offset + col,
                    column_count,
                ));
            }
            row_div = self.with_row_click(row_div, ix, "row", anchor);
            root = root.child(row_div);

            // 展开详情行
            if expandable && self.expanded_rows.contains(&ix) {
                if let Some(detail) = &self.row_detail {
                    root = root.child(
                        Semantics::new(
                            Role::Cell,
                            SharedString::from(format!("第 {} 行详情", ix + 1)),
                        )
                        .id(format!("{:?}-detail-{ix}-a11y", self.id))
                        .row(ix + 1, self.rows.len() + 1)
                        .column(column_offset, column_count)
                        .apply(
                            div()
                                .id(ElementId::Name(format!("{:?}-detail-{ix}", self.id).into()))
                                .border_t_1()
                                .border_color(border_color())
                                .bg(header_bg())
                                .px_4()
                                .py_3()
                                .child(detail(ix)),
                        ),
                    );
                }
            }
        }
        root
    }

    /// 固定左列模式：左栏（勾选列 + 首列）固定，右栏其余列横向滚动，
    /// 统一行高保证两栏逐行对齐
    fn fixed_body(
        &self,
        visible: &[usize],
        cols: usize,
        all_checked: bool,
        some_checked: bool,
        anchor: Option<&Entity<Option<usize>>>,
        window: &mut Window,
        cx: &mut App,
    ) -> Div {
        let column_count = cols + usize::from(self.selectable);
        let column_offset = usize::from(self.selectable);
        let w0 = self
            .widths
            .as_ref()
            .and_then(|w| w.first())
            .copied()
            .unwrap_or(px(default_col_width()));
        let left_w = w0
            + if self.selectable {
                px(selection_column_width())
            } else {
                px(0.)
            };
        let right_w: Pixels = (1..cols)
            .map(|c| {
                self.widths
                    .as_ref()
                    .and_then(|w| w.get(c))
                    .copied()
                    .unwrap_or(px(default_col_width()))
            })
            .fold(px(0.), |a, b| a + b);

        // 左固定栏
        let mut left = div()
            .flex()
            .flex_col()
            .w(left_w)
            .border_r_1()
            .border_color(border_color());
        if self.show_header {
            let mut left_header = div()
                .flex()
                .flex_row()
                .h(px(row_height()))
                .bg(header_bg())
                .border_b_1()
                .border_color(border_color());
            if self.selectable {
                left_header = left_header.child(self.check_all_cell(
                    all_checked,
                    some_checked,
                    visible,
                    true,
                    column_count,
                ));
            }
            left_header = left_header.child(self.header_cell(0, true, column_offset, column_count));
            let semantics = Semantics::new(Role::Row, "固定列表头")
                .id(format!("{:?}-header-left-a11y", self.id))
                .row(0, self.rows.len() + 1);
            left = left.child(semantics.apply(
                left_header.id(ElementId::Name(format!("{:?}-header-left", self.id).into())),
            ));
        }

        // 右滚动栏
        let mut right = div().flex().flex_col().min_w(right_w);
        if self.show_header {
            let mut right_header = div()
                .flex()
                .flex_row()
                .h(px(row_height()))
                .bg(header_bg())
                .border_b_1()
                .border_color(border_color());
            for col in 1..cols {
                right_header = right_header.child(self.header_cell(
                    col,
                    true,
                    column_offset + col,
                    column_count,
                ));
            }
            let semantics = Semantics::new(Role::Row, "滚动列表头")
                .id(format!("{:?}-header-right-a11y", self.id))
                .row(0, self.rows.len() + 1);
            right = right.child(semantics.apply(right_header.id(ElementId::Name(
                format!("{:?}-header-right", self.id).into(),
            ))));
        }

        for (local, &ix) in visible.iter().enumerate() {
            let row = &self.rows[ix];
            let mut left_row = div()
                .id(ElementId::Name(format!("{:?}-row-{ix}", self.id).into()))
                .flex()
                .flex_row()
                .h(px(row_height()));
            let mut right_row = div()
                .id(ElementId::Name(format!("{:?}-rowR-{ix}", self.id).into()))
                .flex()
                .flex_row()
                .h(px(row_height()));
            left_row = left_row.border_b_1().border_color(border_color());
            right_row = right_row.border_b_1().border_color(border_color());
            left_row = self.row_bg(left_row, local, ix);
            right_row = self.row_bg(right_row, local, ix);
            if self.selectable {
                left_row = left_row.child(self.row_check_cell(ix, true, column_count));
            }
            left_row =
                left_row.child(self.data_cell(row, ix, 0, true, column_offset, column_count));
            for col in 1..cols {
                right_row = right_row.child(self.data_cell(
                    row,
                    ix,
                    col,
                    true,
                    column_offset + col,
                    column_count,
                ));
            }
            left_row = self.with_row_click(left_row, ix, "rowL", anchor);
            right_row = self.with_row_click(right_row, ix, "rowR", anchor);
            left = left.child(left_row);
            right = right.child(right_row);
        }

        // 横向滚动句柄：use_keyed_state 跨帧持久，与右栏滚动容器共享 offset，
        // 滚动条拖拽 / 轨道点击 / 滚轮都通过它驱动内容滚动
        let handle = window
            .use_keyed_state(
                ElementId::Name(format!("{:?}-hscroll", self.id).into()),
                cx,
                |_, _| ScrollHandle::new(),
            )
            .read(cx)
            .clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                div().flex().flex_row().w_full().child(left).child(
                    div()
                        .id(ElementId::Name(format!("{:?}-scroll", self.id).into()))
                        .flex_1()
                        .overflow_x_scroll()
                        .track_scroll(&handle)
                        .child(right),
                ),
            )
            // 主题化横向滚动条：对齐右栏下方，左侧留出固定列宽的占位
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .child(div().w(left_w))
                    .child(div().flex_1().child(Scrollbar::horizontal(
                        ElementId::Name(format!("{:?}-hsb", self.id).into()),
                        handle,
                    ))),
            )
    }
}

fn next_sort(current: Option<(usize, bool)>, col: usize) -> (usize, bool) {
    match current {
        Some((c, desc)) if c == col => (col, !desc),
        _ => (col, false),
    }
}

fn range_selection(
    current: &[usize],
    ix: usize,
    anchor: Option<usize>,
    shift: bool,
    ctrl: bool,
) -> Vec<usize> {
    let mut next = current.to_vec();
    if shift {
        let start = anchor.unwrap_or(ix);
        let (lo, hi) = (start.min(ix), start.max(ix));
        for row in lo..=hi {
            if !next.contains(&row) {
                next.push(row);
            }
        }
        next.sort_unstable();
    } else if ctrl {
        if let Some(pos) = next.iter().position(|&row| row == ix) {
            next.remove(pos);
        } else {
            next.push(ix);
            next.sort_unstable();
        }
    } else {
        next = vec![ix];
    }
    next
}

/// 表格勾选列的小方框（checked 全选中 / half 半选横杠）
fn check_box(id: &str, checked: bool, half: bool) -> Stateful<Div> {
    let radius = theme::with(|t| t.border_radius_sm);
    let mut box_el = div()
        .id(ElementId::Name(id.to_string().into()))
        .size(px(theme::with(|t| t.control_interactive_size)))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius))
        .border_1();
    if checked || half {
        box_el = box_el
            .border_color(theme::primary())
            .bg(theme::primary())
            .text_color(theme::with(|t| t.color_text_light_solid))
            .text_xs()
            .child(if checked { "✓" } else { "−" });
    } else {
        box_el = box_el
            .border_color(theme::border())
            .bg(theme::bg_container());
    }
    box_el
}

#[cfg(test)]
mod tests {
    use super::{next_sort, range_selection};

    #[test]
    fn sort_cycles_ascending_then_descending() {
        assert_eq!(next_sort(None, 2), (2, false));
        assert_eq!(next_sort(Some((2, false)), 2), (2, true));
        assert_eq!(next_sort(Some((2, true)), 2), (2, false));
        assert_eq!(next_sort(Some((1, true)), 2), (2, false));
    }

    #[test]
    fn row_selection_preserves_shift_and_toggle_behavior() {
        assert_eq!(range_selection(&[0, 2], 4, None, false, false), vec![4]);
        assert_eq!(
            range_selection(&[0, 2], 4, Some(2), true, false),
            vec![0, 2, 3, 4]
        );
        assert_eq!(range_selection(&[0, 2], 2, Some(0), false, true), vec![0]);
        assert_eq!(
            range_selection(&[0, 2], 1, Some(0), false, true),
            vec![0, 1, 2]
        );
    }
}
