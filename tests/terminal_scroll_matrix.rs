use gpui::px;
use gpui_widgets::terminal::terminal_scroll_lines;

#[test]
fn terminal_scroll_bottom_is_zero() {
    assert_eq!(terminal_scroll_lines(px(0.), px(18.), 100), 0);
    assert_eq!(terminal_scroll_lines(px(5.), px(18.), 100), 0);
}

#[test]
fn terminal_scroll_negative_offset_maps_to_lines() {
    assert_eq!(terminal_scroll_lines(px(-36.), px(18.), 100), 2);
    assert_eq!(terminal_scroll_lines(px(-180.), px(18.), 100), 10);
}

#[test]
fn terminal_scroll_snaps_to_nearest_line() {
    assert_eq!(terminal_scroll_lines(px(-26.), px(18.), 100), 1);
    assert_eq!(terminal_scroll_lines(px(-28.), px(18.), 100), 2);
}

#[test]
fn terminal_scroll_clamps_to_history() {
    assert_eq!(terminal_scroll_lines(px(-3600.), px(18.), 100), 100);
    assert_eq!(terminal_scroll_lines(px(-180.), px(18.), 0), 0);
}

#[test]
fn terminal_scroll_bad_cell_height_is_safe() {
    assert_eq!(terminal_scroll_lines(px(-180.), px(0.), 100), 0);
    assert_eq!(terminal_scroll_lines(px(-180.), px(-18.), 100), 0);
}

#[test]
fn terminal_scroll_conversion_is_pure() {
    for _ in 0..3 {
        assert_eq!(terminal_scroll_lines(px(-90.), px(18.), 100), 5);
    }
}
