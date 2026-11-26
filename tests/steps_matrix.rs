//! Steps 矩阵：filled / outlined × 四种状态 × 当前/非当前 × 点状。
//!
//! 基准由 `harness/antd_baseline/gen_steps_matrix.mjs` 依据 antdv-next
//! `steps/style/status.js` 与组件 token 生成。

use gpui::Hsla;
use gpui_widgets::steps::{StepStatus, StepsVariant, step_state};
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

fn status(text: &str) -> StepStatus {
    match text {
        "wait" => StepStatus::Wait,
        "process" => StepStatus::Process,
        "finish" => StepStatus::Finish,
        "error" => StepStatus::Error,
        other => panic!("坏状态 {other}"),
    }
}

fn check(tokens: &Tokens, suffix: &str) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}steps_matrix_{suffix}.txt")).unwrap();
    let m = metric::StepsMetric::resolve(tokens);
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 19, "坏行: {line}");
        let variant = match p[0] {
            "filled" => StepsVariant::Filled,
            "outlined" => StepsVariant::Outlined,
            other => panic!("坏变体 {other}"),
        };
        let state = step_state(tokens, variant, status(p[1]), p[2] == "1", p[3] == "1");
        assert_close(state.node.bg, parse(p[4]), "节点底", line);
        assert_close(state.node.border, parse(p[5]), "节点描边", line);
        assert_close(state.node.text, parse(p[6]), "节点文字", line);
        assert_close(state.title, parse(p[7]), "标题色", line);
        assert_close(state.content, parse(p[8]), "内容色", line);
        assert_close(state.rail, parse(p[9]), "连接线色", line);
        assert_f32(m.icon_size, p[10], "节点尺寸", line);
        assert_f32(m.icon_font_size, p[11], "节点字号", line);
        assert_f32(m.title_font_size, p[12], "标题字号", line);
        assert_f32(m.title_line_height, p[13], "标题行高", line);
        assert_f32(m.content_font_size, p[14], "内容字号", line);
        assert_f32(m.content_line_height, p[15], "内容行高", line);
        assert_f32(m.rail_width, p[16], "连接线宽", line);
        assert_f32(m.dot_size, p[17], "点尺寸", line);
        assert_f32(m.dot_active_size, p[18], "当前点尺寸", line);
        checked += 1;
    }
    assert_eq!(checked, 24, "steps_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn steps_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn steps_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn steps_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 节点 32 / 标题 16/24 / 连接线 1；small 24 / 14/22 / 间距 12。
#[test]
fn steps_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = metric::StepsMetric::resolve(&tokens);
    assert_eq!(m.icon_size, tokens.control_height);
    assert_eq!(m.icon_size_small, 24.0);
    assert_eq!(m.title_font_size, tokens.font_size_lg);
    assert_eq!(m.title_line_height, tokens.font_height_lg);
    assert_eq!(
        m.heading_height,
        tokens.control_height.max(tokens.font_height_lg)
    );
    assert_eq!(m.item_gap, 16.0);
    assert_eq!(m.rail_gap, 16.0);
    assert_eq!(m.icon_gap, 8.0);
    assert_eq!(m.icon_gap_vertical, 16.0);
    assert_eq!(m.vertical_padding, 12.0);
    assert_eq!(m.rail_width, tokens.line_width);
    assert_eq!(m.dot_size, tokens.control_height / 4.0);
    assert_eq!(m.dot_active_size, tokens.control_height_lg / 4.0);
}
