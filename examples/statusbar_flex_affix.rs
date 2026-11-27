//! StatusBar / Flex / Affix 组合示例：底部状态栏、弹性布局与滚动吸顶。

use gpui::{
    App, Bounds, Context, Entity, IntoElement, KeyBinding, Point, Render, RenderOnce, ScrollHandle,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::{Affix, Button, Flex, StatusBar, Theme, theme};

actions!(statusbar_flex_affix, [Quit]);

struct Panel {
    counter: Entity<usize>,
}

impl Panel {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            counter: cx.new(|_| 0usize),
        }
    }
}

#[derive(IntoElement)]
struct AffixCounter {
    counter: Entity<usize>,
}

impl RenderOnce for AffixCounter {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let _ = window;
        let count = self.counter.read(cx);
        let counter = self.counter.clone();
        div().flex().items_center().gap_2().child(
            Button::new(format!("count {count}"))
                .id("affix-counter-inc")
                .small()
                .on_click(move |_, _, cx| {
                    counter.update(cx, |count, _| *count += 1);
                    cx.refresh_windows();
                }),
        )
    }
}

impl Render for Panel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scroll = window
            .use_keyed_state("status-demo-scroll", cx, |_, _| ScrollHandle::new())
            .read(cx)
            .clone();
        let top_scroll = scroll.clone();
        let bottom_scroll = scroll.clone();

        let fill_alter = theme::with(|tokens| {
            theme::blend(tokens.color_fill_tertiary, tokens.color_bg_container)
        });

        let toolbar = div()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .p_2()
            .bg(theme::bg_container())
            .border_1()
            .border_color(theme::border())
            .rounded(px(6.))
            .child("吸顶工具栏 · 滚动后保持可见")
            .child(AffixCounter {
                counter: self.counter.clone(),
            })
            .child(Button::new("scroll-top").small().on_click(move |_, _, cx| {
                top_scroll.set_offset(Point::new(px(0.), px(0.)));
                cx.refresh_windows();
            }));

        let mut rows = Vec::new();
        for ix in 1..=28 {
            rows.push(
                div()
                    .w_full()
                    .h(px(44.))
                    .flex()
                    .items_center()
                    .px_3()
                    .border_b_1()
                    .border_color(theme::border_secondary())
                    .child(format!("内容行 {ix:02}")),
            );
        }

        let bottom_bar = div()
            .w_full()
            .p_2()
            .bg(theme::bg_container())
            .border_1()
            .border_color(theme::border())
            .rounded(px(6.))
            .child("吸底栏 · 滚动到接近底部时固定");

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme::bg_layout())
            .child(
                div()
                    .h(px(52.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px_4()
                    .bg(theme::bg_container())
                    .border_b_1()
                    .border_color(theme::border())
                    .child("StatusBar / Flex / Affix"),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .id("content")
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(Affix::new("top-affix", toolbar, scroll.clone()).offset_top(px(0.)))
                    .child(Flex::new().wrap().gap(px(8.)).children((1..=10).map(|ix| {
                        div()
                            .px_3()
                            .py_2()
                            .bg(theme::primary_bg())
                            .text_color(theme::primary())
                            .rounded(px(4.))
                            .child(format!("Flex {ix}"))
                    })))
                    .child(
                        div().p_3().bg(fill_alter).rounded(px(6.)).child(
                            Flex::new()
                                .justify_between()
                                .items_center()
                                .child("左")
                                .child("中")
                                .child("右"),
                        ),
                    )
                    .child(
                        Flex::new()
                            .column()
                            .items_start()
                            .gap(px(6.))
                            .child("Flex 垂直排列")
                            .child("第二行")
                            .child("第三行"),
                    )
                    .children(rows)
                    .child(
                        Button::new("scroll-bottom")
                            .secondary()
                            .on_click(move |_, _, cx| {
                                let max = bottom_scroll.max_offset();
                                bottom_scroll.set_offset(Point::new(px(0.), -max.y));
                                cx.refresh_windows();
                            }),
                    )
                    .child(
                        Affix::new("bottom-affix", bottom_bar, scroll.clone())
                            .offset_bottom(px(0.)),
                    )
                    .child(div().h(px(80.))),
            )
            .child(
                StatusBar::new("demo-status")
                    .item("就绪")
                    .separator()
                    .item("Ln 1, Col 1")
                    .spacer()
                    .item("UTF-8")
                    .separator()
                    .item("Rust"),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());

        let bounds = Bounds::centered(None, size(px(920.0), px(680.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("statusbar_flex_affix".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Panel::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
