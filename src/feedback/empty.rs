//! 空状态：antd Empty 令牌与默认 / simple 两套插画（SVG 精确绘制）。

use gpui::{
    AnyElement, Div, IntoElement, Role, SharedString, Stateful, canvas, div, prelude::*, px,
};

use crate::a11y::Semantics;
use crate::svg_path::{ellipse, paint_svg, paint_svg_stroke, svg_segments, translate_segs};
use crate::theme;
use crate::theme::metric::{empty_metric, empty_style};

const DEFAULT_VIEW: (f32, f32, f32, f32) = (0., 0., 184., 152.);
const SIMPLE_VIEW: (f32, f32, f32, f32) = (0., 0., 64., 41.);

const BOX_BORDER: &str =
    "M122 69.7 98.1 40.2a6 6 0 0 0-4.6-2.2H42.1a6 6 0 0 0-4.6 2.2l-24 29.5V85H122z";
const BOX_PANEL: &str =
    "M33.8 0h68a4 4 0 0 1 4 4v93.3a4 4 0 0 1-4 4h-68a4 4 0 0 1-4-4V4a4 4 0 0 1 4-4";
const BOX_DETAIL: &str = "M42.7 10h50.2a2 2 0 0 1 2 2v25a2 2 0 0 1-2 2H42.7a2 2 0 0 1-2-2V12a2 2 0 0 1 2-2m.2 39.8h49.8a2.3 2.3 0 1 1 0 4.5H42.9a2.3 2.3 0 0 1 0-4.5m0 11.7h49.8a2.3 2.3 0 1 1 0 4.6H42.9a2.3 2.3 0 0 1 0-4.6m79 43.5a7 7 0 0 1-6.8 5.4H20.5a7 7 0 0 1-6.7-5.4l-.2-1.8V69.7h26.3c2.9 0 5.2 2.4 5.2 5.4s2.4 5.4 5.3 5.4h34.8c2.9 0 5.3-2.4 5.3-5.4s2.3-5.4 5.2-5.4H122v33.5q0 1-.2 1.8";
const BUBBLE: &str = "m149.1 33.3-6.8 2.6a1 1 0 0 1-1.3-1.2l2-6.2q-4.1-4.5-4.2-10.4c0-10 10.1-18.1 22.6-18.1S184 8.1 184 18.1s-10.1 18-22.6 18q-6.8 0-12.3-2.8";
const ICON_DOTS: &str = "M5.7 5.6H0L2.9.7zM9.3.7h5v5h-5z";

const SIMPLE_BOX: &str = "M55 12.76L44.854 1.258C44.367.474 43.656 0 42.907 0H21.093c-.749 0-1.46.474-1.947 1.257L9 12.761V22h46v-9.24z";
const SIMPLE_CONTENT: &str = "M41.613 15.931c0-1.605.994-2.93 2.227-2.931H55v18.137C55 33.26 53.68 35 52.05 35h-40.1C10.32 35 9 33.259 9 31.137V13h11.16c1.233 0 2.227 1.323 2.227 2.928v.022c0 1.605 1.005 2.901 2.237 2.901h14.752c1.232 0 2.237-1.308 2.237-2.913v-.007z";

/// 空状态占位。
///
/// ```ignore
/// Empty::new("暂无数据")
///     .action(Button::new("新建").primary())
/// ```
pub struct Empty {
    title: SharedString,
    description: Option<SharedString>,
    action: Option<AnyElement>,
    simple: bool,
    small: bool,
}

impl Empty {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            action: None,
            simple: false,
            small: false,
        }
    }

    /// 附加描述行（antd 单 description 之外的扩展行）
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 底部操作区（如「新建」按钮）
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    /// simple 插画（antd PRESENTED_IMAGE_SIMPLE，-normal 档）
    pub fn simple(mut self) -> Self {
        self.simple = true;
        self
    }

    /// -small 档插画高度
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }
}

impl IntoElement for Empty {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let m = theme::with(empty_metric);
        let style = theme::with(empty_style);
        let img_height = if self.simple {
            m.img_height_md
        } else if self.small {
            m.img_height_sm
        } else {
            m.img_height
        };
        let margin_block = if self.simple {
            m.margin_block_normal
        } else if self.small {
            m.margin_block_small
        } else {
            0.
        };
        let (view, aspect) = if self.simple {
            (SIMPLE_VIEW, 64. / 41.)
        } else {
            (DEFAULT_VIEW, 184. / 152.)
        };
        let simple = self.simple;
        let shadow_opacity = m.shadow_opacity;
        let empty_id = format!("empty-{}", self.title);
        let mut semantics = Semantics::new(Role::Status, self.title.clone()).id(empty_id.clone());
        if let Some(description) = &self.description {
            semantics = semantics.description(description.clone());
        }

        let image = canvas(
            |bounds, _, _| bounds,
            move |bounds, _, window, _| {
                if simple {
                    let shadow = translate_segs(&ellipse(32., 33., 32., 7.), 0., 1.);
                    paint_svg(bounds, view, &shadow, style.simple_shadow, window);
                    let box_segs = translate_segs(&svg_segments(SIMPLE_BOX), 0., 1.);
                    paint_svg_stroke(bounds, view, &box_segs, style.simple_border, 1., window);
                    let content = translate_segs(&svg_segments(SIMPLE_CONTENT), 0., 1.);
                    paint_svg(bounds, view, &content, style.simple_content, window);
                    paint_svg_stroke(bounds, view, &content, style.simple_border, 1., window);
                } else {
                    let shadow = translate_segs(&ellipse(67.8, 106.9, 67.8, 12.7), 24., 31.7);
                    paint_svg(
                        bounds,
                        view,
                        &shadow,
                        style.shadow.opacity(shadow_opacity),
                        window,
                    );
                    let border = translate_segs(&svg_segments(BOX_BORDER), 24., 31.7);
                    paint_svg(bounds, view, &border, style.border, window);
                    let panel = translate_segs(&svg_segments(BOX_PANEL), 24., 31.7);
                    paint_svg(bounds, view, &panel, style.panel_bg, window);
                    let detail = translate_segs(&svg_segments(BOX_DETAIL), 24., 31.7);
                    paint_svg(bounds, view, &detail, style.detail, window);
                    paint_svg(bounds, view, &svg_segments(BUBBLE), style.detail, window);
                    let dots = translate_segs(&svg_segments(ICON_DOTS), 149.7, 15.4);
                    paint_svg(bounds, view, &dots, style.icon, window);
                    let circle = translate_segs(&ellipse(20.7, 3.2, 2.8, 2.8), 149.7, 15.4);
                    paint_svg(bounds, view, &circle, style.icon, window);
                }
            },
        )
        .w(px(img_height * aspect))
        .h(px(img_height));

        let mut root = div()
            .id(gpui::ElementId::Name(empty_id.into()))
            .flex()
            .flex_col()
            .items_center()
            .mx(px(m.margin_inline))
            .my(px(margin_block))
            .child(
                div()
                    .h(px(img_height))
                    .mb(px(m.img_margin_bottom))
                    .child(image),
            )
            .child(
                div()
                    .text_size(px(m.desc_size))
                    .line_height(px(m.desc_line_height))
                    .text_color(style.desc)
                    .child(self.title),
            );

        if let Some(description) = self.description {
            root = root.child(
                div()
                    .text_size(px(m.desc_size))
                    .line_height(px(m.desc_line_height))
                    .text_color(style.desc)
                    .child(description),
            );
        }
        if let Some(action) = self.action {
            root = root.child(div().mt(px(m.footer_margin_top)).child(action));
        }

        semantics.apply(root)
    }
}
