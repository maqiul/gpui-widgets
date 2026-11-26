//! 日期选择器：输入族触发框 + antd 日历浮层（月导航 / 今日快捷 / 选中高亮）。
//!
//! 值与展开状态受控；显示的月份也受控（`month` + `on_month_change`），
//! 未指定时默认显示选中值所在月（无值则当前月）。

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
use crate::svg_path::{paint_svg, svg_segments};
use crate::text_input::{self, InputSize, InputStatus, InputVariant};
use crate::theme::metric::{date_cell_state, date_picker_metric, date_picker_style};
use crate::{popup::anchored, theme};

/// 日期（年, 月, 日）
pub type Date = (i32, u32, u32);

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

/// 日期选择器。
///
/// ```ignore
/// DatePicker::new("due", self.date)
///     .open(self.date_open)
///     .on_open_change(cx.listener(|this, open, _, cx| { this.date_open = *open; cx.notify(); }))
///     .on_change(cx.listener(|this, date, _, cx| { this.date = *date; cx.notify(); }))
///     .month(self.date_month)
///     .on_month_change(cx.listener(|this, m, _, cx| { this.date_month = Some(*m); cx.notify(); }))
/// ```
pub struct DatePicker {
    id: ElementId,
    value: Option<Date>,
    open: bool,
    disabled: bool,
    allow_clear: bool,
    placeholder: SharedString,
    month: Option<(i32, u32)>,
    size: InputSize,
    variant: InputVariant,
    status: InputStatus,
    on_change: Option<Rc<dyn Fn(&Option<Date>, &mut Window, &mut App)>>,
    on_clear: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_open_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    on_month_change: Option<Rc<dyn Fn(&(i32, u32), &mut Window, &mut App)>>,
}

impl DatePicker {
    pub fn new(id: impl Into<ElementId>, value: Option<Date>) -> Self {
        Self {
            id: id.into(),
            value,
            open: false,
            disabled: false,
            allow_clear: false,
            placeholder: "选择日期…".into(),
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

    /// 点击清空按钮时回调（未设置时回退为 on_change(None)）
    pub fn on_clear(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear = Some(Rc::new(listener));
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 日历当前显示的月份（受控），默认取选中值所在月或当前月
    pub fn month(mut self, month: Option<(i32, u32)>) -> Self {
        self.month = month;
        self
    }

    pub fn on_change(
        mut self,
        listener: impl Fn(&Option<Date>, &mut Window, &mut App) + 'static,
    ) -> Self {
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

    /// 点击月导航时回调目标月份
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

impl IntoElement for DatePicker {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let open = self.open && !self.disabled;
        let suppress_toggle = Rc::new(Cell::new(false));
        let m = theme::with(date_picker_metric);
        let ps = theme::with(date_picker_style);
        // antd 的 open 受控属性不置 focused：展开态触发框仍用默认描边
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
            Some((y, mo, d)) => format!("{y:04}-{mo:02}-{d:02}"),
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
            let on_clear = self.on_clear.clone();
            let on_change = self.on_change.clone();
            let clear = a11y::Semantics::new(Role::Button, "Clear date")
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

        if let Some(shadow) = &style.shadow {
            trigger = trigger.shadow(vec![shadow.clone()]);
        }
        if !self.disabled {
            let border = hover_style.border_color();
            trigger = trigger
                .cursor(CursorStyle::PointingHand)
                .hover(move |s| s.border_color(border));
        } else {
            trigger = trigger.cursor(CursorStyle::Arrow);
        }
        let mut semantics = a11y::Semantics::new(Role::ComboBox, "Date")
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

        let mut popup = None;
        if open {
            popup = Some(div().child(self.calendar(&suppress_toggle)));
        }

        anchored(trigger.into_any_element(), popup)
    }
}

impl DatePicker {
    /// 日历浮层内容
    fn calendar(&self, suppress_toggle: &Rc<Cell<bool>>) -> Stateful<Div> {
        let m = theme::with(date_picker_metric);
        let ps = theme::with(date_picker_style);
        let today = today();
        let (year, month) = self
            .month
            .or(self.value.map(|(y, mo, _)| (y, mo)))
            .unwrap_or((today.0, today.1));

        let shell = div()
            .id(ElementId::Name(format!("{:?}-calendar", self.id).into()))
            .occlude()
            .flex()
            .flex_col()
            .w(px(m.panel_width))
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
        let mut shell = a11y::Semantics::new(Role::Dialog, "Date calendar")
            .id(format!("{:?}-calendar", self.id))
            .apply(shell);

        // 头部：年/月导航 + 标题
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
        shell = shell.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(m.header_height))
                .px(px(m.header_pad_h))
                .border_b_1()
                .border_color(ps.split)
                .child(nav_button(
                    ElementId::Name(format!("{:?}-prev-y", self.id).into()),
                    DOUBLE_LEFT_ICON,
                    "Previous year",
                    (year - 1, month),
                    m.nav_icon,
                    m.font_size * 1.6,
                    &ps,
                    self.on_month_change.clone(),
                ))
                .child(nav_button(
                    ElementId::Name(format!("{:?}-prev-m", self.id).into()),
                    LEFT_ICON,
                    "Previous month",
                    prev,
                    m.nav_icon,
                    m.font_size * 1.6,
                    &ps,
                    self.on_month_change.clone(),
                ))
                .child(
                    div()
                        .flex_1()
                        .text_center()
                        .text_size(px(m.font_size))
                        .line_height(px(m.header_height))
                        .font_weight(gpui::FontWeight(m.title_weight))
                        .text_color(ps.header_fg)
                        .child(format!("{} {year}", MONTH_NAMES[(month - 1) as usize])),
                )
                .child(nav_button(
                    ElementId::Name(format!("{:?}-next-m", self.id).into()),
                    RIGHT_ICON,
                    "Next month",
                    next,
                    m.nav_icon,
                    m.font_size * 1.6,
                    &ps,
                    self.on_month_change.clone(),
                ))
                .child(nav_button(
                    ElementId::Name(format!("{:?}-next-y", self.id).into()),
                    DOUBLE_RIGHT_ICON,
                    "Next year",
                    (year + 1, month),
                    m.nav_icon,
                    m.font_size * 1.6,
                    &ps,
                    self.on_month_change.clone(),
                )),
        );

        let grid_id = format!("{:?}-calendar-grid", self.id);
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
                .py(px(m.body_pad_v))
                .px(px(m.pad_h)),
        );

        // 星期表头（周日起始，与 en 参照一致）
        let mut week = div().flex().flex_row();
        for (ix, name) in WEEK_NAMES.into_iter().enumerate() {
            let header = a11y::Semantics::new(Role::ColumnHeader, WEEKDAY_NAMES[ix])
                .id(format!("{:?}-weekday-{ix}", self.id))
                .column(ix + 1, WEEK_NAMES.len())
                .apply(
                    div()
                        .id(ElementId::Name(
                            format!("{:?}-weekday-{ix}", self.id).into(),
                        ))
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

        // 6 × 7 日期网格
        let offset = ((weekday_monday0(year, month, 1) + 1) % 7) as usize;
        let dim = days_in_month(year, month) as usize;
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
                let cs = theme::with(|t| {
                    date_cell_state(
                        t,
                        in_view,
                        in_view && today == date,
                        in_view && self.value == Some(date),
                    )
                });
                let mut inner = div()
                    .w(px(m.cell_inner))
                    .h(px(m.cell_inner))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(m.cell_radius))
                    .text_size(px(m.font_size))
                    .line_height(px(m.cell_inner))
                    .text_color(cs.fg)
                    .child(format!("{shown}"));
                if let Some(bg) = cs.bg {
                    inner = inner.bg(bg);
                }
                if let Some(border) = cs.border {
                    inner = inner.border_1().border_color(border);
                }
                let cell = div()
                    .id(ElementId::Name(format!("{:?}-d-{n}", self.id).into()))
                    .w(px(m.cell_width))
                    .h(px(m.th_height))
                    .flex()
                    .items_center()
                    .justify_center();
                let mut cell = a11y::Semantics::new(
                    Role::GridCell,
                    format!("{} {shown}, {year}", MONTH_NAMES[(month - 1) as usize]),
                )
                .id(format!("{:?}-d-{n}", self.id))
                .value(format!("{shown:02}"))
                .selected(in_view && self.value == Some(date))
                .row(row + 1, 6)
                .column(col + 1, 7)
                .disabled(!in_view)
                .focusable(in_view)
                .apply(cell);
                if in_view {
                    cell = cell.cursor(CursorStyle::PointingHand);
                    let hover_bg = ps.cell_hover_bg;
                    let highlighted = cs.bg.is_some();
                    if !highlighted {
                        inner = inner.hover(move |s| s.bg(hover_bg));
                    }
                    let on_change = self.on_change.clone();
                    let on_open_change = self.on_open_change.clone();
                    cell = a11y::click_void(cell, move |window, cx| {
                        if let Some(on_change) = &on_change {
                            on_change(&Some(date), window, cx);
                        }
                        if let Some(on_open_change) = &on_open_change {
                            on_open_change(&false, window, cx);
                        }
                    });
                }
                row_div = row_div.child(cell.child(inner));
            }
            body = body.child(row_div);
        }
        shell = shell.child(body);

        // 「Today」快捷行
        let on_change = self.on_change.clone();
        let on_open_change = self.on_open_change.clone();
        shell = shell.child(
            div()
                .flex()
                .flex_row()
                .justify_center()
                .border_t_1()
                .border_color(ps.split)
                .px(px(m.footer_pad_h))
                .child({
                    let today_button = a11y::Semantics::new(Role::Button, "Today")
                        .id(format!("{:?}-today", self.id))
                        .value(format!("{:04}-{:02}-{:02}", today.0, today.1, today.2))
                        .selected(self.value == Some(today))
                        .focusable(true)
                        .apply(
                            div()
                                .id(ElementId::Name(format!("{:?}-today", self.id).into()))
                                .line_height(px(m.footer_line))
                                .text_size(px(m.font_size))
                                .text_color(ps.link)
                                .cursor(CursorStyle::PointingHand)
                                .child("Today"),
                        );
                    a11y::click_void(today_button, move |window, cx| {
                        if let Some(on_change) = &on_change {
                            on_change(&Some(today), window, cx);
                        }
                        if let Some(on_open_change) = &on_open_change {
                            on_open_change(&false, window, cx);
                        }
                    })
                }),
        );

        shell
    }
}

/// 月/年导航按钮
fn nav_button(
    id: ElementId,
    d: &'static str,
    label: &'static str,
    target: (i32, u32),
    size: f32,
    m_nav_width: f32,
    ps: &crate::theme::metric::DatePickerStyle,
    on_month_change: Option<Rc<dyn Fn(&(i32, u32), &mut Window, &mut App)>>,
) -> Stateful<Div> {
    let color = ps.icon;
    let hover = ps.icon_hover;
    let semantics_id = format!("{:?}", id);
    let group = gpui::SharedString::from(format!("dp-nav-{id:?}"));
    let mut button = div()
        .id(ElementId::Name(semantics_id.clone().into()))
        .group(group.clone())
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .w(px(m_nav_width))
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

/// 公历转天数（Howard Hinnant 算法，1970-01-01 为第 0 天）
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = y as i64 - if m <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// 天数转公历
fn civil_from_days(z: i64) -> Date {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((y + if m <= 2 { 1 } else { 0 }) as i32, m, d)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub(crate) fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// 星期（周一 = 0 … 周日 = 6）
pub(crate) fn weekday_monday0(y: i32, m: u32, d: u32) -> i64 {
    (days_from_civil(y, m, d) + 3).rem_euclid(7)
}

/// 当前日期（UTC，演示用途足够）
pub fn today() -> Date {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    civil_from_days((secs / 86400) as i64)
}
