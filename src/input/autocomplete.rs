//! 自动完成输入框：可输入过滤的下拉建议，复用 popup 浮层基建。

use std::{cell::Cell, rc::Rc};

use gpui::{
    AccessibleAction, App, Context, CursorStyle, ElementId, Entity, FocusHandle, Focusable,
    IntoElement, KeyDownEvent, ParentElement, Render, Role, ScrollHandle, SharedString, Styled,
    Window, div, prelude::*, px,
};

use crate::a11y::{self, Semantics};

use crate::text_input::{
    self, InputSize, InputStatus, InputStyle, InputVariant, apply_frame, hover_frame,
};
use crate::{Scrollbar, TextInput, popup::anchored, theme};

/// Autocomplete 外框状态矩阵，语义与 [`text_input::state`] 一致
///（antdv-next 的 Autocomplete 就是 Input + 下拉，没有独立外框）。
pub fn state(
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    tokens: &crate::theme::Tokens,
    hover: bool,
    focused: bool,
    disabled: bool,
) -> InputStyle {
    text_input::state(variant, status, size, tokens, hover, focused, disabled)
}

/// 自动完成输入框（Entity）。
///
/// ```ignore
/// let fruit = cx.new(|cx| Autocomplete::new(cx, vec!["苹果".into(), "香蕉".into()])
///     .on_select(|item, _, cx| { /* 选中回调 */ }));
/// ```
pub struct Autocomplete {
    input: Entity<TextInput>,
    items: Vec<SharedString>,
    placeholder: SharedString,
    open: bool,
    highlight: usize,
    on_select: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    disabled: bool,
}

impl Autocomplete {
    pub fn new(cx: &mut Context<Self>, items: Vec<SharedString>) -> Self {
        let entity = cx.entity();
        let input = cx.new(|cx| {
            TextInput::new(cx).bare(true).on_change(move |_, _, cx| {
                let _ = entity.update(cx, |this, cx| {
                    this.open = true;
                    this.highlight = 0;
                    cx.notify();
                });
            })
        });
        Self {
            input,
            items,
            placeholder: "输入以筛选…".into(),
            open: false,
            highlight: 0,
            on_select: None,
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            disabled: false,
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

    /// 尺寸（small / middle / large）
    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    /// 禁用态：灰底、不可输入、不展开建议
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 选中建议项回调
    pub fn on_select(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }

    /// 当前文本
    pub fn text(&self, cx: &App) -> String {
        self.input.read(cx).text().to_string()
    }

    fn filtered(&self, cx: &App) -> Vec<SharedString> {
        let query = self.input.read(cx).text().to_lowercase();
        self.items
            .iter()
            .filter(|item| query.is_empty() || item.to_lowercase().contains(&query))
            .cloned()
            .collect()
    }

    fn select(&mut self, item: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        // 静默写入：避免内部 on_change 反向 update 已租约的自身
        let input = self.input.clone();
        input.update(cx, |input, cx| input.set_text_silent(item.clone(), cx));
        self.open = false;
        if let Some(on_select) = self.on_select.clone() {
            on_select(&item, window, cx);
        }
        cx.notify();
    }

    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.open == open {
            return;
        }
        self.open = open;
        self.highlight = 0;
        self.input.focus_handle(cx).focus(window, cx);
        cx.notify();
    }

    fn toggle_open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_open(!self.open, window, cx);
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        let filtered = self.filtered(cx);
        if filtered.is_empty() {
            return;
        }
        match event.keystroke.key.as_str() {
            "down" => {
                self.highlight = (self.highlight + 1).min(filtered.len() - 1);
                cx.notify();
                cx.stop_propagation();
            }
            "up" => {
                self.highlight = self.highlight.saturating_sub(1);
                cx.notify();
                cx.stop_propagation();
            }
            "enter" => {
                let item = filtered[self.highlight.min(filtered.len() - 1)].clone();
                self.select(item, window, cx);
                cx.stop_propagation();
            }
            "escape" => {
                self.open = false;
                cx.notify();
                cx.stop_propagation();
            }
            _ => {}
        }
    }
}

impl Render for Autocomplete {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.input.focus_handle(cx).is_focused(window);
        let placeholder = self.placeholder.clone();
        let (variant, status, size, disabled) =
            (self.variant, self.status, self.size, self.disabled);
        self.input.update(cx, |input, cx| {
            input.set_placeholder(placeholder.clone());
            input.set_form_state(variant, status, size, disabled, cx);
        });

        let open = self.open;
        let filtered = self.filtered(cx);
        let show_menu = !disabled && open && !filtered.is_empty();
        let highlight = self.highlight.min(filtered.len().saturating_sub(1));

        let style = theme::with(|t| state(variant, status, size, t, false, focused, disabled));
        let hover_style = theme::with(|t| state(variant, status, size, t, true, false, disabled));
        let gap = theme::with(|t| t.padding_xxs);

        // 触发框：与 Input 同源的外框，内部 TextInput 为 bare
        let suppress_toggle: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        let mut trigger = div()
            .flex()
            .items_center()
            .gap(px(gap))
            .w_full()
            .overflow_hidden()
            .line_height(px(style.line_height))
            .text_size(px(style.font_size))
            .text_color(style.fg)
            .px(px(style.padding_inline))
            .py(px(style.padding_block))
            .child(div().flex_1().min_w(px(0.)).child(self.input.clone()));
        trigger = apply_frame(trigger, &style);
        trigger = hover_frame(trigger, &hover_style, focused);
        let mut trigger = trigger.id(ElementId::Name(
            format!("autocomplete-trigger-{:?}", cx.entity_id()).into(),
        ));
        if disabled {
            trigger = trigger.cursor(CursorStyle::Arrow);
        } else {
            let entity = cx.entity();
            let suppress = suppress_toggle.clone();
            trigger = trigger.cursor(CursorStyle::IBeam);
            trigger = a11y::click_void(trigger, move |window, cx| {
                if suppress.take() {
                    return;
                }
                entity.update(cx, |this, cx| this.toggle_open(window, cx));
            });
        }
        let semantics = Semantics::new(Role::ComboBox, "自动完成")
            .id(format!("autocomplete-trigger-{:?}", cx.entity_id()))
            .value(self.input.read(cx).a11y_value())
            .placeholder(placeholder)
            .expanded(show_menu)
            .disabled(disabled)
            .invalid(status == InputStatus::Error)
            .focusable(!disabled);
        if !disabled {
            trigger = trigger.track_focus(&self.input.focus_handle(cx));
        }
        let mut trigger = self.input.update(cx, |input, cx| {
            input.with_a11y_actions(semantics.apply(trigger), cx)
        });
        if !disabled {
            let entity = cx.entity();
            trigger = trigger
                .on_a11y_action(AccessibleAction::Expand, {
                    let entity = entity.clone();
                    move |_, window, cx| {
                        let _ = entity.update(cx, |this, cx| this.set_open(true, window, cx));
                    }
                })
                .on_a11y_action(AccessibleAction::Collapse, {
                    move |_, window, cx| {
                        let _ = entity.update(cx, |this, cx| this.set_open(false, window, cx));
                    }
                });
        }

        // 建议菜单
        let mut popup = None;
        if show_menu {
            let entity_id = format!("autocomplete-{:?}", cx.entity_id());
            let scroll_handle = window
                .use_keyed_state(
                    ElementId::Name(format!("{entity_id}-scroll-handle").into()),
                    cx,
                    |_, _| ScrollHandle::new(),
                )
                .read(cx)
                .clone();
            let menu = div()
                .w_full()
                .max_h(px(240.))
                .overflow_hidden()
                .occlude()
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(theme::border())
                .bg(theme::surface())
                .shadow_lg()
                .flex()
                .flex_row()
                .on_mouse_down_out({
                    let entity = cx.entity();
                    let suppress = suppress_toggle.clone();
                    move |_, _, cx| {
                        suppress.set(true);
                        entity.update(cx, |this, cx| {
                            this.open = false;
                            cx.notify();
                        });
                    }
                });
            let mut menu = menu.child(
                Semantics::new(Role::ListBox, "自动完成建议")
                    .id(format!("{entity_id}-list"))
                    .disabled(disabled)
                    .apply(
                        div()
                            .id(ElementId::Name(format!("{entity_id}-list").into()))
                            .flex_1()
                            .min_w(px(0.))
                            .min_h(px(0.))
                            .overflow_y_scroll()
                            .track_scroll(&scroll_handle),
                    ),
            );

            for (ix, item) in filtered.iter().enumerate() {
                let entity = cx.entity();
                let item = item.clone();
                let mut row = div()
                    .id(ElementId::Name(format!("{entity_id}-item-{ix}").into()))
                    .px_3()
                    .py(px(6.))
                    .text_sm()
                    .cursor(CursorStyle::PointingHand)
                    .child(item.clone());
                if ix == highlight {
                    row = row.bg(theme::accent_soft()).text_color(theme::accent());
                    row = row.aria_active_descendant();
                } else {
                    row = row.hover(|style| style.bg(theme::hover_bg()));
                }
                let semantics = Semantics::new(Role::ListBoxOption, item.clone())
                    .id(format!("{entity_id}-item-{ix}"))
                    .selected(ix == highlight)
                    .position(ix + 1, filtered.len())
                    .disabled(disabled);
                row = semantics.apply(row);
                if !disabled {
                    menu = menu.child(a11y::click_void(row, move |window, cx| {
                        entity.update(cx, |this, cx| this.select(item.clone(), window, cx));
                    }));
                } else {
                    menu = menu.child(row);
                }
            }
            popup = Some(menu.child(Scrollbar::vertical(
                ElementId::Name(format!("{entity_id}-scrollbar").into()),
                scroll_handle,
            )));
        }

        let mut root = div()
            .id(ElementId::Name(
                format!("autocomplete-{:?}", cx.entity_id()).into(),
            ))
            .w_full();
        if !disabled {
            root = root.on_key_down(cx.listener(Self::on_key_down));
        }
        root.child(anchored(trigger.into_any_element(), popup))
    }
}

impl Focusable for Autocomplete {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}
