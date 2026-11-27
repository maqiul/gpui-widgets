//! 弹性布局容器：对 gpui flex 风格的轻量封装，统一方向、间距、对齐与伸缩配置。

use gpui::{AnyElement, IntoElement, Length, Pixels, div, prelude::*};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Wrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Align {
    #[default]
    Stretch,
    Start,
    Center,
    End,
    Baseline,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// 通用弹性布局容器。
#[derive(IntoElement)]
pub struct Flex {
    vertical: bool,
    wrap: Wrap,
    gap: Option<Pixels>,
    align: Align,
    justify: Justify,
    grow: Option<f32>,
    shrink: Option<f32>,
    basis: Option<Length>,
    children: Vec<AnyElement>,
}

impl Flex {
    pub fn new() -> Self {
        Self {
            vertical: false,
            wrap: Wrap::NoWrap,
            gap: None,
            align: Align::Stretch,
            justify: Justify::Start,
            grow: None,
            shrink: None,
            basis: None,
            children: Vec::new(),
        }
    }

    pub fn row(mut self) -> Self {
        self.vertical = false;
        self
    }

    pub fn column(mut self) -> Self {
        self.vertical = true;
        self
    }

    pub fn wrap(mut self) -> Self {
        self.wrap = Wrap::Wrap;
        self
    }

    pub fn wrap_reverse(mut self) -> Self {
        self.wrap = Wrap::WrapReverse;
        self
    }

    pub fn nowrap(mut self) -> Self {
        self.wrap = Wrap::NoWrap;
        self
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = Some(gap.into());
        self
    }

    pub fn items_start(mut self) -> Self {
        self.align = Align::Start;
        self
    }

    pub fn items_center(mut self) -> Self {
        self.align = Align::Center;
        self
    }

    pub fn items_end(mut self) -> Self {
        self.align = Align::End;
        self
    }

    pub fn items_baseline(mut self) -> Self {
        self.align = Align::Baseline;
        self
    }

    pub fn items_stretch(mut self) -> Self {
        self.align = Align::Stretch;
        self
    }

    pub fn justify_start(mut self) -> Self {
        self.justify = Justify::Start;
        self
    }

    pub fn justify_center(mut self) -> Self {
        self.justify = Justify::Center;
        self
    }

    pub fn justify_end(mut self) -> Self {
        self.justify = Justify::End;
        self
    }

    pub fn justify_between(mut self) -> Self {
        self.justify = Justify::SpaceBetween;
        self
    }

    pub fn justify_around(mut self) -> Self {
        self.justify = Justify::SpaceAround;
        self
    }

    pub fn justify_evenly(mut self) -> Self {
        self.justify = Justify::SpaceEvenly;
        self
    }

    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = Some(grow);
        self
    }

    pub fn shrink(mut self, shrink: f32) -> Self {
        self.shrink = Some(shrink);
        self
    }

    pub fn basis(mut self, basis: impl Into<Length>) -> Self {
        self.basis = Some(basis.into());
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|child| child.into_any_element()));
        self
    }
}

impl Default for Flex {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Flex {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let mut flex = div().flex();
        flex = if self.vertical {
            flex.flex_col()
        } else {
            flex.flex_row()
        };
        flex = match self.wrap {
            Wrap::NoWrap => flex.flex_nowrap(),
            Wrap::Wrap => flex.flex_wrap(),
            Wrap::WrapReverse => flex.flex_wrap_reverse(),
        };
        if let Some(gap) = self.gap {
            flex = flex.gap(gap);
        }
        flex = match self.align {
            Align::Stretch => flex.items_stretch(),
            Align::Start => flex.items_start(),
            Align::Center => flex.items_center(),
            Align::End => flex.items_end(),
            Align::Baseline => flex.items_baseline(),
        };
        flex = match self.justify {
            Justify::Start => flex.justify_start(),
            Justify::Center => flex.justify_center(),
            Justify::End => flex.justify_end(),
            Justify::SpaceBetween => flex.justify_between(),
            Justify::SpaceAround => flex.justify_around(),
            Justify::SpaceEvenly => flex.justify_evenly(),
        };
        if let Some(grow) = self.grow {
            flex = flex.flex_grow(grow);
        }
        if let Some(shrink) = self.shrink {
            flex = flex.flex_shrink(shrink);
        }
        if let Some(basis) = self.basis {
            flex = flex.flex_basis(basis);
        }
        flex.children(self.children)
    }
}
