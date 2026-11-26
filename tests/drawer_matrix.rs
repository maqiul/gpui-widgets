//! Drawer 矩阵：遮罩/面板/分隔线/关闭按钮取色与几何。
//!
//! 基准由 `harness/antd_baseline/gen_drawer_matrix.mjs` 依据 antdv-next
//! `drawer/style/index.js` 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{DrawerMetric, drawer_style},
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
        std::fs::read_to_string(format!("{BASELINE_DIR}drawer_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 21, "坏行: {line}");

    let style = drawer_style(tokens);
    let m = DrawerMetric::resolve(tokens);
    assert_close(style.mask, parse(p[0]), "遮罩色", line);
    assert_close(style.bg, parse(p[1]), "面板底", line);
    assert_close(style.border, parse(p[2]), "分隔线色", line);
    assert_close(style.text, parse(p[3]), "标题色", line);
    assert_close(style.close, parse(p[4]), "关闭色", line);
    assert_close(style.close_hover, parse(p[5]), "关闭悬停色", line);
    assert_close(style.close_hover_bg, parse(p[6]), "关闭悬停底", line);
    assert_close(style.close_active_bg, parse(p[7]), "关闭按下底", line);
    assert_f32(m.width, p[8], "默认宽", line);
    assert_f32(m.header_padding_block, p[9], "头部纵向内边距", line);
    assert_f32(m.header_padding_inline, p[10], "头部横向内边距", line);
    assert_f32(m.header_font_size, p[11], "头部字号", line);
    assert_f32(m.header_line_height, p[12], "头部行高", line);
    assert_f32(m.border_width, p[13], "分隔线宽", line);
    assert_f32(m.close_size, p[14], "关闭按钮尺寸", line);
    assert_f32(m.close_gap, p[15], "关闭与标题间距", line);
    assert_f32(m.close_radius, p[16], "关闭按钮圆角", line);
    assert_f32(m.body_padding, p[17], "正文内边距", line);
    assert_f32(m.footer_padding_block, p[18], "底部纵向内边距", line);
    assert_f32(m.footer_padding_inline, p[19], "底部横向内边距", line);
    assert_f32(m.footer_gap, p[20], "底部按钮间距", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn drawer_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn drawer_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn drawer_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn drawer_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = DrawerMetric::resolve(&tokens);
    assert_eq!(m.width, 378.0);
    assert_eq!(m.header_padding_block, 16.0);
    assert_eq!(m.header_padding_inline, 24.0);
    assert_eq!(m.header_font_size, 16.0);
    assert_eq!(m.header_line_height, 24.0);
    assert_eq!(m.border_width, 1.0);
    assert_eq!(m.close_size, 24.0);
    assert_eq!(m.close_gap, 8.0);
    assert_eq!(m.close_radius, 4.0);
    assert_eq!(m.body_padding, 24.0);
    assert_eq!(m.footer_padding_block, 8.0);
    assert_eq!(m.footer_padding_inline, 16.0);
    assert_eq!(m.footer_gap, 8.0);
}
