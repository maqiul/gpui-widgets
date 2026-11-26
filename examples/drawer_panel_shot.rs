//! 截图基线用面板：默认 / 无底部 / 无关闭按钮（全右侧），
//! 与 `harness/antd_input_reference.html?kind=drawer` 对齐（行高 200、无标签列）。

use std::env;

use gpui::{
    AnyElement, App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Button, Drawer, Theme, theme, theme::metric::DrawerMetric};

actions!(drawer_panel_shot, [Quit]);

const ROWS: [&str; 3] = ["default", "nofooter", "noclose"];

const PAD: f32 = 16.0;
const CELL_WIDTH: f32 = 388.0;
const ROW_HEIGHT: f32 = 200.0;
const ROW_GAP: f32 = 16.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 664.0;

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

fn sample(index: usize) -> Option<AnyElement> {
    let m = theme::with(DrawerMetric::resolve);
    let mut drawer = Drawer::new("Basic Drawer").child(div().child("Some contents..."));
    if index != 1 {
        drawer = drawer.footer(
            div()
                .flex()
                .flex_row()
                .gap(px(m.footer_gap))
                .child(Button::new("Cancel"))
                .child(Button::new("OK").primary()),
        );
    }
    if index == 2 {
        drawer = drawer.closable(false);
    }
    Some(
        div()
            .relative()
            .w(px(CELL_WIDTH))
            .h(px(ROW_HEIGHT))
            .overflow_hidden()
            .child(drawer)
            .into_any_element(),
    )
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

        for (ix, _label) in ROWS.into_iter().enumerate() {
            let content: AnyElement =
                sample(ix).unwrap_or_else(|| div().w(px(CELL_WIDTH)).h(px(0.)).into_any_element());
            panel = panel.child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .flex_shrink_0()
                    .h(px(ROW_HEIGHT))
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
                    title: Some("drawer_panel_shot".into()),
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
