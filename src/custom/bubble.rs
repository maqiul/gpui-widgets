//! 对话气泡：角色头像 + 内容 + 左右位置 + 加载态 + 变体（对标 antd X Bubble）。
//!
//! ```ignore
//! Bubble::new("你好，有什么可以帮你？").avatar("助手")
//! Bubble::new("帮我写段代码").avatar("我").right()
//! Bubble::new("").loading(true)
//! ```

use gpui::{Div, ElementId, IntoElement, Role, SharedString, Stateful, div, prelude::*, px};

use crate::{Avatar, Spinner, a11y, theme};
/// 气泡位置。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BubblePlacement {
    Left,
    Right,
}

/// 气泡外观变体。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BubbleVariant {
    /// 填充底色（默认）。
    Filled,
    /// 描边。
    Outlined,
    /// 阴影卡片。
    Shadow,
}

/// 对话气泡。受控组件（无回调）。
pub struct Bubble {
    id: ElementId,
    content: SharedString,
    avatar: Option<SharedString>,
    placement: BubblePlacement,
    loading: bool,
    variant: BubbleVariant,
}

impl Bubble {
    pub fn new(content: impl Into<SharedString>) -> Self {
        let content = content.into();
        let id = ElementId::Name(format!("bubble-{:p}", content.as_ptr()).into());
        Self {
            id,
            content,
            avatar: None,
            placement: BubblePlacement::Left,
            loading: false,
            variant: BubbleVariant::Filled,
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 头像名称（取首字符 + 哈希底色）。
    pub fn avatar(mut self, name: impl Into<SharedString>) -> Self {
        self.avatar = Some(name.into());
        self
    }

    pub fn placement(mut self, placement: BubblePlacement) -> Self {
        self.placement = placement;
        self
    }

    /// 靠右（用户侧）。
    pub fn right(mut self) -> Self {
        self.placement = BubblePlacement::Right;
        self
    }

    /// 加载态（显示 Spinner 代替内容）。
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn variant(mut self, variant: BubbleVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl IntoElement for Bubble {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let m = theme::with(crate::theme::metric::bubble_metric);
        let is_right = self.placement == BubblePlacement::Right;
        let root_id = format!("bubble-list-{:?}", self.id);
        let item_id = format!("bubble-item-{:?}", self.id);
        let speaker = self.avatar.clone().unwrap_or_else(|| "对话气泡".into());
        let mut row = div()
            .id(ElementId::Name(root_id.clone().into()))
            .flex()
            .flex_row()
            .items_start()
            .gap_2()
            .w_full();
        if is_right {
            row = row.flex_row_reverse();
        }

        if let Some(name) = self.avatar.clone() {
            row = row.child(
                Avatar::new(name)
                    .id(SharedString::from(format!("{item_id}-avatar")))
                    .size(px(m.avatar_size)),
            );
        }

        let semantics = a11y::Semantics::new(
            Role::ListItem,
            if self.loading {
                SharedString::from(format!("{speaker}：正在生成回复"))
            } else {
                speaker.clone()
            },
        )
        .id(item_id.clone())
        .value(if self.loading {
            "正在生成回复".into()
        } else {
            self.content.clone()
        });
        let mut bubble = semantics.apply(
            div()
                .id(ElementId::Name(item_id.into()))
                .px_3()
                .py_2()
                .rounded_lg()
                .max_w(px(m.max_width))
                .min_w(px(0.))
                .flex_shrink(1.0),
        );
        bubble = match self.variant {
            BubbleVariant::Filled => bubble.bg(if is_right {
                theme::primary_bg()
            } else {
                theme::control_item_bg_hover()
            }),
            BubbleVariant::Outlined => bubble
                .border_1()
                .border_color(theme::border())
                .bg(theme::bg_container()),
            BubbleVariant::Shadow => bubble.bg(theme::bg_container()).shadow_md(),
        };

        if self.loading {
            bubble = bubble.child(Spinner::new().size(px(m.spinner_size)));
        } else {
            bubble = bubble.child(
                div()
                    .text_sm()
                    .text_color(theme::text())
                    .child(self.content),
            );
        }

        a11y::Semantics::new(Role::List, "对话消息")
            .id(root_id)
            .apply(row)
            .child(bubble)
    }
}
