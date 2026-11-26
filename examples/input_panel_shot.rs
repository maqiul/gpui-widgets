//! 截图基线用面板：渲染与 `harness/antd_input_reference.html` 的 `__panel` 完全同构的 Input 面板，
//! 供 `harness/capture_rust.ps1` 抓客户端区域，再由 `harness/shot_diff.py` 逐行比对。
//!
//! ```text
//! cargo run --offline --example input_panel_shot -- --variant outlined --appearance light
//! cargo run --offline --example input_panel_shot -- --variant filled --appearance dark --accent 8b5cf6
//! ```
//!
//! 面板几何（420×688、pad 16、行带 56、label 140、行内 gap 8、控件宽 240）是两侧共同约定，
//! 改一处必须同步改 `antd_input_reference.html` 与 `shot_diff.py`。

use std::env;

use gpui::{
    App, Bounds, Context, Entity, IntoElement, KeyBinding, ParentElement, Render, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::text_input::{InputSize, InputStatus, InputVariant};
use gpui_widgets::{TextInput, Theme, theme};

actions!(input_panel_shot, [Quit]);

/// 与参照页 PANEL_ROWS 一一对应：标签、状态、尺寸、是否禁用
const ROWS: [(&str, InputStatus, InputSize, bool); 12] = [
    ("default-S", InputStatus::Default, InputSize::Small, false),
    ("default-M", InputStatus::Default, InputSize::Middle, false),
    ("default-L", InputStatus::Default, InputSize::Large, false),
    ("default-off", InputStatus::Default, InputSize::Middle, true),
    ("error-S", InputStatus::Error, InputSize::Small, false),
    ("error-M", InputStatus::Error, InputSize::Middle, false),
    ("error-L", InputStatus::Error, InputSize::Large, false),
    ("error-off", InputStatus::Error, InputSize::Middle, true),
    ("warning-S", InputStatus::Warning, InputSize::Small, false),
    ("warning-M", InputStatus::Warning, InputSize::Middle, false),
    ("warning-L", InputStatus::Warning, InputSize::Large, false),
    ("warning-off", InputStatus::Warning, InputSize::Middle, true),
];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 40.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 688.0;

struct Args {
    variant: InputVariant,
    dark: bool,
    accent: Option<u32>,
}

fn parse_args() -> Args {
    let argv: Vec<String> = env::args().skip(1).collect();
    let mut args = Args {
        variant: InputVariant::Outlined,
        dark: false,
        accent: None,
    };
    let mut i = 0;
    while i < argv.len() {
        let value = argv.get(i + 1).map(String::as_str).unwrap_or_default();
        match argv[i].as_str() {
            "--variant" => args.variant = variant_of(value),
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

fn variant_of(name: &str) -> InputVariant {
    match name {
        "outlined" => InputVariant::Outlined,
        "filled" => InputVariant::Filled,
        "borderless" => InputVariant::Borderless,
        "underlined" => InputVariant::Underlined,
        other => panic!("未知变体 {other}"),
    }
}

struct Panel(Vec<Entity<TextInput>>);

impl Panel {
    fn new(variant: InputVariant, cx: &mut Context<Self>) -> Self {
        let inputs = ROWS
            .iter()
            .map(|(_, status, size, disabled)| {
                let (status, size, disabled) = (*status, *size, *disabled);
                cx.new(move |cx| {
                    TextInput::new(cx)
                        .placeholder(SharedString::default())
                        .variant(variant)
                        .status(status)
                        .size(size)
                        .disabled(disabled)
                })
            })
            .collect();
        Self(inputs)
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
        for ((label, _, _, _), input) in ROWS.iter().zip(self.0.iter()) {
            panel = panel.child(row(label, input.clone()));
        }
        panel
    }
}

fn row(label: &'static str, input: Entity<TextInput>) -> impl IntoElement {
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
    let variant = args.variant;
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
                    title: Some(format!("input_panel_shot {variant:?}").into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Panel::new(variant, cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
