//! 截图基线用面板：与 `harness/antd_input_reference.html` kind=navmenu 同构。
//!
//! ```text
//! cargo run --offline --example navmenu_panel_shot -- --appearance light
//! ```
//!
//! 面板几何 420×496（pad 16、行带 160、label 140、gap 8、cell 240）；
//! 行高 144 = 3 条目 ×(itemHeight 40 + marginBlock 8)。

use std::env;

use gpui::{
    App, Bounds, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{MenuEntry, NavMenu, Theme, theme};

actions!(navmenu_panel_shot, [Quit]);

const ROWS: [&str; 3] = ["selected", "submenu", "states"];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 144.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 496.0;

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

fn menu_for(ix: usize) -> NavMenu {
    match ix {
        0 => NavMenu::new(
            "nav0",
            vec![
                MenuEntry::new("home", "首页"),
                MenuEntry::new("docs", "文档"),
                MenuEntry::new("about", "关于"),
            ],
        )
        .selected(Some("home".into())),
        1 => NavMenu::new(
            "nav1",
            vec![MenuEntry::new("settings", "设置").children([
                MenuEntry::new("profile", "个人资料"),
                MenuEntry::new("security", "安全"),
            ])],
        )
        .selected(Some("profile".into()))
        .expanded(vec!["settings".into()]),
        _ => NavMenu::new(
            "nav2",
            vec![
                MenuEntry::new("icon", "带图标").icon("⌂"),
                MenuEntry::new("off", "禁用").disabled(true),
                MenuEntry::new("plain", "普通"),
            ],
        )
        .selected(Some("plain".into())),
    }
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
                    .child(
                        div()
                            .w(px(CELL_WIDTH))
                            .h(px(ROW_HEIGHT))
                            .child(menu_for(ix)),
                    ),
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
                    title: Some("navmenu_panel_shot".into()),
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
