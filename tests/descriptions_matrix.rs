//! Descriptions 配色与度量：标签、标题、分隔线、勾选间距与冒号边距。
//!
//! 基准由 `harness/antd_baseline/gen_descriptions_matrix.mjs` 依据 antdv-next
//! `descriptions/style/index.js` 生成。

use gpui::Hsla;
use gpui_widgets::descriptions::{DescriptionsStyle, descriptions_style};
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{self, DescriptionsMetric},
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

/// 两侧都叠加到同一个面板底色上再比，避免 alpha 表达差异（如 colorSplit 已预混合）。
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
        std::fs::read_to_string(format!("{BASELINE_DIR}descriptions_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 19, "坏行: {line}");
    let style: DescriptionsStyle = descriptions_style(tokens);
    let m = DescriptionsMetric::resolve(tokens);
    let bg = rgba(tokens.color_bg_container);

    assert_close(style.label_bg, parse(p[0]), bg, "标签底", suffix);
    assert_close(style.label_color, parse(p[1]), bg, "标签色", suffix);
    assert_close(
        style.label_color_bordered,
        parse(p[2]),
        bg,
        "带边框标签色",
        suffix,
    );
    assert_close(style.title_color, parse(p[3]), bg, "标题色", suffix);
    assert_close(style.content_color, parse(p[4]), bg, "内容色", suffix);
    assert_close(style.border_color, parse(p[5]), bg, "分隔线", suffix);

    assert_f32(m.title_size, p[6].parse().unwrap(), "标题字号", suffix);
    assert_f32(
        m.title_line_height,
        p[7].parse().unwrap(),
        "标题行高",
        suffix,
    );
    assert_f32(
        m.title_margin_bottom,
        p[8].parse().unwrap(),
        "标题下距",
        suffix,
    );
    assert_f32(
        m.item_padding_bottom,
        p[9].parse().unwrap(),
        "行下距",
        suffix,
    );
    assert_f32(m.item_padding_end, p[10].parse().unwrap(), "行右距", suffix);
    assert_f32(
        m.bordered_padding_block,
        p[11].parse().unwrap(),
        "带边框纵向内边距",
        suffix,
    );
    assert_f32(
        m.bordered_padding_inline,
        p[12].parse().unwrap(),
        "带边框横向内边距",
        suffix,
    );
    assert_f32(
        m.colon_margin_left,
        p[13].parse().unwrap(),
        "冒号左距",
        suffix,
    );
    assert_f32(
        m.colon_margin_right,
        p[14].parse().unwrap(),
        "冒号右距",
        suffix,
    );
    assert_f32(m.border_radius, p[15].parse().unwrap(), "外框圆角", suffix);
    assert_f32(m.item_size, p[16].parse().unwrap(), "项字号", suffix);
    assert_f32(
        m.line_height,
        (m.item_size * p[17].parse::<f32>().unwrap()).round(),
        "行高",
        suffix,
    );
    assert_eq!(
        metric::DESCRIPTIONS_TITLE_WEIGHT,
        p[18].parse::<f32>().unwrap()
    );
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn descriptions_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn descriptions_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn descriptions_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn descriptions_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = DescriptionsMetric::resolve(&tokens);
    assert_eq!(m.title_size, 16.0);
    assert_eq!(m.title_line_height, 24.0);
    assert_eq!(m.title_margin_bottom, 20.0);
    assert_eq!(m.item_padding_bottom, 16.0);
    assert_eq!(m.bordered_padding_inline, 24.0);
    assert_eq!(m.colon_margin_left, 2.0);
    assert_eq!(m.colon_margin_right, 8.0);
    assert_eq!(m.border_radius, 8.0);
    assert_eq!(m.line_height, 22.0);
}
