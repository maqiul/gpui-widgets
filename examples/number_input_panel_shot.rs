//! 截图基线用面板：与 `harness/antd_input_reference.html` kind=numberinput 同构。
//!
//! ```text
//! cargo run --offline --example number_input_panel_shot -- --appearance light
//! ```
//!
//! 面板几何 420×352（pad 16、行带 56、label 140、gap 8、cell 240）两侧共同约定。
//! 步进列用 `.controls_visible(true)`（antd 组件令牌 handleVisible:true）常显。

use std::env;

use gpui::{
    App, Bounds, Context, Entity, IntoElement, KeyBinding, ParentElement, Render, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::text_input::{InputSize, InputStatus, InputVariant};
use gpui_widgets::{NumberInput, Theme, theme};

actions!(number_input_panel_shot, [Quit]);

const ROWS: [(&str, InputVariant, InputSize, InputStatus, bool); 6] = [
    (
        "default",
        InputVariant::Outlined,
        InputSize::Middle,
        InputStatus::Default,
        false,
    ),
    (
        "filled",
        InputVariant::Filled,
        InputSize::Middle,
        InputStatus::Default,
        false,
    ),
    (
        "small",
        InputVariant::Outlined,
        InputSize::Small,
        InputStatus::Default,
        false,
    ),
    (
        "large",
        InputVariant::Outlined,
        InputSize::Large,
        InputStatus::Default,
        false,
    ),
    (
        "disabled",
        InputVariant::Outlined,
        InputSize::Middle,
        InputStatus::Default,
        true,
    ),
    (
        "error",
        InputVariant::Outlined,
        InputSize::Middle,
        InputStatus::Error,
        false,
    ),
];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 40.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 352.0;

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
    inputs: Vec<Entity<NumberInput>>,
    init: bool,
}

impl Panel {
    fn new(cx: &mut Context<Self>) -> Self {
        let inputs = ROWS
            .iter()
            .map(|(_, variant, size, status, disabled)| {
                let (variant, size, status, disabled) = (*variant, *size, *status, *disabled);
                cx.new(move |cx| {
                    NumberInput::new(cx)
                        .placeholder(SharedString::default())
                        .variant(variant)
                        .size(size)
                        .status(status)
                        .disabled(disabled)
                        .controls_visible(true)
                })
            })
            .collect();
        Self {
            inputs,
            init: false,
        }
    }
}

impl Render for Panel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.init {
            self.init = true;
            for input in &self.inputs {
                input.update(cx, |input, cx| input.set_value(Some(3.), window, cx));
            }
        }
        let mut panel = div()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .p(px(PAD))
            .gap(px(ROW_GAP))
            .bg(theme::bg_container());
        for ((label, _, _, _, _), input) in ROWS.iter().zip(self.inputs.iter()) {
            panel = panel.child(row(label, input.clone()));
        }
        panel
    }
}

fn row(label: &'static str, input: Entity<NumberInput>) -> impl IntoElement {
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
        .child(div().w(px(CELL_WIDTH)).child(input))
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
                    title: Some("number_input_panel_shot".into()),
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
