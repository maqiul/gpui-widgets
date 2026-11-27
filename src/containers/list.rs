//! 列表：带头像 / 描述 / 额外元素的条目列表（对应 antd List）。

use std::rc::Rc;

use gpui::{
    AnyElement, App, Div, ElementId, IntoElement, Pixels, Role, SharedString, Stateful,
    UniformListScrollHandle, Window, div, prelude::*, px, uniform_list,
};

use crate::a11y;
use crate::scrollbar::Scrollbar;
use crate::theme;

/// 列表条目
pub struct ListItem {
    title: SharedString,
    description: Option<SharedString>,
    avatar: Option<SharedString>,
    extra: Option<AnyElement>,
}

impl ListItem {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            avatar: None,
            extra: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 左侧圆形头像（字符 / emoji）
    pub fn avatar(mut self, avatar: impl Into<SharedString>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }

    /// 右侧额外元素
    pub fn extra(mut self, extra: impl IntoElement) -> Self {
        self.extra = Some(extra.into_any_element());
        self
    }
}

/// 虚拟列表配置：固定视口 / 统一行高 / 按索引生成行
struct VirtualList {
    height: Pixels,
    row_height: Pixels,
    count: usize,
    row: Rc<dyn Fn(usize) -> ListItem>,
    scroll: Option<UniformListScrollHandle>,
}

/// 通用列表。
///
/// ```ignore
/// List::new(vec![
///     ListItem::new("条目一").description("说明文字").avatar("①"),
///     ListItem::new("条目二").extra(Badge::new("新").info()),
/// ])
/// .header("列表标题")
/// ```
#[derive(IntoElement)]
pub struct List {
    id: Option<SharedString>,
    origin: SharedString,
    header: Option<SharedString>,
    footer: Option<SharedString>,
    items: Vec<ListItem>,
    virtualized: Option<VirtualList>,
}

impl List {
    #[track_caller]
    pub fn new(items: Vec<ListItem>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "list-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            header: None,
            footer: None,
            items,
            virtualized: None,
        }
    }

    /// 虚拟列表：只渲染视口内的行，适合万级数据。
    ///
    /// - `height`：表体视口高度（不含头部/底部）；
    /// - `row_height`：统一行高。虚拟列表按统一行高布局，两行（含 description）
    ///   或带头像的行请显式给足高度（建议 ≥ 56px），否则内容可能被裁切；
    /// - `count` + `row`：按全局行下标生成条目，`row` 会在滚动/测量时被多次调用。
    #[track_caller]
    pub fn virtualized(
        height: impl Into<Pixels>,
        row_height: impl Into<Pixels>,
        count: usize,
        row: impl Fn(usize) -> ListItem + 'static,
    ) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "list-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            header: None,
            footer: None,
            items: Vec::new(),
            virtualized: Some(VirtualList {
                height: height.into(),
                row_height: row_height.into(),
                count,
                row: Rc::new(row),
                scroll: None,
            }),
        }
    }

    /// 复用外部滚动句柄（可选）：可据此做回到顶部、滚到某行等控制。
    pub fn virtual_scroll_handle(mut self, handle: UniformListScrollHandle) -> Self {
        if let Some(virtualized) = &mut self.virtualized {
            virtualized.scroll = Some(handle);
        }
        self
    }

    /// 实例 ID；同页多个列表应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn footer(mut self, footer: impl Into<SharedString>) -> Self {
        self.footer = Some(footer.into());
        self
    }
}

impl RenderOnce for List {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (m, padding_sm, padding) = theme::with(|t| {
            (
                crate::theme::metric::list_metric(t),
                t.padding_sm,
                t.padding,
            )
        });
        let list_id = self
            .id
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| self.origin.to_string());
        let item_prefix = list_id.clone();
        let mut list = a11y::Semantics::new(
            Role::List,
            self.header.clone().unwrap_or_else(|| "列表".into()),
        )
        .id(list_id.clone())
        .apply(div().id(ElementId::Name(list_id.clone().into())))
        .flex()
        .flex_col()
        .w_full()
        .bg(theme::bg_container())
        .border_1()
        .border_color(theme::border())
        .rounded_lg()
        .overflow_hidden();

        if let Some(header) = &self.header {
            list = list.child(
                a11y::Semantics::new(Role::Heading, header.clone())
                    .id(format!("{list_id}-heading"))
                    .level(2)
                    .apply(div().id(ElementId::Name(format!("{list_id}-heading").into())))
                    .px(px(padding))
                    .py(px(padding_sm))
                    .border_b_1()
                    .border_color(theme::border())
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme::text())
                    .child(header.clone()),
            );
        }

        if let Some(virt) = self.virtualized {
            let count = virt.count;
            let row_height = virt.row_height;
            let row = virt.row.clone();
            let scroll = virt.scroll.clone().unwrap_or_else(|| {
                window
                    .use_keyed_state(
                        ElementId::Name(format!("{list_id}-vscroll").into()),
                        cx,
                        |_, _| UniformListScrollHandle::new(),
                    )
                    .read(cx)
                    .clone()
            });
            let bar = Scrollbar::vertical(
                ElementId::Name(format!("{list_id}-vbar").into()),
                scroll.0.borrow().base_handle.clone(),
            );
            let prefix = item_prefix.clone();
            let rows = uniform_list(
                ElementId::Name(format!("{list_id}-vbody").into()),
                count,
                move |range, _, _| {
                    range
                        .map(|ix| {
                            row_element(
                                row(ix),
                                ix,
                                count,
                                &prefix,
                                Some(row_height),
                                m.avatar_size,
                                padding,
                                padding_sm,
                            )
                        })
                        .collect::<Vec<_>>()
                },
            )
            .size_full()
            .track_scroll(&scroll);
            list = list.child(
                div()
                    .relative()
                    .w_full()
                    .h(virt.height)
                    .overflow_hidden()
                    .child(rows)
                    .child(div().absolute().top_0().right_0().h_full().child(bar)),
            );
        } else {
            let count = self.items.len();
            for (ix, item) in self.items.into_iter().enumerate() {
                list = list.child(row_element(
                    item,
                    ix,
                    count,
                    &item_prefix,
                    None,
                    m.avatar_size,
                    padding,
                    padding_sm,
                ));
            }
        }

        if let Some(footer) = &self.footer {
            list = list.child(
                div()
                    .px(px(padding))
                    .py(px(m.footer_padding_block))
                    .border_t_1()
                    .border_color(theme::border())
                    .text_xs()
                    .text_color(theme::text_secondary())
                    .child(footer.clone()),
            );
        }

        list
    }
}

#[allow(clippy::too_many_arguments)]
fn row_element(
    item: ListItem,
    ix: usize,
    count: usize,
    item_prefix: &str,
    height: Option<Pixels>,
    avatar_size: f32,
    padding: f32,
    padding_sm: f32,
) -> Stateful<Div> {
    let item_id = format!("{item_prefix}-item-{ix}");
    let mut semantics = a11y::Semantics::new(Role::ListItem, item.title.clone())
        .id(item_id.clone())
        .position(ix + 1, count);
    if let Some(description) = &item.description {
        semantics = semantics.description(description.clone());
    }
    let mut row = semantics
        .apply(div().id(ElementId::Name(item_id.into())))
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px(px(padding))
        .py(px(padding_sm));
    if ix < count - 1 {
        row = row.border_b_1().border_color(theme::border());
    }
    if let Some(height) = height {
        row = row.h(height).overflow_hidden();
    }

    if let Some(avatar) = &item.avatar {
        row = row.child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(avatar_size))
                .flex_shrink_0()
                .rounded_full()
                .bg(theme::primary_bg())
                .text_color(theme::primary())
                .text_sm()
                .child(avatar.clone()),
        );
    }

    let mut body = div().flex().flex_col().gap_1().flex_1().child(
        div()
            .text_sm()
            .text_color(theme::text())
            .child(item.title.clone()),
    );
    if let Some(description) = &item.description {
        body = body.child(
            div()
                .text_xs()
                .text_color(theme::text_secondary())
                .child(description.clone()),
        );
    }
    row = row.child(body);

    if let Some(extra) = item.extra {
        row = row.child(extra);
    }
    row
}
