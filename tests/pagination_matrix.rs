//! Pagination 矩阵：页码项（small × active × disabled）与翻页按钮（small × disabled）。
//!
//! 基准由 `harness/antd_baseline/gen_pagination_matrix.mjs` 依据 antdv-next Pagination
//! 样式生成，关键值已用 `harness/_pagination_probe.html` 的 computed style 核对。

use gpui::Hsla;
use gpui_widgets::pagination::{item_state, nav_state};
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

fn assert_style(
    style: &gpui_widgets::pagination::PaginationStyle,
    p: &[&str],
    offset: usize,
    row: &str,
) {
    assert_close(style.bg, parse(p[offset]), "底色", row);
    assert_close(style.fg, parse(p[offset + 1]), "文字色", row);
    assert_close(style.border, parse(p[offset + 2]), "描边色", row);
    assert_close(style.hover_bg, parse(p[offset + 3]), "悬停底色", row);
    assert_close(style.hover_fg, parse(p[offset + 4]), "悬停文字色", row);
    assert_close(style.hover_border, parse(p[offset + 5]), "悬停描边色", row);
    assert_eq!(
        style.size,
        p[offset + 6].parse::<f32>().unwrap(),
        "{row} 尺寸"
    );
    assert_eq!(
        style.spacing,
        p[offset + 7].parse::<f32>().unwrap(),
        "{row} 间距"
    );
    assert_eq!(
        style.radius,
        p[offset + 8].parse::<f32>().unwrap(),
        "{row} 圆角"
    );
    assert_eq!(
        style.font_size,
        p[offset + 9].parse::<f32>().unwrap(),
        "{row} 字号"
    );
    assert_eq!(
        style.font_weight.0,
        p[offset + 10].parse::<f32>().unwrap(),
        "{row} 字重"
    );
    assert_eq!(
        style.line_width,
        p[offset + 11].parse::<f32>().unwrap(),
        "{row} 描边宽"
    );
    assert_eq!(
        style.padding_inline,
        p[offset + 12].parse::<f32>().unwrap(),
        "{row} 横向内边距"
    );
}

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}pagination_item_matrix_{suffix}.txt"))
            .unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 16, "坏行: {line}");
        let style = item_state(tokens, p[0] == "1", p[1] == "1", p[2] == "1");
        assert_style(&style, &p, 3, line);
        checked += 1;
    }
    assert_eq!(checked, 8, "pagination_item_matrix_{suffix} 行数");

    let text = std::fs::read_to_string(format!("{BASELINE_DIR}pagination_nav_matrix_{suffix}.txt"))
        .unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 15, "坏行: {line}");
        let style = nav_state(tokens, p[0] == "1", p[1] == "1");
        assert_style(&style, &p, 2, line);
        checked += 1;
    }
    assert_eq!(checked, 4, "pagination_nav_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn pagination_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn pagination_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn pagination_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 页码项尺寸/间距/圆角来自控制高度与间距令牌；选中态用强字重令牌。
#[test]
fn pagination_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let item = item_state(&tokens, false, false, false);
    assert_eq!(item.size, tokens.control_height);
    assert_eq!(item.spacing, tokens.margin_xs);
    assert_eq!(item.radius, tokens.border_radius);
    let small = item_state(&tokens, true, false, false);
    assert_eq!(small.size, tokens.control_height_sm);
    assert_eq!(small.spacing, tokens.padding_xxs);
    let active = item_state(&tokens, false, true, false);
    assert_eq!(active.font_weight.0, tokens.font_weight_strong);
    assert_eq!(active.fg, tokens.color_primary);
    let nav = nav_state(&tokens, false, true);
    assert_eq!(nav.fg, tokens.color_text_disabled);
}
