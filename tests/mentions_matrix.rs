//! Mentions 回归：内部就是 TextInput，外框逐格复用 Input 状态矩阵。
//!
//! `input_matrix_*.txt` 已由浏览器 computed style 校验；这里断言
//! `mentions::state` 与 `text_input::state` 在 4 变体 × 3 状态 × 3 尺寸 × 4 交互态上完全一致。

use gpui_widgets::mentions::state;
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
                    assert_eq!(
                        state(variant, status, size, tokens, hover, focused, disabled),
                        text_input::state(variant, status, size, tokens, hover, focused, disabled),
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
fn mentions_light_matrix() {
    check_all_states(&tokens_for(None, Appearance::Light));
}

#[test]
fn mentions_dark_matrix() {
    check_all_states(&tokens_for(None, Appearance::Dark));
}

#[test]
fn mentions_violet_matrix() {
    check_all_states(&tokens_for(Some(0x8b5cf6), Appearance::Light));
}

/// 外框尺寸随 size 档位取令牌；浮层与候选列表不属于 Input 令牌范围
#[test]
fn mentions_uses_size_tokens() {
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
