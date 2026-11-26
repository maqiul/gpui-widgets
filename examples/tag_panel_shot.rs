//! 截图基线用面板：12 行 Tag（outlined / filled / solid × 颜色 / 禁用），
//! 与 `harness/antd_input_reference.html?panel=outlined&kind=tag` 同构。
//!
//! ```text
//! cargo run --offline --example tag_panel_shot -- --appearance light
//! ```

use std::env;

use gpui::{
    App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::tag::{Tag, TagColor, TagVariant};
use gpui_widgets::{Theme, theme};

actions!(tag_panel_shot, [Quit]);

const ROWS: [(&str, TagColor, TagVariant, bool); 12] = [
    (
        "out-default",
        TagColor::Default,
        TagVariant::Outlined,
        false,
    ),
    ("out-blue", TagColor::Blue, TagVariant::Outlined, false),
    (
        "out-success",
        TagColor::Success,
        TagVariant::Outlined,
        false,
    ),
    ("out-error", TagColor::Error, TagVariant::Outlined, false),
    (
        "out-warning",
        TagColor::Warning,
        TagVariant::Outlined,
        false,
    ),
    ("fill-default", TagColor::Default, TagVariant::Filled, false),
    ("fill-blue", TagColor::Blue, TagVariant::Filled, false),
    ("fill-success", TagColor::Success, TagVariant::Filled, false),
    ("solid-default", TagColor::Default, TagVariant::Solid, false),
    ("solid-blue", TagColor::Blue, TagVariant::Solid, false),
    (
        "off-outlined",
        TagColor::Default,
        TagVariant::Outlined,
        true,
    ),
    ("off-solid", TagColor::Blue, TagVariant::Solid, true),
];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 120.0;
const ROW_HEIGHT: f32 = 40.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 688.0;

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    let mut dark = false;
    let mut accent = None;
    let mut i = 0;
    while i < argv.len() {
        let value = argv.get(i + 1).map(String::as_str).unwrap_or_default();
        match argv[i].as_str() {
            "--appearance" => dark = value == "dark",
            "--accent" => {
                accent = Some(
                    u32::from_str_radix(value.trim_start_matches('#'), 16)
                        .expect("--accent 需要 6 位十六进制"),
                )
            }
            other => panic!("未知参数 {other}"),
        }
        i += 2;
    }

    gpui_platform::application().run(move |cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());

        let mut theme = if dark { Theme::dark() } else { Theme::light() };
        if let Some(accent) = accent {
            theme = theme.with_accent(rgb(accent));
        }
        theme::set_theme(theme);

        let bounds = Bounds::centered(None, size(px(PANEL_WIDTH), px(PANEL_HEIGHT)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("tag_panel_shot".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Panel),
        )
        .unwrap();
        cx.activate(true);
    });
}

struct Panel;

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut panel = div()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .p(px(PAD))
            .gap(px(ROW_GAP))
            .bg(theme::bg_container());
        for (label, color, variant, disabled) in ROWS {
            panel = panel.child(row(label, color, variant, disabled));
        }
        panel
    }
}

fn row(
    label: &'static str,
    color: TagColor,
    variant: TagVariant,
    disabled: bool,
) -> impl IntoElement {
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
            div().w(px(CELL_WIDTH)).child(
                Tag::new("Tag")
                    .color(color)
                    .variant(variant)
                    .disabled(disabled),
            ),
        )
}
