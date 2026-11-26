//! 时间线：纵向节点序列，节点用空心圆环标记事件，支持左右两栏与交替排布。
//!
//! 结构对齐 antd Timeline（由 dot 型 Steps 渲染）：无标题时圆点贴左、内容单列；
//! 任一项带标题时切左右两栏，圆点与连接线居中。

use gpui::{
    Div, Hsla, InteractiveElement, IntoElement, ParentElement, Role, SharedString, Stateful,
    Styled, div, px, transparent_black,
};

use crate::{
    a11y::Semantics,
    theme::{self, Tokens, metric},
};

/// 节点颜色（antd `TimelineItem.color` 的四个语义色，另支持自定义色）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimelineColor {
    Blue,
    Red,
    Green,
    Gray,
    Custom(Hsla),
}

impl From<Hsla> for TimelineColor {
    fn from(color: Hsla) -> Self {
        Self::Custom(color)
    }
}

/// 节点圆环的纯数据外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineDotState {
    pub size: f32,
    pub border_width: f32,
    pub border: Hsla,
    pub fill: Hsla,
    pub inner_size: f32,
    pub inner: Hsla,
}

/// 连接线的纯数据外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineRailState {
    pub width: f32,
    pub color: Hsla,
    pub dotted: bool,
}

/// 时间线文字的纯数据颜色。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineTextState {
    pub title: Hsla,
    pub content: Hsla,
    pub meta: Hsla,
}

/// 两种布局：无标题贴左单列；有标题时左右两栏、节点居中。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineLayout {
    Plain,
    Centered,
}

/// 圆环状态：语义色描边、透明填充；进行中状态在内侧加主色小圆点。
pub fn timeline_dot_state(
    tokens: &Tokens,
    color: TimelineColor,
    loading: bool,
) -> TimelineDotState {
    let color = match color {
        TimelineColor::Blue => tokens.color_primary,
        TimelineColor::Red => tokens.color_error,
        TimelineColor::Green => tokens.color_success,
        TimelineColor::Gray => tokens.color_text_disabled,
        TimelineColor::Custom(color) => color,
    };
    if loading {
        TimelineDotState {
            size: 10.0,
            border_width: 2.0,
            border: tokens.color_primary,
            fill: transparent_black(),
            inner_size: 4.0,
            inner: tokens.color_primary,
        }
    } else {
        TimelineDotState {
            size: 10.0,
            border_width: 2.0,
            border: color,
            fill: transparent_black(),
            inner_size: 0.0,
            inner: color,
        }
    }
}

/// 连接线状态：`colorSplit`、`lineWidthBold`；进行中节点后的线为虚线（antd dotted）。
pub fn timeline_rail_state(tokens: &Tokens, loading: bool) -> TimelineRailState {
    TimelineRailState {
        width: tokens.line_width_bold,
        color: tokens.color_split,
        dotted: loading,
    }
}

/// 时间线文字颜色：标题与内容 `colorText`，辅助文字 `colorTextDescription`。
pub fn timeline_text_state(tokens: &Tokens) -> TimelineTextState {
    TimelineTextState {
        title: tokens.color_text,
        content: tokens.color_text,
        meta: tokens.color_text_description,
    }
}

/// 任一项带标题即使用左右两栏布局。
pub fn timeline_layout(has_title: bool) -> TimelineLayout {
    if has_title {
        TimelineLayout::Centered
    } else {
        TimelineLayout::Plain
    }
}

/// 时间线节点。
///
/// ```ignore
/// TimelineItem::new("发布 v1.0")
///     .title("2026-11-16")
///     .time("10:30")
///     .description("首个稳定版本")
/// ```
pub struct TimelineItem {
    content: SharedString,
    title: Option<SharedString>,
    description: Option<SharedString>,
    time: Option<SharedString>,
    color: TimelineColor,
    loading: bool,
}

impl TimelineItem {
    pub fn new(content: impl Into<SharedString>) -> Self {
        Self {
            content: content.into(),
            title: None,
            description: None,
            time: None,
            color: TimelineColor::Blue,
            loading: false,
        }
    }

    /// 左侧（或交替时另一侧）栏目标题；任一项设置后整条时间线切两栏布局
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 内容下方的辅助说明
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 辅助时间文字（无标题时显示在内容下方，有标题时显示在标题下方）
    pub fn time(mut self, time: impl Into<SharedString>) -> Self {
        self.time = Some(time.into());
        self
    }

    /// 节点颜色，默认主色
    pub fn color(mut self, color: impl Into<TimelineColor>) -> Self {
        self.color = color.into();
        self
    }

    /// 进行中：节点内侧加主色点，后续连接线改虚线
    pub fn loading(mut self) -> Self {
        self.loading = true;
        self
    }
}

/// 时间线。
///
/// ```ignore
/// Timeline::new(vec![
///     TimelineItem::new("创建项目").time("09:00"),
///     TimelineItem::new("提交代码").time("11:30").description("修复下拉定位"),
/// ])
/// .alternate()          // 两栏内容左右交替
/// .pending("加载中…")   // 末尾进行中节点
/// ```
pub struct Timeline {
    id: Option<SharedString>,
    origin: SharedString,
    items: Vec<TimelineItem>,
    alternate: bool,
    pending: Option<SharedString>,
}

impl Timeline {
    #[track_caller]
    pub fn new(items: Vec<TimelineItem>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "timeline-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            items,
            alternate: false,
            pending: None,
        }
    }

    /// 实例 ID；同页多个时间线应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 两栏内容左右交替排布
    pub fn alternate(mut self) -> Self {
        self.alternate = true;
        self
    }

    /// 末尾追加"进行中"节点
    pub fn pending(mut self, label: impl Into<SharedString>) -> Self {
        self.pending = Some(label.into());
        self
    }

    fn dot(&self, item: &TimelineItem, tokens: &Tokens) -> Div {
        let state = timeline_dot_state(tokens, item.color, item.loading);
        let mut dot = div()
            .flex_shrink_0()
            .w(px(state.size))
            .h(px(state.size))
            .rounded_full()
            .border(px(state.border_width))
            .border_color(state.border)
            .bg(state.fill)
            .flex()
            .items_center()
            .justify_center();
        if state.inner_size > 0.0 {
            dot = dot.child(
                div()
                    .w(px(state.inner_size))
                    .h(px(state.inner_size))
                    .rounded_full()
                    .bg(state.inner),
            );
        }
        dot
    }

    fn rail_body(&self, item: &TimelineItem, tokens: &Tokens) -> Div {
        let state = timeline_rail_state(tokens, item.loading);
        let mut rail = div().h_full().border_color(state.color);
        rail = if state.dotted {
            rail.w(px(0.)).border_l(px(state.width)).border_dashed()
        } else {
            rail.w(px(state.width)).bg(state.color)
        };
        rail
    }

    fn rail_left(&self, item: &TimelineItem, tokens: &Tokens, m: &metric::TimelineMetric) -> Div {
        div()
            .absolute()
            .left(px(m.dot_size / 2.0 - m.rail_width / 2.0))
            .top(px(m.rail_top))
            .bottom(px(-m.rail_bottom))
            .child(self.rail_body(item, tokens))
    }

    fn rail_center(&self, item: &TimelineItem, tokens: &Tokens, m: &metric::TimelineMetric) -> Div {
        div()
            .absolute()
            .left(px(0.))
            .right(px(0.))
            .top(px(m.rail_top))
            .bottom(px(-m.rail_bottom))
            .flex()
            .flex_row()
            .justify_center()
            .child(self.rail_body(item, tokens))
    }

    fn dot_center(&self, item: &TimelineItem, tokens: &Tokens, m: &metric::TimelineMetric) -> Div {
        div()
            .absolute()
            .left(px(0.))
            .right(px(0.))
            .top(px(0.))
            .flex()
            .flex_row()
            .justify_center()
            .child(self.dot(item, tokens).mt(px(m.dot_top)))
    }

    fn meta(
        &self,
        value: &SharedString,
        text: &TimelineTextState,
        m: &metric::TimelineMetric,
    ) -> Div {
        div()
            .text_size(px(m.meta_size))
            .line_height(px(m.meta_line))
            .text_color(text.meta)
            .child(value.clone())
    }

    fn header(
        &self,
        item: &TimelineItem,
        text: &TimelineTextState,
        m: &metric::TimelineMetric,
        end: bool,
    ) -> Div {
        let mut header = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .min_h(px(m.header_min_height));
        if let Some(title) = &item.title {
            header = header.child(
                div()
                    .text_size(px(m.title_size))
                    .line_height(px(m.title_line))
                    .text_color(text.title)
                    .child(title.clone()),
            );
        }
        if let Some(time) = &item.time {
            header = header.child(self.meta(time, text, m));
        }
        if !end {
            header = header.text_right();
        }
        header
    }

    fn content(
        &self,
        item: &TimelineItem,
        text: &TimelineTextState,
        m: &metric::TimelineMetric,
        time_here: bool,
    ) -> Div {
        let mut content = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .min_h(px(m.header_min_height))
            .child(
                div()
                    .text_size(px(m.title_size))
                    .line_height(px(m.title_line))
                    .text_color(text.content)
                    .child(item.content.clone()),
            );
        if time_here {
            if let Some(time) = &item.time {
                content = content.child(self.meta(time, text, m));
            }
        }
        if let Some(description) = &item.description {
            content = content.child(self.meta(description, text, m));
        }
        content
    }

    fn plain_item(
        &self,
        item: &TimelineItem,
        last: bool,
        tokens: &Tokens,
        m: &metric::TimelineMetric,
        text: &TimelineTextState,
    ) -> Div {
        let mut row = div()
            .relative()
            .flex()
            .flex_row()
            .items_start()
            .min_h(px(m.item_min_height))
            .pb(px(if last { 0. } else { m.padding_plain }))
            .child(
                div()
                    .flex_shrink_0()
                    .w(px(m.dot_size))
                    .mt(px(m.dot_top))
                    .child(self.dot(item, tokens)),
            )
            .child(
                self.content(item, text, m, true)
                    .pl(px(m.icon_gap))
                    .pt(px(m.content_top)),
            );
        if !last {
            row = row.child(self.rail_left(item, tokens, m));
        }
        row
    }

    fn centered_item(
        &self,
        item: &TimelineItem,
        ix: usize,
        last: bool,
        tokens: &Tokens,
        m: &metric::TimelineMetric,
        text: &TimelineTextState,
    ) -> Div {
        let end = self.alternate && ix % 2 == 1;
        let header = self.header(item, text, m, end);
        let content = self.content(item, text, m, item.title.is_none());
        let content = if end { content.text_right() } else { content };
        let mut section = div()
            .flex()
            .flex_row()
            .items_start()
            .w_full()
            .gap(px(m.content_gap));
        section = if end {
            section.child(content).child(header)
        } else {
            section.child(header).child(content)
        };

        let mut item_el = div()
            .relative()
            .w_full()
            .pb(px(m.padding_alternate))
            .child(section);
        if !last {
            item_el = item_el.child(self.rail_center(item, tokens, m));
        }
        item_el.child(self.dot_center(item, tokens, m))
    }
}

impl IntoElement for Timeline {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        theme::with(|tokens| {
            let m = metric::TimelineMetric::resolve(tokens);
            let text = timeline_text_state(tokens);
            let has_title = self.items.iter().any(|item| item.title.is_some());
            let centered = timeline_layout(has_title) == TimelineLayout::Centered;

            let pending = self
                .pending
                .as_ref()
                .map(|pending| TimelineItem::new(pending.clone()).loading());
            let mut items: Vec<&TimelineItem> = self.items.iter().collect();
            if let Some(pending) = &pending {
                items.push(pending);
            }
            let last = items.len().saturating_sub(1);
            let owner = self.id.as_deref().unwrap_or(&self.origin);
            let list_id = format!("{owner}-list");
            let list_semantics = Semantics::new(Role::List, "时间线").id(list_id.clone());
            let mut root = list_semantics.apply(
                div()
                    .id(gpui::ElementId::Name(list_id.into()))
                    .flex()
                    .flex_col()
                    .w_full(),
            );
            for (ix, item) in items.iter().enumerate() {
                let is_last = ix == last;
                let item_element = if centered {
                    self.centered_item(item, ix, is_last, tokens, &m, &text)
                } else {
                    self.plain_item(item, is_last, tokens, &m, &text)
                };
                let label = [
                    item.title.as_ref(),
                    Some(&item.content),
                    item.time.as_ref(),
                    item.description.as_ref(),
                ]
                .into_iter()
                .flatten()
                .map(|text| text.to_string())
                .collect::<Vec<_>>()
                .join("，");
                let semantics = Semantics::new(Role::ListItem, label)
                    .id(format!("{owner}-item-{ix}"))
                    .position(ix + 1, items.len());
                root = root.child(semantics.apply(
                    item_element.id(gpui::ElementId::Name(format!("{owner}-item-{ix}").into())),
                ));
            }
            root
        })
    }
}
