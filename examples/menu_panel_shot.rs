//! 截图基线用面板：用零高度触发器把 DropdownMenu 浮层钉在固定坐标，
//! 与 `harness/antd_input_reference.html?kind=menu` 的 antd Dropdown 覆盖同一行带。
//!
//! ```text
//! cargo run --offline --example menu_panel_shot -- --appearance light
//! ```

use std::env;

use gpui::{
    App, Bounds, Context, ElementId, IntoElement, KeyBinding, ParentElement, Render, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::menu::{DropdownMenu, MenuItem};
use gpui_widgets::{Theme, theme};

actions!(menu_panel_shot, [Quit]);

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
/// 触发器的绝对 top（与参照页同值）：浮层顶边 = 该值 + 4px 偏移
const MENU_TOP: f32 = PAD + 36.0;

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

fn items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("编辑"),
        MenuItem::new("复制"),
        MenuItem::separator(),
        MenuItem::new("删除").danger(),
        MenuItem::new("禁用项").disabled(true),
    ]
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
                .top(px(MENU_TOP))
                .w(px(CELL_WIDTH))
                .child(
                    DropdownMenu::new(
                        ElementId::Name(SharedString::from("menu-shot")),
                        div().w(px(CELL_WIDTH)).h(px(0.)),
                        items(),
                    )
                    .width(px(CELL_WIDTH))
                    .open(true),
                ),
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
                    title: Some("menu_panel_shot".into()),
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
