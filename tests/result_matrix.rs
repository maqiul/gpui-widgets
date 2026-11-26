//! Result 矩阵：状态色、文字层级、图标与正文块几何。
//!
//! 基准由 `harness/antd_baseline/gen_result_matrix.mjs` 依据 antdv-next
//! `result/style/index.js` 的组件 token 生成。

use gpui::Hsla;
use gpui_widgets::result::result_style;
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive, metric::ResultMetric};

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
        std::fs::read_to_string(format!("{BASELINE_DIR}result_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 17, "坏行: {line}");

    let style = result_style(tokens);
    let m = ResultMetric::resolve(tokens);
    assert_close(style.success, parse(p[0]), "成功色", line);
    assert_close(style.error, parse(p[1]), "错误色", line);
    assert_close(style.warning, parse(p[2]), "警告色", line);
    assert_close(style.info, parse(p[3]), "信息色", line);
    assert_close(style.title, parse(p[4]), "标题色", line);
    assert_close(style.subtitle, parse(p[5]), "副标题色", line);
    assert_close(style.body_bg, parse(p[6]), "正文块底色", line);
    assert_f32(m.icon_size, p[7], "图标尺寸", line);
    assert_f32(m.title_size, p[8], "标题字号", line);
    assert_f32(m.title_line_height, p[9], "标题行高", line);
    assert_f32(m.title_margin, p[10], "标题边距", line);
    assert_f32(m.subtitle_size, p[11], "副标题字号", line);
    assert_f32(m.subtitle_line_height, p[12], "副标题行高", line);
    assert_f32(m.icon_margin_bottom, p[13], "图标下距", line);
    assert_f32(m.body_padding_inline, p[14], "正文块横向内边距", line);
    assert_f32(m.extra_margin, p[15], "操作区间距", line);
    assert_f32(m.root_padding_inline, p[16], "根容器横向内边距", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn result_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn result_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn result_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn result_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = ResultMetric::resolve(&tokens);
    assert_eq!(m.icon_size, 72.0);
    assert_eq!(m.title_size, 24.0);
    assert_eq!(m.title_line_height, 32.0);
    assert_eq!(m.subtitle_size, 14.0);
    assert_eq!(m.subtitle_line_height, 22.0);
    assert_eq!(m.icon_margin_bottom, 24.0);
    assert_eq!(m.root_padding_block, 48.0);
    assert_eq!(m.body_padding_inline, 40.0);
}
