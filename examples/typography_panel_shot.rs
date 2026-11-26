//! 截图基线用面板：标题 1/3/5、默认/次要/错误文本、代码与高亮，
//! 与 `harness/antd_input_reference.html?kind=typography` 对齐（行高 72）。

use std::env;

use gpui::{
    AnyElement, App, Bounds, Context, IntoElement, KeyBinding, ParentElement, Render,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Text, Theme, Title, theme};

actions!(typography_panel_shot, [Quit]);

const ROWS: [&str; 8] = [
    "title1",
    "title3",
    "title5",
    "text",
    "secondary",
    "danger",
    "code",
    "mark",
];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const ROW_HEIGHT: f32 = 72.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 720.0;

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

        for (ix, label) in ROWS.into_iter().enumerate() {
            let content: AnyElement = match ix {
                0 => Title::new(1, "Title 1").into_any_element(),
                1 => Title::new(3, "Title 3").into_any_element(),
                2 => Title::new(5, "Title 5").into_any_element(),
                3 => Text::new("Default text").into_any_element(),
                4 => Text::new("Secondary").secondary().into_any_element(),
                5 => Text::new("Danger").danger().into_any_element(),
                6 => Text::new("code").code().into_any_element(),
                _ => Text::new("mark").mark().into_any_element(),
            };
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
                    title: Some("typography_panel_shot".into()),
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
