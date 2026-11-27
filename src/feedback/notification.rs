//! 通知提醒：右上角浮层通知堆栈，比 Message 更重（标题 + 描述 + 操作按钮）。
//!
//! 与 ToastStack 相同：应在根视图中、滚动容器之外渲染，`deferred` 置顶绘制。
//! `NotificationStack` 是受控底层；`NotificationHost` 自动管理条目与到期时间。
//! 视觉对齐 antdv-next Notification：384 宽卡片、colorBgElevated、二级阴影、
//! 24px 类型图标 + 16/24 标题 + 14 描述、右上角 22px 关闭钮。

use std::rc::Rc;
use std::time::{Duration, Instant};

use gpui::{
    App, Context, CursorStyle, Deferred, Div, ElementId, IntoElement, Render, Role, SharedString,
    Window, canvas, deferred, div, prelude::*, px,
};

use crate::a11y::{self, Semantics};
use crate::button::Button;
use crate::feedback::notice_timer::{NOTICE_TICK, NoticeTimer, trim_oldest};
use crate::svg_path::{paint_svg, svg_segments};
use crate::theme::{self, metric};

/// 通知类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationVariant {
    Info,
    Success,
    Warning,
    Danger,
}

/// 单条通知
#[derive(Clone)]
pub struct NotificationItem {
    title: SharedString,
    description: Option<SharedString>,
    variant: NotificationVariant,
    read: bool,
    timestamp: Option<SharedString>,
    action_label: Option<SharedString>,
    action: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    key: Option<SharedString>,
    duration: Option<Duration>,
    pause_on_hover: Option<bool>,
}

impl NotificationItem {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            variant: NotificationVariant::Info,
            read: false,
            timestamp: None,
            action_label: None,
            action: None,
            key: None,
            duration: None,
            pause_on_hover: None,
        }
    }

    pub fn success(mut self) -> Self {
        self.variant = NotificationVariant::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.variant = NotificationVariant::Warning;
        self
    }

    pub fn danger(mut self) -> Self {
        self.variant = NotificationVariant::Danger;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn read(mut self, read: bool) -> Self {
        self.read = read;
        self
    }

    pub fn set_read(&mut self, read: bool) {
        self.read = read;
    }

    pub fn timestamp(mut self, timestamp: impl Into<SharedString>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// 条目的稳定身份。同一通知再次 `show` 时按 key 更新。
    pub fn key(mut self, key: impl Into<SharedString>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// `NotificationHost` 的自动关闭时长；`Duration::ZERO` 表示常驻。
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// 覆盖 `NotificationConfig` 的悬停暂停设置。
    pub fn pause_on_hover(mut self, pause: bool) -> Self {
        self.pause_on_hover = Some(pause);
        self
    }

    pub fn title(&self) -> &SharedString {
        &self.title
    }

    pub fn description_text(&self) -> Option<&SharedString> {
        self.description.as_ref()
    }

    pub fn variant(&self) -> NotificationVariant {
        self.variant
    }

    pub fn is_read(&self) -> bool {
        self.read
    }

    pub fn timestamp_text(&self) -> Option<&SharedString> {
        self.timestamp.as_ref()
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

    /// 操作按钮（small primary，右对齐，如"查看详情"）
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        listener: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action_label = Some(label.into());
        self.action = Some(Rc::new(listener));
        self
    }
}

/// `NotificationHost` 的预设 TTL，与 antd Notification 默认值一致。
pub fn default_notification_duration() -> Duration {
    Duration::from_millis(4_500)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationDismissed {
    pub key: Option<SharedString>,
    pub index: usize,
    pub by_timer: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct NotificationConfig {
    pub max_count: usize,
    pub pause_on_hover: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            max_count: usize::MAX,
            pause_on_hover: true,
        }
    }
}

type NotificationDismissListener = Rc<dyn Fn(&NotificationDismissed, &mut Window, &mut App)>;
type NotificationHostDismissListener = Rc<dyn Fn(&NotificationDismissed, &mut App)>;
type NotificationHoverListener = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

const ICON_INFO: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm32 664c0 4.4-3.6 8-8 8h-48c-4.4 0-8-3.6-8-8V456c0-4.4 3.6-8 8-8h48c4.4 0 8 3.6 8 8v272zm-32-344a48.01 48.01 0 010-96 48.01 48.01 0 010 96z";
const ICON_SUCCESS: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm193.5 301.7l-210.6 292a31.8 31.8 0 01-51.7 0L318.5 484.9c-3.8-5.3 0-12.7 6.5-12.7h46.9c10.2 0 19.9 4.9 25.9 13.3l71.2 98.8 157.2-218c6-8.3 15.6-13.3 25.9-13.3H699c6.5 0 10.3 7.4 6.5 12.7z";
const ICON_WARNING: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm-32 232c0-4.4 3.6-8 8-8h48c4.4 0 8 3.6 8 8v272c0 4.4-3.6 8-8 8h-48c-4.4 0-8-3.6-8-8V296zm32 440a48.01 48.01 0 010-96 48.01 48.01 0 010 96z";
const ICON_ERROR: &str = "M512 64c247.4 0 448 200.6 448 448S960 512 960 512 759.4 960 512 960 64 759.4 64 512 264.6 64 512 64zm127.98 274.82h-.04l-.08.06L512 466.75 384.14 338.88c-.04-.05-.06-.06-.08-.06a.12.12 0 00-.07 0c-.03 0-.05.01-.09.05l-45.02 45.02a.2.2 0 00-.05.09.12.12 0 000 .07v.02a.27.27 0 00.06.06L466.75 512 338.88 639.86c-.05.04-.06.06-.08.06a.12.12 0 000 .07c0 .03.01.05.05.09l45.02 45.02a.2.2 0 00.09.05.12.12 0 000 .07v.02a.27.27 0 00-.06.06L512 557.25l127.86 127.87c.04.01.06.05.08.05a.12.12 0 00.07 0c.03-.01.05-.05.05-.09l45.02-45.02a.2.2 0 00.05-.09.12.12 0 000-.07v-.02a.27.27 0 00-.06-.06L557.25 512l127.87-127.86c.01-.04.05-.06.05-.08a.12.12 0 000-.07c0-.03-.05-.05-.09-.05l-45.02-45.02a.2.2 0 00-.09-.05.12.12 0 000-.07v-.02a.27.27 0 00.06-.06z";
const ICON_VIEW: (f32, f32, f32, f32) = (64.0, 64.0, 896.0, 896.0);

pub(crate) fn variant_icon(variant: NotificationVariant) -> &'static str {
    match variant {
        NotificationVariant::Info => ICON_INFO,
        NotificationVariant::Success => ICON_SUCCESS,
        NotificationVariant::Warning => ICON_WARNING,
        NotificationVariant::Danger => ICON_ERROR,
    }
}

pub(crate) fn icon_canvas(d: &'static str, size: f32, color: gpui::Hsla) -> impl IntoElement {
    let segs = svg_segments(d);
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, ICON_VIEW, &segs, color, window),
    )
    .w(px(size))
    .h(px(size))
    .flex_shrink_0()
}

fn notification_identity(item: &NotificationItem, index: usize) -> SharedString {
    item.key
        .clone()
        .unwrap_or_else(|| format!("index-{index}").into())
}

/// 单张通知卡片（堆栈与截图示例共用）。`on_close` 为 None 时不渲染关闭钮，
/// 标题/描述也不留 closable 右距（antd `:only-child` 规则）。
pub fn notification_card(
    item: &NotificationItem,
    ix: usize,
    on_close: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
) -> impl IntoElement {
    let on_close = on_close.map(|close| {
        Rc::new(
            move |event: &NotificationDismissed, window: &mut Window, cx: &mut App| {
                close(&event.index, window, cx);
            },
        ) as NotificationDismissListener
    });
    notification_card_internal(item, ix, on_close, None)
}

fn notification_card_internal(
    item: &NotificationItem,
    ix: usize,
    on_close: Option<NotificationDismissListener>,
    on_hover: Option<NotificationHoverListener>,
) -> impl IntoElement {
    let (m, cs, shadow) = theme::with(|t| {
        (
            metric::notification_metric(t),
            metric::notice_style(t),
            t.shadow_secondary.0.clone(),
        )
    });
    let closable = on_close.is_some();
    let identity = notification_identity(item, ix);
    let notification_id = format!("notification-{identity}");
    let mut semantics =
        Semantics::new(Role::Status, item.title.clone()).id(notification_id.clone());
    if let Some(description) = &item.description {
        semantics = semantics.description(description.clone());
    }
    let icon_color = match item.variant {
        NotificationVariant::Info => cs.icon_info,
        NotificationVariant::Success => cs.icon_success,
        NotificationVariant::Warning => cs.icon_warning,
        NotificationVariant::Danger => cs.icon_error,
    };

    let mut section = div()
        .flex()
        .flex_col()
        .gap(px(m.section_gap))
        .flex_1()
        .min_w_0();
    let mut title = div()
        .text_size(px(m.title_font))
        .line_height(px(m.title_line))
        .text_color(cs.title_fg)
        .child(item.title.clone());
    if closable {
        title = title.pr(px(m.closable_pad_end));
    }
    section = section.child(title);
    if let Some(description) = &item.description {
        // antd: title + description 同时存在时描述不留 closable 右距
        section = section.child(
            div()
                .text_size(px(m.desc_font))
                .line_height(px(m.line))
                .text_color(cs.text_fg)
                .child(description.clone()),
        );
    }
    if let (Some(label), Some(action)) = (&item.action_label, &item.action) {
        let action = action.clone();
        section = section.child(
            div()
                .flex()
                .justify_end()
                .mt(px(m.actions_margin_top))
                .child(
                    Button::new(label.clone())
                        .id(format!("notification-action-{identity}"))
                        .small()
                        .primary()
                        .on_click(move |_, window, cx| action(window, cx)),
                ),
        );
    }

    let mut card = div()
        .id(ElementId::Name(notification_id.into()))
        .relative()
        .flex()
        .flex_row()
        .items_start()
        .gap(px(m.icon_gap))
        .w(px(m.width))
        // antd 在 Windows 首选 Segoe UI；gpui 默认取系统 UI 字体（中文系统为
        // Microsoft YaHei），拉丁文宽约 12%，文字量大的卡片必须钉字体
        .font_family("Segoe UI")
        .py(px(m.padding_vertical))
        .px(px(m.padding_horizontal))
        .rounded(px(m.radius))
        .bg(cs.bg)
        .shadow(shadow)
        .child(icon_canvas(
            variant_icon(item.variant),
            m.icon_size,
            icon_color,
        ))
        .child(section);

    if let Some(on_hover) = on_hover {
        card = card.on_hover(move |hovered, window, cx| on_hover(hovered, window, cx));
    }

    if let Some(on_close) = on_close {
        let idle = theme::with(|t| metric::notice_close_state(t, false, false));
        let hover = theme::with(|t| metric::notice_close_state(t, true, false));
        let active = theme::with(|t| metric::notice_close_state(t, true, true));
        let close_id = format!("notif-close-{identity}");
        let close = div()
            .id(ElementId::Name(close_id.clone().into()))
            .absolute()
            .top(px(m.close_top))
            .right(px(m.close_right))
            .flex()
            .items_center()
            .justify_center()
            .size(px(m.close_size))
            .rounded(px(m.close_radius))
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
                &NotificationDismissed {
                    key: key.clone(),
                    index: ix,
                    by_timer: false,
                },
                window,
                cx,
            );
        });
        let close = Semantics::new(Role::Button, "关闭通知")
            .id(close_id)
            .focusable(true)
            .apply(close);
        card = card.child(close);
    }
    semantics.apply(card)
}

pub(crate) fn notification_stack_container() -> Div {
    let m = theme::with(metric::notification_metric);
    div()
        .absolute()
        .top(px(m.edge))
        .right(px(m.edge))
        .flex()
        .flex_col()
        .gap(px(m.stack_gap))
        .w(px(m.width))
}

/// 通知堆栈（右上角，顺序由调用方决定）。
///
/// ```ignore
/// NotificationStack::new(self.notifications.clone())
///     .on_dismiss(cx.listener(|this, event, _, cx| {
///         if let Some(key) = &event.key {
///             this.notifications.retain(|item| item.key_value() != Some(key));
///         }
///         cx.notify();
///     }))
/// ```
pub struct NotificationStack {
    items: Vec<NotificationItem>,
    on_dismiss: Option<NotificationDismissListener>,
}

impl NotificationStack {
    pub fn new(items: Vec<NotificationItem>) -> Self {
        Self {
            items,
            on_dismiss: None,
        }
    }

    /// 兼容旧 API：点击关闭钮时回调当前下标。新代码应使用 `on_dismiss` 和稳定 key。
    pub fn on_close(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(
            move |event: &NotificationDismissed, window: &mut Window, cx: &mut App| {
                listener(&event.index, window, cx);
            },
        ));
        self
    }

    /// 点击关闭钮时回调稳定 key、当前下标与关闭原因。
    pub fn on_dismiss(
        mut self,
        listener: impl Fn(&NotificationDismissed, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for NotificationStack {
    type Element = Deferred;

    fn into_element(self) -> Self::Element {
        let mut stack = notification_stack_container();
        let items = Rc::new(self.items.clone());
        for (ix, item) in self.items.iter().enumerate() {
            let key = item.key.clone();
            let items = items.clone();
            let listener = self.on_dismiss.clone().map(|listener| {
                Rc::new(
                    move |event: &NotificationDismissed, window: &mut Window, cx: &mut App| {
                        let index = key
                            .as_ref()
                            .and_then(|key| {
                                items.iter().position(|item| item.key.as_ref() == Some(key))
                            })
                            .unwrap_or(event.index);
                        listener(
                            &NotificationDismissed {
                                key: event.key.clone(),
                                index,
                                by_timer: event.by_timer,
                            },
                            window,
                            cx,
                        );
                    },
                ) as NotificationDismissListener
            });
            stack = stack.child(notification_card_internal(item, ix, listener, None));
        }
        deferred(stack)
    }
}

struct NotificationHostEntry {
    item: NotificationItem,
    key: SharedString,
    timer: NoticeTimer,
}

/// 自管通知容器。渲染在根视图、滚动容器之外，条目按最新在上排列。
pub struct NotificationHost {
    entries: Vec<NotificationHostEntry>,
    next_key: u64,
    config: NotificationConfig,
    on_dismiss: Option<NotificationHostDismissListener>,
    ticker_running: bool,
}

impl NotificationHost {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            entries: Vec::new(),
            next_key: 1,
            config: NotificationConfig::default(),
            on_dismiss: None,
            ticker_running: false,
        }
    }

    pub fn config(mut self, config: NotificationConfig) -> Self {
        self.config = config;
        self
    }

    pub fn on_dismiss(
        mut self,
        listener: impl Fn(&NotificationDismissed, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Rc::new(listener));
        self
    }

    pub fn show(&mut self, item: NotificationItem, cx: &mut Context<Self>) {
        let duration = item.duration.unwrap_or_else(default_notification_duration);
        self.show_with(item, duration, cx);
    }

    pub fn show_with(
        &mut self,
        mut item: NotificationItem,
        duration: Duration,
        cx: &mut Context<Self>,
    ) {
        item.duration = Some(duration);
        let now = Instant::now();
        let key = item.key.clone().unwrap_or_else(|| {
            let key = SharedString::from(format!("notification-{}", self.next_key));
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
                NotificationHostEntry {
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
        let event = NotificationDismissed {
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
                expired.push(NotificationDismissed {
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

impl Render for NotificationHost {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut stack = notification_stack_container();
        for (ix, entry) in self.entries.iter().enumerate() {
            let key = entry.key.clone();
            let listener = cx.listener(move |this, _: &NotificationDismissed, _, cx| {
                this.dismiss_inner(&key, cx);
            });
            let key = entry.key.clone();
            let hover = cx.listener(move |this, hovered: &bool, _, cx| {
                this.set_hovered(&key, *hovered, cx);
            });
            stack = stack.child(notification_card_internal(
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
    fn notification_default_matches_antd() {
        assert_eq!(
            default_notification_duration(),
            Duration::from_millis(4_500)
        );
    }

    #[test]
    fn explicit_key_controls_stable_element_ids() {
        let keyed = NotificationItem::new("a").key("login");
        let plain = NotificationItem::new("b");
        assert_eq!(
            notification_identity(&keyed, 0),
            SharedString::from("login")
        );
        assert_eq!(
            notification_identity(&plain, 0),
            SharedString::from("index-0")
        );
    }

    #[test]
    fn ticker_stops_without_active_timers_and_restarts_after_hover() {
        let now = Instant::now();
        let mut host = NotificationHost {
            entries: vec![NotificationHostEntry {
                item: NotificationItem::new("a"),
                key: SharedString::from("a"),
                timer: NoticeTimer::new(Duration::from_secs(3), now),
            }],
            next_key: 2,
            config: NotificationConfig::default(),
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
