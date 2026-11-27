//! 桌面工具栏：动作按钮、选中态、分隔线与弹性占位。

use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, CursorStyle, ElementId, FocusHandle, IntoElement, Orientation, RenderOnce,
    Role, SharedString, Window, div, prelude::*, px, transparent_black,
};

use crate::{a11y, theme};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ToolbarItemKind {
    #[default]
    Action,
    Separator,
    Spacer,
}

/// 工具栏动作项或结构项。
pub struct ToolbarItem {
    id: SharedString,
    label: SharedString,
    icon: Option<AnyElement>,
    kind: ToolbarItemKind,
    icon_only: bool,
    selected: bool,
    disabled: bool,
    on_click: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl ToolbarItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            kind: ToolbarItemKind::Action,
            icon_only: false,
            selected: false,
            disabled: false,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    pub fn icon_only(mut self) -> Self {
        self.icon_only = true;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
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

    pub fn separator() -> Self {
        Self {
            id: String::new().into(),
            label: String::new().into(),
            icon: None,
            kind: ToolbarItemKind::Separator,
            icon_only: false,
            selected: false,
            disabled: false,
            on_click: None,
        }
    }

    pub fn spacer() -> Self {
        Self {
            id: String::new().into(),
            label: String::new().into(),
            icon: None,
            kind: ToolbarItemKind::Spacer,
            icon_only: false,
            selected: false,
            disabled: false,
            on_click: None,
        }
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn label(&self) -> &SharedString {
        &self.label
    }

    pub fn is_separator(&self) -> bool {
        self.kind == ToolbarItemKind::Separator
    }

    pub fn is_spacer(&self) -> bool {
        self.kind == ToolbarItemKind::Spacer
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ToolbarItemState {
    #[default]
    Normal,
    Hover,
    Selected,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolbarItemStyle {
    pub fg: gpui::Hsla,
    pub bg: gpui::Hsla,
}

pub fn toolbar_item_state(
    tokens: &theme::Tokens,
    selected: bool,
    hover: bool,
    disabled: bool,
) -> ToolbarItemStyle {
    let state = if disabled {
        ToolbarItemState::Disabled
    } else if selected {
        ToolbarItemState::Selected
    } else if hover {
        ToolbarItemState::Hover
    } else {
        ToolbarItemState::Normal
    };
    match state {
        ToolbarItemState::Normal => ToolbarItemStyle {
            fg: tokens.color_text,
            bg: transparent_black(),
        },
        ToolbarItemState::Hover => ToolbarItemStyle {
            fg: tokens.color_text,
            bg: tokens.color_fill_tertiary,
        },
        ToolbarItemState::Selected => ToolbarItemStyle {
            fg: tokens.color_primary,
            bg: tokens.color_primary_bg,
        },
        ToolbarItemState::Disabled => ToolbarItemStyle {
            fg: tokens.color_text_disabled,
            bg: transparent_black(),
        },
    }
}

/// 横向或纵向工具栏。
///
/// ```ignore
/// Toolbar::new("editor-tools", vec![
///     ToolbarItem::new("bold", "粗体").icon_only().selected(true),
///     ToolbarItem::separator(),
///     ToolbarItem::spacer(),
///     ToolbarItem::new("save", "保存"),
/// ])
/// .on_action(|id, _, _| { /* ... */ })
/// ```
#[derive(IntoElement)]
pub struct Toolbar {
    id: ElementId,
    items: Vec<ToolbarItem>,
    vertical: bool,
    on_action: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
}

impl Toolbar {
    pub fn new(id: impl Into<ElementId>, items: Vec<ToolbarItem>) -> Self {
        Self {
            id: id.into(),
            items,
            vertical: false,
            on_action: None,
        }
    }

    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    pub fn on_action(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_action = Some(Rc::new(listener));
        self
    }
}

fn toolbar_target(current: usize, key: &str, len: usize, vertical: bool) -> Option<usize> {
    let key = key.strip_prefix("arrow").unwrap_or(key);
    if len == 0 {
        return None;
    }
    match key {
        "home" => Some(0),
        "end" => Some(len - 1),
        "right" if !vertical => Some((current + 1) % len),
        "left" if !vertical => Some((current + len - 1) % len),
        "down" if vertical => Some((current + 1) % len),
        "up" if vertical => Some((current + len - 1) % len),
        _ => None,
    }
}

impl RenderOnce for Toolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let owner = format!("{:?}", self.id);
        let metrics = theme::with(|t| theme::metric::ToolbarMetric::resolve(t));
        let actionable: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(ix, item)| (!item.is_separator() && !item.is_spacer()).then_some(ix))
            .collect();
        let focus_state = window.use_keyed_state(
            ElementId::Name(format!("{owner}-focus-handles").into()),
            cx,
            |_, _| Rc::new(RefCell::new(Vec::<FocusHandle>::new())),
        );
        let mut handles = focus_state.read(cx).borrow_mut();
        while handles.len() < actionable.len() {
            handles.push(cx.focus_handle());
        }

        let orientation = if self.vertical {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        };
        let action_count = actionable.len();
        let toolbar_id = format!("{owner}-toolbar");
        let mut toolbar = a11y::Semantics::new(Role::Toolbar, "工具栏")
            .id(toolbar_id.clone())
            .orientation(orientation)
            .apply(
                div()
                    .id(ElementId::Name(toolbar_id.into()))
                    .flex()
                    .items_center()
                    .gap(px(metrics.gap))
                    .p(px(metrics.padding))
                    .rounded(px(theme::with(|t| t.border_radius)))
                    .bg(theme::with(|t| t.color_bg_container))
                    .border_1()
                    .border_color(theme::with(|t| t.color_border_secondary)),
            );
        toolbar = if self.vertical {
            toolbar.flex_col().w(px(metrics.height))
        } else {
            toolbar.flex_row().h(px(metrics.height))
        };

        for (item_index, item) in self.items.into_iter().enumerate() {
            if item.is_spacer() {
                toolbar = toolbar.child(div().flex_1());
                continue;
            }
            if item.is_separator() {
                let line_width = theme::with(|t| t.line_width);
                let separator = if self.vertical {
                    div().h(px(line_width)).w(px(metrics.separator_length))
                } else {
                    div().w(px(line_width)).h(px(metrics.separator_length))
                };
                toolbar = toolbar.child(
                    div()
                        .flex_shrink_0()
                        .flex_col()
                        .items_center()
                        .px(px(metrics.separator_margin))
                        .child(separator.bg(theme::with(|t| t.color_border_secondary))),
                );
                continue;
            }

            let action_index = actionable
                .iter()
                .position(|ix| *ix == item_index)
                .unwrap_or(0);
            let focus_handle = handles[action_index].clone();
            let style = theme::with(|t| toolbar_item_state(t, item.selected, false, item.disabled));
            let hover_style = theme::with(|t| toolbar_item_state(t, item.selected, true, false));
            let item_id = format!("{owner}-item-{}", item.id);
            let mut button = a11y::Semantics::new(Role::Button, item.label.clone())
                .id(item_id.clone())
                .toggled(item.selected.into())
                .disabled(item.disabled)
                .focusable(!item.disabled)
                .position(action_index + 1, action_count)
                .apply(
                    div()
                        .id(ElementId::Name(item_id.into()))
                        .track_focus(&focus_handle)
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap(px(metrics.gap))
                        .h(px(metrics.height - metrics.padding * 2.))
                        .px(px(if item.icon_only {
                            metrics.item_padding_inline
                        } else {
                            metrics.item_padding_inline + metrics.gap
                        }))
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
                );
            let has_icon = item.icon.is_some();
            if let Some(icon) = item.icon {
                button = button.child(
                    div()
                        .w(px(metrics.icon_size))
                        .h(px(metrics.icon_size))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon),
                );
            }
            if !item.icon_only || !has_icon {
                button = button.child(item.label.clone());
            }
            if !item.disabled {
                button =
                    button.hover(move |style| style.bg(hover_style.bg).text_color(hover_style.fg));
                let item_on_click = item.on_click.clone();
                let on_action = self.on_action.clone();
                let action_id = item.id.clone();
                button = a11y::click_void(button, move |window, cx| {
                    if let Some(listener) = &item_on_click {
                        listener(window, cx);
                    }
                    if let Some(listener) = &on_action {
                        listener(&action_id, window, cx);
                    }
                });
                let target_handles = handles.clone();
                let vertical = self.vertical;
                button = button.on_key_down(move |event, window, cx| {
                    if let Some(target) = toolbar_target(
                        action_index,
                        event.keystroke.key.as_str(),
                        action_count,
                        vertical,
                    ) {
                        cx.stop_propagation();
                        target_handles[target].focus(window, cx);
                    }
                });
            }
            toolbar = toolbar.child(button);
        }
        toolbar
    }
}

#[cfg(test)]
mod tests {
    use super::toolbar_target;

    #[test]
    fn toolbar_keyboard_targets_follow_orientation() {
        assert_eq!(toolbar_target(0, "right", 3, false), Some(1));
        assert_eq!(toolbar_target(0, "left", 3, false), Some(2));
        assert_eq!(toolbar_target(0, "down", 3, false), None);
        assert_eq!(toolbar_target(2, "down", 3, true), Some(0));
        assert_eq!(toolbar_target(0, "up", 3, true), Some(2));
        assert_eq!(toolbar_target(1, "home", 3, true), Some(0));
        assert_eq!(toolbar_target(1, "end", 3, true), Some(2));
    }
}
