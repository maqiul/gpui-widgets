//! Popconfirm 矩阵：图标/标题/描述配色 + message/按钮几何 + Popover 外壳。
//!
//! 基准由 `harness/antd_baseline/gen_popconfirm_matrix.mjs` 依据 antdv-next
//! `popconfirm/style/index.js` + `popover/style/index.js` 生成。

use gpui::Hsla;
use gpui_widgets::popover::popover_state;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{popconfirm_metric, popconfirm_style},
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
        std::fs::read_to_string(format!("{BASELINE_DIR}popconfirm_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 11, "坏行: {line}");

    let cs = popconfirm_style(tokens);
    let m = popconfirm_metric(tokens);
    let shell = popover_state(tokens);
    assert_close(cs.icon, parse(p[0]), "警告图标", line);
    assert_close(cs.title_fg, parse(p[1]), "标题色", line);
    assert_close(cs.desc_fg, parse(p[2]), "描述色", line);
    assert_close(shell.bg, parse(p[3]), "外壳底", line);
    assert_close(shell.fg, parse(p[2]), "外壳字", line);
    assert_f32(m.message_margin_bottom, p[4], "message底距", line);
    assert_f32(m.icon_gap, p[4], "图标右距", line);
    assert_f32(m.button_gap, p[4], "按钮间距", line);
    assert_f32(m.icon_size, p[5], "图标尺寸", line);
    assert_f32(shell.font_size, p[5], "外壳字号", line);
    assert_f32(m.desc_margin_top, p[6], "描述顶距", line);
    assert_f32(m.title_weight, p[7], "标题字重", line);
    assert_f32(shell.padding, p[8], "外壳内边距", line);
    assert_f32(shell.radius, p[9], "外壳圆角", line);
    assert_f32(shell.line_height, p[10], "外壳行高", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn popconfirm_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn popconfirm_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn popconfirm_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn popconfirm_metric_follows_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = popconfirm_metric(&tokens);
    let shell = popover_state(&tokens);
    assert_f32(m.message_margin_bottom, "8", "message底距", "follow");
    assert_f32(m.icon_size, "14", "图标尺寸", "follow");
    assert_f32(m.desc_margin_top, "4", "描述顶距", "follow");
    assert_f32(m.title_weight, "600", "标题字重", "follow");
    assert_f32(shell.padding, "12", "外壳内边距", "follow");
    assert_f32(shell.radius, "8", "外壳圆角", "follow");
    assert_f32(shell.line_height, "22", "外壳行高", "follow");
}
