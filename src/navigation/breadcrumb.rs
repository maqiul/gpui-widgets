//! 面包屑导航：层级路径展示，点击中间层级跳转（末项为当前页，不可点击）。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, IntoElement, Role, SharedString, Stateful, Window, div,
    prelude::*, px,
};

use crate::a11y;
use crate::theme::{self, Tokens, metric};

/// 面包屑外观（antdv-next Breadcrumb 实测）：非末项与分隔符 `colorTextDescription`，
/// 末项 `colorText`，字号 14、行高 22，分隔符两侧各 8px。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreadcrumbStyle {
    pub link: Hsla,
    pub current: Hsla,
    pub separator: Hsla,
    pub font_size: f32,
    pub line_height: f32,
    pub gap: f32,
}

pub fn breadcrumb_state(tokens: &Tokens) -> BreadcrumbStyle {
    let m = metric::BreadcrumbMetric::resolve(tokens);
    BreadcrumbStyle {
        link: tokens.color_text_description,
        current: tokens.color_text,
        separator: tokens.color_text_description,
        font_size: m.font_size,
        line_height: m.line_height,
        gap: m.gap,
    }
}

/// 面包屑导航。
///
/// ```ignore
/// Breadcrumb::new(["首页", "产品中心", "控件库"])
///     .on_click(cx.listener(|this, ix, _, cx| { /* 跳转到第 ix 层 */ }))
/// ```
pub struct Breadcrumb {
    id: Option<SharedString>,
    origin: SharedString,
    items: Vec<SharedString>,
    separator: SharedString,
    on_click: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl Breadcrumb {
    #[track_caller]
    pub fn new(items: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "breadcrumb-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            items: items.into_iter().map(Into::into).collect(),
            separator: "/".into(),
            on_click: None,
        }
    }

    /// 实例 ID；同页多个面包屑应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 自定义分隔符（默认 "/"）
    pub fn separator(mut self, separator: impl Into<SharedString>) -> Self {
        self.separator = separator.into();
        self
    }

    /// 点击非末项时回调其下标
    pub fn on_click(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Breadcrumb {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let style = theme::with(breadcrumb_state);
        let owner = self.id.as_deref().unwrap_or(&self.origin);
        let root_id = format!("{owner}-navigation");
        let mut root = a11y::Semantics::new(Role::Navigation, "面包屑导航")
            .id(root_id.clone())
            .apply(div().id(ElementId::Name(root_id.into())))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(style.gap))
            .h(px(style.line_height))
            .text_size(px(style.font_size))
            .line_height(px(style.line_height));
        let last = self.items.len().saturating_sub(1);

        for (ix, item) in self.items.iter().enumerate() {
            if ix > 0 {
                root = root.child(
                    div()
                        .flex_shrink_0()
                        .text_color(style.separator)
                        .child(self.separator.clone()),
                );
            }
            if ix == last {
                root = root.child(
                    a11y::Semantics::new(Role::ListItem, item.clone())
                        .id(format!("{owner}-current-{ix}"))
                        .position(ix + 1, last + 1)
                        .apply(div().id(ElementId::Name(format!("{owner}-current-{ix}").into())))
                        .text_color(style.current)
                        .child(item.clone()),
                );
            } else {
                let mut crumb = a11y::Semantics::new(Role::Link, item.clone())
                    .id(format!("{owner}-item-{ix}"))
                    .focusable(self.on_click.is_some())
                    .position(ix + 1, last + 1)
                    .apply(div().id(ElementId::Name(format!("{owner}-item-{ix}").into())))
                    .text_color(style.link)
                    .child(item.clone());
                if let Some(on_click) = self.on_click.clone() {
                    crumb = crumb
                        .cursor(CursorStyle::PointingHand)
                        .hover(move |s| s.text_color(style.current));
                    crumb = a11y::click_void(crumb, move |window, cx| on_click(&ix, window, cx));
                }
                root = root.child(crumb);
            }
        }

        root
    }
}
