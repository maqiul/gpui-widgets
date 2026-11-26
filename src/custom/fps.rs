//! 帧率 / 性能监视器：实测渲染 FPS 与帧耗时，最高支持 120fps（对标高性能演示需求）。
//!
//! 本身是一个 `Entity`。开启后通过 [`gpui::Window::request_animation_frame`] 驱动连续渲染，
//! 该调用**只 notify 当前视图**，因此高频刷新仅重绘本控件这一小子树，不会带动整页重绘，
//! 这是高帧率下保持低开销的关键。
//!
//! 帧率上限由显示器刷新率（vsync）决定：GPUI 在 Windows 上读取 DWM 合成周期，
//! 120Hz 屏幕即可跑到 ~120fps；动画均为时间基（delta-time），高刷新率下依然平滑。
//!
//! ```ignore
//! let meter = cx.new(|cx| FpsMeter::new(cx));          // 默认开启，上限 120fps
//! let meter = cx.new(|cx| FpsMeter::new(cx).max_fps(120).running(false));
//! // render 中：div().child(meter.clone())
//! // 控制：meter.update(cx, |m, cx| m.start(cx)); / m.stop(cx);
//! ```

use std::collections::VecDeque;
use std::time::Instant;

use gpui::{Context, ElementId, IntoElement, Render, Role, Window, div, prelude::*, px};

use crate::{Sparkline, a11y, theme};

/// 帧耗时采样窗口（用于平滑 FPS 与绘制曲线）。
const SAMPLES: usize = 60;

/// 帧率 / 性能监视器。受控组件（内部自驱动）。
pub struct FpsMeter {
    running: bool,
    max_fps: u32,
    last: Option<Instant>,
    /// 最近若干帧的耗时（毫秒）。
    samples: VecDeque<f32>,
    fps: f32,
    frame_ms: f32,
}

impl FpsMeter {
    /// 创建监视器，默认开启、上限 120fps。
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            running: true,
            max_fps: 120,
            last: None,
            samples: VecDeque::with_capacity(SAMPLES),
            fps: 0.,
            frame_ms: 0.,
        }
    }

    /// 是否开启连续刷新。
    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    /// 标称帧率上限（默认 120）。实际帧率跟随显示器 vsync。
    pub fn max_fps(mut self, max_fps: u32) -> Self {
        self.max_fps = max_fps.max(1);
        self
    }

    /// 开始连续刷新并重置采样。
    pub fn start(&mut self, cx: &mut Context<Self>) {
        self.running = true;
        self.last = None;
        self.samples.clear();
        cx.notify();
    }

    /// 停止连续刷新。
    pub fn stop(&mut self, cx: &mut Context<Self>) {
        self.running = false;
        self.last = None;
        cx.notify();
    }

    /// 当前平滑 FPS。
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// 当前平均帧耗时（毫秒）。
    pub fn frame_ms(&self) -> f32 {
        self.frame_ms
    }

    /// 是否正在连续刷新。
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// 依据帧率选择状态色：流畅 / 一般 / 卡顿。
    fn status_color(&self) -> gpui::Hsla {
        if self.fps >= 55. {
            theme::success()
        } else if self.fps >= 30. {
            theme::warning()
        } else {
            theme::danger()
        }
    }
}

impl Render for FpsMeter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let m = theme::with(crate::theme::metric::fps_metric);
        // 采样与驱动：仅在本视图内连续刷新，避免整页重绘。
        if self.running {
            let now = Instant::now();
            if let Some(last) = self.last {
                let ms = (now - last).as_secs_f32() * 1000.;
                if ms > 0. {
                    self.samples.push_back(ms);
                    if self.samples.len() > SAMPLES {
                        self.samples.pop_front();
                    }
                    let avg = self.samples.iter().sum::<f32>() / self.samples.len() as f32;
                    self.frame_ms = avg;
                    self.fps = 1000. / avg;
                }
            }
            self.last = Some(now);
            // 只 notify 本实体，驱动下一帧（vsync 跟随显示器刷新率）。
            window.request_animation_frame();
        } else {
            self.last = None;
        }

        let color = self.status_color();
        let curve: Vec<f32> = self.samples.iter().copied().collect();
        let id = ElementId::Name(format!("fps-meter-{:?}", cx.entity_id()).into());
        let semantics = a11y::Semantics::new(Role::Meter, "帧率监视器")
            .id(format!("fps-meter-{:?}", cx.entity_id()))
            .numeric(self.fps as f64)
            .numeric_range(0., self.max_fps as f64)
            .value(format!(
                "{:.2} ms/帧，{}",
                self.frame_ms,
                if self.running {
                    "运行中"
                } else {
                    "已停止"
                }
            ))
            .read_only(true);

        let root = div()
            .id(id)
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .bg(theme::bg_container())
            .border_1()
            .border_color(theme::border())
            .rounded_lg()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_4()
                    // 左：FPS 大数字 + 帧耗时
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_end()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(m.value_font_size))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(color)
                                            .child(format!("{:.0}", self.fps)),
                                    )
                                    .child(
                                        div()
                                            .pb(px(m.unit_padding_bottom))
                                            .text_sm()
                                            .text_color(theme::text_secondary())
                                            .child("FPS"),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_secondary())
                                    .child(format!("{:.2} ms/帧", self.frame_ms)),
                            ),
                    )
                    // 右：帧耗时曲线
                    .child(
                        div().flex_1().min_w(px(0.)).child(
                            Sparkline::new(curve)
                                .w(px(m.graph_width))
                                .h(px(m.graph_height))
                                .color(color)
                                .area(true),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .w(px(m.dot_size))
                            .h(px(m.dot_size))
                            .rounded_full()
                            .bg(if self.running { color } else { theme::border() }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .child(format!(
                                "{} · 上限 {}fps · 仅重绘本控件 · 时间基动画",
                                if self.running {
                                    "运行中"
                                } else {
                                    "已停止"
                                },
                                self.max_fps
                            )),
                    ),
            );

        semantics.apply(root)
    }
}
