//! 截图基线用面板：与 `harness/antd_input_reference.html` kind=treeselect2 同构（展开浮层）。
//!
//! ```text
//! cargo run --offline --example treeselect2_panel_shot -- --appearance light
//! ```
//!
//! 面板几何 420×408（pad 16、行带 196、label 140、gap 8、cell 240）；
//! 行高 180 容纳触发框 32 + 浮层间隙 4 + 树浮层（8×2 内边距 + 4 节点）。

use std::env;

use gpui::{
    App, Bounds, IntoElement, KeyBinding, ParentElement, Render, SharedString, TitlebarOptions,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_widgets::{Theme, TreeNode, TreeSelect, theme};

actions!(treeselect2_panel_shot, [Quit]);

const ROWS: [&str; 2] = ["selected", "collapsed"];

const PAD: f32 = 16.0;
const LABEL_WIDTH: f32 = 140.0;
const CELL_WIDTH: f32 = 240.0;
const ROW_HEIGHT: f32 = 180.0;
const ROW_GAP: f32 = 16.0;
const CELL_GAP: f32 = 8.0;
const PANEL_WIDTH: f32 = 420.0;
const PANEL_HEIGHT: f32 = 408.0;

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

fn nodes() -> Vec<TreeNode> {
    vec![
        TreeNode::new("src", "src")
            .child(TreeNode::new("src-main", "main.rs"))
            .child(TreeNode::new("src-lib", "lib.rs")),
        TreeNode::new("readme", "README.md"),
    ]
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
            let mut select = TreeSelect::new(SharedString::from(format!("row{ix}")), nodes())
                .placeholder("Please select")
                .open(true)
                .focused(false);
            if ix == 0 {
                select = select
                    .value(Some("src-lib".into()))
                    .expanded(vec!["src".into()]);
            }
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
                    .child(div().w(px(CELL_WIDTH)).child(select)),
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
                    title: Some("treeselect2_panel_shot".into()),
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
