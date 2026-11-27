//! Typography 受控编辑、复制反馈、截断展开与段落修饰运行时示例。

use gpui::{
    App, Bounds, Context, Div, IntoElement, KeyBinding, ParentElement, Render, TitlebarOptions,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
use gpui_widgets::input::text_input::bind_keys;
use gpui_widgets::{
    CopyConfig, EditableConfig, EllipsisConfig, Paragraph, Text, Theme, Title, theme,
};

actions!(typography_interactive_demo, [Quit]);

fn caption(text: &str) -> Div {
    div()
        .text_sm()
        .text_color(theme::text_secondary())
        .child(text.to_string())
}

struct Panel {
    title: String,
    editing: bool,
    expanded: bool,
    status: String,
}

impl Panel {
    fn new(_: &mut Context<Self>) -> Self {
        Self {
            title: "受控编辑标题".into(),
            editing: false,
            expanded: false,
            status: "点击标题旁的编辑图标开始修改".into(),
        }
    }
}

impl Render for Panel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_6()
            .bg(theme::bg_layout())
            .child(
                div()
                    .text_lg()
                    .text_color(theme::text())
                    .child("Typography Interactive"),
            )
            .child(caption(
                "编辑为受控模式：Enter 提交，Escape 取消；复制后显示短暂反馈。",
            ))
            .child(
                Title::new(2, self.title.clone())
                    .id("editable-title")
                    .title("这是一个可编辑标题")
                    .editable(
                        EditableConfig::default()
                            .editing(self.editing)
                            .max_length(40)
                            .on_editing_change(cx.listener(|this, editing, _, cx| {
                                this.editing = *editing;
                                this.status = if *editing {
                                    "正在编辑：Enter 提交，Escape 取消".into()
                                } else {
                                    this.status.clone()
                                };
                                cx.notify();
                            }))
                            .on_change(cx.listener(|this, value: &str, _, cx| {
                                this.title = value.to_string();
                                this.status = format!("已提交：{value}");
                                cx.notify();
                            }))
                            .on_cancel({
                                let view = view.clone();
                                move |_, cx| {
                                    view.update(cx, |this, cx| {
                                        this.status = "已取消编辑".into();
                                        cx.notify();
                                    });
                                }
                            }),
                    ),
            )
            .child(
                Paragraph::new("点击右侧复制图标，将整段文本写入剪贴板。")
                    .id("copyable-paragraph")
                    .copyable(CopyConfig::default().tooltip("复制段落").on_copy({
                        let view = view.clone();
                        move |_, cx| {
                            view.update(cx, |this, cx| {
                                this.status = "段落已复制到剪贴板".into();
                                cx.notify();
                            });
                        }
                    })),
            )
            .child(
                Text::new("这是一段较长的行内文本，用于验证按字素簇截断和受控展开。")
                    .id("ellipsis-text")
                    .ellipsis(
                        EllipsisConfig::default()
                            .max_chars(12)
                            .expandable(true)
                            .expanded(self.expanded)
                            .on_expand(cx.listener(|this, expanded, _, cx| {
                                this.expanded = *expanded;
                                this.status = if *expanded {
                                    "已展开完整文本".into()
                                } else {
                                    "已收起文本".into()
                                };
                                cx.notify();
                            })),
                    ),
            )
            .child(
                Text::new("短文本不显示展开按钮")
                    .id("short-ellipsis-text")
                    .ellipsis(EllipsisConfig::default().max_chars(20).expandable(true)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Paragraph::new("加粗文本").id("p-strong").strong())
                    .child(Paragraph::new("斜体文本").id("p-italic").italic())
                    .child(Paragraph::new("下划线文本").id("p-underline").underline())
                    .child(Paragraph::new("删除线文本").id("p-delete").delete())
                    .child(Paragraph::new("行内代码").id("p-code").code())
                    .child(Paragraph::new("高亮标记").id("p-mark").mark()),
            )
            .child(
                div()
                    .mt_2()
                    .p_3()
                    .rounded(px(8.))
                    .bg(theme::bg_container())
                    .border_1()
                    .border_color(theme::border())
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child(self.status.clone()),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());
        bind_keys(cx);
        theme::set_theme(Theme::light());
        let bounds = Bounds::centered(None, size(px(760.), px(620.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Typography Interactive".into()),
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
