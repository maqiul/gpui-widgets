//! Input 几何基准：高度 / 内边距 / 字号 / 行高 / 圆角 / 四边宽度。
//!
//! 数据同样来自浏览器 computed style（`harness/antd_baseline/input_geometry.txt`），
//! 每行 `变体|尺寸|高|上|右|下|左|字号|行高|圆角|上|右|下|左边宽`，全部是 CSS px。

use gpui::px;
use gpui_widgets::text_input::{InputSize, InputStatus, InputVariant, state};
use gpui_widgets::theme::{Appearance, Seed, derive};

const BASELINE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/harness/antd_baseline/input_geometry.txt"
);

fn parse_variant(text: &str) -> InputVariant {
    match text {
        "outlined" => InputVariant::Outlined,
        "filled" => InputVariant::Filled,
        "borderless" => InputVariant::Borderless,
        "underlined" => InputVariant::Underlined,
        other => panic!("未知变体 {other}"),
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

fn num(text: &str) -> f32 {
    text.trim().trim_end_matches("px").parse().unwrap()
}

#[test]
fn input_geometry() {
    let tokens = derive(&Seed::default(), Appearance::Light);
    let text = std::fs::read_to_string(BASELINE).expect("读取 input_geometry.txt 失败");
    let mut checked = 0;
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let col: Vec<&str> = line.split('|').collect();
        assert_eq!(col.len(), 14, "基准行格式异常: {line}");
        let style = state(
            parse_variant(col[0]),
            InputStatus::Default,
            parse_size(col[1]),
            &tokens,
            false,
            false,
            false,
        );
        assert_eq!(style.height, num(col[2]), "高度不匹配: {line}");
        assert_eq!(style.padding_block, num(col[3]), "上内边距不匹配: {line}");
        assert_eq!(style.padding_inline, num(col[4]), "右内边距不匹配: {line}");
        assert_eq!(style.padding_block, num(col[5]), "下内边距不匹配: {line}");
        assert_eq!(style.padding_inline, num(col[6]), "左内边距不匹配: {line}");
        assert_eq!(style.font_size, num(col[7]), "字号不匹配: {line}");
        assert_eq!(style.line_height, num(col[8]), "行高不匹配: {line}");
        assert_eq!(style.radius, num(col[9]), "圆角不匹配: {line}");
        assert_eq!(
            style.border_width[0],
            num(col[10]),
            "上边框宽不匹配: {line}"
        );
        assert_eq!(
            style.border_width[1],
            num(col[11]),
            "右边框宽不匹配: {line}"
        );
        assert_eq!(
            style.border_width[2],
            num(col[12]),
            "下边框宽不匹配: {line}"
        );
        assert_eq!(
            style.border_width[3],
            num(col[13]),
            "左边框宽不匹配: {line}"
        );
        checked += 1;
    }
    assert_eq!(checked, 12, "应覆盖 4 变体 × 3 尺寸");
}

/// 尺寸直接来自令牌：高度 24 / 32 / 40，大号字 16px、行高 24px
#[test]
fn sizes_follow_control_height_tokens() {
    let tokens = derive(&Seed::default(), Appearance::Light);
    for (size, height, font_size) in [
        (InputSize::Small, tokens.control_height_sm, tokens.font_size),
        (InputSize::Middle, tokens.control_height, tokens.font_size),
        (
            InputSize::Large,
            tokens.control_height_lg,
            tokens.font_size_lg,
        ),
    ] {
        let style = state(
            InputVariant::Outlined,
            InputStatus::Default,
            size,
            &tokens,
            false,
            false,
            false,
        );
        assert_eq!(style.height, height);
        assert_eq!(style.font_size, font_size);
        assert_eq!(style.border_width, [tokens.line_width; 4]);
        assert_eq!(px(style.height), px(height));
    }
}
