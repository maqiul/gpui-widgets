//! 气泡确认框：点击触发元素弹出确认 / 取消气泡（对应 antd Popconfirm）。
//!
//! 展开状态受控，外壳复用 Popover 表面（colorBgElevated / paddingSM / borderRadiusLG）；
//! 警告图标 ExclamationCircleFilled（colorWarning）+ 标题 fontWeightStrong/colorTextHeading
//! （无描述时 only-child 降为 normal）+ 描述 marginXXS/colorText；
//! 按钮行右对齐，取消/确认均 small（确认 primary，可 ok_danger）。

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, CursorStyle, ElementId, FontWeight, IntoElement, Role, SharedString, Window,
    canvas, div, prelude::*, px,
};

use crate::a11y::{self, Semantics};
use crate::button::ButtonVariant;

use crate::svg_path::{paint_svg, svg_segments};
use crate::theme::metric::{popconfirm_metric, popconfirm_style};
use crate::{Button, popover::popover_state, popup::anchored, theme};

const ICON: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm-32 232c0-4.4 3.6-8 8-8h48c4.4 0 8 3.6 8 8v272c0 4.4-3.6 8-8 8h-48c-4.4 0-8-3.6-8-8V296zm32 440a48.01 48.01 0 010-96 48.01 48.01 0 010 96z";
const ICON_VIEW: (f32, f32, f32, f32) = (64.0, 64.0, 896.0, 896.0);

/// 气泡确认框。
///
/// ```ignore
/// Popconfirm::new("del", "确定删除吗？")
///     .description("删除后不可恢复")
///     .open(self.popconfirm_open)
///     .on_open_change(cx.listener(|this, open, _, cx| { this.popconfirm_open = *open; cx.notify(); }))
///     .on_confirm(cx.listener(|this, _, cx| { /* 执行删除 */ }))
///     .child(Button::new("删除").danger())
/// ```
#[derive(IntoElement)]
pub struct Popconfirm {
    id: ElementId,
    title: SharedString,
    description: Option<SharedString>,
    ok_text: SharedString,
    cancel_text: SharedString,
    ok_danger: bool,
    open: bool,
    child: Option<AnyElement>,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    on_confirm: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_cancel: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl Popconfirm {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            ok_text: "OK".into(),
            cancel_text: "Cancel".into(),
            ok_danger: false,
            open: false,
            child: None,
            on_open_change: None,
            on_confirm: None,
            on_cancel: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn ok_text(mut self, text: impl Into<SharedString>) -> Self {
        self.ok_text = text.into();
        self
    }

    pub fn cancel_text(mut self, text: impl Into<SharedString>) -> Self {
        self.cancel_text = text.into();
        self
    }

    /// 确认按钮用 danger 描边（antd `okType="danger"` 走 Button type 语义）
    pub fn ok_danger(mut self, ok_danger: bool) -> Self {
        self.ok_danger = ok_danger;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// 触发元素
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }

    pub fn on_open_change(
        mut self,
        listener: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(listener));
        self
    }

    /// 点击确认（同时收起）
    pub fn on_confirm(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_confirm = Some(Rc::new(listener));
        self
    }

    /// 点击取消（同时收起）
    pub fn on_cancel(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(listener));
        self
    }
}

/// 气泡表面（外壳 + 图标标题描述 + 按钮行），渲染与截图基线共用。
pub fn popconfirm_surface(
    title: SharedString,
    description: Option<SharedString>,
    ok_text: SharedString,
    cancel_text: SharedString,
    ok_danger: bool,
    on_confirm: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_cancel: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
) -> impl IntoElement {
    let (m, cs, shell, shadow) = theme::with(|t| {
        (
            popconfirm_metric(t),
            popconfirm_style(t),
            popover_state(t),
            t.shadow_secondary.0.clone(),
        )
    });
    let surface_id = format!("popconfirm-{title}");
    let mut semantics = Semantics::new(Role::Dialog, title.clone())
        .id(surface_id.clone())
        .focusable(true);
    if let Some(description) = &description {
        semantics = semantics.description(description.clone());
    }

    let icon_segs = svg_segments(ICON);
    let icon_color = cs.icon;
    let icon = canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, ICON_VIEW, &icon_segs, icon_color, window),
    )
    .w(px(m.icon_size))
    .h(px(m.icon_size))
    .flex_shrink_0();

    let mut text = div().flex().flex_col().child(
        div()
            .font_weight(if description.is_some() {
                FontWeight(m.title_weight)
            } else {
                FontWeight::NORMAL
            })
            .text_color(cs.title_fg)
            .child(title),
    );
    if let Some(description) = &description {
        text = text.child(
            div()
                .mt(px(m.desc_margin_top))
                .text_color(cs.desc_fg)
                .child(description.clone()),
        );
    }
    let message = div()
        .flex()
        .flex_row()
        .items_start()
        .mb(px(m.message_margin_bottom))
        .child(div().mr(px(m.icon_gap)).child(icon))
        .child(text);

    let cancel = Button::new(cancel_text)
        .small()
        .on_click(move |_, window, cx| {
            if let Some(on_cancel) = &on_cancel {
                on_cancel(window, cx);
            }
        });
    let ok = Button::new(ok_text).small();
    // antd okType 走 Button `type`：danger = 描边红（非实心）
    let ok = if ok_danger {
        ok.color(crate::button::ButtonColor::Danger)
            .variant(ButtonVariant::Outlined)
    } else {
        ok.primary()
    };
    let ok = ok.on_click(move |_, window, cx| {
        if let Some(on_confirm) = &on_confirm {
            on_confirm(window, cx);
        }
    });
    let buttons = div()
        .flex()
        .flex_row()
        .justify_end()
        .child(cancel)
        .child(div().ml(px(m.button_gap)).child(ok));

    semantics.apply(
        div()
            .id(ElementId::Name(surface_id.into()))
            .occlude()
            .p(px(shell.padding))
            .rounded(px(shell.radius))
            .bg(shell.bg)
            .text_color(shell.fg)
            .text_size(px(shell.font_size))
            .line_height(px(shell.line_height))
            .shadow(shadow)
            .child(message)
            .child(buttons),
    )
}

impl RenderOnce for Popconfirm {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let open = self.open;
        let suppress_toggle = Rc::new(Cell::new(false));

        // 触发元素包一层以接管点击
        let trigger_id = format!("{:?}-trigger", self.id);
        let mut trigger = div()
            .id(ElementId::Name(trigger_id.clone().into()))
            .cursor(CursorStyle::PointingHand);
        if let Some(child) = self.child {
            trigger = trigger.child(child);
        }
        if let Some(on_open_change) = self.on_open_change.clone() {
            let suppress = suppress_toggle.clone();
            trigger = a11y::click_void(trigger, move |window, cx| {
                if suppress.take() {
                    return;
                }
                on_open_change(&!open, window, cx);
            });
            trigger = Semantics::new(Role::Button, self.title.clone())
                .id(trigger_id)
                .expanded(open)
                .focusable(true)
                .apply(trigger);
        }

        let popup = if open {
            let close = {
                let on_open_change = self.on_open_change.clone();
                let suppress = suppress_toggle.clone();
                Rc::new(move |window: &mut Window, cx: &mut App| {
                    suppress.set(true);
                    if let Some(on_open_change) = &on_open_change {
                        on_open_change(&false, window, cx);
                    }
                })
            };
            let on_confirm = {
                let on_confirm = self.on_confirm.clone();
                let close = close.clone();
                Rc::new(move |window: &mut Window, cx: &mut App| {
                    if let Some(on_confirm) = &on_confirm {
                        on_confirm(window, cx);
                    }
                    close(window, cx);
                })
            };
            let on_cancel = {
                let on_cancel = self.on_cancel.clone();
                let close = close.clone();
                Rc::new(move |window: &mut Window, cx: &mut App| {
                    if let Some(on_cancel) = &on_cancel {
                        on_cancel(window, cx);
                    }
                    close(window, cx);
                })
            };
            let surface = popconfirm_surface(
                self.title.clone(),
                self.description.clone(),
                self.ok_text.clone(),
                self.cancel_text.clone(),
                self.ok_danger,
                Some(on_confirm),
                Some(on_cancel),
            );
            Some(
                div().child(
                    div()
                        .id(ElementId::Name(format!("{:?}-shell", self.id).into()))
                        .on_mouse_down_out({
                            let on_open_change = self.on_open_change.clone();
                            let suppress = suppress_toggle.clone();
                            move |_, window, cx| {
                                suppress.set(true);
                                if let Some(on_open_change) = &on_open_change {
                                    on_open_change(&false, window, cx);
                                }
                            }
                        })
                        .child(surface),
                ),
            )
        } else {
            None
        };
        anchored(trigger.into_any_element(), popup)
    }
}
