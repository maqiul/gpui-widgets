//! Tour 矩阵：卡片/遮罩/关闭钮/指示点配色与几何。
//!
//! 基准由 `harness/antd_baseline/gen_tour_matrix.mjs` 依据 antdv-next
//! `tour/style/index.js` + `panelRender.js` + `@v-c/tour` Mask 默认 fill 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{tour_metric, tour_style},
};

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

fn check(tokens: &Tokens, suffix: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}tour_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 22, "坏行: {line}");

    let style = tour_style(tokens);
    let m = tour_metric(tokens);
    assert_close(style.bg, parse(p[0]), "卡片底", line);
    assert_close(style.mask, parse(p[1]), "遮罩", line);
    assert_close(style.close, parse(p[2]), "关闭钮", line);
    assert_close(style.close_hover, parse(p[3]), "关闭悬停字", line);
    assert_close(style.close_hover_bg, parse(p[4]), "关闭悬停底", line);
    assert_close(style.close_active_bg, parse(p[5]), "关闭按下底", line);
    assert_close(style.text, parse(p[6]), "正文色", line);
    assert_close(style.indicator, parse(p[7]), "指示点", line);
    assert_close(style.indicator_active, parse(p[8]), "选中指示点", line);
    assert_f32(m.width, p[9], "卡片宽", line);
    assert_f32(m.radius, p[10], "卡片圆角", line);
    assert_f32(m.padding, p[11], "内边距", line);
    assert_f32(m.padding_xs, p[12], "小内边距", line);
    assert_f32(m.close_size, p[13], "关闭钮尺寸", line);
    assert_f32(m.close_radius, p[14], "关闭钮圆角", line);
    assert_f32(m.close_icon, p[15], "关闭图标字号", line);
    assert_f32(m.indicator, p[16], "指示点尺寸", line);
    assert_f32(m.indicator_gap, p[17], "指示点间距", line);
    assert_f32(m.btn_gap, p[18], "按钮间距", line);
    assert_f32(m.title_weight, p[19], "标题字重", line);
    assert_f32(m.text_size, p[20], "正文字号", line);
    assert_f32(m.text_line_height, p[21], "正文行高", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn tour_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn tour_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn tour_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn tour_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = tour_metric(&tokens);
    assert_eq!(m.width, 520.0);
    assert_eq!(m.close_size, 22.0);
    assert_eq!(m.indicator, 6.0);
    assert_eq!(m.indicator_gap, 6.0);
    assert_eq!(m.btn_gap, 8.0);
    assert_eq!(m.title_weight, 600.0);
}
