//! 思维链：可折叠的推理步骤链 + 状态指示（对标 antd X ThoughtChain）。
//!
//! ```ignore
//! ThoughtChain::new(vec![
//!     ThoughtNode::new("理解需求").status(ThoughtStatus::Success),
//!     ThoughtNode::new("生成方案").description("正在分析…").status(ThoughtStatus::Pending),
//! ]).expanded(vec![0]).on_toggle(cx.listener(|this, ix, _, cx| { /* 展开/收起 */ }))
//! ```

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, IntoElement, Orientation, Role, SharedString, Stateful,
    Window, div, prelude::*, px,
};

use crate::{
    a11y,
    theme::{self, metric},
};

/// 步骤状态。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThoughtStatus {
    Pending,
    Success,
    Error,
}

/// 单个推理步骤。
pub struct ThoughtNode {
    title: SharedString,
    description: Option<SharedString>,
    status: ThoughtStatus,
}

impl ThoughtNode {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            status: ThoughtStatus::Success,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn status(mut self, status: ThoughtStatus) -> Self {
        self.status = status;
        self
    }
}

/// 思维链。受控组件（展开集合受控）。
pub struct ThoughtChain {
    id: ElementId,
    nodes: Vec<ThoughtNode>,
    expanded: Vec<usize>,
    on_toggle: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl ThoughtChain {
    pub fn new(nodes: Vec<ThoughtNode>) -> Self {
        Self {
            id: ElementId::Name("thought-chain".into()),
            nodes,
            expanded: Vec::new(),
            on_toggle: None,
        }
    }

    /// 展开的步骤下标集合。
    /// 覆盖列表稳定 ID（同页多实例时使用）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn expanded(mut self, expanded: Vec<usize>) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn on_toggle(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(listener));
        self
    }
}

/// 状态指示图标。
fn status_text(status: ThoughtStatus) -> &'static str {
    match status {
        ThoughtStatus::Pending => "待处理",
        ThoughtStatus::Success => "成功",
        ThoughtStatus::Error => "错误",
    }
}

fn status_icon(status: ThoughtStatus, m: &metric::ThoughtChainMetric) -> Div {
    let style = theme::with(|t| metric::thought_status_style(t, status));
    let mut icon = div()
        .w(px(m.icon_size))
        .h(px(m.icon_size))
        .rounded_full()
        .text_color(style.fg)
        .text_size(px(m.icon_font_size));
    if let Some(border) = style.border {
        icon = icon.border_2().border_color(border);
    }
    if let Some(fill) = style.fill {
        icon = icon.bg(fill).flex().items_center().justify_center();
    }
    if let Some(glyph) = style.glyph {
        icon = icon.child(glyph);
    }
    icon
}

impl IntoElement for ThoughtChain {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let root_id = format!("{:?}", self.id);
        let mut root = div().id(self.id.clone()).flex().flex_col().w_full();
        let m = theme::with(metric::thought_chain_metric);
        let len = self.nodes.len();

        for (ix, node) in self.nodes.into_iter().enumerate() {
            let is_expanded = self.expanded.contains(&ix);
            let is_last = ix + 1 == len;
            let on_toggle = self.on_toggle.clone();
            let interactive = on_toggle.is_some();
            let title = node.title.clone();
            let description = node.description.clone();
            let item_id = format!("thought-{root_id}-item-{ix}");
            let title_id = format!("thought-{root_id}-title-{ix}");

            // 指示列：状态图标 + 向下连接线（非末项）
            let mut indicator = div()
                .flex()
                .flex_col()
                .items_center()
                .w(px(m.indicator_width));
            indicator = indicator.child(status_icon(node.status, &m));
            if !is_last {
                indicator = indicator.child(
                    div()
                        .flex_1()
                        .w(px(m.line_width))
                        .my_1()
                        .bg(theme::border()),
                );
            }

            // 内容列：标题（可点击折叠）+ 展开描述
            let mut content = div().flex().flex_col().gap_1().flex_1().pb_4();
            let mut title_row = div()
                .id(ElementId::Name(title_id.clone().into()))
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .cursor(CursorStyle::PointingHand);
            title_row = title_row.child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme::text())
                    .child(title.clone()),
            );
            title_row = title_row.child(
                div()
                    .text_xs()
                    .text_color(theme::text_secondary())
                    .child(if is_expanded { "▾" } else { "▸" }),
            );
            let mut semantics = a11y::Semantics::new(Role::Button, title.clone())
                .id(title_id)
                .expanded(is_expanded)
                .position(ix + 1, len)
                .focusable(interactive);
            if let Some(description) = description {
                semantics = semantics.description(description);
            }
            title_row = semantics.apply(title_row);
            if let Some(on_toggle) = on_toggle {
                title_row = a11y::click(title_row, move |_, window, cx| on_toggle(&ix, window, cx));
            }
            content = content.child(title_row);

            if is_expanded {
                if let Some(desc) = node.description {
                    content = content.child(
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .child(desc),
                    );
                }
            }

            let item = a11y::Semantics::new(
                Role::ListItem,
                format!("{}，状态：{}", title, status_text(node.status)),
            )
            .id(item_id.clone())
            .position(ix + 1, len)
            .apply(
                div()
                    .id(ElementId::Name(item_id.into()))
                    .flex()
                    .flex_row()
                    .gap_3(),
            );
            root = root.child(item.child(indicator).child(content));
        }

        a11y::Semantics::new(Role::List, "思维链")
            .id(format!("thought-chain-list-{root_id}"))
            .orientation(Orientation::Vertical)
            .apply(root)
    }
}
