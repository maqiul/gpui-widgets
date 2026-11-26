//! 锚点导航（对应 antd Anchor）：连续指示轨 colorSplit + 激活 ink colorPrimary。
//! 受控组件：`active` 指定当前高亮锚点 key，点击通过 `on_click` 上报；
//! 滚动定位由调用方自行处理（GPUI 无浏览器式锚点滚动）。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, IntoElement, Role, SharedString, Stateful, Window, div,
    prelude::*, px,
};

use crate::a11y;
use crate::theme::{
    self,
    metric::{anchor_ink_color, anchor_metric, anchor_rail_color, anchor_title_color},
};

/// 锚点项（可嵌套子锚点）
pub struct AnchorItem {
    key: SharedString,
    title: SharedString,
    children: Vec<AnchorItem>,
}

impl AnchorItem {
    pub fn new(key: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            children: Vec::new(),
        }
    }

    /// 追加子锚点（渲染时缩进一级）
    pub fn child(mut self, child: AnchorItem) -> Self {
        self.children.push(child);
        self
    }
}

/// 锚点导航。
///
/// ```ignore
/// Anchor::new(vec![
///     AnchorItem::new("basic", "基本用法"),
///     AnchorItem::new("api", "API").child(AnchorItem::new("props", "属性")),
/// ])
/// .active(self.anchor_key.clone())
/// .on_click(cx.listener(|this, key: &SharedString, _, cx| {
///     this.anchor_key = key.clone();
///     cx.notify();
/// }))
/// ```
pub struct Anchor {
    id: SharedString,
    items: Vec<AnchorItem>,
    active: Option<SharedString>,
    on_click: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
}

impl Anchor {
    pub fn new(items: Vec<AnchorItem>) -> Self {
        Self {
            id: SharedString::from("anchor"),
            items,
            active: None,
            on_click: None,
        }
    }

    /// 实例标识：同屏多个 Anchor 时区分元素 id（默认 "anchor"）
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    /// 当前高亮锚点 key（受控）
    pub fn active(mut self, key: impl Into<SharedString>) -> Self {
        self.active = Some(key.into());
        self
    }

    pub fn on_click(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(listener));
        self
    }

    /// 单个锚点行：标题链接 + 激活 ink 段；子锚点嵌在父链接的 padding 盒内
    /// （antd：`.ant-anchor-link` 的 paddingBlock 包住整棵子树，子级再缩进 16px）
    fn render_item(
        &self,
        item: &AnchorItem,
        depth: usize,
        index: usize,
        path: &str,
        m: &theme::metric::AnchorMetric,
    ) -> gpui::Stateful<Div> {
        let is_active = self.active.as_ref() == Some(&item.key);
        let key = item.key.clone();
        let on_click = self.on_click.clone();
        let pad_block = if depth == 0 {
            m.link_pad_block
        } else {
            m.nested_pad_block
        };
        let idle = theme::with(|t| anchor_title_color(t, is_active, false));
        let hover = theme::with(|t| anchor_title_color(t, is_active, true));

        let mut title = div()
            .text_size(px(theme::with(|t| t.font_size)))
            .line_height(px(m.font_height))
            .text_color(idle)
            .hover(move |s| s.text_color(hover))
            .child(item.title.clone());
        if !item.children.is_empty() {
            title = title.mb(px(m.title_gap));
        }

        let item_path = format!("{path}-{index}");
        let link_id = format!("{}-{}", self.id, item_path);
        let mut link = a11y::Semantics::new(Role::Link, item.title.clone())
            .id(link_id.clone())
            .focusable(on_click.is_some())
            .apply(div().id(ElementId::Name(link_id.into())))
            .relative()
            .flex()
            .flex_col()
            .py(px(pad_block))
            .pl(px(m.link_pad_left))
            .cursor(CursorStyle::PointingHand)
            .child(title);
        if is_active {
            link = link.aria_description("当前锚点").child(
                div()
                    .absolute()
                    .left(px(-(m.rail_width + m.link_pad_left * depth as f32)))
                    .top(px(pad_block))
                    .w(px(m.rail_width))
                    .h(px(m.font_height))
                    .bg(theme::with(anchor_ink_color)),
            );
        }
        if let Some(on_click) = on_click {
            link = a11y::click_void(link, move |window, cx| on_click(&key, window, cx));
        }
        for (child_ix, child) in item.children.iter().enumerate() {
            link = link.child(self.render_item(child, depth + 1, child_ix, &item_path, m));
        }
        link
    }
}

impl IntoElement for Anchor {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let m = theme::with(anchor_metric);
        let root_id = format!("{}-navigation", self.id);
        let mut root = a11y::Semantics::new(Role::Navigation, "锚点导航")
            .id(root_id.clone())
            .apply(div().id(ElementId::Name(root_id.into())))
            .relative()
            .flex()
            .flex_col()
            .pl(px(m.rail_width))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(px(m.rail_width))
                    .bg(theme::with(anchor_rail_color)),
            );
        for (ix, item) in self.items.iter().enumerate() {
            root = root.child(self.render_item(item, 0, ix, "", &m));
        }
        root
    }
}
