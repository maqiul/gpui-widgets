//! Spin 矩阵：点色/三档 holder 与 item 尺寸/缩放/基透明度/文案几何，
//! 以及 antRotate / antSpinMove 帧函数与四点旋转几何。
//!
//! 基准由 `harness/antd_baseline/gen_spin_matrix.mjs` 依据 antdv-next
//! `spin/style/index.js` 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{spin_dot_centers, spin_frame, spin_metric, spin_style},
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
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}spin_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 12, "坏行: {line}");

    let style = spin_style(tokens);
    let m = spin_metric(tokens);
    assert_close(style.dot, parse(p[0]), "点色", line);
    assert_f32(m.dot_size, p[1], "holder 尺寸", line);
    assert_f32(m.dot_size_sm, p[2], "小 holder", line);
    assert_f32(m.dot_size_lg, p[3], "大 holder", line);
    assert_f32(m.item_size, p[4], "item 尺寸", line);
    assert_f32(m.item_size_sm, p[5], "小 item", line);
    assert_f32(m.item_size_lg, p[6], "大 item", line);
    assert_f32(m.dot_scale, p[7], "点缩放", line);
    assert_f32(m.base_opacity, p[8], "基透明度", line);
    assert_f32(m.gap, p[9], "文案间距", line);
    assert_f32(m.desc_size, p[10], "文案字号", line);
    assert_f32(m.desc_line_height, p[11], "文案行高", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn spin_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn spin_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn spin_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn spin_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = spin_metric(&tokens);
    assert_eq!(m.dot_size, 20.0);
    assert_eq!(m.dot_size_sm, 14.0);
    assert_eq!(m.dot_size_lg, 32.0);
    assert_eq!(m.item_size, 9.0);
    assert_eq!(m.item_size_sm, 6.0);
    assert_eq!(m.item_size_lg, 15.0);
    assert_eq!(m.gap, 12.0);
    assert_eq!(m.desc_size, 14.0);
    assert_eq!(m.desc_line_height, 14.0);
}

#[test]
fn spin_frame_first_frame() {
    let (angle, op) = spin_frame(0.);
    assert!((angle - 45.).abs() < 1e-4);
    assert_eq!(op, [0.3, 0.3, 0.3, 0.3]);
}

#[test]
fn spin_frame_mid_rotation() {
    let (angle, op) = spin_frame(0.6);
    assert!((angle - 225.).abs() < 1e-4);
    assert!((op[0] - 0.72).abs() < 1e-4);
    assert!((op[1] - 0.44).abs() < 1e-4);
    assert_eq!(op[2], 0.3);
    assert_eq!(op[3], 0.3);
}

#[test]
fn spin_frame_full_rotation() {
    let (angle, op) = spin_frame(1.2);
    assert!((angle - 45.).abs() < 1e-4);
    assert!((op[0] - 0.86).abs() < 1e-4);
    assert!((op[1] - 0.86).abs() < 1e-4);
    assert!((op[2] - 0.58).abs() < 1e-4);
    assert_eq!(op[3], 0.3);
}

#[test]
fn spin_dot_centers_rotate_to_edge_midpoints() {
    let centers = spin_dot_centers(20., 9., 45.);
    let edge = 5.5 * 2f32.sqrt();
    let want = [
        (10., 10. - edge),
        (10. + edge, 10.),
        (10., 10. + edge),
        (10. - edge, 10.),
    ];
    for ((x, y), (wx, wy)) in centers.iter().zip(want.iter()) {
        assert!(
            (x - wx).abs() < 1e-3 && (y - wy).abs() < 1e-3,
            "got ({x},{y}) want ({wx},{wy})"
        );
    }
}
