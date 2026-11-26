//! Carousel 矩阵：箭头/指示条取色与几何。
//!
//! 基准由 `harness/antd_baseline/gen_carousel_matrix.mjs` 依据 antdv-next
//! `carousel/style/index.js` 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{CarouselMetric, carousel_style},
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
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}carousel_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 12, "坏行: {line}");

    let style = carousel_style(tokens);
    let m = CarouselMetric::resolve(tokens);
    assert_close(style.arrow, parse(p[0]), "箭头色", line);
    assert_close(style.dot, parse(p[1]), "指示条色", line);
    assert_f32(m.arrow_size, p[2], "箭头尺寸", line);
    assert_f32(m.arrow_offset, p[3], "箭头边距", line);
    assert_f32(m.arrow_opacity, p[4], "箭头不透明度", line);
    assert_f32(m.dot_width, p[5], "指示条宽", line);
    assert_f32(m.dot_height, p[6], "指示条高", line);
    assert_f32(m.dot_gap, p[7], "指示条间距", line);
    assert_f32(m.dot_offset, p[8], "指示条底距", line);
    assert_f32(m.dot_active_width, p[9], "选中指示条宽", line);
    assert_f32(m.dot_opacity, p[10], "指示条不透明度", line);
    assert_f32(m.dot_hover_opacity, p[11], "指示条悬停不透明度", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn carousel_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn carousel_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn carousel_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn carousel_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = CarouselMetric::resolve(&tokens);
    assert_eq!(m.arrow_size, 16.0);
    assert_eq!(m.arrow_offset, 8.0);
    assert_eq!(m.arrow_opacity, 0.4);
    assert_eq!(m.dot_width, 16.0);
    assert_eq!(m.dot_height, 3.0);
    assert_eq!(m.dot_gap, 4.0);
    assert_eq!(m.dot_offset, 12.0);
    assert_eq!(m.dot_active_width, 24.0);
    assert_eq!(m.dot_opacity, 0.2);
    assert_eq!(m.dot_hover_opacity, 0.75);
}
