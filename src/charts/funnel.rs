//! 漏斗图：GPU 路径绘制的居中梯形堆叠 + 阶段标签 + 转化率。
//!
//! ```ignore
//! FunnelChart::new(vec![
//!     FunnelStage::new("访问", 1000.),
//!     FunnelStage::new("注册", 620.),
//!     FunnelStage::new("付费", 280.),
//! ]).height(px(220.))
//! ```

use gpui::{
    App, Bounds, Element, ElementId, GlobalElementId, Hsla, InspectorElementId, IntoElement,
    LayoutId, PathBuilder, Pixels, Role, SharedString, Style, Window, accesskit, point, px,
    relative, white,
};

use crate::a11y;
use crate::chart::chart_id;
use crate::chart_util::{format_value, paint_text, paint_text_centered};
use crate::theme;

/// 漏斗阶段。
pub struct FunnelStage {
    label: SharedString,
    value: f32,
    color: Option<Hsla>,
}

impl FunnelStage {
    pub fn new(label: impl Into<SharedString>, value: f32) -> Self {
        Self {
            label: label.into(),
            value,
            color: None,
        }
    }

    /// 自定义阶段颜色（默认取主题分类色板）。
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// 漏斗图。
#[derive(IntoElement)]
pub struct FunnelChart {
    id: ElementId,
    stages: Vec<FunnelStage>,
    height: Pixels,
    gap: Pixels,
}

impl FunnelChart {
    pub fn new(stages: Vec<FunnelStage>) -> Self {
        let id = chart_id(
            "funnel",
            stages
                .iter()
                .map(|stage| (stage.label.as_ref(), stage.value))
                .collect::<Vec<_>>(),
        );
        Self {
            id,
            stages,
            height: px(220.),
            gap: px(4.),
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    /// 阶段间距。
    pub fn gap(mut self, gap: Pixels) -> Self {
        self.gap = gap;
        self
    }
}

impl gpui::RenderOnce for FunnelChart {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let max = self
            .stages
            .iter()
            .map(|s| s.value.max(0.))
            .fold(0., f32::max)
            .max(1e-6);
        let stages: Vec<ResolvedStage> = self
            .stages
            .iter()
            .enumerate()
            .map(|(i, s)| ResolvedStage {
                label: s.label.clone(),
                value: s.value.max(0.),
                ratio: (s.value.max(0.) / max).clamp(0., 1.),
                color: s.color.unwrap_or_else(|| theme::chart_color(i)),
            })
            .collect();
        FunnelElement {
            id: self.id,
            stages,
            height: self.height,
            gap: self.gap,
        }
    }
}

struct ResolvedStage {
    label: SharedString,
    value: f32,
    ratio: f32,
    color: Hsla,
}

/// 漏斗图内部绘制元素。
struct FunnelElement {
    id: ElementId,
    stages: Vec<ResolvedStage>,
    height: Pixels,
    gap: Pixels,
}

impl FunnelElement {
    fn semantics(&self) -> a11y::Semantics {
        let summary = self
            .stages
            .iter()
            .take(8)
            .enumerate()
            .map(|(index, stage)| {
                let conversion = if index == 0 {
                    String::new()
                } else {
                    let previous = self.stages[index - 1].value.max(1e-6);
                    format!("，转化 {:.0}%", stage.value / previous * 100.)
                };
                format!(
                    "{} {}{}",
                    stage.label,
                    format_value(stage.value),
                    conversion
                )
            })
            .collect::<Vec<_>>()
            .join("；");
        let summary = if self.stages.len() > 8 {
            format!("{summary}；共 {} 阶段", self.stages.len())
        } else {
            summary
        };
        let mut semantics = a11y::Semantics::new(
            Role::Figure,
            format!("漏斗图：{} 阶段；{summary}", self.stages.len()),
        )
        .id(format!("{:?}", self.id));
        if let Some(last) = self.stages.last() {
            semantics = semantics.numeric(last.value as f64);
        }
        semantics
    }
}

impl Element for FunnelElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn a11y_role(&self) -> Option<Role> {
        Some(self.semantics().role())
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        self.semantics().write(node);
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = self.height.into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let n = self.stages.len();
        if n == 0 {
            return;
        }
        let right_pad = px(64.);
        let funnel_w = bounds.size.width - right_pad - px(8.);
        let center_x = bounds.left() + funnel_w / 2.;
        let band_h = (self.height - self.gap * (n as f32 - 1.).max(0.)) / n as f32;
        if funnel_w <= px(0.) || band_h <= px(0.) {
            return;
        }

        for (i, s) in self.stages.iter().enumerate() {
            let top_w = funnel_w * s.ratio.max(0.04);
            let next_ratio = self
                .stages
                .get(i + 1)
                .map(|x| x.ratio)
                .unwrap_or(s.ratio * 0.82);
            let bot_w = funnel_w * next_ratio.max(0.04);
            let y_top = bounds.top() + (band_h + self.gap) * i as f32;
            let y_bot = y_top + band_h;

            let mut trap = PathBuilder::fill();
            trap.move_to(point(center_x - top_w / 2., y_top));
            trap.line_to(point(center_x + top_w / 2., y_top));
            trap.line_to(point(center_x + bot_w / 2., y_bot));
            trap.line_to(point(center_x - bot_w / 2., y_bot));
            trap.close();
            if let Ok(path) = trap.build() {
                window.paint_path(path, s.color);
            }

            // 阶段标签 + 数值（梯形内居中）
            let mid_y = y_top + band_h / 2.;
            paint_text_centered(
                window,
                cx,
                format!("{}  {}", s.label, format_value(s.value)),
                point(center_x, mid_y),
                px(12.),
                white(),
            );

            // 转化率（右侧，相对上一阶段）
            if i > 0 {
                let prev = self.stages[i - 1].value.max(1e-6);
                let pct = (s.value / prev * 100.).clamp(0., 999.);
                paint_text(
                    window,
                    cx,
                    format!("{pct:.0}%"),
                    point(
                        bounds.right() - right_pad + px(10.),
                        y_top - self.gap / 2. - px(6.),
                    ),
                    px(11.),
                    theme::text_muted(),
                );
            }
        }
    }
}
