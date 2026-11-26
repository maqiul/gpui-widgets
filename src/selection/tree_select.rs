//! 树选择：下拉浮层内嵌树形控件（对应 antd TreeSelect）。
//!
//! 触发框走 Select 四变体矩阵；浮层内边距 `paddingXS × paddingXS/2`、底 colorBgElevated，
//! 内部树沿用已迁移的 [`Tree`]。选中 / 展开 / 浮层状态均受控。

use std::{cell::Cell, rc::Rc};

use gpui::{
    App, CursorStyle, ElementId, IntoElement, Role, SharedString, Window, div, prelude::*, px,
};

use crate::text_input::{self, InputSize, InputStatus, InputVariant};
use crate::theme::{self, metric};
use crate::{
    a11y,
    popup::anchored,
    select,
    tree::{Tree, TreeNode},
};

/// 树选择器。
///
/// ```ignore
/// TreeSelect::new("ts", nodes)
///     .value(self.ts_value.clone())
///     .expanded(self.ts_expanded.clone())
///     .open(self.ts_open)
///     .on_open_change(cx.listener(|this, open, _, cx| { this.ts_open = *open; cx.notify(); }))
///     .on_change(cx.listener(|this, id, _, cx| { this.ts_value = Some(id.clone()); cx.notify(); }))
///     .on_toggle(cx.listener(|this, id, _, cx| { /* 切换展开 */ }))
/// ```
#[derive(IntoElement)]
pub struct TreeSelect {
    id: ElementId,
    nodes: Vec<TreeNode>,
    value: Option<SharedString>,
    expanded: Vec<SharedString>,
    open: bool,
    placeholder: SharedString,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    disabled: bool,
    focused: Option<bool>,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    on_change: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_toggle: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
}

impl TreeSelect {
    pub fn new(id: impl Into<ElementId>, nodes: Vec<TreeNode>) -> Self {
        Self {
            id: id.into(),
            nodes,
            value: None,
            expanded: Vec::new(),
            open: false,
            placeholder: "请选择".into(),
            variant: InputVariant::Outlined,
            status: InputStatus::Default,
            size: InputSize::Middle,
            disabled: false,
            focused: None,
            on_open_change: None,
            on_change: None,
            on_toggle: None,
        }
    }

    /// 选中节点 id（受控）
    pub fn value(mut self, value: Option<SharedString>) -> Self {
        self.value = value;
        self
    }

    /// 展开节点 id 集合（受控）
    pub fn expanded(mut self, expanded: Vec<SharedString>) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn status(mut self, status: InputStatus) -> Self {
        self.status = status;
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 触发框聚焦态，默认跟随 open（点击展开即聚焦）。
    /// 受控 open 而未聚焦时（antd `open` prop 不带焦点）显式传 false。
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = Some(focused);
        self
    }

    pub fn on_open_change(
        mut self,
        listener: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(listener));
        self
    }

    /// 点击节点时回调其 id（同时收起浮层）
    pub fn on_change(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(listener));
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
}

impl RenderOnce for TreeSelect {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let open = self.open && !self.disabled;
        let focused = self.focused.unwrap_or(open);
        let suppress_toggle = Rc::new(Cell::new(false));

        let has_value = self.value.is_some();
        let label = self
            .value
            .as_ref()
            .and_then(|v| find_label(&self.nodes, v))
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.placeholder.to_string());

        let style = theme::with(|t| {
            select::state(
                self.variant,
                self.status,
                self.size,
                t,
                false,
                focused,
                self.disabled,
            )
        });
        let hover_style = theme::with(|t| {
            select::state(
                self.variant,
                self.status,
                self.size,
                t,
                true,
                false,
                self.disabled,
            )
        });
        let (gap, icon_color, icon_size) =
            theme::with(|t| (t.padding_xxs, t.color_text_quaternary, t.font_size_icon));

        let mut trigger = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(gap))
            .w_full()
            .line_height(px(style.line_height))
            .text_size(px(style.font_size))
            // antd 在 Windows 首选 Segoe UI；gpui 默认取系统 UI 字体（中文系统
            // 为 Microsoft YaHei），拉丁文更宽会把取值与箭头挤歪
            .font_family("Segoe UI")
            .text_color(style.fg)
            .px(px(style.padding_inline))
            .py(px(style.padding_block))
            .child(
                div()
                    .overflow_hidden()
                    .opacity(if open && has_value {
                        metric::select_content_open_opacity()
                    } else {
                        1.0
                    })
                    .text_color(if has_value {
                        style.fg
                    } else {
                        style.placeholder
                    })
                    .child(label.clone()),
            )
            .child(
                div()
                    .text_size(px(icon_size))
                    .text_color(icon_color)
                    .child("▾"),
            );
        trigger = text_input::apply_frame(trigger, &style);
        trigger = text_input::hover_frame(trigger, &hover_style, open);
        let trigger = trigger.id(self.id.clone());
        let mut semantics = a11y::Semantics::new(Role::ComboBox, label.clone())
            .id(format!("{:?}-trigger", self.id))
            .expanded(open)
            .disabled(self.disabled)
            .focusable(!self.disabled);
        if has_value {
            semantics = semantics.value(label);
        }
        let mut trigger = semantics.apply(trigger);

        if self.disabled {
            trigger = trigger.cursor(CursorStyle::Arrow);
        } else {
            let on_open_change = self.on_open_change.clone();
            let suppress = suppress_toggle.clone();
            trigger = trigger.cursor(CursorStyle::PointingHand);
            trigger = a11y::click_void(trigger, move |window, cx| {
                if suppress.take() {
                    return;
                }
                if let Some(on_open_change) = &on_open_change {
                    on_open_change(&!open, window, cx);
                }
            });
        }

        let popup = if open {
            let m = theme::with(metric::tree_select_metric);
            let bg = theme::with(metric::tree_select_menu_bg);
            let suppress = suppress_toggle.clone();
            let on_open_change = self.on_open_change.clone();
            let on_change = self.on_change.clone();
            let on_toggle = self.on_toggle.clone();
            let tree = Tree::new(
                ElementId::Name(format!("{:?}-tree", self.id).into()),
                self.nodes,
            )
            // TreeSelect 内部树是 antd blockNode：节点底铺满行宽、行距 4、箭头右距 4
            .block_node(true)
            .expanded(self.expanded)
            .selected(self.value)
            .on_toggle(move |id, window, cx| {
                suppress.set(true);
                if let Some(on_toggle) = &on_toggle {
                    on_toggle(id, window, cx);
                }
            })
            .on_select({
                let suppress = suppress_toggle.clone();
                let on_open_change = on_open_change.clone();
                move |id, window, cx| {
                    suppress.set(true);
                    if let Some(on_change) = &on_change {
                        on_change(id, window, cx);
                    }
                    if let Some(on_open_change) = &on_open_change {
                        on_open_change(&false, window, cx);
                    }
                }
            });

            Some(
                div().child(
                    div()
                        .id(ElementId::Name(format!("{:?}-shell", self.id).into()))
                        .occlude()
                        .w_full()
                        .py(px(m.pad_block))
                        .px(px(m.pad_inline))
                        .rounded(px(m.radius))
                        .font_family("Segoe UI")
                        .bg(bg)
                        .shadow_lg()
                        .on_mouse_down_out({
                            let on_open_change = on_open_change.clone();
                            let suppress = suppress_toggle.clone();
                            move |_, window, cx| {
                                suppress.set(true);
                                if let Some(on_open_change) = &on_open_change {
                                    on_open_change(&false, window, cx);
                                }
                            }
                        })
                        .child(tree),
                ),
            )
        } else {
            None
        };
        anchored(trigger.into_any_element(), popup)
    }
}

/// 递归查找节点 label
fn find_label(nodes: &[TreeNode], id: &SharedString) -> Option<SharedString> {
    for node in nodes {
        if node.id() == id {
            return Some(node.label().clone());
        }
        if let Some(found) = find_label(node.child_nodes(), id) {
            return Some(found);
        }
    }
    None
}
