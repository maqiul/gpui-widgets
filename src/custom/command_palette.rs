//! 命令面板：Ctrl+K 唤起的搜索式命令选择器（遮罩 + 居中面板）。

use std::rc::Rc;

use gpui::{
    App, Context, CursorStyle, ElementId, Entity, FocusHandle, Focusable, IntoElement, KeyBinding,
    KeyDownEvent, MouseButton, Orientation, ParentElement, Render, Role, ScrollHandle,
    SharedString, StatefulInteractiveElement, Styled, Window, deferred, div, prelude::*, px,
};

use crate::{Scrollbar, TextInput, a11y, theme};

gpui::actions!(widgets_command_palette, [Toggle]);

/// 注册 Ctrl+K 快捷键（应用启动时调用一次）。
///
/// 触发 [`Toggle`] 动作后，由宿主视图调用 [`CommandPalette::open`] / `close`。
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("ctrl-k", Toggle, None)]);
}

/// 命令项
pub struct PaletteItem {
    label: SharedString,
    hint: Option<SharedString>,
}

impl PaletteItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            hint: None,
        }
    }

    /// 右侧提示（如快捷键）
    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// 命令面板（Entity）。
///
/// ```ignore
/// let palette = cx.new(|cx| CommandPalette::new(cx, vec![
///     PaletteItem::new("新建文件").hint("Ctrl+N"),
///     PaletteItem::new("打开设置"),
/// ]).on_select(|label, _, cx| { /* 执行命令 */ }));
/// // render 中（滚动容器之外）：div().child(palette.clone())
/// // 宿主视图 on_action(cx.listener(... Toggle ...)) 中调用 palette.open(window, cx)
/// ```
pub struct CommandPalette {
    input: Entity<TextInput>,
    items: Vec<PaletteItem>,
    open: bool,
    highlight: usize,
    on_select: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
}

impl CommandPalette {
    pub fn new(cx: &mut Context<Self>, items: Vec<PaletteItem>) -> Self {
        let entity = cx.entity();
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .bare(true)
                .placeholder("输入命令…")
                .on_change(move |_, _, cx| {
                    let _ = entity.update(cx, |this, cx| {
                        this.highlight = 0;
                        cx.notify();
                    });
                })
        });
        Self {
            input,
            items,
            open: false,
            highlight: 0,
            on_select: None,
        }
    }

    /// 选中命令回调
    pub fn on_select(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }

    /// 打开面板并聚焦输入框
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = true;
        self.highlight = 0;
        let input = self.input.clone();
        input.update(cx, |input, cx| input.set_text_silent("", cx));
        self.input.focus_handle(cx).focus(window, cx);
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.notify();
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    fn filtered(&self, cx: &App) -> Vec<&PaletteItem> {
        let query = self.input.read(cx).text().to_lowercase();
        self.items
            .iter()
            .filter(|item| query.is_empty() || item.label.to_lowercase().contains(&query))
            .collect()
    }

    fn select(&mut self, label: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        // 静默写入：避免内部 on_change 反向 update 已租约的自身
        let input = self.input.clone();
        input.update(cx, |input, cx| input.set_text_silent("", cx));
        self.open = false;
        if let Some(on_select) = self.on_select.clone() {
            on_select(&label, window, cx);
        }
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        let count = self.filtered(cx).len();
        match event.keystroke.key.as_str() {
            "escape" => {
                self.close(cx);
                cx.stop_propagation();
            }
            "down" if count > 0 => {
                self.highlight = (self.highlight + 1).min(count - 1);
                cx.notify();
                cx.stop_propagation();
            }
            "up" if count > 0 => {
                self.highlight = self.highlight.saturating_sub(1);
                cx.notify();
                cx.stop_propagation();
            }
            "enter" if count > 0 => {
                let label = self.filtered(cx)[self.highlight.min(count - 1)]
                    .label
                    .clone();
                self.select(label, window, cx);
                cx.stop_propagation();
            }
            _ => {}
        }
    }
}

impl Render for CommandPalette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.open {
            return div().id("command-palette-closed").into_any_element();
        }

        let filtered = self.filtered(cx);
        let highlight = self.highlight.min(filtered.len().saturating_sub(1));
        let m = theme::with(crate::theme::metric::command_palette_metric);
        let entity_id = format!("command-palette-{:?}", cx.entity_id());
        let scroll_handle = window
            .use_keyed_state(
                ElementId::Name(format!("{entity_id}-scroll-handle").into()),
                cx,
                |_, _| ScrollHandle::new(),
            )
            .read(cx)
            .clone();
        let input_id = format!("{entity_id}-input");
        let input_semantics = a11y::Semantics::new(Role::SearchInput, "搜索命令")
            .id(input_id.clone())
            .value(self.input.read(cx).a11y_value())
            .placeholder("输入命令…")
            .disabled(false)
            .read_only(false)
            .focusable(true);
        let input_entity = self.input.clone();
        let input = self.input.update(cx, |input, cx| {
            let host = div()
                .id(ElementId::Name(input_id.into()))
                .flex_1()
                .track_focus(&input.focus_handle(cx));
            input
                .with_a11y_actions(input_semantics.apply(host), cx)
                .child(input_entity)
        });

        let panel = div()
            .id(ElementId::Name(format!("{entity_id}-dialog").into()))
            .flex()
            .flex_col()
            .w(px(m.width))
            .mt(px(m.margin_top))
            .rounded_lg()
            .bg(theme::bg_container())
            .border_1()
            .border_color(theme::border())
            .shadow_lg()
            .overflow_hidden()
            .on_mouse_down(MouseButton::Left, |_, _, cx: &mut App| {
                cx.stop_propagation()
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py(px(m.header_padding_block))
                    .border_b_1()
                    .border_color(theme::border())
                    .child(
                        div()
                            .text_color(theme::text_secondary())
                            .text_sm()
                            .child("›"),
                    )
                    .child(input),
            );

        let list_id = format!("{entity_id}-list");
        let mut list = a11y::Semantics::new(Role::ListBox, "命令结果")
            .id(list_id.clone())
            .orientation(Orientation::Vertical)
            .apply(
                div()
                    .id(ElementId::Name(list_id.into()))
                    .max_h(px(m.list_max_height))
                    .overflow_y_scroll()
                    .py_1(),
            );

        if filtered.is_empty() {
            list = list.child(
                a11y::Semantics::new(Role::Status, "没有匹配的命令")
                    .id(format!("{entity_id}-empty"))
                    .apply(
                        div()
                            .id(ElementId::Name(format!("{entity_id}-empty").into()))
                            .px_4()
                            .py_3()
                            .text_sm()
                            .text_color(theme::text_secondary())
                            .child("没有匹配的命令"),
                    ),
            );
        }

        for (ix, item) in filtered.iter().enumerate() {
            let entity = cx.entity();
            let label = item.label.clone();
            let mut row = div()
                .id(SharedString::from(format!("{entity_id}-item-{ix}")))
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .py(px(m.item_padding_block))
                .text_sm()
                .cursor(CursorStyle::PointingHand)
                .child(item.label.clone());
            if let Some(hint) = &item.hint {
                row = row.child(
                    div()
                        .text_xs()
                        .text_color(theme::text_secondary())
                        .child(hint.clone()),
                );
            }
            if ix == highlight {
                row = row.bg(theme::primary_bg()).text_color(theme::primary());
            } else {
                row = row.hover(|style| style.bg(theme::control_item_bg_hover()));
            }
            let item_id = format!("{entity_id}-item-{ix}");
            row = a11y::Semantics::new(Role::ListBoxOption, item.label.clone())
                .id(item_id)
                .value(item.label.clone())
                .selected(ix == highlight)
                .position(ix + 1, filtered.len())
                .apply(row);
            list = list.child(a11y::click_void(row, move |window, cx| {
                entity.update(cx, |this, cx| this.select(label.clone(), window, cx));
            }));
        }

        let mut panel = a11y::Semantics::new(Role::Dialog, "命令面板")
            .id(format!("{entity_id}-dialog"))
            .apply(panel);
        panel = panel.child(
            div()
                .flex()
                .flex_row()
                .min_h(px(0.))
                .child(
                    list.flex_1()
                        .min_w(px(0.))
                        .min_h(px(0.))
                        .track_scroll(&scroll_handle),
                )
                .child(Scrollbar::vertical(
                    ElementId::Name(format!("{entity_id}-scrollbar").into()),
                    scroll_handle,
                )),
        );

        let backdrop = div()
            .id(ElementId::Name(
                format!("command-palette-backdrop-{:?}", cx.entity_id()).into(),
            ))
            .absolute()
            .inset_0()
            .occlude()
            .bg(m.mask)
            .flex()
            .justify_center()
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &gpui::MouseDownEvent, _, cx| {
                    this.close(cx);
                }),
            )
            .child(panel);

        deferred(backdrop).into_any_element()
    }
}

impl Focusable for CommandPalette {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}
