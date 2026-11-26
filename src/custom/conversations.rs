//! 会话列表：AI 对话历史会话切换（对标 antd X Conversations）。
//!
//! ```ignore
//! Conversations::new(vec![
//!     ConversationItem::new("c1", "新的对话").icon("💬"),
//!     ConversationItem::new("c2", "代码审查"),
//! ]).active("c1").on_select(cx.listener(|this, key, _, cx| { /* 切换会话 */ }))
//! ```

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, IntoElement, Orientation, Role, SharedString, Stateful,
    Window, div, prelude::*,
};

use crate::{a11y, theme};

/// 单条会话。
pub struct ConversationItem {
    key: SharedString,
    label: SharedString,
    icon: Option<SharedString>,
}

impl ConversationItem {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// 会话列表。受控组件。
pub struct Conversations {
    id: ElementId,
    items: Vec<ConversationItem>,
    active: Option<SharedString>,
    on_select: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
}

impl Conversations {
    pub fn new(items: Vec<ConversationItem>) -> Self {
        Self {
            id: ElementId::Name("conversations".into()),
            items,
            active: None,
            on_select: None,
        }
    }

    /// 覆盖列表稳定 ID（同页多实例时使用）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 当前选中会话 key。
    pub fn active(mut self, key: impl Into<SharedString>) -> Self {
        self.active = Some(key.into());
        self
    }

    pub fn on_select(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Conversations {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let count = self.items.len();
        let root_id = format!("{:?}", self.id);
        let mut root = div().id(self.id.clone()).flex().flex_col().gap_1().w_full();

        for (ix, item) in self.items.into_iter().enumerate() {
            let is_active = self.active.as_ref() == Some(&item.key);
            let on_select = self.on_select.clone();
            let interactive = on_select.is_some();
            let key = item.key.clone();
            let label = item.label.clone();
            let row_id = format!("conversation-{root_id}-{}", item.key);

            let mut row = div()
                .id(ElementId::Name(row_id.clone().into()))
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .py_2()
                .rounded_md()
                .cursor(CursorStyle::PointingHand);

            row = if is_active {
                row.bg(theme::primary_bg())
                    .text_color(theme::primary())
                    .font_weight(gpui::FontWeight::MEDIUM)
            } else {
                row.text_color(theme::text())
                    .hover(|style| style.bg(theme::control_item_bg_hover()))
            };

            if let Some(icon) = item.icon {
                row = row.child(div().text_sm().child(icon));
            }
            row = row.child(div().flex_1().text_sm().child(item.label));

            row = a11y::Semantics::new(Role::ListBoxOption, label)
                .id(row_id)
                .value(key.clone())
                .selected(is_active)
                .position(ix + 1, count)
                .focusable(interactive)
                .apply(row);
            if let Some(on_select) = on_select {
                row = a11y::click(row, move |_, window, cx| on_select(&key, window, cx));
            }
            root = root.child(row);
        }

        a11y::Semantics::new(Role::ListBox, "会话列表")
            .id(format!("conversations-list-{root_id}"))
            .orientation(Orientation::Vertical)
            .apply(root)
    }
}
