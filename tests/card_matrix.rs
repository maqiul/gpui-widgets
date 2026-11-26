//! Card 矩阵：有边框 / 无边框。
//!
//! 基准由 `harness/antd_baseline/gen_card_matrix.mjs` 依据 antdv-next
//! `card/style/index.js` 的组件 token 生成。

use gpui::Hsla;
use gpui_widgets::card::card_state;
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

fn check(tokens: &Tokens, suffix: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}card_matrix_{suffix}.txt")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 18, "坏行: {line}");
        let style = card_state(tokens, p[0] == "1");
        assert_close(style.bg, parse(p[1]), "底色", line);
        assert_close(style.border, parse(p[2]), "描边色", line);
        assert_close(style.title, parse(p[3]), "标题色", line);
        assert_close(style.extra, parse(p[4]), "extra 色", line);
        assert_close(style.text, parse(p[5]), "正文色", line);
        assert_close(style.actions, parse(p[6]), "操作色", line);
        assert_close(style.action_hover, parse(p[7]), "操作悬停色", line);
        assert_f32(style.radius, p[8], "圆角", line);
        assert_f32(style.header_height, p[9], "头高", line);
        assert_f32(style.header_padding, p[10], "头内边距", line);
        assert_f32(style.header_font_size, p[11], "头字号", line);
        assert_f32(style.body_padding, p[12], "body 内边距", line);
        assert_f32(style.actions_padding, p[13], "操作栏内边距", line);
        assert_f32(style.actions_font_size, p[14], "操作栏字号", line);
        assert_f32(style.line_height, p[15], "行高", line);
        assert_f32(style.border_width, p[16], "描边宽", line);
        assert_f32(style.divider_width, p[17], "分隔线宽", line);
        checked += 1;
    }
    assert_eq!(checked, 2, "card_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn card_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn card_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn card_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 头高 56、内边距 24、标题 16；无边框只保留阴影。
#[test]
fn card_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let bordered = card_state(&tokens, true);
    assert_eq!(bordered.bg, tokens.color_bg_container);
    assert_eq!(bordered.border, tokens.color_border_secondary);
    assert_eq!(bordered.radius, tokens.border_radius_lg);
    assert_eq!(bordered.header_font_size, tokens.font_size_lg);
    assert_eq!(
        bordered.header_height,
        tokens.font_height_lg + tokens.padding * 2.0
    );
    assert_eq!(bordered.body_padding, tokens.padding_lg);
    assert_eq!(bordered.actions_padding, 12.0);
    assert_eq!(bordered.border_width, tokens.line_width);
    assert_eq!(bordered.divider_width, tokens.line_width);
    assert!(bordered.shadow.is_empty());

    let plain = card_state(&tokens, false);
    assert_eq!(plain.border_width, 0.0);
    assert_eq!(plain.divider_width, tokens.line_width);
    assert!(!plain.shadow.is_empty());
}
