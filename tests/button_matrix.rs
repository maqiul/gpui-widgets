//! Button 状态矩阵回归：逐格比对 antdv-next 1.7.1 真实渲染出的按钮外观。
//!
//! 基准文件 `harness/antd_baseline/button_matrix_*.txt` 由浏览器采集脚本从真实 antdv-next
//! 组件上读取 computed style 生成（Playwright 触发 hover / active，非伪造伪类），每行：
//!
//! ```text
//! 变体|色彩|状态|文字色|背景色|描边色|投影|不透明度
//! ```
//!
//! Rust 侧只调用纯函数 [`state`]，不依赖全局主题：明 / 暗 / 换主色三套基准共用一个断言函数。

use gpui::{BoxShadow, Hsla, px, transparent_black};
use gpui_widgets::button::{ButtonColor, ButtonVariant, state};
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

/// 把 gpui 的 Hsla 拉回 0~255 的 RGB 分量，便于和 CSS 字符串比对
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

/// 解析 `rgba(5, 145, 255, 0.1) 0px 2px 0px 0px`；`none` 表示无投影。
/// 返回（颜色，纵向偏移，CSS 模糊）。
fn parse_shadow(text: &str) -> Option<(Rgba, f32, f32)> {
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
    // 水平偏移 / 垂直偏移 / 模糊 / 扩散
    let _x = offsets.next().unwrap_or(0.0);
    let y = offsets.next().unwrap_or(0.0);
    let blur = offsets.next().unwrap_or(0.0);
    Some((color, y, blur))
}

/// antd 的按钮投影是零模糊硬边，gpui 0.2.2 的 shadow shader 在 blur=0 时积分区间退化、画不出东西，
/// 所以 Rust 侧用 `MIN_SHADOW_BLUR` 代替硬边；除这一项外模糊必须和基准一致。
fn expected_blur(css_blur: f32) -> f32 {
    if css_blur == 0.0 {
        metric::MIN_SHADOW_BLUR
    } else {
        css_blur
    }
}

/// CSS 里的像素值直接和 gpui 的 `Pixels` 比：基准全是整数，不存在浮点漂移
fn assert_px(what: &str, row: &str, got: gpui::Pixels, want: f32) {
    assert_eq!(
        got,
        px(want),
        "{what} 不匹配\n  基准行: {row}\n  期望: {want}px"
    );
}

fn assert_close(what: &str, row: &str, got: Rgba, want: Rgba) {
    // 浏览器把颜色报成 8 位整数值，HSL 往返最多差 1；alpha 报成两位小数
    let ok = (got.r - want.r).abs() <= 1.0
        && (got.g - want.g).abs() <= 1.0
        && (got.b - want.b).abs() <= 1.0
        && (got.a - want.a).abs() <= 0.01;
    assert!(
        ok,
        "{what} 不匹配\n  基准行: {row}\n  期望: {want:?}\n  实际: {got:?}"
    );
}

fn parse_variant(text: &str) -> ButtonVariant {
    match text {
        "solid" => ButtonVariant::Solid,
        "outlined" => ButtonVariant::Outlined,
        "dashed" => ButtonVariant::Dashed,
        "filled" => ButtonVariant::Filled,
        "text" => ButtonVariant::Text,
        "link" => ButtonVariant::Link,
        other => panic!("未知变体 {other}"),
    }
}

fn parse_color_name(text: &str) -> ButtonColor {
    match text {
        "default" => ButtonColor::Default,
        "primary" => ButtonColor::Primary,
        "danger" => ButtonColor::Danger,
        other => panic!("未知色彩 {other}"),
    }
}

/// (hover, pressed, disabled)
fn parse_state(text: &str) -> (bool, bool, bool) {
    match text {
        "normal" => (false, false, false),
        "hover" => (true, false, false),
        "active" => (false, true, false),
        "disabled" => (false, false, true),
        other => panic!("未知状态 {other}"),
    }
}

fn check(baseline: &str, tokens: &Tokens) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}{baseline}"))
        .unwrap_or_else(|e| panic!("读取基准 {baseline} 失败: {e}"));
    let mut checked = 0;
    for line in text.lines() {
        if line.is_empty() || line.starts_with("GEO|") {
            continue;
        }
        let col: Vec<&str> = line.split('|').collect();
        assert_eq!(col.len(), 8, "基准行格式异常: {line}");
        let (hover, pressed, disabled) = parse_state(col[2]);
        let got = state(
            parse_color_name(col[1]),
            parse_variant(col[0]),
            tokens,
            false,
            hover,
            pressed,
            disabled,
        );

        assert_close("文字色", line, rgba(got.fg), parse_color(col[3]));
        assert_close("背景色", line, rgba(got.bg), parse_color(col[4]));
        assert_close("描边色", line, rgba(got.border), parse_color(col[5]));

        match (parse_shadow(col[6]), got.shadow.as_ref()) {
            (None, None) => {}
            (Some((want_color, want_y, want_blur)), Some(shadow)) => {
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
                assert_px("投影扩散", line, shadow.spread_radius, 0.0);
            }
            (want, got_shadow) => {
                panic!("投影有无不一致\n  基准行: {line}\n  期望: {want:?}\n  实际: {got_shadow:?}")
            }
        }
        // antd 靠颜色令牌表达禁用，不压整体透明度（gpui 的 opacity 会连文字一起发灰）
        assert_eq!(col[7], "1", "基准里出现了整体透明度: {line}");
        checked += 1;
    }
    assert_eq!(checked, 72, "{baseline} 应覆盖 6 变体 × 3 色彩 × 4 状态");
}

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

#[test]
fn button_light_matrix() {
    check(
        "button_matrix_light.txt",
        &tokens_for(None, Appearance::Light),
    );
}

#[test]
fn button_dark_matrix() {
    check(
        "button_matrix_dark.txt",
        &tokens_for(None, Appearance::Dark),
    );
}

/// 换主色：整张矩阵必须跟着重新派生，而不是只有静止态变了
#[test]
fn button_violet_matrix() {
    check(
        "button_matrix_violet.txt",
        &tokens_for(Some(0x8b5cf6), Appearance::Light),
    );
}

/// 禁用态文字必须与 antd 一样保持整格不透明（旧实现整体压到 0.55 是错的）
#[test]
fn button_disabled_does_not_dim_whole_button() {
    let tokens = tokens_for(None, Appearance::Light);
    let style = state(
        ButtonColor::Primary,
        ButtonVariant::Solid,
        &tokens,
        false,
        false,
        false,
        true,
    );
    assert_eq!(style.fg, tokens.color_text_disabled);
    assert_eq!(style.bg, tokens.color_bg_container_disabled);
    assert_eq!(style.shadow, None);
}

/// ghost 只换底色：描边按钮在彩色深底上仍然按主色族派生
#[test]
fn button_ghost_keeps_accent_derivation() {
    let tokens = tokens_for(None, Appearance::Light);
    let plain = state(
        ButtonColor::Primary,
        ButtonVariant::Outlined,
        &tokens,
        false,
        false,
        false,
        false,
    );
    let ghost = state(
        ButtonColor::Primary,
        ButtonVariant::Outlined,
        &tokens,
        true,
        false,
        false,
        false,
    );
    assert_eq!(plain.fg, ghost.fg);
    assert_eq!(ghost.bg, rgba_zero());
    assert_ne!(plain.bg, ghost.bg);
}

fn rgba_zero() -> Hsla {
    transparent_black()
}

/// `icon_element` 是加性旋钮：只占渲染槽，不参与 `state` 派生（逐格矩阵由上面的
/// 基准测试覆盖）；这里冒烟三种形态——元素槽 / 字符槽 / 无图标——都能独立建容器。
#[test]
fn button_icon_element_knob_is_additive() {
    use gpui::{IntoElement, ParentElement};
    use gpui_widgets::button::{Button, ICON_GROUP};

    assert_eq!(ICON_GROUP, "gpui-button");
    let _ = Button::new("生成")
        .small()
        .icon_element(gpui::div().child("i"))
        .into_element();
    let _ = Button::new("生成").small().icon("＋").into_element();
    let _ = Button::new("生成").small().into_element();
}
