//! Button 度量回归：三档尺寸的高度 / 内边距 / 字号 / 圆角 / 线宽 / 图标间距必须与
//! antdv-next 真实渲染出的盒模型一致。
//!
//! 基准来自 `harness/antd_baseline/button_matrix_*.txt` 里的 `GEO|` 行，由浏览器读取
//! computed style 生成，格式：
//!
//! ```text
//! GEO|尺寸|高度|水平内边距|字号|圆角|线宽|边框样式|图标间距|字重
//! ```

use gpui_widgets::theme::metric::ButtonMetric;
use gpui_widgets::theme::{Appearance, Seed, Tokens, derive};

const BASELINE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/harness/antd_baseline/");

fn number(text: &str) -> f32 {
    text.trim()
        .trim_end_matches("px")
        .parse()
        .unwrap_or_else(|e| panic!("无法解析度量 {text:?}: {e}"))
}

fn size_index(text: &str) -> usize {
    match text {
        "small" => 0,
        "middle" => 1,
        "large" => 2,
        other => panic!("未知尺寸 {other}"),
    }
}

fn check(baseline: &str, tokens: &Tokens) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}{baseline}"))
        .unwrap_or_else(|e| panic!("读取基准 {baseline} 失败: {e}"));
    let metric = ButtonMetric::resolve(tokens);
    let mut rows = 0;
    for line in text.lines().filter(|l| l.starts_with("GEO|")) {
        let col: Vec<&str> = line.split('|').collect();
        assert_eq!(col.len(), 10, "度量行格式异常: {line}");
        let i = size_index(col[1]);

        for (what, got, want) in [
            ("高度", metric.height[i], number(col[2])),
            ("水平内边距", metric.padding_inline[i], number(col[3])),
            ("字号", metric.font_size[i], number(col[4])),
            ("圆角", metric.radius[i], number(col[5])),
            ("线宽", metric.border_width, number(col[6])),
            ("图标间距", metric.icon_gap, number(col[8])),
            ("字重", metric.font_weight.0, number(col[9])),
        ] {
            assert!(
                (got - want).abs() <= 0.01,
                "{what} 不匹配\n  基准行: {line}\n  期望: {want}\n  实际: {got}"
            );
        }
        assert_eq!(col[7], "normal", "基准行出现了非常规边框样式: {line}");
        rows += 1;
    }
    assert_eq!(rows, 3, "{baseline} 应有 small / middle / large 三行度量");
}

#[test]
fn button_light_geometry() {
    let tokens = derive(&Seed::default(), Appearance::Light);
    check("button_matrix_light.txt", &tokens);
}

/// 深色主题只换色板，盒模型必须与浅色完全一致
#[test]
fn button_dark_geometry() {
    let tokens = derive(&Seed::default(), Appearance::Dark);
    check("button_matrix_dark.txt", &tokens);
}

/// 换主色同样不能改变尺寸
#[test]
fn button_violet_geometry() {
    let mut seed = Seed::default();
    seed.color_primary = gpui_widgets::theme::Color::from_hex(0x8b5cf6);
    check(
        "button_matrix_violet.txt",
        &derive(&seed, Appearance::Light),
    );
}

/// antd 的 `paddingInline` 是 `sizeMS - lineWidth`（小按钮用固定的 8 - lineWidth），
/// 换尺寸种子时这条关系必须继续成立，而不是写死 15px。
#[test]
fn button_padding_tracks_size_token() {
    let mut seed = Seed::default();
    // sizeMS = sizeUnit × 4 = 12，配合 2px 线宽得到 10px 内边距
    seed.size_unit = 3.0;
    seed.line_width = 2.0;
    let metric = ButtonMetric::resolve(&derive(&seed, Appearance::Light));
    assert_eq!(metric.border_width, 2.0);
    assert_eq!(metric.padding_inline[1], 10.0);
    assert_eq!(metric.padding_inline[2], 10.0);
    assert_eq!(metric.padding_inline[0], 6.0);
}
