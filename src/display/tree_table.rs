//! 树形表格：层级展开 / 选中 / 勾选联动 + 表格列布局与虚拟滚动。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, FocusHandle, IntoElement, Pixels, RenderOnce, Role,
    SharedString, Stateful, Toggled, UniformListScrollHandle, Window, div, prelude::*, px,
    uniform_list,
};

use crate::{
    a11y::{self, Semantics},
    scrollbar::Scrollbar,
    theme::{
        self,
        metric::{TableMetric, TableStyle, TreeMetric, TreeStyle, table_style, tree_style},
    },
};

#[derive(Clone, Debug)]
pub struct TreeTableNode {
    id: SharedString,
    cells: Vec<SharedString>,
    children: Vec<TreeTableNode>,
}

impl TreeTableNode {
    pub fn new(
        id: impl Into<SharedString>,
        cells: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        Self {
            id: id.into(),
            cells: cells.into_iter().map(Into::into).collect(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: TreeTableNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = TreeTableNode>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn cells(&self) -> &[SharedString] {
        &self.cells
    }

    pub fn child_nodes(&self) -> &[TreeTableNode] {
        &self.children
    }
}

#[derive(Clone, Debug)]
struct FlatRow {
    id: SharedString,
    cells: Vec<SharedString>,
    depth: usize,
    parent: Option<SharedString>,
    has_children: bool,
    expanded: bool,
}

pub fn tree_table_visible_ids(
    nodes: &[TreeTableNode],
    expanded: &[SharedString],
) -> Vec<SharedString> {
    flatten_nodes(nodes, expanded)
        .into_iter()
        .map(|row| row.id)
        .collect()
}

pub fn tree_table_toggle_expanded(
    expanded: &[SharedString],
    id: &SharedString,
) -> Vec<SharedString> {
    let mut next = expanded.to_vec();
    if let Some(index) = next.iter().position(|current| current == id) {
        next.remove(index);
    } else {
        next.push(id.clone());
    }
    next.sort();
    next
}

pub fn tree_table_navigation_target(
    current: Option<usize>,
    key: &str,
    len: usize,
) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key {
        "up" => Some(current.map_or(len - 1, |index| index.saturating_sub(1))),
        "down" => Some(current.map_or(0, |index| (index + 1).min(len - 1))),
        "home" => Some(0),
        "end" => Some(len - 1),
        _ => None,
    }
}

pub fn tree_table_check_state(
    nodes: &[TreeTableNode],
    checked: &[SharedString],
    id: &SharedString,
) -> (bool, bool) {
    let Some(node) = find_node(nodes, id) else {
        return (false, false);
    };
    let descendants = descendants(node);
    let all = if descendants.is_empty() {
        checked.contains(id)
    } else {
        descendants.iter().all(|child| checked.contains(child))
    };
    let any = checked.contains(id) || descendants.iter().any(|child| checked.contains(child));
    (all, any && !all)
}

pub fn tree_table_cascade_check(
    nodes: &[TreeTableNode],
    checked: &[SharedString],
    id: &SharedString,
    check: bool,
) -> Vec<SharedString> {
    let mut next = checked.to_vec();
    let Some(node) = find_node(nodes, id) else {
        return next;
    };
    let mut affected = vec![id.clone()];
    affected.extend(descendants(node));
    if check {
        for id in affected {
            if !next.contains(&id) {
                next.push(id);
            }
        }
    } else {
        next.retain(|current| !affected.contains(current));
    }
    if let Some(chain) = find_ancestors(nodes, id) {
        for ancestor in chain.iter().rev() {
            let Some(node) = find_node(nodes, ancestor) else {
                continue;
            };
            let all_checked = !node.children.is_empty()
                && node.children.iter().all(|child| next.contains(&child.id));
            if all_checked {
                if !next.contains(ancestor) {
                    next.push(ancestor.clone());
                }
            } else {
                next.retain(|current| current != ancestor);
            }
        }
    }
    next.sort();
    next
}

fn flatten_nodes(nodes: &[TreeTableNode], expanded: &[SharedString]) -> Vec<FlatRow> {
    let mut rows = Vec::new();
    for node in nodes {
        flatten_node(node, 0, None, expanded, &mut rows);
    }
    rows
}

fn flatten_node(
    node: &TreeTableNode,
    depth: usize,
    parent: Option<SharedString>,
    expanded: &[SharedString],
    rows: &mut Vec<FlatRow>,
) {
    let is_expanded = expanded.contains(&node.id);
    rows.push(FlatRow {
        id: node.id.clone(),
        cells: node.cells.clone(),
        depth,
        parent,
        has_children: !node.children.is_empty(),
        expanded: is_expanded,
    });
    if is_expanded {
        for child in &node.children {
            flatten_node(child, depth + 1, Some(node.id.clone()), expanded, rows);
        }
    }
}

fn descendants(node: &TreeTableNode) -> Vec<SharedString> {
    let mut out = Vec::new();
    for child in &node.children {
        out.push(child.id.clone());
        out.extend(descendants(child));
    }
    out
}

fn find_node<'a>(nodes: &'a [TreeTableNode], id: &SharedString) -> Option<&'a TreeTableNode> {
    for node in nodes {
        if &node.id == id {
            return Some(node);
        }
        if let Some(found) = find_node(&node.children, id) {
            return Some(found);
        }
    }
    None
}

fn find_ancestors(nodes: &[TreeTableNode], target: &SharedString) -> Option<Vec<SharedString>> {
    for node in nodes {
        if &node.id == target {
            return Some(Vec::new());
        }
        if let Some(mut chain) = find_ancestors(&node.children, target) {
            chain.push(node.id.clone());
            return Some(chain);
        }
    }
    None
}

fn all_node_ids(nodes: &[TreeTableNode]) -> Vec<SharedString> {
    let mut out = Vec::new();
    for node in nodes {
        out.push(node.id.clone());
        out.extend(all_node_ids(&node.children));
    }
    out
}

/// 树形表格。展开、选中与勾选状态均由调用方持有。
#[derive(IntoElement)]
pub struct TreeTable {
    id: ElementId,
    headers: Vec<SharedString>,
    nodes: Vec<TreeTableNode>,
    expanded: Vec<SharedString>,
    selected: Option<SharedString>,
    checkable: bool,
    checked: Vec<SharedString>,
    widths: Option<Vec<Pixels>>,
    striped: bool,
    bordered: bool,
    show_header: bool,
    virtualized: Option<Pixels>,
    virtual_scroll: Option<UniformListScrollHandle>,
    on_toggle: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_select: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_check: Option<Rc<dyn Fn(&Vec<SharedString>, &mut Window, &mut App)>>,
}

impl TreeTable {
    pub fn new(
        id: impl Into<ElementId>,
        headers: impl IntoIterator<Item = impl Into<SharedString>>,
        nodes: Vec<TreeTableNode>,
    ) -> Self {
        Self {
            id: id.into(),
            headers: headers.into_iter().map(Into::into).collect(),
            nodes,
            expanded: Vec::new(),
            selected: None,
            checkable: false,
            checked: Vec::new(),
            widths: None,
            striped: false,
            bordered: false,
            show_header: true,
            virtualized: None,
            virtual_scroll: None,
            on_toggle: None,
            on_select: None,
            on_check: None,
        }
    }

    pub fn expanded(mut self, expanded: Vec<SharedString>) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn selected(mut self, selected: Option<SharedString>) -> Self {
        self.selected = selected;
        self
    }

    pub fn checkable(mut self, checkable: bool) -> Self {
        self.checkable = checkable;
        self
    }

    pub fn checked(mut self, checked: Vec<SharedString>) -> Self {
        self.checked = checked;
        self
    }

    pub fn column_widths(mut self, widths: Vec<Pixels>) -> Self {
        self.widths = Some(widths);
        self
    }

    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// 固定行高虚拟滚动；`height` 不含表头。
    pub fn virtualized(mut self, height: impl Into<Pixels>) -> Self {
        self.virtualized = Some(height.into());
        self
    }

    pub fn virtual_scroll_handle(mut self, handle: UniformListScrollHandle) -> Self {
        self.virtual_scroll = Some(handle);
        self
    }

    pub fn on_toggle(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(listener));
        self
    }

    pub fn on_select(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }

    pub fn on_check(
        mut self,
        listener: impl Fn(&Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_check = Some(Rc::new(listener));
        self
    }
}

#[derive(Clone)]
struct RowRenderer {
    owner: String,
    nodes: Rc<Vec<TreeTableNode>>,
    cols: usize,
    widths: Rc<Vec<Pixels>>,
    rows: Rc<Vec<FlatRow>>,
    selected: Option<SharedString>,
    checkable: bool,
    checked: Vec<SharedString>,
    striped: bool,
    bordered: bool,
    show_header: bool,
    table_metric: TableMetric,
    tree_metric: TreeMetric,
    table_style: TableStyle,
    tree_style: TreeStyle,
    row_height: f32,
    on_toggle: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_select: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_check: Option<Rc<dyn Fn(&Vec<SharedString>, &mut Window, &mut App)>>,
    focus: FocusHandle,
}

impl RowRenderer {
    fn cell_w(&self, col: usize) -> Div {
        match self.widths.get(col).copied() {
            Some(width) => div().w(width),
            None => div().flex_1().min_w_0(),
        }
    }

    fn check_box(&self, id: &str, checked: bool, half: bool) -> Stateful<Div> {
        let mut box_el = div()
            .id(ElementId::Name(id.to_string().into()))
            .size(px(self.tree_metric.checkbox_size))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(self.tree_metric.checkbox_radius))
            .border_1();
        if checked || half {
            box_el = box_el
                .border_color(self.tree_style.checkbox_border)
                .bg(self.tree_style.checkbox_bg)
                .text_color(self.tree_style.check_fg)
                .text_xs()
                .child(if checked { "✓" } else { "−" });
        } else {
            box_el = box_el
                .border_color(crate::theme::border())
                .bg(crate::theme::bg_container());
        }
        box_el
    }

    fn row_check_cell(&self, row: &FlatRow, row_index: usize) -> Stateful<Div> {
        let (checked, half) = tree_table_check_state(&self.nodes, &self.checked, &row.id);
        let id = format!("{}-check-{}", self.owner, row.id);
        let mut cell = div()
            .id(ElementId::Name(format!("{id}-cell").into()))
            .w(px(self.table_metric.selection_column_width))
            .flex()
            .items_center()
            .justify_center()
            .cursor(CursorStyle::PointingHand)
            .child(self.check_box(&id, checked, half));
        if let Some(on_check) = &self.on_check {
            let on_check = on_check.clone();
            let row_id = row.id.clone();
            let current = self.checked.clone();
            let nodes = self.nodes.clone();
            cell = a11y::click(cell, move |_, window, cx| {
                cx.stop_propagation();
                let next = tree_table_cascade_check(&nodes, &current, &row_id, !checked);
                on_check(&next, window, cx);
            });
        }
        Semantics::new(
            Role::CheckBox,
            format!("勾选 {}", row.cells.first().cloned().unwrap_or_default()),
        )
        .id(format!("{id}-a11y"))
        .toggled(if checked {
            Toggled::True
        } else if half {
            Toggled::Mixed
        } else {
            Toggled::False
        })
        .row(
            row_index + 1,
            self.rows.len() + usize::from(self.show_header),
        )
        .column(0, self.cols + usize::from(self.checkable))
        .focusable(self.on_check.is_some())
        .apply(cell)
    }

    fn arrow(&self, row: &FlatRow) -> Stateful<Div> {
        let switcher_gap =
            self.tree_metric.switcher_size - crate::theme::with(|t| t.control_interactive_size);
        let mut arrow = div()
            .id(ElementId::Name(
                format!("{}-arrow-{}", self.owner, row.id).into(),
            ))
            .w(px(self.tree_metric.switcher_size))
            .h(px(self.tree_metric.switcher_size))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(self.tree_metric.switcher_icon_size))
            .text_color(self.tree_style.switcher)
            .mr(px(switcher_gap / 2.));
        if row.has_children {
            arrow = Semantics::new(
                Role::Button,
                format!("{} {}", if row.expanded { "收起" } else { "展开" }, row.id),
            )
            .id(format!("{}-arrow-{}", self.owner, row.id))
            .expanded(row.expanded)
            .focusable(self.on_toggle.is_some())
            .apply(arrow);
            arrow = arrow
                .cursor(CursorStyle::PointingHand)
                .child(if row.expanded { "▾" } else { "▸" });
            if let Some(on_toggle) = &self.on_toggle {
                let on_toggle = on_toggle.clone();
                let id = row.id.clone();
                arrow = a11y::click(arrow, move |_, window, cx| {
                    cx.stop_propagation();
                    on_toggle(&id, window, cx);
                });
            }
        }
        arrow
    }

    fn first_cell(&self, row: &FlatRow, row_index: usize, col: usize) -> Stateful<Div> {
        let text = row.cells.first().cloned().unwrap_or_default();
        let mut cell = self
            .cell_w(col)
            .id(ElementId::Name(
                format!("{}-cell-{}-{col}", self.owner, row.id).into(),
            ))
            .flex()
            .flex_row()
            .items_center()
            .px(px(self.table_metric.cell_padding_inline))
            .overflow_hidden();
        if self.bordered && col > 0 {
            cell = cell
                .border_l_1()
                .border_color(self.table_style.border_color);
        }
        cell = cell
            .child(
                div()
                    .w(px(row.depth as f32 * self.tree_metric.indent))
                    .flex_shrink_0(),
            )
            .child(self.arrow(row))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(text.clone()),
            );
        Semantics::new(Role::Cell, text)
            .id(format!("{}-cell-{}-{col}-a11y", self.owner, row.id))
            .row(
                row_index + 1,
                self.rows.len() + usize::from(self.show_header),
            )
            .column(
                col + usize::from(self.checkable),
                self.cols + usize::from(self.checkable),
            )
            .apply(cell)
    }

    fn data_cell(&self, row: &FlatRow, row_index: usize, col: usize) -> Stateful<Div> {
        let text = row.cells.get(col).cloned().unwrap_or_default();
        let mut cell = self
            .cell_w(col)
            .id(ElementId::Name(
                format!("{}-cell-{}-{col}", self.owner, row.id).into(),
            ))
            .flex()
            .items_center()
            .px(px(self.table_metric.cell_padding_inline))
            .overflow_hidden()
            .whitespace_nowrap();
        if self.bordered && col > 0 {
            cell = cell
                .border_l_1()
                .border_color(self.table_style.border_color);
        }
        Semantics::new(Role::Cell, text.clone())
            .id(format!("{}-cell-{}-{col}-a11y", self.owner, row.id))
            .row(
                row_index + 1,
                self.rows.len() + usize::from(self.show_header),
            )
            .column(
                col + usize::from(self.checkable),
                self.cols + usize::from(self.checkable),
            )
            .apply(cell.child(div().flex_1().min_w_0().text_ellipsis().child(text)))
    }

    fn render_row(&self, index: usize, local: usize) -> Stateful<Div> {
        let row = &self.rows[index];
        let selected = self.selected.as_ref() == Some(&row.id);
        let label = if row.cells.is_empty() {
            row.id.to_string()
        } else {
            row.cells.join("，")
        };
        let mut row_div = div()
            .id(ElementId::Name(
                format!("{}-row-{}", self.owner, row.id).into(),
            ))
            .flex()
            .flex_row()
            .h(px(self.row_height))
            .border_b_1()
            .border_color(self.table_style.border_color)
            .cursor(CursorStyle::PointingHand);
        if selected {
            row_div = row_div.bg(self.table_style.row_selected_bg);
        } else if self.striped && local % 2 == 1 {
            row_div = row_div.bg(self.table_style.header_bg);
        }
        if self.checkable {
            row_div = row_div.child(self.row_check_cell(row, index));
        }
        for col in 0..self.cols {
            row_div = row_div.child(if col == 0 {
                self.first_cell(row, index, col)
            } else {
                self.data_cell(row, index, col)
            });
        }
        let on_select = self.on_select.clone();
        let focus = self.focus.clone();
        let row_id = row.id.clone();
        row_div = row_div.hover(move |style| {
            if selected {
                style
            } else {
                style.bg(self.table_style.row_hover_bg)
            }
        });
        row_div = a11y::click_void(row_div, move |window, cx| {
            focus.focus(window, cx);
            if let Some(on_select) = &on_select {
                on_select(&row_id, window, cx);
            }
        });
        Semantics::new(Role::Row, label)
            .id(format!("{}-row-{}-a11y", self.owner, row.id))
            .row(index + 1, self.rows.len() + usize::from(self.show_header))
            .selected(selected)
            .focusable(self.on_select.is_some())
            .apply(row_div)
    }
}

impl RenderOnce for TreeTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let owner = format!("{:?}", self.id);
        let rows = Rc::new(flatten_nodes(&self.nodes, &self.expanded));
        let cols = self
            .headers
            .len()
            .max(
                self.nodes
                    .iter()
                    .map(|node| node.cells.len())
                    .max()
                    .unwrap_or(0),
            )
            .max(1);
        let (table_metric, tree_metric, table_style, tree_style) = theme::with(|t| {
            (
                TableMetric::resolve(t),
                TreeMetric::resolve(t),
                table_style(t),
                tree_style(t),
            )
        });
        let row_height = table_metric.line_height + table_metric.cell_padding_block * 2.;
        let focus = window
            .use_keyed_state(
                ElementId::Name(format!("{owner}-focus").into()),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone();
        let root_id = format!("{owner}-table");
        let mut root = Semantics::new(Role::Table, "树形表格")
            .id(root_id.clone())
            .value(format!("共 {} 行，{} 列", rows.len(), cols))
            .focusable(true)
            .apply(
                div()
                    .id(ElementId::Name(root_id.into()))
                    .track_focus(&focus)
                    .flex()
                    .flex_col()
                    .w_full()
                    .rounded(px(table_metric.radius))
                    .bg(crate::theme::bg_container())
                    .overflow_hidden()
                    .text_sm(),
            );
        if self.bordered {
            root = root.border_1().border_color(table_style.border_color);
        }
        if self.show_header {
            root = root.child(self.render_header(
                &owner,
                cols,
                &table_metric,
                &table_style,
                &tree_metric,
                row_height,
            ));
        }
        if rows.is_empty() {
            return root.child(
                div()
                    .py_6()
                    .flex()
                    .justify_center()
                    .text_color(table_style.header_color)
                    .child("暂无数据"),
            );
        }

        let renderer = RowRenderer {
            owner: owner.clone(),
            nodes: Rc::new(self.nodes.clone()),
            cols,
            widths: Rc::new(self.widths.clone().unwrap_or_default()),
            rows: rows.clone(),
            selected: self.selected.clone(),
            checkable: self.checkable,
            checked: self.checked.clone(),
            striped: self.striped,
            bordered: self.bordered,
            show_header: self.show_header,
            table_metric,
            tree_metric,
            table_style,
            tree_style,
            row_height,
            on_toggle: self.on_toggle.clone(),
            on_select: self.on_select.clone(),
            on_check: self.on_check.clone(),
            focus: focus.clone(),
        };

        if let Some(height) = self.virtualized {
            let handle = self.virtual_scroll.clone().unwrap_or_else(|| {
                window
                    .use_keyed_state(
                        ElementId::Name(format!("{owner}-vscroll").into()),
                        cx,
                        |_, _| UniformListScrollHandle::new(),
                    )
                    .read(cx)
                    .clone()
            });
            let bar = Scrollbar::vertical(
                ElementId::Name(format!("{owner}-vbar").into()),
                handle.0.borrow().base_handle.clone(),
            );
            let len = rows.len();
            let list = uniform_list(
                ElementId::Name(format!("{owner}-vbody").into()),
                len,
                move |range, _, _| {
                    range
                        .map(|local| renderer.render_row(local, local))
                        .collect::<Vec<_>>()
                },
            )
            .size_full()
            .track_scroll(&handle);
            root = root.child(
                div()
                    .relative()
                    .w_full()
                    .h(height)
                    .overflow_hidden()
                    .child(list)
                    .child(div().absolute().top_0().right_0().h_full().child(bar)),
            );
        } else {
            for local in 0..rows.len() {
                root = root.child(renderer.render_row(local, local));
            }
        }

        let rows_for_key = rows.clone();
        let nodes_for_key = self.nodes.clone();
        let selected = self.selected.clone();
        let on_toggle = self.on_toggle.clone();
        let on_select = self.on_select.clone();
        let on_check = self.on_check.clone();
        let checkable = self.checkable;
        let checked = self.checked;
        root = root.on_key_down(move |event, window, cx| {
            let key = event.keystroke.key.as_str();
            let current = selected
                .as_ref()
                .and_then(|id| rows_for_key.iter().position(|row| &row.id == id));
            if let Some(target) = tree_table_navigation_target(current, key, rows_for_key.len()) {
                cx.stop_propagation();
                if let Some(on_select) = &on_select {
                    on_select(&rows_for_key[target].id, window, cx);
                }
                return;
            }
            let Some(current) = current else {
                return;
            };
            let row = &rows_for_key[current];
            match key {
                "left" => {
                    cx.stop_propagation();
                    if row.has_children && row.expanded {
                        if let Some(on_toggle) = &on_toggle {
                            on_toggle(&row.id, window, cx);
                        }
                    } else if let Some(parent) = &row.parent {
                        if let Some(on_select) = &on_select {
                            on_select(parent, window, cx);
                        }
                    }
                }
                "right" => {
                    cx.stop_propagation();
                    if row.has_children && !row.expanded {
                        if let Some(on_toggle) = &on_toggle {
                            on_toggle(&row.id, window, cx);
                        }
                    } else if current + 1 < rows_for_key.len() {
                        if let Some(on_select) = &on_select {
                            on_select(&rows_for_key[current + 1].id, window, cx);
                        }
                    }
                }
                "enter" => {
                    cx.stop_propagation();
                    if let Some(on_select) = &on_select {
                        on_select(&row.id, window, cx);
                    }
                }
                "space" if checkable => {
                    cx.stop_propagation();
                    if let Some(on_check) = &on_check {
                        let next = tree_table_cascade_check(
                            &nodes_for_key,
                            &checked,
                            &row.id,
                            !checked.contains(&row.id),
                        );
                        on_check(&next, window, cx);
                    }
                }
                _ => {}
            }
        });
        root
    }
}

impl TreeTable {
    #[allow(clippy::too_many_arguments)]
    fn render_header(
        &self,
        owner: &str,
        cols: usize,
        table_metric: &TableMetric,
        table_style: &TableStyle,
        tree_metric: &TreeMetric,
        row_height: f32,
    ) -> Stateful<Div> {
        let total_rows = tree_table_visible_ids(&self.nodes, &self.expanded).len()
            + usize::from(self.show_header);
        let mut header = div()
            .id(ElementId::Name(format!("{owner}-header-row").into()))
            .flex()
            .flex_row()
            .h(px(row_height))
            .bg(table_style.header_bg)
            .border_b_1()
            .border_color(table_style.border_color);
        if self.checkable {
            let ids = all_node_ids(&self.nodes);
            let all = !ids.is_empty() && ids.iter().all(|id| self.checked.contains(id));
            let some = ids.iter().any(|id| self.checked.contains(id));
            let id = format!("{owner}-check-all");
            let mut cell = div()
                .id(ElementId::Name(format!("{id}-cell").into()))
                .w(px(table_metric.selection_column_width))
                .flex()
                .items_center()
                .justify_center()
                .cursor(CursorStyle::PointingHand)
                .child(check_box_static(
                    &id,
                    all,
                    some && !all,
                    *tree_metric,
                    *table_style,
                ));
            if let Some(on_check) = &self.on_check {
                let on_check = on_check.clone();
                let nodes = self.nodes.clone();
                let current = self.checked.clone();
                let next = if all {
                    Vec::new()
                } else {
                    let mut next = current;
                    for root in &nodes {
                        next = tree_table_cascade_check(&nodes, &next, &root.id, true);
                    }
                    next
                };
                cell = a11y::click_void(cell, move |window, cx| {
                    on_check(&next, window, cx);
                });
            }
            header = header.child(
                Semantics::new(Role::CheckBox, "全选")
                    .id(format!("{id}-a11y"))
                    .toggled(if all {
                        Toggled::True
                    } else if some {
                        Toggled::Mixed
                    } else {
                        Toggled::False
                    })
                    .row(0, total_rows)
                    .column(0, cols + 1)
                    .focusable(self.on_check.is_some())
                    .apply(cell),
            );
        }
        for col in 0..cols {
            let title = self.headers.get(col).cloned().unwrap_or_default();
            let width = self
                .widths
                .as_ref()
                .and_then(|widths| widths.get(col))
                .copied();
            let mut cell = match width {
                Some(width) => div().w(width),
                None => div().flex_1().min_w_0(),
            }
            .id(ElementId::Name(format!("{owner}-header-{col}").into()))
            .flex()
            .items_center()
            .px(px(table_metric.cell_padding_inline))
            .overflow_hidden()
            .whitespace_nowrap()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(table_style.header_color);
            if col == 0 {
                let switcher_gap =
                    tree_metric.switcher_size - crate::theme::with(|t| t.control_interactive_size);
                cell = cell.pl(px(table_metric.cell_padding_inline
                    + tree_metric.switcher_size
                    + switcher_gap / 2.));
            } else if self.bordered {
                cell = cell.border_l_1().border_color(table_style.border_color);
            }
            header = header.child(
                Semantics::new(Role::ColumnHeader, title.clone())
                    .id(format!("{owner}-header-{col}-a11y"))
                    .row(0, total_rows)
                    .column(
                        col + usize::from(self.checkable),
                        cols + usize::from(self.checkable),
                    )
                    .apply(cell.child(title)),
            );
        }
        Semantics::new(Role::Row, "表头")
            .id(format!("{owner}-header-a11y"))
            .row(0, total_rows)
            .apply(header)
    }
}

fn check_box_static(
    id: &str,
    checked: bool,
    half: bool,
    metric: TreeMetric,
    style: TableStyle,
) -> Stateful<Div> {
    let mut box_el = div()
        .id(ElementId::Name(id.to_string().into()))
        .size(px(metric.checkbox_size))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(metric.checkbox_radius))
        .border_1();
    if checked || half {
        box_el = box_el
            .border_color(crate::theme::primary())
            .bg(crate::theme::primary())
            .text_color(crate::theme::with(|t| t.color_text_light_solid))
            .text_xs()
            .child(if checked { "✓" } else { "−" });
    } else {
        box_el = box_el
            .border_color(style.border_color)
            .bg(crate::theme::bg_container());
    }
    box_el
}
