//! Breadcrumb 矩阵：单一展示状态（非末项 / 末项 / 分隔符）。
//!
//! 基准由 `harness/antd_baseline/gen_breadcrumb_matrix.mjs` 依据 antdv-next
//! Breadcrumb 样式生成，关键值已用 `harness/_breadcrumb_probe.html` 核对。

use gpui::Hsla;
use gpui_widgets::breadcrumb::breadcrumb_state;
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
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}breadcrumb_matrix_{suffix}.txt")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 6, "坏行: {line}");
        let style = breadcrumb_state(tokens);
        assert_close(style.link, parse(p[0]), "非末项色", line);
        assert_close(style.current, parse(p[1]), "末项色", line);
        assert_close(style.separator, parse(p[2]), "分隔符色", line);
        assert_f32(style.font_size, p[3], "字号", line);
        assert_f32(style.line_height, p[4], "行高", line);
        assert_f32(style.gap, p[5], "间距", line);
        checked += 1;
    }
    assert_eq!(checked, 1, "breadcrumb_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn breadcrumb_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn breadcrumb_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn breadcrumb_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 字号 14 / 行高 22 / 间距 8；非末项与分隔符用 colorTextDescription。
#[test]
fn breadcrumb_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let style = breadcrumb_state(&tokens);
    assert_eq!(style.link, tokens.color_text_description);
    assert_eq!(style.current, tokens.color_text);
    assert_eq!(style.separator, tokens.color_text_description);
    assert_eq!(style.font_size, tokens.font_size);
    assert_eq!(style.line_height, tokens.font_height);
    assert_eq!(style.gap, 8.0);
}
