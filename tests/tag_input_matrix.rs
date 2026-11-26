//! TagInput 回归：外框直接复用 Input 状态矩阵。
//!
//! antdv-next 没有 TagInput 组件，TagInput 的视觉契约是"Input 外框 + 标签"，
//! 因此逐格断言 `tag_input::state` 与 `text_input::state` 完全一致；
//! 后者已由 `input_matrix_*.txt`（浏览器 computed style）逐格校验。

use gpui_widgets::tag_input::state;
use gpui_widgets::text_input::{self, InputSize, InputStatus, InputVariant};
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive};

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

fn check_all_states(tokens: &Tokens) {
    let variants = [
        InputVariant::Outlined,
        InputVariant::Filled,
        InputVariant::Borderless,
        InputVariant::Underlined,
    ];
    let statuses = [
        InputStatus::Default,
        InputStatus::Error,
        InputStatus::Warning,
    ];
    let sizes = [InputSize::Small, InputSize::Middle, InputSize::Large];
    // 与矩阵基准相同的四态
    let states = [
        (false, false, false),
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ];
    let mut checked = 0;
    for variant in variants {
        for status in statuses {
            for size in sizes {
                for (hover, focused, disabled) in states {
                    let frame = state(variant, status, size, tokens, hover, focused, disabled);
                    let input =
                        text_input::state(variant, status, size, tokens, hover, focused, disabled);
                    assert_eq!(
                        frame, input,
                        "{variant:?}/{status:?}/{size:?}/{hover}{focused}{disabled}"
                    );
                    checked += 1;
                }
            }
        }
    }
    assert_eq!(checked, 144);
}

#[test]
fn tag_input_light_matrix() {
    check_all_states(&tokens_for(None, Appearance::Light));
}

#[test]
fn tag_input_dark_matrix() {
    check_all_states(&tokens_for(None, Appearance::Dark));
}

#[test]
fn tag_input_violet_matrix() {
    check_all_states(&tokens_for(Some(0x8b5cf6), Appearance::Light));
}

/// 外框最小高度 / 内边距 / 字号取尺寸档令牌；标签换行时高度由内容决定
#[test]
fn tag_input_uses_size_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    for (size, height) in [
        (InputSize::Small, tokens.control_height_sm),
        (InputSize::Middle, tokens.control_height),
        (InputSize::Large, tokens.control_height_lg),
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
        assert_eq!(
            style.font_size,
            if size == InputSize::Large {
                tokens.font_size_lg
            } else {
                tokens.font_size
            }
        );
    }
}
