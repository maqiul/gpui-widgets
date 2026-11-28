//! 下拉动作菜单：任意触发元素 + 动作项列表（区别于 Select 的“选值”语义）。
//!
//! 展开状态受控（`open` + `on_open_change`），点击菜单外部或动作项后收起。

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use gpui::{
    AnyElement, App, Bounds, CursorStyle, Div, ElementId, FocusHandle, Hsla, IntoElement, Pixels,
    Point, RenderOnce, Role, SharedString, Stateful, Window, div, prelude::*, px,
    transparent_black,
};

use crate::a11y;
use crate::popup::{PopupBoundsSink, PopupPlacement, anchored, anchored_with_placement_and_bounds};
use crate::theme::{self, Tokens, metric};

/// 菜单容器外观（antd Dropdown / ContextMenu 共用，浏览器 computed style 实测）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuStyle {
    pub bg: Hsla,
    pub radius: f32,
    pub padding: f32,
}

/// 菜单动作项外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuItemStyle {
    pub bg: Hsla,
    pub fg: Hsla,
    pub hover_bg: Hsla,
    pub height: f32,
    pub padding_block: f32,
    pub padding_inline: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
}

/// 浮层菜单：`colorBgElevated` + `borderRadiusLG` + `paddingXXS`，无描边。
pub fn menu_state(tokens: &Tokens) -> MenuStyle {
    let m = metric::SelectMetric::resolve(tokens);
    MenuStyle {
        bg: tokens.color_bg_elevated,
        radius: m.menu_radius,
        padding: m.menu_padding,
    }
}

/// 分隔线：`colorSplit`，宽 `lineWidth`，上下外边距 `paddingXXS`（浏览器实测 4px）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuDividerStyle {
    pub color: Hsla,
    pub width: f32,
    pub margin: f32,
}

pub fn divider_state(tokens: &Tokens) -> MenuDividerStyle {
    let m = metric::SelectMetric::resolve(tokens);
    MenuDividerStyle {
        color: tokens.color_split,
        width: m.line_width,
        margin: m.menu_padding,
    }
}

/// 动作项：高度 / 内边距与 Select 选项同源；危险项取 `colorError`，禁用项取 `colorTextDisabled`
/// 且不改透明度；悬停底色 `controlItemBgHover`。
pub fn item_state(tokens: &Tokens, danger: bool, disabled: bool) -> MenuItemStyle {
    let m = metric::SelectMetric::resolve(tokens);
    MenuItemStyle {
        bg: transparent_black(),
        fg: if disabled {
            tokens.color_text_disabled
        } else if danger {
            tokens.color_error
        } else {
            tokens.color_text
        },
        hover_bg: tokens.control_item_bg_hover,
        height: m.option_height,
        padding_block: m.option_padding_block,
        padding_inline: m.option_padding_inline,
        radius: tokens.border_radius_sm,
        font_size: tokens.font_size,
        line_height: tokens.font_height,
    }
}

/// 浮层菜单容器；调用方负责宽度与 `on_mouse_down_out`。
pub(crate) fn menu_container(id: ElementId) -> Stateful<Div> {
    let style = theme::with(|t| menu_state(t));
    let semantics_id = format!("{id:?}-menu");
    a11y::Semantics::new(Role::Menu, "操作菜单")
        .id(semantics_id.clone())
        .apply(div().id(id).accessibility_id(semantics_id))
        .occlude()
        .flex()
        .flex_col()
        .p(px(style.padding))
        .rounded(px(style.radius))
        .bg(style.bg)
        .shadow_lg()
}

fn menu_divider_with_id(id: ElementId) -> Stateful<Div> {
    let style = theme::with(|t| divider_state(t));
    div()
        .id(id)
        .h(px(style.width))
        .my(px(style.margin))
        .bg(style.color)
}

fn menu_item_row_at(
    id: ElementId,
    item: &MenuItem,
    close: Rc<dyn Fn(&mut Window, &mut App)>,
    set_position: Option<(usize, usize)>,
) -> Stateful<Div> {
    let style = theme::with(|t| item_state(t, item.danger, item.disabled));
    let semantics_id = format!("{id:?}-menu-item");
    let mut semantics = a11y::Semantics::new(Role::MenuItem, item.label.clone())
        .id(semantics_id.clone())
        .disabled(item.disabled)
        .focusable(!item.disabled);
    if let Some((position, size)) = set_position {
        semantics = semantics.position(position, size);
    }
    let mut row = semantics
        .apply(div().id(id).accessibility_id(semantics_id))
        .flex()
        .items_center()
        .min_h(px(style.height))
        .px(px(style.padding_inline))
        .py(px(style.padding_block))
        .rounded(px(style.radius))
        .text_size(px(style.font_size))
        .line_height(px(style.line_height))
        .text_color(style.fg)
        .bg(style.bg)
        .child(item.label.clone());
    if item.disabled {
        row = row.cursor(CursorStyle::Arrow);
    } else {
        let hover_bg = style.hover_bg;
        let on_click = item.on_click.clone();
        row = row
            .cursor(CursorStyle::PointingHand)
            .hover(move |style| style.bg(hover_bg));
        row = a11y::click_void(row, move |window, cx| {
            if let Some(on_click) = &on_click {
                on_click(window, cx);
            }
            close(window, cx);
        });
    }
    row
}

/// 菜单动作项（或分隔线）
#[derive(Clone)]
pub struct MenuItem {
    pub(crate) label: SharedString,
    pub(crate) danger: bool,
    pub(crate) disabled: bool,
    pub(crate) separator: bool,
    pub(crate) submenu: Option<Vec<MenuItem>>,
    pub(crate) on_click: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl MenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            danger: false,
            disabled: false,
            separator: false,
            submenu: None,
            on_click: None,
        }
    }

    /// 子菜单项；鼠标移入或点击后向右展开。
    pub fn submenu(label: impl Into<SharedString>, items: Vec<MenuItem>) -> Self {
        Self {
            label: label.into(),
            danger: false,
            disabled: false,
            separator: false,
            submenu: Some(items),
            on_click: None,
        }
    }

    /// 分隔线（无标签、不可点击）
    pub fn separator() -> Self {
        Self {
            label: "".into(),
            danger: false,
            disabled: false,
            separator: true,
            submenu: None,
            on_click: None,
        }
    }

    /// 危险动作（红色文字）
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(listener));
        self
    }
}

type SetOpenSubmenu = Rc<dyn Fn(Option<SharedString>, &mut Window, &mut App)>;

/// 菜单项焦点表：所有层级共享，路径是相对顶层菜单的稳定链。
pub(crate) type MenuFocusMap = Rc<RefCell<HashMap<SharedString, FocusHandle>>>;

pub(crate) fn menu_focus_map(window: &mut Window, cx: &mut App, id: &ElementId) -> MenuFocusMap {
    window
        .use_keyed_state(
            ElementId::Name(format!("{id:?}-focus-map").into()),
            cx,
            |_, _| MenuFocusMap::default(),
        )
        .read(cx)
        .clone()
}

pub(crate) fn menu_focus_handle(map: &MenuFocusMap, path: &str, cx: &mut App) -> FocusHandle {
    if let Some(handle) = map.borrow().get(path).cloned() {
        return handle;
    }
    let handle = cx.focus_handle();
    map.borrow_mut()
        .insert(SharedString::from(path), handle.clone());
    handle
}

pub(crate) fn focus_menu_handle(handle: FocusHandle, window: &mut Window, cx: &mut App) {
    window.defer(cx, move |window, cx| handle.focus(window, cx));
}

pub(crate) fn menu_navigation_target(
    current: Option<usize>,
    key: &str,
    len: usize,
) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key.strip_prefix("arrow").unwrap_or(key) {
        "down" => Some(current.map_or(0, |index| (index + 1).min(len - 1))),
        "up" => Some(current.map_or(len - 1, |index| index.saturating_sub(1))),
        "home" => Some(0),
        "end" => Some(len - 1),
        _ => None,
    }
}

pub(crate) fn menu_edge_path(
    items: &[MenuItem],
    prefix: &str,
    forward: bool,
) -> Option<SharedString> {
    let mut indices = (0..items.len()).filter(|ix| !items[*ix].separator && !items[*ix].disabled);
    let index = if forward {
        indices.next()
    } else {
        indices.next_back()
    }?;
    Some(SharedString::from(submenu_path(prefix, index)))
}

pub(crate) fn menu_edge_handle(
    map: &MenuFocusMap,
    items: &[MenuItem],
    prefix: &str,
    forward: bool,
    cx: &mut App,
) -> Option<FocusHandle> {
    menu_edge_path(items, prefix, forward).map(|path| menu_focus_handle(map, &path, cx))
}

fn menu_submenu_row(
    id: ElementId,
    item: &MenuItem,
    open: bool,
    set_open_submenu: SetOpenSubmenu,
    path: SharedString,
    set_position: Option<(usize, usize)>,
) -> Stateful<Div> {
    let style = theme::with(|t| item_state(t, item.danger, item.disabled));
    let semantics_id = format!("{id:?}-menu-item");
    let mut semantics = a11y::Semantics::new(Role::MenuItem, item.label.clone())
        .id(semantics_id.clone())
        .expanded(open)
        .disabled(item.disabled)
        .focusable(!item.disabled);
    if let Some((position, size)) = set_position {
        semantics = semantics.position(position, size);
    }
    let arrow_color = theme::with(|t| t.color_text_secondary);
    let mut row = semantics
        .apply(div().id(id).accessibility_id(semantics_id))
        .flex()
        .items_center()
        .min_h(px(style.height))
        .px(px(style.padding_inline))
        .py(px(style.padding_block))
        .rounded(px(style.radius))
        .text_size(px(style.font_size))
        .line_height(px(style.line_height))
        .text_color(style.fg)
        .bg(style.bg)
        .child(item.label.clone())
        .child(
            div()
                .ml_auto()
                .pl(px(style.padding_inline))
                .text_color(arrow_color)
                .child("›"),
        );
    if item.disabled {
        row = row.cursor(CursorStyle::Arrow);
    } else {
        let hover_bg = style.hover_bg;
        let hover_open = set_open_submenu.clone();
        let hover_path = path.clone();
        row = row
            .cursor(CursorStyle::PointingHand)
            .hover(move |style| style.bg(hover_bg));
        row = row.on_mouse_move(move |_, window, cx| {
            hover_open(Some(hover_path.clone()), window, cx);
        });
        row = a11y::click_void(row, move |window, cx| {
            set_open_submenu(Some(path.clone()), window, cx);
        });
    }
    row
}

fn submenu_is_open(open_submenu: Option<&SharedString>, item_path: &str) -> bool {
    open_submenu.is_some_and(|open| {
        open.as_str() == item_path
            || open
                .strip_prefix(item_path)
                .is_some_and(|suffix| suffix.starts_with('.'))
    })
}

fn submenu_path(path_prefix: &str, index: usize) -> String {
    format!("{path_prefix}{index}")
}

fn submenu_parent_path(path_prefix: &str) -> Option<SharedString> {
    if path_prefix.is_empty() {
        None
    } else {
        Some(SharedString::from(path_prefix.trim_end_matches('.')))
    }
}

pub(crate) fn point_in_any_bounds(bounds: &[Bounds<Pixels>], point: Point<Pixels>) -> bool {
    bounds.iter().any(|bounds| bounds.contains(&point))
}

pub(crate) fn render_menu_rows(
    id: ElementId,
    items: &[MenuItem],
    width: Pixels,
    close: Rc<dyn Fn(&mut Window, &mut App)>,
    open_submenu: Option<&SharedString>,
    set_open_submenu: SetOpenSubmenu,
    path_prefix: &str,
    popup_bounds: PopupBoundsSink,
    focus_map: &MenuFocusMap,
    parent_handle: Option<FocusHandle>,
    cx: &mut App,
) -> Stateful<Div> {
    let mut menu = menu_container(ElementId::Name(format!("{id:?}-menu").into())).w(width);
    let nav_items: Vec<(MenuItem, SharedString)> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| !item.separator && !item.disabled)
        .map(|(ix, item)| {
            (
                item.clone(),
                SharedString::from(submenu_path(path_prefix, ix)),
            )
        })
        .collect();
    let focus_targets: Vec<FocusHandle> = nav_items
        .iter()
        .map(|(_, path)| menu_focus_handle(focus_map, path, cx))
        .collect();
    let item_count = items.iter().filter(|item| !item.separator).count();
    let parent_open_path = submenu_parent_path(path_prefix);
    let mut item_position = 0;
    for (ix, item) in items.iter().enumerate() {
        if item.separator {
            menu = menu.child(menu_divider_with_id(ElementId::Name(
                format!("{id:?}-separator-{ix}").into(),
            )));
            continue;
        }
        item_position += 1;
        let item_path = submenu_path(path_prefix, ix);
        let focus_index = nav_items
            .iter()
            .position(|(_, path)| path.as_ref() == item_path.as_str());
        let row_handle = focus_index.map(|index| focus_targets[index].clone());
        if let Some(subitems) = &item.submenu {
            let open = submenu_is_open(open_submenu, &item_path);
            let mut row = menu_submenu_row(
                ElementId::Name(format!("{id:?}-item-{ix}").into()),
                item,
                open,
                set_open_submenu.clone(),
                item_path.clone().into(),
                Some((item_position, item_count)),
            );
            if let Some(handle) = &row_handle {
                row = row.track_focus(handle);
            }
            let popup = if open {
                let nested = render_menu_rows(
                    ElementId::Name(format!("{id:?}-submenu-{item_path}").into()),
                    subitems,
                    width,
                    close.clone(),
                    open_submenu,
                    set_open_submenu.clone(),
                    &format!("{item_path}."),
                    popup_bounds.clone(),
                    focus_map,
                    row_handle.clone(),
                    cx,
                );
                let gap = px(theme::with(|t| t.padding_xxs));
                Some(div().ml(gap).child(nested))
            } else {
                None
            };
            menu = menu.child(anchored_with_placement_and_bounds(
                row.into_any_element(),
                popup,
                PopupPlacement::RightTop,
                Some(popup_bounds.clone()),
            ));
        } else {
            let mut row = menu_item_row_at(
                ElementId::Name(format!("{id:?}-item-{ix}").into()),
                item,
                close.clone(),
                Some((item_position, item_count)),
            );
            if let Some(handle) = &row_handle {
                row = row.track_focus(handle);
            }
            let hover_path = submenu_parent_path(path_prefix);
            let clear_submenu = set_open_submenu.clone();
            let row = row.on_mouse_move(move |_, window, cx| {
                clear_submenu(hover_path.clone(), window, cx);
            });
            menu = menu.child(row);
        }
    }

    let key_targets = focus_targets.clone();
    let key_items = nav_items.clone();
    let key_map = focus_map.clone();
    let key_close = close.clone();
    let key_set_open = set_open_submenu.clone();
    let key_parent = parent_handle;
    let key_parent_open = parent_open_path;
    menu = menu.on_key_down(move |event, window, cx| {
        let key = event.keystroke.key.as_str();
        let key = key.strip_prefix("arrow").unwrap_or(key);
        let current = key_targets
            .iter()
            .position(|handle| handle.is_focused(window));
        if key == "escape" {
            cx.stop_propagation();
            key_close(window, cx);
            return;
        }
        if key == "left" {
            cx.stop_propagation();
            if let Some(parent) = &key_parent {
                key_set_open(key_parent_open.clone(), window, cx);
                parent.focus(window, cx);
            } else {
                key_close(window, cx);
            }
            return;
        }
        if matches!(key, "enter" | "space" | "right") {
            let Some(current) = current else {
                return;
            };
            let Some((item, item_path)) = key_items.get(current) else {
                return;
            };
            if let Some(subitems) = &item.submenu {
                cx.stop_propagation();
                key_set_open(Some(item_path.clone()), window, cx);
                if let Some(handle) =
                    menu_edge_handle(&key_map, subitems, &format!("{item_path}."), true, cx)
                {
                    focus_menu_handle(handle, window, cx);
                }
            } else if key != "right" {
                cx.stop_propagation();
                if let Some(listener) = &item.on_click {
                    listener(window, cx);
                }
                key_close(window, cx);
            }
            return;
        }
        if let Some(target) = menu_navigation_target(current, key, key_targets.len()) {
            cx.stop_propagation();
            key_targets[target].focus(window, cx);
        }
    });
    menu
}

/// 下拉动作菜单。
///
/// ```ignore
/// DropdownMenu::new(
///     "more",
///     Button::new("更多操作").secondary(),
///     vec![
///         MenuItem::new("编辑").on_click(|_, _| {}),
///         MenuItem::separator(),
///         MenuItem::new("删除").danger().on_click(|_, _| {}),
///     ],
/// )
/// .open(self.menu_open)
/// .on_open_change(cx.listener(|this, open, _, cx| {
///     this.menu_open = *open;
///     cx.notify();
/// }))
/// ```
#[derive(IntoElement)]
pub struct DropdownMenu {
    id: ElementId,
    trigger: AnyElement,
    items: Vec<MenuItem>,
    open: bool,
    width: Pixels,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
}

impl DropdownMenu {
    pub fn new(id: impl Into<ElementId>, trigger: impl IntoElement, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            items,
            open: false,
            width: px(180.),
            on_open_change: None,
        }
    }

    /// 当前是否展开（受控）
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// 菜单宽度，默认 180px
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    pub fn on_open_change(
        mut self,
        listener: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for DropdownMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open = self.open;
        let menu_focus = menu_focus_map(window, cx, &self.id);
        let trigger_focus = window
            .use_keyed_state(
                ElementId::Name(format!("{:?}-trigger-focus", self.id).into()),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone();
        let first_item = menu_edge_handle(&menu_focus, &self.items, "", true, cx);
        let last_item = menu_edge_handle(&menu_focus, &self.items, "", false, cx);
        let suppress_toggle = Rc::new(Cell::new(false));
        let submenu_state = window.use_keyed_state(
            ElementId::Name(format!("{:?}-submenu-state", self.id).into()),
            cx,
            |_, _| None::<SharedString>,
        );
        let open_submenu = submenu_state.read(cx).clone();
        let popup_bounds_state = window.use_keyed_state(
            ElementId::Name(format!("{:?}-popup-bounds", self.id).into()),
            cx,
            |_, _| Rc::new(RefCell::new(Vec::new())),
        );
        let popup_bounds = popup_bounds_state.read(cx).clone();
        popup_bounds.borrow_mut().clear();
        let set_open_submenu: SetOpenSubmenu = {
            let state = submenu_state.clone();
            Rc::new(move |path, _window: &mut Window, cx: &mut App| {
                if *state.read(cx) != path {
                    state.write(cx, path);
                    cx.refresh_windows();
                }
            })
        };

        let trigger_id = format!("{:?}-trigger", self.id);
        let mut trigger =
            a11y::Semantics::new(Role::Button, if open { "关闭菜单" } else { "打开菜单" })
                .id(trigger_id.clone())
                .expanded(open)
                .focusable(self.on_open_change.is_some())
                .apply(
                    div()
                        .id(ElementId::Name(trigger_id.into()))
                        .track_focus(&trigger_focus),
                )
                .child(self.trigger);
        if let Some(on_open_change) = self.on_open_change.clone() {
            let suppress_toggle = suppress_toggle.clone();
            let click_submenu = set_open_submenu.clone();
            let click_open = on_open_change.clone();
            let click_first = first_item.clone();
            trigger = trigger.cursor(CursorStyle::PointingHand);
            trigger = a11y::click_void(trigger, move |window, cx| {
                if suppress_toggle.take() {
                    return;
                }
                let next = !open;
                if !next {
                    click_submenu(None, window, cx);
                }
                click_open(&next, window, cx);
                if next && let Some(handle) = &click_first {
                    focus_menu_handle(handle.clone(), window, cx);
                }
            });
            let keyboard_open = on_open_change.clone();
            let keyboard_submenu = set_open_submenu.clone();
            let keyboard_first = first_item.clone();
            let keyboard_last = last_item.clone();
            trigger = trigger.on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                let key = key.strip_prefix("arrow").unwrap_or(key);
                if key == "escape" && open {
                    cx.stop_propagation();
                    keyboard_submenu(None, window, cx);
                    on_open_change(&false, window, cx);
                    return;
                }
                let handle = match key {
                    "down" => keyboard_first.clone(),
                    "up" => keyboard_last.clone(),
                    _ => return,
                };
                cx.stop_propagation();
                keyboard_open(&true, window, cx);
                if let Some(handle) = handle {
                    focus_menu_handle(handle, window, cx);
                }
            });
        }

        let mut popup = None;
        if open {
            let close = {
                let on_open_change = self.on_open_change.clone();
                let set_open_submenu = set_open_submenu.clone();
                let trigger_focus = trigger_focus.clone();
                Rc::new(move |window: &mut Window, cx: &mut App| {
                    set_open_submenu(None, window, cx);
                    if let Some(on_open_change) = &on_open_change {
                        on_open_change(&false, window, cx);
                    }
                    trigger_focus.focus(window, cx);
                })
            };
            let menu = render_menu_rows(
                self.id,
                &self.items,
                self.width,
                close,
                open_submenu.as_ref(),
                set_open_submenu.clone(),
                "",
                popup_bounds.clone(),
                &menu_focus,
                None,
                cx,
            );
            let menu = menu.on_mouse_down_out({
                let on_open_change = self.on_open_change.clone();
                let suppress_toggle = suppress_toggle.clone();
                let set_open_submenu = set_open_submenu.clone();
                move |event, window, cx| {
                    if point_in_any_bounds(&popup_bounds.borrow(), event.position) {
                        return;
                    }
                    suppress_toggle.set(true);
                    set_open_submenu(None, window, cx);
                    if let Some(on_open_change) = &on_open_change {
                        on_open_change(&false, window, cx);
                    }
                }
            });
            popup = Some(div().child(menu));
        }

        anchored(trigger.into_any_element(), popup)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submenu_paths_identify_each_nested_level() {
        assert_eq!(submenu_path("", 0), "0");
        assert_eq!(submenu_path("0.", 2), "0.2");
        assert_eq!(submenu_path("0.2.", 1), "0.2.1");
    }

    #[test]
    fn submenu_parent_paths_preserve_the_hovered_branch() {
        assert_eq!(submenu_parent_path(""), None);
        assert_eq!(submenu_parent_path("2.").as_deref(), Some("2"));
        assert_eq!(submenu_parent_path("2.0.").as_deref(), Some("2.0"));
    }

    #[test]
    fn popup_hit_testing_includes_only_recorded_bounds() {
        let first = Bounds {
            origin: gpui::point(px(10.), px(20.)),
            size: gpui::size(px(30.), px(40.)),
        };
        let second = Bounds {
            origin: gpui::point(px(100.), px(120.)),
            size: gpui::size(px(30.), px(40.)),
        };
        assert!(!point_in_any_bounds(&[], gpui::point(px(15.), px(25.))));
        assert!(point_in_any_bounds(
            &[first, second],
            gpui::point(px(15.), px(25.))
        ));
        assert!(point_in_any_bounds(
            &[first, second],
            gpui::point(px(110.), px(130.))
        ));
        assert!(!point_in_any_bounds(
            &[first, second],
            gpui::point(px(60.), px(70.))
        ));
    }

    #[test]
    fn submenu_open_state_includes_ancestors_but_not_siblings() {
        let cases = [
            (None, "0", false),
            (Some("0"), "0", true),
            (Some("0"), "1", false),
            (Some("0"), "0.2", false),
            (Some("0.2"), "0", true),
            (Some("0.2"), "0.2", true),
            (Some("0.2"), "0.20", false),
            (Some("0.2"), "0.2.1", false),
            (Some("0.2.1"), "0.2.1", true),
            (Some("0.2.1"), "0.2", true),
            (Some("0.2.1"), "0.1", false),
        ];
        for (open, item, expected) in cases {
            let open = open.map(SharedString::from);
            assert_eq!(
                submenu_is_open(open.as_ref(), item),
                expected,
                "{open:?} {item}"
            );
        }
    }

    #[test]
    fn menu_navigation_moves_without_wrapping_and_supports_prefixes() {
        assert_eq!(menu_navigation_target(None, "down", 3), Some(0));
        assert_eq!(menu_navigation_target(None, "arrowup", 3), Some(2));
        assert_eq!(menu_navigation_target(Some(1), "down", 3), Some(2));
        assert_eq!(menu_navigation_target(Some(2), "down", 3), Some(2));
        assert_eq!(menu_navigation_target(Some(1), "up", 3), Some(0));
        assert_eq!(menu_navigation_target(Some(0), "up", 3), Some(0));
        assert_eq!(menu_navigation_target(Some(1), "home", 3), Some(0));
        assert_eq!(menu_navigation_target(Some(1), "end", 3), Some(2));
        assert_eq!(menu_navigation_target(Some(1), "left", 3), None);
        assert_eq!(menu_navigation_target(Some(1), "down", 0), None);
    }

    #[test]
    fn menu_edges_skip_separators_and_disabled_items() {
        let items = vec![
            MenuItem::separator(),
            MenuItem::new("disabled").disabled(true),
            MenuItem::new("first"),
            MenuItem::separator(),
            MenuItem::new("last"),
            MenuItem::new("disabled-last").disabled(true),
        ];
        assert_eq!(menu_edge_path(&items, "2.", true).as_deref(), Some("2.2"));
        assert_eq!(menu_edge_path(&items, "2.", false).as_deref(), Some("2.4"));
        assert_eq!(menu_edge_path(&[], "2.", true), None);
        assert_eq!(menu_edge_path(&[MenuItem::separator()], "2.", false), None);
    }
}
