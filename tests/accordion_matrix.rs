//! Accordion(Collapse) 矩阵：头/内容底、边框、头字三态、三档度量。
//!
//! 基准由 `harness/antd_baseline/gen_layout_matrix.mjs` 依据 antdv-next
//! `collapse/style/index.js` prepareComponentToken + genBaseStyle 生成。
use gpui::Hsla;
use gpui_widgets::text_input::InputSize;
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
    let c = Color::from(color);
    Rgba {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
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

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

fn baseline(name: &str, suffix: &str) -> Vec<String> {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}{name}_matrix_{suffix}.txt")).unwrap();
    text.lines()
        .next()
        .unwrap()
        .split('|')
        .map(str::to_string)
        .collect()
}

use gpui_widgets::theme::metric::{
    collapse_border, collapse_content_bg, collapse_header_bg, collapse_header_fg, collapse_metric,
};

fn check(tokens: &Tokens, suffix: &str) {
    let p = baseline("collapse", suffix);
    assert_eq!(p.len(), 24, "坏行: {p:?}");
    let row = format!("{p:?}");
    assert_close(collapse_header_bg(tokens), parse(&p[0]), "头底", &row);
    assert_close(collapse_content_bg(tokens), parse(&p[1]), "内容底", &row);
    assert_close(collapse_border(tokens), parse(&p[2]), "边框", &row);
    assert_close(
        collapse_header_fg(tokens, false),
        parse(&p[3]),
        "头字",
        &row,
    );
    assert_close(
        collapse_header_fg(tokens, true),
        parse(&p[4]),
        "禁用头字",
        &row,
    );
    let m = collapse_metric(tokens, InputSize::Middle);
    assert_f32(m.radius, &p[5], "圆角", &row);
    assert_f32(m.header_pad_v, &p[6], "头纵垫", &row);
    assert_f32(m.header_pad_h, &p[7], "头横垫", &row);
    assert_f32(m.header_pad_left, &p[8], "头左垫", &row);
    assert_f32(m.body_pad_v, &p[9], "体垫", &row);
    assert_f32(m.icon_gap, &p[10], "箭头间距", &row);
    assert_f32(m.icon_font, &p[11], "箭头字号", &row);
    assert_f32(m.font, &p[12], "头字号", &row);
    assert_f32(m.font_height, &p[13], "头行高", &row);
    let s = collapse_metric(tokens, InputSize::Small);
    assert_f32(s.header_pad_v, &p[14], "small头纵垫", &row);
    assert_f32(s.header_pad_h, &p[15], "small头横垫", &row);
    assert_f32(s.header_pad_left, &p[16], "small头左垫", &row);
    assert_f32(s.body_pad_v, &p[17], "small体垫", &row);
    let l = collapse_metric(tokens, InputSize::Large);
    assert_f32(l.header_pad_v, &p[18], "large头纵垫", &row);
    assert_f32(l.header_pad_h, &p[19], "large头横垫", &row);
    assert_f32(l.header_pad_left, &p[20], "large头左垫", &row);
    assert_f32(l.body_pad_v, &p[21], "large体垫", &row);
    assert_f32(l.font, &p[22], "large头字号", &row);
    assert_f32(l.font_height, &p[23], "large头行高", &row);
}

#[test]
fn accordion_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}
#[test]
fn accordion_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}
#[test]
fn accordion_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}
