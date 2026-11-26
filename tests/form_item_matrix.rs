//! FormItem 矩阵：标签/星号/提示色 + 三档标签高 + 间距度量。
//!
//! 基准由 `harness/antd_baseline/gen_layout_matrix.mjs` 依据 antdv-next
//! `form/style/index.js` prepareComponentToken + genFormSize 生成。
use gpui::Hsla;
use gpui_widgets::text_input::InputSize;
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
    form_label_color, form_metric, form_msg_color, form_required_color,
};

fn check(tokens: &Tokens, suffix: &str) {
    let p = baseline("form", suffix);
    assert_eq!(p.len(), 16, "坏行: {p:?}");
    let row = format!("{p:?}");
    assert_close(form_label_color(tokens), parse(&p[0]), "标签色", &row);
    assert_close(form_required_color(tokens), parse(&p[1]), "星号色", &row);
    assert_close(
        form_msg_color(tokens, true),
        parse(&p[2]),
        "错误提示色",
        &row,
    );
    assert_close(
        form_msg_color(tokens, false),
        parse(&p[3]),
        "help提示色",
        &row,
    );
    let m = form_metric(tokens, InputSize::Middle);
    assert_f32(m.label_height, &p[4], "标签高", &row);
    assert_f32(m.vertical_label_height, &p[5], "纵向标签高", &row);
    let s = form_metric(tokens, InputSize::Small);
    assert_f32(s.label_height, &p[6], "small标签高", &row);
    assert_f32(s.vertical_label_height, &p[7], "small纵向标签高", &row);
    let l = form_metric(tokens, InputSize::Large);
    assert_f32(l.label_height, &p[8], "large标签高", &row);
    assert_f32(m.required_gap, &p[9], "星号间距", &row);
    assert_f32(m.colon_start, &p[10], "冒号前距", &row);
    assert_f32(m.colon_end, &p[11], "冒号后距", &row);
    assert_f32(m.item_gap, &p[12], "项间距", &row);
    assert_f32(m.label_pad_bottom, &p[13], "纵向标签下垫", &row);
    assert_f32(m.msg_font, &p[14], "提示字号", &row);
    assert_f32(m.extra_min_height, &p[15], "extra最小高", &row);
}

#[test]
fn form_item_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}
#[test]
fn form_item_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}
#[test]
fn form_item_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}
