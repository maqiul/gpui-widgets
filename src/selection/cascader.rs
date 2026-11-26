//! 级联选择：多级菜单横向展开（对应 antd Cascader）。
//!
//! 触发框走 Select 四变体矩阵，浮层为多列菜单（列宽/列高/选项内边距见
//! [`crate::theme::metric::cascader_metric`]）。选中路径与展开状态受控。

use std::{cell::Cell, rc::Rc};

use gpui::{
    App, CursorStyle, Div, ElementId, IntoElement, Pixels, Role, SharedString, Stateful,
    StatefulInteractiveElement, Window, div, prelude::*, px,
};

use crate::text_input::{self, InputSize, InputStatus, InputVariant};
use crate::theme::{self, metric};
use crate::{Scrollbar, a11y};
use crate::{popup::anchored, select};

/// 级联节点
#[derive(Clone)]
pub struct CascaderNode {
    value: SharedString,
    label: SharedString,
    children: Vec<CascaderNode>,
    disabled: bool,
}

impl CascaderNode {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            children: Vec::new(),
            disabled: false,
        }
    }

    pub fn child(mut self, child: CascaderNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = CascaderNode>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// 级联选择器。
///
/// ```ignore
/// Cascader::new("region", options)
///     .value(self.region_path.clone())
///     .open(self.cascader_open)
///     .on_open_change(cx.listener(|this, open, _, cx| { this.cascader_open = *open; cx.notify(); }))
///     .on_change(cx.listener(|this, path, _, cx| { this.region_path = path.clone(); cx.notify(); }))
/// ```
#[derive(IntoElement)]
pub struct Cascader {
    id: ElementId,
    options: Vec<CascaderNode>,
    value: Vec<SharedString>,
    open: bool,
    placeholder: SharedString,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    disabled: bool,
    focused: Option<bool>,
    width: Option<Pixels>,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    on_change: Option<Rc<dyn Fn(&Vec<SharedString>, &mut Window, &mut App)>>,
}

impl Cascader {
    pub fn new(id: impl Into<ElementId>, options: Vec<CascaderNode>) -> Self {
        Self {
            id: id.into(),
            options,
            value: Vec::new(),
            open: false,
            placeholder: "请选择".into(),
            variant: InputVariant::Outlined,
            status: InputStatus::Default,
            size: InputSize::Middle,
            disabled: false,
            focused: None,
            width: None,
            on_open_change: None,
            on_change: None,
        }
    }

    /// 选中路径（受控）
    pub fn value(mut self, value: Vec<SharedString>) -> Self {
        self.value = value;
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

    /// 触发框宽度，默认 antd `controlWidth`（184）。
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn on_open_change(
        mut self,
        listener: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(listener));
        self
    }

    /// 路径变化时回调（点击中间层也会回调，叶子节点同时收起浮层）
    pub fn on_change(
        mut self,
        listener: impl Fn(&Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for Cascader {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open = self.open && !self.disabled;
        let focused = self.focused.unwrap_or(open);
        let suppress_toggle = Rc::new(Cell::new(false));

        let label = path_labels(&self.options, &self.value);
        let has_value = !label.is_empty();
        let label = if has_value {
            label.join(" / ")
        } else {
            self.placeholder.to_string()
        };

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
        let (gap, icon_color, icon_size, width) = theme::with(|t| {
            (
                t.padding_xxs,
                t.color_text_quaternary,
                t.font_size_icon,
                metric::cascader_metric(t).control_width,
            )
        });

        let mut trigger = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(gap))
            .w(self.width.unwrap_or(px(width)))
            .line_height(px(style.line_height))
            .text_size(px(style.font_size))
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
            if let Some(on_open_change) = on_open_change {
                trigger = a11y::click(trigger, move |_, window, cx| {
                    if suppress.take() {
                        return;
                    }
                    on_open_change(&!open, window, cx);
                });
            }
        }

        let popup = if open {
            Some(div().child(self.menu(&suppress_toggle, window, cx)))
        } else {
            None
        };
        anchored(trigger.into_any_element(), popup)
    }
}

impl Cascader {
    /// 多列横排菜单：列宽 controlItemWidth、列高 dropdownHeight，列间 colorSplit 分隔线。
    fn menu(
        &self,
        suppress_toggle: &Rc<Cell<bool>>,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let m = theme::with(metric::cascader_metric);
        let (bg, border) = theme::with(|t| {
            (
                metric::cascader_menu_bg(t),
                metric::cascader_column_border(t),
            )
        });
        let mut shell = div()
            .id(ElementId::Name(format!("{:?}-menu", self.id).into()))
            .occlude()
            .flex()
            .flex_row()
            .items_start()
            .rounded(px(m.panel_radius))
            .bg(bg)
            .shadow_lg();
        shell = a11y::Semantics::new(Role::ListBox, "级联选项")
            .id(format!("{:?}-menu", self.id))
            .apply(shell);
        shell = shell.on_mouse_down_out({
            let on_open_change = self.on_open_change.clone();
            let suppress_toggle = suppress_toggle.clone();
            move |_, window, cx| {
                suppress_toggle.set(true);
                if let Some(on_open_change) = &on_open_change {
                    on_open_change(&false, window, cx);
                }
            }
        });

        if self.options.is_empty() {
            let fg = theme::with(|t| t.color_text_disabled);
            let empty = div()
                .id(ElementId::Name(format!("{:?}-empty", self.id).into()))
                .min_w(px(m.item_min_width))
                .p(px(m.menu_pad))
                .child(
                    div()
                        .px(px(m.option_pad_inline))
                        .py(px(m.option_pad_block))
                        .text_size(px(m.font_size))
                        .line_height(px(m.line_height))
                        .text_color(fg)
                        .child("Not Found"),
                );
            return shell.child(
                a11y::Semantics::new(Role::ListItem, "Not Found")
                    .id(format!("{:?}-empty", self.id))
                    .disabled(true)
                    .apply(empty),
            );
        }

        // 逐列展开：第 0 列根节点，第 k 列为 value[k-1] 的子节点
        let mut level = &self.options;
        let mut depth = 0;
        loop {
            let scroll_id = format!("cascader-{:?}-{depth}", self.id);
            let handle = window
                .use_keyed_state(
                    ElementId::Name(format!("{scroll_id}-handle").into()),
                    cx,
                    |_, _| gpui::ScrollHandle::new(),
                )
                .read(cx)
                .clone();
            let mut column = div()
                .id(ElementId::Name(
                    format!("{:?}-col-{}", self.id, depth).into(),
                ))
                .flex()
                .flex_row()
                .min_w(px(m.item_min_width))
                .h(px(m.dropdown_height));
            if depth > 0 {
                column = column.border_l(px(m.column_border)).border_color(border);
            }
            let mut list = div()
                .id(ElementId::Name(format!("{scroll_id}-content").into()))
                .flex()
                .flex_col()
                .flex_1()
                .min_w(px(0.))
                .min_h(px(0.))
                .overflow_y_scroll()
                .p(px(m.menu_pad))
                .track_scroll(&handle);
            list = a11y::Semantics::new(Role::ListBox, format!("第 {} 层选项", depth + 1))
                .id(format!("{scroll_id}-content"))
                .apply(list);

            for (index, node) in level.iter().enumerate() {
                let selected = self.value.get(depth) == Some(&node.value);
                let has_children = !node.children.is_empty();
                let expanded = has_children && selected && depth + 1 < self.value.len();
                let disabled = node.disabled;
                let mut path = self.value[..depth.min(self.value.len())].to_vec();
                path.push(node.value.clone());

                let (idle, hover, icon) = theme::with(|t| {
                    (
                        metric::cascader_item_state(t, selected, false, disabled),
                        metric::cascader_item_state(t, selected, true, disabled),
                        metric::cascader_icon_color(t, disabled),
                    )
                });
                let mut item = div()
                    .id(ElementId::Name(
                        format!("{:?}-{}-{}", self.id, depth, node.value).into(),
                    ))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px(px(m.option_pad_inline))
                    .py(px(m.option_pad_block))
                    .rounded(px(m.item_radius))
                    .text_size(px(m.font_size))
                    .line_height(px(m.line_height))
                    .text_color(idle.fg)
                    .font_weight(idle.font_weight)
                    .bg(idle.bg)
                    .cursor(if disabled {
                        CursorStyle::OperationNotAllowed
                    } else {
                        CursorStyle::PointingHand
                    })
                    .hover(move |style| style.bg(hover.bg).text_color(hover.fg))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .child(node.label.clone()),
                    );
                if has_children {
                    item = item.child(
                        div()
                            .ml(px(m.icon_gap))
                            .text_size(px(m.icon_font))
                            .text_color(icon)
                            .child("›"),
                    );
                }

                item = a11y::Semantics::new(Role::ListBoxOption, node.label.clone())
                    .id(format!("{:?}-{depth}-{}-option", self.id, node.value))
                    .selected(selected)
                    .expanded(expanded)
                    .disabled(disabled)
                    .focusable(!disabled)
                    .position(index + 1, level.len())
                    .level(depth + 1)
                    .apply(item);
                if !disabled {
                    let on_change = self.on_change.clone();
                    let on_open_change = self.on_open_change.clone();
                    let suppress = suppress_toggle.clone();
                    item = a11y::click_void(item, move |window, cx| {
                        suppress.set(true);
                        if let Some(on_change) = &on_change {
                            on_change(&path, window, cx);
                        }
                        if !has_children {
                            if let Some(on_open_change) = &on_open_change {
                                on_open_change(&false, window, cx);
                            }
                        }
                    });
                }
                list = list.child(item);
            }
            column = column.child(list).child(Scrollbar::vertical(
                ElementId::Name(format!("{scroll_id}-scrollbar").into()),
                handle,
            ));
            shell = shell.child(column);

            match self
                .value
                .get(depth)
                .and_then(|v| level.iter().find(|n| &n.value == v))
            {
                Some(node) if !node.children.is_empty() => {
                    level = &node.children;
                    depth += 1;
                }
                _ => break,
            }
        }
        shell
    }
}

/// 沿路径收集 label
fn path_labels(options: &[CascaderNode], path: &[SharedString]) -> Vec<String> {
    let mut labels = Vec::new();
    let mut level = options;
    for value in path {
        match level.iter().find(|n| &n.value == value) {
            Some(node) => {
                labels.push(node.label.to_string());
                level = &node.children;
            }
            None => break,
        }
    }
    labels
}
