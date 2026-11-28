//! 复选框控件：antdv-next Checkbox（16px 方框 + 8px 半选方块）。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, IntoElement, Role, SharedString, Stateful, Toggled,
    Window, div, prelude::*, px,
};

use crate::{
    a11y, motion,
    theme::{self, Tokens, metric},
};

/// 方框外观（antdv-next Checkbox 实测）：
/// 未选中 `colorBgContainer` + `colorBorder`；选中主色填充 + 白色勾；
/// 半选保持容器底 + 8px 主色方块；禁用底色 `colorBgContainerDisabled`、
/// 勾/方块 `colorTextDisabled`；未选中悬停边框主色，选中悬停 `colorPrimaryHover`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckboxStyle {
    pub bg: Hsla,
    pub border: Hsla,
    pub hover_border: Hsla,
    pub mark: Hsla,
    pub label: Hsla,
    pub size: f32,
    pub radius: f32,
    pub line_width: f32,
    pub mark_block: f32,
    pub mark_glyph: f32,
    pub label_gap: f32,
    pub font_size: f32,
    pub line_height: f32,
}

pub fn checkbox_state(
    tokens: &Tokens,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
) -> CheckboxStyle {
    let m = metric::CheckboxMetric::resolve(tokens);
    let marked = checked || indeterminate;
    CheckboxStyle {
        bg: if checked {
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
        border: if checked && !disabled {
            tokens.color_primary
        } else {
            tokens.color_border
        },
        hover_border: if disabled {
            tokens.color_border
        } else if checked {
            tokens.color_primary_hover
        } else {
            tokens.color_primary
        },
        mark: if !marked {
            gpui::transparent_black()
        } else if disabled {
            tokens.color_text_disabled
        } else if checked {
            tokens.color_text_light_solid
        } else {
            tokens.color_primary
        },
        label: if disabled {
            tokens.color_text_disabled
        } else {
            tokens.color_text
        },
        size: m.size,
        radius: m.radius,
        line_width: m.line_width,
        mark_block: m.mark_block,
        mark_glyph: m.mark_glyph,
        label_gap: m.label_gap,
        font_size: m.font_size,
        line_height: m.line_height,
    }
}

/// 通用复选框。
///
/// ```ignore
/// Checkbox::new("同意服务条款", self.agreed)
///     .on_change(cx.listener(|this, checked, _window, cx| {
///         this.agreed = *checked;
///         cx.notify();
///     }))
/// ```
pub struct Checkbox {
    label: Option<SharedString>,
    id: Option<SharedString>,
    origin: SharedString,
    checked: bool,
    indeterminate: bool,
    disabled: bool,
    on_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
}

impl Checkbox {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>, checked: bool) -> Self {
        let location = core::panic::Location::caller();
        Self {
            label: Some(label.into()),
            id: None,
            origin: format!(
                "checkbox-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            checked,
            indeterminate: false,
            disabled: false,
            on_change: None,
        }
    }

    /// 动态列表中反复在同一构造点创建时应显式传入唯一 id。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 不带标签的复选框
    #[track_caller]
    pub fn unlabeled(checked: bool) -> Self {
        let location = core::panic::Location::caller();
        Self {
            label: None,
            id: None,
            origin: format!(
                "checkbox-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            checked,
            indeterminate: false,
            disabled: false,
            on_change: None,
        }
    }

    /// 半选态（仅视觉：方框内显示方块，常用于「全选」父节点）
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Checkbox {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let checked = self.checked;
        let id = ElementId::Name(self.id.clone().unwrap_or_else(|| {
            format!(
                "{}-{}",
                self.origin,
                self.label.as_deref().unwrap_or("checkbox")
            )
            .into()
        }));
        let aria_label = self
            .label
            .clone()
            .unwrap_or_else(|| SharedString::from("Checkbox"));
        let (rest, selected) = theme::with(|t| {
            (
                checkbox_state(t, false, false, self.disabled),
                checkbox_state(t, checked, self.indeterminate, self.disabled),
            )
        });
        let marked = checked || self.indeterminate;
        let hover_border = if marked {
            selected.hover_border
        } else {
            rest.hover_border
        };
        let box_element = div()
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

        let motion_id = format!("checkbox-motion-{id:?}");
        let hover_id = format!("checkbox-hover-{id:?}");
        let box_element = box_element.id(ElementId::Name(format!("{hover_id:?}-surface").into()));
        let mut box_element = motion::AnimatedValue::new_with_interaction(
            motion_id,
            box_element,
            f32::from(marked),
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
                    let mark = if checked {
                        div()
                            .text_size(px(selected.mark_glyph * progress))
                            .line_height(px(selected.size))
                            .text_color(selected.mark)
                            .opacity(progress)
                            .child("✓")
                    } else {
                        div()
                            .size(px(selected.mark_block * progress))
                            .bg(selected.mark)
                            .opacity(progress)
                    };
                    element = element.child(mark);
                }
                element
            },
        )
        .duration(motion::fast_duration());
        if !self.disabled {
            box_element =
                box_element.hover_refresh_value(f32::from(marked), motion::bind_hover_refresh);
        } else {
            box_element = box_element.duration(std::time::Duration::ZERO);
        }

        let mut element = div()
            .id(id.clone())
            .flex()
            .flex_row()
            .items_center()
            .h(px(rest.line_height))
            .gap(px(rest.label_gap))
            .child(box_element);

        if let Some(label) = self.label {
            element = element.child(
                div()
                    .text_size(px(rest.font_size))
                    .line_height(px(rest.line_height))
                    .text_color(rest.label)
                    .child(label),
            );
        }

        if self.disabled {
            let semantics = a11y::Semantics::new(Role::CheckBox, aria_label.clone())
                .id(format!("{:?}", id))
                .toggled(if self.indeterminate {
                    Toggled::Mixed
                } else if checked {
                    Toggled::True
                } else {
                    Toggled::False
                })
                .disabled(true);
            return semantics.apply(element).cursor(CursorStyle::Arrow);
        }

        let semantics = a11y::Semantics::new(Role::CheckBox, aria_label.clone())
            .id(format!("{:?}", id))
            .toggled(if self.indeterminate {
                Toggled::Mixed
            } else if checked {
                Toggled::True
            } else {
                Toggled::False
            })
            .focusable(self.on_change.is_some());
        element = semantics.apply(element).cursor(CursorStyle::PointingHand);

        if let Some(on_change) = self.on_change {
            element = a11y::click(element, move |_, window, cx| {
                on_change(&!checked, window, cx)
            });
        }

        element
    }
}
