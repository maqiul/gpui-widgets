//! Table 矩阵：表头 / 行态 / 边框取色与单元格几何。
//!
//! 基准由 `harness/antd_baseline/gen_table_matrix.mjs` 依据 antdv-next
//! `table/style/index.js` 的组件 token 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{TableMetric, table_style},
};

const BASELINE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/harness/antd_baseline/");

#[derive(Clone, Copy, Debug)]
struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

fn rgba(color: Hsla) -> Rgba {
    let color = Color::from(color);
    Rgba {
        r: color.r,
        g: color.g,
        b: color.b,
        a: color.a,
    }
}

fn parse(text: &str) -> Rgba {
    let text = text.trim();
    if let Some(hex) = text.strip_prefix('#') {
        let s = if hex.len() == 3 {
            hex.chars().flat_map(|c| [c, c]).collect::<String>()
        } else {
            hex.to_string()
        };
        let v = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).unwrap() as f32;
        return Rgba {
            r: v(0),
            g: v(2),
            b: v(4),
            a: 1.,
        };
    }
    let body = text
        .trim_start_matches("rgb(")
        .trim_start_matches("rgba(")
        .trim_end_matches(')');
    let mut parts = body.split(',').map(|p| {
        p.trim()
            .parse::<f32>()
            .unwrap_or_else(|_| panic!("坏颜色 {text:?}"))
    });
    Rgba {
        r: parts.next().unwrap(),
        g: parts.next().unwrap(),
        b: parts.next().unwrap(),
        a: parts.next().unwrap_or(1.0),
    }
}

fn assert_close(actual: Hsla, expected: Rgba, what: &str, row: &str) {
    let a = rgba(actual);
    let near = (a.r - expected.r).abs() <= 1.0
        && (a.g - expected.g).abs() <= 1.0
        && (a.b - expected.b).abs() <= 1.0
        && (a.a - expected.a).abs() <= 0.01;
    assert!(
        near,
        "{row} {what}: got rgba({},{},{},{}), want rgba({},{},{},{})",
        a.r, a.g, a.b, a.a, expected.r, expected.g, expected.b, expected.a
    );
}

fn assert_f32(actual: f32, expected: &str, what: &str, row: &str) {
    let expected = expected.parse::<f32>().unwrap();
    assert!(
        (actual - expected).abs() < 1e-4,
        "{row} {what}: got {actual}, want {expected}"
    );
}

fn check(tokens: &Tokens, suffix: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}table_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 15, "坏行: {line}");

    let style = table_style(tokens);
    let m = TableMetric::resolve(tokens);
    assert_close(style.header_bg, parse(p[0]), "表头底色", line);
    assert_close(style.header_color, parse(p[1]), "表头文字色", line);
    assert_close(style.border_color, parse(p[2]), "边框色", line);
    assert_close(style.row_hover_bg, parse(p[3]), "行悬停底色", line);
    assert_close(style.row_selected_bg, parse(p[4]), "选中行底色", line);
    assert_close(
        style.row_selected_hover_bg,
        parse(p[5]),
        "选中行悬停底色",
        line,
    );
    assert_close(style.footer_bg, parse(p[6]), "表尾底色", line);
    assert_close(style.footer_color, parse(p[7]), "表尾文字色", line);
    assert_f32(m.cell_padding_block, p[8], "单元格纵向内边距", line);
    assert_f32(m.cell_padding_inline, p[9], "单元格横向内边距", line);
    assert_f32(m.font_size, p[10], "单元格字号", line);
    assert_f32(m.line_height, p[11], "单元格行高", line);
    assert_f32(m.selection_column_width, p[12], "勾选列宽", line);
    assert_f32(m.expand_column_width, p[13], "展开列宽", line);
    assert_f32(m.radius, p[14], "表头圆角", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn table_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn table_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn table_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn table_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = TableMetric::resolve(&tokens);
    assert_eq!(m.cell_padding_block, 16.0);
    assert_eq!(m.cell_padding_inline, 16.0);
    assert_eq!(m.line_height, 22.0);
    assert_eq!(m.selection_column_width, 32.0);
    assert_eq!(m.expand_column_width, 48.0);
    assert_eq!(m.radius, 8.0);
}
