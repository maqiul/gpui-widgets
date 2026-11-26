//! Countdown 矩阵：Statistic 度量 + formatTimeStr 纯函数语义。
//!
//! 基准由 `harness/antd_baseline/gen_navmenu_countdown_matrix.mjs` 依据 antdv-next
//! `statistic/style/index.js` prepareComponentToken 生成；
//! 格式化语义对照 `statistic/utils.js` formatTimeStr。
use gpui::Hsla;
use gpui_widgets::countdown::format_counter;
use gpui_widgets::theme::metric::countdown_metric;
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

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

fn baseline(name: &str, suffix: &str) -> Vec<String> {
    let text =
        std::fs::read_to_string(format!("{BASELINE_DIR}{name}_matrix_{suffix}.txt")).unwrap();
    text.lines()
        .next()
        .unwrap()
        .split('|')
        .map(str::to_string)
        .collect()
}

fn check(tokens: &Tokens, suffix: &str) {
    let p = baseline("countdown", suffix);
    assert_eq!(p.len(), 6, "坏行: {p:?}");
    let row = format!("{p:?}");

    let m = countdown_metric(tokens);
    assert_f32(m.content_font_size, &p[0], "内容字号", &row);
    assert_close(m.content_color, parse(&p[1]), "内容字色", &row);
    assert_f32(m.title_font_size, &p[2], "标题字号", &row);
    assert_close(m.title_color, parse(&p[3]), "标题字色", &row);
    assert_f32(m.title_padding_bottom, &p[4], "标题下边距", &row);
    assert_f32(m.line_height, &p[5], "行高倍数", &row);
}

#[test]
fn countdown_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn countdown_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn countdown_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn countdown_format_matrix() {
    // 默认 HH:mm:ss；H 为总小时不 mod 24（同 antd formatTimeStr）
    assert_eq!(format_counter(90_000, "HH:mm:ss"), "00:01:30");
    assert_eq!(format_counter(0, "HH:mm:ss"), "00:00:00");
    assert_eq!(format_counter(90_000_000, "HH:mm:ss"), "25:00:00");
    // 天/时/分/秒 与 [转义]
    let d = ((2 * 86400 + 3 * 3600 + 25 * 60 + 55) as u64) * 1000;
    assert_eq!(format_counter(d, "D 天 HH:mm:ss"), "2 天 03:25:55");
    assert_eq!(format_counter(3_600_000, "[还剩] H [小时]"), "还剩 1 小时");
    // 连续单位字符按个数补零
    assert_eq!(format_counter(3_600_000, "HHH"), "001");
    // 毫秒 S 与单位扣减顺序
    assert_eq!(format_counter(1234, "s.SSS"), "1.234");
    assert_eq!(format_counter(90_000_000, "D H"), "1 1");
    // Y=365天 M=30天（antd timeUnits 字面量）
    assert_eq!(format_counter(31_536_000_000, "Y"), "1");
    assert_eq!(format_counter(2_592_000_000, "M"), "1");
    // 数字位数超过补零宽度时原样输出
    assert_eq!(format_counter(360_000_000, "H"), "100");
}
