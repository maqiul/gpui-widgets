//! 数据表格：表头 + 文本行，支持斑马纹 / 行选中 / 排序 / 多选 / 分页 / 展开行 / 固定左列 / 全网格边框。

use std::{collections::HashMap, rc::Rc};

use gpui::{
    AnyElement, App, CursorStyle, Div, ElementId, Entity, FocusHandle, Focusable, IntoElement,
    MouseButton, Pixels, RenderOnce, Role, ScrollHandle, SharedString, Stateful,
    StatefulInteractiveElement, Toggled, UniformListScrollHandle, Window, div, prelude::*, px,
    uniform_list,
};

use crate::{
    a11y::{self, Semantics},
    input::text_input::TextInput,
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

/// 按不区分大小写的子串筛选行，返回原始行下标。
pub fn table_filtered_indices(rows: &[Vec<SharedString>], query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return (0..rows.len()).collect();
    }
    rows.iter()
        .enumerate()
        .filter(|(_, row)| row.iter().any(|cell| cell.to_lowercase().contains(&query)))
        .map(|(index, _)| index)
        .collect()
}

/// 按像素增量调整一列宽度，并限制到最小宽度。
pub fn table_resize_column(
    widths: &[Pixels],
    column: usize,
    delta: f32,
    min_width: f32,
) -> Vec<Pixels> {
    let mut next = widths.to_vec();
    if let Some(width) = next.get_mut(column) {
        *width = px((f32::from(*width) + delta).max(min_width));
    }
    next
}

/// 返回固定布局的左列数、中间滚动列数、右列数。
pub fn table_fixed_column_ranges(
    columns: usize,
    fixed_left: bool,
    fixed_right: usize,
) -> (usize, usize, usize) {
    let left = usize::from(fixed_left).min(columns);
    let right = fixed_right.min(columns.saturating_sub(left));
    (left, columns - left - right, right)
}

#[derive(Clone, Copy, Debug)]
struct ColumnDrag {
    column: usize,
    start_x: f32,
    start_width: f32,
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
    fixed_right: usize,
    bordered: bool,
    range_select: bool,
    show_header: bool,
    filter: SharedString,
    filterable: bool,
    resizable_columns: bool,
    editable_columns: Vec<usize>,
    editing: Option<(usize, usize)>,
    /// 虚拟滚动视口高度；`None` 走全量渲染
    virtualized: Option<Pixels>,
    /// 虚拟滚动的跨帧滚动句柄（不传时按 id 存进 window 状态）
    virtual_scroll: Option<UniformListScrollHandle>,
    on_row_click: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_sort: Option<Rc<dyn Fn(&(usize, bool), &mut Window, &mut App)>>,
    on_selected_rows: Option<Rc<dyn Fn(&Vec<usize>, &mut Window, &mut App)>>,
    on_page_change: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_expand: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_filter_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    on_column_resize: Option<Rc<dyn Fn(&Vec<Pixels>, &mut Window, &mut App)>>,
    on_edit_start: Option<Rc<dyn Fn(&(usize, usize), &mut Window, &mut App)>>,
    on_cell_change: Option<Rc<dyn Fn(&((usize, usize), SharedString), &mut Window, &mut App)>>,
    on_edit_end: Option<Rc<dyn Fn(&(usize, usize), &mut Window, &mut App)>>,
    resolved_widths: Vec<Pixels>,
    resize_drag: Option<Entity<Option<ColumnDrag>>>,
    filter_input: Option<Entity<TextInput>>,
    edit_input: Option<Entity<TextInput>>,
    editable_focus: HashMap<(usize, usize), FocusHandle>,
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
            fixed_right: 0,
            bordered: false,
            range_select: false,
            show_header: true,
            filter: SharedString::default(),
            filterable: false,
            resizable_columns: false,
            editable_columns: Vec::new(),
            editing: None,
            virtualized: None,
            virtual_scroll: None,
            on_row_click: None,
            on_sort: None,
            on_selected_rows: None,
            on_page_change: None,
            on_expand: None,
            on_filter_change: None,
            on_column_resize: None,
            on_edit_start: None,
            on_cell_change: None,
            on_edit_end: None,
            resolved_widths: Vec::new(),
            resize_drag: None,
            filter_input: None,
            edit_input: None,
            editable_focus: HashMap::new(),
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

    /// 固定末尾若干数据列；可与 `fixed_left()` 组合，中间列横向滚动。
    pub fn fixed_right_columns(mut self, columns: usize) -> Self {
        self.fixed_right = columns;
        self
    }

    /// 开启表头列宽拖拽；拖拽结果通过 `on_column_resize` 回传。
    pub fn resizable_columns(mut self, resizable: bool) -> Self {
        self.resizable_columns = resizable;
        self
    }

    pub fn on_column_resize(
        mut self,
        listener: impl Fn(&Vec<Pixels>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_column_resize = Some(Rc::new(listener));
        self
    }

    /// 开启筛选栏；`filter` 为受控查询词，匹配所有单元格文本。
    pub fn filterable(mut self, filterable: bool) -> Self {
        self.filterable = filterable;
        self
    }

    pub fn filter(mut self, filter: impl Into<SharedString>) -> Self {
        self.filter = filter.into();
        self
    }

    pub fn on_filter_change(
        mut self,
        listener: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_filter_change = Some(Rc::new(listener));
        self
    }

    /// 可编辑列下标集合；双击单元格（或键盘激活）进入编辑。
    pub fn editable_columns(mut self, columns: Vec<usize>) -> Self {
        self.editable_columns = columns;
        self
    }

    /// 当前编辑单元格（全局行下标, 列下标），受控。
    pub fn editing(mut self, cell: Option<(usize, usize)>) -> Self {
        self.editing = cell;
        self
    }

    pub fn on_edit_start(
        mut self,
        listener: impl Fn(&(usize, usize), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_edit_start = Some(Rc::new(listener));
        self
    }

    pub fn on_cell_change(
        mut self,
        listener: impl Fn(&((usize, usize), SharedString), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_cell_change = Some(Rc::new(listener));
        self
    }

    pub fn on_edit_end(
        mut self,
        listener: impl Fn(&(usize, usize), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_edit_end = Some(Rc::new(listener));
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

    /// 虚拟滚动：只渲染视口内（含少量预渲染）的行，适合万级数据。
    /// `height` 是表体视口高度（不含表头/分页栏）。
    ///
    /// 行高统一为单元格单行高度，因此与 `row_detail()` / `fixed_left()` 不兼容；
    /// 这两个选项同时存在时本方法不生效，仍走全量渲染。
    pub fn virtualized(mut self, height: impl Into<Pixels>) -> Self {
        self.virtualized = Some(height.into());
        self
    }

    /// 复用外部滚动句柄（可选）：可据此做回到顶部、滚到某行等控制。
    pub fn virtual_scroll_handle(mut self, handle: UniformListScrollHandle) -> Self {
        self.virtual_scroll = Some(handle);
        self
    }
}

impl RenderOnce for Table {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut this = self;
        let cols = this.headers.len().max(1);
        let fixed_layout = this.fixed_left || this.fixed_right > 0;
        let expandable = this.row_detail.is_some() && !fixed_layout;
        let table_id = format!("{:?}", this.id);

        if this.resizable_columns || fixed_layout {
            this.resolved_widths = (0..cols)
                .map(|col| {
                    this.widths
                        .as_ref()
                        .and_then(|widths| widths.get(col))
                        .copied()
                        .unwrap_or(px(default_col_width()))
                })
                .collect();
        }

        let resize_drag = if this.resizable_columns && this.on_column_resize.is_some() {
            Some(window.use_keyed_state(
                ElementId::Name(format!("{table_id}-column-drag").into()),
                cx,
                |_, _| None::<ColumnDrag>,
            ))
        } else {
            None
        };
        this.resize_drag = resize_drag;

        let filter_input = if this.filterable {
            let on_filter_change = this.on_filter_change.clone();
            let initial = this.filter.clone();
            let filter_a11y_id = format!("{table_id}-filter-a11y");
            let input = window
                .use_keyed_state(
                    ElementId::Name(format!("{table_id}-filter-input").into()),
                    cx,
                    move |_, cx| {
                        cx.new(move |cx| {
                            let mut input = TextInput::new(cx).bare(true).placeholder("筛选表格");
                            input.set_a11y_semantics(
                                Semantics::new(Role::TextInput, "筛选表格").id(filter_a11y_id),
                            );
                            if let Some(on_filter_change) = on_filter_change.clone() {
                                input = input.on_change(move |text, window, cx| {
                                    on_filter_change(text, window, cx)
                                });
                            }
                            input.set_text_silent(initial.clone(), cx);
                            input
                        })
                    },
                )
                .read(cx)
                .clone();
            if input.read(cx).text() != this.filter.as_str() {
                let filter = this.filter.clone();
                input.update(cx, |input, cx| input.set_text_silent(filter, cx));
            }
            Some(input)
        } else {
            None
        };
        this.filter_input = filter_input;

        let filtered = table_filtered_indices(&this.rows, &this.filter);
        let total = filtered.len();

        if let Some((row, col)) = this.editing
            && this.editable_columns.contains(&col)
            && row < this.rows.len()
        {
            let text = this.rows[row].get(col).cloned().unwrap_or_default();
            let on_cell_change = this.on_cell_change.clone();
            let on_edit_end = this.on_edit_end.clone();
            let edit_label = this
                .headers
                .get(col)
                .cloned()
                .unwrap_or_else(|| SharedString::from("单元格"));
            let edit_a11y_id = format!("{table_id}-edit-{row}-{col}-a11y");
            let input = window
                .use_keyed_state(
                    ElementId::Name(format!("{table_id}-edit-{row}-{col}").into()),
                    cx,
                    move |_, cx| {
                        cx.new(move |cx| {
                            let mut input = TextInput::new(cx).bare(true);
                            input.set_a11y_semantics(
                                Semantics::new(Role::TextInput, format!("编辑 {edit_label}"))
                                    .id(edit_a11y_id),
                            );
                            if let Some(on_cell_change) = on_cell_change.clone() {
                                input = input.on_change(move |text, window, cx| {
                                    on_cell_change(
                                        &((row, col), SharedString::from(text)),
                                        window,
                                        cx,
                                    );
                                });
                            }
                            if let Some(on_edit_end) = on_edit_end.clone() {
                                input = input.on_blur(move |_, window, cx| {
                                    on_edit_end(&(row, col), window, cx)
                                });
                            }
                            input.set_text_silent(text, cx);
                            input
                        })
                    },
                )
                .read(cx)
                .clone();
            Focusable::focus_handle(input.read(cx), cx).focus(window, cx);
            this.edit_input = Some(input);
        }

        // 分页切片：visible 保存全局行下标，回调统一用全局下标
        let visible: Vec<usize> = match this.page {
            Some((page, size)) => {
                let offset = page.saturating_sub(1) * size;
                filtered.iter().skip(offset).take(size).copied().collect()
            }
            None => filtered,
        };

        this.editable_focus.clear();
        if this.on_edit_start.is_some() && !this.editable_columns.is_empty() {
            this.editable_focus
                .reserve(visible.len().saturating_mul(this.editable_columns.len()));
            let editable_columns = this.editable_columns.clone();
            this.editable_focus.extend(
                visible
                    .iter()
                    .flat_map(|row| editable_columns.iter().map(move |col| (*row, *col)))
                    .filter(|(_, col)| *col < cols)
                    .map(|(row, col)| {
                        let focus = window
                            .use_keyed_state(
                                ElementId::Name(
                                    format!("{table_id}-cell-focus-{row}-{col}").into(),
                                ),
                                cx,
                                |_, cx| cx.focus_handle(),
                            )
                            .read(cx)
                            .clone();
                        ((row, col), focus)
                    }),
            );
        }

        let radius = theme::with(|t| TableMetric::resolve(t).radius);
        let column_count = cols + usize::from(this.selectable) + usize::from(expandable);
        let semantics = Semantics::new(Role::Table, "数据表格")
            .id(format!("{:?}-table-a11y", this.id))
            .value(SharedString::from(format!(
                "共 {total} 行，{column_count} 列"
            )));
        let mut root = semantics.apply(
            div()
                .id(ElementId::Name(format!("{:?}-table", this.id).into()))
                .flex()
                .flex_col()
                .w_full()
                .rounded(px(radius))
                .bg(theme::bg_container())
                .overflow_hidden()
                .text_sm(),
        );
        if this.bordered {
            root = root.border_1().border_color(border_color());
        }

        if this.filterable {
            let mut bar = div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px(px(cell_padding_inline()))
                .py(px(cell_padding_block()))
                .border_b_1()
                .border_color(border_color())
                .bg(header_bg())
                .child(div().text_xs().text_color(header_color()).child("筛选"));
            if let Some(input) = &this.filter_input {
                bar = bar.child(div().w(px(240.)).child(input.clone()));
            }
            root = root.child(bar);
        }

        if let (Some(drag), Some(on_column_resize)) =
            (this.resize_drag.clone(), this.on_column_resize.clone())
        {
            let widths = this.resolved_widths.clone();
            let move_drag = drag.clone();
            root = root
                .on_mouse_move(move |event: &gpui::MouseMoveEvent, window, cx| {
                    let Some(active) = move_drag.read(cx).as_ref() else {
                        return;
                    };
                    let delta = f32::from(event.position.x) - active.start_x;
                    if delta.abs() < 0.5 {
                        return;
                    }
                    let mut next = widths.clone();
                    if let Some(width) = next.get_mut(active.column) {
                        *width = px((active.start_width + delta).max(64.));
                    }
                    on_column_resize(&next, window, cx);
                })
                .on_mouse_up(MouseButton::Left, {
                    let drag = drag.clone();
                    move |_, _, cx| drag.update(cx, |slot, _| *slot = None)
                })
                .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                    drag.update(cx, |slot, _| *slot = None)
                });
        }

        // 空态
        if total == 0 {
            return root.child(
                div()
                    .py_6()
                    .flex()
                    .justify_center()
                    .text_color(header_color())
                    .child(if this.filterable && !this.filter.is_empty() {
                        "无匹配数据"
                    } else {
                        "暂无数据"
                    }),
            );
        }

        let all_checked =
            !visible.is_empty() && visible.iter().all(|ix| this.selected_rows.contains(ix));
        let some_checked = visible.iter().any(|ix| this.selected_rows.contains(ix));

        // 区间 / 切换多选：锚点跨帧持久于窗口，供 Shift 点击计算连续区间
        let anchor = this.range_select.then(|| {
            window.use_keyed_state(
                ElementId::Name(format!("{:?}-selanchor", this.id).into()),
                cx,
                |_, _| None::<usize>,
            )
        });
        if let Some(height) = this.virtualized.filter(|_| !expandable && !fixed_layout) {
            let pagination = this.pagination_bar(total);
            root = root.child(this.virtual_body(
                &visible,
                cols,
                all_checked,
                some_checked,
                anchor.as_ref(),
                height,
                window,
                cx,
            ));
            if let Some(pagination) = pagination {
                root = root.child(pagination);
            }
            return root;
        }

        if fixed_layout {
            root = root.child(this.fixed_body(
                &visible,
                cols,
                all_checked,
                some_checked,
                anchor.as_ref(),
                window,
                cx,
            ));
        } else {
            root = this.normal_body(
                root,
                &visible,
                cols,
                all_checked,
                some_checked,
                expandable,
                anchor.as_ref(),
            );
        }

        if let Some(pagination) = this.pagination_bar(total) {
            root = root.child(pagination);
        }

        root
    }
}

impl Table {
    /// 底部分页栏（无分页配置时返回 None）
    fn pagination_bar(&self, total: usize) -> Option<Div> {
        let (page, size) = self.page?;
        let pages = ((total + size - 1) / size).max(1);
        let mut pag = Pagination::new(
            ElementId::Name(format!("{:?}-page", self.id).into()),
            page,
            pages,
        );
        if let Some(on_page_change) = self.on_page_change.clone() {
            pag = pag.on_change(move |p, window, cx| on_page_change(p, window, cx));
        }
        Some(
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
        )
    }
}

impl Table {
    /// 列宽容器：普通模式指定宽或 flex_1；固定 / 可拖拽模式指定宽或默认宽
    fn cell_w(&self, col: usize, fixed: bool) -> Div {
        let w = self.resolved_widths.get(col).copied().or_else(|| {
            self.widths
                .as_ref()
                .and_then(|widths| widths.get(col))
                .copied()
        });
        match (fixed || self.resizable_columns, w) {
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
            .relative()
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
        if self.resizable_columns
            && let Some(on_column_resize) = self.on_column_resize.clone()
        {
            let width = self
                .resolved_widths
                .get(col)
                .copied()
                .unwrap_or(px(default_col_width()));
            let mut handle = div()
                .id(ElementId::Name(
                    format!("{:?}-resize-{col}", self.id).into(),
                ))
                .absolute()
                .right_0()
                .top_0()
                .bottom_0()
                .w(px(6.))
                .cursor(CursorStyle::ResizeLeftRight)
                .hover(|style| style.bg(theme::with(|t| t.color_primary_bg)));
            if let Some(drag) = self.resize_drag.clone() {
                handle = handle.on_mouse_down(MouseButton::Left, move |event, _, cx| {
                    cx.stop_propagation();
                    drag.update(cx, |slot, _| {
                        *slot = Some(ColumnDrag {
                            column: col,
                            start_x: f32::from(event.position.x),
                            start_width: f32::from(width),
                        });
                    });
                });
            }
            let on_key_resize = on_column_resize.clone();
            let widths = Rc::new(self.resolved_widths.clone());
            let key_widths = widths.clone();
            handle = handle
                .on_key_down(move |event, window, cx| {
                    let delta = match event.keystroke.key.as_str() {
                        "left" | "arrowleft" => -8.,
                        "right" | "arrowright" => 8.,
                        _ => return,
                    };
                    cx.stop_propagation();
                    let next = table_resize_column(&key_widths, col, delta, 64.);
                    on_key_resize(&next, window, cx);
                })
                .on_a11y_action(gpui::AccessibleAction::Increment, {
                    let on_column_resize = on_column_resize.clone();
                    let widths = widths.clone();
                    move |_, window, cx| {
                        let next = table_resize_column(&widths, col, 8., 64.);
                        on_column_resize(&next, window, cx);
                    }
                })
                .on_a11y_action(gpui::AccessibleAction::Decrement, {
                    let on_column_resize = on_column_resize.clone();
                    let widths = widths.clone();
                    move |_, window, cx| {
                        let next = table_resize_column(&widths, col, -8., 64.);
                        on_column_resize(&next, window, cx);
                    }
                });
            cell = cell.child(
                Semantics::new(
                    Role::Slider,
                    SharedString::from(format!(
                        "调整列宽 {}",
                        self.headers.get(col).cloned().unwrap_or_default()
                    )),
                )
                .id(format!("{:?}-resize-{col}-a11y", self.id))
                .numeric(f64::from(width))
                .numeric_range(64., 4096.)
                .numeric_step(8.)
                .focusable(true)
                .apply(handle),
            );
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
            .id(ElementId::Name(
                format!("{:?}-cell-{ix}-{col}", self.id).into(),
            ))
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
        let editable = self.editable_columns.contains(&col) && self.on_edit_start.is_some();
        let semantics = Semantics::new(Role::Cell, text.clone())
            .id(format!("{:?}-cell-{ix}-{col}-a11y", self.id))
            .row(ix + 1, self.rows.len() + usize::from(self.show_header))
            .column(column_index, column_count)
            .focusable(editable);

        if self.editing == Some((ix, col))
            && let Some(input) = &self.edit_input
        {
            let cell = cell
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_key_down(|event, window, cx| {
                    if event.keystroke.key.as_str() == "enter" {
                        cx.stop_propagation();
                        window.blur(cx);
                    }
                });
            return semantics.apply(cell.child(input.clone()));
        }

        if editable {
            if let Some(focus) = self.editable_focus.get(&(ix, col)) {
                cell = cell.track_focus(focus);
            }
            if let Some(on_edit_start) = self.on_edit_start.clone() {
                let keyboard_edit_start = on_edit_start.clone();
                cell = cell.on_key_down(move |event, window, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        cx.stop_propagation();
                        keyboard_edit_start(&(ix, col), window, cx);
                    }
                });
                cell = a11y::click(cell, move |event, window, cx| {
                    if event.is_keyboard() || event.click_count() >= 2 {
                        cx.stop_propagation();
                        on_edit_start(&(ix, col), window, cx);
                    }
                });
            }
        }
        semantics.apply(cell.child(text))
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

    /// 普通模式表头（无固定列）
    fn plain_header(
        &self,
        visible: &[usize],
        cols: usize,
        all_checked: bool,
        some_checked: bool,
        expandable: bool,
        column_count: usize,
        column_offset: usize,
    ) -> Stateful<Div> {
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
            header = header.child(self.header_cell(col, false, column_offset + col, column_count));
        }
        let semantics = Semantics::new(Role::Row, "表头")
            .id(format!("{:?}-header-row-a11y", self.id))
            .row(0, self.rows.len() + 1);
        semantics.apply(header.id(ElementId::Name(format!("{:?}-header-row", self.id).into())))
    }

    /// 单行（普通模式 / 虚拟模式共用）
    fn row_element(
        &self,
        ix: usize,
        local: usize,
        cols: usize,
        expandable: bool,
        column_count: usize,
        column_offset: usize,
        anchor: Option<&Entity<Option<usize>>>,
    ) -> Stateful<Div> {
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
        self.with_row_click(row_div, ix, "row", anchor)
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
            root = root.child(self.plain_header(
                visible,
                cols,
                all_checked,
                some_checked,
                expandable,
                column_count,
                column_offset,
            ));
        }

        for (local, &ix) in visible.iter().enumerate() {
            root = root.child(self.row_element(
                ix,
                local,
                cols,
                expandable,
                column_count,
                column_offset,
                anchor,
            ));

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

    /// 虚拟模式：表头 + 固定高度视口内的按需行渲染 + 主题滚动条
    fn virtual_body(
        self,
        visible: &[usize],
        cols: usize,
        all_checked: bool,
        some_checked: bool,
        anchor: Option<&Entity<Option<usize>>>,
        height: Pixels,
        window: &mut Window,
        cx: &mut App,
    ) -> Div {
        let column_count = cols + usize::from(self.selectable);
        let column_offset = usize::from(self.selectable);
        let mut body = div().w_full().flex().flex_col();
        if self.show_header {
            body = body.child(self.plain_header(
                visible,
                cols,
                all_checked,
                some_checked,
                false,
                column_count,
                column_offset,
            ));
        }

        let table_id = format!("{:?}", self.id);
        let handle = self.virtual_scroll.clone().unwrap_or_else(|| {
            window
                .use_keyed_state(
                    ElementId::Name(format!("{table_id}-vscroll").into()),
                    cx,
                    |_, _| UniformListScrollHandle::new(),
                )
                .read(cx)
                .clone()
        });
        let bar = Scrollbar::vertical(
            ElementId::Name(format!("{table_id}-vbar").into()),
            handle.0.borrow().base_handle.clone(),
        );

        let visible: Vec<usize> = visible.to_vec();
        let anchor = anchor.cloned();
        let list = uniform_list(
            ElementId::Name(format!("{table_id}-vbody").into()),
            visible.len(),
            move |range, _, _| {
                range
                    .map(|local| {
                        let ix = visible[local];
                        self.row_element(
                            ix,
                            local,
                            cols,
                            false,
                            column_count,
                            column_offset,
                            anchor.as_ref(),
                        )
                    })
                    .collect::<Vec<_>>()
            },
        )
        .size_full()
        .track_scroll(&handle);

        body.child(
            div()
                .relative()
                .w_full()
                .h(height)
                .overflow_hidden()
                .child(list)
                .child(div().absolute().top_0().right_0().h_full().child(bar)),
        )
    }

    /// 固定列模式：左栏 / 右栏固定，中间栏横向滚动，统一行高逐行对齐。
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
        let (left_count, middle_count, right_count) =
            table_fixed_column_ranges(cols, self.fixed_left, self.fixed_right);
        let middle_start = left_count;
        let middle_end = cols - right_count;
        let right_start = cols - right_count;
        let width = |col: usize| {
            self.resolved_widths
                .get(col)
                .copied()
                .unwrap_or(px(default_col_width()))
        };
        let selection_w = if self.selectable {
            px(selection_column_width())
        } else {
            px(0.)
        };
        let left_w = selection_w + (0..left_count).fold(px(0.), |acc, col| acc + width(col));
        let middle_w = (middle_start..middle_end).fold(px(0.), |acc, col| acc + width(col));
        let right_w = (right_start..cols).fold(px(0.), |acc, col| acc + width(col));

        let mut left = div().flex().flex_col().w(left_w);
        if left_count > 0 || self.selectable {
            left = left.border_r_1().border_color(border_color());
        }
        let mut middle = div().flex().flex_col().min_w(middle_w);
        let mut right = div().flex().flex_col().w(right_w);
        if right_count > 0 {
            right = right.border_l_1().border_color(border_color());
        }

        if self.show_header {
            if left_count > 0 || self.selectable {
                let mut header = div()
                    .flex()
                    .flex_row()
                    .h(px(row_height()))
                    .bg(header_bg())
                    .border_b_1()
                    .border_color(border_color());
                if self.selectable {
                    header = header.child(self.check_all_cell(
                        all_checked,
                        some_checked,
                        visible,
                        true,
                        column_count,
                    ));
                }
                for col in 0..left_count {
                    header = header.child(self.header_cell(
                        col,
                        true,
                        column_offset + col,
                        column_count,
                    ));
                }
                left = left.child(
                    Semantics::new(Role::Row, "固定列表头")
                        .id(format!("{:?}-header-left-a11y", self.id))
                        .row(0, self.rows.len() + 1)
                        .apply(
                            header.id(ElementId::Name(format!("{:?}-header-left", self.id).into())),
                        ),
                );
            }
            if middle_count > 0 {
                let mut header = div()
                    .flex()
                    .flex_row()
                    .h(px(row_height()))
                    .bg(header_bg())
                    .border_b_1()
                    .border_color(border_color());
                for col in middle_start..middle_end {
                    header = header.child(self.header_cell(
                        col,
                        true,
                        column_offset + col,
                        column_count,
                    ));
                }
                middle = middle.child(
                    Semantics::new(Role::Row, "滚动列表头")
                        .id(format!("{:?}-header-middle-a11y", self.id))
                        .row(0, self.rows.len() + 1)
                        .apply(header.id(ElementId::Name(
                            format!("{:?}-header-middle", self.id).into(),
                        ))),
                );
            }
            if right_count > 0 {
                let mut header = div()
                    .flex()
                    .flex_row()
                    .h(px(row_height()))
                    .bg(header_bg())
                    .border_b_1()
                    .border_color(border_color());
                for col in right_start..cols {
                    header = header.child(self.header_cell(
                        col,
                        true,
                        column_offset + col,
                        column_count,
                    ));
                }
                right = right.child(
                    Semantics::new(Role::Row, "固定右列表头")
                        .id(format!("{:?}-header-right-a11y", self.id))
                        .row(0, self.rows.len() + 1)
                        .apply(header.id(ElementId::Name(
                            format!("{:?}-header-right", self.id).into(),
                        ))),
                );
            }
        }

        for (local, &ix) in visible.iter().enumerate() {
            let row = &self.rows[ix];
            if left_count > 0 || self.selectable {
                let mut left_row = div()
                    .id(ElementId::Name(format!("{:?}-rowL-{ix}", self.id).into()))
                    .flex()
                    .flex_row()
                    .h(px(row_height()))
                    .border_b_1()
                    .border_color(border_color());
                left_row = self.row_bg(left_row, local, ix);
                if self.selectable {
                    left_row = left_row.child(self.row_check_cell(ix, true, column_count));
                }
                for col in 0..left_count {
                    left_row = left_row.child(self.data_cell(
                        row,
                        ix,
                        col,
                        true,
                        column_offset + col,
                        column_count,
                    ));
                }
                left = left.child(self.with_row_click(left_row, ix, "rowL", anchor));
            }
            if middle_count > 0 {
                let mut middle_row = div()
                    .id(ElementId::Name(format!("{:?}-rowM-{ix}", self.id).into()))
                    .flex()
                    .flex_row()
                    .h(px(row_height()))
                    .border_b_1()
                    .border_color(border_color());
                middle_row = self.row_bg(middle_row, local, ix);
                for col in middle_start..middle_end {
                    middle_row = middle_row.child(self.data_cell(
                        row,
                        ix,
                        col,
                        true,
                        column_offset + col,
                        column_count,
                    ));
                }
                middle = middle.child(self.with_row_click(middle_row, ix, "rowM", anchor));
            }
            if right_count > 0 {
                let mut right_row = div()
                    .id(ElementId::Name(format!("{:?}-rowR-{ix}", self.id).into()))
                    .flex()
                    .flex_row()
                    .h(px(row_height()))
                    .border_b_1()
                    .border_color(border_color());
                right_row = self.row_bg(right_row, local, ix);
                for col in right_start..cols {
                    right_row = right_row.child(self.data_cell(
                        row,
                        ix,
                        col,
                        true,
                        column_offset + col,
                        column_count,
                    ));
                }
                right = right.child(self.with_row_click(right_row, ix, "rowR", anchor));
            }
        }

        let handle = window
            .use_keyed_state(
                ElementId::Name(format!("{:?}-hscroll", self.id).into()),
                cx,
                |_, _| ScrollHandle::new(),
            )
            .read(cx)
            .clone();

        let mut root = div().flex().flex_col().w_full().child(
            div()
                .flex()
                .flex_row()
                .w_full()
                .child(left)
                .child(
                    div()
                        .id(ElementId::Name(format!("{:?}-scroll", self.id).into()))
                        .flex_1()
                        .overflow_x_scroll()
                        .track_scroll(&handle)
                        .child(middle),
                )
                .child(right),
        );
        if middle_count > 0 {
            root = root.child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .child(div().w(left_w))
                    .child(div().flex_1().child(Scrollbar::horizontal(
                        ElementId::Name(format!("{:?}-hsb", self.id).into()),
                        handle,
                    )))
                    .child(div().w(right_w)),
            );
        }
        root
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
    use gpui::px;

    use super::{
        next_sort, range_selection, table_filtered_indices, table_fixed_column_ranges,
        table_resize_column,
    };

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

    #[test]
    fn filtering_is_case_insensitive_and_preserves_original_indices() {
        let rows = vec![
            vec!["Alpha".into(), "Ready".into()],
            vec!["Beta".into(), "Sync".into()],
            vec!["Gamma".into(), "alpha".into()],
        ];
        assert_eq!(table_filtered_indices(&rows, ""), vec![0, 1, 2]);
        assert_eq!(table_filtered_indices(&rows, "ALPHA"), vec![0, 2]);
        assert_eq!(table_filtered_indices(&rows, "sync"), vec![1]);
        assert!(table_filtered_indices(&rows, "missing").is_empty());
    }

    #[test]
    fn column_resize_clamps_to_minimum_and_keeps_other_columns() {
        let widths = vec![px(100.), px(120.), px(140.)];
        let next = table_resize_column(&widths, 1, 25., 64.);
        assert_eq!(next, vec![px(100.), px(145.), px(140.)]);
        let next = table_resize_column(&widths, 0, -500., 64.);
        assert_eq!(next, vec![px(64.), px(120.), px(140.)]);
        assert_eq!(table_resize_column(&widths, 9, 20., 64.), widths);
    }

    #[test]
    fn fixed_column_ranges_account_for_overlap() {
        assert_eq!(table_fixed_column_ranges(5, true, 1), (1, 3, 1));
        assert_eq!(table_fixed_column_ranges(5, false, 2), (0, 3, 2));
        assert_eq!(table_fixed_column_ranges(2, true, 9), (1, 0, 1));
        assert_eq!(table_fixed_column_ranges(0, true, 9), (0, 0, 0));
    }
}
