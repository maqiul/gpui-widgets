//! RangePicker 矩阵：双月面板配色/几何 + 范围格状态（端点/连续底/半侧底）。
//!
//! 基准由 `harness/antd_baseline/gen_range_picker_matrix.mjs` 依据 antdv-next
//! `date-picker/style/token.js` + `panel.js` + `index.js` 生成（与 DatePicker 同源）。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{
        RangeEnd, date_picker_metric, date_picker_style, range_cell_state, range_picker_metric,
    },
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
        std::fs::read_to_string(format!("{BASELINE_DIR}range_picker_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 25, "坏行: {line}");

    let style = date_picker_style(tokens);
    let m = date_picker_metric(tokens);
    let rm = range_picker_metric(tokens);
    assert_close(style.bg, parse(p[0]), "面板底", line);
    assert_close(style.split, parse(p[1]), "分隔线", line);
    assert_close(style.header_fg, parse(p[2]), "标题色", line);
    assert_close(style.icon, parse(p[3]), "导航图标", line);
    assert_close(style.icon_hover, parse(p[4]), "导航悬停", line);
    assert_close(style.cell_fg, parse(p[6]), "本月字色", line);
    assert_close(style.cell_out_fg, parse(p[7]), "补位字色", line);
    assert_close(style.cell_hover_bg, parse(p[8]), "格悬停底", line);
    assert_close(style.selected_fg, parse(p[9]), "端点字色", line);
    assert_close(style.suffix, parse(p[11]), "后缀色", line);
    assert_f32(m.cell_width, p[12], "格宽", line);
    assert_f32(m.cell_height, p[13], "格高", line);
    assert_f32(m.cell_pad_v, p[14], "格纵padding", line);
    assert_f32(m.pad_h, p[15], "面板横padding", line);
    assert_f32(m.panel_width, p[16], "单面板宽", line);
    assert_f32(m.header_height, p[17], "头高", line);
    assert_f32(m.th_height, p[18], "表头高", line);
    assert_f32(m.radius, p[19], "面板圆角", line);
    assert_f32(m.cell_radius, p[20], "格圆角", line);
    assert_f32(m.body_pad_v, p[21], "body纵padding", line);
    assert_f32(m.font_size, p[22], "字号", line);
    assert_f32(m.title_weight, p[23], "标题字重", line);
    assert_f32(rm.separator_size, p[24], "分隔符尺寸", line);
    assert_f32(rm.panel_gap, "0", "双面板间距", line);

    let start = range_cell_state(tokens, true, false, true, false, false);
    assert_close(start.inner_bg.unwrap(), parse(p[5]), "起点底", line);
    assert_close(start.fg, parse(p[9]), "起点字", line);
    assert_close(start.band.unwrap(), parse(p[10]), "起点连续底", line);
    assert_eq!(start.end, Some(RangeEnd::Start));
    assert_eq!(start.band_half, Some(RangeEnd::Start));
    let end = range_cell_state(tokens, true, false, false, true, false);
    assert_eq!(end.end, Some(RangeEnd::End));
    assert_eq!(end.band_half, Some(RangeEnd::End));
    let mid = range_cell_state(tokens, true, false, false, false, true);
    assert_close(mid.band.unwrap(), parse(p[10]), "区间连续底", line);
    assert!(mid.inner_bg.is_none() && mid.band_half.is_none());
    assert_close(mid.fg, parse(p[6]), "区间字色", line);
    let today = range_cell_state(tokens, true, true, false, false, false);
    assert_close(today.border.unwrap(), parse(p[5]), "今日描边", line);
    let out = range_cell_state(tokens, false, false, false, false, false);
    assert_close(out.fg, parse(p[7]), "补位字", line);
    assert!(out.inner_bg.is_none() && out.band.is_none() && out.border.is_none());
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn range_picker_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn range_picker_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn range_picker_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn range_picker_metric_follows_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = date_picker_metric(&tokens);
    let rm = range_picker_metric(&tokens);
    assert_f32(m.panel_width, "288", "单面板宽", "follow");
    assert_f32(m.cell_width, "36", "格宽", "follow");
    assert_f32(m.th_height, "36", "行高", "follow");
    assert_f32(rm.separator_size, "16", "分隔符尺寸", "follow");
}

#[test]
fn range_picker_cell_priority() {
    let tokens = tokens_for(None, Appearance::Light);
    // 起终点同格：无连续底、无半侧
    let same = range_cell_state(&tokens, true, false, true, true, false);
    assert!(same.band.is_none() && same.band_half.is_none());
    assert_eq!(same.end, Some(RangeEnd::Start));
    // 端点优先于今日描边
    let both = range_cell_state(&tokens, true, true, false, true, false);
    assert!(both.inner_bg.is_some() && both.border.is_none());
    // 补位格即使"端点/区间"也不上色
    let out = range_cell_state(&tokens, false, true, true, false, true);
    assert!(out.inner_bg.is_none() && out.band.is_none() && out.border.is_none());
}
