//! 悬浮按钮：固定在容器右下角的圆形按钮（对应 antd FloatButton）。
//!
//! 需放在 `relative` 定位的容器内，用 `bottom` / `right` 调整偏移
//! （默认 bottom 48 / right 24，即 antd 的 marginXXL / marginLG）。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, ElementId, IntoElement, Role, SharedString, Window, div, prelude::*, px,
};

use crate::a11y;
use crate::{button, theme, tooltip};

/// 悬浮按钮。
///
/// ```ignore
/// // 根容器需 .relative()
/// root.child(
///     FloatButton::new("fb", "↑")
///         .primary()
///         .on_click(cx.listener(|this, _, cx| { /* 回到顶部等 */ }))
/// )
/// ```
#[derive(IntoElement)]
pub struct FloatButton {
    id: ElementId,
    label: SharedString,
    tooltip: Option<SharedString>,
    bottom: Option<f32>,
    right: Option<f32>,
    primary: bool,
    square: bool,
    disabled: bool,
    on_click: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl FloatButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            tooltip: None,
            bottom: None,
            right: None,
            primary: false,
            square: false,
            disabled: false,
            on_click: None,
        }
    }

    /// 距容器底部偏移（默认 marginXXL = 48）
    pub fn bottom(mut self, bottom: f32) -> Self {
        self.bottom = Some(bottom);
        self
    }

    /// 距容器右侧偏移（默认 marginLG = 24）
    pub fn right(mut self, right: f32) -> Self {
        self.right = Some(right);
        self
    }

    /// 主色实心（antd `type="primary"`），默认白底描边
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }

    /// 方形（antd `shape="square"`），默认正圆
    pub fn square(mut self) -> Self {
        self.square = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 悬停提示文字（悬停时显示在按钮左侧）
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_click(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for FloatButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let m = theme::with(|t| crate::theme::metric::float_button_metric(t));
        let (color, variant) = if self.primary {
            (button::ButtonColor::Primary, button::ButtonVariant::Solid)
        } else {
            (
                button::ButtonColor::Default,
                button::ButtonVariant::Outlined,
            )
        };
        // individual 的 boxShadowSecondary 覆盖按钮自带投影
        let shadow = theme::with(|t| t.shadow_secondary.0.clone());
        let (idle, hover, active) = theme::with(|t| {
            (
                button::state(color, variant, t, false, false, false, self.disabled),
                button::state(color, variant, t, false, true, false, self.disabled),
                button::state(color, variant, t, false, false, true, self.disabled),
            )
        });

        let mut button = div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size(px(m.size))
            .py(px(m.padding_block))
            .gap(px(m.gap))
            .rounded(if self.square {
                px(m.radius_square)
            } else {
                px(m.size)
            })
            .border(px(m.border_width))
            .border_color(idle.border)
            .bg(idle.bg)
            .text_color(idle.fg)
            .text_size(px(m.icon_size))
            .shadow(shadow)
            .child(self.label.clone());
        if self.disabled {
            // antd 按钮禁用态：底色 colorBgContainerDisabled（半透明，需要先与面板底
            // 合成，否则 gpui 不裁切的投影会透上来）、描边 colorBorder、前景
            // colorTextDisabled（`button::state` 的默认禁用色不是这套）
            let (bg, border, fg) = theme::with(|t| {
                (
                    t.color_bg_container.blend(t.color_bg_container_disabled),
                    t.color_border,
                    t.color_text_disabled,
                )
            });
            button = button.bg(bg).border_color(border).text_color(fg);
        } else {
            button = button
                .cursor(CursorStyle::PointingHand)
                .hover(move |style| {
                    style
                        .bg(hover.bg)
                        .border_color(hover.border)
                        .text_color(hover.fg)
                })
                .active(move |style| {
                    style
                        .bg(active.bg)
                        .border_color(active.border)
                        .text_color(active.fg)
                        // Style.text 非 refineable：active 参与布局，须重申字号防按下变大
                        .text_size(px(m.icon_size))
                });
        }

        let semantics = a11y::Semantics::new(
            Role::Button,
            self.tooltip.clone().unwrap_or_else(|| self.label.clone()),
        )
        .id(format!("{:?}-button", self.id))
        .disabled(self.disabled)
        .focusable(!self.disabled);
        let semantics = if let Some(tooltip) = &self.tooltip {
            semantics.description(tooltip.clone())
        } else {
            semantics
        };
        button = semantics.apply(button);

        if let Some(on_click) = self.on_click.clone() {
            if !self.disabled {
                button = a11y::click_void(button, move |window, cx| on_click(window, cx));
            }
        }

        let group = format!("{:?}-group", self.id);
        let mut wrapper = div()
            .id(ElementId::Name(format!("{:?}-wrap", self.id).into()))
            .group(group.clone())
            .absolute()
            .bottom(px(self.bottom.unwrap_or(m.inset_block_end)))
            .right(px(self.right.unwrap_or(m.inset_inline_end)))
            .flex()
            .flex_row()
            .items_center()
            .gap_2();

        if let Some(tooltip) = &self.tooltip {
            wrapper = wrapper.child(
                div()
                    .id(ElementId::Name(format!("{:?}-tip", self.id).into()))
                    .opacity(0.)
                    .group_hover(group, |style| style.opacity(1.))
                    .child(tooltip::tooltip_bubble(tooltip.clone())),
            );
        }
        wrapper.child(button)
    }
}
