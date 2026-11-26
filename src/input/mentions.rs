//! 提及输入框：输入 @ 触发候选浮层，选中后替换为 "@名称 "（对应 antd Mentions）。
//!
//! 本身是 Entity，内部包装 TextInput。使用前需调用 `text_input::bind_keys`。
//! 触发规则：文本中最后一个 @ 之后不含空白字符时，以其后的内容作为过滤词。

use std::rc::Rc;

use gpui::{
    App, Context, CursorStyle, ElementId, Entity, Focusable, IntoElement, Render, Role,
    ScrollHandle, SharedString, StatefulInteractiveElement, Window, div, prelude::*, px,
};

use crate::a11y::{self, Semantics};

use crate::text_input::{self, InputSize, InputStatus, InputStyle, InputVariant};
use crate::{Scrollbar, TextInput, popup::anchored, theme};

/// Mentions 外框状态矩阵，语义与 [`text_input::state`] 一致（内部即 TextInput）。
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

/// 提及输入框（Entity，用法同 TextInput）。
///
/// ```ignore
/// let mentions = cx.new(|cx| Mentions::new(cx)
///     .options(vec!["张三".into(), "李四".into()])
///     .on_select(|name, _, cx| { /* 选中了某人 */ }));
/// ```
pub struct Mentions {
    input: Entity<TextInput>,
    options: Vec<SharedString>,
    placeholder: SharedString,
    open: bool,
    query: String,
    mention_start: usize,
    on_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    on_select: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    disabled: bool,
}

impl Mentions {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let entity = cx.entity();
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder("输入 @ 提及某人")
                .on_change(move |text, window, cx| {
                    let text = text.to_string();
                    let _ = entity.update(cx, |this, cx| {
                        this.handle_change(&text, window, cx);
                    });
                })
        });
        Self {
            input,
            options: Vec::new(),
            placeholder: "输入 @ 提及某人".into(),
            open: false,
            query: String::new(),
            mention_start: 0,
            on_change: None,
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

    /// 禁用态：灰底、不可输入
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 候选名单
    pub fn options(mut self, options: Vec<SharedString>) -> Self {
        self.options = options;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 内容变化回调
    pub fn on_change(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    /// 选中候选项时回调其名称
    pub fn on_select(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }

    pub fn text(&self, cx: &App) -> String {
        self.input.read(cx).text().to_string()
    }

    pub fn set_text(
        &mut self,
        text: impl Into<SharedString>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open = false;
        let input = self.input.clone();
        input.update(cx, |input, cx| input.set_text_silent(text, cx));
    }

    /// 解析最后一个 @，决定是否展开候选
    fn handle_change(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(at) = text.rfind('@') {
            let after = &text[at + 1..];
            if !after.chars().any(|c| c.is_whitespace()) {
                self.query = after.to_string();
                self.mention_start = at;
                self.open = !self.options.is_empty();
            } else {
                self.open = false;
            }
        } else {
            self.open = false;
        }
        if let Some(on_change) = self.on_change.clone() {
            on_change(text, window, cx);
        }
        cx.notify();
    }

    fn set_open(&mut self, open: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.open == open || (open && self.options.is_empty()) {
            return;
        }
        if open {
            let text = self.input.read(cx).text().to_string();
            self.mention_start = text.rfind('@').unwrap_or(text.len());
            self.query.clear();
        }
        self.open = open;
        self.input.read(cx).focus_handle(cx).focus(window, cx);
        cx.notify();
    }

    /// 选中候选项：替换 "@查询词" 为 "@名称 "。
    /// 内部用静默写入，避免回调链反向 update 本实体。
    fn select(&mut self, item: &SharedString, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).text().to_string();
        let new_text = if self.mention_start <= text.len() {
            format!("{}@{} ", &text[..self.mention_start], item)
        } else {
            text
        };
        let input = self.input.clone();
        input.update(cx, |input, cx| input.set_text_silent(new_text.clone(), cx));
        self.open = false;
        self.query.clear();
        if let Some(on_change) = self.on_change.clone() {
            on_change(&new_text, window, cx);
        }
        if let Some(on_select) = self.on_select.clone() {
            on_select(item, window, cx);
        }
        cx.notify();
    }
}

impl Render for Mentions {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let placeholder = self.placeholder.clone();
        let (variant, status, size, disabled) =
            (self.variant, self.status, self.size, self.disabled);
        let query = self.query.to_lowercase();
        let expanded = !disabled
            && self.open
            && self
                .options
                .iter()
                .any(|option| option.to_lowercase().contains(&query));
        let entity = cx.entity();
        self.input.update(cx, |input, cx| {
            input.set_placeholder(placeholder);
            input.set_form_state(variant, status, size, disabled, cx);
            input.set_a11y_semantics(
                Semantics::new(Role::ComboBox, "提及")
                    .id(format!("mentions-{:?}", cx.entity_id()))
                    .expanded(expanded)
                    .disabled(disabled)
                    .focusable(!disabled),
            );
            input.set_a11y_set_open({
                let entity = entity.clone();
                move |open, window, cx| {
                    let _ = entity.update(cx, |this, cx| this.set_open(open, window, cx));
                }
            });
        });

        let popup = if !disabled && self.open {
            let query = self.query.to_lowercase();
            let filtered: Vec<SharedString> = self
                .options
                .iter()
                .filter(|o| o.to_lowercase().contains(&query))
                .cloned()
                .collect();

            if filtered.is_empty() {
                None
            } else {
                let entity = cx.entity();
                let entity_id = format!("mentions-{:?}", cx.entity_id());
                let scroll_handle = window
                    .use_keyed_state(
                        ElementId::Name(format!("{entity_id}-scroll-handle").into()),
                        cx,
                        |_, _| ScrollHandle::new(),
                    )
                    .read(cx)
                    .clone();
                let menu = div()
                    .occlude()
                    .w(px(200.))
                    .max_h(px(200.))
                    .overflow_hidden()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(theme::border())
                    .bg(theme::surface())
                    .shadow_lg()
                    .flex()
                    .flex_row()
                    .on_mouse_down_out({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            let _ = entity.update(cx, |this, cx| {
                                this.open = false;
                                cx.notify();
                            });
                        }
                    });
                let mut list = Semantics::new(Role::ListBox, "提及建议")
                    .id(format!("{entity_id}-list"))
                    .disabled(disabled)
                    .apply(
                        div()
                            .id(ElementId::Name(format!("{entity_id}-list").into()))
                            .flex_1()
                            .min_w(px(0.))
                            .min_h(px(0.))
                            .flex()
                            .flex_col()
                            .overflow_y_scroll()
                            .track_scroll(&scroll_handle),
                    );

                let option_count = filtered.len();
                for (ix, option) in filtered.into_iter().enumerate() {
                    let entity = entity.clone();
                    let item = option.clone();
                    let row = div()
                        .id(ElementId::Name(format!("{entity_id}-item-{ix}").into()))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py(px(6.))
                        .text_sm()
                        .cursor(CursorStyle::PointingHand)
                        .hover(|style| style.bg(theme::hover_bg()))
                        .child(div().text_color(theme::accent()).child("@"))
                        .child(option);
                    let row = Semantics::new(Role::ListBoxOption, item.clone())
                        .id(format!("{entity_id}-item-{ix}"))
                        .selected(false)
                        .position(ix + 1, option_count)
                        .disabled(disabled)
                        .apply(row);
                    if !disabled {
                        list = list.child(a11y::click_void(row, move |window, cx| {
                            let _ = entity.update(cx, |this, cx| {
                                this.select(&item, window, cx);
                            });
                        }));
                    } else {
                        list = list.child(row);
                    }
                }
                Some(menu.child(list).child(Scrollbar::vertical(
                    ElementId::Name(format!("{entity_id}-scrollbar").into()),
                    scroll_handle,
                )))
            }
        } else {
            None
        };

        anchored(self.input.clone().into_any_element(), popup)
    }
}
