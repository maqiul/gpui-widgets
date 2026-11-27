//! Markdown 源码编辑与实时预览。

use std::rc::Rc;

use gpui::{
    App, Div, Entity, IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled,
    Window, div, prelude::*, px,
};

use crate::{Markdown, TextArea, a11y::Semantics, theme};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkdownEditorMode {
    #[default]
    Split,
    Editor,
    Preview,
}

#[derive(IntoElement)]
pub struct MarkdownEditor {
    id: SharedString,
    text: SharedString,
    title: SharedString,
    placeholder: SharedString,
    mode: MarkdownEditorMode,
    height: gpui::Pixels,
    on_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
}

impl MarkdownEditor {
    #[track_caller]
    pub fn new(text: impl Into<SharedString>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: format!(
                "markdown-editor-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            text: text.into(),
            title: "Markdown 编辑器".into(),
            placeholder: "输入 Markdown 源码…".into(),
            mode: MarkdownEditorMode::default(),
            height: px(320.),
            on_change: None,
        }
    }

    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn mode(mut self, mode: MarkdownEditorMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn height(mut self, height: gpui::Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    fn preview(&self) -> Stateful<Div> {
        div()
            .id(SharedString::from(format!("{}-preview-scroll", self.id)))
            .flex_1()
            .min_w_0()
            .h(self.height)
            .overflow_y_scroll()
            .p_4()
            .bg(theme::bg_container())
            .child(
                Markdown::new(self.text.clone())
                    .id(SharedString::from(format!("{}-preview", self.id))),
            )
    }

    fn editor(&self, input: Entity<TextArea>) -> impl IntoElement {
        div().flex_1().min_w_0().child(input)
    }
}

struct EditorState {
    input: Entity<TextArea>,
}

impl RenderOnce for MarkdownEditor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(
            gpui::ElementId::Name(format!("markdown-editor-state-{}", self.id).into()),
            cx,
            |_, state_cx| EditorState {
                input: state_cx.new(|input_cx| {
                    let mut input = TextArea::new(input_cx)
                        .initial_text(self.text.clone())
                        .placeholder(self.placeholder.clone())
                        .height(self.height)
                        .monospace()
                        .line_numbers(true);
                    input.set_a11y_semantics(
                        Semantics::new(gpui::Role::MultilineTextInput, "Markdown 源码")
                            .id(format!("{}-input", self.id)),
                    );
                    if let Some(on_change) = self.on_change.clone() {
                        input =
                            input.on_change(move |text, window, cx| on_change(text, window, cx));
                    }
                    input.set_caret(0, input_cx);
                    input
                }),
            },
        );
        let input = state.read(cx).input.clone();
        let current = input.read(cx).text().to_string();
        if current != self.text {
            input.update(cx, |input, cx| input.set_text_silent(self.text.clone(), cx));
        }

        let lines = self.text.lines().count();
        let chars = self.text.chars().count();
        let label = format!("{}，{lines} 行，{chars} 字符", self.title);
        let mode_label = match self.mode {
            MarkdownEditorMode::Split => "分栏",
            MarkdownEditorMode::Editor => "仅源码",
            MarkdownEditorMode::Preview => "仅预览",
        };
        let mut body = div().flex().w_full().min_w_0();
        if self.mode != MarkdownEditorMode::Preview {
            body = body.child(self.editor(input));
        }
        if self.mode != MarkdownEditorMode::Editor {
            body = body.child(self.preview());
        }

        Semantics::new(gpui::Role::Group, label)
            .id(self.id.clone())
            .apply(
                div()
                    .id(self.id.clone())
                    .flex()
                    .flex_col()
                    .w_full()
                    .min_w_0()
                    .rounded_lg()
                    .border_1()
                    .border_color(theme::border())
                    .overflow_hidden()
                    .bg(theme::bg_container())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .h(px(38.))
                            .px_3()
                            .border_b_1()
                            .border_color(theme::border_secondary())
                            .bg(theme::bg_elevated())
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text())
                                    .child(self.title),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_tertiary())
                                    .child(format!("{lines} 行 · {chars} 字符 · {mode_label}")),
                            ),
                    )
                    .child(body),
            )
    }
}
