//! SVG path 绘制：把 antd 图标 / 插画的 path d 串按 viewBox 缩放后画到元素盒里。
//! 解析是纯函数（svg_segments），绘制走 gpui canvas + PathBuilder。
//!
//! 这些绘制函数不创建无障碍节点：图标与插画通常只是宿主控件的装饰，
//! 逐个 path 暴露会重复朗读。有意义的图形应由宿主元素提供一次语义与标签，
//! 纯装饰路径保持不在无障碍树中。

use gpui::{Bounds, Hsla, PathBuilder, Pixels, Point, Window, point, px};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Seg {
    Move(Point<f32>),
    Line(Point<f32>),
    Cubic(Point<f32>, Point<f32>, Point<f32>),
    Quad(Point<f32>, Point<f32>),
    Arc(Point<f32>, f32, bool, bool, Point<f32>),
    Close,
}

struct Lexer<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Lexer<'a> {
    fn skip_sep(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b',' | b'\t' | b'\n' | b'\r')
        {
            self.i += 1;
        }
    }

    fn peek_cmd(&mut self) -> Option<u8> {
        self.skip_sep();
        match self.s.get(self.i) {
            Some(c) if c.is_ascii_alphabetic() => Some(*c),
            _ => None,
        }
    }

    fn num(&mut self) -> f32 {
        self.skip_sep();
        let start = self.i;
        if self.i < self.s.len() && matches!(self.s[self.i], b'+' | b'-') {
            self.i += 1;
        }
        while self.i < self.s.len() && (self.s[self.i].is_ascii_digit() || self.s[self.i] == b'.') {
            self.i += 1;
        }
        if self.i < self.s.len() && matches!(self.s[self.i], b'e' | b'E') {
            self.i += 1;
            if self.i < self.s.len() && matches!(self.s[self.i], b'+' | b'-') {
                self.i += 1;
            }
            while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                self.i += 1;
            }
        }
        std::str::from_utf8(&self.s[start..self.i])
            .ok()
            .and_then(|t| t.parse().ok())
            .unwrap_or(0.)
    }

    fn flag(&mut self) -> bool {
        self.skip_sep();
        let f = self.s.get(self.i) == Some(&b'1');
        self.i += 1;
        f
    }
}

/// 解析 SVG path d 串（viewBox 坐标）。支持 M/L/H/V/C/S/Q/T/A/Z 及相对命令与隐式重复。
pub fn svg_segments(d: &str) -> Vec<Seg> {
    let mut lex = Lexer {
        s: d.as_bytes(),
        i: 0,
    };
    let mut out = Vec::new();
    let mut cur = Point::new(0., 0.);
    let mut start = cur;
    let mut last_cubic_ctrl: Option<Point<f32>> = None;
    let mut last_quad_ctrl: Option<Point<f32>> = None;
    let mut cmd = b'M';
    loop {
        if let Some(c) = lex.peek_cmd() {
            lex.i += 1;
            cmd = c;
        } else if lex.i >= lex.s.len() {
            break;
        }
        let rel = cmd.is_ascii_lowercase();
        let base = if rel { cur } else { Point::new(0., 0.) };
        let p = |x: f32, y: f32| Point::new(base.x + x, base.y + y);
        match cmd.to_ascii_uppercase() {
            b'M' | b'L' => {
                let pt = p(lex.num(), lex.num());
                if cmd.to_ascii_uppercase() == b'M' {
                    out.push(Seg::Move(pt));
                    start = pt;
                    cmd = if rel { b'l' } else { b'L' };
                } else {
                    out.push(Seg::Line(pt));
                }
                cur = pt;
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            b'H' => {
                let pt = Point::new(base.x + lex.num(), cur.y);
                out.push(Seg::Line(pt));
                cur = pt;
            }
            b'V' => {
                let pt = Point::new(cur.x, base.y + lex.num());
                out.push(Seg::Line(pt));
                cur = pt;
            }
            b'C' | b'S' => {
                let (c1, c2, to) = if cmd.to_ascii_uppercase() == b'C' {
                    (
                        p(lex.num(), lex.num()),
                        p(lex.num(), lex.num()),
                        p(lex.num(), lex.num()),
                    )
                } else {
                    let c1 = match last_cubic_ctrl {
                        Some(c) => Point::new(2. * cur.x - c.x, 2. * cur.y - c.y),
                        None => cur,
                    };
                    (c1, p(lex.num(), lex.num()), p(lex.num(), lex.num()))
                };
                out.push(Seg::Cubic(c1, c2, to));
                last_cubic_ctrl = Some(c2);
                last_quad_ctrl = None;
                cur = to;
            }
            b'Q' | b'T' => {
                let (c1, to) = if cmd.to_ascii_uppercase() == b'Q' {
                    (p(lex.num(), lex.num()), p(lex.num(), lex.num()))
                } else {
                    let c1 = match last_quad_ctrl {
                        Some(c) => Point::new(2. * cur.x - c.x, 2. * cur.y - c.y),
                        None => cur,
                    };
                    (c1, p(lex.num(), lex.num()))
                };
                out.push(Seg::Quad(c1, to));
                last_quad_ctrl = Some(c1);
                last_cubic_ctrl = None;
                cur = to;
            }
            b'A' => {
                let radii = Point::new(lex.num(), lex.num());
                let rot = lex.num();
                let large = lex.flag();
                let sweep = lex.flag();
                let to = p(lex.num(), lex.num());
                out.push(Seg::Arc(radii, rot, large, sweep, to));
                cur = to;
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            b'Z' => {
                out.push(Seg::Close);
                cur = start;
                last_cubic_ctrl = None;
                last_quad_ctrl = None;
            }
            _ => break,
        }
    }
    out
}

/// 平移 segments（SVG g transform=translate 等价）。
pub fn translate_segs(segs: &[Seg], dx: f32, dy: f32) -> Vec<Seg> {
    let t = |p: Point<f32>| Point::new(p.x + dx, p.y + dy);
    segs.iter()
        .map(|s| match *s {
            Seg::Move(p) => Seg::Move(t(p)),
            Seg::Line(p) => Seg::Line(t(p)),
            Seg::Cubic(a, b, c) => Seg::Cubic(t(a), t(b), t(c)),
            Seg::Quad(a, b) => Seg::Quad(t(a), t(b)),
            Seg::Arc(r, rot, large, sweep, p) => Seg::Arc(r, rot, large, sweep, t(p)),
            Seg::Close => Seg::Close,
        })
        .collect()
}

/// 椭圆 → 两段半圆弧（SVG ellipse 等价）。
pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Vec<Seg> {
    vec![
        Seg::Move(Point::new(cx - rx, cy)),
        Seg::Arc(Point::new(rx, ry), 0., true, true, Point::new(cx + rx, cy)),
        Seg::Arc(Point::new(rx, ry), 0., true, true, Point::new(cx - rx, cy)),
        Seg::Close,
    ]
}

/// 把 viewBox 坐标的 segments 缩放平移到元素盒并填充绘制。
pub fn paint_svg(
    bounds: Bounds<Pixels>,
    view_box: (f32, f32, f32, f32),
    segs: &[Seg],
    color: Hsla,
    window: &mut Window,
) {
    if let Some(path) = build_path(bounds, view_box, segs, None) {
        window.paint_path(path, color);
    }
}

/// 描边绘制：width 为 viewBox 单位线宽（随缩放换算）。
pub fn paint_svg_stroke(
    bounds: Bounds<Pixels>,
    view_box: (f32, f32, f32, f32),
    segs: &[Seg],
    color: Hsla,
    width: f32,
    window: &mut Window,
) {
    if let Some(path) = build_path(bounds, view_box, segs, Some(width)) {
        window.paint_path(path, color);
    }
}

fn build_path(
    bounds: Bounds<Pixels>,
    view_box: (f32, f32, f32, f32),
    segs: &[Seg],
    stroke: Option<f32>,
) -> Option<gpui::Path> {
    let (vx, vy, vw, vh) = view_box;
    let sx = f32::from(bounds.size.width) / vw;
    let sy = f32::from(bounds.size.height) / vh;
    let map = |p: Point<f32>| {
        point(
            px(f32::from(bounds.origin.x) + (p.x - vx) * sx),
            px(f32::from(bounds.origin.y) + (p.y - vy) * sy),
        )
    };
    let mut builder = match stroke {
        Some(w) => PathBuilder::stroke(px(w * sx)),
        None => PathBuilder::fill(),
    };
    for seg in segs {
        match *seg {
            Seg::Move(p) => builder.move_to(map(p)),
            Seg::Line(p) => builder.line_to(map(p)),
            Seg::Cubic(c1, c2, to) => builder.cubic_bezier_to(map(to), map(c1), map(c2)),
            Seg::Quad(c, to) => builder.curve_to(map(to), map(c)),
            Seg::Arc(r, rot, large, sweep, to) => builder.arc_to(
                point(px(r.x * sx), px(r.y * sy)),
                px(rot),
                large,
                sweep,
                map(to),
            ),
            Seg::Close => builder.close(),
        }
    }
    builder.build().ok()
}
