//! 截图基线用面板：默认插画 / simple（--part 2 时 footer / 无描述），
//! 与 `harness/antd_input_reference.html?kind=empty` / `kind=empty2` 的同参数 antd Empty 对齐。
//! 面板限两行：整屏截图高度放不下四行 180px。
//!
//! ```text
//! cargo run --offline --example empty_panel_shot -- --appearance light --part 1
//! ```

use std::env;

use gpui::{
    AnyElement, App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Button, Empty, Theme, theme};

actions!(empty_panel_shot, [Quit]);

const ROWS: [&str; 2] = ["default", "simple"];
const ROWS2: [&str; 2] = ["footer", "nodesc"];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 180.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 408.0;

struct Args {
    dark: bool,
    accent: Option<u32>,
    part: usize,
}

fn parse_args() -> Args {
    let argv: Vec<String> = env::args().skip(1).collect();
    let mut args = Args {
        dark: false,
        accent: None,
        part: 1,
    };
    let mut i = 0;
    while i < argv.len() {
        let value = argv.get(i + 1).map(String::as_str).unwrap_or_default();
        match argv[i].as_str() {
            "--appearance" => args.dark = value == "dark",
            "--part" => args.part = value.parse().expect("--part 需要 1 或 2"),
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

fn sample(index: usize) -> Option<Empty> {
    Some(match index {
        0 => Empty::new("No data"),
        1 => Empty::new("No data").simple(),
        2 => Empty::new("No data").action(Button::new("Create Now").primary()),
        3 => Empty::new(""),
        _ => return None,
    })
}

struct Panel {
    part: usize,
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut panel = div()
            .relative()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(PANEL_HEIGHT))
            .p(px(PAD))
            .gap(px(ROW_GAP))
            .bg(theme::bg_container());

        let rows = if self.part == 2 { ROWS2 } else { ROWS };
        let offset = (self.part - 1) * 2;
        for (ix, label) in rows.into_iter().enumerate() {
            let content: AnyElement = match sample(ix + offset) {
                Some(empty) => div().w(px(CELL_WIDTH)).child(empty).into_any_element(),
                None => div().w(px(CELL_WIDTH)).h(px(0.)).into_any_element(),
            };
            panel = panel.child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
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
                    .child(content),
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
                    title: Some("empty_panel_shot".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Panel { part: args.part }),
        )
        .unwrap();
        cx.activate(true);
    });
}
