//! 导航菜单：垂直多级菜单（对应 antd Menu 的 inline 模式）。
//!
//! 选中 / 展开状态受控，状态由父视图持有。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, IntoElement, Orientation, Role, SharedString, Window, div,
    prelude::*, px,
};

use crate::a11y;
use crate::theme;

/// 菜单条目（key 用于选中 / 展开标识，同一菜单内需唯一）
pub struct MenuEntry {
    key: SharedString,
    label: SharedString,
    icon: Option<SharedString>,
    disabled: bool,
    children: Vec<MenuEntry>,
}

impl MenuEntry {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            children: Vec::new(),
        }
    }

    /// 前置图标（字符 / emoji）
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn child(mut self, child: MenuEntry) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = MenuEntry>) -> Self {
        self.children.extend(children);
        self
    }
}

/// 垂直导航菜单。
///
/// ```ignore
/// NavMenu::new("nav", vec![
///     MenuEntry::new("home", "首页").icon("⌂"),
///     MenuEntry::new("settings", "设置").child(MenuEntry::new("profile", "个人资料")),
/// ])
/// .selected(self.nav_selected.clone())
/// .on_select(cx.listener(|this, key, _, cx| { this.nav_selected = Some(key.clone()); cx.notify(); }))
/// .expanded(self.nav_expanded.clone())
/// .on_toggle(cx.listener(|this, key, _, cx| { /* 切换 key 展开 */ }))
/// ```
#[derive(IntoElement)]
pub struct NavMenu {
    id: ElementId,
    entries: Vec<MenuEntry>,
    selected: Option<SharedString>,
    expanded: Vec<SharedString>,
    on_select: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_toggle: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
}

impl NavMenu {
    pub fn new(id: impl Into<ElementId>, entries: Vec<MenuEntry>) -> Self {
        Self {
            id: id.into(),
            entries,
            selected: None,
            expanded: Vec::new(),
            on_select: None,
            on_toggle: None,
        }
    }

    /// 当前选中条目 key（受控）
    pub fn selected(mut self, selected: Option<SharedString>) -> Self {
        self.selected = selected;
        self
    }

    /// 当前展开的父条目 key 集合（受控）
    pub fn expanded(mut self, expanded: Vec<SharedString>) -> Self {
        self.expanded = expanded;
        self
    }

    /// 点击叶子条目时回调其 key
    pub fn on_select(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }

    /// 点击父条目时回调其 key（由调用方切换展开状态）
    pub fn on_toggle(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for NavMenu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let m = theme::with(crate::theme::metric::nav_menu_metric);
        let mut rows = Vec::new();
        let siblings = self.entries.len();
        for (ix, entry) in self.entries.iter().enumerate() {
            rows.push(entry_row(entry, 0, ix, siblings, "", &self, &m));
        }
        a11y::Semantics::new(Role::Tree, "导航菜单")
            .id(format!("{:?}-tree", self.id))
            .orientation(Orientation::Vertical)
            .apply(div().id(self.id.clone()))
            .flex()
            .flex_col()
            .w_full()
            .border_r(px(m.border_width))
            .border_color(m.border_color)
            .text_size(px(m.font_size))
            .children(rows)
    }
}

/// 后代中是否有选中 key（父级标题字色用）
fn contains_selected(entry: &MenuEntry, selected: &SharedString) -> bool {
    entry.key == *selected
        || entry
            .children
            .iter()
            .any(|c| contains_selected(c, selected))
}

/// 递归构建：行 + 展开时的子列表（子列表底 colorFillAlter，对应 ant-menu-sub）
fn entry_row(
    entry: &MenuEntry,
    depth: usize,
    index: usize,
    siblings: usize,
    path: &str,
    menu: &NavMenu,
    m: &crate::theme::metric::NavMenuMetric,
) -> Div {
    let has_children = !entry.children.is_empty();
    let is_expanded = menu.expanded.contains(&entry.key);
    let is_selected = menu.selected.as_ref() == Some(&entry.key);
    let level = depth as f32 + 1.0;

    let (state, hover_bg) = theme::with(|t| {
        let state = if has_children {
            crate::theme::metric::nav_menu_parent_state(
                t,
                menu.selected
                    .as_ref()
                    .is_some_and(|s| contains_selected(entry, s)),
                false,
                entry.disabled,
            )
        } else {
            crate::theme::metric::nav_menu_item_state(t, is_selected, false, entry.disabled)
        };
        (state, t.color_bg_text_hover)
    });

    let item_path = format!("{path}-{index}");
    let row_id = format!("{:?}-{}-{}", menu.id, entry.key, item_path);
    let mut semantics = a11y::Semantics::new(Role::TreeItem, entry.label.clone())
        .id(row_id.clone())
        .selected(is_selected)
        .disabled(entry.disabled)
        .focusable(
            !entry.disabled
                && if has_children {
                    menu.on_toggle.is_some()
                } else {
                    menu.on_select.is_some()
                },
        )
        .position(index + 1, siblings)
        .level(depth + 1);
    if has_children {
        semantics = semantics.expanded(is_expanded);
    }
    let mut row = semantics
        .apply(div().id(ElementId::Name(row_id.into())))
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .h(px(m.item_height))
        .mx(px(m.item_margin_inline))
        // antd 的相邻条目 marginBlock 会折叠成一个 4（44 的行距）；gpui 的
        // 兄弟 margin 不折叠，这里只留上边距复现折叠后的结果
        .mt(px(m.item_margin_block))
        .pl(px(m.inline_indent * level))
        .pr(px(if has_children {
            m.title_padding_end_with_arrow
        } else {
            m.item_padding_inline
        }))
        .rounded(px(if depth == 0 {
            m.item_radius
        } else {
            m.sub_item_radius
        }))
        .bg(state.bg)
        .text_color(state.fg);

    if !entry.disabled {
        row = row.cursor(CursorStyle::PointingHand);
        row = row.hover(move |style| style.bg(hover_bg));

        let key = entry.key.clone();
        if has_children {
            let on_toggle = menu.on_toggle.clone();
            if let Some(on_toggle) = on_toggle {
                row = a11y::click_void(row, move |window, cx| on_toggle(&key, window, cx));
            }
        } else {
            let on_select = menu.on_select.clone();
            if let Some(on_select) = on_select {
                row = a11y::click_void(row, move |window, cx| on_select(&key, window, cx));
            }
        }
    }

    if let Some(icon) = &entry.icon {
        row = row.child(
            div()
                .min_w(px(m.icon_size))
                .text_size(px(m.icon_size))
                .text_center()
                .child(icon.clone()),
        );
    }
    row = row.child(
        div()
            .flex_1()
            .min_w_0()
            .ml(px(if entry.icon.is_some() {
                m.icon_margin_inline_end
            } else {
                0.
            }))
            .child(entry.label.clone()),
    );
    if has_children {
        row = row.child(
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .right(px(m.arrow_inset_end))
                .flex()
                .items_center()
                .text_size(px(m.arrow_size))
                .child(if is_expanded { "▾" } else { "▸" }),
        );
    }

    let mut col = div().flex().flex_col().child(row);
    if has_children && is_expanded {
        let siblings = entry.children.len();
        let children: Vec<Div> = entry
            .children
            .iter()
            .enumerate()
            .map(|(child_ix, child)| {
                entry_row(child, depth + 1, child_ix, siblings, &item_path, menu, m)
            })
            .collect();
        col = col.child(div().flex().flex_col().bg(m.sub_item_bg).children(children));
    }
    col
}
