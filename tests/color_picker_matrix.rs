//! ColorPicker 矩阵：触发框/色块/预设取色与三档几何，isBright 与选中勾折线。
//!
//! 基准由 `harness/antd_baseline/gen_color_picker_matrix.mjs` 依据 antdv-next
//! `color-picker/style/index.js` + `color-block.js` + `presets.js` 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{color_check_points, color_picker_metric, color_picker_style, color_preset_bright},
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
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}color_picker_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 34, "坏行: {line}");

    let style = color_picker_style(tokens);
    let m = color_picker_metric(tokens);
    assert_close(style.trigger_bg, parse(p[0]), "触发框底", line);
    assert_close(style.border, parse(p[1]), "触发框描边", line);
    assert_close(style.hover, parse(p[2]), "悬停描边", line);
    assert_close(style.active, parse(p[3]), "展开描边", line);
    assert_close(style.outline, parse(p[4]), "展开外圈", line);
    assert_close(style.disabled_text, parse(p[5]), "禁用文字", line);
    assert_close(style.disabled_bg, parse(p[6]), "禁用底", line);
    assert_close(style.inner, parse(p[7]), "色块内描边", line);
    assert_close(style.inset, parse(p[8]), "色块 inset", line);
    assert_close(style.check, parse(p[9]), "选中勾", line);
    assert_close(style.check_bright, parse(p[10]), "亮色选中勾", line);
    assert_close(style.text, parse(p[11]), "文字色", line);
    assert_close(style.preset_hover, parse(p[12]), "预设悬停环", line);
    assert_f32(m.trigger_size, p[13], "触发框尺寸", line);
    assert_f32(m.trigger_size_sm, p[14], "小触发框", line);
    assert_f32(m.trigger_size_lg, p[15], "大触发框", line);
    assert_f32(m.trigger_radius, p[16], "触发框圆角", line);
    assert_f32(m.trigger_radius_sm, p[17], "小触发框圆角", line);
    assert_f32(m.trigger_radius_lg, p[18], "大触发框圆角", line);
    assert_f32(m.trigger_padding, p[19], "触发框内边距", line);
    assert_f32(m.block_size, p[20], "色块尺寸", line);
    assert_f32(m.block_size_sm, p[21], "小色块", line);
    assert_f32(m.block_size_lg, p[22], "大色块", line);
    assert_f32(m.block_radius, p[23], "色块圆角", line);
    assert_f32(m.block_radius_sm, p[24], "小色块圆角", line);
    assert_f32(m.block_radius_lg, p[25], "大色块圆角", line);
    assert_f32(m.preset_size, p[26], "预设尺寸", line);
    assert_f32(m.preset_gap, p[27], "预设间距", line);
    assert_f32(m.check_stroke, p[28], "勾线宽", line);
    assert_f32(m.panel_width, p[29], "面板宽", line);
    assert_f32(m.panel_padding, p[30], "面板内边距", line);
    assert_f32(m.text_size, p[31], "文字字号", line);
    assert_f32(m.text_gap, p[32], "文字间距", line);
    assert_f32(style.outline_width, p[33], "外圈宽", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn color_picker_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn color_picker_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn color_picker_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn color_picker_metrics_follow_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = color_picker_metric(&tokens);
    assert_eq!(m.trigger_size, 32.0);
    assert_eq!(m.trigger_size_sm, 24.0);
    assert_eq!(m.trigger_size_lg, 40.0);
    assert_eq!(m.trigger_padding, 3.0);
    assert_eq!(m.block_size, 24.0);
    assert_eq!(m.block_size_sm, 16.0);
    assert_eq!(m.block_size_lg, 32.0);
    assert_eq!(m.preset_size, 24.0);
    assert_eq!(m.preset_gap, 6.0);
    assert_eq!(m.check_stroke, 2.0);
    assert_eq!(m.panel_width, 234.0);
}

#[test]
fn color_preset_bright_luma_and_alpha() {
    let bg = gpui::white();
    assert!(color_preset_bright(255., 255., 255., 1., bg));
    assert!(!color_preset_bright(239., 68., 68., 1., bg));
    // 亮度阈值 192：0xc0c0c0 = 192 不亮，0xc1c1c1 亮
    assert!(!color_preset_bright(192., 192., 192., 1., bg));
    assert!(color_preset_bright(193., 193., 193., 1., bg));
    // alpha<=.5 叠底后看 v：半透明白叠白底 v=1 亮；半透明黑叠白底 v=.5 不亮
    assert!(color_preset_bright(255., 255., 255., 0.5, bg));
    assert!(!color_preset_bright(0., 0., 0., 0.5, bg));
}

#[test]
fn color_check_points_inside_preset_box() {
    let pts = color_check_points(24.);
    let w = 24. / 13. * 5.;
    let h = 24. / 13. * 8.;
    let s = 2f32.sqrt() / 2.;
    let want = [
        (24. * 0.215 + w / 2. + h * s, 24. * 0.5 + h / 2. - h * s),
        (24. * 0.215 + w / 2., 24. * 0.5 + h / 2.),
        (24. * 0.215 + w / 2. - w * s, 24. * 0.5 + h / 2. - w * s),
    ];
    for ((x, y), (wx, wy)) in pts.iter().zip(want.iter()) {
        assert!(
            (x - wx).abs() < 1e-3 && (y - wy).abs() < 1e-3,
            "got ({x},{y}) want ({wx},{wy})"
        );
        assert!(
            (0. ..=24.).contains(x) && (0. ..=24.).contains(y),
            "勾越界 ({x},{y})"
        );
    }
}
