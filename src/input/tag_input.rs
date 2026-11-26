//! 标签输入框：回车生成标签、退格删除末尾标签、✕ 移除单个标签。

use std::rc::Rc;

use gpui::{
    App, Context, CursorStyle, Entity, FocusHandle, Focusable, IntoElement, KeyDownEvent,
    ParentElement, Render, Role, SharedString, Styled, Window, div, prelude::*, px,
};

use crate::a11y::Semantics;

use crate::text_input::{
    self, InputSize, InputStatus, InputStyle, InputVariant, apply_frame_auto, hover_frame,
};
use crate::{Tag, TextInput, theme};

/// TagInput 外框状态矩阵，语义与 [`text_input::state`] 一致（同一套 Input 令牌）。
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

/// 标签输入框（Entity）。
///
/// ```ignore
/// let tags = cx.new(|cx| TagInput::new(cx).placeholder("输入后回车添加"));
/// // 读取：tags.read(cx).tags()
/// ```
pub struct TagInput {
    input: Entity<TextInput>,
    tags: Vec<SharedString>,
    placeholder: SharedString,
    on_change: Option<Rc<dyn Fn(&Vec<SharedString>, &mut Window, &mut App)>>,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    disabled: bool,
}

impl TagInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new(cx).bare(true));
        Self {
            input,
            tags: Vec::new(),
            placeholder: "输入后回车添加".into(),
            on_change: None,
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            disabled: false,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 初始标签
    pub fn initial_tags(mut self, tags: Vec<SharedString>) -> Self {
        self.tags = tags;
        self
    }

    /// 标签集合变化回调
    pub fn on_change(
        mut self,
        listener: impl Fn(&Vec<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
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

    /// 当前标签集合
    pub fn tags(&self) -> &[SharedString] {
        &self.tags
    }

    fn notify_tags(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.tags, window, cx);
        }
        cx.notify();
    }

    fn commit_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let draft = self.input.read(cx).text().trim().to_string();
        if draft.is_empty() {
            return;
        }
        let draft: SharedString = draft.into();
        if !self.tags.contains(&draft) {
            self.tags.push(draft);
        }
        let input = self.input.clone();
        input.update(cx, |input, cx| input.set_text("", window, cx));
        self.notify_tags(window, cx);
    }

    fn remove_tag(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        if ix < self.tags.len() {
            self.tags.remove(ix);
            self.notify_tags(window, cx);
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "enter" => {
                self.commit_draft(window, cx);
                cx.stop_propagation();
            }
            "backspace" => {
                // 草稿为空时，退格删除最后一个标签
                if self.input.read(cx).text().is_empty() && !self.tags.is_empty() {
                    self.tags.pop();
                    self.notify_tags(window, cx);
                    cx.stop_propagation();
                }
            }
            _ => {}
        }
    }
}

impl Render for TagInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity_tag = format!("{:?}", cx.entity_id());
        let focused = self.input.focus_handle(cx).is_focused(window);
        let disabled = self.disabled;
        let (variant, status, size) = (self.variant, self.status, self.size);
        let style = theme::with(|t| state(variant, status, size, t, false, focused, disabled));
        let hover_style = theme::with(|t| state(variant, status, size, t, true, false, disabled));
        let gap = theme::with(|t| t.padding_xxs);
        let placeholder = self.placeholder.clone();
        let draft = self.input.read(cx).text().to_string();
        let tags = self
            .tags
            .iter()
            .map(|tag| tag.as_ref())
            .collect::<Vec<_>>()
            .join(", ");
        let a11y_value = match (tags.is_empty(), draft.is_empty()) {
            (true, _) => draft,
            (_, true) => tags,
            (false, false) => format!("{tags} {draft}"),
        };
        self.input.update(cx, |input, cx| {
            input.set_placeholder(placeholder.clone());
            input.set_form_state(variant, status, size, disabled, cx);
        });

        let mut row = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(gap))
            .w_full()
            .px(px(style.padding_inline))
            .py(px(style.padding_block))
            .min_h(px(style.height))
            .line_height(px(style.line_height))
            .text_size(px(style.font_size))
            .text_color(style.fg);
        // 标签会换行，外框高度自动增长：只借用底色 / 描边 / 聚焦环，高度交给 min_h。
        row = apply_frame_auto(row, &style);
        row = hover_frame(row, &hover_style, focused);
        let semantics = Semantics::new(Role::TextInput, "标签输入")
            .id(format!("tag-input-{entity_tag}"))
            .value(a11y_value)
            .placeholder(placeholder)
            .disabled(disabled)
            .invalid(status == InputStatus::Error)
            .focusable(!disabled);
        let mut row = row.id(semantics.element_id());
        if disabled {
            row = row.cursor(CursorStyle::Arrow);
        } else {
            row = row
                .track_focus(&self.input.focus_handle(cx))
                .on_key_down(cx.listener(Self::on_key_down));
        }

        for (ix, tag) in self.tags.iter().enumerate() {
            let entity = cx.entity();
            row = row.child(
                Tag::new(tag.clone())
                    .id(format!("tag-input-{entity_tag}-tag-{ix}"))
                    .disabled(disabled)
                    .on_close(move |_, window, cx| {
                        entity.update(cx, |this, cx| this.remove_tag(ix, window, cx));
                    }),
            );
        }

        let row = row.child(div().flex_1().min_w(px(0.)).child(self.input.clone()));
        self.input.update(cx, |input, cx| {
            input.with_a11y_actions(semantics.apply(row), cx)
        })
    }
}

impl Focusable for TagInput {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}
