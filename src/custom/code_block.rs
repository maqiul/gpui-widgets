//! 代码块：等宽字体面板，右上角语言标签 / 复制按钮 / 自定义动作。
//!
//! 语言名能匹配 [`CodeLanguage`] 时按行着色（复用代码编辑器的词法预设），
//! 否则按纯文本展示。

use std::ops::Range;
use std::rc::Rc;

use gpui::{
    App, ClipboardItem, ElementId, Font, Hsla, IntoElement, ParentElement, RenderOnce, Role,
    ScrollHandle, SharedString, Styled, StyledText, TextRun, Window, div, prelude::*, px,
};

use crate::code_editor::CodeLanguage;
use crate::text_area::CodeToken;
use crate::{Button, a11y, theme};

/// 代码正文的等宽字体族（与 CodeEditor / Markdown 行内代码一致）
const CODE_FONT_FAMILY: &str = "Consolas";

/// 代码块右上角的自定义动作（如「填入命令条」）。
///
/// 回调收到代码块自身的文本，宿主可据此把代码写入其它控件。
pub struct CodeAction {
    label: SharedString,
    disabled: bool,
    handler: Rc<dyn Fn(&str, &mut Window, &mut App)>,
}

impl CodeAction {
    pub fn new(
        label: impl Into<SharedString>,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            handler: Rc::new(handler),
        }
    }

    /// 禁用状态（仍展示按钮，但点击不触发）
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Clone for CodeAction {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            disabled: self.disabled,
            handler: self.handler.clone(),
        }
    }
}

/// 代码块。
///
/// ```ignore
/// CodeBlock::new("fn main() {\n    println!(\"hi\");\n}")
///     .language("rust")
///     .action(CodeAction::new("填入", |code, _, _| { /* 使用 code */ }))
/// ```
#[derive(IntoElement)]
pub struct CodeBlock {
    code: SharedString,
    id: Option<SharedString>,
    language: Option<SharedString>,
    actions: Vec<CodeAction>,
    copy_label: SharedString,
    copied_label: SharedString,
    text_size: Option<f32>,
}

impl CodeBlock {
    pub fn new(code: impl Into<SharedString>) -> Self {
        Self {
            code: code.into(),
            id: None,
            language: None,
            actions: Vec::new(),
            copy_label: "复制".into(),
            copied_label: "已复制 ✓".into(),
            text_size: None,
        }
    }

    /// 实例唯一 id；不传则回退代码指针，以保持 RenderOnce 跨帧稳定。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 右上角显示的语言标签（仅展示，不做语法高亮）
    pub fn language(mut self, language: impl Into<SharedString>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// 追加一个动作按钮（显示在复制按钮左侧）
    pub fn action(mut self, action: CodeAction) -> Self {
        self.actions.push(action);
        self
    }

    /// 复制按钮的常态 / 已复制态文案（多语言宿主传入译文；默认中文，零回归）。
    pub fn copy_labels(
        mut self,
        copy: impl Into<SharedString>,
        copied: impl Into<SharedString>,
    ) -> Self {
        self.copy_label = copy.into();
        self.copied_label = copied.into();
        self
    }

    /// 覆盖代码正文字号（行高按既有额外行距派生）；不调用则跟随度量默认。
    /// CodeBlock 是 RenderOnce：宿主每帧带新值重建即活更新，无需 setter。
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = Some(size);
        self
    }
}

/// 派生代码块根节点、滚动区、正文与滚动条 id：实例 id 优先，未传则回退代码指针。
fn element_ids(id: Option<&str>, fallback_key: &str) -> (String, String, String, String) {
    let key = id.unwrap_or(fallback_key);
    (
        format!("code-block-root-{key}"),
        format!("code-block-scroll-{key}"),
        format!("code-block-body-{key}"),
        format!("code-block-scrollbar-{key}"),
    )
}

impl RenderOnce for CodeBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let m = theme::with(crate::theme::metric::code_block_metric);
        let m = match self.text_size {
            Some(size) => m.with_font_size(size),
            None => m,
        };
        let code = self.code.clone();
        let copied = cx.default_global::<CodeCopyState>().0.clone();
        let is_copied = copied.map(|c| c == code).unwrap_or(false);
        let highlighter = self
            .language
            .as_ref()
            .and_then(|name| CodeLanguage::from_name(name))
            .and_then(|language| language.highlighter());
        let font = code_font(window);
        let scroll_tag = format!("{:p}", code.as_ptr());
        let (root_id, scroll_id, body_id, scrollbar_id) =
            element_ids(self.id.as_deref(), &scroll_tag);
        let language = self.language.clone().unwrap_or_else(|| "code".into());
        let line_count = code.lines().count().max(1);

        let mut buttons = div()
            .id(ElementId::Name(format!("{root_id}-actions").into()))
            .flex()
            .items_center()
            .gap(px(m.action_gap));
        for (index, action) in self.actions.iter().enumerate() {
            let handler = action.handler.clone();
            let action_code = code.clone();
            buttons = buttons.child(
                Button::new(action.label.clone())
                    .id(format!("{root_id}-action-{index}"))
                    .small()
                    .disabled(action.disabled)
                    .on_click(move |_, window, cx| handler(&action_code, window, cx)),
            );
        }

        let copy_code = code.clone();
        buttons = buttons.child(
            Button::new(if is_copied {
                self.copied_label.clone()
            } else {
                self.copy_label.clone()
            })
            .id(format!("{root_id}-copy"))
            .small()
            .on_click(move |_, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(copy_code.to_string()));
                cx.default_global::<CodeCopyState>().0 = Some(copy_code.clone());
            }),
        );

        let handle = window
            .use_keyed_state(ElementId::Name(scroll_id.into()), cx, |_, _| {
                ScrollHandle::new()
            })
            .read(cx)
            .clone();
        let body = a11y::Semantics::new(Role::Code, format!("{language} 代码"))
            .id(body_id.clone())
            .value(code.clone())
            .read_only(true)
            .apply(
                div()
                    .id(ElementId::Name(body_id.into()))
                    .w_full()
                    .overflow_x_scroll()
                    .track_scroll(&handle)
                    .font_family(crate::theme::mono_family_or(CODE_FONT_FAMILY))
                    .text_size(px(m.font_size))
                    .line_height(px(m.line_height))
                    .text_color(theme::text())
                    .children(self.code.split('\n').map(|line| {
                        let line = if line.is_empty() { " " } else { line };
                        match &highlighter {
                            Some(highlight) => {
                                let runs = line_runs(line, &highlight(line), &font);
                                div()
                                    .child(StyledText::new(line.to_string()).with_runs(runs))
                                    .into_any_element()
                            }
                            None => div().child(line.to_string()).into_any_element(),
                        }
                    })),
            );

        let buttons = a11y::Semantics::new(Role::Toolbar, "代码操作")
            .id(format!("{root_id}-actions"))
            .apply(buttons);
        let root = div()
            .id(ElementId::Name(root_id.clone().into()))
            .relative()
            .w_full()
            .rounded_md()
            .bg(theme::bg_layout())
            .border_1()
            .border_color(theme::border())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .pt_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .child(language.clone()),
                    )
                    .child(buttons),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .px_3()
                    .pb_3()
                    .pt_1()
                    .child(body)
                    .child(crate::Scrollbar::horizontal(
                        ElementId::Name(scrollbar_id.into()),
                        handle,
                    )),
            );

        a11y::Semantics::new(Role::Region, format!("{language} 代码块，{line_count} 行"))
            .id(root_id)
            .apply(root)
    }
}

/// 代码正文用的等宽字体（字号/行高由外层 text style 决定）
fn code_font(window: &Window) -> Font {
    let mut font = window.text_style().font();
    font.family = crate::theme::mono_family_or(CODE_FONT_FAMILY).into();
    font
}

/// 把一行词法 token 转成覆盖整行的 TextRun；token 之间的空隙用正文色补齐。
fn line_runs(line: &str, tokens: &[CodeToken], font: &Font) -> Vec<TextRun> {
    let mut runs: Vec<TextRun> = Vec::new();
    let mut push = |range: Range<usize>, color: Hsla, bold: bool| {
        if range.end <= range.start {
            return;
        }
        let mut font = font.clone();
        if bold {
            font = font.bold();
        }
        runs.push(TextRun {
            len: range.end - range.start,
            font,
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        });
    };

    let mut cursor = 0;
    for token in tokens {
        if token.range.start > cursor {
            push(cursor..token.range.start, theme::text(), false);
        }
        push(token.range.clone(), token.color, token.bold);
        cursor = token.range.end;
    }
    if cursor < line.len() {
        push(cursor..line.len(), theme::text(), false);
    }
    runs
}

/// 记录最近复制的代码内容（用于按钮短暂显示"已复制"）
#[derive(Default)]
struct CodeCopyState(Option<SharedString>);

impl gpui::Global for CodeCopyState {}
#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> Font {
        gpui::font(crate::theme::mono_family_or(CODE_FONT_FAMILY))
    }

    #[test]
    fn code_block_line_runs_cover_whole_line() {
        let line = "let answer = 42; // 注释";
        let tokens = CodeLanguage::Rust.highlighter().unwrap()(line);
        let runs = line_runs(line, &tokens, &font());
        assert_eq!(runs.iter().map(|run| run.len).sum::<usize>(), line.len());
        assert!(runs.iter().all(|run| run.font.family == CODE_FONT_FAMILY));
    }

    #[test]
    fn code_block_line_runs_keyword_is_bold_primary() {
        let line = "let x = 1;";
        let tokens = CodeLanguage::Rust.highlighter().unwrap()(line);
        let runs = line_runs(line, &tokens, &font());
        let keyword = &runs[0];
        assert_eq!(keyword.len, "let".len());
        assert_eq!(keyword.color, theme::primary());
        assert_eq!(keyword.font.weight, gpui::FontWeight::BOLD);
    }

    #[test]
    fn code_block_line_runs_without_tokens_is_plain() {
        let line = "任意文本 arbitrary";
        let runs = line_runs(line, &[], &font());
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].len, line.len());
        assert_eq!(runs[0].color, theme::text());
    }

    #[test]
    fn code_block_line_runs_fill_token_gaps() {
        let line = "ab cd";
        let tokens = vec![
            CodeToken::new(0..2, theme::danger()),
            CodeToken::new(3..5, theme::success()),
        ];
        let runs = line_runs(line, &tokens, &font());
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[1].len, 1);
        assert_eq!(runs[1].color, theme::text());
        assert_eq!(runs.iter().map(|run| run.len).sum::<usize>(), line.len());
    }

    #[test]
    fn code_block_unknown_language_stays_plain() {
        assert!(CodeLanguage::from_name("brainfuck").is_none());
        assert!(
            CodeLanguage::from_name("rust")
                .unwrap()
                .highlighter()
                .is_some()
        );
        assert!(
            CodeLanguage::from_name("plain")
                .unwrap()
                .highlighter()
                .is_none()
        );
    }

    #[test]
    fn code_block_id_default_derives_from_pointer_key() {
        let (root, scroll, body, scrollbar) = element_ids(None, "0x1234");
        assert_eq!(root, "code-block-root-0x1234");
        assert_eq!(scroll, "code-block-scroll-0x1234");
        assert_eq!(body, "code-block-body-0x1234");
        assert_eq!(scrollbar, "code-block-scrollbar-0x1234");
    }

    #[test]
    fn code_block_id_stable_across_repeated_calls() {
        for _ in 0..3 {
            assert_eq!(element_ids(None, "0x1234"), element_ids(None, "0x1234"));
            assert_eq!(
                element_ids(Some("sample"), "0x1234"),
                element_ids(Some("sample"), "0x1234")
            );
        }
    }

    #[test]
    fn code_block_id_distinct_pointer_keys_do_not_collide() {
        let first = element_ids(None, "0x1234");
        let second = element_ids(None, "0x5678");
        assert_ne!(first.0, second.0);
        assert_ne!(first.1, second.1);
        assert_ne!(first.2, second.2);
        assert_ne!(first.3, second.3);
    }

    #[test]
    fn code_block_id_explicit_overrides_pointer_key() {
        let ids = element_ids(Some("sample-7"), "0x1234");
        assert_eq!(ids.0, "code-block-root-sample-7");
        assert_eq!(ids.1, "code-block-scroll-sample-7");
        assert_eq!(ids.2, "code-block-body-sample-7");
        assert_eq!(ids.3, "code-block-scrollbar-sample-7");
    }

    #[test]
    fn code_block_id_parts_share_key_with_distinct_suffixes() {
        let ids = element_ids(Some("k"), "0x1234");
        assert_eq!(ids.0.strip_prefix("code-block-root-"), Some("k"));
        assert_eq!(ids.1.strip_prefix("code-block-scroll-"), Some("k"));
        assert_eq!(ids.2.strip_prefix("code-block-body-"), Some("k"));
        assert_eq!(ids.3.strip_prefix("code-block-scrollbar-"), Some("k"));
        let parts = [ids.0, ids.1, ids.2, ids.3];
        for i in 0..parts.len() {
            for j in i + 1..parts.len() {
                assert_ne!(parts[i], parts[j]);
            }
        }
    }

    #[test]
    fn code_block_id_same_pointer_key_still_collides() {
        // 同内容默认共用指针 key，叠层必须由宿主传 .id()。
        assert_eq!(element_ids(None, "0x1234"), element_ids(None, "0x1234"));
        assert_ne!(
            element_ids(Some("a"), "0x1234"),
            element_ids(Some("b"), "0x1234")
        );
    }
}
