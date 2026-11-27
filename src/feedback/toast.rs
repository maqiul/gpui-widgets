//! 轻提示（antd Message）：顶部居中浮层堆栈。
//!
//! 与 Modal 相同：应在根视图中、滚动容器之外渲染，`deferred` 置顶绘制。
//! `ToastStack` 是受控底层；`ToastHost` 自动管理条目、到期、悬停暂停与数量上限。
//! 视觉对齐 antdv-next Message：max-content 宽卡片、colorBgElevated、二级阴影、
//! 16px 类型图标 + 14/22 正文、垂直内边距 (controlHeightLG-fontHeight)/2。

use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{
    App, Context, CursorStyle, Deferred, Div, ElementId, IntoElement, Render, Role, SharedString,
    Window, canvas, deferred, div, prelude::*, px,
};

use crate::a11y::{self, Semantics};
use crate::feedback::notice_timer::{NOTICE_TICK, NoticeTimer, trim_oldest};
use crate::svg_path::{paint_svg, svg_segments};
use crate::theme::{self, metric};

/// 轻提示类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    key: Option<SharedString>,
    duration: Option<Duration>,
    pause_on_hover: Option<bool>,
}

impl ToastItem {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            description: None,
            variant: ToastVariant::Info,
            key: None,
            duration: None,
            pause_on_hover: None,
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

    /// 条目的稳定身份。同一提示再次 `show` 时按 key 更新。
    pub fn key(mut self, key: impl Into<SharedString>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// `ToastHost` 的自动关闭时长；`Duration::ZERO` 表示常驻。
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// 覆盖 `ToastConfig` 的悬停暂停设置。
    pub fn pause_on_hover(mut self, pause: bool) -> Self {
        self.pause_on_hover = Some(pause);
        self
    }

    pub fn key_value(&self) -> Option<&SharedString> {
        self.key.as_ref()
    }

    pub fn duration_value(&self) -> Option<Duration> {
        self.duration
    }

    pub(crate) fn set_key(&mut self, key: SharedString) {
        self.key = Some(key);
    }
}

/// `ToastHost` 的预设 TTL：info/success 3s、warning 4.5s、error 6s。
pub fn default_toast_duration(variant: ToastVariant) -> Duration {
    match variant {
        ToastVariant::Info | ToastVariant::Success => Duration::from_secs(3),
        ToastVariant::Warning => Duration::from_millis(4_500),
        ToastVariant::Danger => Duration::from_secs(6),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToastDismissed {
    pub key: Option<SharedString>,
    pub index: usize,
    pub by_timer: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ToastConfig {
    pub max_count: usize,
    pub pause_on_hover: bool,
}

impl Default for ToastConfig {
    fn default() -> Self {
        Self {
            max_count: usize::MAX,
            pause_on_hover: true,
        }
    }
}

type ToastDismissListener = Rc<dyn Fn(&ToastDismissed, &mut Window, &mut App)>;
type ToastHoverListener = Rc<dyn Fn(&bool, &mut Window, &mut App)>;
type ToastHostDismissListener = Rc<dyn Fn(&ToastDismissed, &mut App)>;

const ICON_INFO: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm32 664c0 4.4-3.6 8-8 8h-48c-4.4 0-8-3.6-8-8V456c0-4.4 3.6-8 8-8h48c4.4 0 8 3.6 8 8v272zm-32-344a48.01 48.01 0 010-96 48.01 48.01 0 010 96z";
const ICON_SUCCESS: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm193.5 301.7l-210.6 292a31.8 31.8 0 01-51.7 0L318.5 484.9c-3.8-5.3 0-12.7 6.5-12.7h46.9c10.2 0 19.9 4.9 25.9 13.3l71.2 98.8 157.2-218c6-8.3 15.6-13.3 25.9-13.3H699c6.5 0 10.3 7.4 6.5 12.7z";
const ICON_WARNING: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm-32 232c0-4.4 3.6-8 8-8h48c4.4 0 8 3.6 8 8v272c0 4.4-3.6 8-8 8h-48c-4.4 0-8-3.6-8-8V296zm32 440a48.01 48.01 0 010-96 48.01 48.01 0 010 96z";
const ICON_ERROR: &str = "M512 64c247.4 0 448 200.6 448 448S960 512 960 512 759.4 960 512 960 64 759.4 64 512 264.6 64 512 64zm127.98 274.82h-.04l-.08.06L512 466.75 384.14 338.88c-.04-.05-.06-.06-.08-.06a.12.12 0 00-.07 0c-.03 0-.05.01-.09.05l-45.02 45.02a.2.2 0 00-.05.09.12.12 0 000 .07v.02a.27.27 0 00.06.06L466.75 512 338.88 639.86c-.05.04-.06.06-.08.06a.12.12 0 000 .07c0 .03.01.05.05.09l45.02 45.02a.2.2 0 00.09.05.12.12 0 000 .07v.02a.27.27 0 00-.06.06L512 557.25l127.86 127.87c.04.01.06.05.08.05a.12.12 0 00.07 0c.03-.01.05-.05.05-.09l45.02-45.02a.2.2 0 00.05-.09.12.12 0 000-.07v-.02a.27.27 0 00-.06-.06L557.25 512l127.87-127.86c.01-.04.05-.06.05-.08a.12.12 0 000-.07c0-.03-.05-.05-.09-.05l-45.02-45.02a.2.2 0 00-.09-.05.12.12 0 000-.07v-.02a.27.27 0 00.06-.06z";
const ICON_VIEW: (f32, f32, f32, f32) = (64.0, 64.0, 896.0, 896.0);

fn variant_icon(variant: ToastVariant) -> &'static str {
    match variant {
        ToastVariant::Info => ICON_INFO,
        ToastVariant::Success => ICON_SUCCESS,
        ToastVariant::Warning => ICON_WARNING,
        ToastVariant::Danger => ICON_ERROR,
    }
}

fn toast_identity(item: &ToastItem, index: usize) -> SharedString {
    item.key
        .clone()
        .unwrap_or_else(|| format!("index-{index}").into())
}

/// 单条 Message 卡片（堆栈与截图示例共用）。`on_close` 为扩展能力：
/// antd Message 默认无关闭钮，提供时在右侧渲染一个同 Notification 规格的关闭钮。
pub fn message_card(
    item: &ToastItem,
    ix: usize,
    on_close: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
) -> impl IntoElement {
    let on_close = on_close.map(|close| {
        Rc::new(
            move |event: &ToastDismissed, window: &mut Window, cx: &mut App| {
                close(&event.index, window, cx);
            },
        ) as ToastDismissListener
    });
    message_card_internal(item, ix, on_close, None)
}

fn message_card_internal(
    item: &ToastItem,
    ix: usize,
    on_close: Option<ToastDismissListener>,
    on_hover: Option<ToastHoverListener>,
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
    let identity = toast_identity(item, ix);
    let toast_id = format!("toast-{identity}");
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

    if let Some(on_hover) = on_hover {
        card = card.on_hover(move |hovered, window, cx| on_hover(hovered, window, cx));
    }

    if let Some(on_close) = on_close {
        let idle = theme::with(|t| metric::notice_close_state(t, false, false));
        let hover = theme::with(|t| metric::notice_close_state(t, true, false));
        let active = theme::with(|t| metric::notice_close_state(t, true, true));
        let close_id = format!("toast-close-{identity}");
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
        let key = item.key.clone();
        let close = a11y::click_void(close, move |window, cx| {
            on_close(
                &ToastDismissed {
                    key: key.clone(),
                    index: ix,
                    by_timer: false,
                },
                window,
                cx,
            );
        });
        let close = Semantics::new(Role::Button, "关闭提示")
            .id(close_id)
            .focusable(true)
            .apply(close);
        card = card.child(close);
    }
    semantics.apply(card)
}

pub(crate) fn toast_stack_container() -> Div {
    let m = theme::with(metric::message_metric);
    div()
        .absolute()
        .top(px(m.edge))
        .left_0()
        .right_0()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(m.stack_gap))
}

/// 轻提示堆栈（顶部居中，顺序由调用方决定）。
///
/// ```ignore
/// ToastStack::new(self.toasts.clone())
///     .on_dismiss(cx.listener(|this, event, _, cx| {
///         if let Some(key) = &event.key {
///             this.toasts.retain(|item| item.key_value() != Some(key));
///         }
///         cx.notify();
///     }))
/// ```
pub struct ToastStack {
    items: Vec<ToastItem>,
    on_dismiss: Option<ToastDismissListener>,
}

impl ToastStack {
    pub fn new(items: Vec<ToastItem>) -> Self {
        Self {
            items,
            on_dismiss: None,
        }
    }

    /// 兼容旧 API：点击关闭钮时回调当前下标。新代码应使用 `on_dismiss` 和稳定 key。
    pub fn on_close(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(
            move |event: &ToastDismissed, window: &mut Window, cx: &mut App| {
                listener(&event.index, window, cx);
            },
        ));
        self
    }

    /// 点击关闭钮时回调稳定 key、当前下标与关闭原因。
    pub fn on_dismiss(
        mut self,
        listener: impl Fn(&ToastDismissed, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for ToastStack {
    type Element = Deferred;

    fn into_element(self) -> Self::Element {
        let mut stack = toast_stack_container();
        let items = Rc::new(self.items.clone());
        for (ix, item) in self.items.iter().enumerate() {
            let key = item.key.clone();
            let items = items.clone();
            let listener = self.on_dismiss.clone().map(|listener| {
                Rc::new(
                    move |event: &ToastDismissed, window: &mut Window, cx: &mut App| {
                        let index = key
                            .as_ref()
                            .and_then(|key| {
                                items.iter().position(|item| item.key.as_ref() == Some(key))
                            })
                            .unwrap_or(event.index);
                        listener(
                            &ToastDismissed {
                                key: event.key.clone(),
                                index,
                                by_timer: event.by_timer,
                            },
                            window,
                            cx,
                        );
                    },
                ) as ToastDismissListener
            });
            stack = stack.child(message_card_internal(item, ix, listener, None));
        }
        deferred(stack)
    }
}

struct ToastHostEntry {
    item: ToastItem,
    key: SharedString,
    timer: NoticeTimer,
}

/// 自管轻提示容器。渲染在根视图、滚动容器之外，条目按最新在上排列。
pub struct ToastHost {
    entries: Vec<ToastHostEntry>,
    next_key: u64,
    config: ToastConfig,
    on_dismiss: Option<ToastHostDismissListener>,
    ticker_running: bool,
}

impl ToastHost {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            entries: Vec::new(),
            next_key: 1,
            config: ToastConfig::default(),
            on_dismiss: None,
            ticker_running: false,
        }
    }

    pub fn config(mut self, config: ToastConfig) -> Self {
        self.config = config;
        self
    }

    pub fn on_dismiss(mut self, listener: impl Fn(&ToastDismissed, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(listener));
        self
    }

    pub fn show(&mut self, item: ToastItem, cx: &mut Context<Self>) {
        let duration = item
            .duration
            .unwrap_or_else(|| default_toast_duration(item.variant));
        self.show_with(item, duration, cx);
    }

    pub fn show_with(&mut self, mut item: ToastItem, duration: Duration, cx: &mut Context<Self>) {
        item.duration = Some(duration);
        let now = Instant::now();
        let key = item.key.clone().unwrap_or_else(|| {
            let key = SharedString::from(format!("toast-{}", self.next_key));
            self.next_key += 1;
            key
        });
        item.set_key(key.clone());
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.key == key) {
            entry.item = item;
            entry.timer.restart(duration, now);
        } else {
            self.entries.insert(
                0,
                ToastHostEntry {
                    item,
                    key,
                    timer: NoticeTimer::new(duration, now),
                },
            );
            trim_oldest(&mut self.entries, self.config.max_count);
        }
        cx.notify();
        self.start_ticker(cx);
    }

    pub fn dismiss(&mut self, key: &SharedString, cx: &mut Context<Self>) -> bool {
        self.dismiss_inner(key, cx)
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.entries.clear();
        cx.notify();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn dismiss_inner(&mut self, key: &SharedString, cx: &mut Context<Self>) -> bool {
        let Some(index) = self.entries.iter().position(|entry| entry.key == *key) else {
            return false;
        };
        self.entries.remove(index);
        let event = ToastDismissed {
            key: Some(key.clone()),
            index,
            by_timer: false,
        };
        if let Some(listener) = &self.on_dismiss {
            listener(&event, cx);
        }
        cx.notify();
        true
    }

    fn set_hovered(&mut self, key: &SharedString, hovered: bool, cx: &mut Context<Self>) {
        let Some(entry) = self.entries.iter_mut().find(|entry| entry.key == *key) else {
            return;
        };
        if entry
            .item
            .pause_on_hover
            .unwrap_or(self.config.pause_on_hover)
        {
            entry.timer.set_hovered(hovered, Instant::now());
        }
        if !hovered {
            self.start_ticker(cx);
        }
    }

    fn has_active_timers(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.timer.has_active_timer())
    }

    fn refresh_ticker_state(&mut self) -> bool {
        self.ticker_running = self.has_active_timers();
        self.ticker_running
    }

    fn start_ticker(&mut self, cx: &mut Context<Self>) {
        if self.ticker_running || !self.has_active_timers() {
            return;
        }
        self.ticker_running = true;
        let this = cx.weak_entity();
        let mut app = cx.to_async();
        cx.foreground_executor()
            .spawn(async move {
                loop {
                    app.background_executor().timer(NOTICE_TICK).await;
                    match this.update(&mut app, |this, cx| this.tick(cx)) {
                        Ok(true) => {}
                        _ => break,
                    }
                }
            })
            .detach();
    }

    fn tick(&mut self, cx: &mut Context<Self>) -> bool {
        let now = Instant::now();
        let mut expired = Vec::new();
        let mut index = 0;
        self.entries.retain(|entry| {
            let current = index;
            index += 1;
            if entry.timer.expired(now) {
                expired.push(ToastDismissed {
                    key: Some(entry.key.clone()),
                    index: current,
                    by_timer: true,
                });
                false
            } else {
                true
            }
        });
        if !expired.is_empty() {
            if let Some(listener) = &self.on_dismiss {
                for event in &expired {
                    listener(event, cx);
                }
            }
            cx.notify();
        }
        self.refresh_ticker_state()
    }
}

impl Render for ToastHost {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut stack = toast_stack_container();
        for (ix, entry) in self.entries.iter().enumerate() {
            let key = entry.key.clone();
            let listener = cx.listener(move |this, _: &ToastDismissed, _, cx| {
                this.dismiss_inner(&key, cx);
            });
            let key = entry.key.clone();
            let hover = cx.listener(move |this, hovered: &bool, _, cx| {
                this.set_hovered(&key, *hovered, cx);
            });
            stack = stack.child(message_card_internal(
                &entry.item,
                ix,
                Some(Rc::new(listener)),
                Some(Rc::new(hover)),
            ));
        }
        deferred(stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_durations_increase_with_severity() {
        assert!(
            default_toast_duration(ToastVariant::Success)
                < default_toast_duration(ToastVariant::Warning)
        );
        assert!(
            default_toast_duration(ToastVariant::Warning)
                < default_toast_duration(ToastVariant::Danger)
        );
        assert!(default_toast_duration(ToastVariant::Danger) < Duration::from_secs(30));
    }

    #[test]
    fn explicit_key_controls_stable_element_ids() {
        let keyed = ToastItem::new("a").key("login");
        let plain = ToastItem::new("b");
        assert_eq!(toast_identity(&keyed, 0), SharedString::from("login"));
        assert_eq!(toast_identity(&plain, 0), SharedString::from("index-0"));
    }

    #[test]
    fn ticker_stops_without_active_timers_and_restarts_after_hover() {
        let now = Instant::now();
        let mut host = ToastHost {
            entries: vec![ToastHostEntry {
                item: ToastItem::new("a"),
                key: SharedString::from("a"),
                timer: NoticeTimer::new(Duration::from_secs(3), now),
            }],
            next_key: 2,
            config: ToastConfig::default(),
            on_dismiss: None,
            ticker_running: true,
        };
        assert!(host.refresh_ticker_state());
        host.entries[0].timer.set_hovered(true, now);
        assert!(!host.refresh_ticker_state());
        host.entries[0]
            .timer
            .set_hovered(false, now + Duration::from_secs(1));
        assert!(host.refresh_ticker_state());
        host.entries.clear();
        assert!(!host.refresh_ticker_state());
    }
}
