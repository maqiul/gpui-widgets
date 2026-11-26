//! 截图基线用面板：与 `harness/antd_input_reference.html` kind=scrollbar 同构。
//!
//! ```text
//! cargo run --offline --example scrollbar_panel_shot -- --appearance light
//! ```
//!
//! 面板几何 420×152（pad 16、单行带 136、label 140、gap 8、cell 240）；行高 120。
//! 滚动区 232×120 + 纵向滚动条槽 8；内容 30 行 ×54（antd Table 默认行高）。

use std::env;

use gpui::{
    App, Bounds, IntoElement, KeyBinding, ParentElement, Render, ScrollHandle, TitlebarOptions,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Scrollbar, Theme, theme};

actions!(scrollbar_panel_shot, [Quit]);

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 120.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 152.0;
const CONTENT_ROW_HEIGHT: f32 = 54.0;
const CONTENT_ROWS: usize = 30;

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

struct Panel {
    handle: ScrollHandle,
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let mut content = div().flex().flex_col().w(px(CELL_WIDTH - 8.));
        for i in 0..CONTENT_ROWS {
            content = content.child(
                div()
                    .h(px(CONTENT_ROW_HEIGHT))
                    .flex_shrink_0()
                    .px(px(16.))
                    .flex()
                    .items_center()
                    .text_size(px(14.))
                    .text_color(theme::text())
                    .child(format!("row {i}")),
            );
        }
        let area = div()
            .id("scrollbar-shot-area")
            .w(px(CELL_WIDTH - 8.))
            .h(px(ROW_HEIGHT))
            .overflow_y_scroll()
            .track_scroll(&self.handle)
            .child(content);
        div()
            .relative()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(PANEL_HEIGHT))
            .p(px(PAD))
            .gap(px(ROW_GAP))
            .bg(theme::bg_container())
            .child(
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
                            .child("vertical"),
                    )
                    .child(
                        div()
                            .w(px(CELL_WIDTH))
                            .h(px(ROW_HEIGHT))
                            .flex()
                            .flex_row()
                            .child(area)
                            .child(Scrollbar::vertical(
                                "scrollbar-shot-sb",
                                self.handle.clone(),
                            )),
                    ),
            )
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
                    title: Some("scrollbar_panel_shot".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| Panel {
                    handle: ScrollHandle::new(),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
