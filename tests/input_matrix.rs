//! Input 状态矩阵回归：逐格比对 antdv-next 1.5.6 真实渲染出的输入框外观。
//!
//! 基准文件 `harness/antd_baseline/input_matrix_*.txt` 由 `harness/antd_input_reference.html`
//! 在浏览器里生成（72 个真实 `<Input>`，hover / focus 通过改写 `:hover` / `:focus` 规则为可切换类
//! 后读取 computed style，禁用态直接读真实 `disabled` 元素），每行：
//!
//! ```text
//! 变体|状态|尺寸|交互态|文字色|背景色|上|右|下|左|投影|占位符色
//! ```
//!
//! Rust 侧只调用纯函数 [`state`]，明 / 暗 / 换主色三套基准共用一个断言函数。

use gpui::{BoxShadow, Hsla, px, transparent_black};
use gpui_widgets::text_input::{
    InputAlign, InputSize, InputStatus, InputVariant, align_offset, state,
};
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive, metric};

const BASELINE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/harness/antd_baseline/");

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Rgba {
    fn of(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

fn rgba(color: Hsla) -> Rgba {
    let color = Color::from(color);
    Rgba::of(color.r, color.g, color.b, color.a)
}

/// 解析 `rgb(22, 119, 255)` / `rgba(0,0,0,0.75)` / `transparent`
fn parse_color(text: &str) -> Rgba {
    let text = text.trim();
    if text == "transparent" {
        return rgba(transparent_black());
    }
    let body = text
        .trim_start_matches("rgb(")
        .trim_start_matches("rgba(")
        .trim_end_matches(')');
    let mut parts = body.split(',').map(|p| p.trim().parse::<f32>().unwrap());
    let (r, g, b) = (
        parts.next().unwrap(),
        parts.next().unwrap(),
        parts.next().unwrap(),
    );
    let a = parts.next().unwrap_or(1.0);
    Rgba::of(r, g, b, a)
}

/// 解析 `rgba(5, 145, 255, 0.1) 0px 0px 0px 2px`；`none` 表示无投影。
/// 返回（颜色，纵向偏移，CSS 模糊，扩散）。
fn parse_shadow(text: &str) -> Option<(Rgba, f32, f32, f32)> {
    let text = text.trim();
    if text == "none" {
        return None;
    }
    let open = text.find(')')?;
    let color = parse_color(&text[..=open]);
    let mut offsets = text[open + 1..].split_whitespace().map(|v| {
        v.trim_end_matches("px")
            .trim_end_matches("em")
            .parse::<f32>()
            .unwrap_or(0.0)
    });
    let _x = offsets.next().unwrap_or(0.0);
    let y = offsets.next().unwrap_or(0.0);
    let blur = offsets.next().unwrap_or(0.0);
    let spread = offsets.next().unwrap_or(0.0);
    Some((color, y, blur, spread))
}

/// 和按钮一样：gpui 0.2.2 画不出零模糊硬边，Rust 侧用 `MIN_SHADOW_BLUR` 代替。
fn expected_blur(css_blur: f32) -> f32 {
    if css_blur == 0.0 {
        metric::MIN_SHADOW_BLUR
    } else {
        css_blur
    }
}

fn assert_px(what: &str, row: &str, got: gpui::Pixels, want: f32) {
    assert_eq!(
        got,
        px(want),
        "{what} 不匹配\n  基准行: {row}\n  期望: {want}px"
    );
}

fn assert_close(what: &str, row: &str, got: Rgba, want: Rgba) {
    let ok = (got.r - want.r).abs() <= 1.0
        && (got.g - want.g).abs() <= 1.0
        && (got.b - want.b).abs() <= 1.0
        && (got.a - want.a).abs() <= 0.01;
    assert!(
        ok,
        "{what} 不匹配\n  基准行: {row}\n  期望: {want:?}\n  实际: {got:?}"
    );
}

fn parse_variant(text: &str) -> InputVariant {
    match text {
        "outlined" => InputVariant::Outlined,
        "filled" => InputVariant::Filled,
        "borderless" => InputVariant::Borderless,
        "underlined" => InputVariant::Underlined,
        other => panic!("未知变体 {other}"),
    }
}

fn parse_status(text: &str) -> InputStatus {
    match text {
        "default" => InputStatus::Default,
        "error" => InputStatus::Error,
        "warning" => InputStatus::Warning,
        other => panic!("未知状态 {other}"),
    }
}

fn parse_size(text: &str) -> InputSize {
    match text {
        "small" => InputSize::Small,
        "middle" => InputSize::Middle,
        "large" => InputSize::Large,
        other => panic!("未知尺寸 {other}"),
    }
}

/// (hover, focused, disabled)
fn parse_state(text: &str) -> (bool, bool, bool) {
    match text {
        "normal" => (false, false, false),
        "hover" => (true, false, false),
        "focus" => (false, true, false),
        "disabled" => (false, false, true),
        other => panic!("未知交互态 {other}"),
    }
}

fn check(baseline: &str, tokens: &Tokens) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}{baseline}"))
        .unwrap_or_else(|e| panic!("读取基准 {baseline} 失败: {e}"));
    let mut checked = 0;
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let col: Vec<&str> = line.split('|').collect();
        assert_eq!(col.len(), 12, "基准行格式异常: {line}");
        let (hover, focused, disabled) = parse_state(col[3]);
        let got = state(
            parse_variant(col[0]),
            parse_status(col[1]),
            parse_size(col[2]),
            tokens,
            hover,
            focused,
            disabled,
        );

        assert_close("文字色", line, rgba(got.fg), parse_color(col[4]));
        assert_close("背景色", line, rgba(got.bg), parse_color(col[5]));
        assert_close("上描边", line, rgba(got.border[0]), parse_color(col[6]));
        assert_close("右描边", line, rgba(got.border[1]), parse_color(col[7]));
        assert_close("下描边", line, rgba(got.border[2]), parse_color(col[8]));
        assert_close("左描边", line, rgba(got.border[3]), parse_color(col[9]));
        assert_close(
            "占位符色",
            line,
            rgba(got.placeholder),
            parse_color(col[11]),
        );

        match (parse_shadow(col[10]), got.shadow.as_ref()) {
            (None, None) => {}
            (Some((want_color, want_y, want_blur, want_spread)), Some(shadow)) => {
                let shadow: &BoxShadow = shadow;
                assert_close("投影色", line, rgba(shadow.color), want_color);
                assert_px("投影纵向偏移", line, shadow.offset.y, want_y);
                assert_px("投影横向偏移", line, shadow.offset.x, 0.0);
                assert_px(
                    "投影模糊",
                    line,
                    shadow.blur_radius,
                    expected_blur(want_blur),
                );
                assert_px("投影扩散", line, shadow.spread_radius, want_spread);
            }
            (want, got_shadow) => {
                panic!("投影有无不一致\n  基准行: {line}\n  期望: {want:?}\n  实际: {got_shadow:?}")
            }
        }
        checked += 1;
    }
    assert_eq!(
        checked, 144,
        "{baseline} 应覆盖 4 变体 × 3 状态 × 3 尺寸 × 3 可用态 + 36 禁用行"
    );
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn input_light_matrix() {
    check(
        "input_matrix_light.txt",
        &tokens_for(None, Appearance::Light),
    );
}

#[test]
fn input_dark_matrix() {
    check("input_matrix_dark.txt", &tokens_for(None, Appearance::Dark));
}

/// 换主色：主色族（focus / hover / outline）必须整体重算，错误与警告色不受影响
#[test]
fn input_violet_matrix() {
    check(
        "input_matrix_violet.txt",
        &tokens_for(Some(0x8b5cf6), Appearance::Light),
    );
}

/// underlined 只有一条底边，且是直角
#[test]
fn underlined_has_bottom_border_only() {
    let tokens = tokens_for(None, Appearance::Light);
    let style = state(
        InputVariant::Underlined,
        InputStatus::Default,
        InputSize::Middle,
        &tokens,
        false,
        false,
        false,
    );
    assert_eq!(style.border_width, [1.0, 0.0, 1.0, 0.0]);
    assert_eq!(style.radius, 0.0);
    assert_eq!(style.border[0].a, 0.0);
    assert_eq!(style.border[2], tokens.color_border);
}

/// borderless 聚焦时没有任何视觉反馈（antd 只给 focus-visible 描 outline，gpui 无 outline）
#[test]
fn borderless_focus_stays_flat() {
    let tokens = tokens_for(None, Appearance::Light);
    let style = state(
        InputVariant::Borderless,
        InputStatus::Default,
        InputSize::Middle,
        &tokens,
        false,
        true,
        false,
    );
    assert_eq!(style.shadow, None);
    assert!(style.border.iter().all(|color| color.a == 0.0));
    assert_eq!(style.bg, transparent_black());
}

/// filled 聚焦会回到容器底色 + 主色描边，但没有聚焦环
#[test]
fn filled_focus_moves_to_container_bg() {
    let tokens = tokens_for(None, Appearance::Light);
    let normal = state(
        InputVariant::Filled,
        InputStatus::Default,
        InputSize::Middle,
        &tokens,
        false,
        false,
        false,
    );
    let focused = state(
        InputVariant::Filled,
        InputStatus::Default,
        InputSize::Middle,
        &tokens,
        false,
        true,
        false,
    );
    assert_eq!(normal.bg, tokens.color_fill_tertiary);
    assert_eq!(focused.bg, tokens.color_bg_container);
    assert_eq!(focused.border, [tokens.color_primary; 4]);
    assert_eq!(focused.shadow, None);
}

/// 禁用态用 colorTextDisabled / colorBgContainerDisabled，不压整体透明度
#[test]
fn disabled_uses_disabled_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    let style = state(
        InputVariant::Outlined,
        InputStatus::Default,
        InputSize::Middle,
        &tokens,
        false,
        false,
        true,
    );
    assert_eq!(style.fg, tokens.color_text_disabled);
    assert_eq!(style.bg, tokens.color_bg_container_disabled);
    assert_eq!(style.border, [tokens.color_border; 4]);
    assert_eq!(style.shadow, None);
}

/// 宿主覆盖排版（命令条 12px 等宽）必须是加性的：两个开关都不开时逐字等于令牌值。
#[test]
fn input_typography_matrix() {
    let tokens = tokens_for(None, Appearance::Light);
    let mono = metric::mono_font_family();
    assert!(!mono.is_empty(), "等宽族在目标平台必须有兜底");

    for size in [InputSize::Small, InputSize::Middle, InputSize::Large] {
        let base = state(
            InputVariant::Outlined,
            InputStatus::Default,
            size,
            &tokens,
            false,
            false,
            false,
        )
        .font_size;
        assert_eq!(
            metric::input_typography(base, None, false),
            (base, None),
            "{size:?} 默认排版不该被改动"
        );
        assert_eq!(
            metric::input_typography(base, None, true),
            (base, Some(mono)),
            "{size:?} 只开等宽不该动字号"
        );
        assert_eq!(
            metric::input_typography(base, Some(12.0), false),
            (12.0, None),
            "{size:?} 只覆盖字号不该动字体"
        );
        assert_eq!(
            metric::input_typography(base, Some(12.0), true),
            (12.0, Some(mono))
        );
    }
}

/// 对齐偏移必须是加性的：默认左对齐恒 0（与未引入对齐前逐字一致），
/// 居中把差值平分并在行宽超出容器时夹回 0。
#[test]
fn input_align_offset_matrix() {
    assert_eq!(InputAlign::default(), InputAlign::Left);

    for (container, line) in [(100.0, 40.0), (50.0, 0.0), (300.0, 300.0), (80.0, 120.0)] {
        assert_eq!(
            align_offset(InputAlign::Left, container, line),
            0.0,
            "左对齐在任意宽度下都不该产生偏移"
        );
    }

    assert_eq!(align_offset(InputAlign::Center, 100.0, 40.0), 30.0);
    assert_eq!(align_offset(InputAlign::Center, 50.0, 0.0), 25.0);
    assert_eq!(align_offset(InputAlign::Center, 100.0, 100.0), 0.0);
    assert_eq!(
        align_offset(InputAlign::Center, 100.0, 140.0),
        0.0,
        "行比容器宽时夹回 0，不能负偏移"
    );
}
