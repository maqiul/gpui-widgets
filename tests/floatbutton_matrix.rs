//! FloatButton 矩阵：度量 + default/primary 三态 + 禁用。
//!
//! 基准由 `harness/antd_baseline/gen_float_split_matrix.mjs` 依据 antdv-next
//! `float-button/style/index.js` + `button.js` + `button/style/index.js` 生成。
use gpui::Hsla;
use gpui_widgets::button::{self, ButtonColor, ButtonVariant};
use gpui_widgets::theme::metric::float_button_metric;
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
    let p = baseline("floatbutton", suffix);
    assert_eq!(p.len(), 18, "坏行: {p:?}");
    let row = format!("{p:?}");

    let m = float_button_metric(tokens);
    assert_f32(m.size, &p[0], "尺寸", &row);
    assert_f32(m.icon_size, &p[1], "图标字号", &row);
    assert_f32(m.padding_block, &p[2], "纵向内边距", &row);
    assert_f32(m.gap, &p[3], "间隔", &row);
    assert_f32(m.inset_block_end, &p[4], "底部偏移", &row);
    assert_f32(m.inset_inline_end, &p[5], "右侧偏移", &row);
    assert_f32(m.radius_square, &p[6], "方形圆角", &row);
    assert_f32(m.border_width, &p[7], "描边宽", &row);

    let idle = button::state(
        ButtonColor::Default,
        ButtonVariant::Outlined,
        tokens,
        false,
        false,
        false,
        false,
    );
    assert_close(idle.bg, parse(&p[8]), "默认底", &row);
    assert_close(idle.border, parse(&p[9]), "默认描边", &row);
    assert_close(idle.fg, parse(&p[10]), "默认文字", &row);

    let hover = button::state(
        ButtonColor::Default,
        ButtonVariant::Outlined,
        tokens,
        false,
        true,
        false,
        false,
    );
    assert_close(hover.border, parse(&p[11]), "悬停描边", &row);
    assert_close(hover.fg, parse(&p[11]), "悬停文字", &row);
    assert_close(hover.bg, parse(&p[8]), "悬停底不变", &row);

    let active = button::state(
        ButtonColor::Default,
        ButtonVariant::Outlined,
        tokens,
        false,
        false,
        true,
        false,
    );
    assert_close(active.border, parse(&p[12]), "按下描边", &row);
    assert_close(active.fg, parse(&p[12]), "按下文字", &row);

    let primary = button::state(
        ButtonColor::Primary,
        ButtonVariant::Solid,
        tokens,
        false,
        false,
        false,
        false,
    );
    assert_close(primary.bg, parse(&p[13]), "主色底", &row);
    assert_close(primary.fg, parse(&p[14]), "主色文字", &row);
    assert_eq!(rgba(primary.border).a, 0.0, "主色无描边");

    let disabled = button::state(
        ButtonColor::Default,
        ButtonVariant::Outlined,
        tokens,
        false,
        false,
        false,
        true,
    );
    assert_close(disabled.bg, parse(&p[15]), "禁用底", &row);
    assert_close(disabled.fg, parse(&p[16]), "禁用文字", &row);
    assert_close(disabled.border, parse(&p[17]), "禁用描边", &row);
}

#[test]
fn floatbutton_light_matrix() {
    check(&tokens_for(None, Appearance::Light), "light");
}

#[test]
fn floatbutton_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark), "dark");
}

#[test]
fn floatbutton_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light), "violet");
}

#[test]
fn floatbutton_state_matrix() {
    let tokens = tokens_for(None, Appearance::Light);
    // 主色族 hover / active 逐级跟随种子派生色
    let hover = button::state(
        ButtonColor::Primary,
        ButtonVariant::Solid,
        &tokens,
        false,
        true,
        false,
        false,
    );
    assert_close(hover.bg, parse("#4096ff"), "主色悬停底", "state");
    let active = button::state(
        ButtonColor::Primary,
        ButtonVariant::Solid,
        &tokens,
        false,
        false,
        true,
        false,
    );
    assert_close(active.bg, parse("#0958d9"), "主色按下底", "state");
    // 禁用主色也走容器禁用底，而不是主色变灰
    let disabled = button::state(
        ButtonColor::Primary,
        ButtonVariant::Solid,
        &tokens,
        false,
        true,
        false,
        true,
    );
    assert_close(
        disabled.bg,
        parse("rgba(0,0,0,0.04)"),
        "主色禁用底",
        "state",
    );
    assert_close(
        disabled.fg,
        parse("rgba(0,0,0,0.25)"),
        "主色禁用文字",
        "state",
    );
}
