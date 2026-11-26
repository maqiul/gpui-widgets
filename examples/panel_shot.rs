//! 截图基线用面板：渲染与 `harness/antd_reference.html` 的 `__panel` 完全同构的 Button 面板，
//! 供 `harness/capture_rust.ps1` 抓客户端区域，再由 `harness/shot_diff.py` 逐行比对。
//!
//! ```text
//! cargo run --offline --example panel_shot -- --variant solid --appearance light
//! cargo run --offline --example panel_shot -- --variant filled --appearance dark --accent 8b5cf6
//! ```
//!
//! 面板几何（420×688、pad 16、行带 56、label 140、行内 gap 8）是两侧共同约定，
//! 改一处必须同步改 `antd_reference.html` 与 `shot_diff.py`。

use std::env;

use gpui::{
    AnyElement, App, Bounds, Context, IntoElement, KeyBinding, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::button::{ButtonColor, ButtonSize, ButtonVariant};
use gpui_widgets::{Button, Theme, theme};

actions!(panel_shot, [Quit]);

/// 与参照页 PANEL_ROWS 一一对应：标签、色彩、尺寸、按钮文字、是否禁用
const ROWS: [(&str, ButtonColor, ButtonSize, &str, bool); 12] = [
    (
        "default-S",
        ButtonColor::Default,
        ButtonSize::Small,
        "S",
        false,
    ),
    (
        "default-M",
        ButtonColor::Default,
        ButtonSize::Medium,
        "M",
        false,
    ),
    (
        "default-L",
        ButtonColor::Default,
        ButtonSize::Large,
        "L",
        false,
    ),
    (
        "default-off",
        ButtonColor::Default,
        ButtonSize::Medium,
        "off",
        true,
    ),
    (
        "primary-S",
        ButtonColor::Primary,
        ButtonSize::Small,
        "S",
        false,
    ),
    (
        "primary-M",
        ButtonColor::Primary,
        ButtonSize::Medium,
        "M",
        false,
    ),
    (
        "primary-L",
        ButtonColor::Primary,
        ButtonSize::Large,
        "L",
        false,
    ),
    (
        "primary-off",
        ButtonColor::Primary,
        ButtonSize::Medium,
        "off",
        true,
    ),
    (
        "danger-S",
        ButtonColor::Danger,
        ButtonSize::Small,
        "S",
        false,
    ),
    (
        "danger-M",
        ButtonColor::Danger,
        ButtonSize::Medium,
        "M",
        false,
    ),
    (
        "danger-L",
        ButtonColor::Danger,
        ButtonSize::Large,
        "L",
        false,
    ),
    (
        "danger-off",
        ButtonColor::Danger,
        ButtonSize::Medium,
        "off",
        true,
    ),
];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const ROW_HEIGHT: f32 = 40.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 688.0;

struct Args {
    variant: ButtonVariant,
    dark: bool,
    accent: Option<u32>,
}

fn parse_args() -> Args {
    let argv: Vec<String> = env::args().skip(1).collect();
    let mut args = Args {
        variant: ButtonVariant::Solid,
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

fn variant_of(name: &str) -> ButtonVariant {
    match name {
        "solid" => ButtonVariant::Solid,
        "outlined" => ButtonVariant::Outlined,
        "dashed" => ButtonVariant::Dashed,
        "filled" => ButtonVariant::Filled,
        "text" => ButtonVariant::Text,
        "link" => ButtonVariant::Link,
        other => panic!("未知变体 {other}"),
    }
}

struct Panel(ButtonVariant);

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut panel = div()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .p(px(PAD))
            .gap(px(ROW_GAP))
            .bg(theme::bg_container());
        for (label, color, size, text, disabled) in ROWS {
            panel = panel.child(row(label, color, size, text, disabled, self.0));
        }
        panel
    }
}

fn row(
    label: &'static str,
    color: ButtonColor,
    size: ButtonSize,
    text: &'static str,
    disabled: bool,
    variant: ButtonVariant,
) -> AnyElement {
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
        .child(
            Button::new(text)
                .color(color)
                .variant(variant)
                .size(size)
                .disabled(disabled),
        )
        .into_any_element()
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
                    title: Some(format!("panel_shot {variant:?}").into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Panel(variant)),
        )
        .unwrap();
        cx.activate(true);
    });
}
