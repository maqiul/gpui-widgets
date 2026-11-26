use gpui::Hsla;
use gpui_widgets::theme::metric::{StepperMetric, stepper_button_state, stepper_metric};

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
        std::fs::read_to_string(format!("harness/antd_baseline/stepper_matrix_{suffix}.txt"))
            .unwrap();
    text.lines()
        .next()
        .unwrap()
        .split('|')
        .map(str::to_string)
        .collect()
}

fn check_matrix(theme: &str) {
    let t = tokens_for(theme);
    let m: StepperMetric = stepper_metric(&t);
    let p = baseline(theme);
    assert_eq!(p.len(), 11, "stepper baseline field count");

    let f = |ix: usize| p[ix].parse::<f32>().unwrap();
    assert_f32(m.height, f(0), "height");
    assert_f32(m.radius, f(1), "radius");
    assert_f32(m.border_width, f(2), "border_width");
    assert_close(rgba(m.border_color), parse(&p[3]), "border_color");
    assert_close(rgba(m.bg), parse(&p[4]), "bg");
    assert_close(rgba(m.disabled_bg), parse(&p[5]), "disabled_bg");
    assert_close(rgba(m.disabled_fg), parse(&p[6]), "disabled_fg");
    assert_f32(m.button_size, f(7), "button_size");
    assert_f32(m.value_width, f(8), "value_width");
    assert_f32(m.font_size, f(9), "font_size");
    assert_close(rgba(m.fg), parse(&p[10]), "fg");

    // 映射自 antd InputNumber 容器令牌 + 通用交互态（无 antd 横向步进器对应）
    assert_f32(m.height, t.control_height, "height controlHeight");
    assert_f32(
        m.button_size,
        t.control_height - 2.0 * t.line_width,
        "button inner height",
    );
    assert_close(
        rgba(m.border_color),
        rgba(t.color_border),
        "border colorBorder",
    );
}

#[test]
fn stepper_light_matrix() {
    check_matrix("light");
}

#[test]
fn stepper_dark_matrix() {
    check_matrix("dark");
}

#[test]
fn stepper_violet_matrix() {
    check_matrix("accent-violet");
}

#[test]
fn stepper_button_state_matrix() {
    let t = tokens_for("light");
    let transparent = Rgba8 {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    let idle = stepper_button_state(&t, false, false, false, false);
    assert_eq!(rgba(idle.bg), transparent);
    assert_close(rgba(idle.fg), rgba(t.color_text), "idle fg");

    let hover = stepper_button_state(&t, false, false, true, false);
    assert_close(rgba(hover.bg), rgba(t.color_bg_text_hover), "hover bg");

    let active = stepper_button_state(&t, false, false, true, true);
    assert_close(rgba(active.bg), rgba(t.control_item_bg_active), "active bg");

    let disabled = stepper_button_state(&t, true, false, true, true);
    assert_eq!(rgba(disabled.bg), transparent);
    assert_close(
        rgba(disabled.fg),
        rgba(t.color_text_disabled),
        "disabled fg",
    );

    let at_bound = stepper_button_state(&t, false, true, true, true);
    assert_eq!(rgba(at_bound.bg), transparent);
    assert_close(
        rgba(at_bound.fg),
        rgba(t.color_text_disabled),
        "at bound fg",
    );
}
