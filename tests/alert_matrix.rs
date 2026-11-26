//! Alert 矩阵：四种变体 × 默认 / 带描述 / banner。
//!
//! 基准由 `harness/antd_baseline/gen_alert_matrix.mjs` 依据 antdv-next
//! `alert/style/index.js` 的组件 token 生成。

use gpui::Hsla;
use gpui_widgets::alert::{AlertVariant, alert_state};
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

fn variant(name: &str) -> AlertVariant {
    match name {
        "info" => AlertVariant::Info,
        "success" => AlertVariant::Success,
        "warning" => AlertVariant::Warning,
        "error" => AlertVariant::Danger,
        other => panic!("坏变体 {other}"),
    }
}

fn check(tokens: &Tokens, suffix: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}alert_matrix_{suffix}.txt")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 15, "坏行: {line}");
        let style = alert_state(tokens, variant(p[0]), p[1] == "1", p[2] == "1");
        assert_close(style.bg, parse(p[3]), "底色", line);
        assert_close(style.border, parse(p[4]), "描边色", line);
        assert_close(style.icon, parse(p[5]), "图标色", line);
        assert_f32(style.radius, p[6], "圆角", line);
        assert_f32(style.padding_v, p[7], "纵向内边距", line);
        assert_f32(style.padding_h, p[8], "横向内边距", line);
        assert_f32(style.icon_gap, p[9], "图标间距", line);
        assert_f32(style.icon_size, p[10], "图标尺寸", line);
        assert_f32(style.title_size, p[11], "标题字号", line);
        assert_f32(style.title_margin, p[12], "标题下边距", line);
        assert_f32(style.line_height, p[13], "行高", line);
        assert_f32(style.border_width, p[14], "描边宽", line);
        checked += 1;
    }
    assert_eq!(checked, 6, "alert_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn alert_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn alert_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn alert_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 带描述：20×24 内边距、24 图标、16 标题；banner：无边框圆角。
#[test]
fn alert_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let plain = alert_state(&tokens, AlertVariant::Info, false, false);
    assert_eq!(plain.padding_v, 8.0);
    assert_eq!(plain.padding_h, 12.0);
    assert_eq!(plain.icon_size, 14.0);
    assert_eq!(plain.title_size, tokens.font_size);
    assert_eq!(plain.radius, tokens.border_radius_lg);
    assert_eq!(plain.bg, tokens.color_info_bg);
    assert_eq!(plain.icon, tokens.color_info);

    let described = alert_state(&tokens, AlertVariant::Success, true, false);
    assert_eq!(described.padding_v, 20.0);
    assert_eq!(described.padding_h, 24.0);
    assert_eq!(described.icon_size, tokens.font_size_heading3);
    assert_eq!(described.title_size, tokens.font_size_lg);
    assert_eq!(described.title_margin, 8.0);

    let banner = alert_state(&tokens, AlertVariant::Warning, false, true);
    assert_eq!(banner.radius, 0.0);
    assert_eq!(banner.border_width, 0.0);
}
