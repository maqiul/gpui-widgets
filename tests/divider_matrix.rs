//! Divider 配色与度量：线条色、文字/plain 样式、边距与竖线尺寸。
//!
//! 基准由 `harness/antd_baseline/gen_divider_matrix.mjs` 依据 antdv-next
//! `divider/style/index.js` 生成。

use gpui::Hsla;
use gpui_widgets::misc::divider_style;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{self, DividerMetric},
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

fn over(front: Rgba, back: Rgba) -> Rgba {
    let a = front.a + back.a * (1.0 - front.a);
    if a <= f32::EPSILON {
        return back;
    }
    Rgba {
        r: (front.r * front.a + back.r * back.a * (1.0 - front.a)) / a,
        g: (front.g * front.a + back.g * back.a * (1.0 - front.a)) / a,
        b: (front.b * front.a + back.b * back.a * (1.0 - front.a)) / a,
        a,
    }
}

fn assert_close(actual: Hsla, expected: Rgba, bg: Rgba, what: &str, theme: &str) {
    let a = over(rgba(actual), bg);
    let e = over(expected, bg);
    let near = (a.r - e.r).abs() <= 1.0 && (a.g - e.g).abs() <= 1.0 && (a.b - e.b).abs() <= 1.0;
    assert!(
        near,
        "{theme} {what}: got rgb({},{},{}), want rgb({},{},{})",
        a.r, a.g, a.b, e.r, e.g, e.b
    );
}

fn assert_f32(actual: f32, expected: f32, what: &str, theme: &str) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "{theme} {what}: got {actual}, want {expected}"
    );
}

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}divider_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 14, "坏行: {line}");
    let style = divider_style(tokens);
    let m = DividerMetric::resolve(tokens);
    let bg = rgba(tokens.color_bg_container);
    assert_close(style.line, parse(p[0]), bg, "线条色", suffix);
    assert_close(style.text, parse(p[1]), bg, "标题色", suffix);
    assert_close(style.plain_text, parse(p[2]), bg, "plain 文字色", suffix);
    assert_f32(m.width, p[3].parse().unwrap(), "线宽", suffix);
    assert_f32(m.margin, p[4].parse().unwrap(), "无文字上下距", suffix);
    assert_f32(
        m.margin_with_text,
        p[5].parse().unwrap(),
        "带文字上下距",
        suffix,
    );
    assert_f32(m.text_size, p[6].parse().unwrap(), "文字字号", suffix);
    assert_f32(
        m.plain_text_size,
        p[7].parse().unwrap(),
        "plain 字号",
        suffix,
    );
    assert_f32(
        m.text_padding_inline,
        p[8].parse().unwrap(),
        "文字两侧内边距",
        suffix,
    );
    assert_f32(
        m.plain_text_padding_inline,
        p[9].parse().unwrap(),
        "plain 两侧内边距",
        suffix,
    );
    assert_f32(
        m.orientation_margin,
        p[10].parse().unwrap(),
        "朝向留白",
        suffix,
    );
    assert_f32(
        m.vertical_margin_inline,
        p[11].parse().unwrap(),
        "竖线左右距",
        suffix,
    );
    assert_f32(
        m.vertical_height,
        m.plain_text_size * p[12].parse::<f32>().unwrap(),
        "竖线高",
        suffix,
    );
    assert_eq!(metric::DIVIDER_TEXT_WEIGHT, p[13].parse::<f32>().unwrap());
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn divider_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn divider_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn divider_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn divider_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = DividerMetric::resolve(&tokens);
    assert_eq!(m.width, 1.0);
    assert_eq!(m.margin, 24.0);
    assert_eq!(m.margin_with_text, 16.0);
    assert_eq!(m.text_size, 16.0);
    assert_eq!(m.plain_text_size, 14.0);
    assert_eq!(m.vertical_margin_inline, 8.0);
    assert_eq!(m.vertical_height.round(), 13.0);
}
