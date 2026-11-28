//! 通知中心：通知历史、未读计数、已读 / 删除与键盘导航。

use std::rc::Rc;

use gpui::{
    App, ElementId, IntoElement, RenderOnce, Role, ScrollStrategy, SharedString,
    UniformListScrollHandle, Window, deferred, div, prelude::*, px, uniform_list,
};

use crate::a11y::{self, Semantics};
use crate::button::Button;
use crate::controls::scrollbar::Scrollbar;
use crate::feedback::notification::{
    NotificationItem, NotificationVariant, icon_canvas, variant_icon,
};
use crate::theme::{self, metric};

pub fn unread_count(items: &[NotificationItem]) -> usize {
    items.iter().filter(|item| !item.is_read()).count()
}

pub fn notification_center_row_height(tokens: &crate::theme::Tokens) -> f32 {
    let m = metric::notification_metric(tokens);
    m.title_line + m.line + tokens.padding_sm
}

pub fn notification_center_target(current: Option<usize>, key: &str, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key.strip_prefix("arrow").unwrap_or(key) {
        "up" => Some(match current {
            Some(current) => (current + len - 1) % len,
            None => len - 1,
        }),
        "down" => Some(current.map_or(0, |current| (current + 1) % len)),
        "home" => Some(0),
        "end" => Some(len - 1),
        _ => None,
    }
}

#[derive(IntoElement)]
pub struct NotificationCenter {
    id: ElementId,
    items: Vec<NotificationItem>,
    open: bool,
    selected: Option<usize>,
    title: SharedString,
    empty_text: SharedString,
    on_select: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_read: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_remove: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_mark_all_read: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_dismiss: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl NotificationCenter {
    pub fn new(id: impl Into<ElementId>, items: Vec<NotificationItem>) -> Self {
        Self {
            id: id.into(),
            items,
            open: false,
            selected: None,
            title: "通知中心".into(),
            empty_text: "暂无通知".into(),
            on_select: None,
            on_read: None,
            on_remove: None,
            on_mark_all_read: None,
            on_dismiss: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected.filter(|index| *index < self.items.len());
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn empty_text(mut self, text: impl Into<SharedString>) -> Self {
        self.empty_text = text.into();
        self
    }

    pub fn on_select(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }

    pub fn on_read(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_read = Some(Rc::new(listener));
        self
    }

    pub fn on_remove(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(listener));
        self
    }

    pub fn on_mark_all_read(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_mark_all_read = Some(Rc::new(listener));
        self
    }

    pub fn on_dismiss(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for NotificationCenter {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.open {
            return deferred(div());
        }

        let owner = format!("{:?}", self.id);
        let root_id = format!("{owner}-panel");
        let unread = unread_count(&self.items);
        let selected = self.selected;
        let item_count = self.items.len();
        let (m, cs, tokens) = theme::with(|t| {
            (
                metric::notification_metric(t),
                metric::notice_style(t),
                t.clone(),
            )
        });
        let row_height = notification_center_row_height(&tokens);
        let scroll = window
            .use_keyed_state(
                ElementId::Name(format!("{owner}-scroll").into()),
                cx,
                |_, _| UniformListScrollHandle::new(),
            )
            .read(cx)
            .clone();

        let mut header = div()
            .id(ElementId::Name(format!("{owner}-header").into()))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(tokens.margin_sm))
            .h(px(tokens.control_height_lg))
            .px(px(m.padding_horizontal))
            .border_b_1()
            .border_color(tokens.color_border_secondary)
            .child(
                div()
                    .flex_1()
                    .text_size(px(m.title_font))
                    .line_height(px(m.title_line))
                    .text_color(cs.title_fg)
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_size(px(m.desc_font))
                    .text_color(tokens.color_text_tertiary)
                    .child(format!("{unread} 条未读")),
            );
        if unread > 0 {
            if let Some(listener) = &self.on_mark_all_read {
                let listener = listener.clone();
                header = header.child(
                    Button::new("全部已读")
                        .id(format!("{owner}-mark-all-read"))
                        .small()
                        .ghost_bg(true)
                        .on_click(move |_, window, cx| listener(window, cx)),
                );
            }
        }
        if let Some(listener) = &self.on_dismiss {
            let listener = listener.clone();
            header = header.child(
                Button::new("关闭")
                    .id(format!("{owner}-dismiss"))
                    .small()
                    .ghost_bg(true)
                    .on_click(move |_, window, cx| listener(window, cx)),
            );
        }

        let mut root = div()
            .id(ElementId::Name(root_id.clone().into()))
            .absolute()
            .top_0()
            .right_0()
            .h_full()
            .w(px(m.width))
            .flex()
            .flex_col()
            .bg(cs.bg)
            .border_l_1()
            .border_color(tokens.color_border_secondary)
            .shadow(tokens.shadow_secondary.0.clone())
            .child(header);

        if self.items.is_empty() {
            root = root.child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(tokens.color_text_tertiary)
                    .child(self.empty_text.clone()),
            );
        } else {
            let items = self.items.clone();
            let on_select = self.on_select.clone();
            let on_read = self.on_read.clone();
            let on_remove = self.on_remove.clone();
            let owner_for_rows = owner.clone();
            let rows = uniform_list(
                ElementId::Name(format!("{owner}-rows").into()),
                item_count,
                move |range, _, _| {
                    range
                        .map(|index| {
                            let item = &items[index];
                            let is_selected = selected == Some(index);
                            let is_read = item.is_read();
                            let row_bg = if is_selected {
                                tokens.color_primary_bg
                            } else {
                                gpui::transparent_black()
                            };
                            let row_hover = if is_selected {
                                tokens.color_primary_bg_hover
                            } else {
                                tokens.color_fill_tertiary
                            };
                            let title_fg = if is_read {
                                tokens.color_text_secondary
                            } else {
                                cs.title_fg
                            };
                            let icon_color = match item.variant() {
                                NotificationVariant::Info => cs.icon_info,
                                NotificationVariant::Success => cs.icon_success,
                                NotificationVariant::Warning => cs.icon_warning,
                                NotificationVariant::Danger => cs.icon_error,
                            };
                            let mut content = div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .gap(px(m.section_gap));
                            let mut title_row = div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(tokens.margin_xs));
                            if !is_read {
                                title_row = title_row.child(
                                    div()
                                        .w(px(tokens.font_size_sm * 0.5))
                                        .h(px(tokens.font_size_sm * 0.5))
                                        .rounded_full()
                                        .bg(tokens.color_primary)
                                        .flex_shrink_0(),
                                );
                            }
                            title_row = title_row.child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .whitespace_nowrap()
                                    .text_ellipsis()
                                    .text_size(px(m.title_font))
                                    .line_height(px(m.title_line))
                                    .text_color(title_fg)
                                    .child(item.title().clone()),
                            );
                            if let Some(timestamp) = item.timestamp_text() {
                                title_row = title_row.child(
                                    div()
                                        .flex_shrink_0()
                                        .text_size(px(tokens.font_size_sm))
                                        .text_color(tokens.color_text_tertiary)
                                        .child(timestamp.clone()),
                                );
                            }
                            content = content.child(title_row);
                            if let Some(description) = item.description_text() {
                                content = content.child(
                                    div()
                                        .w_full()
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .text_size(px(m.desc_font))
                                        .line_height(px(m.line))
                                        .text_color(cs.text_fg)
                                        .child(description.clone()),
                                );
                            }

                            let mut row = div()
                                .id(ElementId::Name(
                                    format!("{owner_for_rows}-item-{index}").into(),
                                ))
                                .w_full()
                                .h(px(row_height))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(m.icon_gap))
                                .px(px(m.padding_horizontal))
                                .py(px(tokens.padding_xs))
                                .bg(row_bg)
                                .cursor_pointer()
                                .child(icon_canvas(
                                    variant_icon(item.variant()),
                                    m.icon_size,
                                    icon_color,
                                ))
                                .child(content);
                            if !is_selected {
                                row = row.hover(|style| style.bg(row_hover));
                            }
                            let row = Semantics::new(Role::ListItem, item.title().clone())
                                .id(format!("{owner_for_rows}-item-{index}"))
                                .selected(is_selected)
                                .focusable(true)
                                .apply(row);
                            let row = a11y::click_void(row, {
                                let on_select = on_select.clone();
                                let on_read = on_read.clone();
                                move |window, cx| {
                                    if let Some(listener) = &on_select {
                                        listener(&index, window, cx);
                                    }
                                    if !is_read {
                                        if let Some(listener) = &on_read {
                                            listener(&index, window, cx);
                                        }
                                    }
                                }
                            });
                            let row = row.on_key_down({
                                let on_select = on_select.clone();
                                let on_read = on_read.clone();
                                move |event, window, cx| {
                                    if event.keystroke.key == "enter" {
                                        cx.stop_propagation();
                                        if let Some(listener) = &on_select {
                                            listener(&index, window, cx);
                                        }
                                        if !is_read {
                                            if let Some(listener) = &on_read {
                                                listener(&index, window, cx);
                                            }
                                        }
                                    }
                                }
                            });
                            if let Some(listener) = &on_remove {
                                let listener = listener.clone();
                                row.child(
                                    Button::new("✕")
                                        .id(format!("{owner_for_rows}-remove-{index}"))
                                        .small()
                                        .ghost_bg(true)
                                        .on_click(move |_, window, cx| {
                                            cx.stop_propagation();
                                            listener(&index, window, cx);
                                        }),
                                )
                            } else {
                                row
                            }
                        })
                        .collect::<Vec<_>>()
                },
            )
            .size_full()
            .track_scroll(&scroll);
            let scrollbar = Scrollbar::vertical(
                ElementId::Name(format!("{owner}-scrollbar").into()),
                scroll.0.borrow().base_handle.clone(),
            );
            root = root.child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .overflow_hidden()
                    .child(rows)
                    .child(div().absolute().top_0().right_0().h_full().child(scrollbar)),
            );
        }

        let on_select = self.on_select.clone();
        let on_dismiss = self.on_dismiss.clone();
        let key_scroll = scroll.clone();
        let root = root.on_key_down(move |event, window, cx| {
            let key = event.keystroke.key.as_str();
            let key = key.strip_prefix("arrow").unwrap_or(key);
            if key == "escape" {
                cx.stop_propagation();
                if let Some(listener) = &on_dismiss {
                    listener(window, cx);
                }
                return;
            }
            if let Some(target) = notification_center_target(selected, key, item_count) {
                cx.stop_propagation();
                if let Some(listener) = &on_select {
                    listener(&target, window, cx);
                }
                key_scroll.scroll_to_item(target, ScrollStrategy::Nearest);
            }
        });
        deferred(
            Semantics::new(Role::Dialog, self.title)
                .id(root_id)
                .apply(root),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::notification_center_target;

    #[test]
    fn notification_navigation_wraps_and_supports_arrow_prefixes() {
        assert_eq!(notification_center_target(None, "down", 3), Some(0));
        assert_eq!(notification_center_target(None, "arrowup", 3), Some(2));
        assert_eq!(notification_center_target(Some(0), "up", 3), Some(2));
        assert_eq!(notification_center_target(Some(2), "down", 3), Some(0));
        assert_eq!(notification_center_target(Some(1), "home", 3), Some(0));
        assert_eq!(notification_center_target(Some(1), "end", 3), Some(2));
        assert_eq!(notification_center_target(Some(1), "left", 3), None);
        assert_eq!(notification_center_target(Some(1), "down", 0), None);
    }
}
