//! 截图基线用面板：Notification 卡片行内排布，
//! 与 `harness/antd_input_reference.html?kind=notification` 的 antd PurePanel 同一行带。
//!
//! ```text
//! cargo run --offline --example notification_panel_shot -- --appearance light
//! ```

use std::env;

use gpui::{
    App, Bounds, Context, KeyBinding, ParentElement, Render, TitlebarOptions, Window, WindowBounds,
    WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{
    Theme,
    notification::{NotificationItem, notification_card},
    theme,
};

actions!(notification_panel_shot, [Quit]);

const PAD: f32 = 16.0;
const ROW_HEIGHT: f32 = 160.0;
const ROW_GAP: f32 = 16.0;
const PANEL_WIDTH: f32 = 416.0;
const PANEL_HEIGHT: f32 = 560.0;

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
        let items = [
            NotificationItem::new("Notification Title")
                .description("This is the content of the notification.")
                .action("Confirm", |_, _| {}),
            NotificationItem::new("Success").success(),
            NotificationItem::new("Warning Title")
                .warning()
                .description("This is the content of the notification."),
        ];

        let mut panel = div()
            .relative()
            .flex()
            .flex_col()
            .w(px(PANEL_WIDTH))
            .h(px(PANEL_HEIGHT))
            .p(px(PAD))
            .gap(px(ROW_GAP))
            .bg(theme::bg_container());

        for (ix, item) in items.iter().enumerate() {
            let on_close = if ix == 2 {
                None
            } else {
                Some(
                    std::rc::Rc::new(|_: &usize, _: &mut Window, _: &mut App| {})
                        as std::rc::Rc<dyn Fn(&usize, &mut Window, &mut App)>,
                )
            };
            panel = panel.child(
                div()
                    .h(px(ROW_HEIGHT))
                    .child(notification_card(item, ix, on_close)),
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
                    title: Some("notification_panel_shot".into()),
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
