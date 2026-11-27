use gpui_widgets::input::otp_input::{insert_otp_text, normalize_otp_text, otp_state, otp_target};
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
                        otp_state(variant, status, size, tokens, hover, focused, disabled),
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
fn otp_input_light_matrix() {
    check_all_states(&tokens_for(None, Appearance::Light));
}

#[test]
fn otp_input_dark_matrix() {
    check_all_states(&tokens_for(None, Appearance::Dark));
}

#[test]
fn otp_input_violet_matrix() {
    check_all_states(&tokens_for(Some(0x8b5cf6), Appearance::Light));
}

#[test]
fn otp_input_uses_input_size_tokens() {
    let tokens = tokens_for(None, Appearance::Light);
    for (size, height) in [
        (InputSize::Small, tokens.control_height_sm),
        (InputSize::Middle, tokens.control_height),
        (InputSize::Large, tokens.control_height_lg),
    ] {
        let style = otp_state(
            InputVariant::Outlined,
            InputStatus::Default,
            size,
            &tokens,
            false,
            false,
            false,
        );
        assert_eq!(style.height, height);
    }
}

#[test]
fn otp_input_paste_fills_from_active_cell_and_clamps() {
    let mut chars = ['\0'; 6];
    let mut active = 1;
    assert!(insert_otp_text(&mut chars, &mut active, " 12-34 "));
    assert_eq!(chars, ['\0', '1', '2', '3', '4', '\0']);
    assert_eq!(active, 5);
    assert!(insert_otp_text(&mut chars, &mut active, "56789"));
    assert_eq!(chars, ['\0', '1', '2', '3', '4', '5']);
    assert_eq!(active, 5);
    assert_eq!(normalize_otp_text("a中_ !"), vec!['a', '中']);
}

#[test]
fn otp_input_keyboard_navigation_clamps_at_edges() {
    assert_eq!(otp_target(0, "left", 6), Some(0));
    assert_eq!(otp_target(5, "right", 6), Some(5));
    assert_eq!(otp_target(3, "home", 6), Some(0));
    assert_eq!(otp_target(3, "end", 6), Some(5));
}
