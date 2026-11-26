//! Transfer 矩阵：双列表取色与几何。
//!
//! 基准由 `harness/antd_baseline/gen_transfer_matrix.mjs` 依据 antdv-next
//! `transfer/style/index.js` 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{TransferMetric, transfer_style},
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
        std::fs::read_to_string(format!("{BASELINE_DIR}transfer_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 34, "坏行: {line}");

    let style = transfer_style(tokens);
    let m = TransferMetric::resolve(tokens);
    assert_close(style.border, parse(p[0]), "外边框", line);
    assert_close(style.split, parse(p[1]), "分隔线", line);
    assert_close(style.header_bg, parse(p[2]), "头部底色", line);
    assert_close(style.header_text, parse(p[3]), "头部文字", line);
    assert_close(style.item_checked, parse(p[4]), "选中行底色", line);
    assert_close(style.item_hover, parse(p[5]), "悬停底色", line);
    assert_close(style.item_checked_hover, parse(p[6]), "选中悬停底色", line);
    assert_close(style.disabled, parse(p[7]), "禁用文字", line);
    assert_close(style.disabled_bg, parse(p[8]), "禁用底色", line);
    assert_close(style.checkbox_bg, parse(p[9]), "勾选底色", line);
    assert_close(style.check_fg, parse(p[10]), "勾色", line);
    assert_close(style.button_hover, parse(p[11]), "按钮悬停", line);
    assert_f32(m.list_width, p[12], "列表宽", line);
    assert_f32(m.list_height, p[13], "列表高", line);
    assert_f32(m.header_height, p[14], "头部高", line);
    assert_f32(m.header_padding_top, p[15], "头部上内边距", line);
    assert_f32(m.header_padding_bottom, p[16], "头部下内边距", line);
    assert_f32(m.header_padding_inline, p[17], "头部横向内边距", line);
    assert_f32(m.header_gap, p[18], "头部间距", line);
    assert_f32(m.item_height, p[19], "条目高", line);
    assert_f32(m.item_padding_block, p[20], "条目纵向内边距", line);
    assert_f32(m.item_padding_inline, p[21], "条目横向内边距", line);
    assert_f32(m.item_gap, p[22], "条目间距", line);
    assert_f32(m.radius, p[23], "圆角", line);
    assert_f32(m.content_radius, p[24], "内容圆角", line);
    assert_f32(m.checkbox_size, p[25], "勾选框尺寸", line);
    assert_f32(m.checkbox_radius, p[26], "勾选框圆角", line);
    assert_f32(m.actions_margin, p[27], "按钮列外边距", line);
    assert_f32(m.actions_gap, p[28], "按钮间距", line);
    assert_f32(m.button_size, p[29], "按钮尺寸", line);
    assert_f32(m.button_radius, p[30], "按钮圆角", line);
    assert_f32(m.icon_size, p[31], "头部图标字号", line);
    assert_f32(m.font_size, p[32], "字号", line);
    assert_f32(m.line_height, p[33], "行高", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn transfer_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn transfer_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn transfer_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn transfer_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = TransferMetric::resolve(&tokens);
    assert_eq!(m.list_width, 180.0);
    assert_eq!(m.list_height, 200.0);
    assert_eq!(m.header_height, 40.0);
    assert_eq!(m.header_padding_top, 8.0);
    assert_eq!(m.header_padding_bottom, 9.0);
    assert_eq!(m.header_padding_inline, 12.0);
    assert_eq!(m.item_height, 32.0);
    assert_eq!(m.item_padding_block, 5.0);
    assert_eq!(m.item_padding_inline, 12.0);
    assert_eq!(m.item_gap, 8.0);
    assert_eq!(m.radius, 8.0);
    assert_eq!(m.content_radius, 7.0);
    assert_eq!(m.checkbox_size, 16.0);
    assert_eq!(m.checkbox_radius, 4.0);
    assert_eq!(m.actions_margin, 8.0);
    assert_eq!(m.actions_gap, 4.0);
    assert_eq!(m.button_size, 32.0);
    assert_eq!(m.button_radius, 6.0);
    assert_eq!(m.icon_size, 12.0);
}
