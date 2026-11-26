//! Link 矩阵：常态 / 悬停 / 按下 / 禁用。
//!
//! 基准由 `harness/antd_baseline/gen_link_matrix.mjs` 依据 antdv-next
//! `typography/style/mixins.js` 的 `operationUnit` 生成。

use gpui::Hsla;
use gpui_widgets::link::{LinkState, link_state};
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
        } else if hex.len() == 6 {
            hex.to_string()
        } else {
            panic!("坏颜色 {text:?}")
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

fn state(text: &str) -> LinkState {
    match text {
        "normal" => LinkState::Normal,
        "hover" => LinkState::Hover,
        "active" => LinkState::Active,
        "disabled" => LinkState::Disabled,
        other => panic!("坏状态 {other}"),
    }
}

fn check(tokens: &Tokens, suffix: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}link_matrix_{suffix}.txt")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 3, "坏行: {line}");
        let style = link_state(tokens, state(p[0]));
        let actual = rgba(style.text);
        let expected = parse(p[1]);
        let near = (actual.r - expected.r).abs() <= 1.0
            && (actual.g - expected.g).abs() <= 1.0
            && (actual.b - expected.b).abs() <= 1.0
            && (actual.a - expected.a).abs() <= 0.01;
        assert!(
            near,
            "{line} 文字色: got rgba({},{},{},{}), want rgba({},{},{},{})",
            actual.r, actual.g, actual.b, actual.a, expected.r, expected.g, expected.b, expected.a
        );
        assert_eq!(style.underline, p[2] == "1", "{line} 下划线");
        checked += 1;
    }
    assert_eq!(checked, 4, "link_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn link_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn link_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn link_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}
