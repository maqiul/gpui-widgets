//! 分段选择器：圆角轨道内的互斥选项，适合 2~5 个短选项。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, IntoElement, Role, SharedString, Stateful, Styled,
    Window, div, prelude::*, px,
};

use crate::a11y;
use crate::motion;
use crate::theme::{self, Tokens, metric};

/// 选项的交互状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentedState {
    Normal,
    Hover,
    Active,
    Selected,
    Disabled,
    SelectedDisabled,
}

/// 选项的纯数据外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentedItemStyle {
    pub text: Hsla,
    pub bg: Hsla,
    pub shadow: bool,
}

/// 选项状态：选中换容器底 + 阴影，悬停叠 `colorFillSecondary`、按下叠 `colorFill`；
/// 禁用只换 `colorTextDisabled`，选中禁用仍保留选中底。
pub fn segmented_item_state(tokens: &Tokens, state: SegmentedState) -> SegmentedItemStyle {
    let elevated = tokens.color_bg_elevated;
    match state {
        SegmentedState::Normal => SegmentedItemStyle {
            text: tokens.color_text_label,
            bg: gpui::transparent_black(),
            shadow: false,
        },
        SegmentedState::Hover => SegmentedItemStyle {
            text: tokens.color_text,
            bg: tokens.color_fill_secondary,
            shadow: false,
        },
        SegmentedState::Active => SegmentedItemStyle {
            text: tokens.color_text,
            bg: tokens.color_fill,
            shadow: false,
        },
        SegmentedState::Selected => SegmentedItemStyle {
            text: tokens.color_text,
            bg: elevated,
            shadow: true,
        },
        SegmentedState::Disabled => SegmentedItemStyle {
            text: tokens.color_text_disabled,
            bg: gpui::transparent_black(),
            shadow: false,
        },
        SegmentedState::SelectedDisabled => SegmentedItemStyle {
            text: tokens.color_text_disabled,
            bg: elevated,
            shadow: true,
        },
    }
}

/// 轨道底色 `colorBgLayout`。
pub fn segmented_track_state(tokens: &Tokens) -> Hsla {
    tokens.color_bg_layout
}

/// 分段选择器。
///
/// ```ignore
/// Segmented::new(["日", "周", "月"], self.mode)
///     .on_select(cx.listener(|this, ix, _, cx| {
///         this.mode = *ix;
///         cx.notify();
///     }))
/// ```
pub struct Segmented {
    id: Option<SharedString>,
    origin: SharedString,
    items: Vec<SharedString>,
    active: usize,
    small: bool,
    large: bool,
    block: bool,
    disabled: bool,
    on_select: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl Segmented {
    #[track_caller]
    pub fn new(items: impl IntoIterator<Item = impl Into<SharedString>>, active: usize) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "segmented-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            items: items.into_iter().map(Into::into).collect(),
            active,
            small: false,
            large: false,
            block: false,
            disabled: false,
            on_select: None,
        }
    }

    /// 实例 ID；同页多个分段选择器应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 小尺寸（高 24，圆角 4/2）
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    /// 大尺寸（高 40，字号 16，圆角 8/6）
    pub fn large(mut self) -> Self {
        self.large = true;
        self
    }

    /// 撑满父容器宽度，选项均分
    pub fn block(mut self) -> Self {
        self.block = true;
        self
    }

    /// 全部选项禁用
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn on_select(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Segmented {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        theme::with(|tokens| {
            let m = metric::SegmentedMetric::resolve(tokens);
            let size = if self.small {
                0
            } else if self.large {
                2
            } else {
                1
            };
            let radius = m.container_radius[size];
            let owner = self.id.as_ref().unwrap_or(&self.origin);
            let list_id = format!("{owner}-tab-list");

            let item_count = self.items.len();
            let mut container = a11y::Semantics::new(Role::TabList, "分段选择")
                .id(list_id.clone())
                .apply(div().id(ElementId::Name(list_id.into())))
                .flex()
                .flex_row()
                .p(px(m.track_padding))
                .rounded(px(radius))
                .bg(segmented_track_state(tokens));
            if self.block {
                container = container.w_full();
            }

            for (ix, label) in self.items.into_iter().enumerate() {
                let selected = ix == self.active;
                let state = if self.disabled {
                    if selected {
                        SegmentedState::SelectedDisabled
                    } else {
                        SegmentedState::Disabled
                    }
                } else if selected {
                    SegmentedState::Selected
                } else {
                    SegmentedState::Normal
                };
                let style = SegmentedItemStyle {
                    bg: segmented_track_state(tokens),
                    ..segmented_item_state(tokens, state)
                };
                let selected_style = segmented_item_state(
                    tokens,
                    if self.disabled {
                        SegmentedState::SelectedDisabled
                    } else {
                        SegmentedState::Selected
                    },
                );
                let hover = segmented_item_state(tokens, SegmentedState::Hover);
                let on_select = self.on_select.clone();

                let segment_id = format!("{owner}-segment-{ix}");
                let selected_overlay = motion::AnimatedValue::new(
                    format!("{segment_id}-selected"),
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(px(m.item_radius[size]))
                        .bg(selected_style.bg),
                    f32::from(selected),
                    move |overlay, progress| overlay.opacity(progress),
                )
                .duration(if self.disabled {
                    std::time::Duration::ZERO
                } else {
                    motion::fast_duration()
                });
                let label_motion = motion::AnimatedValue::new(
                    format!("{segment_id}-label"),
                    div().child(label.clone()),
                    f32::from(selected),
                    move |label, progress| {
                        label.text_color(motion::mix_color(
                            style.text,
                            selected_style.text,
                            progress,
                        ))
                    },
                )
                .duration(if self.disabled {
                    std::time::Duration::ZERO
                } else {
                    motion::fast_duration()
                });
                let mut segment = a11y::Semantics::new(Role::Tab, label.clone())
                    .id(segment_id.clone())
                    .selected(selected)
                    .disabled(self.disabled)
                    .focusable(!self.disabled && self.on_select.is_some())
                    .position(ix + 1, item_count)
                    .apply(div().id(ElementId::Name(segment_id.clone().into())))
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .min_w_0()
                    .h(px(m.label_height[size]))
                    .px(px(m.padding_inline[size]))
                    .rounded(px(m.item_radius[size]))
                    .line_height(px(m.label_height[size]))
                    .text_size(px(m.font_size[size]))
                    .text_color(style.text)
                    .bg(segmented_track_state(tokens))
                    .cursor(if self.disabled {
                        CursorStyle::Arrow
                    } else {
                        CursorStyle::PointingHand
                    });

                segment = segment.child(selected_overlay);
                segment = segment.child(if selected {
                    label_motion.into_any_element()
                } else {
                    div().child(label).into_any_element()
                });

                if style.shadow {
                    segment = segment.shadow(tokens.shadow_tertiary.0.clone());
                }
                if self.block {
                    segment = segment.flex_1();
                }
                if !self.disabled {
                    if let Some(on_select) = on_select {
                        let keyboard = on_select.clone();
                        if !selected {
                            segment = a11y::click_void(segment, move |window, cx| {
                                on_select(&ix, window, cx);
                            });
                        }
                        segment = segment.on_key_down(move |event, window, cx| {
                            if let Some(target) =
                                segment_target(ix, event.keystroke.key.as_str(), item_count)
                            {
                                if target != ix {
                                    cx.stop_propagation();
                                    keyboard(&target, window, cx);
                                }
                            }
                        });
                    }
                }

                let mut segment_motion = motion::AnimatedValue::new(
                    format!("{segment_id}-hover"),
                    segment,
                    0.0,
                    move |segment, progress| {
                        segment
                            .bg(motion::mix_color(style.bg, hover.bg, progress))
                            .text_color(motion::mix_color(style.text, hover.text, progress))
                    },
                )
                .duration(if self.disabled {
                    std::time::Duration::ZERO
                } else {
                    motion::fast_duration()
                });
                if !self.disabled && !selected {
                    segment_motion = segment_motion.hover_refresh(motion::bind_hover_refresh);
                }
                container = container.child(segment_motion);
            }

            container
        })
    }
}

fn segment_target(current: usize, key: &str, len: usize) -> Option<usize> {
    let key = key.strip_prefix("arrow").unwrap_or(key);
    if len == 0 {
        return None;
    }
    match key {
        "right" | "down" => Some((current + 1) % len),
        "left" | "up" => Some((current + len - 1) % len),
        "home" => Some(0),
        "end" => Some(len - 1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::segment_target;

    #[test]
    fn segment_target_wraps_and_ignores_other_keys() {
        assert_eq!(segment_target(0, "right", 3), Some(1));
        assert_eq!(segment_target(0, "arrowright", 3), Some(1));
        assert_eq!(segment_target(2, "arrowright", 3), Some(0));
        assert_eq!(segment_target(0, "arrowleft", 3), Some(2));
        assert_eq!(segment_target(1, "home", 3), Some(0));
        assert_eq!(segment_target(1, "end", 3), Some(2));
        assert_eq!(segment_target(1, "enter", 3), None);
        assert_eq!(segment_target(0, "arrowright", 0), None);
    }
}
