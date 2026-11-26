//! Calendar 矩阵：全尺寸月/年视图配色/几何 + date 格状态（选中/今日/补位）。
//!
//! 基准由 `harness/antd_baseline/gen_calendar_matrix.mjs` 依据 antdv-next
//! `calendar/style/index.js` 生成；头部下拉几何跟随 SelectMetric。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{SelectMetric, calendar_cell_state, calendar_metric, calendar_style},
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
        std::fs::read_to_string(format!("{BASELINE_DIR}calendar_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 33, "坏行: {line}");

    let cs = calendar_style(tokens);
    let m = calendar_metric(tokens);
    assert_close(cs.bg, parse(p[0]), "日历底", line);
    assert_close(cs.split, parse(p[1]), "顶描边", line);
    assert_close(cs.fg, parse(p[2]), "值字色", line);
    assert_close(cs.out_fg, parse(p[3]), "补位字色", line);
    assert_close(cs.hover_bg, parse(p[4]), "格悬停底", line);
    assert_close(cs.selected_bg, parse(p[5]), "选中底", line);
    assert_close(cs.selected_fg, parse(p[6]), "选中字色", line);
    assert_close(cs.today_border, parse(p[6]), "今日顶描边", line);
    assert_close(cs.mode_checked_hover_fg, parse(p[7]), "选中模式悬停", line);
    assert_close(cs.mode_border, parse(p[8]), "模式描边", line);
    assert_close(cs.mode_fg, parse(p[2]), "模式字色", line);
    assert_close(cs.mode_bg, parse(p[0]), "模式底", line);
    assert_close(cs.mode_checked_fg, parse(p[6]), "选中模式字", line);
    assert_close(cs.mode_hover_fg, parse(p[6]), "模式悬停字", line);
    assert_close(cs.suffix, parse(p[9]), "后缀色", line);
    assert_close(cs.option_selected_bg, parse(p[5]), "选项选中底", line);
    assert_close(cs.option_hover_bg, parse(p[4]), "选项悬停底", line);
    assert_close(cs.option_fg, parse(p[2]), "选项字色", line);
    assert_close(cs.menu_bg, parse(p[10]), "菜单底", line);
    assert_f32(m.header_pad_v, p[11], "头纵padding", line);
    assert_f32(m.year_width, p[12], "年选择宽", line);
    assert_f32(m.month_width, p[13], "月选择宽", line);
    assert_f32(m.control_gap, p[14], "控件间距", line);
    assert_f32(m.week_height, p[15], "周行高", line);
    assert_f32(m.week_pad_bottom, p[16], "周行底padding", line);
    assert_f32(m.week_pad_end, p[11], "周行尾padding", line);
    assert_f32(m.body_pad_v, p[17], "body纵padding", line);
    assert_f32(m.date_margin_h, p[18], "date横margin", line);
    assert_f32(m.date_pad_top, p[19], "date顶padding", line);
    assert_f32(m.date_pad_h, p[17], "date横padding", line);
    assert_f32(m.date_value_height, p[20], "值行高", line);
    assert_f32(m.date_content_height, p[21], "内容高", line);
    assert_f32(m.date_border, p[22], "顶描边宽", line);
    assert_f32(m.cell_height, p[23], "格高", line);
    assert_f32(m.font_size, p[24], "字号", line);
    assert_f32(m.content_line_height, p[25], "内容行高", line);
    assert_f32(m.mode_height, p[26], "模式高", line);
    assert_f32(m.mode_pad_inline, p[27], "模式横padding", line);
    assert_f32(m.mode_radius, p[28], "模式圆角", line);
    assert_f32(m.mode_line_height, p[29], "模式行高", line);
    assert_f32(m.menu_max_height, p[30], "菜单限高", line);
    assert_f32(m.icon_size, p[31], "后缀图标", line);
    assert_f32(cs.option_weight, p[32], "选项选中字重", line);

    let selected = calendar_cell_state(tokens, true, false, true);
    assert_close(selected.value_fg, parse(p[6]), "选中值", line);
    assert_close(selected.bg.unwrap(), parse(p[5]), "选中格底", line);
    assert_close(selected.border_top, parse(p[1]), "选中非今日描边", line);
    let today = calendar_cell_state(tokens, true, true, false);
    assert_close(today.border_top, parse(p[6]), "今日描边", line);
    assert_close(today.value_fg, parse(p[2]), "今日值", line);
    assert!(today.bg.is_none());
    let plain = calendar_cell_state(tokens, true, false, false);
    assert_close(plain.value_fg, parse(p[2]), "普通值", line);
    assert_close(plain.border_top, parse(p[1]), "普通描边", line);
    assert!(plain.bg.is_none());
    let out = calendar_cell_state(tokens, false, false, false);
    assert_close(out.value_fg, parse(p[3]), "补位值", line);
    assert!(out.bg.is_none());
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn calendar_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn calendar_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn calendar_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn calendar_metric_follows_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = calendar_metric(&tokens);
    assert_f32(m.week_height, "18", "周行高", "follow");
    assert_f32(m.date_value_height, "24", "值行高", "follow");
    assert_f32(m.date_content_height, "86", "内容高", "follow");
    assert_f32(m.cell_height, "116", "格高", "follow");
    assert_f32(m.mode_pad_inline, "15", "模式横padding", "follow");
    assert_f32(m.mode_line_height, "30", "模式行高", "follow");
    assert_f32(m.mode_radius, "6", "模式圆角", "follow");
    assert_eq!(m.year_offset, 10);
    assert_eq!(m.year_total, 20);
    let sm = SelectMetric::resolve(&tokens);
    assert_f32(
        m.select_height,
        &sm.height[1].to_string(),
        "下拉高",
        "follow",
    );
    assert_f32(
        m.select_pad_inline,
        &sm.padding_inline[1].to_string(),
        "下拉横padding",
        "follow",
    );
    assert_f32(
        m.select_radius,
        &sm.radius[1].to_string(),
        "下拉圆角",
        "follow",
    );
    assert_f32(
        m.option_height,
        &sm.option_height.to_string(),
        "选项高",
        "follow",
    );
    assert_f32(
        m.option_pad_inline,
        &sm.option_padding_inline.to_string(),
        "选项横padding",
        "follow",
    );
    assert_f32(
        m.menu_padding,
        &sm.menu_padding.to_string(),
        "菜单padding",
        "follow",
    );
    assert_f32(
        m.menu_radius,
        &sm.menu_radius.to_string(),
        "菜单圆角",
        "follow",
    );
}

#[test]
fn calendar_cell_priority() {
    let tokens = tokens_for(None, Appearance::Light);
    // 补位格即使今日+选中也不上色
    let out = calendar_cell_state(&tokens, false, true, true);
    assert!(out.bg.is_none());
    assert_close(
        out.value_fg,
        rgba(tokens.color_text_disabled),
        "补位值",
        "prio",
    );
    assert_close(out.border_top, rgba(tokens.color_split), "补位描边", "prio");
    // 今日+选中：主色顶描边与选中底并存
    let both = calendar_cell_state(&tokens, true, true, true);
    assert_close(
        both.border_top,
        rgba(tokens.color_primary),
        "今日描边",
        "prio",
    );
    assert_close(
        both.bg.unwrap(),
        rgba(tokens.control_item_bg_active),
        "选中底",
        "prio",
    );
    assert_close(both.value_fg, rgba(tokens.color_primary), "选中值", "prio");
}
