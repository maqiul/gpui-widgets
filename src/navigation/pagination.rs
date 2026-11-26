//! 分页器：上一页 / 下一页 + 页码（页数多时自动省略号折叠）。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, FontWeight, Hsla, IntoElement, Role, Stateful, Window, div,
    prelude::*, px, transparent_black,
};

use crate::a11y;
use crate::theme::{self, Tokens, metric};

/// 页码项 / 翻页按钮的外观（antdv-next Pagination 实测）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaginationStyle {
    pub bg: Hsla,
    pub fg: Hsla,
    pub border: Hsla,
    pub hover_bg: Hsla,
    pub hover_fg: Hsla,
    pub hover_border: Hsla,
    pub size: f32,
    pub spacing: f32,
    pub radius: f32,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub line_width: f32,
    pub padding_inline: f32,
}

/// 页码项：`colorBgContainer` 底 + 透明描边；选中态文字/描边取主色、字重 600、
/// 底色仍是容器色；非选中项悬停底色 `colorBgTextHover`。
pub fn item_state(tokens: &Tokens, small: bool, active: bool, disabled: bool) -> PaginationStyle {
    let m = metric::PaginationMetric::resolve(tokens);
    let ix = usize::from(small);
    let fg = if disabled {
        tokens.color_text_disabled
    } else if active {
        tokens.color_primary
    } else {
        tokens.color_text
    };
    PaginationStyle {
        bg: tokens.color_bg_container,
        fg,
        border: if active && !disabled {
            tokens.color_primary
        } else {
            transparent_black()
        },
        hover_bg: if disabled || active {
            tokens.color_bg_container
        } else {
            tokens.color_bg_text_hover
        },
        hover_fg: if disabled {
            tokens.color_text_disabled
        } else if active {
            tokens.color_primary_hover
        } else {
            tokens.color_text
        },
        hover_border: if active && !disabled {
            tokens.color_primary_hover
        } else {
            transparent_black()
        },
        size: m.size[ix],
        spacing: m.spacing[ix],
        radius: m.radius,
        font_size: m.font_size,
        font_weight: if active {
            m.active_font_weight
        } else {
            m.font_weight
        },
        line_width: m.line_width,
        padding_inline: m.padding_inline,
    }
}

/// 上一页 / 下一页：无底色、透明描边；悬停底色 `colorBgTextHover`，
/// 禁用态文字 `colorTextDisabled` 且 **opacity 保持 1**。
pub fn nav_state(tokens: &Tokens, small: bool, disabled: bool) -> PaginationStyle {
    let m = metric::PaginationMetric::resolve(tokens);
    let ix = usize::from(small);
    PaginationStyle {
        bg: transparent_black(),
        fg: if disabled {
            tokens.color_text_disabled
        } else {
            tokens.color_text
        },
        border: transparent_black(),
        hover_bg: if disabled {
            transparent_black()
        } else {
            tokens.color_bg_text_hover
        },
        hover_fg: if disabled {
            tokens.color_text_disabled
        } else {
            tokens.color_text
        },
        hover_border: transparent_black(),
        size: m.size[ix],
        spacing: m.spacing[ix],
        radius: m.radius,
        font_size: m.font_size,
        font_weight: m.font_weight,
        line_width: m.line_width,
        padding_inline: m.padding_inline,
    }
}

/// 分页器（页码从 1 开始）。
///
/// ```ignore
/// Pagination::new("list-page", self.page, 12)
///     .on_change(cx.listener(|this, page, _, cx| {
///         this.page = *page;
///         cx.notify();
///     }))
/// ```
pub struct Pagination {
    id: ElementId,
    current: usize,
    total: usize,
    simple: bool,
    small: bool,
    on_change: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl Pagination {
    pub fn new(id: impl Into<ElementId>, current: usize, total: usize) -> Self {
        Self {
            id: id.into(),
            current: current.max(1),
            total: total.max(1),
            simple: false,
            small: false,
            on_change: None,
        }
    }

    /// 简洁模式：仅上一页 / 「当前 / 总页数」/ 下一页
    pub fn simple(mut self) -> Self {
        self.simple = true;
        self
    }

    /// 小尺寸（24px 按钮）
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }
}

/// 生成页码序列，None 表示省略号
fn page_items(current: usize, total: usize) -> Vec<Option<usize>> {
    if total <= 7 {
        return (1..=total).map(Some).collect();
    }
    let mut pages = vec![1];
    let lo = current.saturating_sub(1).max(2);
    let hi = (current + 1).min(total - 1);
    if lo > 2 {
        pages.push(0);
    }
    pages.extend(lo..=hi);
    if hi < total - 1 {
        pages.push(0);
    }
    pages.push(total);
    pages
        .into_iter()
        .map(|p| if p == 0 { None } else { Some(p) })
        .collect()
}

impl IntoElement for Pagination {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let current = self.current.min(self.total);
        let spacing = theme::with(|t| {
            let m = metric::PaginationMetric::resolve(t);
            m.spacing[usize::from(self.small)]
        });
        let root_id = format!("{:?}-pagination", self.id);
        let mut root = a11y::Semantics::new(Role::Navigation, "分页")
            .id(root_id.clone())
            .apply(div().id(ElementId::Name(root_id.into())))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing));

        // 上一页
        root = root.child(nav_button(
            ElementId::Name(format!("{:?}-prev", self.id).into()),
            "‹",
            "上一页",
            self.small,
            current <= 1,
            self.on_change.is_some(),
            {
                let on_change = self.on_change.clone();
                let target = current - 1;
                move |window, cx| {
                    if let Some(on_change) = &on_change {
                        on_change(&target, window, cx);
                    }
                }
            },
        ));

        if self.simple {
            let style = theme::with(|t| item_state(t, self.small, false, false));
            // 简洁模式：当前 / 总页数
            root = root.child(
                a11y::Semantics::new(Role::Status, "分页状态")
                    .id(format!("{:?}-status", self.id))
                    .value(format!("{current} / {}", self.total))
                    .apply(
                        div()
                            .id(ElementId::Name(format!("{:?}-status", self.id).into()))
                            .px(px(style.padding_inline))
                            .text_size(px(style.font_size))
                            .text_color(theme::text())
                            .child(format!("{current} / {}", self.total)),
                    ),
            );
        } else {
            let ellipsis_style = theme::with(|t| item_state(t, self.small, false, true));
            let pages = page_items(current, self.total);
            let page_count = pages.len();
            for (ix, page) in pages.into_iter().enumerate() {
                match page {
                    None => {
                        root = root.child(
                            div()
                                .w(px(ellipsis_style.size))
                                .text_center()
                                .text_size(px(ellipsis_style.font_size))
                                .text_color(ellipsis_style.fg)
                                .child("…"),
                        );
                    }
                    Some(page) => {
                        let active = page == current;
                        let on_change = self.on_change.clone();
                        let style = theme::with(|t| item_state(t, self.small, active, false));
                        let button_id = format!("{:?}-page-{ix}", self.id);
                        let mut button =
                            a11y::Semantics::new(Role::Button, format!("第 {page} 页"))
                                .id(button_id.clone())
                                .selected(active)
                                .focusable(!active && self.on_change.is_some())
                                .position(ix + 1, page_count)
                                .apply(div().id(ElementId::Name(button_id.into())))
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(style.size))
                                .h(px(style.size))
                                .rounded(px(style.radius))
                                .border(px(style.line_width))
                                .border_color(style.border)
                                .bg(style.bg)
                                .text_size(px(style.font_size))
                                .font_weight(style.font_weight)
                                .text_color(style.fg)
                                .hover(move |s| {
                                    s.bg(style.hover_bg)
                                        .text_color(style.hover_fg)
                                        .border_color(style.hover_border)
                                })
                                .child(format!("{page}"));
                        if !active {
                            let page = page;
                            button = button.cursor(CursorStyle::PointingHand);
                            if let Some(on_change) = on_change {
                                button = a11y::click_void(button, move |window, cx| {
                                    on_change(&page, window, cx);
                                });
                            }
                        }
                        root = root.child(button);
                    }
                }
            }
        }

        // 下一页
        root = root.child(nav_button(
            ElementId::Name(format!("{:?}-next", self.id).into()),
            "›",
            "下一页",
            self.small,
            current >= self.total,
            self.on_change.is_some(),
            {
                let on_change = self.on_change.clone();
                let target = current + 1;
                move |window, cx| {
                    if let Some(on_change) = &on_change {
                        on_change(&target, window, cx);
                    }
                }
            },
        ));

        root
    }
}

/// 上一页 / 下一页按钮（边界置灰不响应）
fn nav_button(
    id: ElementId,
    label: &'static str,
    aria_label: &'static str,
    small: bool,
    disabled: bool,
    interactive: bool,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let style = theme::with(|t| nav_state(t, small, disabled));
    let semantics_id = format!("{id:?}-nav-button");
    let mut button = a11y::Semantics::new(Role::Button, aria_label)
        .id(semantics_id.clone())
        .disabled(disabled)
        .focusable(!disabled && interactive)
        .apply(div().id(id).accessibility_id(semantics_id))
        .flex()
        .items_center()
        .justify_center()
        .w(px(style.size))
        .h(px(style.size))
        .rounded(px(style.radius))
        .border(px(style.line_width))
        .border_color(style.border)
        .bg(style.bg)
        .text_size(px(style.font_size))
        .text_color(style.fg)
        .child(label);
    if disabled {
        button = button.cursor(CursorStyle::Arrow);
    } else {
        button = button
            .cursor(CursorStyle::PointingHand)
            .hover(move |s| s.bg(style.hover_bg).text_color(style.hover_fg));
        if interactive {
            button = a11y::click_void(button, on_click);
        }
    }
    button
}
