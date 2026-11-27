//! Badge 矩阵：计数 / 圆点 / 封顶 / 自定义色。
//!
//! 基准由 `harness/antd_baseline/gen_badge_matrix.mjs` 依据 antdv-next Badge
//! 样式生成，关键值已用 `harness/_badge_probe.html` 的 computed style 核对。

use gpui::{Hsla, px};
use gpui_widgets::badge::{
    badge_state, badge_state_sized, badge_status_color, badge_variant_colors,
};
use gpui_widgets::theme::metric::{BadgeMetric, badge_ring_shadow};
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive};
use gpui_widgets::{BadgeSize, BadgeStatus, BadgeVariant};

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

fn check(tokens: &Tokens, suffix: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}badge_matrix_{suffix}.txt")).unwrap();
    let metric = BadgeMetric::resolve(tokens);
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 11, "坏行: {line}");
        let dot = p[0] == "1";
        let color = if p[1] == "#1677ff" {
            Some(Color::from_hex(0x1677ff).hsla())
        } else {
            None
        };
        let style = badge_state(tokens, dot, color);
        assert_close(style.bg, parse(p[1]), "底色", line);
        assert_close(style.text, parse(p[2]), "文字色", line);
        assert_f32(style.height, p[3], "高度", line);
        assert_f32(style.radius, p[4], "圆角", line);
        assert_f32(style.font_size, p[5], "字号", line);
        assert_f32(style.line_height, p[6], "行高", line);
        assert_f32(style.dot_size, p[7], "圆点尺寸", line);
        let chars = if p[8] == "37" { 3 } else { 1 };
        let width = if dot {
            style.dot_size
        } else {
            metric.count_width(chars)
        };
        assert_f32(width, p[8], "角标宽", line);
        assert_close(style.ring.color, parse(p[9]), "外圈色", line);
        assert_eq!(style.ring.spread_radius, px(1.0), "{line} 外圈宽度");
        checked += 1;
    }
    assert_eq!(checked, 5, "badge_matrix_{suffix} 行数");
    let ring = badge_ring_shadow(tokens.color_bg_container);
    assert_eq!(ring.spread_radius, px(1.0));
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn badge_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn badge_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn badge_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 高 20、圆角 10、字号 12、圆点 6；默认底 error、自定义色直用。
#[test]
fn badge_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let count = badge_state(&tokens, false, None);
    assert_eq!(count.bg, tokens.color_error);
    assert_eq!(count.text, tokens.color_text_light_solid);
    assert_eq!(count.height, 20.0);
    assert_eq!(count.radius, 10.0);
    assert_eq!(count.font_size, tokens.font_size_sm);
    assert_eq!(count.line_height, 20.0);
    assert_eq!(count.dot_size, 6.0);
    assert_eq!(count.ring.color, tokens.color_bg_container);

    let custom = badge_state(&tokens, false, Some(Color::from_hex(0x8b5cf6).hsla()));
    assert_eq!(custom.bg, Color::from_hex(0x8b5cf6).hsla());

    let m = BadgeMetric::resolve(&tokens);
    assert_eq!(m.count_width(1), 20.0);
    assert_eq!(m.count_width(3), 37.0);
}

#[test]
fn badge_status_colors_follow_tokens() {
    for tokens in [
        tokens_for(None, Appearance::Light),
        tokens_for(None, Appearance::Dark),
        tokens_for(Some(0x8b5cf6), Appearance::Light),
    ] {
        assert_eq!(
            badge_status_color(&tokens, BadgeStatus::Default, None),
            tokens.color_text_placeholder
        );
        assert_eq!(
            badge_status_color(&tokens, BadgeStatus::Processing, None),
            tokens.color_info
        );
        assert_eq!(
            badge_status_color(&tokens, BadgeStatus::Success, None),
            tokens.color_success
        );
        assert_eq!(
            badge_status_color(&tokens, BadgeStatus::Error, None),
            tokens.color_error
        );
        assert_eq!(
            badge_status_color(&tokens, BadgeStatus::Warning, None),
            tokens.color_warning
        );
    }
}

#[test]
fn badge_variant_colors_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    assert_eq!(
        badge_variant_colors(&tokens, BadgeVariant::Neutral, None),
        (tokens.control_item_bg_hover, tokens.color_text_secondary)
    );
    assert_eq!(
        badge_variant_colors(&tokens, BadgeVariant::Info, None),
        (tokens.color_primary_bg, tokens.color_primary)
    );
    assert_eq!(
        badge_variant_colors(&tokens, BadgeVariant::Success, None),
        (tokens.color_success_bg, tokens.color_success)
    );
    assert_eq!(
        badge_variant_colors(&tokens, BadgeVariant::Warning, None),
        (tokens.color_warning_bg, tokens.color_warning)
    );
    assert_eq!(
        badge_variant_colors(&tokens, BadgeVariant::Danger, None),
        (tokens.color_error_bg, tokens.color_error)
    );
}

#[test]
fn badge_custom_color_overrides_variant_and_status() {
    let tokens = tokens_for(None, Appearance::Light);
    let custom = Color::from_hex(0x722ed1).hsla();
    let (bg, fg) = badge_variant_colors(&tokens, BadgeVariant::Danger, Some(custom));
    assert_eq!(fg, custom);
    assert_eq!(bg.a, 0.12);
    assert_eq!(
        badge_status_color(&tokens, BadgeStatus::Success, Some(custom)),
        custom
    );
}

#[test]
fn badge_small_count_metrics_are_compact() {
    let tokens = tokens_for(None, Appearance::Light);
    let normal = badge_state_sized(&tokens, false, None, BadgeSize::Default);
    let small = badge_state_sized(&tokens, false, None, BadgeSize::Small);
    assert_eq!(normal.height, 20.0);
    assert_eq!(small.height, 14.0);
    assert_eq!(small.min_width, 14.0);
    assert_eq!(small.radius, 7.0);
    assert_eq!(small.line_height, 14.0);
    assert_eq!(small.font_size, tokens.font_size_sm);
    assert_eq!(small.dot_size, normal.dot_size);
    assert_eq!(small.bg, tokens.color_error);
    assert_eq!(small.text, tokens.color_text_light_solid);
    assert_eq!(small.ring.color, tokens.color_bg_container);
}
