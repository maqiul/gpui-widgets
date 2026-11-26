use gpui::Hsla;
use gpui_widgets::theme::metric::{KbdMetric, kbd_metric};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rgba8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

fn rgba(color: Hsla) -> Rgba8 {
    let c = gpui_widgets::theme::Color::from(color);
    let byte = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    Rgba8 {
        r: byte(c.r),
        g: byte(c.g),
        b: byte(c.b),
        a: byte(c.a * 255.0),
    }
}

fn parse(text: &str) -> Rgba8 {
    if let Some(hex) = text.strip_prefix('#') {
        let value = u32::from_str_radix(hex, 16).unwrap();
        return Rgba8 {
            r: ((value >> 16) & 0xff) as u8,
            g: ((value >> 8) & 0xff) as u8,
            b: (value & 0xff) as u8,
            a: 255,
        };
    }
    let inner = text
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or_else(|| panic!("unexpected color {text}"));
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    let byte = |s: &str| s.parse::<f32>().unwrap().round() as u8;
    Rgba8 {
        r: byte(parts[0]),
        g: byte(parts[1]),
        b: byte(parts[2]),
        a: (parts[3].parse::<f32>().unwrap() * 255.0).round() as u8,
    }
}

fn assert_close(actual: Rgba8, expected: Rgba8, label: &str) {
    let diff = |a: u8, b: u8| (a as i16 - b as i16).abs();
    assert!(
        diff(actual.r, expected.r) <= 1
            && diff(actual.g, expected.g) <= 1
            && diff(actual.b, expected.b) <= 1
            && diff(actual.a, expected.a) <= 1,
        "{label}: actual {actual:?}, expected {expected:?}"
    );
}

fn assert_f32(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.01,
        "{label}: actual {actual}, expected {expected}"
    );
}

fn tokens_for(name: &str) -> gpui_widgets::theme::Tokens {
    use gpui_widgets::theme::{Appearance, Color, Seed, derive};
    let mut seed = Seed::default();
    let appearance = match name {
        "light" => Appearance::Light,
        "dark" => Appearance::Dark,
        "accent-violet" => {
            seed.color_primary = Color::from_hex(0x8b5cf6);
            Appearance::Light
        }
        other => panic!("unknown theme {other}"),
    };
    derive(&seed, appearance)
}

fn baseline(theme: &str) -> Vec<String> {
    let suffix = match theme {
        "light" => "light",
        "dark" => "dark",
        "accent-violet" => "violet",
        other => panic!("unknown theme {other}"),
    };
    let text =
        std::fs::read_to_string(format!("harness/antd_baseline/kbd_matrix_{suffix}.txt")).unwrap();
    text.lines()
        .next()
        .unwrap()
        .split('|')
        .map(str::to_string)
        .collect()
}

fn check_matrix(theme: &str) {
    let t = tokens_for(theme);
    let m: KbdMetric = kbd_metric(&t);
    let p = baseline(theme);
    assert_eq!(p.len(), 11, "kbd baseline field count");

    let f = |ix: usize| p[ix].parse::<f32>().unwrap();
    assert_f32(m.font_size, f(0), "font_size");
    assert_f32(m.padding_inline, f(1), "padding_inline");
    assert_f32(m.padding_block_start, f(2), "padding_block_start");
    assert_f32(m.padding_block_end, f(3), "padding_block_end");
    assert_f32(m.margin_inline, f(4), "margin_inline");
    assert_f32(m.radius, f(5), "radius");
    assert_f32(m.border_width, f(6), "border_width");
    assert_f32(m.border_bottom_width, f(7), "border_bottom_width");
    assert_close(rgba(m.bg), parse(&p[8]), "bg");
    assert_close(rgba(m.border_color), parse(&p[9]), "border_color");
    assert_close(rgba(m.fg), parse(&p[10]), "fg");

    // antd typography/style/mixins.js kbd 重置样式：90% / 0.4em / 0.15em / 0.1em / 0.2em / 3px / 2px
    assert_f32(m.font_size, t.font_size * 0.9, "font 90%");
    assert_f32(m.radius, 3.0, "radius literal 3");
    assert_f32(m.border_bottom_width, 2.0, "border bottom literal 2");
}

#[test]
fn kbd_light_matrix() {
    check_matrix("light");
}

#[test]
fn kbd_dark_matrix() {
    check_matrix("dark");
}

#[test]
fn kbd_violet_matrix() {
    check_matrix("accent-violet");
}

#[test]
fn kbd_font_scale_matrix() {
    let mut t = tokens_for("light");
    t.font_size = 20.0;
    t.line_height = 1.5;
    let m = kbd_metric(&t);
    assert_f32(m.font_size, 18.0, "font 20*0.9");
    assert_f32(m.padding_inline, 7.2, "padding 0.4em");
    assert_f32(m.padding_block_start, 2.7, "padding 0.15em");
    assert_f32(m.padding_block_end, 1.8, "padding 0.1em");
    assert_f32(m.margin_inline, 3.6, "margin 0.2em");
}
