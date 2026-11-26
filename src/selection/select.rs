//! 下拉选择器：触发按钮 + 浮层菜单（贴触发框下方，自动避免溢出窗口）。
//!
//! 展开状态由父视图持有（`open` + `on_open_change`），
//! 点击菜单外部自动收起。多选见 [`MultiSelect`]。

use std::{cell::Cell, collections::HashMap, rc::Rc};

use gpui::{
    AccessibleAction, App, CursorStyle, Div, ElementId, Entity, Focusable, FontWeight, Hsla,
    IntoElement, Pixels, Role, ScrollHandle, SharedString, Stateful, StatefulInteractiveElement,
    Toggled, Window, div, prelude::*, px, transparent_black,
};

use crate::controls::scrollbar::VerticalScroll;
use crate::text_input::{self, InputSize, InputStatus, InputStyle, InputVariant};
use crate::theme::{self, Tokens, metric};
use crate::{Scrollbar, TextInput, a11y, popup::anchored};

/// antd `fontWeightStrong`（Select 选中项加粗）。
const SELECTED_FONT_WEIGHT: f32 = 600.0;

/// 菜单项外观（纯数据，可直接和浏览器基准比对）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectOptionStyle {
    pub bg: Hsla,
    pub fg: Hsla,
    pub font_weight: FontWeight,
    pub height: f32,
    pub padding_block: f32,
    pub padding_inline: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
}

/// 菜单容器外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectMenuStyle {
    pub bg: Hsla,
    pub radius: f32,
    pub padding: f32,
}

/// 触发框状态矩阵：antdv-next `select/style/select-input.js` 的四变体规则。
///
/// 与 Input 的差异（已用浏览器 computed style 核对）：
/// - 禁用态不改正文色：默认态仍是 `colorText`，error/warning 仍取状态色；
/// - filled 的 error/warning 正文取 `colorError` / `colorWarning`（不是 `color*Text`）；
/// - borderless 聚焦用 `lineWidthFocus` 外扩描边（Input 的 borderless 无聚焦环）。
pub fn state(
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    tokens: &Tokens,
    hover: bool,
    focused: bool,
    disabled: bool,
) -> InputStyle {
    let resolved = metric::SelectMetric::resolve(tokens);
    let index = size.index();
    let transparent = transparent_black();
    let line = resolved.line_width;

    let status_color = match status {
        InputStatus::Default => tokens.color_text,
        InputStatus::Error => tokens.color_error,
        InputStatus::Warning => tokens.color_warning,
    };
    let status_border = match status {
        InputStatus::Default => tokens.color_border,
        InputStatus::Error => tokens.color_error,
        InputStatus::Warning => tokens.color_warning,
    };
    let status_hover_border = match status {
        InputStatus::Default => tokens.color_primary_hover,
        InputStatus::Error => tokens.color_error_border_hover,
        InputStatus::Warning => tokens.color_warning_border_hover,
    };
    let status_focus_border = match status {
        InputStatus::Default => tokens.color_primary,
        InputStatus::Error | InputStatus::Warning => status_border,
    };
    let status_outline = match status {
        InputStatus::Default => tokens.color_primary_outline,
        InputStatus::Error => tokens.color_error_outline,
        InputStatus::Warning => tokens.color_warning_outline,
    };

    let bg = if disabled {
        match variant {
            InputVariant::Borderless => transparent,
            _ => tokens.color_bg_container_disabled,
        }
    } else {
        match variant {
            InputVariant::Outlined | InputVariant::Underlined => tokens.color_bg_container,
            InputVariant::Filled => {
                if focused {
                    tokens.color_bg_container
                } else if hover {
                    match status {
                        InputStatus::Default => tokens.color_fill_secondary,
                        InputStatus::Error => tokens.color_error_bg_hover,
                        InputStatus::Warning => tokens.color_warning_bg_hover,
                    }
                } else {
                    match status {
                        InputStatus::Default => tokens.color_fill_tertiary,
                        InputStatus::Error => tokens.color_error_bg,
                        InputStatus::Warning => tokens.color_warning_bg,
                    }
                }
            }
            InputVariant::Borderless => transparent,
        }
    };

    let border_color = if disabled {
        match variant {
            InputVariant::Borderless => transparent,
            InputVariant::Outlined | InputVariant::Filled => tokens.color_border,
            InputVariant::Underlined => status_border,
        }
    } else if focused {
        match variant {
            InputVariant::Borderless => transparent,
            _ => status_focus_border,
        }
    } else if hover {
        match variant {
            InputVariant::Outlined | InputVariant::Underlined => status_hover_border,
            InputVariant::Filled | InputVariant::Borderless => transparent,
        }
    } else {
        match variant {
            InputVariant::Outlined | InputVariant::Underlined => status_border,
            InputVariant::Filled | InputVariant::Borderless => transparent,
        }
    };

    let border = match variant {
        InputVariant::Borderless => [transparent; 4],
        InputVariant::Underlined => [transparent, transparent, border_color, transparent],
        _ => [border_color; 4],
    };
    let border_width = match variant {
        InputVariant::Underlined => [line, 0.0, line, 0.0],
        _ => [line; 4],
    };

    let shadow = if focused && !disabled {
        match variant {
            InputVariant::Outlined => Some(metric::input_focus_shadow(
                status_outline,
                resolved.outline_width,
            )),
            InputVariant::Borderless => Some(metric::input_focus_shadow(
                status_focus_border,
                tokens.line_width_focus,
            )),
            InputVariant::Filled | InputVariant::Underlined => None,
        }
    } else {
        None
    };

    InputStyle {
        fg: status_color,
        placeholder: tokens.color_text_placeholder,
        affix: tokens.color_text_quaternary,
        bg,
        border,
        border_width,
        shadow,
        height: resolved.height[index],
        padding_block: resolved.padding_block[index],
        padding_inline: resolved.padding_inline[index],
        radius: if variant == InputVariant::Underlined {
            0.0
        } else {
            resolved.radius[index]
        },
        font_size: resolved.font_size[index],
        line_height: resolved.line_height[index],
    }
}

/// 菜单项状态矩阵：选中 / 悬停 / 禁用。
pub fn option_state(
    tokens: &Tokens,
    selected: bool,
    active: bool,
    disabled: bool,
) -> SelectOptionStyle {
    let m = metric::SelectMetric::resolve(tokens);
    let (bg, fg, font_weight) = if disabled {
        (
            if selected {
                tokens.color_bg_container_disabled
            } else {
                transparent_black()
            },
            tokens.color_text_disabled,
            FontWeight::NORMAL,
        )
    } else if selected {
        (
            if active {
                tokens.control_item_bg_active_hover
            } else {
                tokens.control_item_bg_active
            },
            tokens.color_text,
            FontWeight(SELECTED_FONT_WEIGHT),
        )
    } else if active {
        (
            tokens.control_item_bg_hover,
            tokens.color_text,
            FontWeight::NORMAL,
        )
    } else {
        (transparent_black(), tokens.color_text, FontWeight::NORMAL)
    };

    SelectOptionStyle {
        bg,
        fg,
        font_weight,
        height: m.option_height,
        padding_block: m.option_padding_block,
        padding_inline: m.option_padding_inline,
        radius: tokens.border_radius_sm,
        font_size: tokens.font_size,
        line_height: tokens.font_height,
    }
}

/// 菜单容器：`colorBgElevated` + `borderRadiusLG` + `paddingXXS`。
pub fn menu_state(tokens: &Tokens) -> SelectMenuStyle {
    let m = metric::SelectMetric::resolve(tokens);
    SelectMenuStyle {
        bg: tokens.color_bg_elevated,
        radius: m.menu_radius,
        padding: m.menu_padding,
    }
}

/// 多选触发框内的已选芯片（antd `selection-item`，浏览器实测）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MultiChipStyle {
    pub bg: Hsla,
    pub fg: Hsla,
    pub height: f32,
    pub line_height: f32,
    pub radius: f32,
    pub padding_start: f32,
    pub padding_end: f32,
    pub margin_block: f32,
    pub margin_end: f32,
    pub font_size: f32,
    pub remove_color: Hsla,
    pub remove_size: f32,
}

/// 芯片底色：filled 变体用 `colorBgContainer`（避免与浅色底混在一起），其余用 `colorFillSecondary`；
/// 正文色跟随触发器状态色，移除图标取 `colorTextTertiary`。
pub fn chip_state(
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    tokens: &Tokens,
    disabled: bool,
) -> MultiChipStyle {
    let m = metric::SelectMetric::resolve(tokens);
    let index = size.index();
    let trigger = state(variant, status, size, tokens, false, false, disabled);
    MultiChipStyle {
        bg: match variant {
            InputVariant::Filled => tokens.color_bg_container,
            _ => tokens.color_fill_secondary,
        },
        fg: trigger.fg,
        height: m.chip_height[index],
        line_height: m.chip_line_height[index],
        radius: m.chip_radius[index],
        padding_start: m.chip_padding_start,
        padding_end: m.chip_padding_end,
        margin_block: m.chip_margin_block[index],
        margin_end: m.chip_margin_end,
        font_size: m.font_size[index],
        remove_color: tokens.color_text_tertiary,
        remove_size: m.chip_remove_size,
    }
}

/// 通用下拉选择器。
///
/// ```ignore
/// Select::new("city", ["北京", "上海", "广州"], self.city)
///     .open(self.city_open)
///     .on_open_change(cx.listener(|this, open, _, cx| {
///         this.city_open = *open;
///         cx.notify();
///     }))
///     .on_select(cx.listener(|this, ix, _, cx| {
///         this.city = *ix;
///         cx.notify();
///     }))
/// ```
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    items: Vec<SharedString>,
    value: Option<usize>,
    open: bool,
    disabled: bool,
    allow_clear: bool,
    searchable: bool,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    placeholder: SharedString,
    menu_width: Option<Pixels>,
    on_select: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_clear: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
}

impl Select {
    pub fn new(
        id: impl Into<ElementId>,
        items: impl IntoIterator<Item = impl Into<SharedString>>,
        selected: usize,
    ) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().map(Into::into).collect(),
            value: Some(selected),
            open: false,
            disabled: false,
            allow_clear: false,
            searchable: false,
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            placeholder: "请选择…".into(),
            menu_width: None,
            on_select: None,
            on_clear: None,
            on_open_change: None,
        }
    }

    /// 外观变体（默认 outlined）
    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 校验状态（默认 / 错误 / 警告）
    pub fn status(mut self, status: InputStatus) -> Self {
        self.status = status;
        self
    }

    /// 尺寸档（small / middle / large）
    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    /// 当前是否展开（受控）
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// 受控值：None 表示未选（显示 placeholder）
    pub fn value(mut self, value: Option<usize>) -> Self {
        self.value = value;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 有值时右侧显示 ✕，点击触发 on_clear
    pub fn allow_clear(mut self, allow_clear: bool) -> Self {
        self.allow_clear = allow_clear;
        self
    }

    /// 展开时菜单顶部显示过滤输入框
    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 菜单宽度，默认与触发框同宽
    pub fn menu_width(mut self, width: Pixels) -> Self {
        self.menu_width = Some(width);
        self
    }

    pub fn on_select(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }

    /// 点击 ✕ 清空（无事件参，用 cx.entity() 句柄回写 value=None）
    pub fn on_clear(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear = Some(Rc::new(listener));
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

/// searchable Select 的过滤输入框实体（跨渲染复用，按 id 索引）
#[derive(Default)]
struct SelectSearchState {
    inputs: HashMap<ElementId, Entity<TextInput>>,
}
impl gpui::Global for SelectSearchState {}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let open = self.open && !self.disabled;
        let has_value = self.value.is_some();
        let label = self
            .value
            .and_then(|ix| self.items.get(ix).cloned())
            .unwrap_or(self.placeholder.clone());

        // 收起时重置过滤词，下次展开从全量开始
        if !open && self.searchable {
            if let Some(search) = cx
                .default_global::<SelectSearchState>()
                .inputs
                .get(&self.id)
                .cloned()
            {
                search.update(cx, |input, cx| input.set_text_silent("", cx));
            }
        }

        // 同一帧内共享：菜单的“外部点击”先于触发按钮的按下事件，
        // 用该标记避免“收起后又被触发按钮重新展开”
        let suppress_toggle = Rc::new(Cell::new(false));

        // antd 在展开（focused）时换 primary 描边并加聚焦环
        let style = theme::with(|t| {
            state(
                self.variant,
                self.status,
                self.size,
                t,
                false,
                open,
                self.disabled,
            )
        });
        let hover_style = theme::with(|t| {
            state(
                self.variant,
                self.status,
                self.size,
                t,
                true,
                false,
                self.disabled,
            )
        });
        let (gap, icon_color, icon_hover) =
            theme::with(|t| (t.padding_xxs, t.color_text_quaternary, t.color_icon));
        let icon_size = theme::with(|t| t.font_size_icon);

        let trigger_focus = window
            .use_keyed_state(
                ElementId::Name(format!("select-{:?}-trigger-focus", self.id).into()),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone();
        // 每个选项一个持久焦点句柄，供菜单内方向键 roving focus 使用
        let option_focus: Vec<_> = (0..self.items.len())
            .map(|ix| {
                window
                    .use_keyed_state(
                        ElementId::Name(format!("select-{:?}-option-focus-{ix}", self.id).into()),
                        cx,
                        |_, cx| cx.focus_handle(),
                    )
                    .read(cx)
                    .clone()
            })
            .collect();

        let mut trigger = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap(px(gap))
            .w_full()
            .line_height(px(style.line_height))
            .text_size(px(style.font_size))
            .text_color(style.fg)
            .px(px(style.padding_inline))
            .py(px(style.padding_block))
            .child(
                div()
                    .overflow_hidden()
                    .text_color(if has_value {
                        style.fg
                    } else {
                        style.placeholder
                    })
                    .child(label.clone()),
            );
        trigger = text_input::apply_frame(trigger, &style);
        trigger = text_input::hover_frame(trigger, &hover_style, open);

        if self.allow_clear && has_value && !self.disabled {
            let on_clear = self.on_clear.clone();
            let mut clear = div()
                .id(ElementId::Name(format!("{:?}-clear", self.id).into()))
                .cursor(CursorStyle::PointingHand)
                .text_size(px(icon_size))
                .text_color(icon_color)
                .hover(move |style| style.text_color(icon_hover))
                .child("✕");
            clear = a11y::Semantics::new(Role::Button, format!("清除 {}", label))
                .id(format!("{:?}-clear", self.id))
                .focusable(true)
                .apply(clear);
            if let Some(on_clear) = on_clear {
                clear = a11y::click(clear, move |_, window, cx| {
                    cx.stop_propagation();
                    on_clear(window, cx);
                });
            }
            trigger = trigger.child(clear);
        } else {
            trigger = trigger.child(
                div()
                    .text_size(px(icon_size))
                    .text_color(icon_color)
                    .child("▾"),
            );
        }

        // frame helper 返回 Div，元素 id 必须在这之后再挂（会变成 Stateful<Div>）
        let trigger = trigger.id(self.id.clone());
        let trigger = trigger.track_focus(&trigger_focus);
        let mut semantics = a11y::Semantics::new(Role::ComboBox, label.clone())
            .id(format!("{:?}-trigger", self.id))
            .expanded(open)
            .disabled(self.disabled)
            .focusable(!self.disabled);
        if has_value {
            semantics = semantics.value(label);
        }
        let mut trigger = semantics.apply(trigger);

        if !self.disabled {
            let on_open_change = self.on_open_change.clone();
            let suppress_toggle = suppress_toggle.clone();
            trigger = trigger.cursor(CursorStyle::PointingHand);
            trigger = a11y::click_void(trigger, move |window, cx| {
                if suppress_toggle.take() {
                    return;
                }
                if let Some(on_open_change) = &on_open_change {
                    on_open_change(&!open, window, cx);
                }
            });
            if let Some(on_open_change) = self.on_open_change.clone() {
                let keyboard_change = on_open_change.clone();
                let escape_focus = trigger_focus.clone();
                let focus_targets: Vec<_> = option_focus.clone();
                let item_count = self.items.len();
                let selected_ix = self.value.filter(|ix| *ix < item_count);
                // 可搜索 Select 展开后由搜索框持有焦点，方向键再从搜索框进入选项
                let searchable = self.searchable;
                trigger = trigger.on_key_down(move |event, window, cx| {
                    let key = event
                        .keystroke
                        .key
                        .strip_prefix("arrow")
                        .unwrap_or(event.keystroke.key.as_str());
                    match key {
                        "down" | "up" if !open => {
                            cx.stop_propagation();
                            keyboard_change(&true, window, cx);
                            if searchable {
                                return;
                            }
                            let target = if key == "down" {
                                selected_ix.unwrap_or(0)
                            } else {
                                item_count.saturating_sub(1)
                            };
                            if let Some(handle) = focus_targets.get(target) {
                                handle.focus(window, cx);
                            }
                        }
                        "down" | "up" if open => {
                            cx.stop_propagation();
                            if searchable {
                                return;
                            }
                            let target = if key == "down" {
                                selected_ix.unwrap_or(0)
                            } else {
                                item_count.saturating_sub(1)
                            };
                            if let Some(handle) = focus_targets.get(target) {
                                handle.focus(window, cx);
                            }
                        }
                        "escape" if open => {
                            cx.stop_propagation();
                            keyboard_change(&false, window, cx);
                            escape_focus.focus(window, cx);
                        }
                        _ => {}
                    }
                });
                if open {
                    trigger = trigger
                        .on_a11y_action(AccessibleAction::Collapse, move |_, window, cx| {
                            on_open_change(&false, window, cx)
                        });
                } else {
                    trigger = trigger
                        .on_a11y_action(AccessibleAction::Expand, move |_, window, cx| {
                            on_open_change(&true, window, cx)
                        });
                }
            }
        } else {
            trigger = trigger.cursor(CursorStyle::Arrow);
        }

        let mut popup = None;
        if open {
            let mut menu = menu_shell(&self.id, self.menu_width).on_mouse_down_out({
                let on_open_change = self.on_open_change.clone();
                let suppress_toggle = suppress_toggle.clone();
                move |_, window, cx| {
                    suppress_toggle.set(true);
                    if let Some(on_open_change) = &on_open_change {
                        on_open_change(&false, window, cx);
                    }
                }
            });

            let mut visible: Vec<usize> = (0..self.items.len()).collect();
            let mut search_focus = None;
            if self.searchable {
                if !cx
                    .default_global::<SelectSearchState>()
                    .inputs
                    .contains_key(&self.id)
                {
                    let entity = cx.new(|cx| {
                        TextInput::new(cx)
                            .bare(true)
                            .placeholder("输入过滤…")
                            .on_change(|_, _, cx| cx.refresh_windows())
                    });
                    cx.default_global::<SelectSearchState>()
                        .inputs
                        .insert(self.id.clone(), entity);
                }
                let search = cx
                    .default_global::<SelectSearchState>()
                    .inputs
                    .get(&self.id)
                    .unwrap()
                    .clone();
                let query = search.read(cx).text().to_lowercase();
                if !query.is_empty() {
                    visible.retain(|ix| self.items[*ix].to_lowercase().contains(&query));
                }
                // 菜单内 roving focus 生效后不再每帧把焦点抢回搜索框
                let options_focused = visible
                    .iter()
                    .any(|&ix| option_focus[ix].is_focused(window));
                let handle = search.focus_handle(cx);
                if !options_focused {
                    handle.focus(window, cx);
                }
                search_focus = Some(handle);
                let (px_search, py_search, border) =
                    theme::with(|t| (t.padding_xs, t.padding_xxs, t.color_border));
                let close = self.on_open_change.clone();
                let escape_focus = trigger_focus.clone();
                let first_focus = visible.first().map(|&ix| option_focus[ix].clone());
                menu = menu.child(
                    div()
                        .px(px(px_search))
                        .py(px(py_search))
                        .border_b(px(theme::with(|t| t.line_width)))
                        .border_color(border)
                        .on_key_down(move |event, window, cx| {
                            let key = event
                                .keystroke
                                .key
                                .strip_prefix("arrow")
                                .unwrap_or(event.keystroke.key.as_str());
                            match key {
                                "down" => {
                                    if let Some(first) = &first_focus {
                                        first.focus(window, cx);
                                        cx.stop_propagation();
                                    }
                                }
                                "escape" => {
                                    if let Some(close) = &close {
                                        close(&false, window, cx);
                                    }
                                    escape_focus.focus(window, cx);
                                    cx.stop_propagation();
                                }
                                _ => {}
                            }
                        })
                        .child(search),
                );
            }

            let menu_max_height =
                theme::with(|t| metric::command_palette_metric(t).list_max_height);
            let scroll_id = ElementId::Name(format!("select-{:?}-scroll", self.id).into());
            let handle_id = ElementId::Name(format!("select-{:?}-scroll-handle", self.id).into());
            let scroll_handle = window
                .use_keyed_state(handle_id, cx, |_, _| ScrollHandle::new())
                .read(cx)
                .clone();
            let mut options = a11y::Semantics::new(Role::ListBox, "选项")
                .id(format!("{:?}-options", self.id))
                .apply(
                    div()
                        .id(scroll_id)
                        .flex_1()
                        .min_w(px(0.))
                        .overflow_y_scroll()
                        .track_scroll(&scroll_handle),
                );

            let option_count = visible.len();
            let visible_focus: Vec<_> =
                visible.iter().map(|&ix| option_focus[ix].clone()).collect();
            let active_pos = visible
                .iter()
                .position(|&ix| option_focus[ix].is_focused(window))
                .or_else(|| {
                    self.value
                        .and_then(|v| visible.iter().position(|&ix| ix == v))
                })
                .unwrap_or(0);
            for (pos, &ix) in visible.iter().enumerate() {
                let item = self.items[ix].clone();
                let is_selected = self.value == Some(ix);
                let on_select = self.on_select.clone();
                let on_open_change = self.on_open_change.clone();

                let (opt, opt_active) = theme::with(|t| {
                    (
                        option_state(t, is_selected, false, false),
                        option_state(t, is_selected, true, false),
                    )
                });
                let mut row = div()
                    .id(ElementId::Name(format!("{:?}-opt-{ix}", self.id).into()))
                    .flex()
                    .flex_row()
                    .items_center()
                    .min_h(px(opt.height))
                    .px(px(opt.padding_inline))
                    .py(px(opt.padding_block))
                    .rounded(px(opt.radius))
                    .text_size(px(opt.font_size))
                    .line_height(px(opt.line_height))
                    .text_color(opt.fg)
                    .font_weight(opt.font_weight)
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |style| style.bg(opt_active.bg).text_color(opt_active.fg))
                    .child(item.clone());

                if is_selected {
                    row = row.bg(opt.bg);
                }

                row = a11y::Semantics::new(Role::ListBoxOption, item)
                    .id(format!("{:?}-opt-{ix}", self.id))
                    .selected(is_selected)
                    .focusable(true)
                    .position(pos + 1, option_count)
                    .apply(row);
                let focus_handle = option_focus[ix].clone();
                let nav_focus = visible_focus.clone();
                let nav_scroll = scroll_handle.clone();
                let escape_focus = trigger_focus.clone();
                let escape_close = self.on_open_change.clone();
                let back_search = search_focus.clone();
                row = row
                    .track_focus(&focus_handle)
                    .tab_stop(pos == active_pos)
                    .on_key_down(move |event, window, cx| {
                        let key = event
                            .keystroke
                            .key
                            .strip_prefix("arrow")
                            .unwrap_or(event.keystroke.key.as_str());
                        let target = match key {
                            "down" => (pos + 1).min(option_count.saturating_sub(1)),
                            "up" if pos == 0 => {
                                if let Some(search) = &back_search {
                                    search.focus(window, cx);
                                    cx.stop_propagation();
                                }
                                return;
                            }
                            "up" => pos - 1,
                            "home" => 0,
                            "end" => option_count.saturating_sub(1),
                            "escape" => {
                                cx.stop_propagation();
                                if let Some(close) = &escape_close {
                                    close(&false, window, cx);
                                }
                                escape_focus.focus(window, cx);
                                return;
                            }
                            "tab" => {
                                if let Some(close) = &escape_close {
                                    close(&false, window, cx);
                                }
                                return;
                            }
                            _ => return,
                        };
                        if target != pos {
                            if let Some(handle) = nav_focus.get(target) {
                                handle.focus(window, cx);
                            }
                            nav_scroll.scroll_to_item(target);
                            cx.stop_propagation();
                        }
                    });
                let click_focus = trigger_focus.clone();
                row = a11y::click_void(row, move |window, cx| {
                    if let Some(on_select) = &on_select {
                        on_select(&ix, window, cx);
                    }
                    if let Some(on_open_change) = &on_open_change {
                        on_open_change(&false, window, cx);
                    }
                    click_focus.focus(window, cx);
                });

                options = options.child(row);
            }
            if self.searchable && visible.is_empty() {
                let empty = theme::with(|t| option_state(t, false, false, true));
                let empty_row = div()
                    .id(ElementId::Name(format!("{:?}-empty", self.id).into()))
                    .min_h(px(empty.height))
                    .px(px(empty.padding_inline))
                    .py(px(empty.padding_block))
                    .text_size(px(empty.font_size))
                    .line_height(px(empty.line_height))
                    .text_color(empty.fg)
                    .child("无匹配项");
                options = options.child(
                    a11y::Semantics::new(Role::ListItem, "无匹配项")
                        .id(format!("{:?}-empty", self.id))
                        .disabled(true)
                        .apply(empty_row),
                );
            }

            menu = menu.child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .max_h(px(menu_max_height))
                    .child(options)
                    .child(Scrollbar::vertical(
                        ElementId::Name(format!("select-{:?}-scrollbar", self.id).into()),
                        scroll_handle,
                    )),
            );

            popup = Some(div().child(menu));
        }

        anchored(trigger.into_any_element(), popup)
    }
}

/// 多选下拉选择器：触发框内以芯片展示已选项（✕ 可移除），
/// 菜单行带勾选框，点击选项切换选中且不收起菜单。
///
/// ```ignore
/// MultiSelect::new("skills", ["Rust", "GPUI", "桌面端"], self.skills.clone())
///     .open(self.skills_open)
///     .on_open_change(cx.listener(|this, open, _, cx| {
///         this.skills_open = *open;
///         cx.notify();
///     }))
///     .on_change(cx.listener(|this, sel, _, cx| {
///         this.skills = sel.clone();
///         cx.notify();
///     }))
/// ```
pub struct MultiSelect {
    id: ElementId,
    items: Vec<SharedString>,
    selected: Vec<usize>,
    open: bool,
    disabled: bool,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    placeholder: SharedString,
    menu_width: Option<Pixels>,
    on_change: Option<Rc<dyn Fn(&Vec<usize>, &mut Window, &mut App)>>,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
}

impl MultiSelect {
    pub fn new(
        id: impl Into<ElementId>,
        items: impl IntoIterator<Item = impl Into<SharedString>>,
        selected: Vec<usize>,
    ) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().map(Into::into).collect(),
            selected,
            open: false,
            disabled: false,
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            placeholder: "请选择…".into(),
            menu_width: None,
            on_change: None,
            on_open_change: None,
        }
    }

    /// 外观变体（默认 outlined，与 antd ConfigProvider 缺省一致）
    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 校验状态（默认 / error / warning）
    pub fn status(mut self, status: InputStatus) -> Self {
        self.status = status;
        self
    }

    /// 尺寸档（small / middle / large）
    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    /// 当前是否展开（受控）
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 菜单宽度，默认与触发框同宽
    pub fn menu_width(mut self, width: Pixels) -> Self {
        self.menu_width = Some(width);
        self
    }

    /// 选中集合变化（升序去重后的下标集合）
    pub fn on_change(
        mut self,
        listener: impl Fn(&Vec<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(listener));
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

impl IntoElement for MultiSelect {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let open = self.open && !self.disabled;
        let mut selected = self.selected.clone();
        selected.sort_unstable();
        selected.dedup();
        let selected_labels: Vec<String> = selected
            .iter()
            .filter_map(|ix| self.items.get(*ix).map(|item| item.to_string()))
            .collect();
        let value = selected_labels.join("、");
        let label = if selected_labels.is_empty() {
            self.placeholder.clone()
        } else {
            SharedString::from(value.clone())
        };

        let suppress_toggle = Rc::new(Cell::new(false));

        let index = self.size.index();
        let metrics = theme::with(|t| metric::SelectMetric::resolve(t));
        let style = theme::with(|t| {
            state(
                self.variant,
                self.status,
                self.size,
                t,
                false,
                open,
                self.disabled,
            )
        });
        let hover_style = theme::with(|t| {
            state(
                self.variant,
                self.status,
                self.size,
                t,
                true,
                false,
                self.disabled,
            )
        });
        let chip =
            theme::with(|t| chip_state(self.variant, self.status, self.size, t, self.disabled));
        let (icon_size, icon_color, icon_hover) =
            theme::with(|t| (t.font_size_icon, t.color_text_quaternary, t.color_icon));
        let remove_hover = theme::with(|t| t.color_text);

        let mut chips = div().flex_1().flex().flex_row().flex_wrap().items_center();
        if selected.is_empty() {
            chips = chips.child(
                div()
                    .text_color(style.placeholder)
                    .child(self.placeholder.clone()),
            );
        } else {
            for ix in &selected {
                let Some(label) = self.items.get(*ix) else {
                    continue;
                };
                let on_change = self.on_change.clone();
                let next: Vec<usize> = selected.iter().copied().filter(|i| i != ix).collect();
                let mut chip_view = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(chip.height))
                    .line_height(px(chip.line_height))
                    .pl(px(chip.padding_start))
                    .pr(px(chip.padding_end))
                    .my(px(chip.margin_block))
                    .mr(px(chip.margin_end))
                    .rounded(px(chip.radius))
                    .bg(chip.bg)
                    .text_color(chip.fg)
                    .text_size(px(chip.font_size))
                    .child(label.clone());
                if !self.disabled {
                    let mut remove = div()
                        .id(ElementId::Name(format!("{:?}-chip-{ix}", self.id).into()))
                        .ml(px(chip.padding_end))
                        .text_size(px(chip.remove_size))
                        .text_color(chip.remove_color)
                        .cursor(CursorStyle::PointingHand)
                        .hover(move |style| style.text_color(remove_hover))
                        .child("✕");
                    remove =
                        a11y::Semantics::new(Role::Button, format!("移除 {}", self.items[*ix]))
                            .id(format!("{:?}-chip-{ix}", self.id))
                            .focusable(true)
                            .apply(remove);
                    if let Some(on_change) = on_change {
                        remove = a11y::click(remove, move |_, window, cx| {
                            cx.stop_propagation();
                            on_change(&next, window, cx);
                        });
                    }
                    chip_view = chip_view.child(remove);
                }
                chips = chips.child(chip_view);
            }
        }

        let mut trigger = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .pl(px(metrics.multi_padding_start))
            .pr(px(metrics.multi_padding_inline[index]))
            .py(px(metrics.multi_padding_block))
            .text_size(px(style.font_size))
            .line_height(px(style.line_height))
            .text_color(style.fg)
            .child(chips)
            .child(
                div()
                    .ml(px(metrics.chip_margin_end))
                    .text_size(px(icon_size))
                    .text_color(icon_color)
                    .hover(move |style| style.text_color(icon_hover))
                    .child("▾"),
            );
        trigger = text_input::apply_frame(trigger, &style);
        trigger = text_input::hover_frame(trigger, &hover_style, open);

        // frame helper 返回 Div，元素 id 必须在这之后再挂（会变成 Stateful<Div>）
        let trigger = trigger.id(self.id.clone());
        let mut semantics = a11y::Semantics::new(Role::ComboBox, label)
            .id(format!("{:?}-trigger", self.id))
            .expanded(open)
            .disabled(self.disabled)
            .focusable(!self.disabled);
        if !selected_labels.is_empty() {
            semantics = semantics.value(value);
        }
        let mut trigger = semantics.apply(trigger);

        if self.disabled {
            trigger = trigger.cursor(CursorStyle::Arrow);
        } else {
            let on_open_change = self.on_open_change.clone();
            let suppress_toggle = suppress_toggle.clone();
            trigger = trigger.cursor(CursorStyle::PointingHand);
            trigger = a11y::click_void(trigger, move |window, cx| {
                if suppress_toggle.take() {
                    return;
                }
                if let Some(on_open_change) = &on_open_change {
                    on_open_change(&!open, window, cx);
                }
            });
        }

        let mut popup = None;
        if open {
            let mut menu = menu_shell(&self.id, self.menu_width).on_mouse_down_out({
                let on_open_change = self.on_open_change.clone();
                let suppress_toggle = suppress_toggle.clone();
                move |_, window, cx| {
                    suppress_toggle.set(true);
                    if let Some(on_open_change) = &on_open_change {
                        on_open_change(&false, window, cx);
                    }
                }
            });

            let menu_max_height =
                theme::with(|t| metric::command_palette_metric(t).list_max_height);
            let mut options = div().flex_1().min_w(px(0.));

            for (ix, item) in self.items.iter().enumerate() {
                let is_selected = selected.contains(&ix);
                let on_change = self.on_change.clone();
                let mut next = selected.clone();
                if is_selected {
                    next.retain(|i| *i != ix);
                } else {
                    next.push(ix);
                    next.sort_unstable();
                }

                let base = theme::with(|t| option_state(t, is_selected, false, false));
                let active = theme::with(|t| option_state(t, is_selected, true, false));
                let check_color = theme::with(|t| t.color_primary);

                let mut row = div()
                    .id(ElementId::Name(format!("{:?}-opt-{ix}", self.id).into()))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .min_h(px(base.height))
                    .px(px(base.padding_inline))
                    .py(px(base.padding_block))
                    .rounded(px(base.radius))
                    .text_size(px(base.font_size))
                    .line_height(px(base.line_height))
                    .text_color(base.fg)
                    .font_weight(base.font_weight)
                    .bg(base.bg)
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |style| style.bg(active.bg).text_color(active.fg))
                    .child(div().flex_1().child(item.clone()));
                if is_selected {
                    row = row.child(
                        div()
                            .ml(px(base.padding_inline))
                            .text_color(check_color)
                            .child("✓"),
                    );
                }
                row = a11y::Semantics::new(Role::ListBoxOption, item.clone())
                    .id(format!("{:?}-opt-{ix}", self.id))
                    .selected(is_selected)
                    .toggled(if is_selected {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .focusable(true)
                    .position(ix + 1, self.items.len())
                    .apply(row);
                row = a11y::click_void(row, move |window, cx| {
                    if let Some(on_change) = &on_change {
                        on_change(&next, window, cx);
                    }
                });

                options = options.child(row);
            }

            menu = menu.child(
                VerticalScroll::new(
                    ElementId::Name(format!("multi-select-{:?}-scroll", self.id).into()),
                    options.into_any_element(),
                )
                .max_h(px(menu_max_height)),
            );

            popup = Some(div().child(menu));
        }

        anchored(trigger.into_any_element(), popup)
    }
}

/// 菜单容器：antd 下拉无边框，底色 `colorBgElevated` + `borderRadiusLG` + `paddingXXS`；
/// 宽度默认撑满定位层（即触发框宽）。阴影暂用 gpui 的 `shadow_lg` 近似 `boxShadowSecondary`。
fn menu_shell(id: &ElementId, width: Option<Pixels>) -> Stateful<Div> {
    let style = theme::with(|t| menu_state(t));
    let menu = div()
        .id(ElementId::Name(format!("{id:?}-menu").into()))
        .occlude()
        .flex()
        .flex_col()
        .p(px(style.padding))
        .rounded(px(style.radius))
        .bg(style.bg)
        .shadow_lg();
    let menu = a11y::Semantics::new(Role::ListBox, "选择选项")
        .id(format!("{id:?}-menu"))
        .apply(menu);
    match width {
        Some(w) => menu.w(w),
        None => menu.w_full(),
    }
}
