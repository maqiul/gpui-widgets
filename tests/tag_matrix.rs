//! Tag 状态矩阵回归：颜色 × 变体 × 禁用 → 底色 / 文字 / 描边。
//!
//! 基准 `harness/antd_baseline/tag_matrix_*.txt` 由 `gen_tag_matrix.mjs` 依据
//! antdv-next 的 Tag 样式规则 + antd 实测 tokens + `@ant-design/colors` 预置色阶生成。
//! 每行：`颜色|变体|禁用|底色|文字色|描边色`。

use gpui::Hsla;
use gpui_widgets::tag::{TagColor, TagVariant, state};
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive};

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
    if text == "transparent" {
        return Rgba {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        };
    }
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
    let mut parts = body.split(',').map(|p| p.trim().parse::<f32>().unwrap());
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

fn color_of(name: &str) -> TagColor {
    match name {
        "default" => TagColor::Default,
        "blue" => TagColor::Blue,
        "green" => TagColor::Green,
        "orange" => TagColor::Orange,
        "red" => TagColor::Red,
        "purple" => TagColor::Purple,
        "cyan" => TagColor::Cyan,
        "magenta" => TagColor::Magenta,
        "success" => TagColor::Success,
        "processing" => TagColor::Processing,
        "error" => TagColor::Error,
        "warning" => TagColor::Warning,
        other => panic!("未知颜色 {other}"),
    }
}

fn variant_of(name: &str) -> TagVariant {
    match name {
        "outlined" => TagVariant::Outlined,
        "filled" => TagVariant::Filled,
        "solid" => TagVariant::Solid,
        other => panic!("未知变体 {other}"),
    }
}

fn check(tokens: &Tokens, file: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}{file}")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let parts: Vec<&str> = line.split('|').collect();
        assert_eq!(parts.len(), 6, "坏行: {line}");
        let color = color_of(parts[0]);
        let variant = variant_of(parts[1]);
        let disabled = parts[2] == "1";
        let style = state(color, variant, tokens, disabled);
        assert_close(style.bg, parse(parts[3]), "底色", line);
        assert_close(style.fg, parse(parts[4]), "文字色", line);
        assert_close(style.border, parse(parts[5]), "描边色", line);
        checked += 1;
    }
    assert_eq!(checked, 72, "{file} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn tag_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "tag_matrix_light.txt");
}

#[test]
fn tag_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "tag_matrix_dark.txt");
}

#[test]
fn tag_violet_matrix() {
    check(
        &tokens_for(Some(0x8b5cf6), Appearance::Light),
        "tag_matrix_violet.txt",
    );
}

/// 半径 / 内边距 / 行高 / 关闭图标度量取 Tag 令牌
#[test]
fn tag_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = gpui_widgets::theme::metric::TagMetric::resolve(&tokens);
    assert_eq!(m.padding_inline, 7.0);
    assert_eq!(m.font_size, tokens.font_size_sm);
    assert_eq!(m.line_height, tokens.line_height_sm * tokens.font_size_sm);
    assert_eq!(m.radius, tokens.border_radius_sm);
    assert_eq!(m.icon_margin, tokens.padding_xxs - tokens.line_width);
    assert_eq!(m.icon_size, tokens.font_size_icon - tokens.line_width * 2.0);
    assert_eq!(m.border_width, tokens.line_width);
}
