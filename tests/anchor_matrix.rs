//! Anchor 矩阵：轨/ink/标题三态 + 度量。
//!
//! 基准由 `harness/antd_baseline/gen_layout_matrix.mjs` 依据 antdv-next
//! `anchor/style/index.js` prepareComponentToken + mergeToken 生成。
use gpui::Hsla;
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
    anchor_ink_color, anchor_metric, anchor_rail_color, anchor_title_color,
};

fn check(tokens: &Tokens, suffix: &str) {
    let p = baseline("anchor", suffix);
    assert_eq!(p.len(), 11, "坏行: {p:?}");
    let row = format!("{p:?}");
    let m = anchor_metric(tokens);
    assert_f32(m.rail_width, &p[0], "轨宽", &row);
    assert_close(anchor_rail_color(tokens), parse(&p[1]), "轨色", &row);
    assert_close(anchor_ink_color(tokens), parse(&p[2]), "ink色", &row);
    assert_close(
        anchor_title_color(tokens, false, false),
        parse(&p[3]),
        "标题_idle",
        &row,
    );
    assert_close(
        anchor_title_color(tokens, true, false),
        parse(&p[4]),
        "标题_active",
        &row,
    );
    assert_close(
        anchor_title_color(tokens, false, true),
        parse(&p[5]),
        "标题_hover",
        &row,
    );
    assert_f32(m.link_pad_block, &p[6], "link纵垫", &row);
    assert_f32(m.nested_pad_block, &p[7], "子级纵垫", &row);
    assert_f32(m.link_pad_left, &p[8], "link左垫", &row);
    assert_f32(m.title_gap, &p[9], "标题下间距", &row);
    assert_f32(m.font_height, &p[10], "ink高", &row);
}

#[test]
fn anchor_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}
#[test]
fn anchor_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}
#[test]
fn anchor_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}
