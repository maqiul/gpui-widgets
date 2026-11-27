//! DiffView 与 MarkdownEditor 的运行时交互验收示例。

use gpui::{
    App, Bounds, Context, Div, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::input::text_area::bind_keys;
use gpui_widgets::{
    Button, ButtonVariant, DiffMode, DiffView, MarkdownEditor, MarkdownEditorMode, Theme, theme,
};

actions!(diff_markdown_demo, [Quit]);

const OLD: &str = "# Release notes\n- Stable editor\n- Old export path\nPerformance baseline";
const NEW: &str =
    "# Release notes\n- Stable editor\n- New export path\n- Markdown preview\nPerformance baseline";
const SOURCE: &str = "# Markdown 实时预览\n\n**编辑左侧源码**，右侧立即更新。\n\n- 支持标题、列表与表格\n- Enter 换行，Tab 缩进\n\n```rust\nlet value = 42;\n```\n\n| 功能 | 状态 |\n| --- | --- |\n| 编辑 | 完成 |\n| 预览 | 完成 |";

fn caption(text: &str) -> Div {
    div()
        .text_sm()
        .text_color(theme::text_secondary())
        .child(text.to_string())
}

fn mode_button(
    label: &'static str,
    id: &'static str,
    selected: bool,
    cx: &mut Context<DiffMarkdownDemo>,
) -> Button {
    Button::new(label)
        .id(id)
        .small()
        .variant(if selected {
            ButtonVariant::Solid
        } else {
            ButtonVariant::Outlined
        })
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_mode(label, cx);
        }))
}

struct DiffMarkdownDemo {
    source: String,
    editor_mode: MarkdownEditorMode,
    diff_mode: DiffMode,
    status: String,
    dark: bool,
}

impl DiffMarkdownDemo {
    fn new(_: &mut Context<Self>) -> Self {
        Self {
            source: SOURCE.into(),
            editor_mode: MarkdownEditorMode::Split,
            diff_mode: DiffMode::Split,
            status: "实时预览已就绪".into(),
            dark: false,
        }
    }

    fn set_mode(&mut self, label: &str, cx: &mut Context<Self>) {
        match label {
            "分栏" => self.editor_mode = MarkdownEditorMode::Split,
            "仅源码" => self.editor_mode = MarkdownEditorMode::Editor,
            "仅预览" => self.editor_mode = MarkdownEditorMode::Preview,
            "统一" => self.diff_mode = DiffMode::Unified,
            "分栏差异" => self.diff_mode = DiffMode::Split,
            _ => {}
        }
        self.status = format!("已切换到{label}");
        cx.notify();
    }
}

impl Render for DiffMarkdownDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("diff-markdown-demo-scroll")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .bg(theme::bg_layout())
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme::text())
                    .child("Diff / Markdown Editor"),
            )
            .child(caption(
                "上行级 Myers 差异；下方源码受控同步，支持分栏、仅源码和仅预览。",
            ))
            .child(
                div().flex().justify_end().child(
                    Button::new(if self.dark {
                        "浅色主题"
                    } else {
                        "深色主题"
                    })
                    .id("toggle-theme")
                    .small()
                    .secondary()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.dark = !this.dark;
                        theme::set_theme(if this.dark {
                            Theme::dark()
                        } else {
                            Theme::light()
                        });
                        cx.notify();
                    })),
                ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(caption("差异视图"))
                    .child(mode_button(
                        "分栏差异",
                        "diff-mode-split",
                        self.diff_mode == DiffMode::Split,
                        cx,
                    ))
                    .child(mode_button(
                        "统一",
                        "diff-mode-unified",
                        self.diff_mode == DiffMode::Unified,
                        cx,
                    )),
            )
            .child(
                DiffView::new(OLD, NEW)
                    .id("release-diff")
                    .title("v0.3.1 变更")
                    .mode(self.diff_mode),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(caption("Markdown 模式"))
                    .child(mode_button(
                        "分栏",
                        "markdown-mode-split",
                        self.editor_mode == MarkdownEditorMode::Split,
                        cx,
                    ))
                    .child(mode_button(
                        "仅源码",
                        "markdown-mode-editor",
                        self.editor_mode == MarkdownEditorMode::Editor,
                        cx,
                    ))
                    .child(mode_button(
                        "仅预览",
                        "markdown-mode-preview",
                        self.editor_mode == MarkdownEditorMode::Preview,
                        cx,
                    ))
                    .child(
                        Button::new("程序化追加")
                            .id("append-source")
                            .small()
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.source
                                    .push_str("\n\n> 程序化追加也会同步到源码与预览。");
                                this.status = "已通过受控文本追加内容".into();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                MarkdownEditor::new(self.source.clone())
                    .id("markdown-demo")
                    .mode(self.editor_mode)
                    .on_change(cx.listener(|this, text: &str, _, cx| {
                        this.source = text.to_string();
                        this.status = format!("源码已更新：{} 字符", text.chars().count());
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child(self.status.clone()),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        bind_keys(cx);
        theme::set_theme(Theme::light());
        let bounds = Bounds::centered(None, size(px(1180.), px(860.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Diff / Markdown Editor".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(DiffMarkdownDemo::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
