//! Slider 矩阵：常态 / 悬停 / 按下 / 禁用。
//!
//! 基准由 `harness/antd_baseline/gen_slider_matrix.mjs` 依据 antdv-next
//! `slider/style/index.js` 与组件 token 生成。

use gpui::Hsla;
use gpui_widgets::slider::{SliderColors, SliderStateKind, slider_state};
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

/// 把前景叠到底色上（antd `onBackground`），用于对比 `colorPrimaryOutline`
/// 这种以最小 alpha 表示的派生色。
fn over(front: Hsla, back: Hsla) -> Rgba {
    let f = rgba(front);
    let b = rgba(back);
    Rgba {
        r: f.r * f.a + b.r * (1. - f.a),
        g: f.g * f.a + b.g * (1. - f.a),
        b: f.b * f.a + b.b * (1. - f.a),
        a: 1.,
    }
}

fn assert_solid(actual: Rgba, expected: Rgba, what: &str, row: &str) {
    let near = (actual.r - expected.r).abs() <= 1.0
        && (actual.g - expected.g).abs() <= 1.0
        && (actual.b - expected.b).abs() <= 1.0;
    assert!(
        near,
        "{row} {what}: got rgb({},{},{}), want rgb({},{},{})",
        actual.r, actual.g, actual.b, expected.r, expected.g, expected.b
    );
}

fn state(text: &str) -> SliderStateKind {
    match text {
        "normal" => SliderStateKind::Normal,
        "hover" => SliderStateKind::Hover,
        "active" => SliderStateKind::Active,
        "disabled" => SliderStateKind::Disabled,
        other => panic!("坏状态 {other}"),
    }
}

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}slider_matrix_{suffix}.txt")).unwrap();
    let m = metric::SliderMetric::resolve(tokens);
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 23, "坏行: {line}");
        let style = slider_state(tokens, state(p[0]));
        assert_close(style.rail, parse(p[1]), "轨道", line);
        assert_close(style.track, parse(p[2]), "已选轨道", line);
        assert_close(style.handle, parse(p[3]), "滑块底", line);
        assert_close(style.handle_ring, parse(p[4]), "滑块环", line);
        assert_solid(
            over(style.handle_outline, tokens.color_bg_container),
            parse(p[5]),
            "滑块外圈",
            line,
        );
        assert_close(style.dot_border, parse(p[6]), "刻度点边框", line);
        assert_close(style.dot_active, parse(p[7]), "已过刻度点边框", line);
        assert_f32(m.control_size, p[8], "容器高", line);
        assert_f32(m.rail_size, p[9], "轨道高", line);
        assert_f32(m.handle_size, p[10], "滑块", line);
        assert_f32(m.handle_size_hover, p[11], "悬停滑块", line);
        assert_f32(m.dot_size, p[12], "刻度点", line);
        assert_f32(m.ring_width, p[13], "滑块环", line);
        assert_f32(m.ring_width_hover, p[14], "悬停滑块环", line);
        assert_f32(m.outline_width, p[15], "外圈", line);
        assert_f32(m.handle_pos, p[16], "滑块偏移", line);
        assert_f32(m.dot_offset, p[17], "刻度点偏移", line);
        assert_f32(m.mark_top, p[18], "刻度顶距", line);
        assert_f32(m.mark_font, p[19], "刻度字号", line);
        assert_f32(m.margin_part, p[20], "块向外边距", line);
        assert_f32(m.margin_full, p[21], "行向外边距", line);
        assert_f32(m.margin_part_with_mark, p[22], "带刻度下边距", line);
        checked += 1;
    }
    assert_eq!(checked, 4, "slider_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn slider_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn slider_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn slider_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 容器高 10、轨道 4、滑块 10/12、环 2/2.5、点 8、外边距 11/5、刻度顶距 12。
#[test]
fn slider_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = metric::SliderMetric::resolve(&tokens);
    assert_eq!(m.control_size, 10.0);
    assert_eq!(m.rail_size, 4.0);
    assert_eq!(m.handle_size, 10.0);
    assert_eq!(m.handle_size_hover, 12.0);
    assert_eq!(m.dot_size, 8.0);
    assert_eq!(m.ring_width, 2.0);
    assert_eq!(m.ring_width_hover, 2.5);
    assert_eq!(m.outline_width, 6.0);
    assert_eq!(m.handle_pos, 1.0);
    assert_eq!(m.dot_offset, -2.0);
    assert_eq!(m.mark_top, 12.0);
    assert_eq!(m.mark_font, 14.0);
    assert_eq!(m.margin_part, 11.0);
    assert_eq!(m.margin_full, 5.0);
    assert_eq!(m.margin_part_with_mark, 30.0);
}

/// 逐实例覆写：空覆写原样返回，单项覆写只改该字段，`handle_outline` 与刻度点永远跟令牌。
#[test]
fn slider_colors_override_matrix() {
    let tokens = tokens_for(None, Appearance::Light);
    let normal = slider_state(&tokens, SliderStateKind::Normal);
    let hover = slider_state(&tokens, SliderStateKind::Hover);
    let disabled = slider_state(&tokens, SliderStateKind::Disabled);

    assert_eq!(
        SliderColors::new().apply(normal),
        normal,
        "空覆写等于令牌派生值"
    );
    assert_eq!(
        SliderColors::default().apply(hover),
        hover,
        "default 同 new"
    );
    assert_eq!(SliderColors::default().apply(disabled), disabled);

    // 取四个互不相同的令牌色当覆写值，避免与 base 的任一字段撞色
    let rail: Hsla = tokens.color_error;
    let track: Hsla = tokens.color_success;
    let handle: Hsla = tokens.color_warning;
    let ring: Hsla = tokens.color_info;
    assert_ne!(rail, normal.rail, "覆写值需与令牌值不同");
    assert_ne!(track, normal.track);
    assert_ne!(handle, normal.handle);
    assert_ne!(ring, normal.handle_ring);

    let only_rail = SliderColors::new().rail(rail).apply(normal);
    assert_eq!(only_rail.rail, rail);
    assert_eq!(only_rail.track, normal.track, "未覆写字段保持");
    assert_eq!(only_rail.handle, normal.handle);
    assert_eq!(only_rail.handle_ring, normal.handle_ring);

    let only_track = SliderColors::new().track(track).apply(normal);
    assert_eq!(only_track.track, track);
    assert_eq!(only_track.rail, normal.rail);

    let full = SliderColors::new()
        .rail(rail)
        .track(track)
        .handle(handle)
        .handle_ring(ring)
        .apply(hover);
    assert_eq!(full.rail, rail);
    assert_eq!(full.track, track);
    assert_eq!(full.handle, handle);
    assert_eq!(full.handle_ring, ring);
    // 不开放的三项在任意状态下都不受覆写影响
    assert_eq!(full.handle_outline, hover.handle_outline);
    assert_eq!(full.dot_border, hover.dot_border);
    assert_eq!(full.dot_active, hover.dot_active);

    // 覆写与交互态正交：同一份覆写套到禁用态，仍只改那四项
    let disabled_over = SliderColors::new()
        .track(track)
        .handle_ring(ring)
        .apply(disabled);
    assert_eq!(disabled_over.track, track);
    assert_eq!(disabled_over.handle_ring, ring);
    assert_eq!(disabled_over.rail, disabled.rail, "禁用态底槽保持令牌值");
    assert_eq!(disabled_over.handle, disabled.handle);
}
