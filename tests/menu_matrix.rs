//! Dropdown / ContextMenu 菜单矩阵：动作项 × 危险/禁用 + 容器 + 分隔线。
//!
//! 基准由 `harness/antd_baseline/gen_menu_matrix.mjs` 依据 antdv-next Dropdown 样式
//! 与 antd 实测 tokens 生成，关键值已用 `harness/_dropdown_probe.html` 的 computed style 核对。

use gpui::Hsla;
use gpui_widgets::menu::{divider_state, item_state, menu_state};
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

fn check(tokens: &Tokens, suffix: &str) {
    let items =
        std::fs::read_to_string(format!("{BASELINE_DIR}menu_item_matrix_{suffix}.txt")).unwrap();
    let mut checked = 0;
    for line in items.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 11, "坏行: {line}");
        let style = item_state(tokens, p[0] == "1", p[1] == "1");
        assert_close(style.bg, parse(p[2]), "底色", line);
        assert_close(style.fg, parse(p[3]), "文字色", line);
        assert_close(style.hover_bg, parse(p[4]), "悬停底色", line);
        assert_eq!(style.height, p[5].parse::<f32>().unwrap(), "{line} 高度");
        assert_eq!(
            style.padding_block,
            p[6].parse::<f32>().unwrap(),
            "{line} 纵向内边距"
        );
        assert_eq!(
            style.padding_inline,
            p[7].parse::<f32>().unwrap(),
            "{line} 横向内边距"
        );
        assert_eq!(style.radius, p[8].parse::<f32>().unwrap(), "{line} 圆角");
        assert_eq!(style.font_size, p[9].parse::<f32>().unwrap(), "{line} 字号");
        assert_eq!(
            style.line_height,
            p[10].parse::<f32>().unwrap(),
            "{line} 行高"
        );
        checked += 1;
    }
    assert_eq!(checked, 4, "menu_item_matrix_{suffix} 行数");

    let panel =
        std::fs::read_to_string(format!("{BASELINE_DIR}menu_panel_matrix_{suffix}.txt")).unwrap();
    let p: Vec<&str> = panel.trim().split('|').collect();
    let menu = menu_state(tokens);
    assert_close(menu.bg, parse(p[0]), "菜单底色", suffix);
    assert_eq!(
        menu.radius,
        p[1].parse::<f32>().unwrap(),
        "{suffix} 菜单圆角"
    );
    assert_eq!(
        menu.padding,
        p[2].parse::<f32>().unwrap(),
        "{suffix} 菜单内边距"
    );

    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}menu_divider_matrix_{suffix}.txt")).unwrap();
    let p: Vec<&str> = text.trim().split('|').collect();
    let divider = divider_state(tokens);
    assert_close(divider.color, parse(p[0]), "分隔线颜色", suffix);
    assert_eq!(
        divider.width,
        p[1].parse::<f32>().unwrap(),
        "{suffix} 分隔线宽"
    );
    assert_eq!(
        divider.margin,
        p[2].parse::<f32>().unwrap(),
        "{suffix} 分隔线外边距"
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
fn menu_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn menu_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn menu_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 菜单度量与 Select 菜单同源：容器 `paddingXXS` / `borderRadiusLG`，动作项 32 高 / `controlPaddingHorizontal`
#[test]
fn menu_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let menu = menu_state(&tokens);
    assert_eq!(menu.padding, tokens.padding_xxs);
    assert_eq!(menu.radius, tokens.border_radius_lg);
    let item = item_state(&tokens, false, false);
    assert_eq!(item.height, tokens.control_height);
    assert_eq!(item.padding_inline, tokens.control_padding_horizontal);
    assert_eq!(item.line_height, tokens.font_height);
    assert_eq!(item.radius, tokens.border_radius_sm);
}
