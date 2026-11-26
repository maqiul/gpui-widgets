//! 截图基线用面板：把 Tooltip 气泡放在固定坐标，
//! 与 `harness/antd_input_reference.html?kind=tooltip` 打开状态的 antd Tooltip 覆盖同一行带。
//!
//! Tooltip 本身由 GPUI 原生悬停机制定位，无法在静态截图里控制位置，
//! 因此这里直接渲染同一个 `tooltip_bubble` 表面（与原生 tooltip 共用渲染函数）。
//!
//! ```text
//! cargo run --offline --example tooltip_panel_shot -- --appearance light
//! ```

use std::env;

use gpui::{
    App, Bounds, Context, KeyBinding, ParentElement, Render, TitlebarOptions, Window, WindowBounds,
    WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Theme, theme, tooltip::tooltip_bubble};

actions!(tooltip_panel_shot, [Quit]);

const ROWS: [&str; 12] = [
    "default-S",
    "default-M",
    "default-L",
    "default-off",
    "error-S",
    "error-M",
    "error-L",
    "error-off",
    "warning-S",
    "warning-M",
    "warning-L",
    "warning-off",
];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 40.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 688.0;
/// 与参照页 antd Tooltip 容器同一位置（底部对齐触发框 + 12px 箭头偏移）
const BUBBLE_TOP: f32 = PAD + 48.0;

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

        for label in ROWS {
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
                            .text_size(px(12.))
                            .text_color(theme::text_tertiary())
                            .child(label),
                    )
                    .child(div().w(px(CELL_WIDTH)).h(px(0.))),
            );
        }

        panel = panel.child(
            div()
                .absolute()
                .left(px(PAD + LABEL_WIDTH + CELL_GAP))
                .top(px(BUBBLE_TOP))
                .child(tooltip_bubble("工具提示文字")),
        );

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
                    title: Some("tooltip_panel_shot".into()),
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
