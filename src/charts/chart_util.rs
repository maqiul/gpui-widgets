//! 图表自绘共享 helper：极坐标、扇环 / 圆弧路径、颜色插值、文字绘制。
//!
//! 供 pie_chart / gauge / scatter / radar / funnel 等高级图表复用，
//! 全部基于 GPUI 原生 `PathBuilder` + `window.paint_path`，无第三方依赖。

use std::f32::consts::TAU;

use gpui::{App, Hsla, PathBuilder, Pixels, Point, SharedString, Window, point, px};

/// 极坐标转直角坐标（角度为弧度，屏幕坐标 y 向下，0 指向右、顺时针增大）。
pub(crate) fn polar(center: Point<Pixels>, radius: Pixels, angle: f32) -> Point<Pixels> {
    point(
        center.x + radius * angle.cos(),
        center.y + radius * angle.sin(),
    )
}

/// 扇环填充路径（饼图 / 环图切片）：`r_inner <= 0` 时为实心扇形。
/// 角度从 `a0` 到 `a1`，外弧正向、内弧反向闭合，背景无关（真环 segments）。
pub(crate) fn sector(
    center: Point<Pixels>,
    r_outer: Pixels,
    r_inner: Pixels,
    a0: f32,
    a1: f32,
) -> PathBuilder {
    let mut p = PathBuilder::fill();
    let sweep = (a1 - a0).abs();
    let segments = ((sweep / TAU) * 96.).ceil().max(2.) as usize;
    for i in 0..=segments {
        let a = a0 + (a1 - a0) * (i as f32 / segments as f32);
        let q = polar(center, r_outer, a);
        if i == 0 {
            p.move_to(q);
        } else {
            p.line_to(q);
        }
    }
    if r_inner > px(0.) {
        for i in 0..=segments {
            let a = a1 - (a1 - a0) * (i as f32 / segments as f32);
            p.line_to(polar(center, r_inner, a));
        }
    } else {
        p.line_to(center);
    }
    p.close();
    p
}

/// 圆弧描边路径（仪表盘 / 环形轨道），多段折线逼近。
pub(crate) fn arc(
    center: Point<Pixels>,
    radius: Pixels,
    a0: f32,
    a1: f32,
    thickness: Pixels,
) -> PathBuilder {
    let mut p = PathBuilder::stroke(thickness);
    let sweep = (a1 - a0).abs();
    let segments = ((sweep / TAU) * 96.).ceil().max(2.) as usize;
    for i in 0..=segments {
        let a = a0 + (a1 - a0) * (i as f32 / segments as f32);
        let q = polar(center, radius, a);
        if i == 0 {
            p.move_to(q);
        } else {
            p.line_to(q);
        }
    }
    p
}

/// 实心圆填充路径（散点 / 仪表盘轴心）。
pub(crate) fn circle(center: Point<Pixels>, radius: Pixels) -> PathBuilder {
    let mut p = PathBuilder::fill();
    let segments = 32;
    for i in 0..=segments {
        let a = TAU * i as f32 / segments as f32;
        let q = polar(center, radius, a);
        if i == 0 {
            p.move_to(q);
        } else {
            p.line_to(q);
        }
    }
    p.close();
    p
}

/// 颜色线性插值（Hsla 各分量独立插值，t 夹取到 0~1）。
pub(crate) fn lerp_color(a: Hsla, b: Hsla, t: f32) -> Hsla {
    let t = t.clamp(0., 1.);
    Hsla {
        h: a.h + (b.h - a.h) * t,
        s: a.s + (b.s - a.s) * t,
        l: a.l + (b.l - a.l) * t,
        a: a.a + (b.a - a.a) * t,
    }
}

/// 在指定原点（左上角）绘制单行文字，返回文字宽度。
pub(crate) fn paint_text(
    window: &mut Window,
    cx: &mut App,
    text: impl Into<SharedString>,
    origin: Point<Pixels>,
    font_size: Pixels,
    color: Hsla,
) -> Pixels {
    let text = text.into();
    let style = window.text_style();
    let run = gpui::TextRun {
        len: text.len(),
        font: style.font(),
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window
        .text_system()
        .shape_line(text, font_size, &[run], None);
    let width = line.width;
    let _ = line.paint(
        origin,
        font_size * 1.4,
        gpui::TextAlign::default(),
        None,
        window,
        cx,
    );
    width
}

/// 以 center 为中心绘制单行文字（水平垂直居中）。
pub(crate) fn paint_text_centered(
    window: &mut Window,
    cx: &mut App,
    text: impl Into<SharedString>,
    center: Point<Pixels>,
    font_size: Pixels,
    color: Hsla,
) {
    let text = text.into();
    let style = window.text_style();
    let run = gpui::TextRun {
        len: text.len(),
        font: style.font(),
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = window
        .text_system()
        .shape_line(text, font_size, &[run], None);
    let origin = point(center.x - line.width / 2., center.y - font_size * 0.7);
    let _ = line.paint(
        origin,
        font_size * 1.4,
        gpui::TextAlign::default(),
        None,
        window,
        cx,
    );
}

/// 数值格式化：整数不带小数点，否则保留一位小数。
pub(crate) fn format_value(value: f32) -> String {
    if (value - value.round()).abs() < 1e-6 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.1}")
    }
}
