//! Segmented 矩阵：常态 / 悬停 / 按下 / 选中 / 禁用 / 选中禁用。
//!
//! 基准由 `harness/antd_baseline/gen_segmented_matrix.mjs` 依据 antdv-next
//! `segmented/style/index.js` 与组件 token 生成。

use gpui::Hsla;
use gpui_widgets::segmented::{SegmentedState, segmented_item_state, segmented_track_state};
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

fn state(text: &str) -> SegmentedState {
    match text {
        "normal" => SegmentedState::Normal,
        "hover" => SegmentedState::Hover,
        "active" => SegmentedState::Active,
        "selected" => SegmentedState::Selected,
        "disabled" => SegmentedState::Disabled,
        "selected-disabled" => SegmentedState::SelectedDisabled,
        other => panic!("坏状态 {other}"),
    }
}

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}segmented_matrix_{suffix}.txt")).unwrap();
    let m = metric::SegmentedMetric::resolve(tokens);
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 21, "坏行: {line}");
        let style = segmented_item_state(tokens, state(p[0]));
        assert_close(style.text, parse(p[1]), "文字色", line);
        assert_close(style.bg, parse(p[2]), "底色", line);
        assert_eq!(style.shadow, p[3] == "1", "{line} 阴影");
        assert_close(segmented_track_state(tokens), parse(p[4]), "轨道底色", line);
        assert_f32(m.track_padding, p[5], "轨道内边距", line);
        assert_f32(m.container_radius[0], p[6], "容器圆角 sm", line);
        assert_f32(m.container_radius[1], p[7], "容器圆角", line);
        assert_f32(m.container_radius[2], p[8], "容器圆角 lg", line);
        assert_f32(m.item_radius[0], p[9], "选项圆角 sm", line);
        assert_f32(m.item_radius[1], p[10], "选项圆角", line);
        assert_f32(m.item_radius[2], p[11], "选项圆角 lg", line);
        assert_f32(m.label_height[0], p[12], "选项高 sm", line);
        assert_f32(m.label_height[1], p[13], "选项高", line);
        assert_f32(m.label_height[2], p[14], "选项高 lg", line);
        assert_f32(m.padding_inline[0], p[15], "水平内边距 sm", line);
        assert_f32(m.padding_inline[1], p[16], "水平内边距", line);
        assert_f32(m.padding_inline[2], p[17], "水平内边距 lg", line);
        assert_f32(m.font_size[0], p[18], "字号 sm", line);
        assert_f32(m.font_size[1], p[19], "字号", line);
        assert_f32(m.font_size[2], p[20], "字号 lg", line);
        checked += 1;
    }
    assert_eq!(checked, 6, "segmented_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn segmented_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn segmented_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn segmented_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 轨道 padding 2；选项高 28/20/36；圆角 6/4 与 4/2；字号 14/14/16。
#[test]
fn segmented_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = metric::SegmentedMetric::resolve(&tokens);
    assert_eq!(m.track_padding, 2.0);
    assert_eq!(m.container_radius, [4.0, 6.0, 8.0]);
    assert_eq!(m.item_radius, [2.0, 4.0, 6.0]);
    assert_eq!(m.label_height, [20.0, 28.0, 36.0]);
    assert_eq!(m.padding_inline, [7.0, 11.0, 11.0]);
    assert_eq!(m.font_size, [14.0, 14.0, 16.0]);
}
