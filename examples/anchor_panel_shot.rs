//! 截图基线用面板：与 `harness/antd_input_reference.html` kind=anchor 同构。
//!
//! ```text
//! cargo run --offline --example anchor_panel_shot -- --appearance light
//! ```
//!
//! 面板几何 420×680（pad 16、行带 166、行高 150、label 140、gap 8、cell 240）。
//! 连续轨 colorSplit + 激活 ink colorPrimary；四行分别激活顶层 / 子级 / 无 / 末叶。

use std::env;

use gpui::{
    App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Anchor, AnchorItem, Theme, theme};

actions!(anchor_panel_shot, [Quit]);

const ROWS: [(&str, Option<&str>); 4] = [
    ("active", Some("a")),
    ("child", Some("b1")),
    ("none", None),
    ("leaf", Some("c")),
];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 150.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 680.0;

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

fn items() -> Vec<AnchorItem> {
    vec![
        AnchorItem::new("a", "Basic"),
        AnchorItem::new("b", "API")
            .child(AnchorItem::new("b1", "Props"))
            .child(AnchorItem::new("b2", "Events")),
        AnchorItem::new("c", "FAQ"),
    ]
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
        for (i, (label, active)) in ROWS.iter().enumerate() {
            let mut anchor = Anchor::new(items()).id(format!("row{i}"));
            if let Some(key) = active {
                anchor = anchor.active(*key);
            }
            panel = panel.child(row(label, anchor));
        }
        panel
    }
}

fn row(label: &'static str, anchor: Anchor) -> impl IntoElement {
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
        .child(div().w(px(CELL_WIDTH)).child(anchor))
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
                    title: Some("anchor_panel_shot".into()),
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
