//! 标签页控件：水平页签 + 下划线/卡片两种形态，状态由父视图持有。

use std::rc::Rc;

use gpui::{
    App, Context, CursorStyle, Div, ElementId, Hsla, IntoElement, MouseButton, Render, Role,
    SharedString, Stateful, Window, div, prelude::*, px, transparent_black,
};

use crate::a11y;
use crate::theme::{self, Tokens, metric};

/// 页签形态：下划线（默认）/ 卡片。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabsKind {
    Line,
    Card,
}

/// 页签交互状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabsState {
    Normal,
    Hover,
    Active,
    Selected,
    Disabled,
    SelectedDisabled,
}

/// 页签的纯数据外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsItemStyle {
    pub text: Hsla,
    pub bg: Hsla,
    pub border: Hsla,
    /// 下划线风格的指示条是否落在这一项
    pub ink: bool,
}

/// 页签状态：文字 `colorText` → 悬停 `colorPrimaryHover` → 按下 `colorPrimaryActive`，
/// 选中 `colorPrimary`，禁用 `colorTextDisabled`；卡片底色 `colorFillAlter`，
/// 选中/选中禁用改用 `colorBgContainer`，边框 `colorBorderSecondary`。
pub fn tabs_item_state(tokens: &Tokens, kind: TabsKind, state: TabsState) -> TabsItemStyle {
    let (text, selected) = match state {
        TabsState::Normal => (tokens.color_text, false),
        TabsState::Hover => (tokens.color_primary_hover, false),
        TabsState::Active => (tokens.color_primary_active, false),
        TabsState::Selected => (tokens.color_primary, true),
        TabsState::Disabled => (tokens.color_text_disabled, false),
        TabsState::SelectedDisabled => (tokens.color_text_disabled, true),
    };
    let (bg, border) = match kind {
        TabsKind::Line => (transparent_black(), transparent_black()),
        TabsKind::Card => (
            if selected {
                tokens.color_bg_container
            } else {
                tokens.color_fill_alter
            },
            tokens.color_border_secondary,
        ),
    };
    TabsItemStyle {
        text,
        bg,
        border,
        ink: selected,
    }
}

/// 导航底部分隔线颜色（`colorBorderSecondary`）。
pub fn tabs_rail_state(tokens: &Tokens) -> Hsla {
    tokens.color_border_secondary
}

/// 下划线指示条颜色（`colorPrimary`）。
pub fn tabs_ink_state(tokens: &Tokens) -> Hsla {
    tokens.color_primary
}

/// 通用标签页。
///
/// ```ignore
/// Tabs::new(["概览", "详情", "设置"], self.tab)
///     .on_select(cx.listener(|this, ix, _, cx| {
///         this.tab = *ix;
///         cx.notify();
///     }))
/// ```
pub struct Tabs {
    id: Option<ElementId>,
    origin: SharedString,
    items: Vec<SharedString>,
    active: usize,
    disabled_tabs: Vec<usize>,
    closable: bool,
    card: bool,
    centered: bool,
    draggable: bool,
    small: bool,
    large: bool,
    on_select: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_close: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_reorder: Option<Rc<dyn Fn(&(usize, usize), &mut Window, &mut App)>>,
}

impl Tabs {
    #[track_caller]
    pub fn new(items: impl IntoIterator<Item = impl Into<SharedString>>, active: usize) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "tabs-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            items: items.into_iter().map(Into::into).collect(),
            active,
            disabled_tabs: Vec::new(),
            closable: false,
            card: false,
            centered: false,
            draggable: false,
            small: false,
            large: false,
            on_select: None,
            on_close: None,
            on_reorder: None,
        }
    }

    /// 实例 id：同页多个 Tabs 时用于区分元素 id（可拖拽时建议设置不同 id）
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn on_select(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }

    /// 禁用的页签下标集合
    pub fn disabled_tabs(mut self, tabs: Vec<usize>) -> Self {
        self.disabled_tabs = tabs;
        self
    }

    /// 每个页签显示 ✕，点击触发 on_close（不自动切换选中）
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn on_close(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(listener));
        self
    }

    /// 卡片式页签（圆角顶边框盒子，替代下划线风格）
    pub fn card(mut self) -> Self {
        self.card = true;
        self
    }

    /// 页签居中排列（默认靠左）
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    /// 允许拖拽页签排序（拖起时显示标签幽灵，释放到目标页签时触发 on_reorder）
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// 小尺寸（块向内边距 8，卡片高 32）
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    /// 大尺寸（字号 16、行高 24，块向内边距 16，卡片高 48）
    pub fn large(mut self) -> Self {
        self.large = true;
        self
    }

    /// 页签被拖放后回调 (from, to)：表示把 from 位置的页签移动到 to 位置
    /// （页签顺序由调用方持有并重排）
    pub fn on_reorder(
        mut self,
        listener: impl Fn(&(usize, usize), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_reorder = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Tabs {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        theme::with(|tokens| {
            let m = metric::TabsMetric::resolve(tokens);
            let kind = if self.card {
                TabsKind::Card
            } else {
                TabsKind::Line
            };
            let size = if self.small {
                0
            } else if self.large {
                2
            } else {
                1
            };
            let rail = tabs_rail_state(tokens);
            let ink = tabs_ink_state(tokens);

            let item_count = self.items.len();
            let disabled_tabs = self.disabled_tabs.clone();
            // 拖拽载荷按实例归属标记，避免跨 Tabs 实例误命中
            let prefix = self
                .id
                .as_ref()
                .map(|id| format!("{id:?}"))
                .unwrap_or_default();
            let owner: SharedString = if prefix.is_empty() {
                self.origin.clone()
            } else {
                prefix.clone().into()
            };
            let tab_list_id = format!("{owner}-tab-list");
            let mut bar = a11y::Semantics::new(Role::TabList, "标签页")
                .id(tab_list_id.clone())
                .apply(div().id(ElementId::Name(tab_list_id.into())))
                .flex()
                .flex_row()
                .items_center()
                .border_b(px(m.divider_width))
                .border_color(rail);
            if self.centered {
                bar = bar.justify_center();
            }

            for (ix, label) in self.items.into_iter().enumerate() {
                let is_active = ix == self.active;
                let is_disabled = self.disabled_tabs.contains(&ix);
                let on_select = self.on_select.clone();
                let on_reorder = self.on_reorder.clone();
                let tab_id = format!("{owner}-tab-{ix}");
                let close_id = format!("{owner}-tab-close-{ix}");

                let state = if is_disabled {
                    if is_active {
                        TabsState::SelectedDisabled
                    } else {
                        TabsState::Disabled
                    }
                } else if is_active {
                    TabsState::Selected
                } else {
                    TabsState::Normal
                };
                let style = tabs_item_state(tokens, kind, state);
                let hover = tabs_item_state(tokens, kind, TabsState::Hover);
                let active = tabs_item_state(tokens, kind, TabsState::Active);

                let mut tab = a11y::Semantics::new(Role::Tab, label.clone())
                    .id(tab_id.clone())
                    .selected(is_active)
                    .disabled(is_disabled)
                    .focusable(!is_disabled && self.on_select.is_some())
                    .position(ix + 1, item_count)
                    .apply(div().id(ElementId::Name(tab_id.into())))
                    .relative()
                    .flex()
                    .flex_row()
                    .items_center()
                    .flex_shrink_0()
                    .text_size(px(m.font_size[size]))
                    .line_height(px(m.line_height[size]))
                    .text_color(style.text);
                if ix > 0 {
                    let gap = if self.card { m.card_gutter } else { m.item_gap };
                    tab = tab.ml(px(gap));
                }

                if self.card {
                    tab = tab
                        .px(px(m.card_padding_inline[size]))
                        .pt(px(m.card_padding_block[size]))
                        .pb(px(m.card_padding_block[size]))
                        .rounded_t(px(m.card_radius[size]))
                        .border_1()
                        .border_color(style.border)
                        .bg(style.bg)
                        .mb(px(-m.divider_width));
                    if style.ink {
                        // 选中卡片的下边框与容器同色，盖住导航分隔线
                        tab = tab.child(
                            div()
                                .absolute()
                                .left_0()
                                .right_0()
                                .bottom(px(-m.divider_width))
                                .h(px(m.divider_width))
                                .bg(style.bg),
                        );
                    }
                } else {
                    tab = tab
                        .pt(px(m.padding_block[size]))
                        .pb(px(m.padding_block[size] - m.ink_height))
                        .border_b(px(m.ink_height))
                        .border_color(if style.ink { ink } else { transparent_black() });
                }

                // 拖拽排序：拖起时渲染标签幽灵，拖到目标页签时高亮并释放回调
                if self.draggable && !is_disabled {
                    let drag = TabDrag {
                        owner: owner.clone(),
                        from: ix,
                        label: label.clone(),
                    };
                    tab = tab.on_drag(drag, |value, _offset, _window, cx| {
                        cx.new(|_| TabDragGhost {
                            label: value.label.clone(),
                        })
                    });

                    if let Some(on_reorder) = on_reorder {
                        let drag_bg = tokens.color_primary_bg;
                        let owner_for_can = owner.clone();
                        let owner_for_drop = owner.clone();
                        tab = tab
                            .drag_over::<TabDrag>(move |style, _drag, _window, _cx| {
                                style.bg(drag_bg)
                            })
                            .can_drop(move |value, _window, _cx| {
                                value
                                    .downcast_ref::<TabDrag>()
                                    .map(|d| d.owner == owner_for_can && d.from != ix)
                                    .unwrap_or(false)
                            })
                            .on_drop::<TabDrag>(move |dragged, window, cx| {
                                if dragged.owner != owner_for_drop || dragged.from == ix {
                                    return;
                                }
                                on_reorder(&(dragged.from, ix), window, cx);
                            });
                    }
                }

                tab = tab.child(label.clone());
                if self.closable && !is_disabled {
                    let on_close = self.on_close.clone();
                    let close_color = tokens.color_icon;
                    let close_hover = tokens.color_text_heading;
                    let mut close =
                        a11y::Semantics::new(Role::Button, format!("关闭标签页 {label}"))
                            .id(close_id.clone())
                            .focusable(self.on_close.is_some())
                            .apply(div().id(ElementId::Name(close_id.into())))
                            .flex()
                            .items_center()
                            .flex_shrink_0()
                            .ml(px(m.close_gap))
                            .text_size(px(m.close_size))
                            .line_height(px(m.close_size))
                            .text_color(close_color)
                            .hover(move |style| {
                                style
                                    .text_color(close_hover)
                                    .text_size(px(m.close_size))
                                    .line_height(px(m.close_size))
                            })
                            .child("✕")
                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                cx.stop_propagation();
                            });
                    if let Some(on_close) = on_close {
                        close = a11y::click_void(close, move |window, cx| {
                            cx.stop_propagation();
                            on_close(&ix, window, cx);
                        });
                    }
                    tab = tab.child(close);
                }

                if is_disabled {
                    tab = tab.cursor(CursorStyle::Arrow);
                } else {
                    tab = tab
                        .cursor(CursorStyle::PointingHand)
                        .hover(move |style| {
                            // gpui 0.2.2 Style.text 非 refineable：状态 refinement 整块替换 text，
                            // 只写色会丢基态字号行高（active 参与布局 ⇒ 按下变大）。
                            style
                                .text_color(hover.text)
                                .text_size(px(m.font_size[size]))
                                .line_height(px(m.line_height[size]))
                        })
                        .active(move |style| {
                            style
                                .text_color(active.text)
                                .text_size(px(m.font_size[size]))
                                .line_height(px(m.line_height[size]))
                        });
                    if let Some(on_select) = on_select {
                        let keyboard = on_select.clone();
                        let disabled = disabled_tabs.clone();
                        tab = a11y::click_void(tab, move |window, cx| {
                            on_select(&ix, window, cx);
                        })
                        .on_key_down(move |event, window, cx| {
                            if let Some(target) =
                                tab_target(ix, event.keystroke.key.as_str(), item_count, &disabled)
                            {
                                if target != ix {
                                    cx.stop_propagation();
                                    keyboard(&target, window, cx);
                                }
                            }
                        });
                    }
                }

                bar = bar.child(tab);
            }

            bar
        })
    }
}

fn tab_target(current: usize, key: &str, len: usize, disabled: &[usize]) -> Option<usize> {
    let key = key.strip_prefix("arrow").unwrap_or(key);
    if len == 0 {
        return None;
    }
    let first = (0..len).find(|ix| !disabled.contains(ix));
    match key {
        "right" | "down" => (1..=len)
            .map(|step| (current + step) % len)
            .find(|ix| !disabled.contains(ix)),
        "left" | "up" => (1..=len)
            .map(|step| (current + len - step % len) % len)
            .find(|ix| !disabled.contains(ix)),
        "home" => first,
        "end" => (0..len).rev().find(|ix| !disabled.contains(ix)),
        _ => None,
    }
}

/// 拖拽载荷（内部类型，on_drop 按此类型匹配）
#[derive(Clone)]
struct TabDrag {
    /// 所属 Tabs 实例标记
    owner: SharedString,
    /// 源页签下标
    from: usize,
    label: SharedString,
}

/// 拖拽跟随的幽灵视图
struct TabDragGhost {
    label: SharedString,
}

impl Render for TabDragGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        theme::with(|tokens| {
            div()
                .px(px(tokens.padding_sm))
                .py(px(tokens.padding_xs))
                .rounded(px(tokens.border_radius))
                .border_1()
                .border_color(tokens.color_primary)
                .bg(tokens.color_bg_container)
                .text_size(px(tokens.font_size))
                .shadow(tokens.shadow_tertiary.0.clone())
                .child(self.label.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::tab_target;

    #[test]
    fn tab_target_skips_disabled_and_wraps() {
        assert_eq!(tab_target(0, "right", 4, &[1]), Some(2));
        assert_eq!(tab_target(0, "arrowright", 4, &[1]), Some(2));
        assert_eq!(tab_target(3, "arrowright", 4, &[]), Some(0));
        assert_eq!(tab_target(0, "arrowleft", 4, &[3]), Some(2));
        assert_eq!(tab_target(2, "arrowleft", 4, &[1]), Some(0));
        assert_eq!(tab_target(2, "home", 4, &[0]), Some(1));
        assert_eq!(tab_target(0, "end", 4, &[3]), Some(2));
        assert_eq!(tab_target(0, "enter", 4, &[]), None);
        assert_eq!(tab_target(0, "arrowright", 0, &[]), None);
        assert_eq!(tab_target(0, "arrowright", 2, &[0, 1]), None);
    }
}
