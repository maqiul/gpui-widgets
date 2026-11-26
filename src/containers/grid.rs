//! 栅格布局：24 列 Row / Col（对应 antd Grid）。
//!
//! gutter 语义与 antd 一致：Row 两侧负外边距 -g/2，Col 两侧内边距 g/2，
//! 因此 span 占比按「含 gutter 的行宽」计算。

use gpui::{AnyElement, IntoElement, Length, div, prelude::*, px, relative};

use crate::theme::metric;

/// 栅格列（span 1-24，占 24 分之 span 宽）
///
/// ```ignore
/// Row::new().gutter(16.).child(
///     Col::new(12).child("左半")
/// ).child(
///     Col::new(12).child("右半")
/// )
/// ```
#[derive(IntoElement)]
pub struct Col {
    span: u8,
    offset: u8,
    children: Vec<AnyElement>,
}

impl Col {
    pub fn new(span: u8) -> Self {
        Self {
            span: span.clamp(1, 24),
            offset: 0,
            children: Vec::new(),
        }
    }

    /// 左侧偏移列数
    pub fn offset(mut self, offset: u8) -> Self {
        self.offset = offset.min(23);
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

    fn render_with_gutter(self, gutter: f32) -> gpui::Div {
        let mut col = div()
            .relative()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .max_w(relative(1.))
            .min_h(px(1.));
        col.style().flex_grow = Some(0.);
        col.style().flex_basis = Some(Length::Definite(relative(metric::grid_col_basis(
            self.span as u32,
        ))));
        if self.offset > 0 {
            col.style().margin.left = Some(Length::Definite(relative(metric::grid_offset_margin(
                self.offset as u32,
            ))));
        }
        if gutter > 0. {
            col = col.px(px(metric::grid_col_gutter_padding(gutter)));
        }
        col.children(self.children)
    }
}

impl RenderOnce for Col {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.render_with_gutter(0.)
    }
}

/// 栅格行（水平排列 Col，gutter 为列间距）
#[derive(IntoElement)]
pub struct Row {
    gutter: f32,
    align_start: bool,
    cols: Vec<Col>,
}

impl Row {
    pub fn new() -> Self {
        Self {
            gutter: 0.,
            align_start: false,
            cols: Vec::new(),
        }
    }

    /// 列间距（px）：Row 负外边距 + Col 内边距各半（antd 语义）
    pub fn gutter(mut self, gutter: f32) -> Self {
        self.gutter = gutter;
        self
    }

    /// 顶端对齐（默认拉伸等高）
    pub fn align_start(mut self) -> Self {
        self.align_start = true;
        self
    }

    pub fn child(mut self, col: Col) -> Self {
        self.cols.push(col);
        self
    }

    pub fn children(mut self, cols: impl IntoIterator<Item = Col>) -> Self {
        self.cols.extend(cols);
        self
    }
}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Row {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        // 宽度 auto + grow：负外边距才能像 antd 块级行一样把行宽撑到 父宽+gutter
        let mut row = div().flex().flex_row().flex_wrap().min_w_0();
        row.style().flex_grow = Some(1.);
        if self.gutter > 0. {
            let margin = px(metric::grid_row_gutter_margin(self.gutter));
            row = row.ml(margin).mr(margin);
        }
        if self.align_start {
            row = row.items_start();
        }
        row.children(
            self.cols
                .into_iter()
                .map(|col| col.render_with_gutter(self.gutter)),
        )
    }
}
