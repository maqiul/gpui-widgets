//! 轮播图：左右切换 + 底部圆点指示器，受控当前页。

use std::rc::Rc;

use gpui::{
    AnyElement, App, CursorStyle, ImageSource, IntoElement, ParentElement, Pixels, RenderOnce,
    Role, SharedString, Styled, Window, div, prelude::*, px,
};

use crate::{
    ImageView, a11y, theme,
    theme::metric::{CarouselMetric, carousel_style},
};

/// 轮播页内容：图片或任意元素（纯色块等）
enum Slide {
    Image(ImageSource),
    Element(AnyElement),
}

/// 轮播图（受控）。
///
/// ```ignore
/// Carousel::new("banner", vec![path1.into(), path2.into()])
///     .current(self.banner_page)
///     .on_change(cx.listener(|this, page: &usize, _, cx| {
///         this.banner_page = *page; cx.notify();
///     }))
/// ```
#[derive(IntoElement)]
pub struct Carousel {
    id: SharedString,
    slides: Vec<Slide>,
    current: usize,
    height: Pixels,
    on_change: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl Carousel {
    pub fn new(id: impl Into<SharedString>, sources: Vec<ImageSource>) -> Self {
        Self {
            id: id.into(),
            slides: sources.into_iter().map(Slide::Image).collect(),
            current: 0,
            height: px(200.),
            on_change: None,
        }
    }

    /// 以任意元素作为轮播页（纯色块、图文混排等）
    pub fn with_slides(id: impl Into<SharedString>, slides: Vec<AnyElement>) -> Self {
        Self {
            id: id.into(),
            slides: slides.into_iter().map(Slide::Element).collect(),
            current: 0,
            height: px(200.),
            on_change: None,
        }
    }

    /// 当前页（0-based，越界自动夹取）
    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }

    /// 展示高度
    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for Carousel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let m = theme::with(CarouselMetric::resolve);
        let style = theme::with(carousel_style);
        let count = self.slides.len();
        if count == 0 {
            return div()
                .id(self.id.clone())
                .w_full()
                .h(self.height)
                .bg(theme::hover_bg())
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(theme::text_muted())
                .child("暂无图片");
        }

        let current = self.current.min(count - 1);
        let on_change = self.on_change.clone();
        let id = self.id.clone();

        let slide = match self.slides.into_iter().nth(current).unwrap() {
            Slide::Image(source) => ImageView::new(source)
                .alt(SharedString::from(format!(
                    "轮播图第 {} 张，共 {} 张",
                    current + 1,
                    count
                )))
                .fit_cover()
                .w_full()
                .h_full()
                .into_any_element(),
            Slide::Element(el) => el,
        };
        let semantics = a11y::Semantics::new(Role::Group, "轮播图")
            .id(format!("{id}-a11y"))
            .value(SharedString::from(format!(
                "第 {} 张，共 {} 张",
                current + 1,
                count
            )));
        let mut root = semantics.apply(
            div()
                .id(id.clone())
                .relative()
                .w_full()
                .h(self.height)
                .overflow_hidden()
                .child(slide),
        );

        if count > 1 {
            // 左右箭头：16×16、距边 marginXS、白色 40% 不透明度（antd 无圆底）
            for (dir, label, left) in [(-1i32, "‹", true), (1i32, "›", false)] {
                let target = (current as i32 + dir).rem_euclid(count as i32) as usize;
                let arrow_color = style.arrow.opacity(m.arrow_opacity);
                let button_label = if left { "上一张" } else { "下一张" };
                let semantics = a11y::Semantics::new(Role::Button, button_label)
                    .id(format!(
                        "{id}-nav-{}-a11y",
                        if left { "prev" } else { "next" }
                    ))
                    .focusable(on_change.is_some());
                let mut button = semantics.apply(
                    div()
                        .id(SharedString::from(format!(
                            "{id}-nav-{}",
                            if left { "prev" } else { "next" }
                        )))
                        .absolute()
                        .top_1_2()
                        .mt(px(-m.arrow_size / 2.))
                        .w(px(m.arrow_size))
                        .h(px(m.arrow_size))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(m.arrow_size))
                        .line_height(px(m.arrow_size))
                        .text_color(arrow_color)
                        .cursor(CursorStyle::PointingHand)
                        .hover(move |s| s.text_color(style.arrow))
                        .child(label),
                );
                if let Some(on_change) = on_change.clone() {
                    button = a11y::click_void(button, move |window, cx| {
                        on_change(&target, window, cx);
                    });
                }
                button = if left {
                    button.left(px(m.arrow_offset))
                } else {
                    button.right(px(m.arrow_offset))
                };
                root = root.child(button);
            }

            // 底部指示条：16×3 / 选中 24×3、间距 2*dotGap、距底 dotOffset
            let mut dots = div()
                .absolute()
                .bottom(px(m.dot_offset))
                .left_0()
                .right_0()
                .flex()
                .justify_center()
                .gap(px(m.dot_gap * 2.));
            for ix in 0..count {
                let active = ix == current;
                let dot_color = style.dot.opacity(if active { 1. } else { m.dot_opacity });
                let hover_color = style
                    .dot
                    .opacity(if active { 1. } else { m.dot_hover_opacity });
                let semantics = a11y::Semantics::new(
                    Role::RadioButton,
                    SharedString::from(format!("第 {} 张", ix + 1)),
                )
                .id(format!("{id}-dot-{ix}-a11y"))
                .selected(active)
                .position(ix + 1, count)
                .focusable(on_change.is_some());
                let mut dot = semantics.apply(
                    div()
                        .id(SharedString::from(format!("{id}-dot-{ix}")))
                        .w(px(if active {
                            m.dot_active_width
                        } else {
                            m.dot_width
                        }))
                        .h(px(m.dot_height))
                        .rounded(px(m.dot_height))
                        .bg(dot_color)
                        .cursor(CursorStyle::PointingHand)
                        .hover(move |s| s.bg(hover_color)),
                );
                if let Some(on_change) = on_change.clone() {
                    dot = a11y::click_void(dot, move |window, cx| on_change(&ix, window, cx));
                }
                dots = dots.child(dot);
            }
            root = root.child(dots);
        }

        root
    }
}
