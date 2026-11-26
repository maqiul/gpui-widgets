//! 倒计时控件：对应 antd Statistic.Countdown（StatisticTimer type="countdown"）。
//!
//! 本身是一个 `Entity`，内部每秒刷新一次；归零时触发 `on_finish` 并停止计时。
//! 显示为 Statistic 内容行：字号 fontSizeHeading3、字色 colorTextHeading，
//! 按 `format` 格式化（默认 `HH:mm:ss`，`[文本]` 转义，单位 Y/M/D/H/m/s/S）。

use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::{App, Context, IntoElement, Render, Role, SharedString, Window, div, prelude::*, px};

use crate::a11y::Semantics;
use crate::theme;
use crate::theme::metric::countdown_metric;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// antd `statistic/utils.js` 的 timeUnits（毫秒）：Y=365天 M=30天 D H m s S。
const TIME_UNITS: [(char, u64); 7] = [
    ('Y', 31_536_000_000),
    ('M', 2_592_000_000),
    ('D', 86_400_000),
    ('H', 3_600_000),
    ('m', 60_000),
    ('s', 1_000),
    ('S', 1),
];

/// 纯函数：把剩余毫秒按 antd formatTimeStr 规则格式化。
/// 单位按 Y→M→D→H→m→s→S 依次取整扣减；连续单位字符按个数补零；
/// `[...]` 内文本原样保留。H 为总小时数（不 mod 24），与 antd 一致。
pub fn format_counter(duration_ms: u64, format: &str) -> String {
    let mut keeps: Vec<String> = Vec::new();
    let mut template = String::new();
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '[' {
            let mut keep = String::new();
            for c2 in chars.by_ref() {
                if c2 == ']' {
                    break;
                }
                keep.push(c2);
            }
            keeps.push(keep);
            template.push_str("[]");
        } else {
            template.push(c);
        }
    }

    let mut left = duration_ms;
    for (unit, ms) in TIME_UNITS {
        if !template.contains(unit) {
            continue;
        }
        let value = left / ms;
        left -= value * ms;
        let digits = value.to_string();
        let mut out = String::new();
        let mut run = 0usize;
        for c in template.chars() {
            if c == unit {
                run += 1;
            } else {
                if run > 0 {
                    out.push_str(&pad_number(&digits, run));
                    run = 0;
                }
                out.push(c);
            }
        }
        if run > 0 {
            out.push_str(&pad_number(&digits, run));
        }
        template = out;
    }

    let mut result = String::new();
    let mut keep_index = 0;
    let mut tchars = template.chars().peekable();
    while let Some(c) = tchars.next() {
        if c == '[' && tchars.peek() == Some(&']') {
            tchars.next();
            result.push_str(&keeps[keep_index]);
            keep_index += 1;
        } else {
            result.push(c);
        }
    }
    result
}

fn pad_number(digits: &str, width: usize) -> String {
    if digits.len() >= width {
        return digits.to_string();
    }
    format!("{}{}", "0".repeat(width - digits.len()), digits)
}

/// 倒计时。通过 `cx.new(|cx| Countdown::new(target, cx))` 创建后作为子视图渲染。
///
/// ```ignore
/// // 目标为 unix 秒时间戳：
/// let cd = cx.new(|cx| Countdown::new(target_secs, cx).on_finish(|cx| {
///     // 归零回调（仅触发一次）
/// }));
/// // 或从当前时间起倒计时 90 秒：
/// let cd = cx.new(|cx| Countdown::after(Duration::from_secs(90), cx));
/// // 自定义格式 / 标题（同 antd format / title）：
/// let cd = cx.new(|cx| Countdown::after(d, cx).format("D 天 HH:mm:ss").title("距结束"));
/// // render 中：div().child(cd.clone())
/// ```
pub struct Countdown {
    target: u64,
    remaining: u64,
    finished: bool,
    format: SharedString,
    title: Option<SharedString>,
    on_finish: Option<Rc<dyn Fn(&mut App)>>,
}

impl Countdown {
    /// 目标时刻（unix 秒时间戳），创建后立即开始每秒刷新
    pub fn new(target_unix_secs: u64, cx: &mut Context<Self>) -> Self {
        let remaining = target_unix_secs.saturating_sub(now_secs());

        // 每秒 tick（前台 executor + owned AsyncApp，避免借用生命周期问题）
        let this = cx.weak_entity();
        let mut app = cx.to_async();
        cx.foreground_executor()
            .spawn(async move {
                loop {
                    app.background_executor()
                        .timer(Duration::from_secs(1))
                        .await;
                    match this.update(&mut app, |this, cx| this.tick(cx)) {
                        Ok(true) => {}
                        // 归零或实体已释放时退出循环
                        _ => break,
                    }
                }
            })
            .detach();

        Self {
            target: target_unix_secs,
            remaining,
            finished: false,
            format: SharedString::from("HH:mm:ss"),
            title: None,
            on_finish: None,
        }
    }

    /// 从当前时间起倒计时指定时长
    pub fn after(duration: Duration, cx: &mut Context<Self>) -> Self {
        Self::new(now_secs() + duration.as_secs(), cx)
    }

    /// 显示格式（antd format，默认 `HH:mm:ss`；`[文本]` 转义）
    pub fn format(mut self, format: impl Into<SharedString>) -> Self {
        self.format = format.into();
        self
    }

    /// 标题（antd title，显示在内容上方，colorTextDescription / fontSize）
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 归零回调（仅触发一次，在计时归零的那次刷新中调用）
    pub fn on_finish(mut self, listener: impl Fn(&mut App) + 'static) -> Self {
        self.on_finish = Some(Rc::new(listener));
        self
    }

    /// 当前剩余秒数
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// 是否已归零
    pub fn finished(&self) -> bool {
        self.finished
    }

    /// 每秒刷新；返回 false 表示计时结束，停止循环
    fn tick(&mut self, cx: &mut Context<Self>) -> bool {
        self.remaining = self.target.saturating_sub(now_secs());
        if self.remaining == 0 && !self.finished {
            self.finished = true;
            if let Some(on_finish) = self.on_finish.clone() {
                on_finish(cx);
            }
        }
        cx.notify();
        !self.finished
    }
}

impl Render for Countdown {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let m = theme::with(countdown_metric);
        let text = format_counter(self.remaining * 1000, &self.format);

        let mut col = div().flex().flex_col();
        if let Some(title) = &self.title {
            col = col.child(
                div()
                    .pb(px(m.title_padding_bottom))
                    .text_size(px(m.title_font_size))
                    .line_height(px(m.title_font_size * m.line_height))
                    .text_color(m.title_color)
                    .child(title.clone()),
            );
        }
        let semantics = Semantics::new(
            Role::Timer,
            self.title
                .clone()
                .unwrap_or_else(|| SharedString::from("倒计时")),
        )
        .id(format!("countdown-{}", self.target))
        .value(text.clone())
        .numeric(self.remaining as f64);
        let element_id = semantics.element_id();
        col.child(
            semantics.apply(
                div()
                    .id(element_id)
                    .text_size(px(m.content_font_size))
                    .line_height(px(m.content_font_size * m.line_height))
                    .text_color(m.content_color)
                    .child(text),
            ),
        )
    }
}
