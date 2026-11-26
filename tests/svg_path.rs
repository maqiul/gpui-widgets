//! SVG path 解析：隐式重复、相对命令、arc 单字符 flag、平滑三次反射、闭合回起点。

use gpui::Point;
use gpui_widgets::svg_path::{Seg, svg_segments};

#[test]
fn implicit_lineto_after_moveto() {
    let segs = svg_segments("M0 0 10 10 20 20");
    assert_eq!(
        segs,
        vec![
            Seg::Move(Point::new(0., 0.)),
            Seg::Line(Point::new(10., 10.)),
            Seg::Line(Point::new(20., 20.)),
        ]
    );
}

#[test]
fn arc_flags_without_separators() {
    // antd StarFilled 里的 "a32.05 32.05 0 00.6 45.3"：flag 两位连写 00，dx=.6
    let segs = svg_segments("M0 0l10 0a5 5 0 00.6 45.3z");
    assert_eq!(segs[0], Seg::Move(Point::new(0., 0.)));
    assert_eq!(segs[1], Seg::Line(Point::new(10., 0.)));
    assert_eq!(
        segs[2],
        Seg::Arc(Point::new(5., 5.), 0., false, false, Point::new(10.6, 45.3))
    );
    assert_eq!(segs[3], Seg::Close);
}

#[test]
fn smooth_cubic_reflects_previous_control() {
    let segs = svg_segments("M0 0C1 1 2 2 3 3S5 5 6 6");
    assert_eq!(
        segs[2],
        Seg::Cubic(Point::new(4., 4.), Point::new(5., 5.), Point::new(6., 6.))
    );
}

#[test]
fn close_returns_to_subpath_start() {
    let segs = svg_segments("M10 10L20 20ZL30 30");
    assert_eq!(segs[2], Seg::Close);
    assert_eq!(segs[3], Seg::Line(Point::new(30., 30.)));
}

#[test]
fn star_path_is_closed_and_bounded() {
    let d = "M908.1 353.1l-253.9-36.9L540.7 86.1c-3.1-6.3-8.2-11.4-14.5-14.5-15.8-7.8-35-1.3-42.9 14.5L369.8 316.2l-253.9 36.9c-7 1-13.4 4.3-18.3 9.3a32.05 32.05 0 00.6 45.3l183.7 179.1-43.4 252.9a31.95 31.95 0 0046.4 33.7L512 754l227.1 119.4c6.2 3.3 13.4 4.4 20.3 3.2 17.4-3 29.1-19.5 26.1-36.9l-43.4-252.9 183.7-179.1c5-4.9 8.3-11.3 9.3-18.3 2.7-17.5-9.5-33.7-27-36.3z";
    let segs = svg_segments(d);
    assert_eq!(segs.first(), Some(&Seg::Move(Point::new(908.1, 353.1))));
    assert_eq!(segs.last(), Some(&Seg::Close));
    assert!(segs.iter().any(|s| matches!(s, Seg::Arc(..))));
    for seg in &segs {
        let pts: Vec<Point<f32>> = match *seg {
            Seg::Move(p) | Seg::Line(p) => vec![p],
            Seg::Cubic(a, b, c) => vec![a, b, c],
            Seg::Quad(a, b) => vec![a, b],
            Seg::Arc(_, _, _, _, p) => vec![p],
            Seg::Close => vec![],
        };
        for p in pts {
            assert!(
                (0. ..=1024.).contains(&p.x) && (0. ..=1024.).contains(&p.y),
                "越界 {p:?}"
            );
        }
    }
}
