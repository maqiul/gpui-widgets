//! 右键上下文菜单：包裹任意内容，右键在光标处弹出动作菜单。

use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, ElementId, IntoElement, MouseButton, ParentElement, Pixels, Point, RenderOnce,
    Role, SharedString, Window, deferred, div, prelude::*, px,
};

use crate::MenuItem;
use crate::a11y::{self, Semantics};
use crate::menu::{divider_state, point_in_any_bounds, render_menu_rows};
use crate::theme::{self, metric};

/// 上下文菜单状态（全局共享，同一时刻只展开一个）
#[derive(Default)]
struct ContextMenuState {
    open_id: Option<SharedString>,
    position: Point<Pixels>,
}

impl gpui::Global for ContextMenuState {}

/// 右键上下文菜单。
///
/// ```ignore
/// ContextMenu::new("file-menu", vec![
///     MenuItem::new("打开").on_click(|_, _| {}),
///     MenuItem::separator(),
///     MenuItem::new("删除").danger().on_click(|_, _| {}),
/// ])
/// .child(div().child("右键点击我"))
/// ```
#[derive(IntoElement)]
pub struct ContextMenu {
    id: SharedString,
    items: Vec<MenuItem>,
    children: Vec<AnyElement>,
}

impl ContextMenu {
    pub fn new(id: impl Into<SharedString>, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            items,
            children: Vec::new(),
        }
    }
}

impl ParentElement for ContextMenu {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ContextMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self.id.clone();
        let state = cx.default_global::<ContextMenuState>();
        let is_open = state.open_id.as_ref() == Some(&id);
        let position = state.position;
        let submenu_state = window.use_keyed_state(
            ElementId::Name(format!("{id}-submenu-state").into()),
            cx,
            |_, _| None::<SharedString>,
        );
        let open_submenu = submenu_state.read(cx).clone();
        let popup_bounds_state = window.use_keyed_state(
            ElementId::Name(format!("{id}-popup-bounds").into()),
            cx,
            |_, _| Rc::new(RefCell::new(Vec::new())),
        );
        let popup_bounds = popup_bounds_state.read(cx).clone();
        popup_bounds.borrow_mut().clear();
        let set_open_submenu = {
            let state = submenu_state.clone();
            Rc::new(move |path, _window: &mut Window, cx: &mut App| {
                if *state.read(cx) != path {
                    state.write(cx, path);
                    cx.refresh_windows();
                }
            })
        };
        let open_menu = {
            let id = id.clone();
            let set_open_submenu = set_open_submenu.clone();
            Rc::new(
                move |position: Point<Pixels>, window: &mut Window, cx: &mut App| {
                    let state = cx.default_global::<ContextMenuState>();
                    state.open_id = Some(id.clone());
                    state.position = position;
                    set_open_submenu(None, window, cx);
                    cx.stop_propagation();
                    cx.refresh_windows();
                },
            )
        };

        let mut root = div()
            .id(id.clone())
            .relative()
            .children(self.children)
            .on_mouse_down(MouseButton::Right, {
                let open_menu = open_menu.clone();
                move |event, window, cx| open_menu(event.position, window, cx)
            });
        root = a11y::click_void(root, {
            let open_menu = open_menu.clone();
            move |window, cx| open_menu(position, window, cx)
        });

        if is_open {
            let menu_width = px(180.);
            let (metrics, divider, edge) = theme::with(|t| {
                (
                    metric::SelectMetric::resolve(t),
                    divider_state(t),
                    t.padding_xs,
                )
            });
            let rows_height: f32 = self
                .items
                .iter()
                .map(|item| {
                    if item.separator {
                        divider.width + 2.0 * divider.margin
                    } else {
                        metrics.option_height
                    }
                })
                .sum();
            let menu_height = px(rows_height + 2.0 * metrics.menu_padding);
            let viewport = window.viewport_size();
            let x = position
                .x
                .min(viewport.width - menu_width - px(edge))
                .max(px(0.));
            let y = position
                .y
                .min(viewport.height - menu_height - px(edge))
                .max(px(0.));

            let menu_id = format!("{id}-menu");
            let close = {
                let id = id.clone();
                let set_open_submenu = set_open_submenu.clone();
                Rc::new(move |window: &mut Window, cx: &mut App| {
                    set_open_submenu(None, window, cx);
                    let state = cx.default_global::<ContextMenuState>();
                    if state.open_id.as_ref() == Some(&id) {
                        state.open_id = None;
                    }
                    cx.refresh_windows();
                })
            };
            let menu = render_menu_rows(
                ElementId::Name(menu_id.clone().into()),
                &self.items,
                menu_width,
                close,
                open_submenu.as_ref(),
                set_open_submenu.clone(),
                "",
                popup_bounds.clone(),
            );
            let menu = menu.on_mouse_down_out({
                let id = id.clone();
                let set_open_submenu = set_open_submenu.clone();
                move |event, window, cx| {
                    if point_in_any_bounds(&popup_bounds.borrow(), event.position) {
                        return;
                    }
                    set_open_submenu(None, window, cx);
                    let state = cx.default_global::<ContextMenuState>();
                    if state.open_id.as_ref() == Some(&id) {
                        state.open_id = None;
                        cx.refresh_windows();
                    }
                }
            });
            let menu = Semantics::new(Role::Menu, "上下文菜单")
                .id(menu_id)
                .apply(menu);

            root = root.child(deferred(div().absolute().left(x).top(y).child(menu)));
        }

        Semantics::new(Role::Button, format!("上下文菜单：{id}"))
            .id(id)
            .expanded(is_open)
            .focusable(true)
            .apply(root)
    }
}
