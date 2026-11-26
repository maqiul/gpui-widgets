//! Notification 矩阵：卡片配色/几何 + 关闭钮三态。
//!
//! 基准由 `harness/antd_baseline/gen_notice_matrix.mjs` 依据 antdv-next
//! `notification/style/index.js` + `notification/style/notification.js` 生成。

use gpui::Hsla;
use gpui_widgets::theme::{
    Appearance, Color, Seed, Tokens, derive,
    metric::{notice_close_state, notice_style, notification_metric},
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
        std::fs::read_to_string(format!("{BASELINE_DIR}notification_matrix_{suffix}.txt")).unwrap();
    let line = text.lines().next().unwrap();
    let p: Vec<&str> = line.split('|').collect();
    assert_eq!(p.len(), 25, "坏行: {line}");

    let cs = notice_style(tokens);
    let m = notification_metric(tokens);
    assert_close(cs.bg, parse(p[0]), "卡片底", line);
    assert_close(cs.icon_info, parse(p[1]), "info图标", line);
    assert_close(cs.icon_success, parse(p[2]), "success图标", line);
    assert_close(cs.icon_warning, parse(p[3]), "warning图标", line);
    assert_close(cs.icon_error, parse(p[4]), "error图标", line);
    assert_close(cs.title_fg, parse(p[5]), "标题色", line);
    assert_close(cs.text_fg, parse(p[6]), "描述色", line);
    assert_close(cs.close_fg, parse(p[7]), "关闭钮色", line);
    assert_close(cs.close_fg_hover, parse(p[8]), "关闭钮hover色", line);
    assert_close(cs.close_bg_hover, parse(p[9]), "关闭钮hover底", line);
    assert_close(cs.close_bg_active, parse(p[10]), "关闭钮active底", line);
    assert_f32(m.padding_vertical, p[11], "垂直内边距", line);
    assert_f32(m.padding_horizontal, p[12], "水平内边距", line);
    assert_f32(m.radius, p[13], "圆角", line);
    assert_f32(m.font, p[14], "正文字号", line);
    assert_f32(m.line, p[15], "正文行高", line);
    assert_f32(m.title_font, p[16], "标题字号", line);
    assert_f32(m.title_line, p[17], "标题行高", line);
    assert_f32(m.icon_gap, p[18], "图标右距", line);
    assert_f32(m.section_gap, p[19], "section间距", line);
    assert_f32(m.close_size, p[20], "关闭钮尺寸", line);
    assert_f32(m.close_radius, p[21], "关闭钮圆角", line);
    assert_f32(m.stack_gap, p[22], "堆栈间距", line);
    assert_f32(m.edge, p[23], "堆栈边距", line);
    assert_f32(m.width, p[24], "卡片宽", line);
    assert_f32(m.icon_size, p[17], "图标尺寸", line);
    assert_f32(m.desc_font, p[14], "描述字号", line);
    assert_f32(m.close_top, p[11], "关闭钮顶距", line);
    assert_f32(m.close_right, p[12], "关闭钮右距", line);
    assert_f32(m.closable_pad_end, p[12], "closable右留", line);
    assert_f32(m.actions_margin_top, p[18], "actions顶距", line);
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn notification_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn notification_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn notification_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn notification_close_state_matrix() {
    for appearance in [Appearance::Light, Appearance::Dark] {
        let tokens = tokens_for(None, appearance);
        let text = std::fs::read_to_string(format!(
            "{BASELINE_DIR}notification_matrix_{}.txt",
            if appearance == Appearance::Light {
                "light"
            } else {
                "dark"
            }
        ))
        .unwrap();
        let line = text.lines().next().unwrap();
        let p: Vec<&str> = line.split('|').collect();

        let idle = notice_close_state(&tokens, false, false);
        assert_close(idle.fg, parse(p[7]), "idle前景", line);
        assert!(idle.bg.is_none(), "idle 不应有底色");

        let hover = notice_close_state(&tokens, true, false);
        assert_close(hover.fg, parse(p[8]), "hover前景", line);
        assert_close(hover.bg.unwrap(), parse(p[9]), "hover底", line);

        let pressed = notice_close_state(&tokens, true, true);
        assert_close(pressed.fg, parse(p[8]), "active前景", line);
        assert_close(pressed.bg.unwrap(), parse(p[10]), "active底", line);

        let released = notice_close_state(&tokens, false, true);
        assert_close(released.fg, parse(p[7]), "未hover按下回落idle", line);
        assert!(released.bg.is_none());
    }
}

#[test]
fn notification_metric_follows_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let m = notification_metric(&tokens);
    assert_f32(m.width, "384", "卡片宽", "follow");
    assert_f32(m.padding_vertical, "20", "垂直内边距", "follow");
    assert_f32(m.padding_horizontal, "24", "水平内边距", "follow");
    assert_f32(m.icon_size, "24", "图标尺寸", "follow");
    assert_f32(m.icon_gap, "12", "图标右距", "follow");
    assert_f32(m.section_gap, "8", "section间距", "follow");
    assert_f32(m.title_font, "16", "标题字号", "follow");
    assert_f32(m.title_line, "24", "标题行高", "follow");
    assert_f32(m.close_size, "22", "关闭钮尺寸", "follow");
    assert_f32(m.close_radius, "4", "关闭钮圆角", "follow");
    assert_f32(m.stack_gap, "16", "堆栈间距", "follow");
    assert_f32(m.edge, "24", "堆栈边距", "follow");
}
