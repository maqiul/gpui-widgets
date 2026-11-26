//! Terminal buffer_tail 取文纯函数矩阵（阶段四十）。
//! 只测 `tail_join`：不建 Window / 不渲染，670 组截图基线不受影响。
//! 语义锚点：原版 useXterm.ts bufferTail（末尾 max(1,N) 行、去尾空白、掐首尾空行）。

use gpui_widgets::terminal::tail_join;

fn lines(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn tail_takes_the_last_window() {
    let l = lines(&["a", "b", "c", "d"]);
    assert_eq!(tail_join(&l, 2), "c\nd");
    assert_eq!(tail_join(&l, 4), "a\nb\nc\nd");
    assert_eq!(tail_join(&l, 99), "a\nb\nc\nd");
}

#[test]
fn zero_lines_still_takes_one() {
    // 原版 Math.max(1, maxLines)：0 行配置不该取成空。
    let l = lines(&["a", "b"]);
    assert_eq!(tail_join(&l, 0), "b");
}

#[test]
fn trailing_whitespace_is_trimmed_per_line() {
    let l = lines(&["cmd  ", "out\t", ""]);
    assert_eq!(tail_join(&l, 10), "cmd\nout");
}

#[test]
fn leading_and_trailing_blank_lines_are_dropped() {
    let l = lines(&["", "  ", "body", "\t", ""]);
    assert_eq!(tail_join(&l, 10), "body");
    // 中间的空行保留。
    let l = lines(&["a", "", "b"]);
    assert_eq!(tail_join(&l, 10), "a\n\nb");
}

#[test]
fn all_blank_or_empty_returns_empty() {
    assert_eq!(tail_join(&[], 5), "");
    assert_eq!(tail_join(&lines(&["", "  ", "\t"]), 5), "");
}

#[test]
fn window_cut_then_blank_strip() {
    // 先截窗再掐空行：窗口里的前导空行也要掐掉（原版同序）。
    let l = lines(&["old", "", "new", ""]);
    assert_eq!(tail_join(&l, 3), "new");
}
