//! Statistic 色板：标题 / 内容 / 趋势色 + 度量。
//!
//! 基准由 `harness/antd_baseline/gen_statistic_matrix.mjs` 依据 antdv-next
//! `statistic/style/index.js` 生成。

use gpui::Hsla;
use gpui_widgets::statistic::statistic_style;
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive, metric};

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
        } else if hex.len() == 6 {
            hex.to_string()
        } else {
            panic!("坏颜色 {text:?}")
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

fn assert_close(actual: Hsla, expected: Rgba, what: &str, theme: &str) {
    let a = rgba(actual);
    let near = (a.r - expected.r).abs() <= 1.0
        && (a.g - expected.g).abs() <= 1.0
        && (a.b - expected.b).abs() <= 1.0
        && (a.a - expected.a).abs() <= 0.01;
    assert!(
        near,
        "{theme} {what}: got rgba({},{},{},{}), want rgba({},{},{},{})",
        a.r, a.g, a.b, a.a, expected.r, expected.g, expected.b, expected.a
    );
}

fn assert_f32(actual: f32, expected: &str, what: &str, theme: &str) {
    let expected = expected.parse::<f32>().unwrap();
    assert!(
        (actual - expected).abs() < 1e-4,
        "{theme} {what}: got {actual}, want {expected}"
    );
}

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}statistic_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 10, "坏行: {line}");
    let style = statistic_style(tokens);
    let m = metric::StatisticMetric::resolve(tokens);
    assert_close(style.title, parse(p[0]), "标题色", suffix);
    assert_close(style.content, parse(p[1]), "内容色", suffix);
    assert_close(style.trend_up, parse(p[2]), "上升色", suffix);
    assert_close(style.trend_down, parse(p[3]), "下降色", suffix);
    assert_f32(m.title_size, p[4], "标题字号", suffix);
    assert_f32(m.content_size, p[5], "内容字号", suffix);
    assert_f32(m.header_gap, p[6], "标题下距", suffix);
    assert_f32(m.affix_gap, p[7], "前后缀间距", suffix);
    assert_f32(m.trend_size, p[8], "趋势字号", suffix);
    assert_f32(m.trend_gap, p[9], "趋势间距", suffix);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn statistic_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn statistic_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn statistic_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 标题 14 / 内容 24 / 间距 4；趋势 12。
#[test]
fn statistic_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = metric::StatisticMetric::resolve(&tokens);
    assert_eq!(m.title_size, 14.0);
    assert_eq!(m.content_size, 24.0);
    assert_eq!(m.header_gap, 4.0);
    assert_eq!(m.affix_gap, 4.0);
    assert_eq!(m.trend_size, 12.0);
    assert_eq!(m.trend_gap, 4.0);
}
