//! 截图基线用面板：全尺寸日历月视图（--part 2 时年视图），
//! 与 `harness/antd_input_reference.html?kind=calendar` / `kind=calendar2` 对齐。
//!
//! ```text
//! cargo run --offline --example calendar_panel_shot -- --appearance light --part 1
//! ```

use std::env;

use gpui::{
    App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Calendar, CalendarMode, Theme, date_picker::Date, theme};

actions!(calendar_panel_shot, [Quit]);

const PAD: f32 = 16.0;
const CELL_WIDTH: f32 = 760.0;
const PANEL_WIDTH: f32 = 792.0;

// 固定在过去（2025-11-10 周一）：当天标记不随系统日期漂移，基线可长期复现。与
// `harness/antd_input_reference.html` kind=calendar / calendar2 同参数。
const VALUE: Date = (2025, 11, 10);
const MONTH: (i32, u32) = (2025, 11);

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
    value: Option<Date>,
    month: (i32, u32),
    mode: CalendarMode,
}

impl Panel {
    fn new(part: usize) -> Self {
        Self {
            part,
            value: Some(VALUE),
            month: MONTH,
            mode: if part == 2 {
                CalendarMode::Year
            } else {
                CalendarMode::Month
            },
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let row_height = if self.part == 2 { 536.0 } else { 790.0 };
        let panel_height = row_height + PAD * 2.0;
        let calendar = Calendar::new("cal", self.month)
            .value(self.value)
            .mode(self.mode)
            .on_select(cx.listener(|this, date, _, cx| {
                this.value = Some(*date);
                cx.notify();
            }))
            .on_month_change(cx.listener(|this, month, _, cx| {
                this.month = *month;
                cx.notify();
            }))
            .on_mode_change(cx.listener(|this, mode, _, cx| {
                this.mode = *mode;
                cx.notify();
            }));
        div()
            .relative()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(panel_height))
            .p(px(PAD))
            .bg(theme::bg_container())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .h(px(row_height))
                    .child(div().w(px(CELL_WIDTH)).child(calendar)),
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

        let panel_height = if args.part == 2 { 568.0 } else { 822.0 };
        let bounds = Bounds::centered(None, size(px(PANEL_WIDTH), px(panel_height)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("calendar_panel_shot".into()),
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
