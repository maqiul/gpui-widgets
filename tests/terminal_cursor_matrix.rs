//! 终端光标形状 / 度量矩阵（0.1.8 ④）。
//! 默认必须是 Bar 且厚度 2px —— 与改前硬编码 `px(2.)` 逐像素一致，670 组基线不受影响。
//! 厚度取 `line_width_bold`，随 seed 派生，渲染层不再出现魔法像素。

use gpui_widgets::terminal::CursorShape;
use gpui_widgets::theme::{Appearance, Seed, derive, metric::TerminalMetric};

#[test]
fn terminal_cursor_default_is_bar() {
    assert_eq!(CursorShape::default(), CursorShape::Bar);
}

#[test]
fn terminal_cursor_width_matches_legacy_two_px() {
    let tokens = derive(&Seed::default(), Appearance::Light);
    let m = TerminalMetric::resolve(&tokens);
    assert_eq!(m.caret_width, 2.0, "默认 seed 下须等于改前的 px(2.)");
    assert_eq!(m.caret_width, tokens.line_width_bold);
}

#[test]
fn terminal_cursor_width_follows_seed() {
    let mut seed = Seed::default();
    seed.line_width = 3.0;
    let tokens = derive(&seed, Appearance::Light);
    assert_eq!(TerminalMetric::resolve(&tokens).caret_width, 4.0);
}

#[test]
fn terminal_cursor_width_theme_independent() {
    // 光标厚度是几何量，不该随明暗翻转
    let light = TerminalMetric::resolve(&derive(&Seed::default(), Appearance::Light));
    let dark = TerminalMetric::resolve(&derive(&Seed::default(), Appearance::Dark));
    assert_eq!(light, dark);
}
