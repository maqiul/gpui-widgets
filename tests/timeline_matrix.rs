//! Timeline 矩阵：四种语义色 × 普通/进行中。
//!
//! 基准由 `harness/antd_baseline/gen_timeline_matrix.mjs` 依据 antdv-next
//! `timeline/style/index.js` 与组件 token 生成。

use gpui::Hsla;
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive, metric};
use gpui_widgets::timeline::{
    TimelineColor, TimelineLayout, timeline_dot_state, timeline_layout, timeline_rail_state,
    timeline_text_state,
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

fn color(text: &str) -> TimelineColor {
    match text {
        "blue" => TimelineColor::Blue,
        "red" => TimelineColor::Red,
        "green" => TimelineColor::Green,
        "gray" => TimelineColor::Gray,
        other => panic!("坏颜色名 {other}"),
    }
}

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}timeline_matrix_{suffix}.txt")).unwrap();
    let m = metric::TimelineMetric::resolve(tokens);
    let text_state = timeline_text_state(tokens);
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 28, "坏行: {line}");
        let loading = p[1] == "1";
        let dot = timeline_dot_state(tokens, color(p[0]), loading);
        let rail = timeline_rail_state(tokens, loading);
        assert_close(dot.border, parse(p[2]), "圆环色", line);
        assert_close(dot.fill, parse(p[3]), "圆环底", line);
        assert_f32(dot.size, p[4], "圆环尺寸", line);
        assert_f32(dot.border_width, p[5], "圆环描边", line);
        assert_f32(dot.inner_size, p[6], "内点尺寸", line);
        assert_close(dot.inner, parse(p[7]), "内点色", line);
        assert_close(rail.color, parse(p[8]), "连接线色", line);
        assert_f32(rail.width, p[9], "连接线宽", line);
        assert_eq!(rail.dotted, p[10] == "1", "{line} 连接线虚线");
        assert_close(text_state.title, parse(p[11]), "标题色", line);
        assert_close(text_state.content, parse(p[12]), "内容色", line);
        assert_close(text_state.meta, parse(p[13]), "辅助色", line);
        assert_f32(m.title_size, p[14], "标题字号", line);
        assert_f32(m.title_line, p[15], "标题行高", line);
        assert_f32(m.meta_size, p[16], "辅助字号", line);
        assert_f32(m.meta_line, p[17], "辅助行高", line);
        assert_f32(m.padding_plain, p[18], "单列下内边距", line);
        assert_f32(m.padding_alternate, p[19], "两栏下内边距", line);
        assert_f32(m.rail_top, p[20], "连接线顶", line);
        assert_f32(m.rail_bottom, p[21], "连接线越底", line);
        assert_f32(m.icon_gap, p[22], "节点内容间距", line);
        assert_f32(m.content_gap, p[23], "两栏间距", line);
        assert_f32(m.header_min_height, p[24], "栏最小高", line);
        assert_f32(m.dot_top, p[25], "节点距顶", line);
        assert_f32(m.item_min_height, p[26], "项最小高", line);
        assert_f32(m.dot_inner, p[27], "圆环内径", line);
        checked += 1;
    }
    assert_eq!(checked, 8, "timeline_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn timeline_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn timeline_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn timeline_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 自定义色原样透传；没有标题时贴左单列，有标题时两栏居中。
#[test]
fn timeline_custom_color_and_layout() {
    let tokens = tokens_for(None, Appearance::Light);
    let custom = Color::from_hex(0x00b96b).hsla();
    let dot = timeline_dot_state(&tokens, TimelineColor::Custom(custom), false);
    assert_eq!(dot.border, custom);
    assert_eq!(timeline_layout(false), TimelineLayout::Plain);
    assert_eq!(timeline_layout(true), TimelineLayout::Centered);
}

/// 圆环 10 / 描边 2 / 连接线 lineWidthBold / 两栏间距 40 / 单列最小高 48。
#[test]
fn timeline_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = metric::TimelineMetric::resolve(&tokens);
    assert_eq!(m.dot_size, 10.0);
    assert_eq!(m.dot_inner, 8.0);
    assert_eq!(m.dot_border, 2.0);
    assert_eq!(m.dot_top, 7.0);
    assert_eq!(m.icon_gap, 16.0);
    assert_eq!(m.content_gap, 40.0);
    assert_eq!(m.item_min_height, 48.0);
    assert_eq!(m.padding_plain, 12.0);
    assert_eq!(m.padding_alternate, 20.0);
    assert_eq!(m.rail_top, 17.0);
    assert_eq!(m.rail_bottom, 7.0);
    assert_eq!(m.rail_width, tokens.line_width_bold);
    assert_eq!(m.header_min_height, 24.0);
    assert_eq!(m.title_size, tokens.font_size);
    assert_eq!(m.title_line, tokens.font_height);
    assert_eq!(m.meta_size, 12.0);
    assert_eq!(m.meta_line, 20.0);
    assert_eq!(m.content_top, 1.0);
}
