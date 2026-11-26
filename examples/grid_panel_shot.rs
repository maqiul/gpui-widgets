//! 截图基线用面板：与 `harness/antd_input_reference.html` kind=grid 同构。
//!
//! ```text
//! cargo run --offline --example grid_panel_shot -- --appearance light
//! ```
//!
//! 面板几何 420×184（pad 16、行带 56、label 140、gap 8、cell 240）；
//! gutter 16 按 antd 语义拆半：Row 负外边距 -8、Col 内边距 8。

use std::env;

use gpui::{
    App, Bounds, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Col, Row, Theme, theme};

actions!(grid_panel_shot, [Quit]);

const ROWS: [&str; 3] = ["gutter", "thirds", "offset"];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 40.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 184.0;

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

fn grid_cell(text: &'static str) -> impl IntoElement {
    div()
        .h(px(40.))
        .flex()
        .items_center()
        .justify_center()
        .bg(theme::accent())
        .text_color(gpui::white())
        .text_size(px(14.))
        .child(text)
}

struct Panel;

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let mut panel = div()
            .relative()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(PANEL_HEIGHT))
            .p(px(PAD))
            .gap(px(ROW_GAP))
            .bg(theme::bg_container());
        for (ix, label) in ROWS.into_iter().enumerate() {
            let row = match ix {
                0 => Row::new()
                    .gutter(16.)
                    .child(Col::new(12).child(grid_cell("12")))
                    .child(Col::new(12).child(grid_cell("12"))),
                1 => Row::new()
                    .gutter(16.)
                    .child(Col::new(8).child(grid_cell("8")))
                    .child(Col::new(8).child(grid_cell("8")))
                    .child(Col::new(8).child(grid_cell("8"))),
                _ => Row::new().child(Col::new(12).offset(6).child(grid_cell("12 offset 6"))),
            };
            panel = panel.child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .flex_shrink_0()
                    .h(px(ROW_HEIGHT))
                    .gap(px(CELL_GAP))
                    .child(
                        div()
                            .w(px(LABEL_WIDTH))
                            .flex_shrink_0()
                            .text_size(px(12.))
                            .text_color(theme::text_tertiary())
                            .child(label),
                    )
                    .child(div().w(px(CELL_WIDTH)).child(row)),
            );
        }
        panel
    }
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
                    title: Some("grid_panel_shot".into()),
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
