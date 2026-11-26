//! 头像控件：圆形 / 方形背景 + 名字首字符。
//!
//! 对齐 antdv-next Avatar：默认底 `colorTextPlaceholder`、白色文字、字号 14。

use gpui::{Div, Hsla, IntoElement, Pixels, Role, SharedString, Stateful, div, prelude::*, px};

use crate::a11y::Semantics;
use crate::theme::{self, Tokens, metric};

/// antd Avatar 尺寸：small 24 / default 32 / large 40。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarSize {
    Small,
    Default,
    Large,
}

impl AvatarSize {
    fn index(self) -> usize {
        match self {
            AvatarSize::Small => 0,
            AvatarSize::Default => 1,
            AvatarSize::Large => 2,
        }
    }
}

/// 头像外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarStyle {
    pub bg: Hsla,
    pub text: Hsla,
    pub size: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
}

pub fn avatar_state(tokens: &Tokens, size: AvatarSize, square: bool) -> AvatarStyle {
    let m = metric::AvatarMetric::resolve(tokens);
    let ix = size.index();
    AvatarStyle {
        bg: tokens.color_text_placeholder,
        text: tokens.color_text_light_solid,
        size: m.size[ix],
        radius: if square {
            m.radius[ix]
        } else {
            m.size[ix] / 2.0
        },
        font_size: m.font_size,
        line_height: m.line_height,
    }
}

/// 通用头像。
///
/// ```ignore
/// Avatar::new("张三")                          // 默认 32px 圆形
/// Avatar::new("Alice").preset(AvatarSize::Large)
/// Avatar::new("Bob").size(px(48.))
/// ```
pub struct Avatar {
    id: Option<SharedString>,
    origin: SharedString,
    name: SharedString,
    preset: AvatarSize,
    size: Option<Pixels>,
    square: bool,
}

impl Avatar {
    #[track_caller]
    pub fn new(name: impl Into<SharedString>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "avatar-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            name: name.into(),
            preset: AvatarSize::Default,
            size: None,
            square: false,
        }
    }

    /// 实例 ID；同名头像并列显示时应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 自定义边长（antd 的 `style={{ width, height }}`）
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
        self
    }

    pub fn preset(mut self, preset: AvatarSize) -> Self {
        self.preset = preset;
        self
    }

    /// 方形头像（圆角矩形，默认圆形）
    pub fn square(mut self) -> Self {
        self.square = true;
        self
    }

    fn initial(&self) -> String {
        self.name
            .chars()
            .next()
            .map(|c| c.to_string())
            .unwrap_or_default()
    }
}

impl IntoElement for Avatar {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let style = theme::with(|t| avatar_state(t, self.preset, self.square));
        let size = self.size.unwrap_or(px(style.size));
        let radius = if self.square {
            px(style.radius)
        } else {
            size / 2.0
        };

        let semantics = Semantics::new(Role::Image, self.name.clone()).id(format!(
            "{}-{}",
            self.id.as_deref().unwrap_or(&self.origin),
            self.name
        ));
        let element_id = semantics.element_id();
        semantics.apply(
            div()
                .id(element_id)
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .size(size)
                .rounded(radius)
                .bg(style.bg)
                .text_color(style.text)
                .text_size(px(style.font_size))
                .line_height(px(style.line_height))
                .child(self.initial()),
        )
    }
}
