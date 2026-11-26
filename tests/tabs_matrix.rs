//! Tabs 矩阵：下划线 / 卡片 × 常态 / 悬停 / 按下 / 选中 / 禁用 / 选中禁用。
//!
//! 基准由 `harness/antd_baseline/gen_tabs_matrix.mjs` 依据 antdv-next
//! `tabs/style/index.js` 与组件 token 生成。

use gpui::Hsla;
use gpui_widgets::tabs::{TabsKind, TabsState, tabs_ink_state, tabs_item_state, tabs_rail_state};
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive, metric};

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

fn kind(text: &str) -> TabsKind {
    match text {
        "line" => TabsKind::Line,
        "card" => TabsKind::Card,
        other => panic!("坏形态 {other}"),
    }
}

fn state(text: &str) -> TabsState {
    match text {
        "normal" => TabsState::Normal,
        "hover" => TabsState::Hover,
        "active" => TabsState::Active,
        "selected" => TabsState::Selected,
        "disabled" => TabsState::Disabled,
        "selected-disabled" => TabsState::SelectedDisabled,
        other => panic!("坏状态 {other}"),
    }
}

fn check(tokens: &Tokens, suffix: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}tabs_matrix_{suffix}.txt")).unwrap();
    let m = metric::TabsMetric::resolve(tokens);
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 34, "坏行: {line}");
        let style = tabs_item_state(tokens, kind(p[0]), state(p[1]));
        assert_close(style.text, parse(p[2]), "文字色", line);
        assert_close(style.bg, parse(p[3]), "底色", line);
        assert_close(style.border, parse(p[4]), "边框色", line);
        assert_eq!(style.ink, p[5] == "1", "{line} 指示条");
        assert_f32(m.item_gap, p[6], "项间距", line);
        assert_f32(m.nav_margin_bottom, p[7], "导航底部外边距", line);
        assert_f32(m.padding_block[0], p[8], "内边距块 sm", line);
        assert_f32(m.padding_block[1], p[9], "内边距块", line);
        assert_f32(m.padding_block[2], p[10], "内边距块 lg", line);
        assert_f32(m.font_size[0], p[11], "字号 sm", line);
        assert_f32(m.font_size[1], p[12], "字号", line);
        assert_f32(m.font_size[2], p[13], "字号 lg", line);
        assert_f32(m.line_height[0], p[14], "行高 sm", line);
        assert_f32(m.line_height[1], p[15], "行高", line);
        assert_f32(m.line_height[2], p[16], "行高 lg", line);
        assert_f32(m.ink_height, p[17], "指示条高", line);
        assert_f32(m.divider_width, p[18], "分隔线宽", line);
        assert_f32(m.card_height[0], p[19], "卡片高 sm", line);
        assert_f32(m.card_height[1], p[20], "卡片高", line);
        assert_f32(m.card_height[2], p[21], "卡片高 lg", line);
        assert_f32(m.card_padding_block[0], p[22], "卡片内边距块 sm", line);
        assert_f32(m.card_padding_block[1], p[23], "卡片内边距块", line);
        assert_f32(m.card_padding_block[2], p[24], "卡片内边距块 lg", line);
        assert_f32(m.card_padding_inline[0], p[25], "卡片内边距行 sm", line);
        assert_f32(m.card_padding_inline[1], p[26], "卡片内边距行", line);
        assert_f32(m.card_padding_inline[2], p[27], "卡片内边距行 lg", line);
        assert_f32(m.card_radius[0], p[28], "卡片圆角 sm", line);
        assert_f32(m.card_radius[1], p[29], "卡片圆角", line);
        assert_f32(m.card_radius[2], p[30], "卡片圆角 lg", line);
        assert_f32(m.card_gutter, p[31], "卡片间隙", line);
        assert_f32(m.close_gap, p[32], "关闭按钮间距", line);
        assert_f32(m.close_size, p[33], "关闭按钮字号", line);
        checked += 1;
    }
    assert_eq!(checked, 12, "tabs_matrix_{suffix} 行数");
    assert_close(
        tabs_rail_state(tokens),
        parse(if suffix == "dark" {
            "#303030"
        } else {
            "#f0f0f0"
        }),
        "分隔线色",
        "rail",
    );
    assert_close(
        tabs_ink_state(tokens),
        parse(if suffix == "violet" {
            "#8b5cf6"
        } else if suffix == "dark" {
            "#1668dc"
        } else {
            "#1677ff"
        }),
        "指示条色",
        "ink",
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
fn tabs_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn tabs_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn tabs_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 下划线内边距 8/12/16、卡片高 32/40/48、项间距 32、指示条 2。
#[test]
fn tabs_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = metric::TabsMetric::resolve(&tokens);
    assert_eq!(m.item_gap, 32.0);
    assert_eq!(m.nav_margin_bottom, 16.0);
    assert_eq!(m.padding_block, [8.0, 12.0, 16.0]);
    assert_eq!(m.font_size, [14.0, 14.0, 16.0]);
    assert_eq!(m.line_height, [22.0, 22.0, 24.0]);
    assert_eq!(m.ink_height, 2.0);
    assert_eq!(m.divider_width, 1.0);
    assert_eq!(m.card_height, [32.0, 40.0, 48.0]);
    assert_eq!(m.card_padding_block, [4.0, 8.0, 11.0]);
    assert_eq!(m.card_padding_inline, [8.0, 16.0, 16.0]);
    assert_eq!(m.card_radius, [6.0, 8.0, 8.0]);
    assert_eq!(m.card_gutter, 2.0);
    assert_eq!(m.close_gap, 8.0);
    assert_eq!(m.close_size, 12.0);
}
