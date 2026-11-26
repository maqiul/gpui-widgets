//! 排版控件：标题 / 段落 / 行内文本（对应 antd Typography）。

use gpui::{
    App, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Role,
    SharedString, Styled, Window, div, px, rgba,
};

use crate::{
    a11y::Semantics,
    theme::{
        self, Tokens,
        metric::{
            TYPOGRAPHY_CODE_BG, TYPOGRAPHY_CODE_BORDER, TYPOGRAPHY_CODE_RADIUS,
            TYPOGRAPHY_CODE_SIZE_RATIO, TYPOGRAPHY_MARK_BG, TYPOGRAPHY_STRONG_WEIGHT,
            TYPOGRAPHY_TITLE_WEIGHT, TypographyMetric,
        },
    },
};

/// 文本语义类型（对应 antd `type`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypographyType {
    Default,
    Secondary,
    Success,
    Warning,
    Danger,
    Disabled,
}

/// 语义色：默认 `colorText`、次要 `colorTextDescription`、成功 `colorSuccessText`、
/// 警告 `colorWarningText`、错误 `colorErrorText`、禁用 `colorTextDisabled`。
pub fn typography_color(tokens: &Tokens, kind: TypographyType) -> Hsla {
    match kind {
        TypographyType::Default => tokens.color_text,
        TypographyType::Secondary => tokens.color_text_description,
        TypographyType::Success => tokens.color_success_text,
        TypographyType::Warning => tokens.color_warning_text,
        TypographyType::Danger => tokens.color_error_text,
        TypographyType::Disabled => tokens.color_text_disabled,
    }
}

/// 标题（level 1-5，超出按 5 处理）
///
/// ```ignore
/// Title::new(1, "一级标题")
/// ```
#[derive(IntoElement)]
pub struct Title {
    level: u8,
    text: SharedString,
    kind: TypographyType,
}

impl Title {
    pub fn new(level: u8, text: impl Into<SharedString>) -> Self {
        Self {
            level: level.clamp(1, 5),
            text: text.into(),
            kind: TypographyType::Default,
        }
    }

    /// 次要颜色
    pub fn secondary(mut self) -> Self {
        self.kind = TypographyType::Secondary;
        self
    }

    pub fn success(mut self) -> Self {
        self.kind = TypographyType::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.kind = TypographyType::Warning;
        self
    }

    pub fn danger(mut self) -> Self {
        self.kind = TypographyType::Danger;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.kind = TypographyType::Disabled;
        self
    }
}

impl RenderOnce for Title {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (color, m) = theme::with(|tokens| {
            let color = if self.kind == TypographyType::Default {
                tokens.color_text_heading
            } else {
                typography_color(tokens, self.kind)
            };
            (color, TypographyMetric::resolve(tokens))
        });
        let ix = (self.level - 1) as usize;
        let semantics = Semantics::new(Role::Heading, self.text.clone())
            .id(format!("heading-{}-{}", self.level, self.text))
            .level(self.level as usize);
        let element_id = semantics.element_id();
        semantics.apply(
            div()
                .id(element_id)
                .font_weight(FontWeight(TYPOGRAPHY_TITLE_WEIGHT))
                .text_size(px(m.title_size[ix]))
                .line_height(px(m.title_line_height[ix]))
                .mb(px(m.title_margin_bottom))
                .text_color(color)
                .child(self.text.clone()),
        )
    }
}

/// 段落文本
#[derive(IntoElement)]
pub struct Paragraph {
    text: SharedString,
    kind: TypographyType,
}

impl Paragraph {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            kind: TypographyType::Default,
        }
    }

    pub fn secondary(mut self) -> Self {
        self.kind = TypographyType::Secondary;
        self
    }

    pub fn success(mut self) -> Self {
        self.kind = TypographyType::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.kind = TypographyType::Warning;
        self
    }

    pub fn danger(mut self) -> Self {
        self.kind = TypographyType::Danger;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.kind = TypographyType::Disabled;
        self
    }
}

impl RenderOnce for Paragraph {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (color, m) = theme::with(|tokens| {
            (
                typography_color(tokens, self.kind),
                TypographyMetric::resolve(tokens),
            )
        });
        let semantics = Semantics::new(Role::Paragraph, self.text.clone())
            .id(format!("paragraph-{}", self.text));
        let element_id = semantics.element_id();
        semantics.apply(
            div()
                .id(element_id)
                .text_size(px(m.body_size))
                .line_height(px(m.body_line_height))
                .text_color(color)
                .child(self.text.clone()),
        )
    }
}

/// 行内文本（可组合多种修饰）
///
/// ```ignore
/// Text::new("重要").strong().mark()
/// ```
#[derive(IntoElement)]
pub struct Text {
    text: SharedString,
    kind: TypographyType,
    strong: bool,
    italic: bool,
    underline: bool,
    delete: bool,
    code: bool,
    mark: bool,
}

impl Text {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            kind: TypographyType::Default,
            strong: false,
            italic: false,
            underline: false,
            delete: false,
            code: false,
            mark: false,
        }
    }

    pub fn secondary(mut self) -> Self {
        self.kind = TypographyType::Secondary;
        self
    }

    pub fn success(mut self) -> Self {
        self.kind = TypographyType::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.kind = TypographyType::Warning;
        self
    }

    pub fn danger(mut self) -> Self {
        self.kind = TypographyType::Danger;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.kind = TypographyType::Disabled;
        self
    }

    pub fn strong(mut self) -> Self {
        self.strong = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// 删除线
    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }

    /// 行内代码样式
    pub fn code(mut self) -> Self {
        self.code = true;
        self
    }

    /// 高亮标记（黄色底）
    pub fn mark(mut self) -> Self {
        self.mark = true;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (color, m) = theme::with(|tokens| {
            (
                typography_color(tokens, self.kind),
                TypographyMetric::resolve(tokens),
            )
        });
        let role = if self.delete {
            Role::ContentDeletion
        } else if self.code {
            Role::Code
        } else if self.mark {
            Role::Mark
        } else if self.strong {
            Role::Strong
        } else {
            Role::Label
        };
        let semantics = Semantics::new(role, self.text.clone())
            .id(format!("text-{}-{:?}", self.text, self.kind));
        let element_id = semantics.element_id();
        let mut text = semantics.apply(
            div()
                .id(element_id)
                .text_size(px(m.body_size))
                .line_height(px(m.body_line_height))
                .text_color(color)
                .child(self.text.clone()),
        );
        if self.strong {
            text = text.font_weight(FontWeight(TYPOGRAPHY_STRONG_WEIGHT));
        }
        if self.italic {
            text = text.italic();
        }
        if self.underline {
            text = text.underline();
        }
        if self.delete {
            text = text.line_through();
        }
        if self.code {
            let code_size = m.body_size * TYPOGRAPHY_CODE_SIZE_RATIO;
            text = text
                .font_family("Consolas")
                .text_size(px(code_size))
                .px(px(code_size * 0.4))
                .pt(px(code_size * 0.2))
                .pb(px(code_size * 0.1))
                .rounded(px(TYPOGRAPHY_CODE_RADIUS))
                .bg(rgba(TYPOGRAPHY_CODE_BG))
                .border_1()
                .border_color(rgba(TYPOGRAPHY_CODE_BORDER));
        }
        if self.mark {
            text = text.bg(rgba(TYPOGRAPHY_MARK_BG));
        }
        text
    }
}
