//! 标签页溢出示例：滚动溢出（‹ › 翻页箭头 + 活动页自动滚入视野）与「＋」新增。

use gpui::{
    App, Bounds, Context, KeyBinding, Render, TitlebarOptions, Window, WindowBounds, WindowOptions,
    actions, div, prelude::*, px, size,
};
use gpui_widgets::{Tabs, Theme, theme};

actions!(tabs_overflow, [Quit]);

struct Panel {
    line: Vec<String>,
    line_tab: usize,
    card: Vec<String>,
    card_tab: usize,
    added: usize,
}

impl Panel {
    fn new() -> Self {
        Self {
            line: (1..=14).map(|ix| format!("标签页 {ix}")).collect(),
            line_tab: 9,
            card: (1..=8).map(|ix| format!("卡片 {ix}")).collect(),
            card_tab: 0,
            added: 0,
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let line = Tabs::new(self.line.clone(), self.line_tab)
            .id("overflow-line")
            .scrollable()
            .closable(true)
            .on_select(cx.listener(|this, ix, _, cx| {
                this.line_tab = *ix;
                cx.notify();
            }))
            .on_close(cx.listener(|this, ix, _, cx| {
                if this.line.len() > 1 {
                    this.line.remove(*ix);
                    this.line_tab = this.line_tab.min(this.line.len() - 1);
                }
                cx.notify();
            }));

        let entity = cx.entity();
        let card = Tabs::new(self.card.clone(), self.card_tab)
            .id("overflow-card")
            .card()
            .scrollable()
            .on_select(cx.listener(|this, ix, _, cx| {
                this.card_tab = *ix;
                cx.notify();
            }))
            .on_add(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.added += 1;
                    this.card.push(format!("新增 {}", this.added));
                    this.card_tab = this.card.len() - 1;
                    cx.notify();
                })
            });

        let current = self.line.get(self.line_tab).cloned().unwrap_or_default();

        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .bg(theme::bg_container())
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child("Tabs 溢出 · 滚动溢出箭头 + 活动页自动滚入视野 + 「＋」新增"),
            )
            .child(
                div()
                    .w(px(620.))
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(line)
                    .child(card),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text())
                    .child(format!("当前页签：{current}")),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        theme::set_theme(Theme::light());

        let bounds = Bounds::centered(None, size(px(760.0), px(420.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("tabs_overflow".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Panel::new()),
        )
        .unwrap();
        cx.activate(true);
    });
}
