//! 间距容器：水平 / 垂直排列子元素并统一间距（对应 antd Space）。

use gpui::{AnyElement, IntoElement, div, prelude::*, px};

use crate::theme::{self, metric::SpaceMetric};

/// 间距预设（对应 antd Space `size`）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpaceSize {
    #[default]
    Small,
    Middle,
    Large,
}

/// 预设间距：small 8 / middle 16 / large 24。
pub fn space_gap(tokens: &theme::Tokens, size: SpaceSize) -> f32 {
    let m = SpaceMetric::resolve(tokens);
    match size {
        SpaceSize::Small => m.small,
        SpaceSize::Middle => m.middle,
        SpaceSize::Large => m.large,
    }
}

/// 间距容器。
///
/// ```ignore
/// Space::new().child(Button::new("a")).child(Button::new("b"))
/// Space::new().vertical().size(12.).child("上").child("下")
/// ```
#[derive(IntoElement)]
pub struct Space {
    vertical: bool,
    wrap: bool,
    size: SpaceSize,
    custom_size: Option<f32>,
    align_center: bool,
    children: Vec<AnyElement>,
}

impl Space {
    pub fn new() -> Self {
        Self {
            vertical: false,
            wrap: false,
            size: SpaceSize::Small,
            custom_size: None,
            align_center: true,
            children: Vec::new(),
        }
    }

    /// 垂直排列
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }

    /// 自动换行（仅水平方向有效）
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    /// 间距（px，默认 8）
    pub fn size(mut self, size: f32) -> Self {
        self.custom_size = Some(size);
        self
    }

    /// 预设间距 small（8）
    pub fn small(mut self) -> Self {
        self.size = SpaceSize::Small;
        self.custom_size = None;
        self
    }

    /// 预设间距 middle（16）
    pub fn middle(mut self) -> Self {
        self.size = SpaceSize::Middle;
        self.custom_size = None;
        self
    }

    /// 预设间距 large（24）
    pub fn large(mut self) -> Self {
        self.size = SpaceSize::Large;
        self.custom_size = None;
        self
    }

    /// 顶端对齐（默认交叉轴居中）
    pub fn align_start(mut self) -> Self {
        self.align_center = false;
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

impl Default for Space {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Space {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let gap = self
            .custom_size
            .unwrap_or_else(|| theme::with(|tokens| space_gap(tokens, self.size)));
        let mut space = div().flex().gap(px(gap));
        space = if self.vertical {
            space.flex_col()
        } else {
            space.flex_row()
        };
        if self.wrap && !self.vertical {
            space = space.flex_wrap();
        }
        if self.align_center {
            space = space.items_center();
        } else {
            space = space.items_start();
        }
        space.children(self.children)
    }
}
