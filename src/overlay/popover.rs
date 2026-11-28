//! 气泡卡片：任意触发元素 + 任意浮层内容（比 Tooltip 更重，可交互）。
//!
//! 展开状态受控（`open` + `on_open_change`），点击气泡外部收起。

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, CursorStyle, Div, ElementId, Hsla, IntoElement, Pixels, Role, Window, div,
    prelude::*, px,
};

use crate::a11y::{self, Semantics};
use crate::motion;
use crate::popup::anchored;
use crate::theme::{self, Tokens, metric};

/// 气泡容器外观（antdv-next `.ant-popover-container` 实测）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopoverStyle {
    pub bg: Hsla,
    pub fg: Hsla,
    pub padding: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
}

/// `colorBgElevated` 底 + `colorText` 字；`paddingSM` 内边距、`borderRadiusLG` 圆角、无描边。
pub fn popover_state(tokens: &Tokens) -> PopoverStyle {
    let m = metric::PopoverMetric::resolve(tokens);
    PopoverStyle {
        bg: tokens.color_bg_elevated,
        fg: tokens.color_text,
        padding: m.padding,
        radius: m.radius,
        font_size: m.font_size,
        line_height: m.line_height,
    }
}

/// 气泡卡片。
///
/// ```ignore
/// Popover::new(
///     "info",
///     Button::new("查看详情").secondary(),
///     div().flex().flex_col().gap_1()
///         .child(div().child("这里是气泡内容"))
///         .child(Button::new("知道了").primary().small()),
/// )
/// .open(self.popover_open)
/// .on_open_change(cx.listener(|this, open, _, cx| {
///     this.popover_open = *open;
///     cx.notify();
/// }))
/// ```
pub struct Popover {
    id: ElementId,
    trigger: AnyElement,
    content: AnyElement,
    open: bool,
    width: Option<Pixels>,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
}

impl Popover {
    pub fn new(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        content: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            content: content.into_any_element(),
            open: false,
            width: None,
            on_open_change: None,
        }
    }

    /// 当前是否展开（受控）
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// 气泡宽度，默认与触发元素同宽
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn on_open_change(
        mut self,
        listener: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Popover {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let open = self.open;
        let suppress_toggle = Rc::new(Cell::new(false));

        let trigger_id = format!("{:?}-trigger", self.id);
        let mut trigger = div()
            .id(ElementId::Name(trigger_id.clone().into()))
            .child(self.trigger);
        if let Some(on_open_change) = self.on_open_change.clone() {
            let suppress_toggle = suppress_toggle.clone();
            trigger = trigger.cursor(CursorStyle::PointingHand);
            trigger = a11y::click_void(trigger, move |window, cx| {
                if suppress_toggle.take() {
                    return;
                }
                on_open_change(&!open, window, cx);
            });
            trigger = Semantics::new(Role::Button, if open { "收起气泡" } else { "展开气泡" })
                .id(trigger_id)
                .expanded(open)
                .focusable(true)
                .apply(trigger);
        }

        let mut popup = None;
        if open {
            let (style, shadow) = theme::with(|t| (popover_state(t), t.shadow_secondary.0.clone()));
            let shell_id = format!("{:?}-popover", self.id);
            let mut shell = div()
                .id(ElementId::Name(shell_id.clone().into()))
                .occlude()
                .p(px(style.padding))
                .rounded(px(style.radius))
                .bg(style.bg)
                .text_color(style.fg)
                .text_size(px(style.font_size))
                .line_height(px(style.line_height))
                .shadow(shadow)
                .child(self.content)
                .on_mouse_down_out({
                    let on_open_change = self.on_open_change.clone();
                    let suppress_toggle = suppress_toggle.clone();
                    move |_, window, cx| {
                        suppress_toggle.set(true);
                        if let Some(on_open_change) = &on_open_change {
                            on_open_change(&false, window, cx);
                        }
                    }
                });
            shell = match self.width {
                Some(w) => shell.w(w),
                None => shell.w_full(),
            };
            shell = Semantics::new(Role::Dialog, "气泡内容")
                .id(shell_id.clone())
                .apply(shell);
            let shell = motion::enter(
                format!("popover-enter-{shell_id}"),
                shell,
                |shell, progress| shell.opacity(progress).mt(px(-(1.0 - progress) * 4.0)),
            );
            popup = Some(div().child(shell));
        }

        anchored(trigger.into_any_element(), popup)
    }
}
