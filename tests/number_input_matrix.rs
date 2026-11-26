//! NumberInput 矩阵：步进按钮三态 + 列底 + 度量。
//!
//! 基准由 `harness/antd_baseline/gen_input_compound_matrix.mjs` 依据 antdv-next
//! `input-number/style/token.js` + `input-number/style/index.js` 生成。
use gpui::Hsla;
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

use gpui_widgets::theme::metric::{input_number_metric, number_actions_bg, number_handle_state};

fn check(tokens: &Tokens, suffix: &str) {
    let p = baseline("number_input", suffix);
    assert_eq!(p.len(), 10, "坏行: {p:?}");
    let idle = number_handle_state(tokens, false, false, false);
    assert_close(idle.fg, parse(&p[0]), "idle箭头色", &format!("{p:?}"));
    assert!(idle.bg.is_none());
    let hover = number_handle_state(tokens, true, false, false);
    assert_close(hover.fg, parse(&p[1]), "hover箭头色", &format!("{p:?}"));
    let pressed = number_handle_state(tokens, true, true, false);
    assert_close(
        pressed.bg.unwrap(),
        parse(&p[3]),
        "按下底色",
        &format!("{p:?}"),
    );
    let disabled = number_handle_state(tokens, true, true, true);
    assert_close(disabled.fg, parse(&p[2]), "禁用箭头色", &format!("{p:?}"));
    assert!(disabled.bg.is_none());
    assert_close(
        number_actions_bg(tokens, false, false),
        parse(&p[4]),
        "outlined列底",
        &format!("{p:?}"),
    );
    assert_close(
        number_actions_bg(tokens, true, false),
        parse(&p[6]),
        "filled列底",
        &format!("{p:?}"),
    );
    assert_close(
        number_actions_bg(tokens, true, true),
        parse(&p[4]),
        "filled聚焦列底",
        &format!("{p:?}"),
    );
    let m = input_number_metric(tokens);
    let row = format!("{p:?}");
    assert_f32(m.handle_width, &p[7], "步进列宽", &row);
    assert_f32(m.handle_font, &p[8], "箭头字号", &row);
    assert_f32(m.control_width, &p[9], "默认控件宽", &row);
}

#[test]
fn number_input_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn number_input_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn number_input_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn number_input_handle_state_matrix() {
    for appearance in [Appearance::Light, Appearance::Dark] {
        let tokens = tokens_for(None, appearance);
        let suffix = if appearance == Appearance::Light {
            "light"
        } else {
            "dark"
        };
        let p = baseline("number_input", suffix);
        let released = number_handle_state(&tokens, false, true, false);
        assert_close(released.fg, parse(&p[0]), "未hover按下回落idle", suffix);
        assert!(released.bg.is_none());
        let disabled_hover = number_handle_state(&tokens, true, false, true);
        assert_close(disabled_hover.fg, parse(&p[2]), "禁用不变hover色", suffix);
    }
}

#[test]
fn number_input_metric_follows_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = input_number_metric(&tokens);
    assert_f32(m.control_width, "90", "默认宽", "follow");
    assert_f32(m.handle_width, "22", "列宽", "follow");
    assert_f32(m.handle_font, "7", "箭头字号", "follow");
    assert_close(
        number_actions_bg(&tokens, true, false),
        parse("#f0f0f0"),
        "filled列底",
        "follow",
    );
}
