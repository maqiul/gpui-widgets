//! 漫游式引导：遮罩 + 居中步骤卡片（antd Tour 简化版）。
//!
//! 不做目标元素挖孔高亮与 primary 型；卡片几何/配色走 antd Tour 令牌。
//! 遮罩以 `absolute inset_0` 铺满父容器：应挂在 `relative` 的根视图或面板盒内。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, IntoElement, MouseButton, Role, SharedString, Window, div,
    prelude::*, px, relative,
};

use crate::Button;
use crate::a11y::{self, Semantics};
use crate::theme;
use crate::theme::metric::{tour_metric, tour_style};

/// 单个引导步骤
#[derive(Clone)]
pub struct TourStep {
    title: SharedString,
    description: SharedString,
}

impl TourStep {
    pub fn new(title: impl Into<SharedString>, description: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
        }
    }
}

/// 漫游式引导。
///
/// ```ignore
/// // render 中（relative 父容器内）：
/// if self.tour_open {
///     root = root.child(
///         Tour::new(steps)
///             .current(self.tour_step)
///             .on_change(cx.listener(|this, step, _, cx| { this.tour_step = *step; cx.notify(); }))
///             .on_close({
///                 let entity = cx.entity();
///                 move |_, cx| { let _ = entity.update(cx, |this, cx| { this.tour_open = false; cx.notify(); }); }
///             }),
///     );
/// }
/// ```
pub struct Tour {
    steps: Vec<TourStep>,
    current: usize,
    id: Option<SharedString>,
    on_change: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
    on_close: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl Tour {
    pub fn new(steps: Vec<TourStep>) -> Self {
        Self {
            steps,
            current: 0,
            id: None,
            on_change: None,
            on_close: None,
        }
    }

    /// 实例唯一 id；不传则回退第一步标题，无步骤时用 "empty"。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 当前步骤下标（受控）
    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }

    /// 上一步 / 下一步时回调新下标
    pub fn on_change(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    /// 关闭钮 / 完成 / 点击遮罩时回调
    pub fn on_close(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(listener));
        self
    }
}

/// 派生漫游遮罩、卡片与关闭钮 id：实例 id 优先，未传则回退第一步标题。
fn element_ids(id: Option<&str>, first_title: Option<&str>) -> (String, String, String) {
    let key = id.unwrap_or_else(|| first_title.unwrap_or("empty"));
    (
        format!("tour-mask-{key}"),
        format!("tour-card-{key}"),
        format!("tour-close-{key}"),
    )
}

impl IntoElement for Tour {
    type Element = gpui::Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let (mask_id, card_id, close_id) = element_ids(
            self.id.as_deref(),
            self.steps.first().map(|step| step.title.as_ref()),
        );
        let m = theme::with(tour_metric);
        let style = theme::with(tour_style);
        let total = self.steps.len();
        let current = self.current.min(total.saturating_sub(1));
        let step = self.steps.get(current).cloned();

        let mut mask = div()
            .id(ElementId::Name(mask_id.into()))
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .items_center()
            .justify_center()
            .bg(style.mask);
        if let Some(on_close) = self.on_close.clone() {
            mask = mask.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                on_close(window, cx);
            });
        }

        let Some(step) = step else {
            return mask;
        };
        let card_semantics = Semantics::new(Role::Dialog, step.title.clone())
            .id(card_id.clone())
            .description(step.description.clone())
            .value(format!("第 {} 步，共 {} 步", current + 1, total))
            .focusable(true);

        let mut card = div()
            .id(ElementId::Name(card_id.into()))
            .relative()
            .flex()
            .flex_col()
            // antd fontFamily 在 Windows 首选 Segoe UI；gpui 默认取系统 UI 字体
            // （中文系统为 Microsoft YaHei，宽 ~12%），卡片宽度随内容走，须钉字体才对齐
            .font_family("Segoe UI")
            // antd tour：width 520 + maxWidth fit-content → 实际随内容收缩；
            // gpui 文本 min-content≈max-content，须显式 min_w(0) 才能让外层半宽夹住
            .max_w(px(m.width))
            .min_w(px(0.))
            .rounded(px(m.radius))
            .bg(style.bg)
            .shadow(style.shadow.clone())
            // 卡片内的点击不冒泡到遮罩，避免误关闭
            .on_mouse_down(MouseButton::Left, |_, _, cx: &mut App| {
                cx.stop_propagation()
            })
            .child(
                div()
                    .pt(px(m.padding))
                    .pl(px(m.padding))
                    .pb(px(m.padding_xs))
                    .pr(px(m.padding + m.close_size))
                    .text_size(px(m.text_size))
                    .line_height(px(m.text_line_height))
                    .font_weight(gpui::FontWeight(m.title_weight))
                    .text_color(style.text)
                    .child(step.title.clone()),
            )
            .child(
                div()
                    .px(px(m.padding))
                    .text_size(px(m.text_size))
                    .line_height(px(m.text_line_height))
                    .text_color(style.text)
                    .child(step.description.clone()),
            );

        if let Some(on_close) = self.on_close.clone() {
            let close = div()
                .id(ElementId::Name(close_id.clone().into()))
                .absolute()
                .top(px(m.padding))
                .right(px(m.padding))
                .w(px(m.close_size))
                .h(px(m.close_size))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(m.close_radius))
                .text_size(px(m.close_icon))
                .line_height(px(m.close_size))
                .text_color(style.close)
                .cursor(CursorStyle::PointingHand)
                .hover(move |s| s.text_color(style.close_hover).bg(style.close_hover_bg))
                .active(move |s| s.bg(style.close_active_bg))
                .child("✕");
            let close = a11y::click_void(close, move |window, cx| on_close(window, cx));
            let close = Semantics::new(Role::Button, "关闭引导")
                .id(close_id)
                .focusable(true)
                .apply(close);
            card = card.child(close);
        }

        // 底部：指示点 + 按钮（antd footer：paddingXS padding padding，actions 右对齐）
        let mut footer = div()
            .flex()
            .flex_row()
            .items_center()
            .pt(px(m.padding_xs))
            .px(px(m.padding))
            .pb(px(m.padding));
        if total > 1 {
            let mut dots = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(m.indicator_gap));
            for i in 0..total {
                dots = dots.child(
                    div()
                        .w(px(m.indicator))
                        .h(px(m.indicator))
                        .rounded_full()
                        .bg(if i == current {
                            style.indicator_active
                        } else {
                            style.indicator
                        }),
                );
            }
            footer = footer.child(dots);
        }

        let mut actions = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(m.btn_gap))
            .pl(px(m.btn_gap))
            .ml_auto();
        if current > 0 {
            let on_change = self.on_change.clone();
            let prev = current - 1;
            actions = actions.child(Button::new("Previous").small().on_click(
                move |_, window, cx| {
                    if let Some(on_change) = &on_change {
                        on_change(&prev, window, cx);
                    }
                },
            ));
        }
        let is_last = current + 1 >= total;
        let on_change = self.on_change.clone();
        let on_close = self.on_close.clone();
        let next = current + 1;
        let next_btn = Button::new(if is_last { "Finish" } else { "Next" })
            .small()
            .primary();
        actions = actions.child(next_btn.on_click(move |_, window, cx| {
            if is_last {
                if let Some(on_close) = &on_close {
                    on_close(window, cx);
                }
            } else if let Some(on_change) = &on_change {
                on_change(&next, window, cx);
            }
        }));
        footer = footer.child(actions);

        card = card.child(footer);
        let card = card_semantics.apply(card);
        // antd .ant-tour 绝对定位 left 50% + maxWidth fit-content：
        // 可用宽 = 容器宽 − 50%，卡宽 = min(520, max-content, 半宽)；用半宽外层复现
        mask.child(div().w(relative(0.5)).flex().justify_center().child(card))
    }
}

#[cfg(test)]
mod tests {
    use super::element_ids;

    #[test]
    fn tour_id_default_derives_from_first_title() {
        let (mask, card, close) = element_ids(None, Some("欢迎"));
        assert_eq!(mask, "tour-mask-欢迎");
        assert_eq!(card, "tour-card-欢迎");
        assert_eq!(close, "tour-close-欢迎");
    }

    #[test]
    fn tour_id_empty_steps_use_empty_key() {
        let (mask, card, close) = element_ids(None, None);
        assert_eq!(mask, "tour-mask-empty");
        assert_eq!(card, "tour-card-empty");
        assert_eq!(close, "tour-close-empty");
    }

    #[test]
    fn tour_id_stable_across_repeated_calls() {
        for _ in 0..3 {
            assert_eq!(
                element_ids(None, Some("欢迎")),
                element_ids(None, Some("欢迎"))
            );
            assert_eq!(
                element_ids(Some("onboarding"), Some("欢迎")),
                element_ids(Some("onboarding"), Some("欢迎"))
            );
        }
    }

    #[test]
    fn tour_id_distinct_first_titles_do_not_collide() {
        let welcome = element_ids(None, Some("欢迎"));
        let editor = element_ids(None, Some("编辑器"));
        assert_ne!(welcome.0, editor.0);
        assert_ne!(welcome.1, editor.1);
        assert_ne!(welcome.2, editor.2);
    }

    #[test]
    fn tour_id_explicit_overrides_first_title() {
        let (mask, card, close) = element_ids(Some("onboarding-7"), Some("欢迎"));
        assert_eq!(mask, "tour-mask-onboarding-7");
        assert_eq!(card, "tour-card-onboarding-7");
        assert_eq!(close, "tour-close-onboarding-7");
    }

    #[test]
    fn tour_id_parts_share_key_with_distinct_suffixes() {
        let ids = element_ids(Some("k"), Some("欢迎"));
        assert_eq!(ids.0.strip_prefix("tour-mask-"), Some("k"));
        assert_eq!(ids.1.strip_prefix("tour-card-"), Some("k"));
        assert_eq!(ids.2.strip_prefix("tour-close-"), Some("k"));
        assert_ne!(ids.0, ids.1);
        assert_ne!(ids.1, ids.2);
        assert_ne!(ids.0, ids.2);
    }

    #[test]
    fn tour_id_same_first_title_still_collides() {
        // 同标题漫游叠层必须由宿主传 .id()，内容 key 无法区分实例。
        assert_eq!(
            element_ids(None, Some("欢迎")),
            element_ids(None, Some("欢迎"))
        );
        assert_ne!(
            element_ids(Some("a"), Some("欢迎")),
            element_ids(Some("b"), Some("欢迎"))
        );
    }
}
