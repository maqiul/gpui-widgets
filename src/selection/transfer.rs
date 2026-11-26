//! 穿梭框：双列表 + 勾选 + 左右移动（对应 antd Transfer）。
//!
//! 目标列表与勾选状态受控，状态由父视图持有。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, IntoElement, Pixels, Role, SharedString, Stateful,
    StatefulInteractiveElement, Toggled, Window, div, prelude::*, px,
};

use crate::theme::{
    self,
    metric::{TransferMetric, TransferStyle, transfer_style},
};
use crate::{Scrollbar, a11y};

/// 穿梭框条目
#[derive(Clone)]
pub struct TransferItem {
    key: SharedString,
    label: SharedString,
    disabled: bool,
}

impl TransferItem {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        Self { disabled, ..self }
    }
}

/// 穿梭框。
///
/// ```ignore
/// Transfer::new("transfer", items)
///     .target_keys(self.transfer_right.clone())
///     .selected(self.transfer_checked.clone())
///     .on_change(cx.listener(|this, keys, _, cx| { this.transfer_right = keys.clone(); cx.notify(); }))
///     .on_select(cx.listener(|this, keys, _, cx| { this.transfer_checked = keys.clone(); cx.notify(); }))
/// ```
#[derive(IntoElement)]
pub struct Transfer {
    id: ElementId,
    items: Vec<TransferItem>,
    target_keys: Vec<SharedString>,
    selected: Vec<SharedString>,
    titles: (SharedString, SharedString),
    list_width: Option<Pixels>,
    list_height: Option<Pixels>,
    on_change: Option<Rc<dyn Fn(&Vec<SharedString>, &mut Window, &mut App)>>,
    on_select: Option<Rc<dyn Fn(&Vec<SharedString>, &mut Window, &mut App)>>,
}

impl Transfer {
    pub fn new(id: impl Into<ElementId>, items: Vec<TransferItem>) -> Self {
        Self {
            id: id.into(),
            items,
            target_keys: Vec::new(),
            selected: Vec::new(),
            titles: ("源列表".into(), "目标列表".into()),
            list_width: None,
            list_height: None,
            on_change: None,
            on_select: None,
        }
    }

    /// 右侧列表的 key 集合（受控）
    pub fn target_keys(mut self, target_keys: Vec<SharedString>) -> Self {
        self.target_keys = target_keys;
        self
    }

    /// 当前勾选的 key 集合（受控）
    pub fn selected(mut self, selected: Vec<SharedString>) -> Self {
        self.selected = selected;
        self
    }

    /// 两侧列表标题
    pub fn titles(mut self, left: impl Into<SharedString>, right: impl Into<SharedString>) -> Self {
        self.titles = (left.into(), right.into());
        self
    }

    /// 单侧列表宽度（默认令牌 180，对应 antd 组件令牌 listWidth）
    pub fn list_width(mut self, width: Pixels) -> Self {
        self.list_width = Some(width);
        self
    }

    /// 单侧列表高度（默认令牌 200，对应 antd 组件令牌 listHeight）
    pub fn list_height(mut self, height: Pixels) -> Self {
        self.list_height = Some(height);
        self
    }

    /// 目标列表变化时回调新的 key 集合
    pub fn on_change(
        mut self,
        listener: impl Fn(&Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    /// 勾选变化时回调新的 key 集合
    pub fn on_select(
        mut self,
        listener: impl Fn(&Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for Transfer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let m = theme::with(TransferMetric::resolve);
        let style = theme::with(transfer_style);
        let left: Vec<&TransferItem> = self
            .items
            .iter()
            .filter(|i| !self.target_keys.contains(&i.key))
            .collect();
        let right: Vec<&TransferItem> = self
            .items
            .iter()
            .filter(|i| self.target_keys.contains(&i.key))
            .collect();

        // 可移动判定
        let can_right = left
            .iter()
            .any(|i| !i.disabled && self.selected.contains(&i.key));
        let can_left = right
            .iter()
            .any(|i| !i.disabled && self.selected.contains(&i.key));

        let mut root = div().id(self.id.clone()).flex().flex_row().items_center();
        root = a11y::Semantics::new(Role::Group, "穿梭框")
            .id(format!("{:?}", self.id))
            .apply(root);

        root = root.child(self.panel("left", &self.titles.0, &left, &m, &style, window, cx));

        // 中间移动按钮
        let mut buttons = div()
            .flex()
            .flex_col()
            .justify_center()
            .mx(px(m.actions_margin))
            .gap(px(m.actions_gap));
        buttons = buttons.child(move_button(
            ElementId::Name(format!("{:?}-to-right", self.id).into()),
            "→",
            "移至目标列表",
            can_right,
            &m,
            &style,
            {
                let on_change = self.on_change.clone();
                let on_select = self.on_select.clone();
                let selected = self.selected.clone();
                let target = self.target_keys.clone();
                let left_keys: Vec<SharedString> = left
                    .iter()
                    .filter(|i| !i.disabled && selected.contains(&i.key))
                    .map(|i| i.key.clone())
                    .collect();
                move |window, cx| {
                    let mut next = target.clone();
                    next.extend(left_keys.iter().cloned());
                    let remaining: Vec<SharedString> = selected
                        .iter()
                        .filter(|k| !left_keys.contains(k))
                        .cloned()
                        .collect();
                    if let Some(on_change) = &on_change {
                        on_change(&next, window, cx);
                    }
                    if let Some(on_select) = &on_select {
                        on_select(&remaining, window, cx);
                    }
                }
            },
        ));
        buttons = buttons.child(move_button(
            ElementId::Name(format!("{:?}-to-left", self.id).into()),
            "←",
            "移至源列表",
            can_left,
            &m,
            &style,
            {
                let on_change = self.on_change.clone();
                let on_select = self.on_select.clone();
                let selected = self.selected.clone();
                let right_keys: Vec<SharedString> = right
                    .iter()
                    .filter(|i| !i.disabled && selected.contains(&i.key))
                    .map(|i| i.key.clone())
                    .collect();
                let target: Vec<SharedString> = self
                    .target_keys
                    .iter()
                    .filter(|k| !right_keys.contains(k))
                    .cloned()
                    .collect();
                move |window, cx| {
                    let remaining: Vec<SharedString> = selected
                        .iter()
                        .filter(|k| !right_keys.contains(k))
                        .cloned()
                        .collect();
                    if let Some(on_change) = &on_change {
                        on_change(&target, window, cx);
                    }
                    if let Some(on_select) = &on_select {
                        on_select(&remaining, window, cx);
                    }
                }
            },
        ));
        root = root.child(buttons);

        root = root.child(self.panel("right", &self.titles.1, &right, &m, &style, window, cx));
        root
    }
}

impl Transfer {
    /// 单侧面板
    fn panel(
        &self,
        side: &str,
        title: &SharedString,
        items: &[&TransferItem],
        m: &TransferMetric,
        style: &TransferStyle,
        window: &mut Window,
        cx: &mut App,
    ) -> Div {
        let enabled: Vec<&&TransferItem> = items.iter().filter(|i| !i.disabled).collect();
        let checked_enabled = enabled
            .iter()
            .filter(|i| self.selected.contains(&i.key))
            .count();
        let all_checked = !enabled.is_empty() && checked_enabled == enabled.len();
        let part_checked = checked_enabled > 0 && !all_checked;

        let mut panel = div()
            .flex()
            .flex_col()
            .w(self.list_width.unwrap_or(px(m.list_width)))
            .h(self.list_height.unwrap_or(px(m.list_height)))
            .rounded(px(m.radius))
            .border_1()
            .border_color(style.border)
            .bg(style.header_bg)
            .overflow_hidden();

        // 头部：全选勾选框 + 计数 + 右对齐标题
        let mut header = div()
            .id(ElementId::Name(
                format!("{:?}-{side}-header", self.id).into(),
            ))
            .flex()
            .flex_row()
            .items_center()
            .flex_shrink_0()
            .h(px(m.header_height))
            .pt(px(m.header_padding_top))
            .pb(px(m.header_padding_bottom))
            .px(px(m.header_padding_inline))
            .gap(px(m.header_gap))
            .border_b_1()
            .border_color(style.split)
            .bg(style.header_bg)
            .text_size(px(m.font_size))
            .line_height(px(m.line_height))
            .text_color(style.header_text)
            .child(check_box(
                m,
                style,
                all_checked,
                part_checked,
                enabled.is_empty(),
            ))
            .child(
                div()
                    .text_size(px(m.icon_size))
                    .line_height(px(m.icon_size))
                    .child("∨"),
            )
            .child(div().child(format!(
                "{}{} {}",
                if checked_enabled > 0 {
                    format!("{checked_enabled}/")
                } else {
                    String::new()
                },
                items.len(),
                if items.len() == 1 { "item" } else { "items" }
            )));
        header = header.child(div().ml_auto().child(title.clone()));
        header = a11y::Semantics::new(Role::CheckBox, format!("全选 {title}"))
            .id(format!("{:?}-{side}-header", self.id))
            .toggled(if part_checked {
                Toggled::Mixed
            } else if all_checked {
                Toggled::True
            } else {
                Toggled::False
            })
            .disabled(enabled.is_empty())
            .read_only(true)
            .apply(header);
        panel = panel.child(header);

        // 条目列表
        let scroll_id = format!("transfer-{:?}-{side}", self.id);
        let handle = window
            .use_keyed_state(
                ElementId::Name(format!("{scroll_id}-handle").into()),
                cx,
                |_, _| gpui::ScrollHandle::new(),
            )
            .read(cx)
            .clone();
        let mut list = div()
            .id(ElementId::Name(
                format!("{:?}-list-{}", self.id, side).into(),
            ))
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.))
            .min_h(px(0.))
            .overflow_y_scroll()
            .track_scroll(&handle);
        list = a11y::Semantics::new(Role::ListBox, title.clone())
            .id(format!("{:?}-list-{side}", self.id))
            .apply(list);
        for (index, item) in items.iter().enumerate() {
            let checked = self.selected.contains(&item.key);
            let key = item.key.clone();
            let disabled = item.disabled;

            let mut row = div()
                .id(ElementId::Name(
                    format!("{:?}-{}-{}", self.id, side, key).into(),
                ))
                .flex()
                .flex_row()
                .items_center()
                .flex_shrink_0()
                .min_h(px(m.item_height))
                .py(px(m.item_padding_block))
                .px(px(m.item_padding_inline))
                .gap(px(m.item_gap))
                .text_size(px(m.font_size))
                .line_height(px(m.line_height));
            if checked && !disabled {
                row = row.bg(style.item_checked);
            }

            row = row.child(check_box(m, style, checked, false, disabled));

            if disabled {
                row = row.text_color(style.disabled);
            } else {
                let hover_bg = if checked {
                    style.item_checked_hover
                } else {
                    style.item_hover
                };
                row = row
                    .text_color(style.header_text)
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |s| s.bg(hover_bg));
                let on_select = self.on_select.clone();
                let selected = self.selected.clone();
                row = a11y::Semantics::new(Role::ListBoxOption, item.label.clone())
                    .id(format!("{:?}-{side}-{}-option", self.id, key))
                    .selected(checked)
                    .toggled(if checked {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .disabled(disabled)
                    .focusable(!disabled)
                    .position(index + 1, items.len())
                    .apply(row);
                row = a11y::click_void(row, move |window, cx| {
                    let mut next = selected.clone();
                    if let Some(pos) = next.iter().position(|k| k == &key) {
                        next.remove(pos);
                    } else {
                        next.push(key.clone());
                    }
                    if let Some(on_select) = &on_select {
                        on_select(&next, window, cx);
                    }
                });
            }
            row = row.child(div().flex_1().child(item.label.clone()));
            list = list.child(row);
        }
        panel.child(
            div()
                .flex()
                .flex_row()
                .flex_1()
                .min_w(px(0.))
                .min_h(px(0.))
                .child(list)
                .child(Scrollbar::vertical(
                    ElementId::Name(format!("{scroll_id}-scrollbar").into()),
                    handle,
                )),
        )
    }
}

/// 勾选框（含半选态）
fn check_box(
    m: &TransferMetric,
    style: &TransferStyle,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
) -> Div {
    let mut box_div = div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(m.checkbox_size))
        .flex_shrink_0()
        .rounded(px(m.checkbox_radius))
        .border_1();
    if disabled {
        box_div = box_div.border_color(style.border).bg(style.disabled_bg);
        if indeterminate {
            box_div = box_div.child(div().size(px(m.checkbox_size / 2.)).bg(style.disabled));
        } else if checked {
            box_div = box_div.child(
                div()
                    .text_size(px(m.font_size))
                    .text_color(style.disabled)
                    .child("✓"),
            );
        }
    } else if indeterminate {
        // antd 半选：容器底 + 主色描边 + 主色实心小方块（不是主色底白杠）
        box_div = box_div
            .border_color(style.checkbox_bg)
            .bg(style.header_bg)
            .child(div().size(px(m.checkbox_size / 2.)).bg(style.checkbox_bg));
    } else if checked {
        box_div = box_div
            .border_color(style.checkbox_bg)
            .bg(style.checkbox_bg)
            .child(
                div()
                    .text_size(px(m.font_size))
                    .text_color(style.check_fg)
                    .child("✓"),
            );
    } else {
        box_div = box_div
            .border_color(style.checkbox_border)
            .bg(style.header_bg);
    }
    box_div
}

/// 移动按钮
fn move_button(
    id: ElementId,
    label: &'static str,
    accessible_label: &'static str,
    enabled: bool,
    m: &TransferMetric,
    style: &TransferStyle,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let button = div()
        .id(id.clone())
        .flex()
        .items_center()
        .justify_center()
        .size(px(m.button_size))
        .flex_shrink_0()
        .rounded(px(m.button_radius))
        .border_1()
        .text_size(px(m.font_size));
    let mut button = a11y::Semantics::new(Role::Button, accessible_label)
        .id(format!("{id:?}"))
        .disabled(!enabled)
        .focusable(enabled)
        .apply(button);
    if enabled {
        let hover = style.button_hover;
        button = button
            .border_color(style.button_bg)
            .bg(style.button_bg)
            .text_color(style.button_fg)
            .cursor(CursorStyle::PointingHand)
            .hover(move |s| s.bg(hover).border_color(hover));
        button = a11y::click_void(button, on_click);
    } else {
        button = button
            .border_color(style.border)
            .bg(style.disabled_bg)
            .text_color(style.disabled);
    }
    button.child(label)
}
