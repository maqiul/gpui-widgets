//! Select 状态矩阵回归：触发框变体 / 状态 / 尺寸 / 交互态 + 菜单项 + 菜单容器。
//!
//! 基准由 `harness/antd_baseline/gen_select_matrix.mjs` 依据 antdv-next
//! `select/style/{token,select-input,dropdown}.js` + antd 实测 tokens 生成；
//! 禁用 / 状态叠加的关键规则另用浏览器 computed style 核对过。

use gpui::Hsla;
use gpui_widgets::select::{chip_state, menu_state, option_state, state};
use gpui_widgets::text_input::{InputSize, InputStatus, InputVariant};
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

fn variant_of(name: &str) -> InputVariant {
    match name {
        "outlined" => InputVariant::Outlined,
        "filled" => InputVariant::Filled,
        "borderless" => InputVariant::Borderless,
        "underlined" => InputVariant::Underlined,
        other => panic!("未知变体 {other}"),
    }
}

fn status_of(name: &str) -> InputStatus {
    match name {
        "default" => InputStatus::Default,
        "error" => InputStatus::Error,
        "warning" => InputStatus::Warning,
        other => panic!("未知状态 {other}"),
    }
}

fn size_of(name: &str) -> InputSize {
    match name {
        "small" => InputSize::Small,
        "middle" => InputSize::Middle,
        "large" => InputSize::Large,
        other => panic!("未知尺寸 {other}"),
    }
}

fn colors(text: &str) -> Vec<Rgba> {
    text.split(',').map(parse).collect()
}

fn numbers(text: &str) -> Vec<f32> {
    text.split(',').map(|v| v.parse().unwrap()).collect()
}

fn check_trigger(tokens: &Tokens, file: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}{file}")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 16, "坏行: {line}");
        let (variant, status, size) = (variant_of(p[0]), status_of(p[1]), size_of(p[2]));
        let disabled = p[3] == "1";
        let (hover, focused) = match p[4] {
            "normal" => (false, false),
            "hover" => (true, false),
            "focused" => (false, true),
            other => panic!("未知交互态 {other}"),
        };
        let style = state(variant, status, size, tokens, hover, focused, disabled);

        assert_close(style.fg, parse(p[5]), "文字色", line);
        assert_close(style.bg, parse(p[6]), "底色", line);
        for (ix, expected) in colors(p[7]).iter().enumerate() {
            assert_close(style.border[ix], *expected, &format!("描边[{ix}]"), line);
        }
        let widths = numbers(p[8]);
        assert_eq!(
            style.border_width,
            [widths[0], widths[1], widths[2], widths[3]],
            "{line} 描边宽"
        );
        let shadow = p[9].split(';').collect::<Vec<_>>();
        let shadow_w: f32 = shadow[0].parse().unwrap();
        match &style.shadow {
            Some(s) => {
                assert_eq!(s.spread_radius, gpui::px(shadow_w), "{line} 聚焦环宽");
                assert_close(s.color, parse(shadow[1]), "聚焦环色", line);
            }
            None => {
                assert_eq!(shadow_w, 0.0, "{line} 基准有聚焦环但 Rust 没有");
                assert_eq!(parse(shadow[1]).a, 0.0, "{line} 基准无聚焦环");
            }
        }
        let heights = numbers(&format!("{},{},{},{}", p[10], p[11], p[12], p[13]));
        assert_eq!(style.height, heights[0], "{line} 高度");
        assert_eq!(style.padding_block, heights[1], "{line} 纵向内边距");
        assert_eq!(style.padding_inline, heights[2], "{line} 横向内边距");
        assert_eq!(style.radius, heights[3], "{line} 圆角");
        assert_eq!(
            style.font_size,
            p[14].parse::<f32>().unwrap(),
            "{line} 字号"
        );
        assert_eq!(
            style.line_height,
            p[15].parse::<f32>().unwrap(),
            "{line} 行高"
        );
        checked += 1;
    }
    assert_eq!(checked, 216, "{file} 行数");
}

fn check_options(tokens: &Tokens, file: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}{file}")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 12, "坏行: {line}");
        let style = option_state(tokens, p[0] == "1", p[1] == "1", p[2] == "1");
        assert_close(style.bg, parse(p[3]), "底色", line);
        assert_close(style.fg, parse(p[4]), "文字色", line);
        assert_eq!(
            style.font_weight.0,
            p[5].parse::<f32>().unwrap(),
            "{line} 字重"
        );
        assert_eq!(
            style.height,
            p[6].parse::<f32>().unwrap(),
            "{line} 最小高度"
        );
        assert_eq!(
            style.padding_block,
            p[7].parse::<f32>().unwrap(),
            "{line} 纵向内边距"
        );
        assert_eq!(
            style.padding_inline,
            p[8].parse::<f32>().unwrap(),
            "{line} 横向内边距"
        );
        assert_eq!(style.radius, p[9].parse::<f32>().unwrap(), "{line} 圆角");
        assert_eq!(
            style.font_size,
            p[10].parse::<f32>().unwrap(),
            "{line} 字号"
        );
        assert_eq!(
            style.line_height,
            p[11].parse::<f32>().unwrap(),
            "{line} 行高"
        );
        checked += 1;
    }
    assert_eq!(checked, 8, "{file} 行数");
}

fn check_menu(tokens: &Tokens, file: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}{file}")).unwrap();
    let p: Vec<&str> = text.trim().split('|').collect();
    let style = menu_state(tokens);
    assert_close(style.bg, parse(p[0]), "底色", file);
    assert_eq!(style.radius, p[1].parse::<f32>().unwrap(), "{file} 圆角");
    assert_eq!(style.padding, p[2].parse::<f32>().unwrap(), "{file} 内边距");
}

/// 多选芯片：`variant|status|size|disabled|bg|fg|height|line|radius|padStart|padEnd|marginBlock|marginEnd|font|removeColor|removeSize`
fn check_chips(tokens: &Tokens, file: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}{file}")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 16, "坏行: {line}");
        let style = chip_state(
            variant_of(p[0]),
            status_of(p[1]),
            size_of(p[2]),
            tokens,
            p[3] == "1",
        );
        assert_close(style.bg, parse(p[4]), "芯片底色", line);
        assert_close(style.fg, parse(p[5]), "芯片文字色", line);
        assert_eq!(style.height, p[6].parse::<f32>().unwrap(), "{line} 芯片高");
        assert_eq!(
            style.line_height,
            p[7].parse::<f32>().unwrap(),
            "{line} 芯片行高"
        );
        assert_eq!(
            style.radius,
            p[8].parse::<f32>().unwrap(),
            "{line} 芯片圆角"
        );
        assert_eq!(
            style.padding_start,
            p[9].parse::<f32>().unwrap(),
            "{line} 芯片左内边距"
        );
        assert_eq!(
            style.padding_end,
            p[10].parse::<f32>().unwrap(),
            "{line} 芯片右内边距"
        );
        assert_eq!(
            style.margin_block,
            p[11].parse::<f32>().unwrap(),
            "{line} 芯片纵向外边距"
        );
        assert_eq!(
            style.margin_end,
            p[12].parse::<f32>().unwrap(),
            "{line} 芯片右外边距"
        );
        assert_eq!(
            style.font_size,
            p[13].parse::<f32>().unwrap(),
            "{line} 芯片字号"
        );
        assert_close(style.remove_color, parse(p[14]), "移除图标色", line);
        assert_eq!(
            style.remove_size,
            p[15].parse::<f32>().unwrap(),
            "{line} 移除图标字号"
        );
        checked += 1;
    }
    assert_eq!(checked, 72, "{file} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn select_light_matrix() {
    let tokens = tokens_for(None, Appearance::Light);
    check_trigger(&tokens, "select_trigger_matrix_light.txt");
    check_options(&tokens, "select_option_matrix_light.txt");
    check_menu(&tokens, "select_menu_matrix_light.txt");
    check_chips(&tokens, "select_chip_matrix_light.txt");
}

#[test]
fn select_dark_matrix() {
    let tokens = tokens_for(None, Appearance::Dark);
    check_trigger(&tokens, "select_trigger_matrix_dark.txt");
    check_options(&tokens, "select_option_matrix_dark.txt");
    check_menu(&tokens, "select_menu_matrix_dark.txt");
    check_chips(&tokens, "select_chip_matrix_dark.txt");
}

#[test]
fn select_violet_matrix() {
    let tokens = tokens_for(Some(0x8b5cf6), Appearance::Light);
    check_trigger(&tokens, "select_trigger_matrix_violet.txt");
    check_options(&tokens, "select_option_matrix_violet.txt");
    check_menu(&tokens, "select_menu_matrix_violet.txt");
    check_chips(&tokens, "select_chip_matrix_violet.txt");
}

/// 尺寸档：大尺寸只改高度 / 字号 / 行高 / 圆角，水平内边距与 middle 同档
#[test]
fn select_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = gpui_widgets::theme::metric::SelectMetric::resolve(&tokens);
    assert_eq!(
        m.height,
        [
            tokens.control_height_sm,
            tokens.control_height,
            tokens.control_height_lg
        ]
    );
    assert_eq!(
        m.padding_inline,
        [
            tokens.padding_xs - tokens.line_width,
            tokens.padding_sm - tokens.line_width,
            tokens.padding_sm - tokens.line_width
        ]
    );
    assert_eq!(m.option_height, tokens.control_height);
    assert_eq!(m.option_padding_inline, tokens.control_padding_horizontal);
    assert_eq!(m.menu_padding, tokens.padding_xxs);
    assert_eq!(m.menu_radius, tokens.border_radius_lg);
}
