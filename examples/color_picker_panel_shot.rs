//! 截图基线用面板：触发框三档 + 禁用（--part 2 时展开预设浮层两行），
//! 与 `harness/antd_input_reference.html?kind=colorpicker` / `kind=colorpicker2` 对齐。
//!
//! ```text
//! cargo run --offline --example color_picker_panel_shot -- --appearance light --part 1
//! ```

use std::env;

use gpui::{
    AnyElement, App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render, Rgba,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{ColorPicker, Theme, theme};

actions!(color_picker_panel_shot, [Quit]);

const ROWS: [&str; 4] = ["default", "small", "large", "disabled"];
const ROWS2: [&str; 2] = ["open", "open-small"];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;

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

struct Panel {
    part: usize,
    colors: Vec<Rgba>,
    open: Vec<bool>,
}

impl Panel {
    fn new(part: usize) -> Self {
        if part == 2 {
            Self {
                part,
                colors: vec![rgb(0xef4444); 2],
                open: vec![true; 2],
            }
        } else {
            Self {
                part,
                colors: vec![rgb(0x1677ff); 4],
                open: vec![false; 4],
            }
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let part2 = self.part == 2;
        let row_height = if part2 { 180.0 } else { 40.0 };
        let panel_height = if part2 { 408.0 } else { 240.0 };
        let mut panel = div()
            .relative()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(panel_height))
            .p(px(PAD))
            .gap(px(ROW_GAP))
            .bg(theme::bg_container());

        let rows: Vec<&str> = if part2 { ROWS2.to_vec() } else { ROWS.to_vec() };
        for (ix, label) in rows.into_iter().enumerate() {
            let mut picker = ColorPicker::new(
                gpui::SharedString::from(format!("cp-{ix}")),
                self.colors[ix],
            )
            .open(self.open[ix])
            .on_change(cx.listener(move |this, color, _, cx| {
                this.colors[ix] = *color;
                cx.notify();
            }))
            .on_open_change(cx.listener(move |this, open, _, cx| {
                this.open[ix] = *open;
                cx.notify();
            }));
            if part2 {
                if ix == 1 {
                    picker = picker.small();
                }
            } else {
                match ix {
                    1 => picker = picker.small(),
                    2 => picker = picker.large(),
                    3 => picker = picker.disabled(true),
                    _ => {}
                }
            }
            let content: AnyElement = div().w(px(CELL_WIDTH)).child(picker).into_any_element();
            panel = panel.child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .h(px(row_height))
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

        let panel_height = if args.part == 2 { 408.0 } else { 240.0 };
        let bounds = Bounds::centered(None, size(px(PANEL_WIDTH), px(panel_height)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("color_picker_panel_shot".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Panel::new(args.part)),
        )
        .unwrap();
        cx.activate(true);
    });
}
