//! 截图基线用面板：四行遮罩盒（首步 / 中间 / 末步 / 单步），
//! 与 `harness/antd_input_reference.html?kind=tour` 的同参数 antd Tour 对齐。
//! 面板宽 560：antd Tour 卡片 width 520 + maxWidth fit-content（随内容收缩），居中于遮罩盒。
//!
//! ```text
//! cargo run --offline --example tour_panel_shot -- --appearance light
//! ```

use std::env;

use gpui::{
    App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Theme, Tour, TourStep, theme};

actions!(tour_panel_shot, [Quit]);

const ROWS: [&str; 4] = ["first", "middle", "last", "single"];

const PAD: f32 = 16.0;
const ROW_HEIGHT: f32 = 180.0;
const ROW_GAP: f32 = 16.0;
const PANEL_WIDTH: f32 = 560.0;
const PANEL_HEIGHT: f32 = 800.0;

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

fn steps(count: usize) -> Vec<TourStep> {
    [
        ("Step 1", "Upload your dataset to get started."),
        ("Step 2", "Pick a model and tune the parameters."),
        ("Step 3", "Review the report and share it."),
    ]
    .iter()
    .take(count)
    .map(|(t, d)| TourStep::new(*t, *d))
    .collect()
}

struct Panel {
    current: Vec<usize>,
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut panel = div()
            .relative()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(PANEL_HEIGHT))
            .p(px(PAD))
            .gap(px(ROW_GAP))
            .bg(theme::bg_container());

        for ix in 0..ROWS.len() {
            let count = if ix == 3 { 1 } else { 3 };
            let tour = Tour::new(steps(count))
                .current(self.current[ix])
                .on_change(cx.listener(move |this, step, _, cx| {
                    this.current[ix] = *step;
                    cx.notify();
                }))
                .on_close({
                    let entity = cx.entity();
                    move |_, cx| {
                        let _ = entity.update(cx, |this, cx| {
                            this.current[ix] = 0;
                            cx.notify();
                        });
                    }
                });
            panel = panel.child(
                div()
                    .relative()
                    .w(px(PANEL_WIDTH - PAD * 2.))
                    .h(px(ROW_HEIGHT))
                    .overflow_hidden()
                    .child(tour),
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
                    title: Some("tour_panel_shot".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| Panel {
                    current: vec![0, 1, 2, 0],
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
