//! Toast（antd Message）矩阵：卡片几何 + 共享 notice 配色。
//!
//! 基准由 `harness/antd_baseline/gen_notice_matrix.mjs` 依据 antdv-next
//! `message/style/index.js` prepareMessageToken / genMessageItemStyle 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{message_metric, notice_style},
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
        std::fs::read_to_string(format!("{BASELINE_DIR}message_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 10, "坏行: {line}");

    let m = message_metric(tokens);
    let cs = notice_style(tokens);
    assert_f32(m.padding_vertical, p[0], "垂直内边距", line);
    assert_f32(m.padding_horizontal, p[1], "水平内边距", line);
    assert_f32(m.radius, p[2], "圆角", line);
    assert_f32(m.font, p[3], "正文字号", line);
    assert_f32(m.line, p[4], "正文行高", line);
    assert_f32(m.icon_size, p[5], "图标尺寸", line);
    assert_f32(m.icon_gap, p[6], "图标右距", line);
    assert_f32(m.stack_gap, p[7], "堆栈间距", line);
    assert_f32(m.edge, p[8], "堆栈边距", line);
    assert_close(cs.bg, parse(p[9]), "卡片底", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn toast_message_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn toast_message_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn toast_message_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn toast_message_metric_follows_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = message_metric(&tokens);
    assert_f32(m.padding_vertical, "9", "垂直内边距", "follow");
    assert_f32(m.padding_horizontal, "12", "水平内边距", "follow");
    assert_f32(m.radius, "8", "圆角", "follow");
    assert_f32(m.font, "14", "正文字号", "follow");
    assert_f32(m.line, "22", "正文行高", "follow");
    assert_f32(m.icon_size, "16", "图标尺寸", "follow");
    assert_f32(m.icon_gap, "8", "图标右距", "follow");
    assert_f32(m.stack_gap, "16", "堆栈间距", "follow");
    assert_f32(m.edge, "24", "堆栈边距", "follow");
}
