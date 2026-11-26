//! 密码输入框：掩码显示 + 显隐切换（antd Input.Password 令牌：眼睛图标 14px，
//! colorIcon / 悬停 colorIconHover，间距 inputAffixPadding）。

use std::rc::Rc;

use gpui::{
    App, Context, CursorStyle, Entity, FocusHandle, Focusable, Hsla, IntoElement, ParentElement,
    Render, Role, SharedString, Styled, Toggled, Window, canvas, div, prelude::*, px,
};

use crate::a11y::{self, Semantics};

use crate::{
    TextInput,
    svg_path::{paint_svg, svg_segments},
    text_input::{InputSize, InputStatus, InputVariant, apply_frame, hover_frame, state},
    theme,
    theme::metric::{input_affix_metric, password_icon_color},
};

const ICON_EYE: &str = "M942.2 486.2C847.4 286.5 704.1 186 512 186c-192.2 0-335.4 100.5-430.2 300.3a60.3 60.3 0 000 51.5C176.6 737.5 319.9 838 512 838c192.2 0 335.4-100.5 430.2-300.3 7.7-16.2 7.7-35 0-51.5zM512 766c-161.3 0-279.4-81.8-362.7-254C232.6 339.8 350.7 258 512 258c161.3 0 279.4 81.8 362.7 254C791.5 684.2 673.4 766 512 766zm-4-430c-97.2 0-176 78.8-176 176s78.8 176 176 176 176-78.8 176-176-78.8-176-176-176zm0 288c-61.9 0-112-50.1-112-112s50.1-112 112-112 112 50.1 112 112-50.1 112-112 112z";
const ICON_EYE_INVISIBLE: &str = "M942.2 486.2Q889.47 375.11 816.7 305l-50.88 50.88C807.31 395.53 843.45 447.4 874.7 512 791.5 684.2 673.4 766 512 766q-72.67 0-133.87-22.38L323 798.75Q408 838 512 838q288.3 0 430.2-300.3a60.29 60.29 0 000-51.5zm-63.57-320.64L836 122.88a8 8 0 00-11.32 0L715.31 232.2Q624.86 186 512 186q-288.3 0-430.2 300.3a60.3 60.3 0 000 51.5q56.69 119.4 136.5 191.41L112.48 835a8 8 0 000 11.31L155.17 889a8 8 0 0011.31 0l712.15-712.12a8 8 0 000-11.32zM149.3 512C232.6 339.8 350.7 258 512 258c54.54 0 104.13 9.36 149.12 28.39l-70.3 70.3a176 176 0 00-238.13 238.13l-83.42 83.42C223.1 637.49 183.3 582.28 149.3 512zm246.7 0a112.11 112.11 0 01146.2-106.69L401.31 546.2A112 112 0 01396 512z M508 624c-3.46 0-6.87-.16-10.25-.47l-52.82 52.82a176.09 176.09 0 00227.42-227.42l-52.82 52.82c.31 3.38.47 6.79.47 10.25a111.94 111.94 0 01-112 112z";
const ICON_VIEW: (f32, f32, f32, f32) = (64.0, 64.0, 896.0, 896.0);

fn eye_icon(d: &'static str, size: f32, color: Hsla) -> impl IntoElement {
    let segs = svg_segments(d);
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, ICON_VIEW, &segs, color, window),
    )
    .w(px(size))
    .h(px(size))
    .flex_shrink_0()
}

/// 密码输入框（Entity，用法同 TextInput）。
///
/// ```ignore
/// let password = cx.new(|cx| PasswordInput::new(cx).placeholder("请输入密码"));
/// // render 中：div().child(password.clone())
/// ```
pub struct PasswordInput {
    input: Entity<TextInput>,
    visible: bool,
    last_visible: bool,
    placeholder: SharedString,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    disabled: bool,
    on_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
}

impl PasswordInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let entity = cx.entity();
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .bare(true)
                .masked(true)
                .on_change(move |text, window, cx| {
                    let _ = entity.update(cx, |this, cx| {
                        if let Some(on_change) = this.on_change.clone() {
                            on_change(text, window, cx);
                        }
                    });
                })
        });
        Self {
            input,
            visible: false,
            last_visible: false,
            placeholder: "".into(),
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            disabled: false,
            on_change: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
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

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 初始明文显示（默认掩码）
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// 内容变化回调（参数为明文）
    pub fn on_change(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    /// 当前明文
    pub fn text(&self, cx: &App) -> String {
        self.input.read(cx).text().to_string()
    }

    pub fn set_text(
        &mut self,
        text: impl Into<SharedString>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input = self.input.clone();
        input.update(cx, |input, cx| input.set_text_silent(text, cx));
    }

    /// 静默回填（无 Window 版 [`Self::set_text`]）：只改内容，不触发 on_change。
    pub fn set_text_silent(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        let input = self.input.clone();
        input.update(cx, |input, cx| input.set_text_silent(text, cx));
    }

    /// 运行时设置占位文本（render 每帧同步给内层 TextInput）。
    pub fn set_placeholder(
        &mut self,
        placeholder: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.placeholder = placeholder.into();
        cx.notify();
    }

    fn toggle_visible(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.visible = !self.visible;
        self.input.focus_handle(cx).focus(window, cx);
        cx.notify();
    }
}

impl Render for PasswordInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.input.focus_handle(cx).is_focused(window);
        let placeholder = self.placeholder.clone();
        let (variant, status, size, disabled) =
            (self.variant, self.status, self.size, self.disabled);
        let visible = self.visible;
        let needs_mask_sync = visible != self.last_visible;
        self.input.update(cx, |input, cx| {
            input.set_placeholder(placeholder.clone());
            input.set_form_state(variant, status, size, disabled, cx);
            if needs_mask_sync {
                input.set_masked(!visible, cx);
            }
        });
        self.last_visible = visible;
        let style = theme::with(|t| state(variant, status, size, t, false, focused, disabled));
        let hover_style = theme::with(|t| state(variant, status, size, t, true, false, disabled));
        let (m, idle, hover) = theme::with(|t| {
            (
                input_affix_metric(t),
                password_icon_color(t, false, disabled),
                password_icon_color(t, true, disabled),
            )
        });
        let group = SharedString::from(format!("password-toggle-{:?}", cx.entity_id()));
        let d = if visible {
            ICON_EYE
        } else {
            ICON_EYE_INVISIBLE
        };

        let mut toggle = Semantics::new(
            Role::Button,
            if visible {
                "隐藏密码"
            } else {
                "显示密码"
            },
        )
        .id(format!("password-toggle-visibility-{:?}", cx.entity_id()))
        .toggled(if visible {
            Toggled::True
        } else {
            Toggled::False
        })
        .disabled(disabled)
        .focusable(!disabled)
        .apply(
            div()
                .id(group.clone())
                .group(group.clone())
                .relative()
                .w(px(m.icon_font))
                .h(px(m.icon_font))
                .flex_shrink_0()
                .child(
                    div()
                        .group_hover(group.clone(), |s| s.opacity(0.))
                        .child(eye_icon(d, m.icon_font, idle)),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .opacity(0.)
                        .group_hover(group, |s| s.opacity(1.))
                        .child(eye_icon(d, m.icon_font, hover)),
                ),
        );
        if !disabled {
            let entity = cx.entity();
            toggle = toggle.cursor(CursorStyle::PointingHand);
            toggle = a11y::click_void(toggle, move |window, cx| {
                let _ = entity.update(cx, |input, cx| input.toggle_visible(window, cx));
            });
        }

        let row = div()
            .flex()
            .items_center()
            .gap(px(m.gap))
            .w_full()
            .px(px(style.padding_inline))
            .py(px(style.padding_block))
            .line_height(px(style.line_height))
            .text_size(px(style.font_size))
            .text_color(style.fg)
            .child(div().flex_1().min_w_0().child(self.input.clone()))
            .child(toggle);
        let semantics = Semantics::new(Role::PasswordInput, "密码")
            .id(format!("password-input-{:?}", cx.entity_id()))
            .value(self.input.read(cx).a11y_value())
            .placeholder(placeholder)
            .disabled(disabled)
            .invalid(status == InputStatus::Error)
            .focusable(!disabled);
        let mut row = row.id(semantics.element_id());
        if !disabled {
            row = row.track_focus(&self.input.focus_handle(cx));
        }
        let row = self.input.update(cx, |input, cx| {
            input.with_a11y_actions(semantics.apply(row), cx)
        });
        hover_frame(apply_frame(row, &style), &hover_style, focused)
    }
}

impl Focusable for PasswordInput {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}
