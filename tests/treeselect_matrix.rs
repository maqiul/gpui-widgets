//! TreeSelect 矩阵：浮层内边距/底色 + 内部树度量与取色。
//!
//! 基准由 `harness/antd_baseline/gen_select_ext_matrix.mjs` 依据 antdv-next
//! `tree-select/style/index.js` + `tree/style/index.js` 生成。
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
    TreeMetric, tree_select_menu_bg, tree_select_metric, tree_style,
};

fn check(tokens: &Tokens, suffix: &str) {
    let p = baseline("treeselect", suffix);
    assert_eq!(p.len(), 12, "坏行: {p:?}");
    let row = format!("{p:?}");

    let m = tree_select_metric(tokens);
    assert_f32(m.pad_block, &p[0], "浮层纵向内边距", &row);
    assert_f32(m.pad_inline, &p[1], "浮层横向内边距", &row);
    assert_f32(m.radius, &p[2], "浮层圆角", &row);
    assert_close(tree_select_menu_bg(tokens), parse(&p[3]), "浮层底", &row);

    let tree = TreeMetric::resolve(tokens);
    assert_f32(tree.title_height, &p[4], "树节点高", &row);
    assert_f32(tree.indent, &p[5], "树缩进", &row);
    assert_f32(tree.content_padding_inline, &p[6], "树内容内边距", &row);
    assert_f32(tree.radius, &p[7], "树节点圆角", &row);

    let style = tree_style(tokens);
    assert_close(style.selected_bg, parse(&p[8]), "选中底", &row);
    assert_close(style.hover_bg, parse(&p[9]), "悬停底", &row);
    assert_close(style.text, parse(&p[10]), "节点文字", &row);
    assert_close(style.switcher, parse(&p[11]), "展开箭头色", &row);
}

#[test]
fn treeselect_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn treeselect_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn treeselect_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}
