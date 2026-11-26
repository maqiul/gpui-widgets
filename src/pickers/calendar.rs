//! 日历：antd 全尺寸月视图 / 年视图（月格）。
//!
//! 头部 = 年 Select + 月 Select + Month/Year radio 按钮组（右对齐）；
//! date 格 116px、顶描边 2px、值右对齐；选中底 controlItemBgActive + 值主色。
//! 显示月份 / 选中日期 / 视图模式均受控；头部下拉展开态按 id 持久化。

use std::{cell::Cell, rc::Rc};

use gpui::{
    App, CursorStyle, Div, ElementId, Entity, IntoElement, Role, SharedString, Stateful, Window,
    canvas, div, prelude::*, px,
};

use crate::Scrollbar;
use crate::a11y;
use crate::date_picker::{Date, days_in_month, today, weekday_monday0};
use crate::svg_path::{paint_svg, svg_segments};
use crate::text_input::{self, InputSize, InputStatus, InputVariant};
use crate::theme::metric::{calendar_cell_state, calendar_metric, calendar_style};
use crate::{popup::anchored, theme};

const DOWN_ICON: &str = "M884 256h-75c-5.1 0-9.9 2.5-12.9 6.6L512 654.2 227.9 262.6c-3-4.1-7.8-6.6-12.9-6.6h-75c-6.5 0-10.3 7.4-6.5 12.7l352.6 486.1c12.8 17.6 39 17.6 51.7 0l352.6-486.1c3.9-5.3.1-12.7-6.4-12.7z";
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

/// 日历视图模式
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CalendarMode {
    Month,
    Year,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HeaderSelect {
    Year,
    Month,
}

/// 跨帧持久化的头部下拉展开态
struct CalendarState {
    open: Option<HeaderSelect>,
}

/// 全尺寸日历。
///
/// ```ignore
/// Calendar::new("cal", self.cal_month)
///     .value(self.cal_value)
///     .on_select(cx.listener(|this, date, _, cx| { this.cal_value = Some(*date); cx.notify(); }))
///     .on_month_change(cx.listener(|this, month, _, cx| { this.cal_month = *month; cx.notify(); }))
///     .mode(self.cal_mode)
///     .on_mode_change(cx.listener(|this, mode, _, cx| { this.cal_mode = *mode; cx.notify(); }))
/// ```
#[derive(IntoElement)]
pub struct Calendar {
    id: ElementId,
    month: (i32, u32),
    value: Option<Date>,
    mode: CalendarMode,
    on_month_change: Option<Rc<dyn Fn(&(i32, u32), &mut Window, &mut App)>>,
    on_select: Option<Rc<dyn Fn(&Date, &mut Window, &mut App)>>,
    on_mode_change: Option<Rc<dyn Fn(&CalendarMode, &mut Window, &mut App)>>,
}

impl Calendar {
    pub fn new(id: impl Into<ElementId>, month: (i32, u32)) -> Self {
        Self {
            id: id.into(),
            month,
            value: None,
            mode: CalendarMode::Month,
            on_month_change: None,
            on_select: None,
            on_mode_change: None,
        }
    }

    /// 选中日期（受控）
    pub fn value(mut self, value: Option<Date>) -> Self {
        self.value = value;
        self
    }

    /// 视图模式（受控，默认月视图）
    pub fn mode(mut self, mode: CalendarMode) -> Self {
        self.mode = mode;
        self
    }

    /// 切换显示月份时回调
    pub fn on_month_change(
        mut self,
        listener: impl Fn(&(i32, u32), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_month_change = Some(Rc::new(listener));
        self
    }

    /// 点击日期时回调
    pub fn on_select(mut self, listener: impl Fn(&Date, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }

    /// 切换视图模式时回调
    pub fn on_mode_change(
        mut self,
        listener: impl Fn(&CalendarMode, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_mode_change = Some(Rc::new(listener));
        self
    }
}

fn down_icon(size: f32, color: gpui::Hsla) -> impl IntoElement {
    let segs = svg_segments(DOWN_ICON);
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, ICON_VIEW, &segs, color, window),
    )
    .w(px(size))
    .h(px(size))
}

impl RenderOnce for Calendar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state =
            window.use_keyed_state(SharedString::from(format!("{:?}", self.id)), cx, |_, _| {
                CalendarState { open: None }
            });
        let open = state.read(cx).open;
        let suppress_toggle = Rc::new(Cell::new(false));
        let m = theme::with(calendar_metric);
        let cs = theme::with(calendar_style);
        let (year, month) = self.month;

        let mut header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .py(px(m.header_pad_v));
        header = header.child(self.header_select(
            HeaderSelect::Year,
            format!("{year}"),
            m.year_width,
            0.0,
            open == Some(HeaderSelect::Year),
            &state,
            &suppress_toggle,
            window,
            cx,
        ));
        if self.mode == CalendarMode::Month {
            header = header.child(self.header_select(
                HeaderSelect::Month,
                MONTH_NAMES[(month - 1) as usize].to_string(),
                m.month_width,
                m.control_gap,
                open == Some(HeaderSelect::Month),
                &state,
                &suppress_toggle,
                window,
                cx,
            ));
        }

        let mode_group_id = format!("{:?}-modes", self.id);
        let mut mode_switch = a11y::Semantics::new(Role::RadioGroup, "Calendar view")
            .id(mode_group_id.clone())
            .apply(
                div()
                    .id(ElementId::Name(mode_group_id.into()))
                    .flex()
                    .flex_row()
                    .ml(px(m.control_gap)),
            );
        let modes = [(CalendarMode::Month, "Month"), (CalendarMode::Year, "Year")];
        for (ix, (mode, label)) in modes.into_iter().enumerate() {
            let checked = self.mode == mode;
            let prev_checked = ix > 0 && self.mode == modes[ix - 1].0;
            mode_switch = mode_switch.child(mode_button(
                ElementId::Name(format!("{:?}-mode-{:?}", self.id, mode).into()),
                label,
                checked,
                prev_checked,
                ix == 0,
                ix == modes.len() - 1,
                &m,
                &cs,
                mode,
                self.on_mode_change.clone(),
            ));
        }
        header = header.child(mode_switch);

        let mut root = div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .w_full()
            .bg(cs.bg)
            .child(header);
        root = match self.mode {
            CalendarMode::Month => root.child(self.month_view()),
            CalendarMode::Year => root.child(self.year_view()),
        };
        let mut semantics =
            a11y::Semantics::new(Role::Region, "Calendar").id(format!("{:?}", self.id));
        if let Some((y, mo, d)) = self.value {
            semantics = semantics.value(format!("{y:04}-{mo:02}-{d:02}"));
        }
        semantics.apply(root)
    }
}

impl Calendar {
    /// 头部年/月 Select 触发框 + 下拉菜单
    fn header_select(
        &self,
        which: HeaderSelect,
        label: String,
        width: f32,
        margin_start: f32,
        open: bool,
        state: &Entity<CalendarState>,
        suppress_toggle: &Rc<Cell<bool>>,
        window: &mut Window,
        cx: &mut App,
    ) -> Div {
        let m = theme::with(calendar_metric);
        let cs = theme::with(calendar_style);
        let style = theme::with(|t| {
            text_input::state(
                InputVariant::Outlined,
                InputStatus::Default,
                InputSize::Middle,
                t,
                false,
                false,
                false,
            )
        });
        let hover_border = theme::with(|t| {
            text_input::state(
                InputVariant::Outlined,
                InputStatus::Default,
                InputSize::Middle,
                t,
                true,
                false,
                false,
            )
            .border_color()
        });
        let (year, month) = self.month;
        let trigger = div()
            .id(ElementId::Name(
                format!("{:?}-hs-{which:?}", self.id).into(),
            ))
            .flex()
            .flex_row()
            .items_center()
            .w(px(width))
            .h(px(m.select_height))
            .px(px(m.select_pad_inline))
            .rounded(px(m.select_radius))
            .border_1()
            .border_color(style.border_color())
            .bg(style.bg)
            .text_size(px(m.font_size))
            .line_height(px(style.line_height))
            .text_color(style.fg)
            .cursor(CursorStyle::PointingHand)
            .hover(move |s| s.border_color(hover_border))
            .child(div().flex_1().min_w_0().child(label.clone()))
            .child(down_icon(m.icon_size, cs.suffix));
        let mut trigger = a11y::Semantics::new(
            Role::ComboBox,
            if which == HeaderSelect::Year {
                "Year"
            } else {
                "Month"
            },
        )
        .id(format!("{:?}-hs-{which:?}", self.id))
        .value(label)
        .expanded(open)
        .focusable(true)
        .apply(trigger);
        {
            let state = state.clone();
            let suppress_toggle = suppress_toggle.clone();
            trigger = a11y::click_void(trigger, move |window, cx| {
                if suppress_toggle.take() {
                    return;
                }
                let next = if open { None } else { Some(which) };
                state.update(cx, |s, cx| {
                    s.open = next;
                    cx.notify();
                });
                let _ = window;
            });
        }

        let popup = if open {
            let menu = div()
                .id(ElementId::Name(
                    format!("{:?}-menu-{which:?}", self.id).into(),
                ))
                .occlude()
                .flex()
                .flex_col()
                .w(px(width))
                .p(px(m.menu_padding))
                .rounded(px(m.menu_radius))
                .bg(cs.menu_bg)
                .shadow(cs.menu_shadow.clone())
                .on_mouse_down_out({
                    let state = state.clone();
                    let suppress_toggle = suppress_toggle.clone();
                    move |_, _, cx| {
                        suppress_toggle.set(true);
                        state.update(cx, |s, cx| {
                            s.open = None;
                            cx.notify();
                        });
                    }
                });
            let mut menu = a11y::Semantics::new(
                Role::ListBox,
                if which == HeaderSelect::Year {
                    "Years"
                } else {
                    "Months"
                },
            )
            .id(format!("{:?}-menu-{which:?}", self.id))
            .apply(menu);
            let year_scroll = if which == HeaderSelect::Year {
                let scroll_id = format!("calendar-year-{:?}", self.id);
                Some((
                    scroll_id.clone(),
                    window
                        .use_keyed_state(
                            ElementId::Name(format!("{scroll_id}-handle").into()),
                            cx,
                            |_, _| gpui::ScrollHandle::new(),
                        )
                        .read(cx)
                        .clone(),
                ))
            } else {
                None
            };
            if which == HeaderSelect::Year {
                menu = menu.flex_row().h(px(m.menu_max_height));
            }
            let entries: Vec<(String, bool)> = match which {
                HeaderSelect::Year => ((year - m.year_offset)
                    ..(year - m.year_offset + m.year_total))
                    .map(|y| (format!("{y}"), y == year))
                    .collect(),
                HeaderSelect::Month => MONTH_NAMES
                    .iter()
                    .enumerate()
                    .map(|(i, name)| (name.to_string(), i as u32 + 1 == month))
                    .collect(),
            };
            let entry_count = entries.len();
            let mut year_list = div().flex().flex_col();
            for (ix, (label, selected)) in entries.into_iter().enumerate() {
                let target = match which {
                    HeaderSelect::Year => (label.parse::<i32>().unwrap_or(year), month),
                    HeaderSelect::Month => (
                        year,
                        MONTH_NAMES
                            .iter()
                            .position(|n| *n == label)
                            .map(|i| i as u32 + 1)
                            .unwrap_or(month),
                    ),
                };
                let item = div()
                    .id(ElementId::Name(
                        format!("{:?}-opt-{which:?}-{label}", self.id).into(),
                    ))
                    .flex()
                    .items_center()
                    .h(px(m.option_height))
                    .px(px(m.option_pad_inline))
                    .rounded(px(m.menu_radius))
                    .text_size(px(m.font_size))
                    .line_height(px(style.line_height))
                    .text_color(cs.option_fg)
                    .cursor(CursorStyle::PointingHand)
                    .child(label.clone());
                let mut item = a11y::Semantics::new(Role::ListBoxOption, label.clone())
                    .id(format!("{:?}-opt-{which:?}-{ix}", self.id))
                    .selected(selected)
                    .position(ix + 1, entry_count)
                    .focusable(true)
                    .apply(item);
                if selected {
                    item = item
                        .bg(cs.option_selected_bg)
                        .font_weight(gpui::FontWeight(cs.option_weight));
                } else {
                    let hover_bg = cs.option_hover_bg;
                    item = item.hover(move |s| s.bg(hover_bg));
                }
                let on_month_change = self.on_month_change.clone();
                let state = state.clone();
                item = a11y::click_void(item, move |window, cx| {
                    state.update(cx, |s, cx| {
                        s.open = None;
                        cx.notify();
                    });
                    if let Some(on_month_change) = &on_month_change {
                        on_month_change(&target, window, cx);
                    }
                });
                if which == HeaderSelect::Year {
                    year_list = year_list.child(item);
                } else {
                    menu = menu.child(item);
                }
            }
            if let Some((scroll_id, handle)) = year_scroll {
                let list = div()
                    .id(ElementId::Name(format!("{scroll_id}-content").into()))
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(px(0.))
                    .min_h(px(0.))
                    .overflow_y_scroll()
                    .track_scroll(&handle)
                    .child(year_list);
                menu = menu.child(list).child(Scrollbar::vertical(
                    ElementId::Name(format!("{scroll_id}-scrollbar").into()),
                    handle,
                ));
            }
            Some(div().child(menu))
        } else {
            None
        };
        div()
            .ml(px(margin_start))
            .child(anchored(trigger.into_any_element(), popup))
    }

    /// 月视图：周表头 + 6×7 date 格
    fn month_view(&self) -> Stateful<Div> {
        let m = theme::with(calendar_metric);
        let cs = theme::with(calendar_style);
        let (year, month) = self.month;
        let today = today();
        let grid_id = format!("{:?}-month-grid-{year}-{month}", self.id);
        let mut body = a11y::Semantics::new(
            Role::Grid,
            format!("Calendar for {} {year}", MONTH_NAMES[(month - 1) as usize]),
        )
        .id(grid_id.clone())
        .apply(
            div()
                .id(ElementId::Name(grid_id.into()))
                .flex()
                .flex_col()
                .py(px(m.body_pad_v)),
        );

        let mut week = div().flex().flex_row();
        for (ix, name) in WEEK_NAMES.into_iter().enumerate() {
            let header_id = format!("{:?}-weekday-{ix}", self.id);
            let header = a11y::Semantics::new(Role::ColumnHeader, WEEKDAY_NAMES[ix])
                .id(header_id.clone())
                .column(ix + 1, WEEK_NAMES.len())
                .apply(
                    div()
                        .id(ElementId::Name(header_id.into()))
                        .flex_1()
                        .h(px(m.week_height + m.week_pad_bottom))
                        .pb(px(m.week_pad_bottom))
                        .pr(px(m.week_pad_end))
                        .text_right()
                        .text_size(px(m.font_size))
                        .line_height(px(m.week_height))
                        .text_color(cs.fg)
                        .child(name),
                );
            week = week.child(header);
        }
        body = body.child(week);

        let offset = ((weekday_monday0(year, month, 1) + 1) % 7) as usize;
        let dim = days_in_month(year, month) as usize;
        let prev = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };
        let prev_dim = days_in_month(prev.0, prev.1) as usize;

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
                let selected = in_view && self.value == Some(date);
                let cell_state = theme::with(|t| {
                    calendar_cell_state(t, in_view, in_view && today == date, selected)
                });
                let mut date_block = div()
                    .mx(px(m.date_margin_h))
                    .h(px(m.cell_height))
                    .flex()
                    .flex_col()
                    .child(div().h(px(m.date_border)).bg(cell_state.border_top))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .pt(px(m.date_pad_top))
                            .px(px(m.date_pad_h))
                            .child(
                                div()
                                    .h(px(m.date_value_height))
                                    .line_height(px(m.date_value_height))
                                    .text_right()
                                    .text_size(px(m.font_size))
                                    .text_color(cell_state.value_fg)
                                    .child(format!("{shown:02}")),
                            )
                            .child(div().h(px(m.date_content_height))),
                    );
                if let Some(bg) = cell_state.bg {
                    date_block = date_block.bg(bg);
                }
                let cell = div()
                    .id(ElementId::Name(
                        format!("{:?}-{}-{}-{}", self.id, year, month, n).into(),
                    ))
                    .flex_1()
                    .h(px(m.cell_height));
                let mut cell = a11y::Semantics::new(
                    Role::GridCell,
                    format!("{} {shown}, {year}", MONTH_NAMES[(month - 1) as usize]),
                )
                .id(format!("{:?}-{}-{}-{}", self.id, year, month, n))
                .value(format!("{shown:02}"))
                .selected(in_view && self.value == Some(date))
                .row(row + 1, 6)
                .column(col + 1, 7)
                .disabled(!in_view)
                .focusable(in_view)
                .apply(cell);
                if in_view {
                    cell = cell.cursor(CursorStyle::PointingHand);
                    if cell_state.bg.is_none() {
                        let hover_bg = cs.hover_bg;
                        date_block = date_block.hover(move |s| s.bg(hover_bg));
                    }
                    let on_select = self.on_select.clone();
                    cell = a11y::click_void(cell, move |window, cx| {
                        if let Some(on_select) = &on_select {
                            on_select(&date, window, cx);
                        }
                    });
                }
                row_div = row_div.child(cell.child(date_block));
            }
            body = body.child(row_div);
        }
        body
    }

    /// 年视图：3×4 月格（与 date 格同几何，值为月名）
    fn year_view(&self) -> Stateful<Div> {
        let m = theme::with(calendar_metric);
        let cs = theme::with(calendar_style);
        let (year, _) = self.month;
        let today = today();
        let grid_id = format!("{:?}-year-grid-{year}", self.id);
        let mut body = a11y::Semantics::new(Role::Grid, format!("Months in {year}"))
            .id(grid_id.clone())
            .apply(
                div()
                    .id(ElementId::Name(grid_id.into()))
                    .flex()
                    .flex_col()
                    .py(px(m.body_pad_v)),
            );

        for row in 0..4 {
            let mut row_div = div().flex().flex_row();
            for col in 0..3 {
                let month = (row * 3 + col + 1) as u32;
                let selected = self.value.map(|(y, mo, _)| (y, mo)) == Some((year, month));
                let is_now = today.0 == year && today.1 == month;
                let cell_state = theme::with(|t| calendar_cell_state(t, true, is_now, selected));
                let mut date_block = div()
                    .mx(px(m.date_margin_h))
                    .h(px(m.cell_height))
                    .flex()
                    .flex_col()
                    .child(div().h(px(m.date_border)).bg(cell_state.border_top))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .pt(px(m.date_pad_top))
                            .px(px(m.date_pad_h))
                            .child(
                                div()
                                    .h(px(m.date_value_height))
                                    .line_height(px(m.date_value_height))
                                    .text_right()
                                    .text_size(px(m.font_size))
                                    .text_color(cell_state.value_fg)
                                    .child(MONTH_NAMES[(month - 1) as usize]),
                            )
                            .child(div().h(px(m.date_content_height))),
                    );
                if let Some(bg) = cell_state.bg {
                    date_block = date_block.bg(bg);
                } else {
                    let hover_bg = cs.hover_bg;
                    date_block = date_block.hover(move |s| s.bg(hover_bg));
                }
                let on_month_change = self.on_month_change.clone();
                let on_mode_change = self.on_mode_change.clone();
                let cell = div()
                    .id(ElementId::Name(
                        format!("{:?}-y{}-m{}", self.id, year, month).into(),
                    ))
                    .flex_1()
                    .h(px(m.cell_height))
                    .cursor(CursorStyle::PointingHand)
                    .child(date_block);
                let mut cell =
                    a11y::Semantics::new(Role::GridCell, MONTH_NAMES[(month - 1) as usize])
                        .id(format!("{:?}-y{}-m{}", self.id, year, month))
                        .value(MONTH_NAMES[(month - 1) as usize])
                        .selected(selected)
                        .row(row + 1, 4)
                        .column(col + 1, 3)
                        .focusable(true)
                        .apply(cell);
                cell = a11y::click_void(cell, move |window, cx| {
                    if let Some(on_month_change) = &on_month_change {
                        on_month_change(&(year, month), window, cx);
                    }
                    if let Some(on_mode_change) = &on_mode_change {
                        on_mode_change(&CalendarMode::Month, window, cx);
                    }
                });
                row_div = row_div.child(cell);
            }
            body = body.child(row_div);
        }
        body
    }
}

/// Month/Year radio 按钮：相邻共边（margin -1），选中者边框压邻格（左邻选中时本格左边染主色）
fn mode_button(
    id: ElementId,
    label: &'static str,
    checked: bool,
    prev_checked: bool,
    first: bool,
    last: bool,
    m: &crate::theme::metric::CalendarMetric,
    cs: &crate::theme::metric::CalendarStyle,
    mode: CalendarMode,
    on_mode_change: Option<Rc<dyn Fn(&CalendarMode, &mut Window, &mut App)>>,
) -> Stateful<Div> {
    let own_border = if checked {
        cs.mode_checked_fg
    } else {
        cs.mode_border
    };
    let fg = if checked {
        cs.mode_checked_fg
    } else {
        cs.mode_fg
    };
    let hover_fg = if checked {
        cs.mode_checked_hover_fg
    } else {
        cs.mode_hover_fg
    };
    let semantics_id = format!("{:?}", id);
    let mut button = div()
        .id(id.clone())
        .flex()
        .items_center()
        .h(px(m.mode_height))
        .px(px(m.mode_pad_inline))
        .border_1()
        .border_color(own_border)
        .bg(cs.mode_bg)
        .text_size(px(m.font_size))
        .line_height(px(m.mode_line_height))
        .text_color(fg)
        .cursor(CursorStyle::PointingHand)
        .hover(move |s| s.text_color(hover_fg))
        .child(label);
    if !first {
        let left = if prev_checked {
            cs.mode_checked_fg
        } else {
            own_border
        };
        // 共边：去掉自身左边框，用绝对定位色条模拟 z-index（选中左邻的右边框压本格）
        button = button.relative().ml(px(-1.0)).border_l_0().child(
            div()
                .absolute()
                .left_0()
                .top_0()
                .bottom_0()
                .w(px(1.0))
                .bg(left),
        );
    }
    if first {
        button = button.rounded_l(px(m.mode_radius));
    }
    if last {
        button = button.rounded_r(px(m.mode_radius));
    }
    let enabled = on_mode_change.is_some();
    let mut button = a11y::Semantics::new(Role::RadioButton, label)
        .id(semantics_id)
        .selected(checked)
        .disabled(!enabled)
        .focusable(enabled)
        .apply(button);
    if let Some(on_mode_change) = on_mode_change {
        button = a11y::click_void(button, move |window, cx| {
            on_mode_change(&mode, window, cx);
        });
    }
    button
}
