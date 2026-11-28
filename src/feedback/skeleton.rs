//! 骨架屏：加载占位块（对应 antd Skeleton）。

use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, ElementId, Hsla, IntoElement, Pixels, RenderOnce, Role,
    div, prelude::*, px, relative,
};

use crate::a11y::Semantics;
use crate::theme::{self, Tokens, metric};

/// 骨架屏占位块外观（antd Skeleton token）：底色 `colorFillContent`，
/// 非圆形块圆角 `borderRadiusSM`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkeletonStyle {
    pub block: Hsla,
    pub radius: f32,
}

pub fn skeleton_state(tokens: &Tokens) -> SkeletonStyle {
    let m = metric::SkeletonMetric::resolve(tokens);
    SkeletonStyle {
        block: tokens.color_fill_content,
        radius: m.radius,
    }
}

/// 骨架屏占位块。
///
/// ```ignore
/// Skeleton::new().w(px(200.))              // 文本条
/// Skeleton::circle(px(40.))                // 圆形头像占位
/// Skeleton::lines(3)                       // 三行文本占位
/// Skeleton::new().active()                 // 呼吸动画
/// ```
pub struct Skeleton {
    width: Option<Pixels>,
    height: Option<Pixels>,
    circle: bool,
    percent: Option<f32>,
    active: bool,
}

impl Skeleton {
    /// 默认占满宽度、高 16px 的圆角条
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            circle: false,
            percent: None,
            active: false,
        }
    }

    /// 圆形占位（直径 size）
    pub fn circle(size: Pixels) -> Self {
        Self {
            width: Some(size),
            height: Some(size),
            circle: true,
            percent: None,
            active: false,
        }
    }

    /// 多行文本占位：行距 16，三行及以上末行 61% 宽（antd 规则）
    pub fn lines(count: usize) -> SkeletonLines {
        SkeletonLines { count }
    }

    pub fn w(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// 宽度百分比（如 60. 表示 60%）
    pub fn w_percent(mut self, percent: f32) -> Self {
        self.percent = Some(percent.clamp(1., 100.) / 100.);
        self
    }

    pub fn h(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    /// 打开呼吸动画（antd `active`；antd 是位移渐变，这里是透明度近似）
    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

/// 呼吸动画的透明度（纯函数）：`active` 关闭时返回 `None`，外观与 `delta` 无关。
///
/// 这条不变量是「非 active 不挂动画」的依据：既然 delta 不参与绘制，
/// 就不该让 `.repeat()` 动画每帧 `request_animation_frame()` 白烧整窗重绘。
fn breathing_opacity(active: bool, delta: f32) -> Option<f32> {
    if !active {
        return None;
    }
    Some(crate::motion::pulse(delta, 0.45, 1.0))
}

impl IntoElement for Skeleton {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        // 动画包在内部类型上，避免对自身 with_animation 造成递归包装
        let block = SkeletonBlock {
            width: self.width,
            height: self.height,
            circle: self.circle,
            percent: self.percent,
            active: self.active,
            delta: 0.,
        };
        if self.active {
            block
                .with_animation(
                    gpui::ElementId::Name("skeleton".into()),
                    Animation::new(Duration::from_millis(1200))
                        .repeat()
                        .with_easing(gpui::ease_in_out),
                    |mut this, delta| {
                        this.delta = delta;
                        this
                    },
                )
                .into_any_element()
        } else {
            block.into_any_element()
        }
    }
}

/// 实际绘制的占位块（`active` 时被 AnimationElement 包裹，否则直接绘制）
pub struct SkeletonBlock {
    width: Option<Pixels>,
    height: Option<Pixels>,
    circle: bool,
    percent: Option<f32>,
    active: bool,
    delta: f32,
}

impl IntoElement for SkeletonBlock {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let (style, m) = theme::with(|t| (skeleton_state(t), metric::SkeletonMetric::resolve(t)));
        let mut block = div().h(self.height.unwrap_or(px(m.height))).bg(style.block);
        block = if self.circle {
            block.rounded_full()
        } else {
            block.rounded(px(style.radius))
        };
        if let Some(opacity) = breathing_opacity(self.active, self.delta) {
            block = block.opacity(opacity);
        }
        match (self.percent, self.width) {
            (Some(p), _) => block.w(relative(p)),
            (None, Some(w)) => block.w(w),
            (None, None) => block.w_full(),
        }
    }
}

/// 多行文本占位容器。
#[derive(IntoElement)]
pub struct SkeletonLines {
    count: usize,
}

impl RenderOnce for SkeletonLines {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let m = theme::with(metric::SkeletonMetric::resolve);
        let count = self.count.max(1);
        let id = format!("skeleton-lines-{count}");
        let mut root = div()
            .id(ElementId::Name(id.clone().into()))
            .flex()
            .flex_col()
            .gap(px(m.gap))
            .w_full();
        for ix in 0..count {
            let mut line = Skeleton::new();
            if count >= 3 && ix + 1 == count {
                line = line.w_percent(m.last_line_ratio * 100.);
            }
            root = root.child(line);
        }
        Semantics::new(Role::Status, "正在加载").id(id).apply(root)
    }
}

#[cfg(test)]
mod tests {
    use super::breathing_opacity;

    const DELTAS: [f32; 11] = [0., 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.];

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn skeleton_inactive_ignores_delta() {
        // 本轮修复的依据：非 active 时外观与 delta 无关 ⇒ 不挂动画不改变任何一帧
        for d in DELTAS {
            assert_eq!(breathing_opacity(false, d), None);
        }
    }

    #[test]
    fn skeleton_active_always_yields_opacity() {
        for d in DELTAS {
            assert!(breathing_opacity(true, d).is_some());
        }
    }

    #[test]
    fn skeleton_opacity_endpoints_and_peak() {
        assert!(close(breathing_opacity(true, 0.).unwrap(), 0.45));
        assert!(close(breathing_opacity(true, 0.5).unwrap(), 1.0));
        assert!(close(breathing_opacity(true, 1.).unwrap(), 0.45));
    }

    #[test]
    fn skeleton_opacity_stays_in_range() {
        for i in 0..=100 {
            let o = breathing_opacity(true, i as f32 / 100.).unwrap();
            assert!(
                (0.45..=1.0).contains(&o),
                "delta={} 越界 {o}",
                i as f32 / 100.
            );
        }
    }

    #[test]
    fn skeleton_opacity_rises_then_falls() {
        let up: Vec<f32> = (0..50)
            .map(|i| breathing_opacity(true, i as f32 / 100.).unwrap())
            .collect();
        assert!(up.windows(2).all(|w| w[1] > w[0]));
        let down: Vec<f32> = (51..=100)
            .map(|i| breathing_opacity(true, i as f32 / 100.).unwrap())
            .collect();
        assert!(down.windows(2).all(|w| w[1] < w[0]));
    }

    #[test]
    fn skeleton_opacity_symmetric_around_peak() {
        for i in 0..=50 {
            let d = i as f32 / 100.;
            assert!(close(
                breathing_opacity(true, d).unwrap(),
                breathing_opacity(true, 1. - d).unwrap()
            ));
        }
    }

    #[test]
    fn skeleton_opacity_stable_across_repeated_calls() {
        // 值组件每帧重建：同输入必须同输出
        for _ in 0..3 {
            for d in DELTAS {
                assert_eq!(breathing_opacity(true, d), breathing_opacity(true, d));
                assert_eq!(breathing_opacity(false, d), breathing_opacity(false, d));
            }
        }
    }

    #[test]
    fn skeleton_active_differs_from_inactive_at_every_delta() {
        for d in DELTAS {
            assert_ne!(breathing_opacity(true, d), breathing_opacity(false, d));
        }
    }
}
