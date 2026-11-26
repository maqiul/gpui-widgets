use gpui::Hsla;
use gpui_widgets::theme::metric::{
    ScrollbarMetric, scrollbar_metric, scrollbar_thumb_len, scrollbar_track_bg,
};

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
    let text = std::fs::read_to_string(format!(
        "harness/antd_baseline/scrollbar_matrix_{suffix}.txt"
    ))
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
    let m: ScrollbarMetric = scrollbar_metric(&t);
    let p = baseline(theme);
    assert_eq!(p.len(), 5, "scrollbar baseline field count");

    let f = |ix: usize| p[ix].parse::<f32>().unwrap();
    assert_f32(m.slot, f(0), "slot");
    assert_f32(m.thumb_radius, f(1), "thumb_radius");
    assert_f32(m.thumb_min, f(2), "thumb_min");
    assert_close(rgba(m.thumb_color), parse(&p[3]), "thumb_color");
    assert_close(rgba(m.track_hover_bg), parse(&p[4]), "track_hover_bg");

    // rc-virtual-list ScrollBar：槽 8、药丸 99、MIN_SIZE 20；
    // table/tree 的 colorSplit 覆写因 --rc-/--vc- 变量名失配失效 → 回退 rgba(0,0,0,0.5)
    assert_f32(m.slot, 8.0, "slot literal 8");
    assert_f32(m.thumb_radius, 99.0, "radius literal 99");
    assert_f32(m.thumb_min, 20.0, "min literal 20");
    assert_eq!(
        rgba(m.thumb_color),
        Rgba8 {
            r: 0,
            g: 0,
            b: 0,
            a: 128
        },
        "thumb fallback"
    );
}

#[test]
fn scrollbar_light_matrix() {
    check_matrix("light");
}

#[test]
fn scrollbar_dark_matrix() {
    check_matrix("dark");
}

#[test]
fn scrollbar_violet_matrix() {
    check_matrix("accent-violet");
}

#[test]
fn scrollbar_thumb_len_matrix() {
    // getSpinSize = container / scrollRange * container，下限 20、上限槽长
    // visible 120、max 720 → 120/840*120 ≈ 17.14 → 20（下限）
    assert_f32(scrollbar_thumb_len(120., 120., 720.), 20., "min clamp");
    // visible 300、max 300 → 120*300/600 = 60
    assert_f32(scrollbar_thumb_len(120., 300., 300.), 60., "half content");
    // visible 900、max 100 → 120*900/1000 = 108
    assert_f32(
        scrollbar_thumb_len(120., 900., 100.),
        108.,
        "mostly visible",
    );
    // visible 60、max 60 → 60/120*120 = 60
    assert_f32(scrollbar_thumb_len(120., 60., 60.), 60., "proportional");
    // 滑块不超过槽长
    assert_f32(scrollbar_thumb_len(16., 1000., 1.), 16., "track clamp");
}

#[test]
fn scrollbar_track_bg_matrix() {
    let t = tokens_for("light");
    assert_eq!(
        rgba(scrollbar_track_bg(&t, false)),
        Rgba8 {
            r: 0,
            g: 0,
            b: 0,
            a: 0
        }
    );
    assert_close(
        rgba(scrollbar_track_bg(&t, true)),
        rgba(t.color_fill_quaternary),
        "hover colorFillQuaternary",
    );
}
