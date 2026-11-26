//! Progress 矩阵：状态 × 尺寸 × 是否显示指示器。
//!
//! 基准由 `harness/antd_baseline/gen_progress_matrix.mjs` 依据 antdv-next Progress
//! 样式生成，关键值已用 `harness/_progress_probe.html` 的 computed style 核对。

use gpui::Hsla;
use gpui_widgets::progress::{ProgressStatus, progress_state};
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
    if text == "transparent" {
        return Rgba {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.,
        };
    }
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

fn status_of(code: &str) -> ProgressStatus {
    match code {
        "0" => ProgressStatus::Normal,
        "1" => ProgressStatus::Active,
        "2" => ProgressStatus::Success,
        "3" => ProgressStatus::Exception,
        other => panic!("坏状态 {other}"),
    }
}

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}progress_matrix_{suffix}.txt")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 14, "坏行: {line}");
        let style = progress_state(tokens, status_of(p[0]), p[1] == "1", p[2] == "1");
        assert_close(style.track, parse(p[3]), "轨道色", line);
        assert_close(style.rail, parse(p[4]), "底轨色", line);
        assert_close(style.text, parse(p[5]), "指示器色", line);
        assert_close(style.glyph, parse(p[6]), "图标勾色", line);
        assert_f32(style.stroke, p[7], "轨高", line);
        assert_f32(style.radius, p[8], "圆角", line);
        assert_f32(style.gap, p[9], "间距", line);
        assert_f32(style.font_size, p[10], "字号", line);
        assert_f32(style.line_height, p[11], "行高", line);
        assert_f32(style.steps_item_width, p[12], "分段宽", line);
        assert_f32(style.steps_gap, p[13], "分段间距", line);
        checked += 1;
    }
    assert_eq!(checked, 8, "progress_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn progress_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn progress_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn progress_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 轨道 8 / small 6、圆角 100、间距 8；成功/异常指示器换同色图标。
#[test]
fn progress_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let normal = progress_state(&tokens, ProgressStatus::Normal, false, true);
    assert_eq!(normal.track, tokens.color_info);
    assert_eq!(normal.rail, tokens.color_fill_secondary);
    assert_eq!(normal.text, tokens.color_text);
    assert_eq!(normal.stroke, 8.0);
    assert_eq!(normal.radius, 100.0);
    assert_eq!(normal.gap, 8.0);
    assert_eq!(normal.font_size, tokens.font_size);
    assert_eq!(normal.line_height, tokens.font_size);
    assert_eq!(normal.steps_item_width, 14.0);
    assert_eq!(normal.steps_gap, 2.0);

    let small = progress_state(&tokens, ProgressStatus::Normal, true, true);
    assert_eq!(small.stroke, 6.0);
    assert_eq!(small.font_size, tokens.font_size_sm);
    assert_eq!(small.steps_item_width, 2.0);

    let noinfo = progress_state(&tokens, ProgressStatus::Normal, false, false);
    assert_eq!(noinfo.line_height, 8.0);

    let success = progress_state(&tokens, ProgressStatus::Success, false, true);
    assert_eq!(success.track, tokens.color_success);
    assert_eq!(success.text, tokens.color_success);
    assert_eq!(success.glyph, tokens.color_text_light_solid);

    let exception = progress_state(&tokens, ProgressStatus::Exception, false, true);
    assert_eq!(exception.track, tokens.color_error);
    assert_eq!(exception.text, tokens.color_error);

    let active = progress_state(&tokens, ProgressStatus::Active, false, true);
    assert_eq!(active.track, tokens.color_info);
}
