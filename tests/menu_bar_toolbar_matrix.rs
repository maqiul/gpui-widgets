use gpui_widgets::menu_bar::{MenuBarItem, menu_bar_item_state};
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive};
use gpui_widgets::toolbar::{ToolbarItem, toolbar_item_state};

fn tokens_for(accent: Option<u32>, appearance: Appearance) -> Tokens {
    let mut seed = Seed::default();
    if let Some(accent) = accent {
        seed.color_primary = Color::from_hex(accent);
    }
    derive(&seed, appearance)
}

fn check(tokens: &Tokens) {
    let menu_metrics = gpui_widgets::theme::metric::MenuBarMetric::resolve(tokens);
    assert_eq!(menu_metrics.height, tokens.control_height);
    assert_eq!(menu_metrics.item_padding_inline, tokens.padding);
    assert_eq!(menu_metrics.font_size, tokens.font_size);
    assert_eq!(menu_metrics.line_height, tokens.font_height);

    let normal = menu_bar_item_state(tokens, false, false, false, false);
    assert_eq!(normal.fg, tokens.color_text);
    assert_eq!(normal.bg.a, 0.0);
    let hover = menu_bar_item_state(tokens, false, false, true, false);
    assert_eq!(hover.fg, tokens.color_primary);
    assert_eq!(hover.bg, tokens.color_fill_tertiary);
    let selected = menu_bar_item_state(tokens, true, false, false, false);
    assert_eq!(selected.fg, tokens.color_primary);
    assert_eq!(selected.indicator, tokens.color_primary);
    let open = menu_bar_item_state(tokens, true, true, false, false);
    assert_eq!(open.bg, tokens.color_primary_bg);
    assert_eq!(open.indicator, tokens.color_primary);
    let disabled = menu_bar_item_state(tokens, true, true, true, true);
    assert_eq!(disabled.fg, tokens.color_text_disabled);
    assert_eq!(disabled.bg.a, 0.0);
    assert_eq!(disabled.indicator.a, 0.0);

    let toolbar_metrics = gpui_widgets::theme::metric::ToolbarMetric::resolve(tokens);
    assert_eq!(toolbar_metrics.height, tokens.control_height);
    assert_eq!(toolbar_metrics.padding, tokens.padding_xs);
    assert_eq!(toolbar_metrics.item_padding_inline, tokens.padding_sm);
    assert_eq!(toolbar_metrics.icon_size, tokens.font_size_lg);
    assert_eq!(toolbar_metrics.line_height, tokens.font_height);

    let normal = toolbar_item_state(tokens, false, false, false);
    assert_eq!(normal.fg, tokens.color_text);
    assert_eq!(normal.bg.a, 0.0);
    let hover = toolbar_item_state(tokens, false, true, false);
    assert_eq!(hover.bg, tokens.color_fill_tertiary);
    let selected = toolbar_item_state(tokens, true, true, false);
    assert_eq!(selected.fg, tokens.color_primary);
    assert_eq!(selected.bg, tokens.color_primary_bg);
    let disabled = toolbar_item_state(tokens, true, true, true);
    assert_eq!(disabled.fg, tokens.color_text_disabled);
    assert_eq!(disabled.bg.a, 0.0);
}

#[test]
fn menu_bar_toolbar_light_matrix() {
    check(&tokens_for(None, Appearance::Light));
}

#[test]
fn menu_bar_toolbar_dark_matrix() {
    check(&tokens_for(None, Appearance::Dark));
}

#[test]
fn menu_bar_toolbar_violet_matrix() {
    check(&tokens_for(Some(0x8b5cf6), Appearance::Light));
}

#[test]
fn item_builders_preserve_action_and_structure_state() {
    let item = ToolbarItem::new("bold", "粗体")
        .selected(true)
        .disabled(true);
    assert!(item.is_selected());
    assert!(item.is_disabled());
    assert!(!item.is_separator());
    assert!(!item.is_spacer());
    assert!(ToolbarItem::separator().is_separator());
    assert!(ToolbarItem::spacer().is_spacer());

    let menu = MenuBarItem::new("file", "文件").children(vec![gpui_widgets::MenuItem::new("新建")]);
    assert!(menu.has_children());
    assert!(!menu.is_disabled());
    assert!(MenuBarItem::new("help", "帮助").on_click(|_, _| {}).id() == "help");
}
