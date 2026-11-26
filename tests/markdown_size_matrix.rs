//! Markdown 字号覆盖度量矩阵（0.1.9 ⑨）。
//! 只测 `MarkdownMetric::with_font_size` 纯函数：不建 Window / 不渲染。
//! 约定：标题同比缩放、额外行距（line_height - font_size）不变、控件几何不跟随。
//! 不调用覆盖时度量必须与改前逐像素一致，670 组基线不受影响。

use gpui_widgets::theme::{
    Appearance, Seed, derive,
    metric::{MarkdownMetric, markdown_compact_metric, markdown_metric},
};

fn base() -> MarkdownMetric {
    markdown_metric(&derive(&Seed::default(), Appearance::Light))
}

fn compact() -> MarkdownMetric {
    markdown_compact_metric(&derive(&Seed::default(), Appearance::Light))
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn markdown_size_identity_keeps_font_and_headings() {
    let m = base();
    let same = m.with_font_size(m.font_size);
    assert_eq!(same.font_size, m.font_size);
    assert_eq!(same.heading_sizes, m.heading_sizes);
    assert!(close(same.line_height, m.line_height));
    for i in 0..6 {
        assert!(close(
            same.heading_line_heights[i],
            m.heading_line_heights[i]
        ));
    }
}

#[test]
fn markdown_size_body_override_applies() {
    let m = base();
    let scaled = m.with_font_size(12.0);
    assert_eq!(scaled.font_size, 12.0);
    assert!(close(
        scaled.line_height,
        12.0 + (m.line_height - m.font_size)
    ));
}

#[test]
fn markdown_size_extra_leading_preserved() {
    let m = base();
    let extra = m.line_height - m.font_size;
    let scaled = m.with_font_size(20.0);
    assert!(close(scaled.line_height - scaled.font_size, extra));
    for i in 0..6 {
        assert!(
            close(
                scaled.heading_line_heights[i] - scaled.heading_sizes[i],
                extra
            ),
            "h{} 的额外行距不该随字号放大",
            i + 1
        );
    }
}

#[test]
fn markdown_size_headings_scale_proportionally() {
    let m = base();
    let size = 12.0;
    let ratio = size / m.font_size;
    let scaled = m.with_font_size(size);
    for i in 0..6 {
        assert!(
            close(scaled.heading_sizes[i], m.heading_sizes[i] * ratio),
            "h{} 未按同比例缩放",
            i + 1
        );
    }
    // h6 与正文同号的既有约定在覆盖后仍成立
    assert!(close(scaled.heading_sizes[5], scaled.font_size));
}

#[test]
fn markdown_size_compact_ratios_preserved() {
    let m = compact();
    let scaled = m.with_font_size(10.0);
    for i in 0..6 {
        assert!(
            close(
                scaled.heading_sizes[i] / scaled.font_size,
                m.heading_sizes[i] / m.font_size
            ),
            "compact 的 h{} 相对正文比例被改变了",
            i + 1
        );
    }
}

#[test]
fn markdown_size_geometry_not_scaled() {
    let m = base();
    let scaled = m.with_font_size(24.0);
    // 间距与勾选框是控件几何，不跟字号走（用户明确选定）
    assert_eq!(scaled.block_gap, m.block_gap);
    assert_eq!(scaled.list_gap, m.list_gap);
    assert_eq!(scaled.bullet_width, m.bullet_width);
    assert_eq!(scaled.quote_padding, m.quote_padding);
    assert_eq!(scaled.quote_border, m.quote_border);
    assert_eq!(scaled.rule_thickness, m.rule_thickness);
    assert_eq!(scaled.link_thickness, m.link_thickness);
    assert_eq!(scaled.table_cell_padding, m.table_cell_padding);
    assert_eq!(scaled.table_border, m.table_border);
    assert_eq!(scaled.task_box_size, m.task_box_size);
    assert_eq!(scaled.task_check_size, m.task_check_size);
}

#[test]
fn markdown_size_task_box_centering_still_derives() {
    // 渲染层用 (line_height - task_box_size) / 2 做垂直居中余量：
    // 行高跟随字号后该余量自动变大，勾选框本身不变，无需额外夹取。
    let m = base();
    let scaled = m.with_font_size(24.0);
    assert!(scaled.line_height > m.line_height);
    assert_eq!(scaled.task_box_size, m.task_box_size);
    assert!(
        (scaled.line_height - scaled.task_box_size) / 2.0 > (m.line_height - m.task_box_size) / 2.0
    );
}

#[test]
fn markdown_size_monotonic_in_size() {
    let m = base();
    let small = m.with_font_size(10.0);
    let large = m.with_font_size(24.0);
    assert!(small.font_size < large.font_size);
    assert!(small.line_height < large.line_height);
    for i in 0..6 {
        assert!(small.heading_sizes[i] < large.heading_sizes[i]);
    }
}

#[test]
fn markdown_size_theme_independent() {
    let light = markdown_metric(&derive(&Seed::default(), Appearance::Light)).with_font_size(12.0);
    let dark = markdown_metric(&derive(&Seed::default(), Appearance::Dark)).with_font_size(12.0);
    assert_eq!(light.font_size, dark.font_size);
    assert_eq!(light.line_height, dark.line_height);
    assert_eq!(light.heading_sizes, dark.heading_sizes);
}
