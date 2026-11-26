//! Empty 矩阵：插画实色（叠底合成）与三档插画高度 / 边距 / 描述 / footer 几何。
//!
//! 基准由 `harness/antd_baseline/gen_empty_matrix.mjs` 依据 antdv-next
//! `empty/style/index.js` + `empty/empty.js` / `simple.js` / `utils.js` 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{empty_metric, empty_style},
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
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}empty_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 20, "坏行: {line}");

    let style = empty_style(tokens);
    let m = empty_metric(tokens);
    assert_close(style.panel_bg, parse(p[0]), "插画面板底", line);
    assert_close(style.border, parse(p[1]), "插画描边", line);
    assert_close(style.detail, parse(p[2]), "插画细节", line);
    assert_close(style.shadow, parse(p[3]), "插画投影", line);
    assert_close(style.icon, parse(p[4]), "插画图标", line);
    assert_close(style.simple_border, parse(p[5]), "simple 描边", line);
    assert_close(style.simple_shadow, parse(p[6]), "simple 投影", line);
    assert_close(style.simple_content, parse(p[7]), "simple 内容", line);
    assert_close(style.desc, parse(p[8]), "描述色", line);
    assert_f32(m.img_height, p[9], "插画高", line);
    assert_f32(m.img_height_md, p[10], "simple 插画高", line);
    assert_f32(m.img_height_sm, p[11], "small 插画高", line);
    assert_f32(m.img_margin_bottom, p[12], "插画下边距", line);
    assert_f32(m.margin_inline, p[13], "横向外边距", line);
    assert_f32(m.margin_block_normal, p[14], "normal 纵向外边距", line);
    assert_f32(m.margin_block_small, p[15], "small 纵向外边距", line);
    assert_f32(m.shadow_opacity, p[16], "投影透明度", line);
    assert_f32(m.desc_size, p[17], "描述字号", line);
    assert_f32(m.desc_line_height, p[18], "描述行高", line);
    assert_f32(m.footer_margin_top, p[19], "footer 上边距", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn empty_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn empty_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn empty_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn empty_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = empty_metric(&tokens);
    assert_eq!(m.img_height, 100.0);
    assert_eq!(m.img_height_md, 40.0);
    assert_eq!(m.img_height_sm, 35.0);
    assert_eq!(m.img_margin_bottom, 8.0);
    assert_eq!(m.margin_inline, 8.0);
    assert_eq!(m.margin_block_normal, 32.0);
    assert_eq!(m.margin_block_small, 8.0);
    assert_eq!(m.desc_size, 14.0);
    assert_eq!(m.desc_line_height, 22.0);
    assert_eq!(m.footer_margin_top, 16.0);
}
