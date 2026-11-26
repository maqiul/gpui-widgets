//! Modal 矩阵：遮罩/卡片/标题/关闭按钮取色与几何。
//!
//! 基准由 `harness/antd_baseline/gen_modal_matrix.mjs` 依据 antdv-next
//! `modal/style/index.js` 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{ModalMetric, modal_style},
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
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}modal_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 23, "坏行: {line}");

    let style = modal_style(tokens);
    let m = ModalMetric::resolve(tokens);
    assert_close(style.mask, parse(p[0]), "遮罩色", line);
    assert_close(style.content_bg, parse(p[1]), "卡片底", line);
    assert_close(style.title, parse(p[2]), "标题色", line);
    assert_close(style.close, parse(p[3]), "关闭色", line);
    assert_close(style.close_hover, parse(p[4]), "关闭悬停色", line);
    assert_close(style.close_hover_bg, parse(p[5]), "关闭悬停底", line);
    assert_close(style.close_active_bg, parse(p[6]), "关闭按下底", line);
    assert_f32(m.width, p[7], "默认宽", line);
    assert_f32(m.content_padding_block, p[8], "卡片纵向内边距", line);
    assert_f32(m.content_padding_inline, p[9], "卡片横向内边距", line);
    assert_f32(m.radius, p[10], "圆角", line);
    assert_f32(m.header_margin_bottom, p[11], "头部下边距", line);
    assert_f32(m.title_size, p[12], "标题字号", line);
    assert_f32(m.title_line_height, p[13], "标题行高", line);
    assert_f32(m.header_height, p[14], "头部高", line);
    assert_f32(m.close_size, p[15], "关闭按钮尺寸", line);
    assert_f32(m.close_offset, p[16], "关闭按钮偏移", line);
    assert_f32(m.close_radius, p[17], "关闭按钮圆角", line);
    assert_f32(m.close_icon_size, p[18], "关闭图标字号", line);
    assert_f32(m.footer_margin_top, p[19], "底部上边距", line);
    assert_f32(m.footer_gap, p[20], "底部按钮间距", line);
    assert_f32(m.body_size, p[21], "正文字号", line);
    assert_f32(m.body_line_height, p[22], "正文行高", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn modal_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn modal_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn modal_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn modal_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = ModalMetric::resolve(&tokens);
    assert_eq!(m.width, 520.0);
    assert_eq!(m.content_padding_block, 20.0);
    assert_eq!(m.content_padding_inline, 24.0);
    assert_eq!(m.radius, 8.0);
    assert_eq!(m.header_margin_bottom, 8.0);
    assert_eq!(m.title_size, 16.0);
    assert_eq!(m.title_line_height, 24.0);
    assert_eq!(m.header_height, 56.0);
    assert_eq!(m.close_size, 32.0);
    assert_eq!(m.close_offset, 12.0);
    assert_eq!(m.close_radius, 4.0);
    assert_eq!(m.close_icon_size, 16.0);
    assert_eq!(m.footer_margin_top, 12.0);
    assert_eq!(m.footer_gap, 8.0);
    assert_eq!(m.body_size, 14.0);
    assert_eq!(m.body_line_height, 22.0);
}
