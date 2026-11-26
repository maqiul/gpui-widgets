//! Layout 矩阵：header/footer/sider/trigger 度量与底色。
//!
//! 基准由 `harness/antd_baseline/gen_layout_grid_matrix.mjs` 依据 antdv-next
//! `layout/style/index.js` prepareComponentToken + `sider.js` 生成。
use gpui::Hsla;
use gpui_widgets::theme::metric::layout_metric;
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
    let p = baseline("layout", suffix);
    assert_eq!(p.len(), 17, "坏行: {p:?}");
    let row = format!("{p:?}");

    let m = layout_metric(tokens);
    assert_f32(m.header_height, &p[0], "header 高", &row);
    assert_f32(m.header_padding_inline, &p[1], "header 横向内边距", &row);
    assert_close(m.header_bg, parse(&p[2]), "header 底", &row);
    assert_close(m.header_color, parse(&p[3]), "header 文字", &row);
    assert_f32(m.footer_padding_block, &p[4], "footer 纵向内边距", &row);
    assert_f32(m.footer_padding_inline, &p[5], "footer 横向内边距", &row);
    assert_close(m.footer_bg, parse(&p[6]), "footer 底", &row);
    assert_close(m.footer_color, parse(&p[7]), "footer 文字", &row);
    assert_f32(m.footer_font_size, &p[8], "footer 字号", &row);
    assert_f32(m.font_size, &p[8], "header 字号", &row);
    assert_close(m.body_bg, parse(&p[9]), "body 底", &row);
    assert_close(m.sider_bg, parse(&p[10]), "sider 底", &row);
    assert_close(m.light_sider_bg, parse(&p[11]), "light sider 底", &row);
    assert_f32(m.trigger_height, &p[12], "trigger 高", &row);
    assert_close(m.trigger_bg, parse(&p[13]), "trigger 底", &row);
    assert_close(m.trigger_color, parse(&p[14]), "trigger 文字", &row);
    assert_close(m.light_trigger_bg, parse(&p[15]), "light trigger 底", &row);
    assert_close(
        m.light_trigger_color,
        parse(&p[16]),
        "light trigger 文字",
        &row,
    );
}

#[test]
fn layout_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn layout_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn layout_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn layout_literal_matrix() {
    // navy 常量不随种子 / 明暗变化；header 行高等于 header 高
    for tokens in [
        tokens_for(None, Appearance::Light),
        tokens_for(None, Appearance::Dark),
        tokens_for(Some(0x8b5cf6), Appearance::Light),
    ] {
        let m = layout_metric(&tokens);
        assert_close(m.header_bg, parse("#001529"), "navy header", "literal");
        assert_close(m.sider_bg, parse("#001529"), "navy sider", "literal");
        assert_close(m.trigger_bg, parse("#002140"), "navy trigger", "literal");
        assert_f32(m.header_height, "64", "header 高不随主题", "literal");
    }
}
