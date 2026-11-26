//! TextArea 回归：外框逐格对齐 antdv-next，多行度量单独对齐几何基准。
//!
//! TextArea 的外框和单行 Input 是同一套令牌（`textarea_matrix_*.txt` 与 `input_matrix_*.txt`
//! 逐字节相同），差异只在 `height: auto`、`min-height: controlHeight` 和基础行高。
//! 基准来自 `harness/antd_input_reference.html?kind=textarea` 的 computed style。

use gpui::{BoxShadow, Hsla, px, transparent_black};
use gpui_widgets::text_area::{metrics, state};
use gpui_widgets::text_input::{InputSize, InputStatus, InputVariant};
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive, metric};

const BASELINE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/harness/antd_baseline/");

#[derive(Clone, Copy, Debug, PartialEq)]
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
    Rgba {
        r,
        g,
        b,
        a: parts.next().unwrap_or(1.0),
    }
}

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

fn same_color(a: Rgba, b: Rgba) -> bool {
    (a.r - b.r).abs() <= 1.0
        && (a.g - b.g).abs() <= 1.0
        && (a.b - b.b).abs() <= 1.0
        && (a.a - b.a).abs() <= 0.01
}

fn state_of(text: &str) -> (bool, bool, bool) {
    match text {
        "normal" => (false, false, false),
        "hover" => (true, false, false),
        "focus" => (false, true, false),
        "disabled" => (false, false, true),
        other => panic!("未知交互态 {other}"),
    }
}

fn variant_of(text: &str) -> InputVariant {
    match text {
        "outlined" => InputVariant::Outlined,
        "filled" => InputVariant::Filled,
        "borderless" => InputVariant::Borderless,
        "underlined" => InputVariant::Underlined,
        other => panic!("未知变体 {other}"),
    }
}

fn status_of(text: &str) -> InputStatus {
    match text {
        "default" => InputStatus::Default,
        "error" => InputStatus::Error,
        "warning" => InputStatus::Warning,
        other => panic!("未知状态 {other}"),
    }
}

fn size_of(text: &str) -> InputSize {
    match text {
        "small" => InputSize::Small,
        "middle" => InputSize::Middle,
        "large" => InputSize::Large,
        other => panic!("未知尺寸 {other}"),
    }
}

/// CSS 计算值里的行高由浏览器按 4 位小数取整（25.1429），Rust 侧保留完整精度。
fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

fn check_matrix(baseline: &str, tokens: &Tokens) {
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}{baseline}"))
        .unwrap_or_else(|e| panic!("读取基准 {baseline} 失败: {e}"));
    let mut checked = 0;
    for line in text.lines().filter(|line| !line.is_empty()) {
        let col: Vec<&str> = line.split('|').collect();
        assert_eq!(col.len(), 12, "基准行格式异常: {line}");
        let (hover, focused, disabled) = state_of(col[3]);
        let got = state(
            variant_of(col[0]),
            status_of(col[1]),
            size_of(col[2]),
            tokens,
            hover,
            focused,
            disabled,
        );

        assert!(
            same_color(rgba(got.fg), parse_color(col[4])),
            "文字色不匹配: {line}"
        );
        assert!(
            same_color(rgba(got.bg), parse_color(col[5])),
            "背景色不匹配: {line}"
        );
        for (index, column) in [(0, 6), (1, 7), (2, 8), (3, 9)] {
            assert!(
                same_color(rgba(got.border[index]), parse_color(col[column])),
                "第 {index} 边描边不匹配: {line}"
            );
        }
        assert!(
            same_color(rgba(got.placeholder), parse_color(col[11])),
            "占位符色不匹配: {line}"
        );

        match (parse_shadow(col[10]), got.shadow.as_ref()) {
            (None, None) => {}
            (Some((want_color, want_y, want_blur, want_spread)), Some(shadow)) => {
                let shadow: &BoxShadow = shadow;
                assert!(
                    same_color(rgba(shadow.color), want_color),
                    "投影色不匹配: {line}"
                );
                assert_eq!(shadow.offset.y, px(want_y), "投影纵向偏移不匹配: {line}");
                assert_eq!(shadow.offset.x, px(0.0), "投影横向偏移不匹配: {line}");
                let blur = if want_blur == 0.0 {
                    metric::MIN_SHADOW_BLUR
                } else {
                    want_blur
                };
                assert_eq!(shadow.blur_radius, px(blur), "投影模糊不匹配: {line}");
                assert_eq!(
                    shadow.spread_radius,
                    px(want_spread),
                    "投影扩散不匹配: {line}"
                );
            }
            (want, got_shadow) => {
                panic!("投影有无不一致\n  基准行: {line}\n  期望: {want:?}\n  实际: {got_shadow:?}")
            }
        }
        checked += 1;
    }
    assert_eq!(
        checked, 144,
        "{baseline} 应覆盖 4 变体 × 3 状态 × 3 尺寸 × 4 交互态"
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
fn textarea_light_matrix() {
    check_matrix(
        "textarea_matrix_light.txt",
        &tokens_for(None, Appearance::Light),
    );
}

#[test]
fn textarea_dark_matrix() {
    check_matrix(
        "textarea_matrix_dark.txt",
        &tokens_for(None, Appearance::Dark),
    );
}

#[test]
fn textarea_violet_matrix() {
    check_matrix(
        "textarea_matrix_violet.txt",
        &tokens_for(Some(0x8b5cf6), Appearance::Light),
    );
}

/// 几何：内边距 / 字号 / 圆角 / 四边宽度来自 Input 令牌，
/// 行高固定用基础 `lineHeight`（大号 25.1429 而不是 24），最小高度恒为 controlHeight。
#[test]
fn textarea_geometry() {
    let tokens = derive(&Seed::default(), Appearance::Light);
    let text = std::fs::read_to_string(format!("{BASELINE_DIR}textarea_geometry.txt"))
        .expect("读取 textarea_geometry.txt 失败");
    let mut checked = 0;
    for line in text.lines().filter(|line| !line.is_empty()) {
        let col: Vec<&str> = line.split('|').collect();
        assert_eq!(col.len(), 14, "基准行格式异常: {line}");
        let num = |text: &str| text.trim().trim_end_matches("px").parse::<f32>().unwrap();
        let size = size_of(col[1]);
        let m = metrics(size, &tokens, None, None);
        let style = state(
            variant_of(col[0]),
            InputStatus::Default,
            size,
            &tokens,
            false,
            false,
            false,
        );

        assert!(close(m.min_height, num(col[2])), "最小高度不匹配: {line}");
        assert!(close(m.pad_y, num(col[3])), "上内边距不匹配: {line}");
        assert!(close(m.pad_x, num(col[4])), "右内边距不匹配: {line}");
        assert!(close(m.pad_y, num(col[5])), "下内边距不匹配: {line}");
        assert!(close(m.pad_x, num(col[6])), "左内边距不匹配: {line}");
        assert!(close(m.font_size, num(col[7])), "字号不匹配: {line}");
        assert!(close(m.line_height, num(col[8])), "行高不匹配: {line}");
        assert!(close(style.radius, num(col[9])), "圆角不匹配: {line}");
        for (index, column) in [(0, 10), (1, 11), (2, 12), (3, 13)] {
            assert!(
                close(style.border_width[index], num(col[column])),
                "第 {index} 边宽度不匹配: {line}"
            );
        }
        checked += 1;
    }
    assert_eq!(checked, 12, "应覆盖 4 变体 × 3 尺寸");
}

/// 大号 textarea 不切 `lineHeightLG`：16 × 1.5715 = 25.1429
#[test]
fn large_textarea_keeps_base_line_height() {
    let tokens = derive(&Seed::default(), Appearance::Light);
    let m = metrics(InputSize::Large, &tokens, None, None);
    assert!(close(m.line_height, 25.1429), "实际行高 {}", m.line_height);
    assert!(!close(m.line_height, 24.0));
}

/// 字号覆盖按同一比例缩放行高（代码编辑器 13px → 20.43px）
#[test]
fn text_size_override_scales_line_height() {
    let tokens = derive(&Seed::default(), Appearance::Light);
    let base = metrics(InputSize::Middle, &tokens, None, None);
    let custom = metrics(InputSize::Middle, &tokens, None, Some(13.0));
    assert!(close(custom.line_height, base.line_height * 13.0 / 14.0));
    assert_eq!(custom.font_size, 13.0);
}

/// 光标行/列口径照原版 FileEditor.updateCaret（1 起，换行后列归 1）。
#[test]
fn caret_line_col_matrix() {
    use gpui_widgets::text_area::caret_line_col;
    let cases = [
        ("", 0, (1, 1)),
        ("abc", 0, (1, 1)),
        ("abc", 2, (1, 3)),
        ("abc", 3, (1, 4)),
        ("a\nbc", 1, (1, 2)),
        ("a\nbc", 2, (2, 1)),
        ("a\nbc", 4, (2, 3)),
        ("a\n", 2, (2, 1)),
        ("a\nb\nc", 4, (3, 1)),
        ("abc", 99, (1, 4)),
        ("a\nbc", 99, (2, 3)),
    ];
    for (content, pos, want) in cases {
        assert_eq!(caret_line_col(content, pos), want, "{content:?} @{pos}");
    }
}
