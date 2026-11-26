//! Radio 矩阵：disabled × selected。
//!
//! 基准由 `harness/antd_baseline/gen_radio_matrix.mjs` 依据 antdv-next Radio
//! 样式生成，关键值已用 `harness/_radio_probe.html` 的 computed style 核对。

use gpui::Hsla;
use gpui_widgets::radio::radio_state;
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
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}radio_matrix_{suffix}.txt")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 14, "坏行: {line}");
        let style = radio_state(tokens, p[1] == "1", p[0] == "1");
        assert_close(style.bg, parse(p[2]), "底色", line);
        assert_close(style.border, parse(p[3]), "描边色", line);
        assert_close(style.hover_border, parse(p[4]), "悬停描边色", line);
        assert_close(style.dot, parse(p[5]), "内点色", line);
        assert_close(style.label, parse(p[6]), "标签色", line);
        assert_f32(style.size, p[7], "圆尺寸", line);
        assert_f32(style.dot_size, p[8], "内点尺寸", line);
        assert_f32(style.radius, p[9], "圆角", line);
        assert_f32(style.line_width, p[10], "描边宽", line);
        assert_f32(style.label_gap, p[11], "标签间距", line);
        assert_f32(style.font_size, p[12], "标签字号", line);
        assert_f32(style.line_height, p[13], "标签行高", line);
        checked += 1;
    }
    assert_eq!(checked, 4, "radio_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn radio_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn radio_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn radio_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 圆形 16、内点 6、半径取一半；其余颜色/间距/字号来自令牌。
#[test]
fn radio_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let off = radio_state(&tokens, false, false);
    assert_eq!(off.size, 16.0);
    assert_eq!(off.dot_size, 6.0);
    assert_eq!(off.radius, 8.0);
    assert_eq!(off.bg, tokens.color_bg_container);
    assert_eq!(off.border, tokens.color_border);
    assert_eq!(off.hover_border, tokens.color_primary);
    assert_eq!(off.dot, gpui::transparent_black());
    assert_eq!(off.label_gap, tokens.padding_xs);

    let on = radio_state(&tokens, true, false);
    assert_eq!(on.bg, tokens.color_primary);
    assert_eq!(on.border, tokens.color_primary);
    assert_eq!(on.dot, tokens.color_text_light_solid);
    assert_eq!(on.hover_border, tokens.color_primary_hover);

    let disabled = radio_state(&tokens, true, true);
    assert_eq!(disabled.bg, tokens.color_bg_container_disabled);
    assert_eq!(disabled.border, tokens.color_border);
    assert_eq!(disabled.dot, tokens.color_text_disabled);
    assert_eq!(disabled.label, tokens.color_text_disabled);
    assert_eq!(disabled.hover_border, tokens.color_border);
}
