//! 代码块字号覆盖 / 跟随 Markdown 的度量矩阵（0.1.9 ⑨）。
//! 只测 `CodeBlockMetric` 的两个纯函数：不建 Window / 不渲染。
//! 口径：额外行距（line_height - font_size = 7）不变；跟随 Markdown 时**同比缩放**而非
//! 直接取覆盖值，以保住「代码比正文小一档」（本库 14/13，原版 AiMarkdown 12/11）。

use gpui_widgets::theme::{
    Appearance, Seed, derive,
    metric::{code_block_metric, markdown_metric},
};

fn base() -> gpui_widgets::theme::metric::CodeBlockMetric {
    code_block_metric(&derive(&Seed::default(), Appearance::Light))
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn code_block_size_default_is_thirteen_twenty() {
    let m = base();
    assert_eq!(m.font_size, 13.0);
    assert_eq!(m.line_height, 20.0);
    assert_eq!(m.action_gap, 4.0);
}

#[test]
fn code_block_size_identity_unchanged() {
    let m = base();
    let same = m.with_font_size(m.font_size);
    assert_eq!(same.font_size, m.font_size);
    assert!(close(same.line_height, m.line_height));
    assert_eq!(same.action_gap, m.action_gap);
}

#[test]
fn code_block_size_extra_leading_preserved() {
    let m = base();
    let extra = m.line_height - m.font_size;
    assert_eq!(extra, 7.0);
    for size in [10.0_f32, 11.0, 12.0, 16.0, 24.0] {
        let scaled = m.with_font_size(size);
        assert_eq!(scaled.font_size, size);
        assert!(
            close(scaled.line_height - size, extra),
            "字号 {size} 行距变了"
        );
    }
}

#[test]
fn code_block_size_action_gap_not_scaled() {
    let m = base();
    // 按钮间距是控件几何，不跟字号走
    assert_eq!(m.with_font_size(24.0).action_gap, m.action_gap);
}

#[test]
fn code_block_follows_markdown_identity() {
    let md = markdown_metric(&derive(&Seed::default(), Appearance::Light));
    let code = base().following_markdown(md.font_size, md.font_size);
    assert!(close(code.font_size, 13.0), "覆盖值等于基态时代码块不该变");
    assert!(close(code.line_height, 20.0));
}

#[test]
fn code_block_follows_markdown_proportional_not_absolute() {
    let md = markdown_metric(&derive(&Seed::default(), Appearance::Light));
    let size = 12.0;
    let code = base().following_markdown(md.font_size, size);
    let expected = 13.0 * size / md.font_size;
    assert!(close(code.font_size, expected));
    // 关键：不是直接取覆盖值 12，而是同比缩到 ~11.14，仍比正文小一档
    assert!(code.font_size < size);
    assert!(!close(code.font_size, size));
}

#[test]
fn code_block_follows_markdown_keeps_smaller_than_body() {
    let md = markdown_metric(&derive(&Seed::default(), Appearance::Light));
    for size in [10.0_f32, 12.0, 14.0, 18.0, 24.0] {
        let code = base().following_markdown(md.font_size, size);
        assert!(code.font_size < size, "字号 {size} 时代码块不再小于正文");
        assert!(close(code.font_size / size, 13.0 / md.font_size));
    }
}

#[test]
fn code_block_follows_markdown_matches_original_ratio() {
    // 原版 AiMarkdown.vue：正文 12px、代码 11px（.ai-md :deep(code)）。
    // 宿主把 AI 字号设成原版默认 12 时，同比缩放应落在 11 附近。
    let md = markdown_metric(&derive(&Seed::default(), Appearance::Light));
    let code = base().following_markdown(md.font_size, 12.0);
    assert!(
        (code.font_size - 11.0).abs() < 0.2,
        "与原版 11px 偏差过大：{}",
        code.font_size
    );
}

#[test]
fn code_block_follows_markdown_line_height_derived() {
    let md = markdown_metric(&derive(&Seed::default(), Appearance::Light));
    let code = base().following_markdown(md.font_size, 12.0);
    assert!(close(code.line_height - code.font_size, 7.0));
}
