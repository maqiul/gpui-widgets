//! 滚动条取色旋钮（0.2.5 加性）：滑块默认仍是 antd 实测回退 `rgba(0,0,0,0.5)`，
//! 悬停轨道默认仍是 `colorFillQuaternary`。全局覆盖用例在单个测试函数内顺序断言，
//! thumb / track 各占一个函数，避免并行用例串扰（同 tests/mono_family.rs）。

use gpui::{Hsla, rgba};
use gpui_widgets::controls::scrollbar::resolve_layered_color;
use gpui_widgets::theme;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rgba8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

fn rgba8(color: Hsla) -> Rgba8 {
    let c = theme::Color::from(color);
    let byte = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    Rgba8 {
        r: byte(c.r),
        g: byte(c.g),
        b: byte(c.b),
        a: byte(c.a * 255.0),
    }
}

/// 纯函数优先级矩阵：元素级 > 全局 > 主题默认，与全局态无关。
#[test]
fn scrollbar_color_precedence_matrix() {
    let default: Hsla = rgba(0x00000080).into();
    let global: Hsla = rgba(0x11223340).into();
    let element: Hsla = rgba(0xff000080).into();

    assert_eq!(
        resolve_layered_color(None, None, default),
        default,
        "无覆写取主题默认"
    );
    assert_eq!(
        resolve_layered_color(None, Some(global), default),
        global,
        "全局生效"
    );
    assert_eq!(
        resolve_layered_color(Some(element), Some(global), default),
        element,
        "元素级压过全局"
    );
    assert_eq!(
        resolve_layered_color(Some(element), None, default),
        element,
        "元素级压过默认"
    );

    // 只改 alpha 的透明度调整形态：色相不变，alpha 落到 26/255
    let dimmed: Hsla = rgba(0x0000001a).into();
    assert_eq!(resolve_layered_color(None, Some(dimmed), default), dimmed);
    assert_eq!(
        rgba8(resolve_layered_color(None, Some(dimmed), default)),
        Rgba8 {
            r: 0,
            g: 0,
            b: 0,
            a: 26
        },
        "alpha-only override"
    );
}

/// 滑块：默认零变化 + 全局覆盖 roundtrip。
#[test]
fn scrollbar_thumb_override_roundtrip() {
    let antd_fallback = Rgba8 {
        r: 0,
        g: 0,
        b: 0,
        a: 128,
    };
    let metric_default = theme::with(|t| theme::metric::scrollbar_metric(t).thumb_color);
    assert_eq!(
        rgba8(metric_default),
        antd_fallback,
        "scrollbar_metric 仍返回 antd 回退黑 50%"
    );
    assert!(
        theme::scrollbar_thumb_override().is_none(),
        "默认无全局覆盖"
    );
    assert_eq!(
        rgba8(theme::scrollbar_thumb_or(metric_default)),
        antd_fallback
    );

    let dimmed: Hsla = rgba(0x00000033).into();
    theme::set_scrollbar_thumb(Some(dimmed));
    assert_eq!(theme::scrollbar_thumb_override(), Some(dimmed), "覆盖可读");
    assert_eq!(
        rgba8(theme::scrollbar_thumb_or(metric_default)),
        rgba8(dimmed)
    );
    // 元素级仍压过全局
    assert_eq!(
        resolve_layered_color(
            Some(metric_default),
            theme::scrollbar_thumb_override(),
            metric_default
        ),
        metric_default,
        "element beats global"
    );

    theme::set_scrollbar_thumb(None);
    assert!(theme::scrollbar_thumb_override().is_none(), "覆盖已清除");
    assert_eq!(
        rgba8(theme::scrollbar_thumb_or(metric_default)),
        antd_fallback,
        "回到默认"
    );
}

/// 轨道：默认零变化（常态透明 + hover `colorFillQuaternary`）+ 全局覆盖 roundtrip。
#[test]
fn scrollbar_track_override_roundtrip() {
    let idle = theme::with(|t| theme::metric::scrollbar_track_bg(t, false));
    let hover = theme::with(|t| theme::metric::scrollbar_track_bg(t, true));
    assert_eq!(
        rgba8(idle),
        Rgba8 {
            r: 0,
            g: 0,
            b: 0,
            a: 0
        },
        "常态透明"
    );
    assert_eq!(
        rgba8(hover),
        rgba8(theme::with(|t| t.color_fill_quaternary)),
        "hover 仍取 colorFillQuaternary"
    );
    assert!(
        theme::scrollbar_track_override().is_none(),
        "默认无全局覆盖"
    );
    assert_eq!(theme::scrollbar_track_or(hover), hover);

    let tinted: Hsla = rgba(0x1677ff26).into();
    theme::set_scrollbar_track(Some(tinted));
    assert_eq!(theme::scrollbar_track_override(), Some(tinted), "覆盖可读");
    assert_eq!(theme::scrollbar_track_or(hover), tinted, "全局生效");
    // 覆写只换颜色，不改「常态不画轨道」的可见性语义
    assert_eq!(
        resolve_layered_color(None, theme::scrollbar_track_override(), idle),
        tinted
    );

    theme::set_scrollbar_track(None);
    assert!(theme::scrollbar_track_override().is_none(), "覆盖已清除");
    assert_eq!(theme::scrollbar_track_or(hover), hover, "回到令牌值");
}
