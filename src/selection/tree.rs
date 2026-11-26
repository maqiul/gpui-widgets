//! 树形控件：层级节点展开 / 收起 / 选中 / 勾选联动 / 拖拽移动，状态由父视图持有（受控）。

use std::rc::Rc;

use gpui::{
    App, Context, CursorStyle, Div, ElementId, IntoElement, Render, Role, SharedString, Stateful,
    StatefulInteractiveElement, Toggled, Window, div, prelude::*, px,
};

use crate::{
    a11y,
    theme::{
        self,
        metric::{TreeMetric, tree_style},
    },
};

/// 拖拽释放事件：dragged 被拖节点 id，target 放置目标节点 id
#[derive(Clone)]
pub struct TreeDrop {
    pub dragged: SharedString,
    pub target: SharedString,
}

/// 树节点（id 用于展开 / 选中标识，同一棵树内需唯一）
#[derive(Clone)]
pub struct TreeNode {
    id: SharedString,
    label: SharedString,
    children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = TreeNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// 追加子节点（配合 take_node / find_node_mut 做拖拽后的树重组）
    pub fn push_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

    pub(crate) fn id(&self) -> &SharedString {
        &self.id
    }

    pub(crate) fn label(&self) -> &SharedString {
        &self.label
    }

    pub(crate) fn child_nodes(&self) -> &[TreeNode] {
        &self.children
    }
}

/// 树形控件。
///
/// ```ignore
/// Tree::new("files", vec![
///     TreeNode::new("src", "src").child(TreeNode::new("main", "main.rs")),
/// ])
/// .expanded(self.tree_expanded.clone())
/// .on_toggle(cx.listener(|this, id, _, cx| { /* 切换 id 的展开状态 */ }))
/// .selected(self.tree_selected.clone())
/// .on_select(cx.listener(|this, id, _, cx| { this.tree_selected = Some(id.clone()); cx.notify(); }))
/// ```
pub struct Tree {
    id: ElementId,
    nodes: Vec<TreeNode>,
    expanded: Vec<SharedString>,
    selected: Option<SharedString>,
    checkable: bool,
    checked: Vec<SharedString>,
    draggable: bool,
    block_node: bool,
    on_toggle: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_select: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_check: Option<Rc<dyn Fn(&Vec<SharedString>, &mut Window, &mut App)>>,
    on_drop_node: Option<Rc<dyn Fn(&TreeDrop, &mut Window, &mut App)>>,
}

impl Tree {
    pub fn new(id: impl Into<ElementId>, nodes: Vec<TreeNode>) -> Self {
        Self {
            id: id.into(),
            nodes,
            expanded: Vec::new(),
            selected: None,
            checkable: false,
            checked: Vec::new(),
            draggable: false,
            block_node: false,
            on_toggle: None,
            on_select: None,
            on_check: None,
            on_drop_node: None,
        }
    }

    /// 当前展开的节点 id 集合（受控）
    pub fn expanded(mut self, expanded: Vec<SharedString>) -> Self {
        self.expanded = expanded;
        self
    }

    /// 当前选中节点 id（受控）
    pub fn selected(mut self, selected: Option<SharedString>) -> Self {
        self.selected = selected;
        self
    }

    /// 点击展开箭头时回调节点 id
    pub fn on_toggle(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(listener));
        self
    }

    /// 点击行时回调节点 id
    pub fn on_select(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }

    /// 显示勾选框（父子联动：勾选含全部子孙，父节点随子节点全选自动勾选）
    pub fn checkable(mut self, checkable: bool) -> Self {
        self.checkable = checkable;
        self
    }

    /// 当前勾选的节点 id 集合（受控）
    pub fn checked(mut self, checked: Vec<SharedString>) -> Self {
        self.checked = checked;
        self
    }

    /// 勾选变化回调（已含父子联动计算）
    pub fn on_check(
        mut self,
        listener: impl Fn(&Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_check = Some(Rc::new(listener));
        self
    }

    /// 允许拖拽节点行（拖到另一节点上释放触发 on_drop_node，
    /// 目标为自身或自身子孙时不触发；节点树重组由调用方处理）
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    pub fn on_drop_node(
        mut self,
        listener: impl Fn(&TreeDrop, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_drop_node = Some(Rc::new(listener));
        self
    }

    /// 节点行铺满容器宽度（antd `blockNode`，TreeSelect 内部树启用）：
    /// 选中 / 悬停底延伸到行尾，并补上 antd 树节点的 4px 行距与箭头右距。
    pub fn block_node(mut self, block_node: bool) -> Self {
        self.block_node = block_node;
        self
    }
}

impl IntoElement for Tree {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let total = visible_node_count(&self.nodes, &self.expanded);
        let mut rows = Vec::new();
        for node in &self.nodes {
            flatten(node, 0, &self, total, &mut rows);
        }
        let root = div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .w_full()
            .text_sm()
            .children(rows);
        a11y::Semantics::new(Role::Tree, "树形结构")
            .id(format!("{:?}", self.id))
            .apply(root)
    }
}

/// 当前展开状态下可见节点数，用于 TreeItem 的 posinset/setsize。
fn visible_node_count(nodes: &[TreeNode], expanded: &[SharedString]) -> usize {
    nodes
        .iter()
        .map(|node| {
            1 + if expanded.contains(&node.id) {
                visible_node_count(&node.children, expanded)
            } else {
                0
            }
        })
        .sum()
}

/// 递归展平为带缩进的行
fn flatten(
    node: &TreeNode,
    depth: usize,
    tree: &Tree,
    total: usize,
    rows: &mut Vec<Stateful<Div>>,
) {
    let has_children = !node.children.is_empty();
    let is_expanded = tree.expanded.contains(&node.id);
    let is_selected = tree.selected.as_ref() == Some(&node.id);

    let (m, style, node_gap, switcher_gap) = theme::with(|t| {
        let m = TreeMetric::resolve(t);
        (
            m,
            tree_style(t),
            // antd `${tree}-treenode` 的 marginBottom = paddingXS/2（行距 28 = 24 + 4）
            t.padding_xs / 2.0,
            // antd 箭头 marginInlineEnd = (switcherSize - controlInteractiveSize) / 2
            (m.switcher_size - t.control_interactive_size) / 2.0,
        )
    });
    let block_node = tree.block_node;

    // 展开箭头（无子节点时占位对齐）
    let mut arrow = div()
        .id(ElementId::Name(
            format!("{:?}-arrow-{}", tree.id, node.id).into(),
        ))
        .flex()
        .items_center()
        .justify_center()
        .w(px(m.switcher_size))
        .h(px(m.switcher_size))
        .text_size(px(m.switcher_icon_size))
        .text_color(style.switcher);
    if has_children {
        arrow = a11y::Semantics::new(
            Role::Button,
            format!(
                "{} {}",
                if is_expanded { "收起" } else { "展开" },
                node.label
            ),
        )
        .id(format!("{:?}-arrow-{}", tree.id, node.id))
        .expanded(is_expanded)
        .focusable(tree.on_toggle.is_some())
        .apply(arrow);
        let on_toggle = tree.on_toggle.clone();
        let node_id = node.id.clone();
        arrow = arrow
            .cursor(CursorStyle::PointingHand)
            .child(if is_expanded { "▾" } else { "▸" });
        if let Some(on_toggle) = on_toggle {
            arrow = a11y::click(arrow, move |_, window, cx| {
                cx.stop_propagation();
                on_toggle(&node_id, window, cx);
            });
        }
    }

    let mut row = div()
        .id(ElementId::Name(
            format!("{:?}-node-{}", tree.id, node.id).into(),
        ))
        .flex()
        .flex_row()
        .items_center()
        .pl(px(depth as f32 * m.indent))
        .child(if block_node {
            arrow.mr(px(switcher_gap))
        } else {
            arrow
        });
    if block_node {
        row = row.mb(px(node_gap));
    }

    // 勾选框（父子联动）
    if tree.checkable {
        let checked = tree.checked.contains(&node.id);
        let half = !checked && {
            let desc = descendants(node);
            !desc.is_empty() && desc.iter().any(|id| tree.checked.contains(id))
        };
        let on_check = tree.on_check.clone();
        let next = cascade_check(&tree.nodes, &tree.checked, &node.id, !checked);
        let mut box_el = div()
            .id(ElementId::Name(
                format!("{:?}-check-{}", tree.id, node.id).into(),
            ))
            .size(px(m.checkbox_size))
            .mr(px(m.checkbox_gap))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(m.checkbox_radius))
            .border_1()
            .text_xs()
            .cursor(CursorStyle::PointingHand);
        if checked || half {
            box_el = box_el
                .border_color(style.checkbox_border)
                .bg(style.checkbox_bg)
                .text_color(style.check_fg)
                .child(if checked { "✓" } else { "−" });
        } else {
            box_el = box_el
                .border_color(theme::border())
                .bg(theme::bg_container());
        }
        box_el = a11y::Semantics::new(Role::CheckBox, format!("勾选 {}", node.label))
            .id(format!("{:?}-check-{}", tree.id, node.id))
            .toggled(if half {
                Toggled::Mixed
            } else if checked {
                Toggled::True
            } else {
                Toggled::False
            })
            .focusable(on_check.is_some())
            .apply(box_el);
        if let Some(on_check) = on_check {
            box_el = a11y::click(box_el, move |_, window, cx| {
                cx.stop_propagation();
                on_check(&next, window, cx);
            });
        }
        row = row.child(box_el);
    }

    let mut content = div()
        .id(ElementId::Name(
            format!("{:?}-content-{}", tree.id, node.id).into(),
        ))
        .flex()
        .items_center()
        .px(px(m.content_padding_inline))
        .h(px(m.title_height))
        .rounded(px(m.radius))
        .text_color(if is_selected {
            style.selected_text
        } else {
            style.text
        });
    if is_selected {
        content = content.bg(style.selected_bg);
    }
    if block_node {
        // antd blockNode：内容盒 flex:auto，底铺到行尾（右侧仍留 popup 内边距）
        content = content.flex_1().min_w_0();
    }
    content = content.child(node.label.clone());
    content = a11y::Semantics::new(Role::TreeItem, node.label.clone())
        .id(format!("{:?}-content-{}", tree.id, node.id))
        .selected(is_selected)
        .expanded(is_expanded)
        .level(depth + 1)
        .position(rows.len() + 1, total)
        .focusable(tree.on_select.is_some())
        .apply(content);

    if let Some(on_select) = tree.on_select.clone() {
        let node_id = node.id.clone();
        content = content
            .cursor(CursorStyle::PointingHand)
            .hover(move |s| s.bg(style.hover_bg));
        content = a11y::click_void(content, move |window, cx| {
            on_select(&node_id, window, cx);
        });
    }
    row = row.child(content);

    // 拖拽移动：拖起时渲染标签幽灵，释放到目标行触发 on_drop_node
    if tree.draggable {
        let drag = TreeDrag {
            id: node.id.clone(),
            label: node.label.clone(),
        };
        row = row.on_drag(drag, |value, _offset, _window, cx| {
            cx.new(|_| TreeDragGhost {
                label: value.label.clone(),
            })
        });
        if let Some(on_drop_node) = tree.on_drop_node.clone() {
            let target = node.id.clone();
            let dragged_desc = descendants(node);
            row = row
                .can_drop({
                    let target = target.clone();
                    let dragged_desc = dragged_desc.clone();
                    move |value, _window, _cx| {
                        value
                            .downcast_ref::<TreeDrag>()
                            .map(|d| d.id != target && !dragged_desc.contains(&d.id))
                            .unwrap_or(false)
                    }
                })
                .on_drop::<TreeDrag>(move |dragged, window, cx| {
                    if dragged.id == target || dragged_desc.contains(&dragged.id) {
                        return;
                    }
                    on_drop_node(
                        &TreeDrop {
                            dragged: dragged.id.clone(),
                            target: target.clone(),
                        },
                        window,
                        cx,
                    );
                });
        }
    }
    rows.push(row);

    if has_children && is_expanded {
        for child in &node.children {
            flatten(child, depth + 1, tree, total, rows);
        }
    }
}

/// 拖拽载荷（内部类型，on_drop 按此类型匹配）
#[derive(Clone)]
struct TreeDrag {
    id: SharedString,
    label: SharedString,
}

/// 拖拽跟随的幽灵视图
struct TreeDragGhost {
    label: SharedString,
}

impl Render for TreeDragGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(theme::accent())
            .bg(theme::surface())
            .text_sm()
            .shadow_lg()
            .child(self.label.clone())
    }
}

/// 收集节点的全部子孙 id
fn descendants(node: &TreeNode) -> Vec<SharedString> {
    let mut out = Vec::new();
    for child in &node.children {
        out.push(child.id.clone());
        out.extend(descendants(child));
    }
    out
}

/// 查找目标节点的祖先链（根 → 父）
fn find_ancestors(nodes: &[TreeNode], target: &SharedString) -> Option<Vec<SharedString>> {
    for node in nodes {
        if node.id == *target {
            return Some(Vec::new());
        }
        if let Some(mut chain) = find_ancestors(&node.children, target) {
            chain.push(node.id.clone());
            return Some(chain);
        }
    }
    None
}

/// 在节点树中查找节点
fn find_node<'a>(nodes: &'a [TreeNode], id: &SharedString) -> Option<&'a TreeNode> {
    for node in nodes {
        if node.id == *id {
            return Some(node);
        }
        if let Some(found) = find_node(&node.children, id) {
            return Some(found);
        }
    }
    None
}

/// 从节点列表中递归移除并返回指定 id 的节点（拖拽重组用）
pub fn take_node(nodes: &mut Vec<TreeNode>, id: &str) -> Option<TreeNode> {
    if let Some(pos) = nodes.iter().position(|n| n.id == id) {
        return Some(nodes.remove(pos));
    }
    for node in nodes.iter_mut() {
        if let Some(found) = take_node(&mut node.children, id) {
            return Some(found);
        }
    }
    None
}

/// 递归查找指定 id 节点的可变引用（拖拽重组用）
pub fn find_node_mut<'a>(nodes: &'a mut [TreeNode], id: &str) -> Option<&'a mut TreeNode> {
    for node in nodes {
        if node.id == id {
            return Some(node);
        }
        if let Some(found) = find_node_mut(&mut node.children, id) {
            return Some(found);
        }
    }
    None
}

/// 勾选联动：勾选/取消含全部子孙；祖先链按「子节点全选则勾选」同步
fn cascade_check(
    nodes: &[TreeNode],
    checked: &[SharedString],
    id: &SharedString,
    check: bool,
) -> Vec<SharedString> {
    let mut set: Vec<SharedString> = checked.to_vec();
    let Some(node) = find_node(nodes, id) else {
        return set;
    };
    let mut affected = vec![id.clone()];
    affected.extend(descendants(node));
    if check {
        for a in &affected {
            if !set.contains(a) {
                set.push(a.clone());
            }
        }
    } else {
        set.retain(|s| !affected.contains(s));
    }
    // 自父向根同步祖先勾选态
    if let Some(chain) = find_ancestors(nodes, id) {
        for ancestor in chain.iter().rev() {
            let Some(a_node) = find_node(nodes, ancestor) else {
                continue;
            };
            let all_checked =
                !a_node.children.is_empty() && a_node.children.iter().all(|c| set.contains(&c.id));
            if all_checked {
                if !set.contains(ancestor) {
                    set.push(ancestor.clone());
                }
            } else {
                set.retain(|s| s != ancestor);
            }
        }
    }
    set.sort();
    set
}
