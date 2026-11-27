//! 应用菜单栏：顶层菜单按钮、悬停切换、下拉动作与键盘导航。

use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, CursorStyle, ElementId, FocusHandle, IntoElement, Orientation, Pixels,
    RenderOnce, Role, SharedString, Window, div, prelude::*, px, transparent_black,
};

use crate::a11y;
use crate::menu::{MenuItem, point_in_any_bounds, render_menu_rows};
use crate::popup::{PopupBoundsSink, PopupPlacement, anchored_with_placement_and_bounds};
use crate::theme::{self, Tokens};

/// 菜单栏顶层项。有子项时点击/悬停打开菜单，无子项时作为直接动作按钮。
pub struct MenuBarItem {
    id: SharedString,
    label: SharedString,
    icon: Option<AnyElement>,
    children: Vec<MenuItem>,
    disabled: bool,
    on_click: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl MenuBarItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: Vec::new(),
            disabled: false,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn children(mut self, items: Vec<MenuItem>) -> Self {
        self.children = items;
        self
    }

    pub fn on_click(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(listener));
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label(&self) -> &SharedString {
        &self.label
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum MenuBarItemState {
    #[default]
    Normal,
    Hover,
    Selected,
    Open,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuBarItemStyle {
    pub fg: gpui::Hsla,
    pub bg: gpui::Hsla,
    pub indicator: gpui::Hsla,
}

pub fn menu_bar_item_state(
    tokens: &Tokens,
    selected: bool,
    open: bool,
    hover: bool,
    disabled: bool,
) -> MenuBarItemStyle {
    let state = if disabled {
        MenuBarItemState::Disabled
    } else if open {
        MenuBarItemState::Open
    } else if selected {
        MenuBarItemState::Selected
    } else if hover {
        MenuBarItemState::Hover
    } else {
        MenuBarItemState::Normal
    };
    match state {
        MenuBarItemState::Normal => MenuBarItemStyle {
            fg: tokens.color_text,
            bg: transparent_black(),
            indicator: transparent_black(),
        },
        MenuBarItemState::Hover => MenuBarItemStyle {
            fg: tokens.color_primary,
            bg: tokens.color_fill_tertiary,
            indicator: transparent_black(),
        },
        MenuBarItemState::Selected => MenuBarItemStyle {
            fg: tokens.color_primary,
            bg: transparent_black(),
            indicator: tokens.color_primary,
        },
        MenuBarItemState::Open => MenuBarItemStyle {
            fg: tokens.color_primary,
            bg: tokens.color_primary_bg,
            indicator: tokens.color_primary,
        },
        MenuBarItemState::Disabled => MenuBarItemStyle {
            fg: tokens.color_text_disabled,
            bg: transparent_black(),
            indicator: transparent_black(),
        },
    }
}

/// 横向应用菜单栏。
///
/// ```ignore
/// MenuBar::new("app-menu", vec![
///     MenuBarItem::new("file", "文件").children(vec![
///         MenuItem::new("新建").on_click(|_, _| {}),
///         MenuItem::separator(),
///         MenuItem::new("退出").danger(),
///     ]),
///     MenuBarItem::new("help", "帮助"),
/// ])
/// .open(self.open_menu)
/// .selected(self.selected_menu)
/// .on_open_change(cx.listener(|this, open, _, cx| {
///     this.open_menu = *open;
///     cx.notify();
/// }))
/// ```
#[derive(IntoElement)]
pub struct MenuBar {
    id: ElementId,
    items: Vec<MenuBarItem>,
    open: Option<usize>,
    selected: Option<usize>,
    menu_width: Pixels,
    on_open_change: Option<Rc<dyn Fn(&Option<usize>, &mut Window, &mut App)>>,
    on_select: Option<Rc<dyn Fn(usize, &SharedString, &mut Window, &mut App)>>,
}

impl MenuBar {
    pub fn new(id: impl Into<ElementId>, items: Vec<MenuBarItem>) -> Self {
        Self {
            id: id.into(),
            items,
            open: None,
            selected: None,
            menu_width: px(180.),
            on_open_change: None,
            on_select: None,
        }
    }

    pub fn open(mut self, open: Option<usize>) -> Self {
        self.open = open;
        self
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn menu_width(mut self, width: Pixels) -> Self {
        self.menu_width = width;
        self
    }

    pub fn on_open_change(
        mut self,
        listener: impl Fn(&Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(listener));
        self
    }

    pub fn on_select(
        mut self,
        listener: impl Fn(usize, &SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }
}

fn menu_bar_target(current: usize, key: &str, len: usize) -> Option<usize> {
    let key = key.strip_prefix("arrow").unwrap_or(key);
    if len == 0 {
        return None;
    }
    match key {
        "right" => Some((current + 1) % len),
        "left" => Some((current + len - 1) % len),
        "home" => Some(0),
        "end" => Some(len - 1),
        _ => None,
    }
}

fn observe_menu_items(
    items: &mut [MenuItem],
    top_index: usize,
    on_select: &Rc<dyn Fn(usize, &SharedString, &mut Window, &mut App)>,
) {
    for item in items {
        if let Some(children) = &mut item.submenu {
            observe_menu_items(children, top_index, on_select);
            continue;
        }
        if item.separator {
            continue;
        }
        let previous = item.on_click.take();
        let on_select = on_select.clone();
        let label = item.label.clone();
        item.on_click = Some(Rc::new(move |window, cx| {
            if let Some(previous) = &previous {
                previous(window, cx);
            }
            on_select(top_index, &label, window, cx);
        }));
    }
}

impl RenderOnce for MenuBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let owner = format!("{:?}", self.id);
        let menu_was_open = self.open.is_some();
        let metrics = theme::with(|t| theme::metric::MenuBarMetric::resolve(t));
        let focus_state = window.use_keyed_state(
            ElementId::Name(format!("{owner}-focus-handles").into()),
            cx,
            |_, _| Rc::new(RefCell::new(Vec::<FocusHandle>::new())),
        );
        let handles = {
            let mut handles = focus_state.read(cx).borrow_mut();
            while handles.len() < self.items.len() {
                handles.push(cx.focus_handle());
            }
            handles.clone()
        };
        let popup_bounds_state = window.use_keyed_state(
            ElementId::Name(format!("{owner}-popup-bounds").into()),
            cx,
            |_, _| Rc::new(RefCell::new(Vec::new())),
        );
        let popup_bounds: PopupBoundsSink = popup_bounds_state.read(cx).clone();
        popup_bounds.borrow_mut().clear();
        let submenu_state = window.use_keyed_state(
            ElementId::Name(format!("{owner}-submenu-{:?}", self.open).into()),
            cx,
            |_, _| None::<SharedString>,
        );
        let open_submenu = submenu_state.read(cx).clone();
        let set_open_submenu = {
            let state = submenu_state.clone();
            Rc::new(
                move |path: Option<SharedString>, _window: &mut Window, cx: &mut App| {
                    if *state.read(cx) != path {
                        state.write(cx, path);
                        cx.refresh_windows();
                    }
                },
            )
        };
        let close = {
            let on_open_change = self.on_open_change.clone();
            let set_open_submenu = set_open_submenu.clone();
            Rc::new(move |window: &mut Window, cx: &mut App| {
                set_open_submenu(None, window, cx);
                if let Some(on_open_change) = &on_open_change {
                    on_open_change(&None, window, cx);
                }
            })
        };

        let bar_id = format!("{owner}-bar");
        let mut bar = a11y::Semantics::new(Role::MenuBar, "应用菜单")
            .id(bar_id.clone())
            .orientation(Orientation::Horizontal)
            .apply(
                div()
                    .id(ElementId::Name(bar_id.into()))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(metrics.gap))
                    .h(px(metrics.height))
                    .w_full()
                    .px(px(metrics.padding_inline))
                    .bg(theme::with(|t| t.color_bg_container))
                    .border_b_1()
                    .border_color(theme::with(|t| t.color_border_secondary)),
            );

        let item_count = self.items.len();
        for (index, item) in self.items.into_iter().enumerate() {
            let open = self.open == Some(index);
            let selected = self.selected == Some(index);
            let style =
                theme::with(|t| menu_bar_item_state(t, selected, open, false, item.disabled));
            let hover_style = theme::with(|t| menu_bar_item_state(t, selected, open, true, false));
            let focus_handle = handles[index].clone();
            let item_id = format!("{owner}-item-{}", item.id);
            let mut semantics = a11y::Semantics::new(Role::Button, item.label.clone())
                .id(item_id.clone())
                .selected(selected)
                .disabled(item.disabled)
                .focusable(!item.disabled)
                .position(index + 1, item_count);
            if !item.children.is_empty() {
                semantics = semantics.expanded(open);
            }
            let mut button = semantics
                .apply(
                    div()
                        .id(ElementId::Name(item_id.into()))
                        .track_focus(&focus_handle)
                        .relative()
                        .flex()
                        .items_center()
                        .gap(px(metrics.gap))
                        .h(px(metrics.height))
                        .px(px(metrics.item_padding_inline))
                        .rounded(px(metrics.item_radius))
                        .text_size(px(metrics.font_size))
                        .line_height(px(metrics.line_height))
                        .text_color(style.fg)
                        .bg(style.bg)
                        .cursor(if item.disabled {
                            CursorStyle::Arrow
                        } else {
                            CursorStyle::PointingHand
                        }),
                )
                .child(item.label.clone());
            if let Some(icon) = item.icon {
                button = button.child(
                    div()
                        .w(px(metrics.font_size))
                        .h(px(metrics.font_size))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon),
                );
            }
            if selected || open {
                button = button.child(
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .w_full()
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .w(px(metrics.indicator_width))
                                .h(px(metrics.indicator_height))
                                .bg(style.indicator),
                        ),
                );
            }

            if !item.disabled {
                button =
                    button.hover(move |style| style.bg(hover_style.bg).text_color(hover_style.fg));
                if !item.children.is_empty() {
                    let on_open_change = self.on_open_change.clone();
                    let next_open = if open { None } else { Some(index) };
                    button = a11y::click_void(button, move |window, cx| {
                        if let Some(on_open_change) = &on_open_change {
                            on_open_change(&next_open, window, cx);
                        }
                    });
                    let on_open_change = self.on_open_change.clone();
                    button = button.on_mouse_move(move |_, window, cx| {
                        if menu_was_open && !open {
                            if let Some(on_open_change) = &on_open_change {
                                on_open_change(&Some(index), window, cx);
                            }
                        }
                    });
                } else {
                    let item_on_click = item.on_click.clone();
                    let on_select = self.on_select.clone();
                    let label = item.label.clone();
                    button = a11y::click_void(button, move |window, cx| {
                        if let Some(listener) = &item_on_click {
                            listener(window, cx);
                        }
                        if let Some(listener) = &on_select {
                            listener(index, &label, window, cx);
                        }
                    });
                }
                let target_handles = handles.clone();
                let on_open_change = self.on_open_change.clone();
                button = button.on_key_down(move |event, window, cx| {
                    if event.keystroke.key == "escape" {
                        cx.stop_propagation();
                        if let Some(on_open_change) = &on_open_change {
                            on_open_change(&None, window, cx);
                        }
                        return;
                    }
                    if let Some(target) =
                        menu_bar_target(index, event.keystroke.key.as_str(), item_count)
                    {
                        cx.stop_propagation();
                        target_handles[target].focus(window, cx);
                        if self.open.is_some() {
                            if let Some(on_open_change) = &on_open_change {
                                on_open_change(&Some(target), window, cx);
                            }
                        }
                    }
                });
            }

            let popup = if open && !item.children.is_empty() {
                let mut children = item.children.clone();
                if let Some(on_select) = &self.on_select {
                    observe_menu_items(&mut children, index, on_select);
                }
                let menu = render_menu_rows(
                    ElementId::Name(format!("{owner}-menu-{index}").into()),
                    &children,
                    self.menu_width,
                    close.clone(),
                    open_submenu.as_ref(),
                    set_open_submenu.clone(),
                    "",
                    popup_bounds.clone(),
                );
                Some(div().child(menu))
            } else {
                None
            };
            bar = bar.child(anchored_with_placement_and_bounds(
                button.into_any_element(),
                popup,
                PopupPlacement::Bottom,
                Some(popup_bounds.clone()),
            ));
        }

        let on_open_change = self.on_open_change.clone();
        let set_open_submenu = set_open_submenu.clone();
        let outside_was_open = self.open.is_some();
        bar.on_mouse_down_out(move |event, window, cx| {
            if !outside_was_open || point_in_any_bounds(&popup_bounds.borrow(), event.position) {
                return;
            }
            set_open_submenu(None, window, cx);
            if let Some(on_open_change) = &on_open_change {
                on_open_change(&None, window, cx);
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::menu_bar_target;

    #[test]
    fn menu_bar_keyboard_targets_wrap_and_jump() {
        assert_eq!(menu_bar_target(0, "left", 3), Some(2));
        assert_eq!(menu_bar_target(2, "right", 3), Some(0));
        assert_eq!(menu_bar_target(1, "home", 3), Some(0));
        assert_eq!(menu_bar_target(1, "end", 3), Some(2));
        assert_eq!(menu_bar_target(1, "down", 3), None);
    }
}
