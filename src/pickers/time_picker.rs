//! 时间选择：输入族触发框 + antd 时/分/秒三列浮层（Now / OK footer）。
//!
//! 值与展开状态受控，浮层复用 popup 基建。列不自动滚动到选中项（偏差，见 PRD）。

use std::{
    cell::Cell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use gpui::{
    App, CursorStyle, Div, ElementId, IntoElement, Role, SharedString, Stateful, Window, canvas,
    div, prelude::*, px,
};

use crate::a11y;
use crate::controls::scrollbar::VerticalScroll;
use crate::svg_path::{paint_svg, svg_segments};
use crate::text_input::{self, InputSize, InputStatus, InputVariant};
use crate::theme::metric::{time_cell_state, time_picker_metric, time_picker_style};
use crate::{Button, popup::anchored, theme};

const CLOCK_RING: &str = "M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64zm0 820c-205.4 0-372-166.6-372-372s166.6-372 372-372 372 166.6 372 372-166.6 372-372 372z";
const CLOCK_HAND: &str = "M686.7 638.6L544.1 535.5V288c0-4.4-3.6-8-8-8H488c-4.4 0-8 3.6-8 8v275.4c0 2.6 1.2 5 3.3 6.5l165.4 120.6c3.6 2.6 8.6 1.8 11.2-1.7l28.6-39c2.6-3.7 1.8-8.7-1.8-11.2z";
const CLEAR_ICON: &str = "M512 64c247.4 0 448 200.6 448 448S759.4 960 512 960 64 759.4 64 512 264.6 64 512 64zm127.98 274.82h-.04l-.08.06L512 466.75 384.14 338.88c-.04-.05-.06-.06-.08-.06a.12.12 0 00-.07 0c-.03 0-.05.01-.09.05l-45.02 45.02a.2.2 0 00-.05.09.12.12 0 000 .07v.02a.27.27 0 00.06.06L466.75 512 338.88 639.86c-.05.04-.06.06-.06.08a.12.12 0 000 .07c0 .03.01.05.05.09l45.02 45.02a.2.2 0 00.09.05.12.12 0 00.07 0c.02 0 .04-.01.08-.05L512 557.25l127.86 127.87c.04.04.06.05.08.05a.12.12 0 00.07 0c.03 0 .05-.01.09-.05l45.02-45.02a.2.2 0 00.05-.09.12.12 0 000-.07v-.02a.27.27 0 00-.05-.06L557.25 512l127.87-127.86c.04-.04.05-.06.05-.08a.12.12 0 000-.07c0-.03-.01-.05-.05-.09l-45.02-45.02a.2.2 0 00-.09-.05.12.12 0 00-.07 0z";
const ICON_VIEW: (f32, f32, f32, f32) = (0.0, 0.0, 1024.0, 1024.0);

/// 时间选择器。
///
/// ```ignore
/// TimePicker::new("time")
///     .value(self.time_value)
///     .open(self.time_open)
///     .on_open_change(cx.listener(|this, open, _, cx| { this.time_open = *open; cx.notify(); }))
///     .on_change(cx.listener(|this, time, _, cx| { this.time_value = Some(*time); cx.notify(); }))
/// ```
pub struct TimePicker {
    id: ElementId,
    value: Option<(u32, u32, u32)>,
    open: bool,
    disabled: bool,
    allow_clear: bool,
    placeholder: SharedString,
    size: InputSize,
    variant: InputVariant,
    status: InputStatus,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    on_change: Option<Rc<dyn Fn(&(u32, u32, u32), &mut Window, &mut App)>>,
}

impl TimePicker {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            open: false,
            disabled: false,
            allow_clear: false,
            placeholder: "选择时间".into(),
            size: InputSize::Middle,
            variant: InputVariant::Outlined,
            status: InputStatus::Default,
            on_open_change: None,
            on_change: None,
        }
    }

    /// 选中时间（时, 分, 秒，受控）
    pub fn value(mut self, value: Option<(u32, u32, u32)>) -> Self {
        self.value = value;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = InputSize::Small;
        self
    }

    pub fn large(mut self) -> Self {
        self.size = InputSize::Large;
        self
    }

    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn status(mut self, status: InputStatus) -> Self {
        self.status = status;
        self
    }

    /// 有值时显示清空按钮（默认 false）
    pub fn allow_clear(mut self, allow_clear: bool) -> Self {
        self.allow_clear = allow_clear;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn on_open_change(
        mut self,
        listener: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(listener));
        self
    }

    pub fn on_change(
        mut self,
        listener: impl Fn(&(u32, u32, u32), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }
}

fn clock_icon(size: f32, color: gpui::Hsla) -> impl IntoElement {
    let mut segs = svg_segments(CLOCK_RING);
    segs.extend(svg_segments(CLOCK_HAND));
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, ICON_VIEW, &segs, color, window),
    )
    .w(px(size))
    .h(px(size))
}

fn clear_icon(size: f32, color: gpui::Hsla) -> impl IntoElement {
    let segs = svg_segments(CLEAR_ICON);
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, ICON_VIEW, &segs, color, window),
    )
    .w(px(size))
    .h(px(size))
}

impl IntoElement for TimePicker {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let open = self.open && !self.disabled;
        let suppress_toggle = Rc::new(Cell::new(false));
        let m = theme::with(time_picker_metric);
        let ps = theme::with(time_picker_style);
        let style = theme::with(|t| {
            text_input::state(
                self.variant,
                self.status,
                self.size,
                t,
                false,
                false,
                self.disabled,
            )
        });
        let hover_style = theme::with(|t| {
            text_input::state(
                self.variant,
                self.status,
                self.size,
                t,
                true,
                false,
                self.disabled,
            )
        });

        let label = match self.value {
            Some((h, mo, s)) => format!("{h:02}:{mo:02}:{s:02}"),
            None => self.placeholder.to_string(),
        };
        let show_clear = self.allow_clear && self.value.is_some() && !self.disabled;
        let mut trigger = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(style.height))
            .px(px(style.padding_inline))
            .py(px(style.padding_block))
            .rounded(px(style.radius))
            .border_1()
            .border_color(style.border_color())
            .bg(style.bg)
            .text_size(px(style.font_size))
            .line_height(px(style.line_height))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_color(if self.value.is_some() {
                        style.fg
                    } else {
                        style.placeholder
                    })
                    .child(label.clone()),
            );

        if show_clear {
            let on_change = self.on_change.clone();
            let clear = a11y::Semantics::new(Role::Button, "Clear time")
                .id(format!("{:?}-clear", self.id))
                .focusable(true)
                .apply(
                    div()
                        .id(ElementId::Name(format!("{:?}-clear", self.id).into()))
                        .ml(px(m.suffix_gap))
                        .cursor(CursorStyle::PointingHand)
                        .child(clear_icon(m.icon_size, ps.suffix))
                        .hover(|s| s.opacity(0.7)),
                );
            trigger = trigger.child(a11y::click_void(clear, move |window, cx| {
                cx.stop_propagation();
                if let Some(on_change) = &on_change {
                    on_change(&(0, 0, 0), window, cx);
                }
            }));
        } else {
            trigger = trigger.child(
                div()
                    .ml(px(m.suffix_gap))
                    .child(clock_icon(m.icon_size, ps.suffix)),
            );
        }

        if !self.disabled {
            let border = hover_style.border_color();
            trigger = trigger
                .cursor(CursorStyle::PointingHand)
                .hover(move |s| s.border_color(border));
        } else {
            trigger = trigger.cursor(CursorStyle::Arrow);
        }
        let mut semantics = a11y::Semantics::new(Role::ComboBox, "Time")
            .id(format!("{:?}-trigger", self.id))
            .expanded(open)
            .disabled(self.disabled)
            .focusable(!self.disabled);
        semantics = if self.value.is_some() {
            semantics.value(label.clone())
        } else {
            semantics.placeholder(self.placeholder.clone())
        };
        let mut trigger = semantics.apply(trigger);
        if !self.disabled {
            let on_open_change = self.on_open_change.clone();
            let suppress_toggle = suppress_toggle.clone();
            trigger = a11y::click_void(trigger, move |window, cx| {
                if suppress_toggle.take() {
                    return;
                }
                if let Some(on_open_change) = &on_open_change {
                    on_open_change(&!open, window, cx);
                }
            });
        }

        let popup = if open {
            Some(div().child(self.columns(&suppress_toggle)))
        } else {
            None
        };
        anchored(trigger.into_any_element(), popup)
    }
}

impl TimePicker {
    /// 时 / 分 / 秒三列 + Now / OK footer
    fn columns(&self, suppress_toggle: &Rc<Cell<bool>>) -> Stateful<Div> {
        let m = theme::with(time_picker_metric);
        let ps = theme::with(time_picker_style);
        let (h, mi, s) = self.value.unwrap_or((0, 0, 0));
        let shell = div()
            .id(ElementId::Name(format!("{:?}-cols", self.id).into()))
            .occlude()
            .flex()
            .flex_col()
            .rounded(px(m.radius))
            .bg(ps.bg)
            .shadow(ps.shadow.clone())
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
        let mut shell = a11y::Semantics::new(Role::Dialog, "Time picker")
            .id(format!("{:?}-cols", self.id))
            .apply(shell);

        let mut columns = div().flex().flex_row();
        for (part, part_label, count, current) in [
            (0u8, "Hours", 24u32, h),
            (1, "Minutes", 60, mi),
            (2, "Seconds", 60, s),
        ] {
            let scroll_id = format!("time-picker-{:?}-{part}", self.id);
            let column = div()
                .id(ElementId::Name(
                    format!("{:?}-col-{}", self.id, part).into(),
                ))
                .flex()
                .flex_row()
                .w(px(m.column_width))
                .h(px(m.column_height))
                .my(px(m.column_margin_v));
            let mut column = a11y::Semantics::new(Role::ListBox, part_label)
                .id(format!("{:?}-col-{}", self.id, part))
                .value(format!("{current:02}"))
                .apply(column);
            if part > 0 {
                column = column.border_l_1().border_color(ps.split);
            }
            let mut list = div().flex().flex_col().flex_1().min_w(px(0.)).min_h(px(0.));

            for n in 0..count {
                let cs = theme::with(|t| time_cell_state(t, n == current));
                let mut inner = div()
                    .w(px(m.inner_width))
                    .h(px(m.cell_height))
                    .mx(px(m.cell_margin_h))
                    .pl(px(m.inner_pad_start))
                    .flex()
                    .items_center()
                    .rounded(px(m.cell_radius))
                    .text_size(px(m.font_size))
                    .line_height(px(m.cell_height))
                    .text_color(cs.fg)
                    .child(format!("{n:02}"));
                if let Some(bg) = cs.bg {
                    inner = inner.bg(bg);
                } else {
                    let hover_bg = ps.hover_bg;
                    inner = inner.hover(move |st| st.bg(hover_bg));
                }
                let on_change = self.on_change.clone();
                let suppress = suppress_toggle.clone();
                let value = format!("{n:02}");
                let item =
                    a11y::Semantics::new(Role::ListBoxOption, format!("{part_label} {value}"))
                        .id(format!("{:?}-{}-{}", self.id, part, n))
                        .value(value)
                        .selected(n == current)
                        .position(n as usize + 1, count as usize)
                        .focusable(true)
                        .apply(
                            div()
                                .id(ElementId::Name(
                                    format!("{:?}-{}-{}", self.id, part, n).into(),
                                ))
                                .cursor(CursorStyle::PointingHand)
                                .child(inner),
                        );
                list = list.child(a11y::click_void(item, move |window, cx| {
                    suppress.set(true);
                    let next = match part {
                        0 => (n, mi, s),
                        1 => (h, n, s),
                        _ => (h, mi, n),
                    };
                    if let Some(on_change) = &on_change {
                        on_change(&next, window, cx);
                    }
                }));
            }
            column = column.child(VerticalScroll::new(
                ElementId::Name(format!("{scroll_id}-scroll").into()),
                list.into_any_element(),
            ));
            columns = columns.child(column);
        }
        shell = shell.child(columns);

        // footer：Now 链接 + OK 按钮
        let on_change = self.on_change.clone();
        let on_open_change = self.on_open_change.clone();
        let mut footer = div()
            .flex()
            .flex_row()
            .items_center()
            .border_t_1()
            .border_color(ps.split)
            .px(px(m.footer_pad_h))
            .child({
                let now = a11y::Semantics::new(Role::Button, "Now")
                    .id(format!("{:?}-now", self.id))
                    .focusable(true)
                    .apply(
                        div()
                            .id(ElementId::Name(format!("{:?}-now", self.id).into()))
                            .line_height(px(m.footer_line))
                            .text_size(px(m.font_size))
                            .text_color(ps.link)
                            .cursor(CursorStyle::PointingHand)
                            .child("Now"),
                    );
                a11y::click_void(now, move |window, cx| {
                    if let Some(on_change) = &on_change {
                        on_change(&now_utc(), window, cx);
                    }
                })
            });
        let on_open_change2 = on_open_change.clone();
        footer = footer.child(
            div().ml_auto().py(px(m.ok_pad_block)).child(
                Button::new("OK")
                    .id(format!("{:?}-ok", self.id))
                    .small()
                    .primary()
                    .on_click(move |_, window, cx| {
                        if let Some(on_open_change) = &on_open_change2 {
                            on_open_change(&false, window, cx);
                        }
                    }),
            ),
        );
        shell.child(footer)
    }
}

/// 当前 UTC 时分秒（antd Now 为本地时间，此处偏差记 PRD）
fn now_utc() -> (u32, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let day = secs % 86400;
    (
        (day / 3600) as u32,
        (day % 3600 / 60) as u32,
        (day % 60) as u32,
    )
}
