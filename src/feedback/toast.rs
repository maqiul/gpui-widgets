//! 轻提示（antd Message）：顶部居中浮层堆栈，条目由父视图持有（受控）。
//!
//! 与 Modal 相同：应在根视图中、滚动容器之外渲染，`deferred` 置顶绘制。
//! 自动消失可由调用方用定时器移除条目实现。
//! 视觉对齐 antdv-next Message：max-content 宽卡片、colorBgElevated、二级阴影、
//! 16px 类型图标 + 14/22 正文、垂直内边距 (controlHeightLG-fontHeight)/2。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Deferred, ElementId, IntoElement, Role, SharedString, Window, canvas,
    deferred, div, prelude::*, px,
};

use crate::a11y::{self, Semantics};
use crate::svg_path::{paint_svg, svg_segments};
use crate::theme::{self, metric};

/// 轻提示类型
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Info,
    Success,
    Warning,
    Danger,
}

/// 单条轻提示内容
#[derive(Clone)]
pub struct ToastItem {
    message: SharedString,
    description: Option<SharedString>,
    variant: ToastVariant,
}

impl ToastItem {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            description: None,
            variant: ToastVariant::Info,
        }
    }

    pub fn success(mut self) -> Self {
        self.variant = ToastVariant::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.variant = ToastVariant::Warning;
        self
    }

    pub fn danger(mut self) -> Self {
        self.variant = ToastVariant::Danger;
        self
    }

    /// 扩展字段：antd Message 无描述，这里作为正文第二行渲染（同色同字号）。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }
}

const ICON_INFO: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm32 664c0 4.4-3.6 8-8 8h-48c-4.4 0-8-3.6-8-8V456c0-4.4 3.6-8 8-8h48c4.4 0 8 3.6 8 8v272zm-32-344a48.01 48.01 0 010-96 48.01 48.01 0 010 96z";
const ICON_SUCCESS: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm193.5 301.7l-210.6 292a31.8 31.8 0 01-51.7 0L318.5 484.9c-3.8-5.3 0-12.7 6.5-12.7h46.9c10.2 0 19.9 4.9 25.9 13.3l71.2 98.8 157.2-218c6-8.3 15.6-13.3 25.9-13.3H699c6.5 0 10.3 7.4 6.5 12.7z";
const ICON_WARNING: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm-32 232c0-4.4 3.6-8 8-8h48c4.4 0 8 3.6 8 8v272c0 4.4-3.6 8-8 8h-48c-4.4 0-8-3.6-8-8V296zm32 440a48.01 48.01 0 010-96 48.01 48.01 0 010 96z";
const ICON_ERROR: &str = "M512 64c247.4 0 448 200.6 448 448S759.4 960 512 960 64 759.4 64 512 264.6 64 512 64zm127.98 274.82h-.04l-.08.06L512 466.75 384.14 338.88c-.04-.05-.06-.06-.08-.06a.12.12 0 00-.07 0c-.03 0-.05.01-.09.05l-45.02 45.02a.2.2 0 00-.05.09.12.12 0 000 .07v.02a.27.27 0 00.06.06L466.75 512 338.88 639.86c-.05.04-.06.06-.06.08a.12.12 0 000 .07c0 .03.01.05.05.09l45.02 45.02a.2.2 0 00.09.05.12.12 0 00.07 0c.02 0 .04-.01.08-.05L512 557.25l127.86 127.87c.04.04.06.05.08.05a.12.12 0 00.07 0c.03 0 .05-.01.09-.05l45.02-45.02a.2.2 0 00.05-.09.12.12 0 000-.07v-.02a.27.27 0 00-.05-.06L557.25 512l127.87-127.86c.04-.04.05-.06.05-.08a.12.12 0 000-.07c0-.03-.01-.05-.05-.09l-45.02-45.02a.2.2 0 00-.09-.05.12.12 0 00-.07 0z";
const ICON_VIEW: (f32, f32, f32, f32) = (64.0, 64.0, 896.0, 896.0);

fn variant_icon(variant: ToastVariant) -> &'static str {
    match variant {
        ToastVariant::Info => ICON_INFO,
        ToastVariant::Success => ICON_SUCCESS,
        ToastVariant::Warning => ICON_WARNING,
        ToastVariant::Danger => ICON_ERROR,
    }
}

/// 单条 Message 卡片（堆栈与截图示例共用）。`on_close` 为扩展能力：
/// antd Message 默认无关闭钮，提供时在右侧渲染一个同 Notification 规格的关闭钮。
pub fn message_card(
    item: &ToastItem,
    ix: usize,
    on_close: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
) -> impl IntoElement {
    let (m, nm, cs, shadow) = theme::with(|t| {
        (
            metric::message_metric(t),
            metric::notification_metric(t),
            metric::notice_style(t),
            t.shadow_secondary.0.clone(),
        )
    });
    let icon_color = match item.variant {
        ToastVariant::Info => cs.icon_info,
        ToastVariant::Success => cs.icon_success,
        ToastVariant::Warning => cs.icon_warning,
        ToastVariant::Danger => cs.icon_error,
    };
    let toast_id = format!("toast-{ix}");
    let mut semantics = Semantics::new(Role::Status, item.message.clone()).id(toast_id.clone());
    if let Some(description) = &item.description {
        semantics = semantics.description(description.clone());
    }
    let segs = svg_segments(variant_icon(item.variant));
    let icon = canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, ICON_VIEW, &segs, icon_color, window),
    )
    .w(px(m.icon_size))
    .h(px(m.icon_size))
    .flex_shrink_0();

    let mut body = div().flex().flex_col().child(
        div()
            .text_size(px(m.font))
            .line_height(px(m.line))
            .text_color(cs.text_fg)
            .child(item.message.clone()),
    );
    if let Some(description) = &item.description {
        body = body.child(
            div()
                .text_size(px(m.font))
                .line_height(px(m.line))
                .text_color(cs.text_fg)
                .child(description.clone()),
        );
    }

    let mut card = div()
        .id(ElementId::Name(toast_id.into()))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(m.icon_gap))
        .py(px(m.padding_vertical))
        .px(px(m.padding_horizontal))
        .rounded(px(m.radius))
        .bg(cs.bg)
        .shadow(shadow)
        .child(icon)
        .child(body);

    if let Some(on_close) = on_close {
        let idle = theme::with(|t| metric::notice_close_state(t, false, false));
        let hover = theme::with(|t| metric::notice_close_state(t, true, false));
        let active = theme::with(|t| metric::notice_close_state(t, true, true));
        let close_id = format!("toast-close-{ix}");
        let close = div()
            .id(ElementId::Name(close_id.clone().into()))
            .flex()
            .items_center()
            .justify_center()
            .size(px(nm.close_size))
            .flex_shrink_0()
            .rounded(px(nm.close_radius))
            .cursor(CursorStyle::PointingHand)
            .text_size(px(m.font))
            .line_height(px(m.font))
            .text_color(idle.fg)
            .hover(move |s| {
                let mut s = s.text_color(hover.fg);
                if let Some(bg) = hover.bg {
                    s = s.bg(bg);
                }
                s
            })
            .active(move |s| {
                let mut s = s.text_color(active.fg);
                if let Some(bg) = active.bg {
                    s = s.bg(bg);
                }
                // Style.text 非 refineable：active 参与布局，字号/行高须连补防按下变大
                s.text_size(px(m.font)).line_height(px(m.font))
            })
            .child("✕");
        let close = a11y::click_void(close, move |window, cx| on_close(&ix, window, cx));
        let close = Semantics::new(Role::Button, "关闭提示")
            .id(close_id)
            .focusable(true)
            .apply(close);
        card = card.child(close);
    }
    semantics.apply(card)
}

/// 轻提示堆栈（顶部居中，新条目在上）。
///
/// ```ignore
/// // render 中（根视图最后、滚动容器之外）：
/// if !self.toasts.is_empty() {
///     root = root.child(
///         ToastStack::new(self.toasts.clone())
///             .on_close(cx.listener(|this, ix, _, cx| {
///                 this.toasts.remove(*ix);
///                 cx.notify();
///             })),
///     );
/// }
/// ```
pub struct ToastStack {
    items: Vec<ToastItem>,
    on_close: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl ToastStack {
    pub fn new(items: Vec<ToastItem>) -> Self {
        Self {
            items,
            on_close: None,
        }
    }

    /// 点击某条关闭钮时回调其下标（扩展能力，antd Message 无关闭钮）
    pub fn on_close(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for ToastStack {
    type Element = Deferred;

    fn into_element(self) -> Self::Element {
        let m = theme::with(metric::message_metric);
        let mut stack = div()
            .absolute()
            .top(px(m.edge))
            .left_0()
            .right_0()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(m.stack_gap));

        for (ix, item) in self.items.iter().enumerate() {
            stack = stack.child(message_card(item, ix, self.on_close.clone()));
        }

        deferred(stack)
    }
}
