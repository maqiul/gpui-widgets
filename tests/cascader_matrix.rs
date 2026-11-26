//! Cascader 矩阵：选项三态 + 列几何 + 触发框默认宽。
//!
//! 基准由 `harness/antd_baseline/gen_select_ext_matrix.mjs` 依据 antdv-next
//! `cascader/style/index.js` + `cascader/style/columns.js` 生成。
use gpui::Hsla;
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

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

fn baseline(name: &str, suffix: &str) -> Vec<String> {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}{name}_matrix_{suffix}.txt")).unwrap();
    text.lines()
        .next()
        .unwrap()
        .split('|')
        .map(str::to_string)
        .collect()
}

use gpui_widgets::theme::metric::{
    cascader_column_border, cascader_icon_color, cascader_item_state, cascader_menu_bg,
    cascader_metric, select_content_open_opacity,
};

fn check(tokens: &Tokens, suffix: &str) {
    let p = baseline("cascader", suffix);
    assert_eq!(p.len(), 21, "坏行: {p:?}");
    let row = format!("{p:?}");

    let idle = cascader_item_state(tokens, false, false, false);
    assert_close(idle.fg, parse(&p[0]), "常态文字", &row);
    assert_eq!(rgba(idle.bg).a, 0.0, "常态无底");
    assert_f32(idle.font_weight.0, "400", "常态字重", &row);

    let selected = cascader_item_state(tokens, true, false, false);
    assert_close(selected.bg, parse(&p[1]), "路径项底", &row);
    assert_close(selected.fg, parse(&p[0]), "路径项文字", &row);
    assert_f32(selected.font_weight.0, &p[4], "路径项字重", &row);

    let hover = cascader_item_state(tokens, false, true, false);
    assert_close(hover.bg, parse(&p[2]), "悬停底", &row);
    assert_close(hover.fg, parse(&p[0]), "悬停文字", &row);

    let disabled = cascader_item_state(tokens, false, false, true);
    assert_close(disabled.fg, parse(&p[3]), "禁用文字", &row);
    assert_eq!(rgba(disabled.bg).a, 0.0, "禁用无底");

    assert_close(
        cascader_icon_color(tokens, false),
        parse(&p[19]),
        "箭头色",
        &row,
    );
    assert_close(
        cascader_icon_color(tokens, true),
        parse(&p[3]),
        "禁用箭头色",
        &row,
    );
    assert_close(
        cascader_column_border(tokens),
        parse(&p[16]),
        "列分隔色",
        &row,
    );
    assert_close(cascader_menu_bg(tokens), parse(&p[20]), "浮层底", &row);

    let m = cascader_metric(tokens);
    assert_f32(m.option_pad_block, &p[5], "选项纵向内边距", &row);
    assert_f32(m.option_pad_inline, &p[6], "选项横向内边距", &row);
    assert_f32(m.menu_pad, &p[7], "列内边距", &row);
    assert_f32(m.item_radius, &p[8], "选项圆角", &row);
    assert_f32(m.font_size, &p[9], "选项字号", &row);
    assert_f32(m.line_height, &p[10], "选项行高", &row);
    assert_f32(m.icon_gap, &p[11], "箭头间距", &row);
    assert_f32(m.icon_font, &p[12], "箭头字号", &row);
    assert_f32(m.item_min_width, &p[13], "列最小宽", &row);
    assert_f32(m.dropdown_height, &p[14], "列高", &row);
    assert_f32(m.control_width, &p[15], "触发框默认宽", &row);
    assert_f32(m.column_border, &p[17], "列分隔线宽", &row);
    assert_f32(m.panel_radius, &p[18], "浮层圆角", &row);
}

#[test]
fn cascader_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn cascader_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn cascader_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn cascader_item_state_matrix() {
    for appearance in [Appearance::Light, Appearance::Dark] {
        let tokens = tokens_for(None, appearance);
        let suffix = if appearance == Appearance::Light {
            "light"
        } else {
            "dark"
        };
        let p = baseline("cascader", suffix);
        // active 优先于 hover：路径上的项悬停仍保持选中底与加粗
        let both = cascader_item_state(&tokens, true, true, false);
        assert_close(both.bg, parse(&p[1]), "路径项悬停仍选中底", suffix);
        assert_f32(both.font_weight.0, &p[4], "路径项悬停字重", suffix);
        // 禁用项悬停不换底、不变色
        let disabled_hover = cascader_item_state(&tokens, false, true, true);
        assert_close(disabled_hover.fg, parse(&p[3]), "禁用悬停文字", suffix);
        assert_eq!(rgba(disabled_hover.bg).a, 0.0, "禁用悬停无底");
        let disabled_selected = cascader_item_state(&tokens, true, false, true);
        assert_close(disabled_selected.fg, parse(&p[3]), "禁用路径项文字", suffix);
        assert_eq!(rgba(disabled_selected.bg).a, 0.0, "禁用路径项无底");
        assert_f32(
            select_content_open_opacity(),
            "0.25",
            "展开时已选值透明度",
            suffix,
        );
    }
}
