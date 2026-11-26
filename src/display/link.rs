//! 链接：主色文本，悬停/按下换色并加下划线；支持点击回调或打开 URL。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Hsla, IntoElement, ParentElement, RenderOnce, Role, SharedString, Styled,
    Window, div, prelude::*, px,
};

use crate::{a11y, theme, theme::Tokens};

/// 链接交互状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkState {
    Normal,
    Hover,
    Active,
    Disabled,
}

/// 链接的纯数据外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinkStyle {
    pub text: Hsla,
    pub underline: bool,
}

/// 链接状态：`colorLink` → 悬停 `colorLinkHover` + 下划线 → 按下 `colorLinkActive` + 下划线；
/// 禁用 `colorTextDisabled` 且悬停不变色。
pub fn link_state(tokens: &Tokens, state: LinkState) -> LinkStyle {
    match state {
        LinkState::Normal => LinkStyle {
            text: tokens.color_link,
            underline: false,
        },
        LinkState::Hover => LinkStyle {
            text: tokens.color_link_hover,
            underline: true,
        },
        LinkState::Active => LinkStyle {
            text: tokens.color_link_active,
            underline: true,
        },
        LinkState::Disabled => LinkStyle {
            text: tokens.color_text_disabled,
            underline: false,
        },
    }
}

/// 链接文本。
///
/// ```ignore
/// Link::new("查看文档").href("https://zed.dev")
/// Link::new("详情").on_click(|_, cx| { /* ... */ })
/// ```
#[derive(IntoElement)]
pub struct Link {
    label: SharedString,
    id: Option<SharedString>,
    origin: SharedString,
    href: Option<SharedString>,
    disabled: bool,
    on_click: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl Link {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            label: label.into(),
            id: None,
            origin: format!(
                "link-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            href: None,
            disabled: false,
            on_click: None,
        }
    }

    /// 动态列表中反复在同一构造点创建时应显式传入唯一 id。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 点击时用系统浏览器打开的 URL
    pub fn href(mut self, href: impl Into<SharedString>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// 自定义点击回调（与 href 可同时设置，回调先执行）
    pub fn on_click(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for Link {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (normal, hover, active, disabled, font_size) = theme::with(|tokens| {
            (
                link_state(tokens, LinkState::Normal),
                link_state(tokens, LinkState::Hover),
                link_state(tokens, LinkState::Active),
                link_state(tokens, LinkState::Disabled),
                tokens.font_size,
            )
        });
        let initial = if self.disabled { disabled } else { normal };
        let owner = self
            .id
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{}-{}", self.origin, self.label));
        let semantics = a11y::Semantics::new(Role::Link, self.label.clone())
            .id(format!("link-{owner}"))
            .disabled(self.disabled)
            .focusable(!self.disabled && (self.on_click.is_some() || self.href.is_some()));
        let element_id = semantics.element_id();
        let mut link = semantics.apply(
            div()
                .id(element_id)
                .text_size(px(font_size))
                .text_color(initial.text),
        );

        if !self.disabled {
            let on_click = self.on_click.clone();
            let href = self.href.clone();
            link = link
                .cursor(CursorStyle::PointingHand)
                .hover(move |style| {
                    let style = style.text_color(hover.text);
                    if hover.underline {
                        style.underline()
                    } else {
                        style
                    }
                })
                .active(move |style| {
                    let style = style.text_color(active.text);
                    // Style.text 非 refineable：active 参与布局，须重申字号防按下变大
                    let style = style.text_size(px(font_size));
                    if active.underline {
                        style.underline()
                    } else {
                        style
                    }
                });
            link = a11y::click_void(link, move |window, cx| {
                if let Some(on_click) = &on_click {
                    on_click(window, cx);
                }
                if let Some(href) = &href {
                    cx.open_url(href);
                }
            });
        }

        link.child(self.label.clone())
    }
}
