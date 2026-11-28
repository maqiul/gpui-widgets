//! 单选按钮控件：antdv-next Radio（16px 圆 + 6px 内点）。
//!
//! 单选组的状态（当前选中项）由父视图持有，
//! 每个 `Radio` 只接收 `selected` 与 `on_select` 回调。

use std::rc::Rc;

use gpui::{
    App, ClickEvent, CursorStyle, Div, ElementId, Hsla, IntoElement, Role, SharedString, Stateful,
    Window, div, prelude::*, px, transparent_black,
};

use crate::{
    a11y, motion,
    theme::{self, Tokens, metric},
};

/// 圆形外观（antdv-next Radio 实测）：
/// 未选中 `colorBgContainer` + `colorBorder`；选中主色填充/描边 + 6px 白色内点；
/// 禁用底 `colorBgContainerDisabled`、描边 `colorBorder`、内点/标签 `colorTextDisabled`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadioStyle {
    pub bg: Hsla,
    pub border: Hsla,
    pub hover_border: Hsla,
    pub dot: Hsla,
    pub label: Hsla,
    pub size: f32,
    pub dot_size: f32,
    pub radius: f32,
    pub line_width: f32,
    pub label_gap: f32,
    pub font_size: f32,
    pub line_height: f32,
}

pub fn radio_state(tokens: &Tokens, selected: bool, disabled: bool) -> RadioStyle {
    let m = metric::RadioMetric::resolve(tokens);
    RadioStyle {
        bg: if selected {
            if disabled {
                tokens.color_bg_container_disabled
            } else {
                tokens.color_primary
            }
        } else if disabled {
            tokens.color_bg_container_disabled
        } else {
            tokens.color_bg_container
        },
        border: if selected && !disabled {
            tokens.color_primary
        } else {
            tokens.color_border
        },
        hover_border: if disabled {
            tokens.color_border
        } else if selected {
            tokens.color_primary_hover
        } else {
            tokens.color_primary
        },
        dot: if !selected {
            transparent_black()
        } else if disabled {
            tokens.color_text_disabled
        } else {
            tokens.color_text_light_solid
        },
        label: if disabled {
            tokens.color_text_disabled
        } else {
            tokens.color_text
        },
        size: m.size,
        dot_size: m.dot,
        radius: m.radius,
        line_width: m.line_width,
        label_gap: m.label_gap,
        font_size: m.font_size,
        line_height: m.line_height,
    }
}

/// 通用单选按钮。
///
/// ```ignore
/// for (ix, label) in ["免费版", "专业版"].iter().enumerate() {
///     row = row.child(
///         Radio::new(*label, self.plan == ix)
///             .on_select(cx.listener(move |this, _, _window, cx| {
///                 this.plan = ix;
///                 cx.notify();
///             })),
///     );
/// }
/// ```
pub struct Radio {
    label: SharedString,
    id: Option<SharedString>,
    origin: SharedString,
    selected: bool,
    disabled: bool,
    on_select: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
}

impl Radio {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>, selected: bool) -> Self {
        let location = core::panic::Location::caller();
        Self {
            label: label.into(),
            id: None,
            origin: format!(
                "radio-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            selected,
            disabled: false,
            on_select: None,
        }
    }

    /// 动态列表中反复在同一构造点创建时应显式传入唯一 id。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Radio {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let (rest, selected) = theme::with(|t| {
            (
                radio_state(t, false, self.disabled),
                radio_state(t, self.selected, self.disabled),
            )
        });
        let circle = div()
            .relative()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .size(px(rest.size))
            .rounded(px(rest.radius))
            .border(px(rest.line_width))
            .border_color(rest.border)
            .bg(rest.bg);

        let id = ElementId::Name(
            self.id
                .clone()
                .unwrap_or_else(|| format!("{}-{}", self.origin, self.label).into()),
        );
        let motion_id = format!("radio-motion-{id:?}");
        let hover_id = format!("radio-hover-{id:?}");
        let selected_value = f32::from(self.selected);
        let hover_border = if self.selected {
            selected.hover_border
        } else {
            rest.hover_border
        };
        let circle = circle.id(ElementId::Name(format!("{hover_id:?}-surface").into()));
        let mut circle = motion::AnimatedValue::new_with_interaction(
            motion_id,
            circle,
            selected_value,
            move |mut element, progress, interaction| {
                let selected_border = motion::mix_color(rest.border, selected.border, progress);
                element = element
                    .bg(motion::mix_color(rest.bg, selected.bg, progress))
                    .border_color(if self.disabled {
                        selected_border
                    } else {
                        motion::interactive_color(
                            selected_border,
                            hover_border,
                            hover_border,
                            interaction,
                        )
                    });
                if progress > 0.001 {
                    element = element.child(
                        div()
                            .size(px(selected.dot_size * progress))
                            .rounded(px(selected.dot_size * progress / 2.0))
                            .bg(selected.dot)
                            .opacity(progress),
                    );
                }
                element
            },
        )
        .duration(motion::fast_duration());
        if !self.disabled {
            circle = circle.hover_refresh_value(selected_value, motion::bind_hover_refresh);
        } else {
            circle = circle.duration(std::time::Duration::ZERO);
        }
        let mut element = div()
            .id(id.clone())
            .flex()
            .flex_row()
            .items_center()
            .h(px(rest.line_height))
            .gap(px(rest.label_gap))
            .child(circle);

        if !self.label.is_empty() {
            element = element.child(
                div()
                    .text_size(px(rest.font_size))
                    .line_height(px(rest.line_height))
                    .text_color(rest.label)
                    .child(self.label.clone()),
            );
        }

        let semantics = a11y::Semantics::new(Role::RadioButton, self.label.clone())
            .id(format!("{:?}", id))
            .selected(self.selected)
            .disabled(self.disabled)
            .focusable(!self.disabled && self.on_select.is_some());
        element = semantics.apply(element);

        if self.disabled {
            return element.cursor(CursorStyle::Arrow);
        }

        element = element.cursor(CursorStyle::PointingHand);

        if let Some(on_select) = self.on_select {
            element = a11y::click_shared(element, on_select);
        }

        element
    }
}
