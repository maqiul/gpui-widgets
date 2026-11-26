//! 欢迎卡片：图标 + 标题 + 描述，AI 对话开场引导（对标 antd X Welcome）。
//!
//! ```ignore
//! Welcome::new("你好，我是助手")
//!     .icon("🤖")
//!     .description("有什么可以帮你的吗？")
//! ```

use gpui::{Div, ElementId, IntoElement, Role, SharedString, Stateful, div, prelude::*, px};

use crate::{a11y, theme};

/// 欢迎卡片。受控组件（无回调）。
pub struct Welcome {
    id: ElementId,
    icon: Option<SharedString>,
    title: SharedString,
    description: Option<SharedString>,
}

impl Welcome {
    pub fn new(title: impl Into<SharedString>) -> Self {
        let title = title.into();
        let id = ElementId::Name(format!("welcome-{:p}", title.as_ptr()).into());
        Self {
            id,
            icon: None,
            title,
            description: None,
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 左侧图标（emoji 或字符）。
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 标题下方的描述文字。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl IntoElement for Welcome {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let m = theme::with(crate::theme::metric::welcome_metric);
        let heading_id = format!("welcome-heading-{:?}", self.id);
        let description = self.description.clone();
        let mut root = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap_4()
            .w_full()
            .p_5()
            .bg(theme::bg_container())
            .border_1()
            .border_color(theme::border())
            .rounded_lg();

        if let Some(icon) = self.icon {
            root = root.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(m.icon_size))
                    .h(px(m.icon_size))
                    .flex_shrink_0()
                    .rounded_lg()
                    .bg(theme::primary_bg())
                    .text_size(px(m.icon_font_size))
                    .child(icon),
            );
        }

        let mut text_col = div().flex().flex_col().gap_1().flex_1().min_w(px(0.));
        text_col = text_col.child(
            a11y::Semantics::new(Role::Heading, self.title.clone())
                .id(heading_id.clone())
                .level(1)
                .apply(
                    div()
                        .id(ElementId::Name(heading_id.into()))
                        .text_lg()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::text())
                        .child(self.title.clone()),
                ),
        );
        if let Some(desc) = self.description {
            text_col = text_col.child(
                div()
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child(desc),
            );
        }

        let mut semantics = a11y::Semantics::new(Role::Region, self.title.clone())
            .id(format!("welcome-region-{:?}", self.id));
        if let Some(description) = description {
            semantics = semantics.description(description);
        }
        semantics.apply(root).child(text_col)
    }
}
