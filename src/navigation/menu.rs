//! 下拉动作菜单：任意触发元素 + 动作项列表（区别于 Select 的“选值”语义）。
//!
//! 展开状态受控（`open` + `on_open_change`），点击菜单外部或动作项后收起。

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, CursorStyle, Div, ElementId, Hsla, IntoElement, Pixels, Role, SharedString,
    Stateful, Window, div, prelude::*, px, transparent_black,
};

use crate::a11y;
use crate::popup::anchored;
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

pub(crate) fn menu_divider() -> Div {
    let style = theme::with(|t| divider_state(t));
    div()
        .h(px(style.width))
        .my(px(style.margin))
        .bg(style.color)
}

fn menu_divider_with_id(id: ElementId) -> Stateful<Div> {
    let style = theme::with(|t| divider_state(t));
    let semantics_id = format!("{id:?}-separator");
    a11y::Semantics::new(Role::GenericContainer, "菜单分隔线")
        .id(semantics_id.clone())
        .orientation(gpui::Orientation::Horizontal)
        .apply(div().id(id).accessibility_id(semantics_id))
        .h(px(style.width))
        .my(px(style.margin))
        .bg(style.color)
}

/// 单个动作项；点击后先执行 `on_click` 再调用 `close`（收起菜单）。
pub(crate) fn menu_item_row(
    id: ElementId,
    item: &MenuItem,
    close: Rc<dyn Fn(&mut Window, &mut App)>,
) -> Stateful<Div> {
    menu_item_row_at(id, item, close, None)
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
pub struct MenuItem {
    pub(crate) label: SharedString,
    pub(crate) danger: bool,
    pub(crate) disabled: bool,
    pub(crate) separator: bool,
    pub(crate) on_click: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl MenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            danger: false,
            disabled: false,
            separator: false,
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

impl IntoElement for DropdownMenu {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let open = self.open;
        let suppress_toggle = Rc::new(Cell::new(false));

        let trigger_id = format!("{:?}-trigger", self.id);
        let mut trigger =
            a11y::Semantics::new(Role::Button, if open { "关闭菜单" } else { "打开菜单" })
                .id(trigger_id.clone())
                .expanded(open)
                .focusable(self.on_open_change.is_some())
                .apply(div().id(ElementId::Name(trigger_id.into())))
                .child(self.trigger);
        if let Some(on_open_change) = self.on_open_change.clone() {
            let suppress_toggle = suppress_toggle.clone();
            trigger = trigger.cursor(CursorStyle::PointingHand);
            trigger = a11y::click_void(trigger, move |window, cx| {
                if suppress_toggle.take() {
                    return;
                }
                on_open_change(&!open, window, cx);
            });
        }

        let mut popup = None;
        if open {
            let mut menu = menu_container(ElementId::Name(format!("{:?}-menu", self.id).into()))
                .w(self.width)
                .on_mouse_down_out({
                    let on_open_change = self.on_open_change.clone();
                    let suppress_toggle = suppress_toggle.clone();
                    move |_, window, cx| {
                        suppress_toggle.set(true);
                        if let Some(on_open_change) = &on_open_change {
                            on_open_change(&false, window, cx);
                        }
                    }
                });

            let item_count = self.items.iter().filter(|item| !item.separator).count();
            let mut item_position = 0;
            for (ix, item) in self.items.iter().enumerate() {
                if item.separator {
                    menu = menu.child(menu_divider_with_id(ElementId::Name(
                        format!("{:?}-separator-{ix}", self.id).into(),
                    )));
                    continue;
                }
                item_position += 1;

                let close = {
                    let on_open_change = self.on_open_change.clone();
                    Rc::new(move |window: &mut Window, cx: &mut App| {
                        if let Some(on_open_change) = &on_open_change {
                            on_open_change(&false, window, cx);
                        }
                    })
                };
                menu = menu.child(menu_item_row_at(
                    ElementId::Name(format!("{:?}-item-{ix}", self.id).into()),
                    item,
                    close,
                    Some((item_position, item_count)),
                ));
            }

            popup = Some(div().child(menu));
        }

        anchored(trigger.into_any_element(), popup)
    }
}
