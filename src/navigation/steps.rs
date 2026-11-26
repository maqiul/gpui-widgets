//! 步骤条：向导流程的进度指示（完成 / 当前 / 待办 / 失败），
//! 支持横向 / 纵向 / 点状、默认（filled）与 outlined 两种外观。

use gpui::{
    App, Div, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    Stateful, Styled, Window, div, prelude::*, px, transparent_black,
};

use crate::a11y;
use crate::theme::{self, Tokens, metric};

/// 节点外观：antd 默认 `filled`，另有 `outlined`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepsVariant {
    Filled,
    Outlined,
}

/// 当前节点的状态覆盖：默认进行中，可切到失败。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepsStatus {
    Process,
    Error,
}

/// 单个步骤的状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepStatus {
    Wait,
    Process,
    Finish,
    Error,
}

/// 节点圆圈的纯数据外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepNodeStyle {
    pub bg: Hsla,
    pub border: Hsla,
    pub text: Hsla,
    pub border_width: f32,
}

/// 一个步骤的完整纯数据外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepState {
    pub node: StepNodeStyle,
    pub title: Hsla,
    pub content: Hsla,
    pub rail: Hsla,
}

/// 步骤节点状态（antd `steps/style/status.js`）。
///
/// `filled`：待办 `colorFillTertiary` + `colorTextLabel`；进行/完成 `colorPrimaryBg` +
/// `colorPrimary`；失败 `colorErrorBg` + `colorError`；当前节点反色（主色/错误色底 + 白字）。
/// `outlined`：容器底，待办灰描边，进行/完成主色描边，当前只加浅色底。
pub fn step_state(
    tokens: &Tokens,
    variant: StepsVariant,
    status: StepStatus,
    active: bool,
    dot: bool,
) -> StepState {
    let title = match status {
        StepStatus::Wait => tokens.color_text_description,
        StepStatus::Process | StepStatus::Finish => tokens.color_text,
        StepStatus::Error => tokens.color_error,
    };
    let content = match status {
        StepStatus::Wait if active => tokens.color_text,
        StepStatus::Wait => tokens.color_text_description,
        StepStatus::Process | StepStatus::Finish if active => tokens.color_text,
        StepStatus::Process | StepStatus::Finish => tokens.color_text_description,
        StepStatus::Error => tokens.color_error,
    };

    if dot {
        let bg = if active {
            if status == StepStatus::Error {
                tokens.color_error
            } else {
                tokens.color_primary
            }
        } else {
            match status {
                StepStatus::Wait => tokens.color_text_disabled,
                StepStatus::Process | StepStatus::Finish => tokens.color_primary,
                StepStatus::Error => tokens.color_error,
            }
        };
        return StepState {
            node: StepNodeStyle {
                bg,
                border: transparent_black(),
                text: if active {
                    tokens.color_text_light_solid
                } else {
                    bg
                },
                border_width: 2.0,
            },
            title,
            content,
            rail: step_rail_state(tokens, status),
        };
    }

    let (bg, border, text) = if active && variant == StepsVariant::Filled {
        let solid = if status == StepStatus::Error {
            tokens.color_error
        } else {
            tokens.color_primary
        };
        (solid, transparent_black(), tokens.color_text_light_solid)
    } else {
        match (variant, status) {
            (StepsVariant::Filled, StepStatus::Wait) => (
                tokens.color_fill_tertiary,
                transparent_black(),
                tokens.color_text_label,
            ),
            (StepsVariant::Filled, StepStatus::Process | StepStatus::Finish) => (
                tokens.color_primary_bg,
                transparent_black(),
                tokens.color_primary,
            ),
            (StepsVariant::Filled, StepStatus::Error) => (
                tokens.color_error_bg,
                transparent_black(),
                tokens.color_error,
            ),
            (StepsVariant::Outlined, StepStatus::Wait) => (
                tokens.color_bg_container,
                tokens.color_text_disabled,
                tokens.color_text_disabled,
            ),
            (StepsVariant::Outlined, StepStatus::Process | StepStatus::Finish) => (
                tokens.color_bg_container,
                tokens.color_primary,
                tokens.color_primary,
            ),
            (StepsVariant::Outlined, StepStatus::Error) => (
                tokens.color_bg_container,
                tokens.color_error,
                tokens.color_error,
            ),
        }
    };

    let bg = if active && variant == StepsVariant::Outlined {
        match status {
            StepStatus::Error => tokens.color_error_bg,
            StepStatus::Wait => tokens.color_fill_tertiary,
            StepStatus::Process | StepStatus::Finish => tokens.color_primary_bg,
        }
    } else {
        bg
    };

    StepState {
        node: StepNodeStyle {
            bg,
            border,
            text,
            border_width: tokens.line_width,
        },
        title,
        content,
        rail: step_rail_state(tokens, status),
    }
}

/// 节点右侧连接线的颜色：待办灰，其余主色，失败错误色。
pub fn step_rail_state(tokens: &Tokens, status: StepStatus) -> Hsla {
    match status {
        StepStatus::Wait => tokens.color_text_disabled,
        StepStatus::Process | StepStatus::Finish => tokens.color_primary,
        StepStatus::Error => tokens.color_error,
    }
}

/// 步骤条（受控）。
///
/// ```ignore
/// Steps::new(vec!["填写信息".into(), "确认订单".into(), "完成".into()])
///     .current(1)
///     .descriptions(vec!["基础资料".into(), "核对金额".into(), "下单成功".into()])
/// ```
#[derive(IntoElement)]
pub struct Steps {
    id: Option<SharedString>,
    origin: SharedString,
    titles: Vec<SharedString>,
    descriptions: Vec<SharedString>,
    current: usize,
    vertical: bool,
    small: bool,
    progress_dot: bool,
    variant: StepsVariant,
    status: StepsStatus,
}

impl Steps {
    #[track_caller]
    pub fn new(titles: Vec<SharedString>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "steps-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            titles,
            descriptions: Vec::new(),
            current: 0,
            vertical: false,
            small: false,
            progress_dot: false,
            variant: StepsVariant::Filled,
            status: StepsStatus::Process,
        }
    }

    /// 实例 ID；同页多个步骤条应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 当前步骤（0-based，之前的步骤视为已完成）
    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }

    /// 纵向排列
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }

    /// 小尺寸（24px 节点 / 14px 标题）
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    /// 点状步骤条：节点改为小圆点，不显示序号 / 对勾
    pub fn progress_dot(mut self) -> Self {
        self.progress_dot = true;
        self
    }

    /// 每个步骤的描述文字（按下标对应，缺省不显示）
    pub fn descriptions(mut self, descriptions: Vec<SharedString>) -> Self {
        self.descriptions = descriptions;
        self
    }

    /// 节点外观（默认 `Filled`）
    pub fn variant(mut self, variant: StepsVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 当前节点状态（默认 `Process`，可切 `Error`）
    pub fn status(mut self, status: StepsStatus) -> Self {
        self.status = status;
        self
    }

    fn status_at(&self, ix: usize) -> StepStatus {
        if ix < self.current {
            StepStatus::Finish
        } else if ix == self.current {
            match self.status {
                StepsStatus::Process => StepStatus::Process,
                StepsStatus::Error => StepStatus::Error,
            }
        } else {
            StepStatus::Wait
        }
    }

    fn rail_after(&self, ix: usize) -> Option<StepStatus> {
        (ix + 1 < self.titles.len()).then(|| self.status_at(ix + 1))
    }

    fn item_semantics(&self, ix: usize) -> a11y::Semantics {
        let status = match self.status_at(ix) {
            StepStatus::Wait => "待办",
            StepStatus::Process => "进行中",
            StepStatus::Finish => "已完成",
            StepStatus::Error => "失败",
        };
        let label = format!("{}，{status}", self.titles[ix]);
        let mut semantics = a11y::Semantics::new(Role::ListItem, label)
            .id(format!(
                "{}-step-{ix}",
                self.id.as_deref().unwrap_or(&self.origin)
            ))
            .position(ix + 1, self.titles.len());
        if let Some(description) = self.descriptions.get(ix) {
            semantics = semantics.description(description.clone());
        }
        semantics
    }

    fn icon(&self, ix: usize, tokens: &Tokens, m: &metric::StepsMetric) -> gpui::Div {
        let status = self.status_at(ix);
        let active = ix == self.current;
        let state = step_state(tokens, self.variant, status, active, false);
        let side = if self.small {
            m.icon_size_small
        } else {
            m.icon_size
        };
        let glyph = match status {
            StepStatus::Finish | StepStatus::Error => "✓".to_string(),
            StepStatus::Wait | StepStatus::Process => format!("{}", ix + 1),
        };
        div()
            .w(px(side))
            .h(px(side))
            .flex_shrink_0()
            .rounded_full()
            .border(px(state.node.border_width))
            .border_color(state.node.border)
            .bg(state.node.bg)
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(m.icon_font_size))
            .line_height(px(side))
            .text_color(state.node.text)
            .child(glyph)
    }

    fn title(&self, ix: usize, tokens: &Tokens, m: &metric::StepsMetric) -> Stateful<Div> {
        let state = step_state(
            tokens,
            self.variant,
            self.status_at(ix),
            ix == self.current,
            false,
        );
        let (size, line) = if self.small {
            (m.title_font_size_small, m.title_line_height_small)
        } else {
            (m.title_font_size, m.title_line_height)
        };
        a11y::Semantics::new(Role::Heading, self.titles[ix].clone())
            .id(format!(
                "{}-step-{ix}-heading",
                self.id.as_deref().unwrap_or(&self.origin)
            ))
            .level(3)
            .apply(
                div().id(ElementId::Name(
                    format!(
                        "{}-step-{ix}-heading",
                        self.id.as_deref().unwrap_or(&self.origin)
                    )
                    .into(),
                )),
            )
            .min_w_0()
            .text_size(px(size))
            .line_height(px(line))
            .text_color(state.title)
            .child(self.titles[ix].clone())
    }

    fn content(&self, ix: usize, tokens: &Tokens, m: &metric::StepsMetric) -> Option<gpui::Div> {
        let description = self.descriptions.get(ix)?;
        let state = step_state(
            tokens,
            self.variant,
            self.status_at(ix),
            ix == self.current,
            false,
        );
        Some(
            div()
                .text_size(px(m.content_font_size))
                .line_height(px(m.content_line_height))
                .text_color(state.content)
                .child(description.clone()),
        )
    }

    fn render_horizontal(self) -> Stateful<Div> {
        theme::with(|tokens| {
            let m = metric::StepsMetric::resolve(tokens);
            let last = self.titles.len().saturating_sub(1);
            let heading = if self.small {
                m.heading_height_small
            } else {
                m.heading_height
            };
            let item_gap = if self.small {
                m.item_gap_small
            } else {
                m.item_gap
            };
            let rail_gap = if self.small {
                m.rail_gap_small
            } else {
                m.rail_gap
            };

            let mut row = a11y::Semantics::new(Role::List, "步骤")
                .id(format!(
                    "{}-list",
                    self.id.as_deref().unwrap_or(&self.origin)
                ))
                .apply(div().id(ElementId::Name(
                    format!("{}-list", self.id.as_deref().unwrap_or(&self.origin)).into(),
                )))
                .flex()
                .flex_row()
                .items_start()
                .w_full();
            for ix in 0..self.titles.len() {
                let mut header = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(heading))
                    .w_full()
                    .child(self.title(ix, tokens, &m));
                if let Some(next) = self.rail_after(ix) {
                    header = header.child(
                        div()
                            .ml(px(rail_gap))
                            .flex_1()
                            .h(px(m.rail_width))
                            .bg(step_rail_state(tokens, next)),
                    );
                }

                let mut section = div().flex().flex_col().flex_1().min_w_0().child(header);
                if let Some(content) = self.content(ix, tokens, &m) {
                    section = section.child(content);
                }

                let wrapper = div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .min_w_0()
                    .gap(px(m.icon_gap))
                    .child(self.icon(ix, tokens, &m))
                    .child(section);

                let mut item = self
                    .item_semantics(ix)
                    .apply(div().id(ElementId::Name(
                        format!("{}-step-{ix}", self.id.as_deref().unwrap_or(&self.origin)).into(),
                    )))
                    .flex()
                    .flex_row()
                    .items_start()
                    .min_w_0()
                    .when(ix != 0, |item| item.ml(px(item_gap)));
                item = if ix == last {
                    item
                } else {
                    item.flex_grow(1.0)
                };
                row = row.child(item.child(wrapper));
            }
            row
        })
    }

    fn render_vertical(self) -> Stateful<Div> {
        theme::with(|tokens| {
            let m = metric::StepsMetric::resolve(tokens);
            let last = self.titles.len().saturating_sub(1);
            let heading = if self.small {
                m.heading_height_small
            } else {
                m.heading_height
            };
            let icon_side = if self.small {
                m.icon_size_small
            } else {
                m.icon_size
            };

            let mut col = a11y::Semantics::new(Role::List, "步骤")
                .id(format!(
                    "{}-list",
                    self.id.as_deref().unwrap_or(&self.origin)
                ))
                .apply(div().id(ElementId::Name(
                    format!("{}-list", self.id.as_deref().unwrap_or(&self.origin)).into(),
                )))
                .flex()
                .flex_col()
                .w_full();
            for ix in 0..self.titles.len() {
                let mut section = div().flex().flex_col().flex_1().min_w_0().child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .h(px(heading))
                        .child(self.title(ix, tokens, &m)),
                );
                if let Some(content) = self.content(ix, tokens, &m) {
                    section = section.child(content);
                }

                let item = self
                    .item_semantics(ix)
                    .apply(div().id(ElementId::Name(
                        format!("{}-step-{ix}", self.id.as_deref().unwrap_or(&self.origin)).into(),
                    )))
                    .relative()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap(px(m.icon_gap_vertical))
                    .pb(px(if ix == last { 0. } else { m.vertical_padding }))
                    .child(self.icon(ix, tokens, &m))
                    .child(section)
                    .children(self.rail_after(ix).map(|next| {
                        let left = icon_side / 2.0 - m.rail_width / 2.0;
                        div()
                            .absolute()
                            .left(px(left))
                            .top(px(icon_side + m.vertical_rail_margin))
                            .bottom(px(m.vertical_rail_margin))
                            .w(px(m.rail_width))
                            .bg(step_rail_state(tokens, next))
                    }));
                col = col.child(item);
            }
            col
        })
    }

    fn render_dot(self) -> Stateful<Div> {
        theme::with(|tokens| {
            let m = metric::StepsMetric::resolve(tokens);
            let last = self.titles.len().saturating_sub(1);
            let mut row = a11y::Semantics::new(Role::List, "步骤")
                .id(format!(
                    "{}-list",
                    self.id.as_deref().unwrap_or(&self.origin)
                ))
                .apply(div().id(ElementId::Name(
                    format!("{}-list", self.id.as_deref().unwrap_or(&self.origin)).into(),
                )))
                .flex()
                .flex_row()
                .items_start()
                .w_full();

            for ix in 0..self.titles.len() {
                let active = ix == self.current;
                let state = step_state(tokens, self.variant, self.status_at(ix), active, true);
                let side = if active {
                    m.dot_active_size
                } else {
                    m.dot_size
                };
                let mut dot_row = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .w_full()
                    .h(px(m.dot_active_size));
                if ix > 0 {
                    dot_row = dot_row.child(
                        div()
                            .flex_1()
                            .h(px(m.dot_rail_width))
                            .bg(step_rail_state(tokens, self.status_at(ix))),
                    );
                }
                dot_row = dot_row.child(
                    div()
                        .w(px(side))
                        .h(px(side))
                        .flex_shrink_0()
                        .rounded_full()
                        .bg(state.node.bg),
                );
                if let Some(next) = self.rail_after(ix) {
                    dot_row = dot_row.child(
                        div()
                            .flex_1()
                            .h(px(m.dot_rail_width))
                            .bg(step_rail_state(tokens, next)),
                    );
                }

                let title_size = if self.small {
                    m.title_font_size_small
                } else {
                    m.title_font_size
                };
                let title_line = if self.small {
                    m.title_line_height_small
                } else {
                    m.title_line_height
                };
                let item = self
                    .item_semantics(ix)
                    .apply(div().id(ElementId::Name(
                        format!("{}-step-{ix}", self.id.as_deref().unwrap_or(&self.origin)).into(),
                    )))
                    .flex()
                    .flex_col()
                    .items_center()
                    .flex_1()
                    .gap(px(m.dot_wrapper_gap))
                    .when(ix == last, |item| item.flex_shrink_0())
                    .child(dot_row)
                    .child(
                        a11y::Semantics::new(Role::Heading, self.titles[ix].clone())
                            .id(format!(
                                "{}-step-{ix}-heading",
                                self.id.as_deref().unwrap_or(&self.origin)
                            ))
                            .level(3)
                            .apply(
                                div().id(ElementId::Name(
                                    format!(
                                        "{}-step-{ix}-heading",
                                        self.id.as_deref().unwrap_or(&self.origin)
                                    )
                                    .into(),
                                )),
                            )
                            .w_full()
                            .text_center()
                            .text_size(px(title_size))
                            .line_height(px(title_line))
                            .text_color(state.title)
                            .child(self.titles[ix].clone()),
                    );
                row = row.child(item);
            }
            row
        })
    }
}

impl RenderOnce for Steps {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        if self.progress_dot {
            self.render_dot()
        } else if self.vertical {
            self.render_vertical()
        } else {
            self.render_horizontal()
        }
    }
}
