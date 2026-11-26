//! 提示词卡片：网格排列的引导提示，点击触发回调（对标 antd X Prompts）。
//!
//! ```ignore
//! Prompts::new(vec![
//!     PromptItem::new("写一首诗").icon("✍️").description("以春天为题"),
//!     PromptItem::new("解释代码").icon("💡"),
//! ]).on_click(cx.listener(|this, ix, _, cx| { /* 选中第 ix 条 */ }))
//! ```

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, IntoElement, Role, SharedString, Stateful, Window, div,
    prelude::*, px,
};

use crate::{a11y, theme};

/// 单条提示词。
pub struct PromptItem {
    label: SharedString,
    description: Option<SharedString>,
    icon: Option<SharedString>,
}

impl PromptItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            description: None,
            icon: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// 提示词卡片组。受控组件。
pub struct Prompts {
    id: ElementId,
    items: Vec<PromptItem>,
    on_click: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl Prompts {
    pub fn new(items: Vec<PromptItem>) -> Self {
        Self {
            id: ElementId::Name("prompts".into()),
            items,
            on_click: None,
        }
    }

    /// 覆盖列表稳定 ID（同页多实例时使用）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 点击某条提示词回调（下标）。
    pub fn on_click(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Prompts {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let m = theme::with(crate::theme::metric::prompts_metric);
        let count = self.items.len();
        let root_id = format!("{:?}", self.id);
        let mut root = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .flex_wrap()
            .gap_3()
            .w_full();

        for (ix, item) in self.items.into_iter().enumerate() {
            let on_click = self.on_click.clone();
            let interactive = on_click.is_some();
            let label = item.label.clone();
            let description = item.description.clone();
            let card_id = format!("prompt-{root_id}-{ix}");
            let mut card = div()
                .id(ElementId::Name(card_id.clone().into()))
                .flex()
                .flex_col()
                .gap_1()
                .w(px(m.card_width))
                .p_3()
                .bg(theme::bg_container())
                .border_1()
                .border_color(theme::border())
                .rounded_md()
                .cursor(CursorStyle::PointingHand)
                .hover(|style| {
                    style
                        .border_color(theme::primary())
                        .bg(theme::control_item_bg_hover())
                });

            if let Some(icon) = item.icon {
                card = card.child(div().text_base().child(icon));
            }
            card = card.child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme::text())
                    .child(item.label),
            );
            if let Some(desc) = item.description {
                card = card.child(
                    div()
                        .text_xs()
                        .text_color(theme::text_secondary())
                        .child(desc),
                );
            }

            let mut semantics = a11y::Semantics::new(Role::Button, label)
                .id(card_id)
                .position(ix + 1, count)
                .focusable(interactive);
            if let Some(description) = description {
                semantics = semantics.description(description);
            }
            card = semantics.apply(card);
            if let Some(on_click) = on_click {
                card = a11y::click(card, move |_, window, cx| on_click(&ix, window, cx));
            }
            root = root.child(card);
        }

        a11y::Semantics::new(Role::List, "提示词")
            .id(format!("prompts-list-{root_id}"))
            .apply(root)
    }
}
