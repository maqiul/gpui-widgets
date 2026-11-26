//! 截图基线用面板：与 `harness/antd_input_reference.html` kind=form 同构（纵向 Form.Item）。
//!
//! ```text
//! cargo run --offline --example form_panel_shot -- --appearance light
//! ```
//!
//! 面板几何 420×596（pad 16、行带 116、行高 100、label 140、gap 8、cell 240）。
//! 横向布局 / 冒号 / 右对齐由探针与矩阵测试覆盖（参照页面板行内不渲染 horizontal）。

use std::env;

use gpui::{
    App, Bounds, Context, Entity, IntoElement, KeyBinding, ParentElement, Render, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::text_input::{InputSize, InputStatus};
use gpui_widgets::{FormItem, TextInput, Theme, theme};

actions!(form_panel_shot, [Quit]);

const ROWS: [(&str, bool, bool, bool, InputSize); 5] = [
    ("default", false, false, false, InputSize::Middle),
    ("required", true, false, false, InputSize::Middle),
    ("error", false, true, false, InputSize::Middle),
    ("extra", false, false, true, InputSize::Middle),
    ("small", false, false, false, InputSize::Small),
];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 100.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 596.0;

struct Args {
    dark: bool,
    accent: Option<u32>,
}

fn parse_args() -> Args {
    let argv: Vec<String> = env::args().skip(1).collect();
    let mut args = Args {
        dark: false,
        accent: None,
    };
    let mut i = 0;
    while i < argv.len() {
        let value = argv.get(i + 1).map(String::as_str).unwrap_or_default();
        match argv[i].as_str() {
            "--appearance" => args.dark = value == "dark",
            "--accent" => {
                args.accent = Some(
                    u32::from_str_radix(value.trim_start_matches('#'), 16)
                        .expect("--accent 需要 6 位十六进制"),
                )
            }
            other => panic!("未知参数 {other}"),
        }
        i += 2;
    }
    args
}

struct Panel {
    inputs: Vec<Entity<TextInput>>,
}

impl Panel {
    fn new(cx: &mut Context<Self>) -> Self {
        let inputs = ROWS
            .iter()
            .map(|(_, _, error, _, size)| {
                let (error, size) = (*error, *size);
                cx.new(move |cx| {
                    TextInput::new(cx)
                        .placeholder(SharedString::default())
                        .size(size)
                        .status(if error {
                            InputStatus::Error
                        } else {
                            InputStatus::Default
                        })
                })
            })
            .collect();
        Self { inputs }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut panel = div()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .p(px(PAD))
            .gap(px(ROW_GAP))
            .bg(theme::bg_container());
        for ((label, required, error, extra, size), input) in ROWS.iter().zip(self.inputs.iter()) {
            let mut item = FormItem::new("Name").size(*size);
            if *required {
                item = item.required();
            }
            if *error {
                item = item.error("Please input name");
            }
            if *extra {
                item = item.help("Extra help text");
            }
            item = item.child(div().w_full().child(input.clone()));
            panel = panel.child(row(label, item));
        }
        panel
    }
}

fn row(label: &'static str, item: FormItem) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_start()
        .h(px(ROW_HEIGHT))
        .gap(px(CELL_GAP))
        .child(
            div()
                .w(px(LABEL_WIDTH))
                .text_size(px(12.))
                .text_color(theme::text_tertiary())
                .child(label),
        )
        .child(div().w(px(CELL_WIDTH)).child(item))
}

fn main() {
    let args = parse_args();
    gpui_platform::application().run(move |cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        let mut theme = if args.dark {
            Theme::dark()
        } else {
            Theme::light()
        };
        if let Some(accent) = args.accent {
            theme = theme.with_accent(rgb(accent));
        }
        theme::set_theme(theme);
        let bounds = Bounds::centered(None, size(px(PANEL_WIDTH), px(PANEL_HEIGHT)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("form_panel_shot".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Panel::new(cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
