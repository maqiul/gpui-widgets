//! Terminal pane-fit 网格换算纯函数矩阵（阶段三十八-A）。
//! 只测 `fit_grid`：不建 Window / 不渲染，默认旋钮关闭时布局走老路径，
//! 670 组截图基线不受影响。

use gpui::px;
use gpui_widgets::terminal::{fit_grid, terminal_fit_gate};

#[test]
fn terminal_fit_grid_exact() {
    assert_eq!(fit_grid(px(800.), px(480.), px(8.), px(16.)), (100, 30));
}

#[test]
fn terminal_fit_grid_floor_not_round() {
    // 余量不足一格时必须向下取整，round 会多算一行把 PTY 撑出容器。
    assert_eq!(fit_grid(px(807.), px(495.), px(8.), px(16.)), (100, 30));
    assert_eq!(fit_grid(px(808.), px(496.), px(8.), px(16.)), (101, 31));
}

#[test]
fn terminal_fit_grid_min_one() {
    assert_eq!(fit_grid(px(4.), px(4.), px(8.), px(16.)), (1, 1));
    assert_eq!(fit_grid(px(1.), px(1.), px(8.), px(16.)), (1, 1));
}

#[test]
fn terminal_fit_grid_unlaid_bounds_skip() {
    // 容器尚未布局（0 / 负尺寸）返回 (0,0)，调用方跳过 resize。
    assert_eq!(fit_grid(px(0.), px(480.), px(8.), px(16.)), (0, 0));
    assert_eq!(fit_grid(px(800.), px(0.), px(8.), px(16.)), (0, 0));
    assert_eq!(fit_grid(px(-1.), px(480.), px(8.), px(16.)), (0, 0));
    assert_eq!(fit_grid(px(800.), px(-1.), px(8.), px(16.)), (0, 0));
}

#[test]
fn terminal_fit_grid_bad_cell_skip() {
    assert_eq!(fit_grid(px(800.), px(480.), px(0.), px(16.)), (0, 0));
    assert_eq!(fit_grid(px(800.), px(480.), px(8.), px(0.)), (0, 0));
    assert_eq!(fit_grid(px(800.), px(480.), px(-8.), px(16.)), (0, 0));
}

#[test]
fn terminal_fit_grid_fractional_cell() {
    // 非整数单元格度量（DPI 缩放 / 字体回退）同样向下取整。
    assert_eq!(fit_grid(px(100.), px(100.), px(7.5), px(15.5)), (13, 6));
}

#[test]
fn terminal_fit_gate_blocks_transient_grid() {
    // 宿主拖动中的 5×3 瞬态不应触发包内本地 reflow。
    assert_eq!(
        terminal_fit_gate(px(40.), px(54.), px(8.), px(18.), (20, 5)),
        None
    );
    assert_eq!(
        terminal_fit_gate(px(1280.), px(720.), px(8.), px(18.), (20, 5)),
        Some((160, 40))
    );
}

#[test]
fn terminal_fit_gate_defaults_allow_one_cell() {
    assert_eq!(
        terminal_fit_gate(px(4.), px(4.), px(8.), px(18.), (1, 1)),
        Some((1, 1))
    );
    assert_eq!(
        terminal_fit_gate(px(0.), px(720.), px(8.), px(18.), (1, 1)),
        None
    );
    assert_eq!(
        terminal_fit_gate(px(1280.), px(0.), px(8.), px(18.), (1, 1)),
        None
    );
}

#[test]
fn terminal_fit_gate_clamps_bad_minimums() {
    // 0 门槛按 1 处理，不能让闸门反过来吞掉合法 1×1 网格。
    assert_eq!(
        terminal_fit_gate(px(4.), px(4.), px(8.), px(18.), (0, 0)),
        Some((1, 1))
    );
}
