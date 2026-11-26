//! NavMenu 矩阵：inline 度量与条目状态色。
//!
//! 基准由 `harness/antd_baseline/gen_navmenu_countdown_matrix.mjs` 依据 antdv-next
//! `menu/style/index.js` prepareComponentToken + `vertical.js` + rc-menu inlineIndent 生成。
use gpui::Hsla;
use gpui_widgets::theme::metric::{nav_menu_item_state, nav_menu_metric, nav_menu_parent_state};
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

fn check(tokens: &Tokens, suffix: &str) {
    let p = baseline("navmenu", suffix);
    assert_eq!(p.len(), 21, "坏行: {p:?}");
    let row = format!("{p:?}");

    let m = nav_menu_metric(tokens);
    assert_f32(m.item_height, &p[0], "条目高", &row);
    assert_f32(m.item_margin_inline, &p[1], "条目横向外边距", &row);
    assert_f32(m.item_margin_block, &p[2], "条目纵向外边距", &row);
    assert_f32(m.item_padding_inline, &p[3], "条目横向内边距", &row);
    assert_f32(m.item_radius, &p[4], "顶层圆角", &row);
    assert_f32(m.sub_item_radius, &p[5], "子级圆角", &row);
    assert_f32(m.inline_indent, &p[6], "层级缩进", &row);
    assert_f32(m.icon_size, &p[7], "图标字号", &row);
    assert_f32(m.icon_margin_inline_end, &p[8], "图标右间距", &row);
    assert_f32(m.arrow_size, &p[9], "箭头尺寸", &row);
    assert_f32(m.arrow_inset_end, &p[10], "箭头右偏移", &row);
    assert_f32(
        m.title_padding_end_with_arrow,
        &p[11],
        "父标题右内边距",
        &row,
    );
    assert_f32(m.font_size, &p[12], "菜单字号", &row);
    assert_close(m.sub_item_bg, parse(&p[13]), "子列表底", &row);
    assert_f32(m.border_width, &p[19], "root 右边框宽", &row);
    assert_close(m.border_color, parse(&p[20]), "root 右边框色", &row);

    let idle = nav_menu_item_state(tokens, false, false, false);
    assert_close(idle.bg, parse("rgba(0,0,0,0)"), "默认底透明", &row);
    assert_close(idle.fg, parse(&p[14]), "默认字", &row);
    let hover = nav_menu_item_state(tokens, false, true, false);
    assert_close(hover.bg, parse(&p[17]), "hover 底", &row);
    assert_close(hover.fg, parse(&p[14]), "hover 字不变", &row);
    let selected = nav_menu_item_state(tokens, true, false, false);
    assert_close(selected.bg, parse(&p[18]), "选中底", &row);
    assert_close(selected.fg, parse(&p[16]), "选中字", &row);
    let disabled = nav_menu_item_state(tokens, true, true, true);
    assert_close(disabled.bg, parse("rgba(0,0,0,0)"), "禁用底透明", &row);
    assert_close(disabled.fg, parse(&p[15]), "禁用字", &row);
}

#[test]
fn navmenu_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn navmenu_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn navmenu_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn navmenu_parent_state_matrix() {
    // 父级标题：后代选中仅字变主色，底保持透明；hover 才有底；禁用优先
    for tokens in [
        tokens_for(None, Appearance::Light),
        tokens_for(None, Appearance::Dark),
        tokens_for(Some(0x8b5cf6), Appearance::Light),
    ] {
        let p = nav_menu_parent_state(&tokens, true, false, false);
        assert_close(p.fg, rgba(tokens.color_primary), "后代选中父字", "parent");
        assert_close(p.bg, parse("rgba(0,0,0,0)"), "后代选中父底透明", "parent");
        let h = nav_menu_parent_state(&tokens, false, true, false);
        assert_close(
            h.bg,
            rgba(tokens.color_bg_text_hover),
            "父 hover 底",
            "parent",
        );
        assert_close(h.fg, rgba(tokens.color_text), "父 hover 字", "parent");
        let d = nav_menu_parent_state(&tokens, true, true, true);
        assert_close(d.fg, rgba(tokens.color_text_disabled), "父禁用字", "parent");
        assert_close(d.bg, parse("rgba(0,0,0,0)"), "父禁用底", "parent");
        // 缩进随层级线性增长
        let m = nav_menu_metric(&tokens);
        assert_f32(m.inline_indent * 3.0, "72", "三层缩进", "parent");
    }
}
