//! Switch 矩阵：small × on × disabled × loading。
//!
//! 基准由 `harness/antd_baseline/gen_switch_matrix.mjs` 依据 antdv-next Switch
//! 样式生成，关键值已用 `harness/_switch_probe.html` 的 computed style 核对。

use gpui::Hsla;
use gpui_widgets::switch::switch_state;
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

fn check(tokens: &Tokens, suffix: &str) {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}switch_matrix_{suffix}.txt")).unwrap();
    let mut checked = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split('|').collect();
        assert_eq!(p.len(), 19, "坏行: {line}");
        let style = switch_state(tokens, p[0] == "1", p[1] == "1", p[2] == "1", p[3] == "1");
        assert_close(style.bg, parse(p[4]), "底色", line);
        assert_close(style.hover_bg, parse(p[5]), "悬停底色", line);
        assert_close(style.handle_bg, parse(p[6]), "滑块色", line);
        assert_close(style.off_text, parse(p[7]), "关闭文字色", line);
        assert_close(style.on_text, parse(p[8]), "开启文字色", line);
        assert_close(style.accent, parse(p[9]), "主色", line);
        assert_f32(style.opacity, p[10], "透明度", line);
        assert_f32(style.track_width, p[11], "轨道宽", line);
        assert_f32(style.track_height, p[12], "轨道高", line);
        assert_f32(style.handle_size, p[13], "滑块尺寸", line);
        assert_f32(style.padding, p[14], "内边距", line);
        assert_f32(style.radius, p[15], "圆角", line);
        assert_f32(style.text_padding_near, p[16], "文字侧内边距", line);
        assert_f32(style.font_size, p[17], "字号", line);
        assert_f32(style.line_height, p[18], "行高", line);
        checked += 1;
    }
    assert_eq!(checked, 16, "switch_matrix_{suffix} 行数");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn switch_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn switch_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn switch_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

/// 轨道/滑块尺寸是 antd 组件常量，内文字号跟随 `fontSizeSM`；
/// 颜色、主色与透明度来自令牌。
#[test]
fn switch_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let off = switch_state(&tokens, false, false, false, false);
    assert_eq!(off.track_width, 44.0);
    assert_eq!(off.track_height, 22.0);
    assert_eq!(off.handle_size, 18.0);
    assert_eq!(off.font_size, tokens.font_size_sm);
    assert_eq!(off.bg, tokens.color_text_quaternary);
    assert_eq!(off.hover_bg, tokens.color_text_tertiary);
    assert_eq!(off.handle_bg, tokens.color_bg_container);

    let on = switch_state(&tokens, false, true, false, false);
    assert_eq!(on.bg, tokens.color_primary);
    assert_eq!(on.hover_bg, tokens.color_primary_hover);
    assert_eq!(on.on_text, tokens.color_text_light_solid);

    let small = switch_state(&tokens, true, false, false, false);
    assert_eq!(small.track_width, 28.0);
    assert_eq!(small.track_height, 16.0);
    assert_eq!(small.handle_size, 12.0);
    assert_eq!(small.text_padding_near, 6.0);

    let disabled = switch_state(&tokens, false, true, true, false);
    assert_eq!(disabled.opacity, 0.65);
    assert_eq!(disabled.hover_bg, disabled.bg);
    assert_ne!(disabled.bg, tokens.color_primary);
    assert_eq!(disabled.handle_bg, tokens.color_bg_container);
    let loading = switch_state(&tokens, false, false, false, true);
    assert_eq!(loading.opacity, 0.65);
    assert_eq!(loading.hover_bg, loading.bg);
}
