//! 日期范围选择器：输入族触发框 + antd 双月日历浮层，先点起点再点终点（早于起点则重选）。
//!
//! 值与展开状态受控；待选起点通过 `Window::use_keyed_state` 按 id 持久化。
//! 区间连续底 / 端点半侧底走 antd `cellActiveWithRangeBg` 令牌。

use std::{cell::Cell, rc::Rc};

use gpui::{
    App, CursorStyle, Div, ElementId, Entity, IntoElement, Role, SharedString, Stateful, Window,
    canvas, div, prelude::*, px,
};

use crate::a11y;
use crate::date_picker::{Date, days_in_month, today, weekday_monday0};
use crate::svg_path::{paint_svg, svg_segments};
use crate::text_input::{self, InputSize, InputStatus, InputVariant};
use crate::theme::metric::{
    RangeEnd, date_picker_metric, date_picker_style, range_cell_state, range_picker_metric,
};
use crate::{popup::anchored, theme};

const SWAP_ICON: &str = "M873.1 596.2l-164-208A32 32 0 00684 376h-64.8c-6.7 0-10.4 7.7-6.3 13l144.3 183H152c-4.4 0-8 3.6-8 8v60c0 4.4 3.6 8 8 8h695.9c26.8 0 41.7-30.8 25.2-51.8z";
const CALENDAR_ICON: &str = "M880 184H712v-64c0-4.4-3.6-8-8-8h-56c-4.4 0-8 3.6-8 8v64H384v-64c0-4.4-3.6-8-8-8h-56c-4.4 0-8 3.6-8 8v64H144c-17.7 0-32 14.3-32 32v664c0 17.7 14.3 32 32 32h736c17.7 0 32-14.3 32-32V216c0-17.7-14.3-32-32-32zm-40 656H184V460h656v380zM184 392V256h128v48c0 4.4 3.6 8 8 8h56c4.4 0 8-3.6 8-8v-48h256v48c0 4.4 3.6 8 8 8h56c4.4 0 8-3.6 8-8v-48h128v136H184z";
const LEFT_ICON: &str = "M724 218.3V141c0-6.7-7.7-10.4-12.9-6.3L260.3 486.8a31.86 31.86 0 000 50.3l450.8 352.1c5.3 4.1 12.9.4 12.9-6.3v-77.3c0-4.9-2.3-9.6-6.1-12.6l-360-281 360-281.1c3.8-3 6.1-7.7 6.1-12.6z";
const RIGHT_ICON: &str = "M765.7 486.8L314.9 134.7A7.97 7.97 0 00302 141v77.3c0 4.9 2.3 9.6 6.1 12.6l360 281.1-360 281.1c-3.9 3-6.1 7.7-6.1 12.6V883c0 6.7 7.7 10.4 12.9 6.3l450.8-352.1a31.96 31.96 0 000-50.4z";
const DOUBLE_LEFT_ICON: &str = "M272.9 512l265.4-339.1c4.1-5.2.4-12.9-6.3-12.9h-77.3c-4.9 0-9.6 2.3-12.6 6.1L186.8 492.3a31.99 31.99 0 000 39.5l255.3 326.1c3 3.9 7.7 6.1 12.6 6.1H532c6.7 0 10.4-7.7 6.3-12.9L272.9 512zm304 0l265.4-339.1c4.1-5.2.4-12.9-6.3-12.9h-77.3c-4.9 0-9.6 2.3-12.6 6.1L490.8 492.3a31.99 31.99 0 000 39.5l255.3 326.1c3 3.9 7.7 6.1 12.6 6.1H836c6.7 0 10.4-7.7 6.3-12.9L576.9 512z";
const DOUBLE_RIGHT_ICON: &str = "M533.2 492.3L277.9 166.1c-3-3.9-7.7-6.1-12.6-6.1H188c-6.7 0-10.4 7.7-6.3 12.9L447.1 512 181.7 851.1A7.98 7.98 0 00188 864h77.3c4.9 0 9.6-2.3 12.6-6.1l255.3-326.1c9.1-11.7 9.1-27.9 0-39.5zm304 0L581.9 166.1c-3-3.9-7.7-6.1-12.6-6.1H492c-6.7 0-10.4 7.7-6.3 12.9L751.1 512 485.7 851.1A7.98 7.98 0 00492 864h77.3c4.9 0 9.6-2.3 12.6-6.1l255.3-326.1c9.1-11.7 9.1-27.9 0-39.5z";
const CLEAR_ICON: &str = "M512 64c247.4 0 448 200.6 448 448S759.4 960 512 960 64 759.4 64 512 264.6 64 512 64zm127.98 274.82h-.04l-.08.06L512 466.75 384.14 338.88c-.04-.05-.06-.06-.08-.06a.12.12 0 00-.07 0c-.03 0-.05.01-.09.05l-45.02 45.02a.2.2 0 00-.05.09.12.12 0 000 .07v.02a.27.27 0 00.06.06L466.75 512 338.88 639.86c-.05.04-.06.06-.06.08a.12.12 0 000 .07c0 .03.01.05.05.09l45.02 45.02a.2.2 0 00.09.05.12.12 0 00.07 0c.02 0 .04-.01.08-.05L512 557.25l127.86 127.87c.04.04.06.05.08.05a.12.12 0 00.07 0c.03 0 .05-.01.09-.05l45.02-45.02a.2.2 0 00.05-.09.12.12 0 000-.07v-.02a.27.27 0 00-.05-.06L557.25 512l127.87-127.86c.04-.04.05-.06.05-.08a.12.12 0 000-.07c0-.03-.01-.05-.05-.09l-45.02-45.02a.2.2 0 00-.09-.05.12.12 0 00-.07 0z";
const ICON_VIEW: (f32, f32, f32, f32) = (0.0, 0.0, 1024.0, 1024.0);

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const WEEK_NAMES: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
const WEEKDAY_NAMES: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// 跨帧持久化的待选起点
struct RangeState {
    pending: Option<Date>,
}

/// 日期范围选择器。
///
/// ```ignore
/// DateRangePicker::new("stay", self.range)
///     .open(self.range_open)
///     .on_open_change(cx.listener(|this, open, _, cx| { this.range_open = *open; cx.notify(); }))
///     .on_change(cx.listener(|this, range, _, cx| { this.range = *range; cx.notify(); }))
/// ```
#[derive(IntoElement)]
pub struct DateRangePicker {
    id: ElementId,
    value: Option<(Date, Date)>,
    open: bool,
    disabled: bool,
    allow_clear: bool,
    placeholder: (SharedString, SharedString),
    month: Option<(i32, u32)>,
    size: InputSize,
    variant: InputVariant,
    status: InputStatus,
    on_change: Option<Rc<dyn Fn(&Option<(Date, Date)>, &mut Window, &mut App)>>,
    on_clear: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    on_month_change: Option<Rc<dyn Fn(&(i32, u32), &mut Window, &mut App)>>,
}

impl DateRangePicker {
    pub fn new(id: impl Into<ElementId>, value: Option<(Date, Date)>) -> Self {
        Self {
            id: id.into(),
            value,
            open: false,
            disabled: false,
            allow_clear: false,
            placeholder: ("开始日期".into(), "结束日期".into()),
            month: None,
            size: InputSize::Middle,
            variant: InputVariant::Outlined,
            status: InputStatus::Default,
            on_change: None,
            on_clear: None,
            on_open_change: None,
            on_month_change: None,
        }
    }

    /// 当前是否展开（受控）
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

    /// 两段占位文案（开始 / 结束）
    pub fn placeholder(
        mut self,
        start: impl Into<SharedString>,
        end: impl Into<SharedString>,
    ) -> Self {
        self.placeholder = (start.into(), end.into());
        self
    }

    /// 日历当前显示的月份（受控，左面板），默认取起点所在月或当前月
    pub fn month(mut self, month: Option<(i32, u32)>) -> Self {
        self.month = month;
        self
    }

    pub fn on_change(
        mut self,
        listener: impl Fn(&Option<(Date, Date)>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    /// 点击清空按钮时回调（未设置时回退为 on_change(None)）
    pub fn on_clear(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear = Some(Rc::new(listener));
        self
    }

    pub fn on_open_change(
        mut self,
        listener: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Rc::new(listener));
        self
    }

    /// 点击月导航时回调目标月份（左面板）
    pub fn on_month_change(
        mut self,
        listener: impl Fn(&(i32, u32), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_month_change = Some(Rc::new(listener));
        self
    }
}

fn icon(d: &'static str, size: f32, color: gpui::Hsla) -> impl IntoElement {
    let segs = svg_segments(d);
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, ICON_VIEW, &segs, color, window),
    )
    .w(px(size))
    .h(px(size))
}

impl RenderOnce for DateRangePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(
            gpui::SharedString::from(format!("{:?}", self.id)),
            cx,
            |_, _| RangeState { pending: None },
        );
        let pending = state.read(cx).pending;
        let open = self.open && !self.disabled;
        let suppress_toggle = Rc::new(Cell::new(false));
        let m = theme::with(date_picker_metric);
        let rm = theme::with(range_picker_metric);
        let ps = theme::with(date_picker_style);
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

        let fmt = |d: Date| format!("{}-{:02}-{:02}", d.0, d.1, d.2);
        let (start_label, end_label) = match self.value {
            Some((s, e)) => (fmt(s), fmt(e)),
            None => (
                self.placeholder.0.to_string(),
                self.placeholder.1.to_string(),
            ),
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
            // antd 在 Windows 首选 Segoe UI；gpui 默认取系统 UI 字体（中文系统
            // 为 Microsoft YaHei），数字更宽会把分隔符与第二段日期挤歪
            .font_family("Segoe UI")
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_color(if self.value.is_some() {
                        style.fg
                    } else {
                        style.placeholder
                    })
                    .child(start_label.clone()),
            )
            .child(
                div()
                    .w(px(rm.separator_size))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon(SWAP_ICON, rm.separator_size, ps.suffix)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_color(if self.value.is_some() {
                        style.fg
                    } else {
                        style.placeholder
                    })
                    .child(end_label.clone()),
            );

        if show_clear {
            let on_clear = self.on_clear.clone();
            let on_change = self.on_change.clone();
            let clear = a11y::Semantics::new(Role::Button, "Clear date range")
                .id(format!("{:?}-clear", self.id))
                .focusable(true)
                .apply(
                    div()
                        .id(ElementId::Name(format!("{:?}-clear", self.id).into()))
                        .ml(px(m.suffix_gap))
                        .cursor(CursorStyle::PointingHand)
                        .child(icon(CLEAR_ICON, m.icon_size, ps.suffix))
                        .hover(|s| s.opacity(0.7)),
                );
            trigger = trigger.child(a11y::click_void(clear, move |window, cx| {
                cx.stop_propagation();
                if let Some(on_clear) = &on_clear {
                    on_clear(window, cx);
                } else if let Some(on_change) = &on_change {
                    on_change(&None, window, cx);
                }
            }));
        } else {
            trigger = trigger.child(div().ml(px(m.suffix_gap)).child(icon(
                CALENDAR_ICON,
                m.icon_size,
                ps.suffix,
            )));
        }

        if !self.disabled {
            let border = hover_style.border_color();
            trigger = trigger
                .cursor(CursorStyle::PointingHand)
                .hover(move |s| s.border_color(border));
        } else {
            trigger = trigger.cursor(CursorStyle::Arrow);
        }
        let range_label = format!("{start_label} to {end_label}");
        let mut semantics = a11y::Semantics::new(Role::ComboBox, "Date range")
            .id(format!("{:?}-trigger", self.id))
            .expanded(open)
            .disabled(self.disabled)
            .focusable(!self.disabled);
        semantics = if self.value.is_some() {
            semantics.value(range_label)
        } else {
            semantics.placeholder(range_label)
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

        let mut popup = None;
        if open {
            popup = Some(div().child(self.calendar(&suppress_toggle, state, pending)));
        }

        anchored(trigger.into_any_element(), popup)
    }
}

impl DateRangePicker {
    /// 双月日历浮层（范围选择）
    fn calendar(
        &self,
        suppress_toggle: &Rc<Cell<bool>>,
        state: Entity<RangeState>,
        pending: Option<Date>,
    ) -> Stateful<Div> {
        let m = theme::with(date_picker_metric);
        let ps = theme::with(date_picker_style);
        let today = today();
        let (year, month) = self
            .month
            .or(self.value.map(|((y, mo, _), _)| (y, mo)))
            .unwrap_or((today.0, today.1));
        let right = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        let shell = div()
            .id(ElementId::Name(format!("{:?}-calendar", self.id).into()))
            .occlude()
            .flex()
            .flex_row()
            .rounded(px(m.radius))
            .bg(ps.bg)
            .shadow(ps.shadow.clone())
            .on_mouse_down_out({
                let on_open_change = self.on_open_change.clone();
                let suppress_toggle = suppress_toggle.clone();
                let state = state.clone();
                move |_, window, cx| {
                    suppress_toggle.set(true);
                    state.update(cx, |s, _| s.pending = None);
                    if let Some(on_open_change) = &on_open_change {
                        on_open_change(&false, window, cx);
                    }
                }
            });
        let mut shell = a11y::Semantics::new(Role::Dialog, "Date range calendar")
            .id(format!("{:?}-calendar", self.id))
            .apply(shell);

        shell = shell.child(self.panel(year, month, true, &ps, &m, state.clone(), pending));
        shell = shell.child(self.panel(right.0, right.1, false, &ps, &m, state, pending));
        shell
    }

    /// 单侧月面板：left 带年/月后退钮，right 带前进钮
    fn panel(
        &self,
        year: i32,
        month: u32,
        left: bool,
        ps: &crate::theme::metric::DatePickerStyle,
        m: &crate::theme::metric::DatePickerMetric,
        state: Entity<RangeState>,
        pending: Option<Date>,
    ) -> Div {
        let today = today();
        let prev = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };
        let next = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let mut header = div()
            .flex()
            .flex_row()
            .items_center()
            .h(px(m.header_height))
            .px(px(m.header_pad_h))
            .border_b_1()
            .border_color(ps.split);
        if left {
            header = header
                .child(nav_button(
                    ElementId::Name(format!("{:?}-prev-y", self.id).into()),
                    DOUBLE_LEFT_ICON,
                    "Previous year",
                    (year - 1, month),
                    m,
                    ps,
                    self.on_month_change.clone(),
                ))
                .child(nav_button(
                    ElementId::Name(format!("{:?}-prev-m", self.id).into()),
                    LEFT_ICON,
                    "Previous month",
                    prev,
                    m,
                    ps,
                    self.on_month_change.clone(),
                ));
        }
        header = header.child(
            div()
                .flex_1()
                .text_center()
                .text_size(px(m.font_size))
                .line_height(px(m.header_height))
                .font_weight(gpui::FontWeight(m.title_weight))
                .text_color(ps.header_fg)
                .child(format!("{} {year}", MONTH_NAMES[(month - 1) as usize])),
        );
        if !left {
            header = header
                .child(nav_button(
                    ElementId::Name(format!("{:?}-next-m", self.id).into()),
                    RIGHT_ICON,
                    "Next month",
                    next,
                    m,
                    ps,
                    self.on_month_change.clone(),
                ))
                .child(nav_button(
                    ElementId::Name(format!("{:?}-next-y", self.id).into()),
                    DOUBLE_RIGHT_ICON,
                    "Next year",
                    (year + 1, month),
                    m,
                    ps,
                    self.on_month_change.clone(),
                ));
        }

        let panel_id = format!("{:?}-{}-grid", self.id, if left { 'l' } else { 'r' });
        let mut body = a11y::Semantics::new(
            Role::Grid,
            format!("Calendar for {} {year}", MONTH_NAMES[(month - 1) as usize]),
        )
        .id(panel_id.clone())
        .apply(
            div()
                .id(ElementId::Name(panel_id.clone().into()))
                .flex()
                .flex_col()
                .py(px(m.body_pad_v))
                .px(px(m.pad_h)),
        );
        let mut week = div().flex().flex_row();
        for (ix, name) in WEEK_NAMES.into_iter().enumerate() {
            let header = a11y::Semantics::new(Role::ColumnHeader, WEEKDAY_NAMES[ix])
                .id(format!("{panel_id}-weekday-{ix}"))
                .column(ix + 1, WEEK_NAMES.len())
                .apply(
                    div()
                        .id(ElementId::Name(format!("{panel_id}-weekday-{ix}").into()))
                        .w(px(m.cell_width))
                        .h(px(m.th_height))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(m.font_size))
                        .text_color(ps.cell_fg)
                        .child(name),
                );
            week = week.child(header);
        }
        body = body.child(week);

        let offset = ((weekday_monday0(year, month, 1) + 1) % 7) as usize;
        let dim = days_in_month(year, month) as usize;
        let prev_dim = days_in_month(prev.0, prev.1) as usize;
        let (start, end) = match self.value {
            Some((s, e)) => (Some(s), Some(e)),
            None => (None, None),
        };

        for row in 0..6 {
            let mut row_div = div().flex().flex_row();
            for col in 0..7 {
                let n = row * 7 + col;
                let day = n as i64 - offset as i64 + 1;
                let in_view = day >= 1 && day <= dim as i64;
                let shown = if day < 1 {
                    prev_dim as i64 + day
                } else if day > dim as i64 {
                    day - dim as i64
                } else {
                    day
                };
                let date = (year, month, day as u32);
                let is_start = in_view && (start == Some(date) || pending == Some(date));
                let is_end = in_view && end == Some(date);
                let in_range = match (start, end) {
                    (Some(s), Some(e)) => date > s && date < e,
                    _ => false,
                };
                let cs = theme::with(|t| {
                    range_cell_state(
                        t,
                        in_view,
                        in_view && today == date,
                        is_start,
                        is_end,
                        in_range,
                    )
                });

                let cell = div()
                    .id(ElementId::Name(
                        format!("{:?}-{}-d-{n}", self.id, if left { 'l' } else { 'r' }).into(),
                    ))
                    .relative()
                    .w(px(m.cell_width))
                    .h(px(m.th_height))
                    .flex()
                    .items_center()
                    .justify_center();
                let mut cell = a11y::Semantics::new(
                    Role::GridCell,
                    format!("{} {shown}, {year}", MONTH_NAMES[(month - 1) as usize]),
                )
                .id(format!(
                    "{:?}-{}-d-{n}",
                    self.id,
                    if left { 'l' } else { 'r' }
                ))
                .value(format!("{shown:02}"))
                .selected(in_view && (is_start || is_end || in_range))
                .row(row + 1, 6)
                .column(col + 1, 7)
                .disabled(!in_view)
                .focusable(in_view)
                .apply(cell);
                if let Some(band) = cs.band {
                    let half = cs.band_half;
                    let mut band_div = div().absolute().top(px(m.cell_pad_v)).h(px(m.cell_height));
                    band_div = match half {
                        Some(RangeEnd::Start) => band_div.left(px(m.cell_width / 2.0)).right_0(),
                        Some(RangeEnd::End) => band_div.left_0().right(px(m.cell_width / 2.0)),
                        None => band_div.left_0().right_0(),
                    };
                    cell = cell.child(band_div.bg(band));
                }
                let mut inner = div()
                    .w(px(m.cell_inner))
                    .h(px(m.cell_inner))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(m.font_size))
                    .line_height(px(m.cell_inner))
                    .text_color(cs.fg)
                    .child(format!("{shown}"));
                match cs.end {
                    Some(RangeEnd::Start) => {
                        inner = inner.rounded_l(px(m.cell_radius));
                    }
                    Some(RangeEnd::End) => {
                        inner = inner.rounded_r(px(m.cell_radius));
                    }
                    None => {
                        inner = inner.rounded(px(m.cell_radius));
                    }
                }
                if let Some(bg) = cs.inner_bg {
                    inner = inner.bg(bg);
                }
                if let Some(border) = cs.border {
                    inner = inner.border_1().border_color(border);
                }
                if in_view {
                    cell = cell.cursor(CursorStyle::PointingHand);
                    if cs.inner_bg.is_none() {
                        let hover_bg = ps.cell_hover_bg;
                        inner = inner.hover(move |s| s.bg(hover_bg));
                    }
                    let on_change = self.on_change.clone();
                    let on_open_change = self.on_open_change.clone();
                    let state = state.clone();
                    cell = a11y::click_void(cell, move |window, cx| {
                        let pending = state.read(cx).pending;
                        match pending {
                            None => {
                                state.update(cx, |s, cx| {
                                    s.pending = Some(date);
                                    cx.notify();
                                });
                            }
                            Some(s) if date < s => {
                                state.update(cx, |st, cx| {
                                    st.pending = Some(date);
                                    cx.notify();
                                });
                            }
                            Some(s) => {
                                state.update(cx, |st, _| st.pending = None);
                                if let Some(on_change) = &on_change {
                                    on_change(&Some((s, date)), window, cx);
                                }
                                if let Some(on_open_change) = &on_open_change {
                                    on_open_change(&false, window, cx);
                                }
                            }
                        }
                    });
                }
                row_div = row_div.child(cell.child(inner));
            }
            body = body.child(row_div);
        }

        div().flex().flex_col().child(header).child(body)
    }
}

/// 月/年导航按钮
fn nav_button(
    id: ElementId,
    d: &'static str,
    label: &'static str,
    target: (i32, u32),
    m: &crate::theme::metric::DatePickerMetric,
    ps: &crate::theme::metric::DatePickerStyle,
    on_month_change: Option<Rc<dyn Fn(&(i32, u32), &mut Window, &mut App)>>,
) -> Stateful<Div> {
    let color = ps.icon;
    let hover = ps.icon_hover;
    let size = m.nav_icon;
    let semantics_id = format!("{:?}", id);
    let group = gpui::SharedString::from(format!("rp-nav-{id:?}"));
    let mut button = div()
        .id(ElementId::Name(semantics_id.clone().into()))
        .group(group.clone())
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .w(px(m.font_size * 1.6))
        .h_full()
        .child(
            div()
                .group_hover(group.clone(), |s| s.opacity(0.))
                .child(icon(d, size, color)),
        )
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .opacity(0.)
                .group_hover(group, |s| s.opacity(1.))
                .child(icon(d, size, hover)),
        );
    let enabled = on_month_change.is_some();
    if enabled {
        button = button.cursor(CursorStyle::PointingHand);
    }
    let mut button = a11y::Semantics::new(Role::Button, label)
        .id(semantics_id)
        .disabled(!enabled)
        .focusable(enabled)
        .apply(button);
    if let Some(on_month_change) = on_month_change {
        button = a11y::click_void(button, move |window, cx| {
            on_month_change(&target, window, cx);
        });
    }
    button
}
