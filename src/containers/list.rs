//! 列表：带头像 / 描述 / 额外元素的条目列表（对应 antd List）。

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, CursorStyle, Div, ElementId, FocusHandle, IntoElement, Pixels, Role,
    ScrollStrategy, SharedString, Stateful, UniformListScrollHandle, Window, div, prelude::*, px,
    uniform_list,
};

use crate::a11y;
use crate::scrollbar::Scrollbar;
use crate::theme;

/// 列表条目
pub struct ListItem {
    title: SharedString,
    description: Option<SharedString>,
    avatar: Option<SharedString>,
    extra: Option<AnyElement>,
}

impl ListItem {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            avatar: None,
            extra: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 左侧圆形头像（字符 / emoji）
    pub fn avatar(mut self, avatar: impl Into<SharedString>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }

    /// 右侧额外元素
    pub fn extra(mut self, extra: impl IntoElement) -> Self {
        self.extra = Some(extra.into_any_element());
        self
    }
}

/// 虚拟列表配置：固定视口 / 统一行高 / 按索引生成行
struct VirtualList {
    height: Pixels,
    row_height: Pixels,
    count: usize,
    row: Rc<dyn Fn(usize) -> ListItem>,
    scroll: Option<UniformListScrollHandle>,
}

pub fn list_navigation_target(current: Option<usize>, key: &str, len: usize) -> Option<usize> {
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

/// 通用列表。
///
/// ```ignore
/// List::new(vec![
///     ListItem::new("条目一").description("说明文字").avatar("①"),
///     ListItem::new("条目二").extra(Badge::new("新").info()),
/// ])
/// .header("列表标题")
/// ```
#[derive(IntoElement)]
pub struct List {
    id: Option<SharedString>,
    origin: SharedString,
    header: Option<SharedString>,
    footer: Option<SharedString>,
    items: Vec<ListItem>,
    virtualized: Option<VirtualList>,
    selected: Option<usize>,
    on_select: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl List {
    #[track_caller]
    pub fn new(items: Vec<ListItem>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "list-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            header: None,
            footer: None,
            items,
            virtualized: None,
            selected: None,
            on_select: None,
        }
    }

    /// 虚拟列表：只渲染视口内的行，适合万级数据。
    ///
    /// - `height`：表体视口高度（不含头部/底部）；
    /// - `row_height`：统一行高。虚拟列表按统一行高布局，两行（含 description）
    ///   或带头像的行请显式给足高度（建议 ≥ 56px），否则内容可能被裁切；
    /// - `count` + `row`：按全局行下标生成条目，`row` 会在滚动/测量时被多次调用。
    #[track_caller]
    pub fn virtualized(
        height: impl Into<Pixels>,
        row_height: impl Into<Pixels>,
        count: usize,
        row: impl Fn(usize) -> ListItem + 'static,
    ) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "list-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            header: None,
            footer: None,
            items: Vec::new(),
            virtualized: Some(VirtualList {
                height: height.into(),
                row_height: row_height.into(),
                count,
                row: Rc::new(row),
                scroll: None,
            }),
            selected: None,
            on_select: None,
        }
    }

    /// 复用外部滚动句柄（可选）：可据此做回到顶部、滚到某行等控制。
    pub fn virtual_scroll_handle(mut self, handle: UniformListScrollHandle) -> Self {
        if let Some(virtualized) = &mut self.virtualized {
            virtualized.scroll = Some(handle);
        }
        self
    }

    /// 实例 ID；同页多个列表应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn footer(mut self, footer: impl Into<SharedString>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_select(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for List {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (m, padding_sm, padding) = theme::with(|t| {
            (
                crate::theme::metric::list_metric(t),
                t.padding_sm,
                t.padding,
            )
        });
        let list_id = self
            .id
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| self.origin.to_string());
        let item_prefix = list_id.clone();
        let item_count = self
            .virtualized
            .as_ref()
            .map_or(self.items.len(), |virtualized| virtualized.count);
        let selected = self.selected;
        let on_select = self.on_select.clone();
        let navigation_enabled = on_select.is_some() && item_count > 0;
        let navigation_focus = navigation_enabled.then(|| {
            window
                .use_keyed_state(
                    ElementId::Name(format!("{list_id}-navigation-focus").into()),
                    cx,
                    |_, cx| cx.focus_handle(),
                )
                .read(cx)
                .clone()
        });
        let virtual_scroll = self.virtualized.as_ref().map(|_| {
            self.virtualized
                .as_ref()
                .and_then(|virtualized| virtualized.scroll.clone())
                .unwrap_or_else(|| {
                    window
                        .use_keyed_state(
                            ElementId::Name(format!("{list_id}-vscroll").into()),
                            cx,
                            |_, _| UniformListScrollHandle::new(),
                        )
                        .read(cx)
                        .clone()
                })
        });
        let mut list = a11y::Semantics::new(
            Role::List,
            self.header.clone().unwrap_or_else(|| "列表".into()),
        )
        .id(list_id.clone())
        .focusable(navigation_enabled)
        .apply(div().id(ElementId::Name(list_id.clone().into())))
        .flex()
        .flex_col()
        .w_full()
        .bg(theme::bg_container())
        .border_1()
        .border_color(theme::border())
        .rounded_lg()
        .overflow_hidden();
        if let Some(focus) = &navigation_focus {
            list = list.track_focus(focus);
        }
        let navigation_current = navigation_enabled
            .then(|| Rc::new(Cell::new(selected.filter(|index| *index < item_count))));
        if navigation_enabled {
            let nav_on_select = on_select.clone();
            let nav_scroll = virtual_scroll.clone();
            let nav_focus = navigation_focus.clone();
            let nav_current = navigation_current.clone().expect("navigation is enabled");
            list = list.on_key_down(move |event, window, cx| {
                if !nav_focus
                    .as_ref()
                    .is_some_and(|focus| focus.is_focused(window))
                {
                    return;
                }
                let Some(target) = list_navigation_target(
                    nav_current.get(),
                    event.keystroke.key.as_str(),
                    item_count,
                ) else {
                    return;
                };
                cx.stop_propagation();
                nav_current.set(Some(target));
                if let Some(on_select) = &nav_on_select {
                    on_select(&target, window, cx);
                }
                if let Some(scroll) = &nav_scroll {
                    scroll.scroll_to_item(target, ScrollStrategy::Nearest);
                }
            });
        }

        if let Some(header) = &self.header {
            list = list.child(
                a11y::Semantics::new(Role::Heading, header.clone())
                    .id(format!("{list_id}-heading"))
                    .level(2)
                    .apply(div().id(ElementId::Name(format!("{list_id}-heading").into())))
                    .px(px(padding))
                    .py(px(padding_sm))
                    .border_b_1()
                    .border_color(theme::border())
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme::text())
                    .child(header.clone()),
            );
        }

        if let Some(virt) = self.virtualized {
            let count = virt.count;
            let row_height = virt.row_height;
            let row = virt.row.clone();
            let row_navigation_scroll = virtual_scroll.clone();
            let scroll = virtual_scroll.expect("virtualized list has a scroll handle");
            let bar = Scrollbar::vertical(
                ElementId::Name(format!("{list_id}-vbar").into()),
                scroll.0.borrow().base_handle.clone(),
            );
            let prefix = item_prefix.clone();
            let row_on_select = on_select.clone();
            let row_navigation_focus = navigation_focus.clone();
            let rows = uniform_list(
                ElementId::Name(format!("{list_id}-vbody").into()),
                count,
                move |range, _, _| {
                    range
                        .map(|ix| {
                            row_element(
                                row(ix),
                                ix,
                                count,
                                &prefix,
                                Some(row_height),
                                m.avatar_size,
                                padding,
                                padding_sm,
                                selected,
                                row_on_select.clone(),
                                row_navigation_focus.clone(),
                                navigation_current.clone(),
                                row_navigation_scroll.clone(),
                            )
                        })
                        .collect::<Vec<_>>()
                },
            )
            .size_full()
            .track_scroll(&scroll);
            list = list.child(
                div()
                    .relative()
                    .w_full()
                    .h(virt.height)
                    .overflow_hidden()
                    .child(rows)
                    .child(div().absolute().top_0().right_0().h_full().child(bar)),
            );
        } else {
            let count = self.items.len();
            for (ix, item) in self.items.into_iter().enumerate() {
                list = list.child(row_element(
                    item,
                    ix,
                    count,
                    &item_prefix,
                    None,
                    m.avatar_size,
                    padding,
                    padding_sm,
                    selected,
                    on_select.clone(),
                    navigation_focus.clone(),
                    navigation_current.clone(),
                    virtual_scroll.clone(),
                ));
            }
        }

        if let Some(footer) = &self.footer {
            list = list.child(
                div()
                    .px(px(padding))
                    .py(px(m.footer_padding_block))
                    .border_t_1()
                    .border_color(theme::border())
                    .text_xs()
                    .text_color(theme::text_secondary())
                    .child(footer.clone()),
            );
        }

        list
    }
}

#[allow(clippy::too_many_arguments)]
fn row_element(
    item: ListItem,
    ix: usize,
    count: usize,
    item_prefix: &str,
    height: Option<Pixels>,
    avatar_size: f32,
    padding: f32,
    padding_sm: f32,
    selected: Option<usize>,
    on_select: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    navigation_focus: Option<FocusHandle>,
    navigation_current: Option<Rc<Cell<Option<usize>>>>,
    navigation_scroll: Option<UniformListScrollHandle>,
) -> Stateful<Div> {
    let item_id = format!("{item_prefix}-item-{ix}");
    let mut semantics = a11y::Semantics::new(Role::ListItem, item.title.clone())
        .id(item_id.clone())
        .position(ix + 1, count)
        .selected(selected == Some(ix))
        .focusable(on_select.is_some());
    if let Some(description) = &item.description {
        semantics = semantics.description(description.clone());
    }
    let mut row = semantics
        .apply(div().id(ElementId::Name(item_id.into())))
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px(px(padding))
        .py(px(padding_sm));
    if selected == Some(ix) {
        row = row.bg(theme::primary_bg());
    }
    if let Some(on_select) = on_select {
        row = row.cursor(CursorStyle::PointingHand).hover(move |style| {
            if selected == Some(ix) {
                style
            } else {
                style.bg(theme::with(|t| t.color_fill_tertiary))
            }
        });
        let keyboard_on_select = on_select.clone();
        let keyboard_scroll = navigation_scroll.clone();
        let keyboard_current = navigation_current.clone();
        let keyboard_focus = navigation_focus.clone();
        row = row.on_key_down(move |event, window, cx| {
            let Some(target) =
                list_navigation_target(Some(ix), event.keystroke.key.as_str(), count)
            else {
                return;
            };
            cx.stop_propagation();
            if let Some(current) = &keyboard_current {
                current.set(Some(target));
            }
            keyboard_on_select(&target, window, cx);
            if let Some(scroll) = &keyboard_scroll {
                scroll.scroll_to_item(target, ScrollStrategy::Nearest);
            }
            if let Some(focus) = &keyboard_focus {
                focus.focus(window, cx);
            }
        });
        let click_current = navigation_current.clone();
        row = a11y::click_void(row, move |window, cx| {
            if let Some(current) = &click_current {
                current.set(Some(ix));
            }
            if let Some(focus) = &navigation_focus {
                focus.focus(window, cx);
            }
            on_select(&ix, window, cx);
        });
    }
    if ix < count - 1 {
        row = row.border_b_1().border_color(theme::border());
    }
    if let Some(height) = height {
        row = row.h(height).overflow_hidden();
    }

    if let Some(avatar) = &item.avatar {
        row = row.child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(avatar_size))
                .flex_shrink_0()
                .rounded_full()
                .bg(theme::primary_bg())
                .text_color(theme::primary())
                .text_sm()
                .child(avatar.clone()),
        );
    }

    let mut body = div().flex().flex_col().gap_1().flex_1().child(
        div()
            .text_sm()
            .text_color(theme::text())
            .child(item.title.clone()),
    );
    if let Some(description) = &item.description {
        body = body.child(
            div()
                .text_xs()
                .text_color(theme::text_secondary())
                .child(description.clone()),
        );
    }
    row = row.child(body);

    if let Some(extra) = item.extra {
        row = row.child(extra);
    }
    row
}

#[cfg(test)]
mod tests {
    use super::list_navigation_target;

    #[test]
    fn list_navigation_moves_and_jumps_without_wrapping() {
        assert_eq!(list_navigation_target(None, "down", 3), Some(0));
        assert_eq!(list_navigation_target(None, "arrowup", 3), Some(2));
        assert_eq!(list_navigation_target(Some(1), "down", 3), Some(2));
        assert_eq!(list_navigation_target(Some(2), "down", 3), Some(2));
        assert_eq!(list_navigation_target(Some(1), "up", 3), Some(0));
        assert_eq!(list_navigation_target(Some(0), "up", 3), Some(0));
        assert_eq!(list_navigation_target(Some(1), "home", 3), Some(0));
        assert_eq!(list_navigation_target(Some(1), "end", 3), Some(2));
        assert_eq!(list_navigation_target(Some(1), "right", 3), None);
        assert_eq!(list_navigation_target(Some(1), "down", 0), None);
    }
}
