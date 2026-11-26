//! 描述列表：成组的键值对展示（对应 antd Descriptions）。

use gpui::{
    App, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    SharedString, Stateful, Styled, Window, div, px,
};

use crate::{
    a11y::Semantics,
    theme::{
        self, Tokens,
        metric::{DESCRIPTIONS_TITLE_WEIGHT, DescriptionsMetric},
    },
};

/// 描述列表配色：无边框标签用 `colorTextTertiary`，带边框标签用
/// `colorTextSecondary` + `colorFillAlter` 底；分隔线 `colorSplit`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DescriptionsStyle {
    pub label_bg: Hsla,
    pub label_color: Hsla,
    pub label_color_bordered: Hsla,
    pub title_color: Hsla,
    pub content_color: Hsla,
    pub border_color: Hsla,
}

pub fn descriptions_style(tokens: &Tokens) -> DescriptionsStyle {
    DescriptionsStyle {
        label_bg: tokens.color_fill_alter,
        label_color: tokens.color_text_tertiary,
        label_color_bordered: tokens.color_text_secondary,
        title_color: tokens.color_text,
        content_color: tokens.color_text,
        border_color: tokens.color_split,
    }
}

/// 描述列表。
///
/// ```ignore
/// Descriptions::new("用户信息")
///     .column(2)
///     .item("姓名", "张三")
///     .item("电话", "13800000000")
///     .bordered()
/// ```
#[derive(IntoElement)]
pub struct Descriptions {
    id: Option<SharedString>,
    origin: SharedString,
    title: Option<SharedString>,
    column: usize,
    bordered: bool,
    vertical: bool,
    items: Vec<(SharedString, SharedString)>,
}

impl Descriptions {
    #[track_caller]
    pub fn new(title: impl Into<SharedString>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "descriptions-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            title: Some(title.into()),
            column: 3,
            bordered: false,
            vertical: false,
            items: Vec::new(),
        }
    }

    /// 无标题
    #[track_caller]
    pub fn untitled() -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "descriptions-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            title: None,
            column: 3,
            bordered: false,
            vertical: false,
            items: Vec::new(),
        }
    }

    /// 实例 ID；同页多个描述列表应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 每行键值对数量（默认 3）
    pub fn column(mut self, column: usize) -> Self {
        self.column = column.max(1);
        self
    }

    /// 带边框表格样式
    pub fn bordered(mut self) -> Self {
        self.bordered = true;
        self
    }

    /// 垂直布局：标签在上、值在下
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }

    pub fn item(mut self, label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.items.push((label.into(), value.into()));
        self
    }
}

struct Look {
    style: DescriptionsStyle,
    m: DescriptionsMetric,
}

fn label_cell(
    label: SharedString,
    look: &Look,
    bordered: bool,
    divider: bool,
    id: String,
) -> Stateful<gpui::Div> {
    let semantics = Semantics::new(Role::Term, label.clone()).id(id);
    let element_id = semantics.element_id();
    let cell = semantics.apply(
        div()
            .id(element_id)
            .flex_shrink_0()
            .px(px(look.m.bordered_padding_inline))
            .py(px(look.m.bordered_padding_block))
            .text_size(px(look.m.item_size))
            .line_height(px(look.m.line_height))
            .text_color(if bordered {
                look.style.label_color_bordered
            } else {
                look.style.label_color
            })
            .child(label),
    );
    let cell = if bordered {
        cell.bg(look.style.label_bg)
    } else {
        cell
    };
    if divider {
        cell.border_r_1().border_color(look.style.border_color)
    } else {
        cell
    }
}

fn value_cell(value: SharedString, look: &Look, last: bool, id: String) -> Stateful<gpui::Div> {
    let semantics = Semantics::new(Role::Definition, value.clone()).id(id);
    let element_id = semantics.element_id();
    let cell = semantics.apply(
        div()
            .id(element_id)
            .flex_1()
            .min_w_0()
            .px(px(look.m.bordered_padding_inline))
            .py(px(look.m.bordered_padding_block))
            .text_size(px(look.m.item_size))
            .line_height(px(look.m.line_height))
            .text_color(look.style.content_color)
            .child(value),
    );
    if last {
        cell
    } else {
        cell.border_r_1().border_color(look.style.border_color)
    }
}

impl RenderOnce for Descriptions {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let look = theme::with(|tokens| Look {
            style: descriptions_style(tokens),
            m: DescriptionsMetric::resolve(tokens),
        });

        let owner = self.id.as_deref().unwrap_or(&self.origin);
        let semantics = Semantics::new(Role::DescriptionList, "描述列表").id(format!(
            "{owner}-{}",
            self.title.as_deref().unwrap_or("untitled")
        ));
        let element_id = semantics.element_id();
        let mut root = semantics.apply(div().id(element_id).flex().flex_col().w_full());

        if let Some(title) = &self.title {
            let heading = Semantics::new(Role::Heading, title.clone())
                .id(format!("{owner}-title-{title}"))
                .level(2);
            let heading_id = heading.element_id();
            root = root.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .mb(px(look.m.title_margin_bottom))
                    .child(
                        heading.apply(
                            div()
                                .id(heading_id)
                                .flex_1()
                                .text_size(px(look.m.title_size))
                                .line_height(px(look.m.title_line_height))
                                .font_weight(FontWeight(DESCRIPTIONS_TITLE_WEIGHT))
                                .text_color(look.style.title_color)
                                .child(title.clone()),
                        ),
                    ),
            );
        }

        let mut body = div()
            .flex()
            .flex_col()
            .w_full()
            .text_size(px(look.m.item_size));
        if self.bordered {
            body = body
                .border_1()
                .border_color(look.style.border_color)
                .rounded(px(look.m.border_radius))
                .overflow_hidden();
        }

        let chunks: Vec<&[(SharedString, SharedString)]> = self.items.chunks(self.column).collect();
        let last_row = chunks.len().saturating_sub(1);
        for (row_ix, chunk) in chunks.iter().enumerate() {
            let mut row = div().flex().flex_row().w_full().items_start();
            if self.bordered {
                row = row.border_color(look.style.border_color);
                if row_ix < last_row {
                    row = row.border_b_1();
                }
            } else if row_ix > 0 {
                row = row.mt(px(if self.vertical {
                    look.m.item_padding_bottom
                } else {
                    0.0
                }));
            }

            let last_pair = chunk.len().saturating_sub(1);
            for (ix, (label, value)) in chunk.iter().enumerate() {
                if self.bordered && self.vertical {
                    let mut cell = div().flex().flex_col().flex_1().min_w_0();
                    if ix < last_pair {
                        cell = cell.border_r_1().border_color(look.style.border_color);
                    }
                    let term = Semantics::new(Role::Term, label.clone())
                        .id(format!("{owner}-{row_ix}-{ix}-term"));
                    let definition = Semantics::new(Role::Definition, value.clone())
                        .id(format!("{owner}-{row_ix}-{ix}-definition"));
                    let term_id = term.element_id();
                    let definition_id = definition.element_id();
                    cell = cell
                        .child(
                            term.apply(
                                div()
                                    .id(term_id)
                                    .px(px(look.m.bordered_padding_inline))
                                    .py(px(look.m.bordered_padding_block))
                                    .bg(look.style.label_bg)
                                    .border_b_1()
                                    .border_color(look.style.border_color)
                                    .text_size(px(look.m.item_size))
                                    .line_height(px(look.m.line_height))
                                    .text_color(look.style.label_color_bordered)
                                    .child(label.clone()),
                            ),
                        )
                        .child(
                            definition.apply(
                                div()
                                    .id(definition_id)
                                    .px(px(look.m.bordered_padding_inline))
                                    .py(px(look.m.bordered_padding_block))
                                    .text_size(px(look.m.item_size))
                                    .line_height(px(look.m.line_height))
                                    .text_color(look.style.content_color)
                                    .child(value.clone()),
                            ),
                        );
                    row = row.child(cell);
                } else if self.bordered {
                    let lbl = label_cell(
                        label.clone(),
                        &look,
                        true,
                        true,
                        format!("{owner}-{row_ix}-{ix}-term"),
                    );
                    let val = value_cell(
                        value.clone(),
                        &look,
                        ix == last_pair,
                        format!("{owner}-{row_ix}-{ix}-definition"),
                    );
                    row = row.child(lbl).child(val);
                } else if self.vertical {
                    let term = Semantics::new(Role::Term, label.clone())
                        .id(format!("{owner}-{row_ix}-{ix}-term"));
                    let definition = Semantics::new(Role::Definition, value.clone())
                        .id(format!("{owner}-{row_ix}-{ix}-definition"));
                    let term_id = term.element_id();
                    let definition_id = definition.element_id();
                    let mut pair = div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .child(
                            term.apply(
                                div()
                                    .id(term_id)
                                    .pb(px(look.m.item_padding_bottom))
                                    .text_size(px(look.m.item_size))
                                    .line_height(px(look.m.line_height))
                                    .text_color(look.style.label_color)
                                    .child(label.clone()),
                            ),
                        )
                        .child(
                            definition.apply(
                                div()
                                    .id(definition_id)
                                    .text_size(px(look.m.item_size))
                                    .line_height(px(look.m.line_height))
                                    .text_color(look.style.content_color)
                                    .child(value.clone()),
                            ),
                        );
                    if ix < last_pair {
                        pair = pair.pr(px(look.m.item_padding_end));
                    }
                    row = row.child(pair);
                } else {
                    let term = Semantics::new(Role::Term, label.clone())
                        .id(format!("{owner}-{row_ix}-{ix}-term"));
                    let definition = Semantics::new(Role::Definition, value.clone())
                        .id(format!("{owner}-{row_ix}-{ix}-definition"));
                    let term_id = term.element_id();
                    let definition_id = definition.element_id();
                    let mut pair = div()
                        .flex()
                        .flex_row()
                        .flex_1()
                        .min_w_0()
                        .items_start()
                        .child(
                            term.apply(
                                div()
                                    .id(term_id)
                                    .flex_shrink_0()
                                    .text_size(px(look.m.item_size))
                                    .line_height(px(look.m.line_height))
                                    .text_color(look.style.label_color)
                                    .child(label.clone()),
                            ),
                        )
                        .child(
                            div()
                                .flex_shrink_0()
                                .ml(px(look.m.colon_margin_left))
                                .mr(px(look.m.colon_margin_right))
                                .text_size(px(look.m.item_size))
                                .line_height(px(look.m.line_height))
                                .text_color(look.style.label_color)
                                .child(":"),
                        )
                        .child(
                            definition.apply(
                                div()
                                    .id(definition_id)
                                    .flex_1()
                                    .min_w_0()
                                    .text_size(px(look.m.item_size))
                                    .line_height(px(look.m.line_height))
                                    .text_color(look.style.content_color)
                                    .child(value.clone()),
                            ),
                        );
                    if ix < last_pair {
                        pair = pair.pr(px(look.m.item_padding_end));
                    }
                    row = row.child(pair);
                }
            }
            body = body.child(row);
        }

        root.child(body)
    }
}
