//! 状态栏：应用底部的紧凑信息条，支持文本、自定义内容、分隔线与弹性留白。

use gpui::{AnyElement, ElementId, IntoElement, Role, SharedString, div, prelude::*, px};

use crate::a11y;
use crate::theme::{self, metric};

enum StatusBarEntry {
    Child(AnyElement),
    Separator,
    Spacer,
}

/// 底部状态栏。需要唯一 id，便于无障碍树稳定标识。
#[derive(IntoElement)]
pub struct StatusBar {
    id: ElementId,
    entries: Vec<StatusBarEntry>,
}

impl StatusBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            entries: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.entries
            .push(StatusBarEntry::Child(child.into_any_element()));
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.entries.extend(
            children
                .into_iter()
                .map(|child| StatusBarEntry::Child(child.into_any_element())),
        );
        self
    }

    /// 添加一个继承状态栏字色与字号的文本项。
    pub fn item(mut self, label: impl Into<SharedString>) -> Self {
        self.entries.push(StatusBarEntry::Child(
            div()
                .flex()
                .items_center()
                .child(label.into())
                .into_any_element(),
        ));
        self
    }

    /// 添加一条竖向分隔线。
    pub fn separator(mut self) -> Self {
        self.entries.push(StatusBarEntry::Separator);
        self
    }

    /// 添加弹性留白，把后续项目推到右侧。
    pub fn spacer(mut self) -> Self {
        self.entries.push(StatusBarEntry::Spacer);
        self
    }
}

impl RenderOnce for StatusBar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let m = theme::with(metric::status_bar_metric);
        let semantic_id = format!("{:?}", self.id);
        let mut bar = a11y::Semantics::new(Role::Status, "状态栏")
            .id(semantic_id)
            .apply(div().id(self.id))
            .w_full()
            .h(px(m.height))
            .flex_shrink_0()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(m.gap))
            .px(px(m.padding_inline))
            .border_t(px(m.border_width))
            .border_color(m.border_color)
            .bg(m.bg)
            .text_size(px(m.font_size))
            .line_height(px(m.height))
            .text_color(m.fg);

        for entry in self.entries {
            bar = match entry {
                StatusBarEntry::Child(child) => bar.child(child),
                StatusBarEntry::Separator => bar.child(
                    div()
                        .flex_shrink_0()
                        .w(px(m.border_width))
                        .h(px(m.separator_height))
                        .bg(m.border_color),
                ),
                StatusBarEntry::Spacer => bar.child(div().flex_1()),
            };
        }
        bar
    }
}
