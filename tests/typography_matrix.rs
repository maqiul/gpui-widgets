//! Typography 色板与度量：标题 1-5、语义色、代码/标记常量。
//!
//! 基准由 `harness/antd_baseline/gen_typography_matrix.mjs` 依据 antdv-next
//! `typography/style/index.js` 与 `mixins.js` 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{self, TypographyMetric},
};
use gpui_widgets::typography::{TypographyType, truncate_text, typography_color};

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

fn rgba_u32(hex: u32) -> Rgba {
    Rgba {
        r: ((hex >> 24) & 0xff) as f32,
        g: ((hex >> 16) & 0xff) as f32,
        b: ((hex >> 8) & 0xff) as f32,
        a: (hex & 0xff) as f32 / 255.0,
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

fn assert_close(actual: Hsla, expected: Rgba, what: &str, theme: &str) {
    let a = rgba(actual);
    let near = (a.r - expected.r).abs() <= 1.0
        && (a.g - expected.g).abs() <= 1.0
        && (a.b - expected.b).abs() <= 1.0
        && (a.a - expected.a).abs() <= 0.01;
    assert!(
        near,
        "{theme} {what}: got rgba({},{},{},{}), want rgba({},{},{},{})",
        a.r, a.g, a.b, a.a, expected.r, expected.g, expected.b, expected.a
    );
}

fn assert_f32(actual: f32, expected: f32, what: &str, theme: &str) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "{theme} {what}: got {actual}, want {expected}"
    );
}

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}typography_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 26, "坏行: {line}");
    let m = TypographyMetric::resolve(tokens);

    let kinds = [
        (TypographyType::Default, 13, "默认"),
        (TypographyType::Secondary, 14, "次要"),
        (TypographyType::Success, 15, "成功"),
        (TypographyType::Warning, 16, "警告"),
        (TypographyType::Danger, 17, "错误"),
        (TypographyType::Disabled, 18, "禁用"),
    ];
    for (kind, ix, what) in kinds {
        assert_close(typography_color(tokens, kind), parse(p[ix]), what, suffix);
    }
    for ix in 0..5 {
        assert_f32(m.title_size[ix], p[ix].parse().unwrap(), "标题字号", suffix);
        assert_f32(
            m.title_line_height[ix],
            p[5 + ix].parse().unwrap(),
            "标题行高",
            suffix,
        );
    }
    assert_f32(
        m.title_margin_bottom,
        p[10].parse().unwrap(),
        "标题下距",
        suffix,
    );
    assert_f32(m.body_size, p[11].parse().unwrap(), "正文字号", suffix);
    let ratio: f32 = p[12].parse().unwrap();
    assert_f32(
        m.body_line_height,
        (m.body_size * ratio).round(),
        "正文行高",
        suffix,
    );
    assert_eq!(metric::TYPOGRAPHY_TITLE_WEIGHT, 600.0);
    assert_eq!(metric::TYPOGRAPHY_STRONG_WEIGHT, 600.0);
    assert_f32(
        metric::TYPOGRAPHY_CODE_SIZE_RATIO,
        p[21].parse().unwrap(),
        "代码字号比例",
        suffix,
    );
    assert_f32(
        metric::TYPOGRAPHY_CODE_RADIUS,
        p[22].parse().unwrap(),
        "代码圆角",
        suffix,
    );
    let code_bg = rgba_u32(metric::TYPOGRAPHY_CODE_BG);
    let expected_bg = parse(p[23]);
    assert!(
        (code_bg.a - expected_bg.a).abs() < 0.01,
        "{suffix} 代码底色 alpha"
    );
    let code_border = rgba_u32(metric::TYPOGRAPHY_CODE_BORDER);
    let expected_border = parse(p[24]);
    assert!(
        (code_border.a - expected_border.a).abs() < 0.01,
        "{suffix} 代码边框 alpha"
    );
    assert_eq!(metric::TYPOGRAPHY_MARK_BG, 0xffe58fff, "{suffix} mark 底色");
    assert!(!text.lines().next().unwrap().is_empty());
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn typography_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn typography_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn typography_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 标题 38/30/24/20/16、行高 46/38/32/28/24、下距 20；正文 14/22。
#[test]
fn typography_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = TypographyMetric::resolve(&tokens);
    assert_eq!(m.title_size, [38.0, 30.0, 24.0, 20.0, 16.0]);
    assert_eq!(m.title_line_height, [46.0, 38.0, 32.0, 28.0, 24.0]);
    assert_eq!(m.title_margin_bottom, 20.0);
    assert_eq!(m.body_size, 14.0);
    assert_eq!(m.body_line_height, 22.0);
}

#[test]
fn truncate_text_preserves_unicode_graphemes() {
    assert_eq!(truncate_text("abcdef", 3, "…"), "abc…");
    assert_eq!(truncate_text("你好世界", 2, "…"), "你好…");
    assert_eq!(truncate_text("e\u{301}clair", 1, "…"), "e\u{301}…");
    assert_eq!(truncate_text("👩‍💻abc", 1, "…"), "👩‍💻…");
}

#[test]
fn truncate_text_handles_boundaries() {
    assert_eq!(truncate_text("abc", 3, "…"), "abc");
    assert_eq!(truncate_text("abc", 5, "…"), "abc");
    assert_eq!(truncate_text("abc", 0, "…"), "…");
    assert_eq!(truncate_text("", 0, "…"), "");
}
