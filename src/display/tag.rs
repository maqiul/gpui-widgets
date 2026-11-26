//! 标签控件：比徽章更紧凑的标记，可带关闭按钮（常用于筛选条件）。

use std::rc::Rc;

use gpui::{
    App, ClickEvent, CursorStyle, Div, ElementId, Hsla, IntoElement, Role, SharedString, Stateful,
    Styled, Window, black, div, prelude::*, px, transparent_black,
};

use crate::{
    a11y,
    theme::{self, PresetColor, Tokens, metric},
};

/// 标签颜色：默认灰、antd 预置色、状态色（success / processing / error / warning）。
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum TagColor {
    #[default]
    Default,
    Blue,
    Green,
    Orange,
    Red,
    Purple,
    Cyan,
    Magenta,
    Success,
    Processing,
    Error,
    Warning,
}

impl TagColor {
    fn preset(self) -> Option<PresetColor> {
        match self {
            TagColor::Blue => Some(PresetColor::Blue),
            TagColor::Green => Some(PresetColor::Green),
            TagColor::Orange => Some(PresetColor::Orange),
            TagColor::Red => Some(PresetColor::Red),
            TagColor::Purple => Some(PresetColor::Purple),
            TagColor::Cyan => Some(PresetColor::Cyan),
            TagColor::Magenta => Some(PresetColor::Magenta),
            TagColor::Default | TagColor::Success | TagColor::Processing => None,
            TagColor::Error | TagColor::Warning => None,
        }
    }
}

/// 标签外观变体（antdv-next Tag 的 `variant`）。
/// 未设置时 antd 取 ConfigProvider 的 variant，缺省为 `filled`（`useColor`）。
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum TagVariant {
    Outlined,
    #[default]
    Filled,
    Solid,
}

/// 某一状态下标签的最终外观（纯数据，可直接和浏览器基准比对）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TagStyle {
    pub bg: Hsla,
    pub fg: Hsla,
    pub border: Hsla,
    pub border_width: f32,
    pub radius: f32,
    pub padding_inline: f32,
    pub font_size: f32,
    pub line_height: f32,
    /// 关闭图标：默认色 / 悬停色 / 左边距 / 字号
    pub icon: Hsla,
    pub icon_hover: Hsla,
    pub icon_margin: f32,
    pub icon_size: f32,
}

/// 标签状态矩阵：颜色 × 变体 × 禁用 → 外观。
///
/// 与 `button::state` / `text_input::state` 一样只依赖 `Tokens`，便于逐格对照 antd 基准。
pub fn state(color: TagColor, variant: TagVariant, tokens: &Tokens, disabled: bool) -> TagStyle {
    let m = metric::TagMetric::resolve(tokens);
    let transparent = transparent_black();
    let white_text = tokens.color_text_light_solid;

    let default_bg = theme::blend(tokens.color_fill_tertiary, tokens.color_bg_container);

    let preset = color.preset().map(theme::preset_palette);
    let status = match color {
        TagColor::Success => Some((
            tokens.color_success,
            tokens.color_success_bg,
            tokens.color_success_border,
        )),
        TagColor::Processing => Some((
            tokens.color_info,
            tokens.color_info_bg,
            tokens.color_info_border,
        )),
        TagColor::Error => Some((
            tokens.color_error,
            tokens.color_error_bg,
            tokens.color_error_border,
        )),
        TagColor::Warning => Some((
            tokens.color_warning,
            tokens.color_warning_bg,
            tokens.color_warning_border,
        )),
        _ => None,
    };

    let (mut bg, mut fg, mut border) = if let Some(p) = preset {
        match variant {
            TagVariant::Outlined => (p.light.hsla(), p.text.hsla(), p.border.hsla()),
            TagVariant::Filled => (p.light.hsla(), p.text.hsla(), transparent),
            TagVariant::Solid => (p.solid.hsla(), white_text, p.solid.hsla()),
        }
    } else if let Some((solid, light, outline)) = status {
        match variant {
            TagVariant::Outlined => (light, solid, outline),
            TagVariant::Filled => (light, solid, transparent),
            TagVariant::Solid => (solid, white_text, solid),
        }
    } else {
        match variant {
            TagVariant::Outlined => (default_bg, tokens.color_text, tokens.color_border),
            TagVariant::Filled => (default_bg, tokens.color_text, transparent),
            TagVariant::Solid => {
                let solid = tokens.color_bg_solid;
                let text = if is_bright(solid) {
                    black()
                } else {
                    tokens.color_text_light_solid
                };
                (solid, text, transparent)
            }
        }
    };

    if disabled {
        bg = tokens.color_bg_container_disabled;
        fg = tokens.color_text_disabled;
        border = if variant == TagVariant::Outlined {
            tokens.color_border
        } else {
            transparent
        };
    }

    let (icon, icon_hover) = if disabled {
        (tokens.color_text_disabled, tokens.color_text_disabled)
    } else {
        (tokens.color_icon, tokens.color_text_heading)
    };

    TagStyle {
        bg,
        fg,
        border,
        border_width: m.border_width,
        radius: m.radius,
        padding_inline: m.padding_inline,
        font_size: m.font_size,
        line_height: m.line_height,
        icon,
        icon_hover,
        icon_margin: m.icon_margin,
        icon_size: m.icon_size,
    }
}

/// antd `isBright`：半透明色先叠白底取 HSV 明度，实色按 0.299/0.587/0.114 加权与 192/255 比较。
fn is_bright(color: Hsla) -> bool {
    let rgb = color.to_rgb();
    if rgb.a <= 0.5 {
        let over = theme::blend(color, gpui::white());
        let rgb = over.to_rgb();
        rgb.r.max(rgb.g).max(rgb.b) > 0.5
    } else {
        0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b > 192.0 / 255.0
    }
}

/// 通用标签。
///
/// ```ignore
/// Tag::new("Rust").color(TagColor::Blue).variant(TagVariant::Solid)
/// ```
pub struct Tag {
    label: SharedString,
    id: Option<SharedString>,
    color: TagColor,
    variant: TagVariant,
    bordered: bool,
    disabled: bool,
    on_close: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
}

impl Tag {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            id: None,
            color: TagColor::default(),
            variant: TagVariant::default(),
            bordered: true,
            disabled: false,
            on_close: None,
        }
    }

    /// 实例唯一 id；不传则回退标签文本，同文本叠层需显式传 id。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 颜色预设（默认灰底）
    pub fn color(mut self, color: TagColor) -> Self {
        self.color = color;
        self
    }

    /// 外观变体（默认 filled）
    pub fn variant(mut self, variant: TagVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 是否显示边框（默认 true）；`false` 等价于 filled（antd `useColor`）
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// 禁用态：灰底灰字，关闭按钮不可用
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 显示关闭按钮
    pub fn on_close(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Rc::new(listener));
        self
    }
}

/// 派生关闭钮 id：实例 id 优先，未传则回退标签文本。
fn close_id(id: Option<&str>, label: &str) -> String {
    format!("tag-close-{}", id.unwrap_or(label))
}

impl IntoElement for Tag {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let close_id = close_id(self.id.as_deref(), &self.label);
        let variant = if self.bordered {
            self.variant
        } else {
            TagVariant::Filled
        };
        let style = theme::with(|t| state(self.color, variant, t, self.disabled));
        let semantics = a11y::Semantics::new(Role::Group, self.label.clone())
            .id(format!("tag-{}", self.id.as_deref().unwrap_or(&self.label)))
            .disabled(self.disabled);
        let mut tag = semantics.apply(
            div()
                .id(ElementId::Name(
                    self.id.clone().unwrap_or_else(|| self.label.clone()).into(),
                ))
                .flex()
                .flex_row()
                .items_center()
                .px(px(style.padding_inline))
                .rounded(px(style.radius))
                .border(px(style.border_width))
                .border_color(style.border)
                .line_height(px(style.line_height))
                .text_size(px(style.font_size))
                .text_color(style.fg)
                .bg(style.bg)
                .child(self.label.clone()),
        );

        if self.disabled {
            tag = tag.cursor(CursorStyle::Arrow);
        }

        if let Some(on_close) = self.on_close {
            let mut close = div()
                .id(ElementId::Name(close_id.clone().into()))
                .ml(px(style.icon_margin))
                .text_size(px(style.icon_size))
                .text_color(style.icon)
                .child("✕");
            let semantics = a11y::Semantics::new(
                Role::Button,
                SharedString::from(format!("关闭标签 {}", self.label)),
            )
            .id(format!("{close_id}-a11y"))
            .disabled(self.disabled)
            .focusable(!self.disabled);
            close = semantics.apply(close);
            if self.disabled {
                close = close.cursor(CursorStyle::Arrow);
            } else {
                let icon_hover = style.icon_hover;
                close = close
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |s| s.text_color(icon_hover));
                close = a11y::click_shared(close, on_close);
            }
            tag = tag.child(close);
        }

        tag
    }
}

#[cfg(test)]
mod tests {
    use super::close_id;

    #[test]
    fn tag_id_default_derives_from_label() {
        assert_eq!(close_id(None, "Rust"), "tag-close-Rust");
    }

    #[test]
    fn tag_id_stable_across_repeated_calls() {
        for _ in 0..3 {
            assert_eq!(close_id(None, "Rust"), close_id(None, "Rust"));
            assert_eq!(
                close_id(Some("rust-filter"), "Rust"),
                close_id(Some("rust-filter"), "Rust")
            );
        }
    }

    #[test]
    fn tag_id_distinct_labels_do_not_collide() {
        assert_ne!(close_id(None, "Rust"), close_id(None, "GPUI"));
    }

    #[test]
    fn tag_id_explicit_overrides_label() {
        assert_eq!(
            close_id(Some("rust-filter-7"), "Rust"),
            "tag-close-rust-filter-7"
        );
    }

    #[test]
    fn tag_id_same_label_still_collides() {
        assert_eq!(close_id(None, "Rust"), close_id(None, "Rust"));
        assert_ne!(close_id(Some("a"), "Rust"), close_id(Some("b"), "Rust"));
    }

    #[test]
    fn tag_id_empty_label_keeps_prefix() {
        assert_eq!(close_id(None, ""), "tag-close-");
    }
}
