//! 热力图：矩阵色块 + 行列标签 + 色阶图例，颜色按数值在两端色间插值。
//!
//! ```ignore
//! Heatmap::new(vec![
//!     vec![1., 5., 9.],
//!     vec![3., 7., 2.],
//! ]).row_labels(vec!["周一".into(), "周二".into()])
//!   .col_labels(vec!["早".into(), "中".into(), "晚".into()])
//! ```

use gpui::{
    App, ElementId, Hsla, IntoElement, Pixels, Role, SharedString, Window, div, prelude::*, px,
};

use crate::a11y;
use crate::chart::chart_id;
use crate::chart_util::{format_value, lerp_color};
use crate::theme;

/// 热力图。`values` 为行优先的二维矩阵。
#[derive(IntoElement)]
pub struct Heatmap {
    id: ElementId,
    values: Vec<Vec<f32>>,
    row_labels: Vec<SharedString>,
    col_labels: Vec<SharedString>,
    cell_size: Pixels,
    gap: Pixels,
    color_low: Option<Hsla>,
    color_high: Option<Hsla>,
    legend: bool,
}

impl Heatmap {
    pub fn new(values: Vec<Vec<f32>>) -> Self {
        let id = chart_id("heatmap", &values);
        Self {
            id,
            values,
            row_labels: Vec::new(),
            col_labels: Vec::new(),
            cell_size: px(28.),
            gap: px(3.),
            color_low: None,
            color_high: None,
            legend: true,
        }
    }

    /// 覆盖稳定无障碍 ID（不影响布局）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 行标签（左侧）。
    pub fn row_labels(mut self, labels: Vec<SharedString>) -> Self {
        self.row_labels = labels;
        self
    }

    /// 列标签（顶部）。
    pub fn col_labels(mut self, labels: Vec<SharedString>) -> Self {
        self.col_labels = labels;
        self
    }

    /// 单元格边长。
    pub fn cell_size(mut self, size: Pixels) -> Self {
        self.cell_size = size;
        self
    }

    /// 单元格间距。
    pub fn gap(mut self, gap: Pixels) -> Self {
        self.gap = gap;
        self
    }

    /// 色阶两端颜色（默认主题柔和主色 → 主色）。
    pub fn colors(mut self, low: impl Into<Hsla>, high: impl Into<Hsla>) -> Self {
        self.color_low = Some(low.into());
        self.color_high = Some(high.into());
        self
    }

    /// 是否显示底部色阶图例（默认开）。
    pub fn legend(mut self, on: bool) -> Self {
        self.legend = on;
        self
    }
}

impl gpui::RenderOnce for Heatmap {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let low = self.color_low.unwrap_or_else(theme::accent_soft);
        let high = self.color_high.unwrap_or_else(theme::accent);

        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for row in &self.values {
            for v in row {
                min = min.min(*v);
                max = max.max(*v);
            }
        }
        if !min.is_finite() {
            min = 0.;
            max = 1.;
        }
        let range = (max - min).max(1e-6);

        let has_row_labels = !self.row_labels.is_empty();
        let label_w = if has_row_labels { px(56.) } else { px(0.) };
        let cols = self.values.iter().map(|r| r.len()).max().unwrap_or(0);
        let cell_count = self.values.iter().map(Vec::len).sum::<usize>();
        let a11y_id = format!("{:?}", self.id);

        let mut root = div().flex().flex_col().gap(self.gap);

        // 顶部列标签
        if !self.col_labels.is_empty() {
            let mut header = div().flex().flex_row().items_center().gap(self.gap);
            if has_row_labels {
                header = header.child(div().w(label_w));
            }
            for c in 0..cols {
                let text = self.col_labels.get(c).cloned().unwrap_or_default();
                let header_id = format!("{a11y_id}-column-{c}");
                header = header.child(
                    a11y::Semantics::new(
                        Role::ColumnHeader,
                        if text.is_empty() {
                            format!("第 {} 列", c + 1)
                        } else {
                            text.to_string()
                        },
                    )
                    .id(header_id.clone())
                    .column(c + 1, cols)
                    .apply(
                        div()
                            .id(ElementId::Name(header_id.into()))
                            .w(self.cell_size)
                            .text_center()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child(text),
                    ),
                );
            }
            root = root.child(header);
        }

        // 数据行
        let mut cell_index = 0;
        for (r, row) in self.values.iter().enumerate() {
            let row_name = self
                .row_labels
                .get(r)
                .filter(|label| !label.is_empty())
                .cloned()
                .unwrap_or_else(|| SharedString::from(format!("第 {} 行", r + 1)));
            let row_id = format!("{a11y_id}-row-{r}");
            let mut line = a11y::Semantics::new(Role::List, row_name.clone())
                .id(row_id.clone())
                .row(r + 1, self.values.len())
                .apply(
                    div()
                        .id(ElementId::Name(row_id.into()))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(self.gap),
                );
            if has_row_labels {
                let text = self.row_labels.get(r).cloned().unwrap_or_default();
                line = line.child(
                    div()
                        .w(label_w)
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child(text),
                );
            }
            for (c, v) in row.iter().enumerate() {
                cell_index += 1;
                let t = ((v - min) / range).clamp(0., 1.);
                let cell_id = format!("{a11y_id}-cell-{r}-{c}");
                let column_name = self
                    .col_labels
                    .get(c)
                    .filter(|label| !label.is_empty())
                    .cloned()
                    .unwrap_or_else(|| SharedString::from(format!("第 {} 列", c + 1)));
                line = line.child(
                    a11y::Semantics::new(
                        Role::ListItem,
                        format!("{row_name}，{column_name}：{}", format_value(*v)),
                    )
                    .id(cell_id.clone())
                    .numeric(*v as f64)
                    .row(r + 1, self.values.len())
                    .column(c + 1, cols)
                    .position(cell_index, cell_count)
                    .apply(
                        div()
                            .id(ElementId::Name(cell_id.into()))
                            .w(self.cell_size)
                            .h(self.cell_size)
                            .rounded(px(4.))
                            .bg(lerp_color(low, high, t)),
                    ),
                );
            }
            root = root.child(line);
        }

        // 色阶图例
        if self.legend {
            let mut legend = div().flex().flex_row().items_center().gap_2().mt_2().child(
                div()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child(format_value(min)),
            );
            let mut scale = div().flex().flex_row().gap(px(1.));
            for i in 0..10 {
                let t = i as f32 / 9.;
                scale = scale.child(
                    div()
                        .w(px(16.))
                        .h(px(8.))
                        .rounded(px(2.))
                        .bg(lerp_color(low, high, t)),
                );
            }
            legend = legend.child(scale).child(
                div()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child(format_value(max)),
            );
            root = root.child(legend);
        }

        let summary = self
            .values
            .iter()
            .take(4)
            .map(|row| {
                row.iter()
                    .take(6)
                    .map(|value| format_value(*value))
                    .collect::<Vec<_>>()
                    .join("、")
            })
            .collect::<Vec<_>>()
            .join("；");
        a11y::Semantics::new(
            Role::Figure,
            format!(
                "热力图：{} 行、最多 {cols} 列、{cell_count} 个单元格，最小值 {}，最大值 {}；{summary}",
                self.values.len(),
                format_value(min),
                format_value(max)
            ),
        )
        .id(a11y_id.clone())
        .apply(root.id(ElementId::Name(a11y_id.into())))
    }
}
