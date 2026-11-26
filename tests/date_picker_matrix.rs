//! DatePicker 矩阵：日历面板配色/几何 + 日期格状态 + 历法算法。
//!
//! 基准由 `harness/antd_baseline/gen_date_picker_matrix.mjs` 依据 antdv-next
//! `date-picker/style/token.js` + `panel.js` + `index.js` 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{date_cell_state, date_picker_metric, date_picker_style},
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
        std::fs::read_to_string(format!("{BASELINE_DIR}date_picker_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 26, "坏行: {line}");

    let style = date_picker_style(tokens);
    let m = date_picker_metric(tokens);
    assert_close(style.bg, parse(p[0]), "面板底", line);
    assert_close(style.split, parse(p[1]), "分隔线", line);
    assert_close(style.header_fg, parse(p[2]), "标题色", line);
    assert_close(style.icon, parse(p[3]), "导航图标", line);
    assert_close(style.icon_hover, parse(p[4]), "导航悬停", line);
    assert_close(style.view_hover, parse(p[5]), "标题悬停", line);
    assert_close(style.cell_fg, parse(p[6]), "本月字色", line);
    assert_close(style.cell_out_fg, parse(p[7]), "补位字色", line);
    assert_close(style.cell_hover_bg, parse(p[8]), "格悬停底", line);
    assert_close(style.selected_fg, parse(p[9]), "选中字色", line);
    assert_close(style.suffix, parse(p[10]), "后缀色", line);
    assert_f32(m.cell_width, p[11], "格宽", line);
    assert_f32(m.cell_height, p[12], "格高", line);
    assert_f32(m.cell_pad_v, p[13], "格纵padding", line);
    assert_f32(m.pad_h, p[14], "面板横padding", line);
    assert_f32(m.panel_width, p[15], "面板宽", line);
    assert_f32(m.header_height, p[16], "头高", line);
    assert_f32(m.th_height, p[17], "表头高", line);
    assert_f32(m.radius, p[18], "面板圆角", line);
    assert_f32(m.cell_radius, p[19], "格圆角", line);
    assert_f32(m.body_pad_v, p[20], "body纵padding", line);
    assert_f32(m.footer_pad_h, p[21], "footer横padding", line);
    assert_f32(m.footer_line, p[22], "footer行高", line);
    assert_f32(m.suffix_gap, p[23], "后缀间距", line);
    assert_f32(m.title_weight, p[24], "标题字重", line);
    assert_f32(m.font_size, p[25], "字号", line);

    let selected = date_cell_state(tokens, true, false, true);
    assert_close(selected.bg.unwrap(), parse(p[5]), "选中底", line);
    assert_close(selected.fg, parse(p[9]), "选中字", line);
    let today = date_cell_state(tokens, true, true, false);
    assert_close(today.border.unwrap(), parse(p[5]), "今日描边", line);
    let out = date_cell_state(tokens, false, false, false);
    assert_close(out.fg, parse(p[7]), "补位字", line);
    assert!(out.bg.is_none() && out.border.is_none());
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn date_picker_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn date_picker_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn date_picker_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn date_picker_metric_follows_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = date_picker_metric(&tokens);
    assert_f32(m.panel_width, "288", "面板宽", "follow");
    assert_f32(m.cell_width, "36", "格宽", "follow");
    assert_f32(m.cell_inner, "24", "内格", "follow");
    assert_f32(m.th_height, "36", "行高", "follow");
    assert_f32(m.footer_line, "38", "footer行高", "follow");
    assert_eq!(m.nav_icon, 10.0);
}

#[test]
fn date_picker_cell_priority() {
    let tokens = tokens_for(None, Appearance::Light);
    // 选中优先于今日描边
    let both = date_cell_state(&tokens, true, true, true);
    assert!(both.bg.is_some() && both.border.is_none());
    // 补位格即使"选中"也不上色（antd 只给 in-view 格选中态）
    let out = date_cell_state(&tokens, false, true, true);
    assert!(out.bg.is_none() && out.border.is_none());
}
