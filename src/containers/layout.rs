//! 页面布局容器：Layout / Header / Sider / Content / Footer（对应 antd Layout）。
//!
//! 语义化容器，组合使用：
//! ```ignore
//! Layout::new()
//!     .child(Header::new().child("顶栏"))
//!     .child(Layout::new().has_sider()
//!         .child(Sider::new(200.).child("侧栏"))
//!         .child(Content::new().child("正文")))
//!     .child(Footer::new().child("页脚"))
//! ```

use gpui::{AnyElement, ElementId, IntoElement, Role, div, prelude::*, px};

use crate::a11y;
use crate::theme::{self, metric};

/// 布局根容器（底 colorBgLayout；`has_sider` 时横向排列）。
#[derive(IntoElement)]
pub struct Layout {
    has_sider: bool,
    children: Vec<AnyElement>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            has_sider: false,
            children: Vec::new(),
        }
    }

    /// 含侧栏时改为横向 flex（antd `hasSider`）
    pub fn has_sider(mut self) -> Self {
        self.has_sider = true;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Layout {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let bg = theme::with(|t| metric::layout_metric(t).body_bg);
        let mut root = a11y::Semantics::new(Role::Region, "页面布局")
            .id("layout")
            .apply(div().id(ElementId::Name("layout".into())))
            .flex()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .bg(bg);
        root = if self.has_sider {
            root.flex_row()
        } else {
            root.flex_col()
        };
        root.children(self.children)
    }
}

/// 顶栏（高 controlHeight×2，深navy 底，横向内边距 controlHeightLG×1.25）
#[derive(IntoElement)]
pub struct Header {
    children: Vec<AnyElement>,
}

impl Header {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Header {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let m = theme::with(|t| metric::layout_metric(t));
        a11y::Semantics::new(Role::Banner, "页眉")
            .id("layout-header")
            .apply(div().id(ElementId::Name("layout-header".into())))
            .flex()
            .flex_row()
            .items_center()
            .h(px(m.header_height))
            .flex_shrink_0()
            .px(px(m.header_padding_inline))
            .bg(m.header_bg)
            .text_color(m.header_color)
            .text_size(px(m.font_size))
            .line_height(px(m.header_height))
            .children(self.children)
    }
}

/// 侧栏（固定宽度，默认深navy 底；`light()` 换 colorBgContainer）
#[derive(IntoElement)]
pub struct Sider {
    width: f32,
    light: bool,
    children: Vec<AnyElement>,
}

impl Sider {
    pub fn new(width: f32) -> Self {
        Self {
            width,
            light: false,
            children: Vec::new(),
        }
    }

    /// 浅色侧栏（antd `theme="light"`）
    pub fn light(mut self) -> Self {
        self.light = true;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for Sider {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let m = theme::with(|t| metric::layout_metric(t));
        a11y::Semantics::new(Role::Complementary, "侧栏")
            .id("layout-sider")
            .apply(div().id(ElementId::Name("layout-sider".into())))
            .relative()
            .flex()
            .flex_col()
            .w(px(self.width))
            .min_w_0()
            .flex_shrink_0()
            .bg(if self.light {
                m.light_sider_bg
            } else {
                m.sider_bg
            })
            .text_size(px(m.font_size))
            .text_color(m.header_color)
            .children(self.children)
    }
}

/// 内容区（撑满剩余空间，无自带内边距与底色）
#[derive(IntoElement)]
pub struct Content {
    children: Vec<AnyElement>,
}

impl Content {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl Default for Content {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Content {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let color = theme::with(|t| metric::layout_metric(t).footer_color);
        a11y::Semantics::new(Role::Main, "主内容")
            .id("layout-content")
            .apply(div().id(ElementId::Name("layout-content".into())))
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .text_color(color)
            .children(self.children)
    }
}

/// 页脚（纵向 controlHeightSM / 横向 controlHeightLG×1.25 内边距，colorBgLayout 底）
#[derive(IntoElement)]
pub struct Footer {
    children: Vec<AnyElement>,
}

impl Footer {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl Default for Footer {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Footer {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let m = theme::with(|t| metric::layout_metric(t));
        a11y::Semantics::new(Role::ContentInfo, "页脚")
            .id("layout-footer")
            .apply(div().id(ElementId::Name("layout-footer".into())))
            .flex_shrink_0()
            .py(px(m.footer_padding_block))
            .px(px(m.footer_padding_inline))
            .text_size(px(m.footer_font_size))
            .text_color(m.footer_color)
            .bg(m.footer_bg)
            .children(self.children)
    }
}
