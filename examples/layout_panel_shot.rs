//! 截图基线用面板：与 `harness/antd_input_reference.html` kind=layout 同构。
//!
//! ```text
//! cargo run --offline --example layout_panel_shot -- --appearance light
//! ```
//!
//! 面板几何 420×448（pad 16、行带 216、label 140、gap 8、cell 240）；
//! 行高 200 = header 64 + 中段 66 + footer 70。

use std::env;

use gpui::{
    App, Bounds, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Content, Footer, Header, Layout, Sider, Theme, theme};

actions!(layout_panel_shot, [Quit]);

const ROWS: [&str; 2] = ["dark", "light"];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 200.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 448.0;

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
            let mut sider = Sider::new(80.).child("Sider");
            if ix == 1 {
                sider = sider.light();
            }
            let layout = Layout::new()
                .child(Header::new().child("Header"))
                .child(
                    Layout::new()
                        .has_sider()
                        .child(sider)
                        .child(Content::new().child("Content")),
                )
                .child(Footer::new().child("Footer"));
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
                        // 所在格是定高纵向 flex 容器，Layout 的 flex_1 才会撑满
                        // （与 antd 参照页里 layout 撑满 200px 行的效果一致）
                        div()
                            .w(px(CELL_WIDTH))
                            .h(px(ROW_HEIGHT))
                            .flex()
                            .flex_col()
                            .child(layout),
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
                    title: Some("layout_panel_shot".into()),
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
