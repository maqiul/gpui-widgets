//! 颜色选择器：antd ColorPicker 触发框（色块 + 棋盘底）与预设色浮层（选中勾）。

use std::{cell::Cell, rc::Rc};

use gpui::{
    App, BoxShadow, CursorStyle, Div, ElementId, IntoElement, Rgba, Role, Window, canvas, div,
    point, prelude::*, px, rgb,
};

use crate::a11y;
use crate::popup::anchored;
use crate::svg_path::{Seg, paint_svg_stroke};
use crate::theme;
use crate::theme::metric::{
    ColorPickerStyle, color_check_points, color_picker_metric, color_picker_style,
    color_preset_bright,
};

/// 预设色板（8 列 × 3 行）
const PALETTE: [u32; 24] = [
    0x1f2329, 0x8a919f, 0xe2e5e9, 0xffffff, 0xef4444, 0xf97316, 0xf59e0b, 0xeab308, 0x84cc16,
    0x22c55e, 0x14b8a6, 0x06b6d4, 0x3b82f6, 0x6366f1, 0x8b5cf6, 0xd946ef, 0xec4899, 0xf43f5e,
    0x78716c, 0x0ea5e9, 0x10b981, 0x8b5a2b, 0x64748b, 0x0f172a,
];

/// 颜色选择器。
///
/// ```ignore
/// ColorPicker::new("brand", self.color)
///     .open(self.color_open)
///     .on_open_change(cx.listener(|this, open, _, cx| { this.color_open = *open; cx.notify(); }))
///     .on_change(cx.listener(|this, color, _, cx| { this.color = *color; cx.notify(); }))
/// ```
pub struct ColorPicker {
    id: ElementId,
    value: Rgba,
    open: bool,
    small: bool,
    large: bool,
    disabled: bool,
    on_change: Option<Rc<dyn Fn(&Rgba, &mut Window, &mut App)>>,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
}

impl ColorPicker {
    pub fn new(id: impl Into<ElementId>, value: Rgba) -> Self {
        Self {
            id: id.into(),
            value,
            open: false,
            small: false,
            large: false,
            disabled: false,
            on_change: None,
            on_open_change: None,
        }
    }

    /// 当前是否展开（受控）
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    pub fn large(mut self) -> Self {
        self.large = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&Rgba, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    pub fn on_open_change(
        mut self,
        listener: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(listener));
        self
    }
}

/// Rgba 转 0xRRGGBB（忽略透明度）
fn hex_of(color: Rgba) -> u32 {
    let r = (color.r * 255.).round() as u32;
    let g = (color.g * 255.).round() as u32;
    let b = (color.b * 255.).round() as u32;
    (r << 16) | (g << 8) | b
}

/// antd ColorBlock：棋盘透明底 + 内描边 + 色值（gpui 无 inset shadow，外圈 inset 色用边框近似）
fn color_block(size: f32, radius: f32, value: Rgba, style: &ColorPickerStyle) -> Div {
    let inner = style.inner;
    div()
        .relative()
        .w(px(size))
        .h(px(size))
        .rounded(px(radius))
        .border_1()
        .border_color(style.inset)
        .overflow_hidden()
        .child(
            canvas(
                |bounds, _, _| bounds,
                move |bounds, _, window, _| {
                    let tile = bounds.size.width / 2.;
                    for (ox, oy) in [(0., 0.), (1., 1.)] {
                        window.paint_quad(gpui::fill(
                            gpui::Bounds::new(
                                point(bounds.origin.x + tile * ox, bounds.origin.y + tile * oy),
                                gpui::size(tile, tile),
                            ),
                            inner,
                        ));
                    }
                },
            )
            .absolute()
            .inset_0(),
        )
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded(px(radius))
                .border_1()
                .border_color(style.inner)
                .bg(value),
        )
}

impl IntoElement for ColorPicker {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let m = theme::with(color_picker_metric);
        let style = theme::with(color_picker_style);
        let (shadow, popup_radius) =
            theme::with(|t| (t.shadow_secondary.0.clone(), t.border_radius_lg));
        let open = self.open && !self.disabled;
        let suppress_toggle = Rc::new(Cell::new(false));
        let hex = hex_of(self.value);
        let value_label = format!("#{hex:06X}");

        let (trigger_size, trigger_radius, block_size, block_radius) = if self.small {
            (
                m.trigger_size_sm,
                m.trigger_radius_sm,
                m.block_size_sm,
                m.block_radius_sm,
            )
        } else if self.large {
            (
                m.trigger_size_lg,
                m.trigger_radius_lg,
                m.block_size_lg,
                m.block_radius_lg,
            )
        } else {
            (
                m.trigger_size,
                m.trigger_radius,
                m.block_size,
                m.block_radius,
            )
        };

        let border = if self.disabled {
            style.border
        } else if open {
            style.active
        } else {
            style.border
        };
        let mut trigger = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .w(px(trigger_size))
            .h(px(trigger_size))
            .rounded(px(trigger_radius))
            .border_1()
            .border_color(border)
            .bg(if self.disabled {
                style.disabled_bg
            } else {
                style.trigger_bg
            })
            .p(px(m.trigger_padding))
            .child(color_block(block_size, block_radius, self.value, &style))
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::PointingHand
            });
        if !self.disabled {
            trigger = trigger.hover(|s| s.border_color(style.hover));
        }
        if open {
            trigger = trigger.shadow(vec![BoxShadow {
                color: style.outline,
                offset: point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(style.outline_width),
                inset: false,
            }]);
        }
        let mut trigger = a11y::Semantics::new(Role::Button, "Color")
            .id(format!("{:?}-trigger", self.id))
            .value(value_label.clone())
            .expanded(open)
            .disabled(self.disabled)
            .focusable(!self.disabled)
            .apply(trigger);

        if !self.disabled {
            let on_open_change = self.on_open_change.clone();
            let trigger_suppress = suppress_toggle.clone();
            trigger = a11y::click_void(trigger, move |window, cx| {
                if trigger_suppress.take() {
                    return;
                }
                if let Some(on_open_change) = &on_open_change {
                    on_open_change(&!open, window, cx);
                }
            });
        }

        let mut popup = None;
        if open {
            let shell = div()
                .id(ElementId::Name(format!("{:?}-palette", self.id).into()))
                .occlude()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(px(m.preset_gap))
                .w(px(m.panel_width + m.panel_padding * 2.))
                .p(px(m.panel_padding))
                .rounded(px(popup_radius))
                .bg(style.trigger_bg)
                .shadow(shadow)
                .on_mouse_down_out({
                    let on_open_change = self.on_open_change.clone();
                    let suppress_toggle = suppress_toggle.clone();
                    move |_, window, cx| {
                        suppress_toggle.set(true);
                        if let Some(on_open_change) = &on_open_change {
                            on_open_change(&false, window, cx);
                        }
                    }
                });
            let mut shell = a11y::Semantics::new(Role::Grid, "Preset colors")
                .id(format!("{:?}-palette", self.id))
                .apply(shell);

            let elevated = style.trigger_bg;
            for (ix, preset) in PALETTE.iter().enumerate() {
                let is_selected = hex == *preset;
                let color: Rgba = rgb(*preset);
                let bright = color_preset_bright(
                    color.r * 255.,
                    color.g * 255.,
                    color.b * 255.,
                    color.a,
                    elevated,
                );
                let check_color = if bright {
                    style.check_bright
                } else {
                    style.check
                };
                let stroke = m.check_stroke;
                let mut cell = div()
                    .relative()
                    .w(px(m.preset_size))
                    .h(px(m.preset_size))
                    .child(
                        div()
                            .absolute()
                            .top(px(-2.))
                            .left(px(-2.))
                            .w(px(m.preset_size + 4.))
                            .h(px(m.preset_size + 4.))
                            .rounded(px(m.trigger_radius))
                            .border_1()
                            .border_color(gpui::transparent_black())
                            .hover(|s| s.border_color(style.preset_hover)),
                    )
                    .child(color_block(m.preset_size, m.block_radius, color, &style));
                if is_selected {
                    cell = cell.child(
                        canvas(
                            |bounds, _, _| bounds,
                            move |bounds, _, window, _| {
                                let pts = color_check_points(f32::from(bounds.size.width));
                                let segs: Vec<Seg> = pts
                                    .iter()
                                    .enumerate()
                                    .map(|(i, p)| {
                                        let pt = gpui::Point::new(p.0, p.1);
                                        if i == 0 { Seg::Move(pt) } else { Seg::Line(pt) }
                                    })
                                    .collect();
                                paint_svg_stroke(
                                    bounds,
                                    (
                                        0.,
                                        0.,
                                        f32::from(bounds.size.width),
                                        f32::from(bounds.size.height),
                                    ),
                                    &segs,
                                    check_color,
                                    stroke,
                                    window,
                                );
                            },
                        )
                        .absolute()
                        .inset_0(),
                    );
                }

                let on_change = self.on_change.clone();
                let on_open_change = self.on_open_change.clone();
                let color_label = format!("#{:06X}", *preset);
                let cell = a11y::Semantics::new(Role::RadioButton, color_label.clone())
                    .id(format!("{:?}-c-{ix}", self.id))
                    .value(color_label)
                    .selected(is_selected)
                    .position(ix + 1, PALETTE.len())
                    .row(ix / 8 + 1, 3)
                    .column(ix % 8 + 1, 8)
                    .disabled(self.disabled)
                    .focusable(!self.disabled)
                    .apply(
                        div()
                            .id(ElementId::Name(format!("{:?}-c-{ix}", self.id).into()))
                            .cursor(CursorStyle::PointingHand)
                            .child(cell),
                    );
                shell = shell.child(a11y::click_void(cell, move |window, cx| {
                    if let Some(on_change) = &on_change {
                        on_change(&color, window, cx);
                    }
                    if let Some(on_open_change) = &on_open_change {
                        on_open_change(&false, window, cx);
                    }
                }));
            }

            popup = Some(div().child(shell));
        }

        anchored(trigger.into_any_element(), popup)
    }
}
