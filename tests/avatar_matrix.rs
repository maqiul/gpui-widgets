//! Avatar 矩阵：尺寸 × 形状。
//!
//! 基准由 `harness/antd_baseline/gen_avatar_matrix.mjs` 依据 antdv-next Avatar
//! 样式生成，关键值已用 `harness/_avatar_probe.html` 的 computed style 核对。

use gpui::Hsla;
use gpui_widgets::avatar::{AvatarSize, avatar_state};
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
    if text == "transparent" {
        return Rgba {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        };
    }
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

fn size_of(code: &str) -> AvatarSize {
    match code {
        "0" => AvatarSize::Small,
        "1" => AvatarSize::Default,
        "2" => AvatarSize::Large,
        other => panic!("坏尺寸 {other}"),
    }
}

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}avatar_matrix_{suffix}.txt")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 8, "坏行: {line}");
        let style = avatar_state(tokens, size_of(p[0]), p[1] == "1");
        assert_close(style.bg, parse(p[2]), "底色", line);
        assert_close(style.text, parse(p[3]), "文字色", line);
        assert_f32(style.size, p[4], "边长", line);
        assert_f32(style.radius, p[5], "圆角", line);
        assert_f32(style.font_size, p[6], "字号", line);
        assert_f32(style.line_height, p[7], "行高", line);
        checked += 1;
    }
    assert_eq!(checked, 6, "avatar_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn avatar_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn avatar_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn avatar_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 尺寸 24/32/40、方形圆角 4/6/8；文字始终 14px / 22 行高。
#[test]
fn avatar_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let default = avatar_state(&tokens, AvatarSize::Default, false);
    assert_eq!(default.size, 32.0);
    assert_eq!(default.radius, 16.0);
    assert_eq!(default.bg, tokens.color_text_placeholder);
    assert_eq!(default.text, tokens.color_text_light_solid);
    assert_eq!(default.font_size, tokens.font_size);
    assert_eq!(default.line_height, tokens.font_height);

    let small = avatar_state(&tokens, AvatarSize::Small, true);
    assert_eq!(small.size, 24.0);
    assert_eq!(small.radius, tokens.border_radius_sm);

    let large = avatar_state(&tokens, AvatarSize::Large, true);
    assert_eq!(large.size, 40.0);
    assert_eq!(large.radius, tokens.border_radius_lg);
}
