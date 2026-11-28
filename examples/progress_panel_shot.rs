//! 截图基线用面板：水平（正常 / 小尺寸 / 无指示器 / 成功 / 异常 / 进行中）
//! 与分段进度，与 `harness/antd_input_reference.html?kind=progress` 对齐。
//!
//! ```text
//! cargo run --offline --example progress_panel_shot -- --appearance light
//! ```

use std::env;

use gpui::{
    AnyElement, App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::progress::{ProgressBar, ProgressStatus};
use gpui_widgets::{Theme, theme};

actions!(progress_panel_shot, [Quit]);

const ROWS: [&str; 12] = [
    "line-30",
    "line-30-small",
    "line-30-noinfo",
    "line-success",
    "line-exception",
    "line-active",
    "steps-5",
    "steps-5-small",
    "",
    "",
    "",
    "",
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

fn sample(index: usize) -> Option<ProgressBar> {
    Some(match index {
        0 => ProgressBar::new(0.3),
        1 => ProgressBar::new(0.3).small(true),
        2 => ProgressBar::new(0.3).show_info(false),
        3 => ProgressBar::new(1.0).status(ProgressStatus::Success),
        4 => ProgressBar::new(0.5).status(ProgressStatus::Exception),
        5 => ProgressBar::new(0.5)
            .status(ProgressStatus::Active)
            .animated(false),
        6 => ProgressBar::new(0.3).steps(5),
        7 => ProgressBar::new(0.3).steps(5).small(true),
        _ => return None,
    })
}

struct Panel;

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

        for (ix, label) in ROWS.into_iter().enumerate() {
            let content: AnyElement = match sample(ix) {
                Some(progress) => progress.into_any_element(),
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
                    .child(div().w(px(CELL_WIDTH)).flex().flex_col().child(content)),
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
                    title: Some("progress_panel_shot".into()),
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
