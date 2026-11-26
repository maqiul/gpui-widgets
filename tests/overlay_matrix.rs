//! Tooltip / Popover 浮层矩阵：气泡容器色值与度量。
//!
//! 基准由 `harness/antd_baseline/gen_overlay_matrix.mjs` 依据 antdv-next Tooltip / Popover
//! 样式生成，关键值已用 `harness/_tooltip_probe.html` 的 computed style 核对。

use gpui::Hsla;
use gpui_widgets::popover::popover_state;
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive};
use gpui_widgets::tooltip::tooltip_state;

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

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}tooltip_matrix_{suffix}.txt")).unwrap();
    let p: Vec<&str> = text.trim().split('|').collect();
    assert_eq!(p.len(), 9, "tooltip_matrix_{suffix} 列数");
    let tooltip = tooltip_state(tokens);
    assert_close(tooltip.bg, parse(p[0]), "气泡底色", suffix);
    assert_close(tooltip.fg, parse(p[1]), "气泡文字色", suffix);
    assert_eq!(
        tooltip.padding_block,
        p[2].parse::<f32>().unwrap(),
        "{suffix} 纵向内边距"
    );
    assert_eq!(
        tooltip.padding_inline,
        p[3].parse::<f32>().unwrap(),
        "{suffix} 横向内边距"
    );
    assert_eq!(
        tooltip.radius,
        p[4].parse::<f32>().unwrap(),
        "{suffix} 圆角"
    );
    assert_eq!(
        tooltip.min_height,
        p[5].parse::<f32>().unwrap(),
        "{suffix} 最小高度"
    );
    assert_eq!(
        tooltip.max_width,
        p[6].parse::<f32>().unwrap(),
        "{suffix} 最大宽度"
    );
    assert_eq!(
        tooltip.font_size,
        p[7].parse::<f32>().unwrap(),
        "{suffix} 字号"
    );
    assert_eq!(
        tooltip.line_height,
        p[8].parse::<f32>().unwrap(),
        "{suffix} 行高"
    );

    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}popover_matrix_{suffix}.txt")).unwrap();
    let p: Vec<&str> = text.trim().split('|').collect();
    assert_eq!(p.len(), 6, "popover_matrix_{suffix} 列数");
    let popover = popover_state(tokens);
    assert_close(popover.bg, parse(p[0]), "气泡底色", suffix);
    assert_close(popover.fg, parse(p[1]), "气泡文字色", suffix);
    assert_eq!(
        popover.padding,
        p[2].parse::<f32>().unwrap(),
        "{suffix} 内边距"
    );
    assert_eq!(
        popover.radius,
        p[3].parse::<f32>().unwrap(),
        "{suffix} 圆角"
    );
    assert_eq!(
        popover.font_size,
        p[4].parse::<f32>().unwrap(),
        "{suffix} 字号"
    );
    assert_eq!(
        popover.line_height,
        p[5].parse::<f32>().unwrap(),
        "{suffix} 行高"
    );
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn overlay_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn overlay_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn overlay_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 气泡度量全部来自令牌：tooltip 纵向 padding 取 `paddingSM / 2`、最小高 `controlHeight`；
/// popover 内边距 `paddingSM`、圆角 `borderRadiusLG`。
#[test]
fn overlay_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let tooltip = tooltip_state(&tokens);
    assert_eq!(tooltip.padding_block, tokens.padding_sm / 2.0);
    assert_eq!(tooltip.padding_inline, tokens.padding_xs);
    assert_eq!(tooltip.radius, tokens.border_radius);
    assert_eq!(tooltip.min_height, tokens.control_height);
    assert_eq!(tooltip.line_height, tokens.font_height);

    let popover = popover_state(&tokens);
    assert_eq!(popover.padding, tokens.padding_sm);
    assert_eq!(popover.radius, tokens.border_radius_lg);
    assert_eq!(popover.fg, tokens.color_text);
}
