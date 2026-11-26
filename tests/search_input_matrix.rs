//! SearchInput 矩阵：清空图标三态 + 前后缀度量。
//!
//! 基准由 `harness/antd_baseline/gen_input_compound_matrix.mjs` 依据 antdv-next
//! `input/style/index.js` genAllowClearStyle / genAffixStyle 生成。
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

use gpui_widgets::theme::metric::{clear_icon_color, input_affix_metric};

fn check(tokens: &Tokens, suffix: &str) {
    let p = baseline("search_input", suffix);
    assert_eq!(p.len(), 6, "坏行: {p:?}");
    let row = format!("{p:?}");
    assert_close(
        clear_icon_color(tokens, false, false),
        parse(&p[0]),
        "idle清空色",
        &row,
    );
    assert_close(
        clear_icon_color(tokens, true, false),
        parse(&p[1]),
        "hover清空色",
        &row,
    );
    assert_close(
        clear_icon_color(tokens, true, true),
        parse(&p[2]),
        "按下清空色",
        &row,
    );
    let m = input_affix_metric(tokens);
    assert_f32(m.clear_font, &p[3], "清空尺寸", &row);
    assert_f32(m.icon_font, &p[4], "前缀尺寸", &row);
    assert_f32(m.gap, &p[5], "前后缀间距", &row);
}

#[test]
fn search_input_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn search_input_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn search_input_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn search_input_clear_state_matrix() {
    for appearance in [Appearance::Light, Appearance::Dark] {
        let tokens = tokens_for(None, appearance);
        let suffix = if appearance == Appearance::Light {
            "light"
        } else {
            "dark"
        };
        let p = baseline("search_input", suffix);
        let released = clear_icon_color(&tokens, false, true);
        assert_close(released, parse(&p[0]), "未hover按下回落idle", suffix);
    }
}

#[test]
fn search_input_metric_follows_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = input_affix_metric(&tokens);
    assert_f32(m.icon_font, "14", "前缀尺寸", "follow");
    assert_f32(m.clear_font, "12", "清空尺寸", "follow");
    assert_f32(m.gap, "4", "间距", "follow");
}
