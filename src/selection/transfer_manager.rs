//! 传输任务管理器：上传 / 下载队列的总览进度、单项进度与操作按钮。
//!
//! 面向「文件传输面板」场景（如 SFTP 队列）：任务列表由父视图持有，
//! 控件只负责展示与回调，重排/推进进度由调用方处理。
//!
//! ```ignore
//! TransferManager::new("queue", self.tasks.clone())
//!     .title("传输任务")
//!     .on_pause(cx.listener(|this, id, _, cx| this.set_status(id, TransferStatus::Paused, cx)))
//!     .on_clear_done(cx.listener(|this, _, _, cx| { this.remove_done(); cx.notify(); }))
//! ```

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, IntoElement, RenderOnce, Role, SharedString, Window,
    div, prelude::*, px, relative,
};

use crate::{a11y, theme};

/// 传输方向
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum TransferDirection {
    /// 本地 → 远端
    #[default]
    Upload,
    /// 远端 → 本地
    Download,
}

impl TransferDirection {
    pub fn label(self) -> &'static str {
        match self {
            TransferDirection::Upload => "上传",
            TransferDirection::Download => "下载",
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            TransferDirection::Upload => "⬆",
            TransferDirection::Download => "⬇",
        }
    }
}

/// 传输状态
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum TransferStatus {
    /// 排队等待
    #[default]
    Queued,
    /// 正在传输
    Transferring,
    /// 已暂停
    Paused,
    /// 已完成
    Done,
    /// 失败
    Failed,
    /// 已取消
    Canceled,
}

impl TransferStatus {
    pub fn label(self) -> &'static str {
        match self {
            TransferStatus::Queued => "等待中",
            TransferStatus::Transferring => "传输中",
            TransferStatus::Paused => "已暂停",
            TransferStatus::Done => "已完成",
            TransferStatus::Failed => "失败",
            TransferStatus::Canceled => "已取消",
        }
    }

    /// 是否仍在队列中（未结束）
    pub fn is_active(self) -> bool {
        matches!(self, TransferStatus::Queued | TransferStatus::Transferring)
    }

    fn color(self) -> Hsla {
        theme::with(|t| crate::theme::metric::transfer_status_style(t, self).fg)
    }

    fn soft_color(self) -> Hsla {
        theme::with(|t| crate::theme::metric::transfer_status_style(t, self).bg)
    }
}

/// 单个传输任务
#[derive(Clone)]
pub struct TransferTask {
    id: SharedString,
    name: SharedString,
    direction: TransferDirection,
    status: TransferStatus,
    progress: f32,
    size: Option<u64>,
    speed: Option<f64>,
    error: Option<SharedString>,
}

impl TransferTask {
    pub fn new(
        id: impl Into<SharedString>,
        name: impl Into<SharedString>,
        direction: TransferDirection,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            direction,
            status: TransferStatus::default(),
            progress: 0.,
            size: None,
            speed: None,
            error: None,
        }
    }

    /// 上传任务
    pub fn upload(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self::new(id, name, TransferDirection::Upload)
    }

    /// 下载任务
    pub fn download(id: impl Into<SharedString>, name: impl Into<SharedString>) -> Self {
        Self::new(id, name, TransferDirection::Download)
    }

    pub fn status(mut self, status: TransferStatus) -> Self {
        self.status = status;
        self
    }

    /// 进度 0.0 ~ 1.0
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0., 1.);
        self
    }

    /// 文件总字节数（用于展示 已传/总量 与剩余时间）
    pub fn size(mut self, bytes: u64) -> Self {
        self.size = Some(bytes);
        self
    }

    /// 当前速度（字节/秒）
    pub fn speed(mut self, bytes_per_sec: f64) -> Self {
        self.speed = Some(bytes_per_sec);
        self
    }

    /// 失败原因
    pub fn error(mut self, message: impl Into<SharedString>) -> Self {
        self.error = Some(message.into());
        self
    }

    pub fn id(&self) -> &SharedString {
        &self.id
    }

    pub fn name(&self) -> &SharedString {
        &self.name
    }

    pub fn task_status(&self) -> TransferStatus {
        self.status
    }

    /// 当前进度 0.0 ~ 1.0
    pub fn progress_value(&self) -> f32 {
        self.progress
    }

    /// 修改状态（供调用方驱动队列状态机）
    pub fn set_status(&mut self, status: TransferStatus) {
        self.status = status;
    }

    /// 修改进度（供调用方推进）
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0., 1.);
    }
}

/// 传输任务列表（带总进度与行操作）。
#[derive(IntoElement)]
pub struct TransferManager {
    id: ElementId,
    tasks: Vec<TransferTask>,
    title: SharedString,
    show_summary: bool,
    on_pause: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_resume: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_cancel: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_retry: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_remove: Option<Rc<dyn Fn(&SharedString, &mut Window, &mut App)>>,
    on_clear_done: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl TransferManager {
    pub fn new(id: impl Into<ElementId>, tasks: Vec<TransferTask>) -> Self {
        Self {
            id: id.into(),
            tasks,
            title: SharedString::from("传输任务"),
            show_summary: true,
            on_pause: None,
            on_resume: None,
            on_cancel: None,
            on_retry: None,
            on_remove: None,
            on_clear_done: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    /// 是否显示头部总进度条（默认显示）
    pub fn show_summary(mut self, show: bool) -> Self {
        self.show_summary = show;
        self
    }

    /// 点击「暂停」回调任务 id（传输中任务）
    pub fn on_pause(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_pause = Some(Rc::new(listener));
        self
    }

    /// 点击「继续」回调任务 id（已暂停任务）
    pub fn on_resume(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_resume = Some(Rc::new(listener));
        self
    }

    /// 点击「取消」回调任务 id（等待中 / 传输中 / 已暂停任务）
    pub fn on_cancel(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_cancel = Some(Rc::new(listener));
        self
    }

    /// 点击「重试」回调任务 id（失败任务）
    pub fn on_retry(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_retry = Some(Rc::new(listener));
        self
    }

    /// 点击「移除」回调任务 id（已完成 / 已取消 / 失败任务）
    pub fn on_remove(
        mut self,
        listener: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_remove = Some(Rc::new(listener));
        self
    }

    /// 点击头部「清除已完成」回调（无任务 id）
    pub fn on_clear_done(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear_done = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for TransferManager {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let m = theme::with(crate::theme::metric::transfer_manager_metric);
        let id = self.id.clone();
        let total = self.tasks.len();
        let transferring = self
            .tasks
            .iter()
            .filter(|t| t.status == TransferStatus::Transferring)
            .count();
        let finished = self
            .tasks
            .iter()
            .filter(|t| t.status == TransferStatus::Done)
            .count();

        let root = div()
            .id(id.clone())
            .flex()
            .flex_col()
            .w_full()
            .rounded_lg()
            .border_1()
            .border_color(theme::border())
            .bg(theme::bg_container())
            .overflow_hidden();
        let mut root = a11y::Semantics::new(Role::List, self.title.clone())
            .id(format!("{id:?}"))
            .apply(root);

        // 头部：标题 + 统计 + 清除已完成
        let mut sub = vec![format!("共 {total} 个任务")];
        if transferring > 0 {
            sub.push(format!("{transferring} 个传输中"));
        }
        if finished > 0 {
            sub.push(format!("{finished} 个已完成"));
        }

        let mut header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .gap_2()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(theme::border())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(m.meta_gap))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme::text())
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .child(sub.join(" · ")),
                    ),
            );

        if let Some(on_clear) = self.on_clear_done.clone()
            && finished > 0
        {
            let mut clear = div()
                .id(ElementId::Name(format!("{id:?}-clear").into()))
                .px_2()
                .py_1()
                .rounded_md()
                .text_xs()
                .text_color(theme::text_secondary())
                .cursor(CursorStyle::PointingHand)
                .hover(|style| style.bg(theme::control_item_bg_hover()))
                .child("清除已完成");
            clear = a11y::Semantics::new(Role::Button, "清除已完成")
                .id(format!("{id:?}-clear"))
                .focusable(true)
                .apply(clear);
            clear = a11y::click_void(clear, move |window, cx| on_clear(window, cx));
            header = header.child(clear);
        }
        root = root.child(header);

        // 总进度：按文件大小加权（无大小按 1 计），已取消任务不计入
        if self.show_summary && total > 0 {
            let mut weight = 0f64;
            let mut gained = 0f64;
            let mut size_sum = 0u64;
            let mut size_done = 0u64;
            let mut sized = true;
            for task in &self.tasks {
                if task.status == TransferStatus::Canceled {
                    continue;
                }
                let w = task.size.unwrap_or(1) as f64;
                let p = if task.status == TransferStatus::Done {
                    1.
                } else {
                    task.progress as f64
                };
                weight += w;
                gained += w * p;
                match task.size {
                    Some(size) => {
                        size_sum += size;
                        size_done += (size as f64 * p) as u64;
                    }
                    None => sized = false,
                }
            }
            let overall = if weight > 0. {
                (gained / weight) as f32
            } else {
                0.
            };
            let mut detail = vec![format!("总进度 {:.0}%", overall * 100.)];
            if sized && size_sum > 0 {
                detail.push(format!(
                    "{} / {}",
                    format_size(size_done),
                    format_size(size_sum)
                ));
            }
            root = root.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(theme::border())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_secondary())
                                    .child(detail.join(" · ")),
                            ),
                    )
                    .child(progress_bar(
                        &format!("{id:?}-overall-progress"),
                        "总进度",
                        overall,
                        theme::primary(),
                    )),
            );
        }

        // 空状态
        if total == 0 {
            return root.child(
                div()
                    .py(px(m.empty_padding_block))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(theme::text_secondary())
                    .child("暂无传输任务"),
            );
        }

        // 任务行
        for (ix, task) in self.tasks.iter().enumerate() {
            let task_id = task.id.clone();
            let status = task.status;
            let last = ix + 1 == total;

            let (icon_bg, icon_fg) = match (task.direction, status) {
                (_, TransferStatus::Failed) => (theme::danger_bg(), theme::danger()),
                (TransferDirection::Upload, _) => (theme::primary_bg(), theme::primary()),
                (TransferDirection::Download, _) => (theme::success_bg(), theme::success()),
            };

            let name_color = match status {
                TransferStatus::Failed => theme::danger(),
                TransferStatus::Canceled => theme::text_secondary(),
                _ => theme::text(),
            };

            let title_row = div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(name_color)
                        .child(task.name.clone()),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .px(px(m.tag_padding_inline))
                                .py(px(m.tag_padding_block))
                                .rounded_sm()
                                .bg(status.soft_color())
                                .text_xs()
                                .text_color(status.color())
                                .child(status.label()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_secondary())
                                .child(format!("{:.0}%", progress_of(task) * 100.)),
                        ),
                );
            let mut body = div()
                .flex()
                .flex_col()
                .gap_1()
                .flex_1()
                .child(title_row)
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_secondary())
                        .child(meta_text(task)),
                )
                .child(progress_bar(
                    &format!("{id:?}-progress-{}", task.id),
                    &format!("{} 进度", task.name),
                    progress_of(task),
                    if status == TransferStatus::Canceled {
                        theme::text_disabled()
                    } else {
                        status.color()
                    },
                ));

            // 行操作
            let mut actions = div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .pt(px(m.actions_padding_top));
            match status {
                TransferStatus::Transferring => {
                    if let Some(on_pause) = self.on_pause.clone() {
                        let key = task_id.clone();
                        actions = actions.child(action_button(
                            ElementId::Name(format!("{id:?}-pause-{ix}").into()),
                            "⏸",
                            format!("暂停 {}", task.name),
                            theme::text_secondary(),
                            move |window, cx| on_pause(&key, window, cx),
                        ));
                    }
                    if let Some(on_cancel) = self.on_cancel.clone() {
                        let key = task_id.clone();
                        actions = actions.child(action_button(
                            ElementId::Name(format!("{id:?}-cancel-{ix}").into()),
                            "✕",
                            format!("取消 {}", task.name),
                            theme::text_secondary(),
                            move |window, cx| on_cancel(&key, window, cx),
                        ));
                    }
                }
                TransferStatus::Paused => {
                    if let Some(on_resume) = self.on_resume.clone() {
                        let key = task_id.clone();
                        actions = actions.child(action_button(
                            ElementId::Name(format!("{id:?}-resume-{ix}").into()),
                            "▶",
                            format!("继续 {}", task.name),
                            theme::primary(),
                            move |window, cx| on_resume(&key, window, cx),
                        ));
                    }
                    if let Some(on_cancel) = self.on_cancel.clone() {
                        let key = task_id.clone();
                        actions = actions.child(action_button(
                            ElementId::Name(format!("{id:?}-cancel-{ix}").into()),
                            "✕",
                            format!("取消 {}", task.name),
                            theme::text_secondary(),
                            move |window, cx| on_cancel(&key, window, cx),
                        ));
                    }
                }
                TransferStatus::Queued => {
                    if let Some(on_cancel) = self.on_cancel.clone() {
                        let key = task_id.clone();
                        actions = actions.child(action_button(
                            ElementId::Name(format!("{id:?}-cancel-{ix}").into()),
                            "✕",
                            format!("取消 {}", task.name),
                            theme::text_secondary(),
                            move |window, cx| on_cancel(&key, window, cx),
                        ));
                    }
                }
                TransferStatus::Failed => {
                    if let Some(on_retry) = self.on_retry.clone() {
                        let key = task_id.clone();
                        actions = actions.child(action_button(
                            ElementId::Name(format!("{id:?}-retry-{ix}").into()),
                            "↻",
                            format!("重试 {}", task.name),
                            theme::primary(),
                            move |window, cx| on_retry(&key, window, cx),
                        ));
                    }
                    if let Some(on_remove) = self.on_remove.clone() {
                        let key = task_id.clone();
                        actions = actions.child(action_button(
                            ElementId::Name(format!("{id:?}-remove-{ix}").into()),
                            "✕",
                            format!("移除 {}", task.name),
                            theme::text_secondary(),
                            move |window, cx| on_remove(&key, window, cx),
                        ));
                    }
                }
                TransferStatus::Done | TransferStatus::Canceled => {
                    if let Some(on_remove) = self.on_remove.clone() {
                        let key = task_id.clone();
                        actions = actions.child(action_button(
                            ElementId::Name(format!("{id:?}-remove-{ix}").into()),
                            "✕",
                            format!("移除 {}", task.name),
                            theme::text_secondary(),
                            move |window, cx| on_remove(&key, window, cx),
                        ));
                    }
                }
            }
            body = body.child(actions);

            let mut row = div()
                .id(ElementId::Name(format!("{id:?}-row-{ix}").into()))
                .flex()
                .flex_row()
                .items_start()
                .gap_2()
                .px_3()
                .py_2()
                .hover(|style| style.bg(theme::control_item_bg_hover()))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(m.direction_icon_size))
                        .h(px(m.direction_icon_size))
                        .rounded_full()
                        .bg(icon_bg)
                        .text_color(icon_fg)
                        .text_sm()
                        .child(task.direction.glyph()),
                )
                .child(body);

            row = a11y::Semantics::new(Role::ListItem, task.name.clone())
                .id(format!("{id:?}-row-{ix}"))
                .description(format!("{} · {}", status.label(), meta_text(task)))
                .value(format!("{:.0}%", progress_of(task) * 100.))
                .position(ix + 1, total)
                .apply(row);
            if !last {
                row = row.border_b_1().border_color(theme::border());
            }
            root = root.child(row);
        }

        root
    }
}

/// 行操作按钮：固定 22×22 图标按钮，悬停浅底
fn action_button(
    id: ElementId,
    glyph: &'static str,
    label: impl Into<SharedString>,
    color: Hsla,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> gpui::Stateful<Div> {
    let size = theme::with(crate::theme::metric::transfer_manager_metric).action_button_size;
    let button = div()
        .id(id.clone())
        .flex()
        .items_center()
        .justify_center()
        .w(px(size))
        .h(px(size))
        .rounded_md()
        .cursor(CursorStyle::PointingHand)
        .text_xs()
        .text_color(color)
        .hover(|style| style.bg(theme::control_item_bg_hover()))
        .child(glyph);
    let button = a11y::Semantics::new(Role::Button, label)
        .id(format!("{id:?}"))
        .focusable(true)
        .apply(button);
    a11y::click_void(button, on_click)
}

/// 细进度条：进度色由调用方指定
fn progress_bar(id: &str, label: &str, progress: f32, color: Hsla) -> gpui::Stateful<Div> {
    let bar_height = theme::with(crate::theme::metric::transfer_manager_metric).bar_height;
    let progress = progress.clamp(0., 1.);
    let bar = div()
        .id(ElementId::Name(id.into()))
        .w_full()
        .h(px(bar_height))
        .rounded_full()
        .bg(theme::fill_secondary())
        .overflow_hidden()
        .child(
            div()
                .h_full()
                .w(relative(progress))
                .rounded_full()
                .bg(color),
        );
    a11y::Semantics::new(Role::ProgressIndicator, label)
        .id(id)
        .numeric(progress as f64)
        .numeric_range(0., 1.)
        .apply(bar)
}

/// 展示用进度：已完成恒为 100%
fn progress_of(task: &TransferTask) -> f32 {
    if task.status == TransferStatus::Done {
        1.
    } else {
        task.progress
    }
}

/// 任务副标题：大小 / 速度 / 剩余时间 / 失败原因
fn meta_text(task: &TransferTask) -> String {
    let progress = progress_of(task);
    let done_bytes = task
        .size
        .map(|size| (size as f64 * progress as f64).round() as u64);
    let mut parts: Vec<String> = Vec::new();

    match task.status {
        TransferStatus::Queued => {
            parts.push("等待中".into());
            if let Some(size) = task.size {
                parts.push(format_size(size));
            }
        }
        TransferStatus::Transferring | TransferStatus::Paused => {
            match (done_bytes, task.size) {
                (Some(done), Some(size)) => {
                    parts.push(format!("{} / {}", format_size(done), format_size(size)))
                }
                (_, Some(size)) => parts.push(format_size(size)),
                _ => {}
            }
            if task.status == TransferStatus::Paused {
                parts.push("已暂停".into());
            } else {
                if let Some(speed) = task.speed {
                    parts.push(format!("{}/s", format_size(speed.max(0.) as u64)));
                }
                if let (Some(speed), Some(size)) = (task.speed, task.size)
                    && speed > 0.
                {
                    let remaining = size as f64 * (1. - progress as f64) / speed;
                    parts.push(format!("剩余 {}", format_eta(remaining.max(0.) as u64)));
                }
            }
        }
        TransferStatus::Done => {
            if let Some(size) = task.size {
                parts.push(format_size(size));
            }
        }
        TransferStatus::Failed => parts.push(
            task.error
                .as_ref()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "传输失败".into()),
        ),
        TransferStatus::Canceled => parts.push("已取消".into()),
    }

    parts.join(" · ")
}

/// 字节数 → 人类可读
fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024. && unit + 1 < UNITS.len() {
        value /= 1024.;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// 剩余秒数 → mm:ss / h:mm:ss / n 秒
fn format_eta(secs: u64) -> String {
    if secs >= 3600 {
        format!("{}:{:02}:{:02}", secs / 3600, secs % 3600 / 60, secs % 60)
    } else if secs >= 60 {
        format!("{}:{:02}", secs / 60, secs % 60)
    } else {
        format!("{secs} 秒")
    }
}
