//! 搜索输入框：搜索图标 + 一键清空 + 回车搜索回调。
//! 对应 antd Input prefix=SearchOutlined + allowClear：前缀 14px 跟随正文色，
//! 清空图标 CloseCircleFilled 12px（colorTextQuaternary → 悬停 colorIcon → 按下 colorText）。

use std::rc::Rc;

use gpui::{
    App, Context, CursorStyle, Entity, FocusHandle, Focusable, Hsla, IntoElement, KeyDownEvent,
    ParentElement, Render, Role, SharedString, StatefulInteractiveElement, Styled, Window, canvas,
    div, prelude::*, px,
};

use crate::a11y::{self, Semantics};

use crate::{
    TextInput,
    svg_path::{paint_svg, svg_segments},
    text_input::{InputSize, InputStatus, InputVariant, apply_frame, hover_frame, state},
    theme,
    theme::metric::{clear_icon_color, input_affix_metric},
};

const ICON_SEARCH: &str = "M909.6 854.5L649.9 594.8C690.2 542.7 712 479 712 412c0-80.2-31.3-155.4-87.9-212.1-56.6-56.7-132-87.9-212.1-87.9s-155.5 31.3-212.1 87.9C143.2 256.5 112 331.8 112 412c0 80.1 31.3 155.5 87.9 212.1C256.5 680.8 331.8 712 412 712c67 0 130.6-21.8 182.7-62l259.7 259.6a8.2 8.2 0 0011.6 0l43.6-43.5a8.2 8.2 0 000-11.6zM570.4 570.4C528 612.7 471.8 636 412 636s-116-23.3-158.4-65.6C211.3 528 188 471.8 188 412s23.3-116.1 65.6-158.4C296 211.3 352.2 188 412 188s116.1 23.2 158.4 65.6S636 352.2 636 412s-23.3 116.1-65.6 158.4z";
const ICON_CLEAR: &str = "M512 64c247.4 0 448 200.6 448 448S759.4 960 512 960 64 759.4 64 512 264.6 64 512 64zm127.98 274.82h-.04l-.08.06L512 466.75 384.14 338.88c-.04-.05-.06-.06-.08-.06a.12.12 0 00-.07 0c-.03 0-.05.01-.09.05l-45.02 45.02a.2.2 0 00-.05.09.12.12 0 000 .07v.02a.27.27 0 00.06.06L466.75 512 338.88 639.86c-.05.04-.06.06-.06.08a.12.12 0 000 .07c0 .03.01.05.05.09l45.02 45.02a.2.2 0 00.09.05.12.12 0 00.07 0c.02 0 .04-.01.08-.05L512 557.25l127.86 127.87c.04.04.06.05.08.05a.12.12 0 00.07 0c.03 0 .05-.01.09-.05l45.02-45.02a.2.2 0 00.05-.09.12.12 0 000-.07v-.02a.27.27 0 00-.05-.06L557.25 512l127.87-127.86c.04-.04.05-.06.05-.08a.12.12 0 000-.07c0-.03-.01-.05-.05-.09l-45.02-45.02a.2.2 0 00-.09-.05.12.12 0 00-.07 0z";
const ICON_VIEW: (f32, f32, f32, f32) = (64.0, 64.0, 896.0, 896.0);

fn affix_icon(d: &'static str, size: f32, color: Hsla) -> impl IntoElement {
    let segs = svg_segments(d);
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, ICON_VIEW, &segs, color, window),
    )
    .w(px(size))
    .h(px(size))
    .flex_shrink_0()
}

/// 搜索输入框（Entity，用法同 TextInput）。
///
/// ```ignore
/// let search = cx.new(|cx| SearchInput::new(cx)
///     .placeholder("搜索控件…")
///     .on_search(|text, _, cx| { /* 回车或点图标时触发 */ }));
/// ```
pub struct SearchInput {
    input: Entity<TextInput>,
    placeholder: SharedString,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    disabled: bool,
    on_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    on_search: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
}

impl SearchInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let entity = cx.entity();
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .bare(true)
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
            placeholder: "搜索…".into(),
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            disabled: false,
            on_change: None,
            on_search: None,
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

    /// 内容变化回调
    pub fn on_change(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    /// 回车搜索回调
    pub fn on_search(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_search = Some(Rc::new(listener));
        self
    }

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

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let input = self.input.clone();
        input.update(cx, |input, cx| input.set_text_silent("", cx));
        // 静默写入不触发内部转发，手动通知外层
        if let Some(on_change) = self.on_change.clone() {
            on_change("", window, cx);
        }
        self.input.focus_handle(cx).focus(window, cx);
        cx.notify();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key == "enter" {
            if let Some(on_search) = self.on_search.clone() {
                let text = self.input.read(cx).text().to_string();
                on_search(&text, window, cx);
            }
            cx.stop_propagation();
        }
    }
}

impl Render for SearchInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.input.focus_handle(cx).is_focused(window);
        let placeholder = self.placeholder.clone();
        let (variant, status, size, disabled) =
            (self.variant, self.status, self.size, self.disabled);
        self.input.update(cx, |input, cx| {
            input.set_placeholder(placeholder.clone());
            input.set_form_state(variant, status, size, disabled, cx);
        });
        let has_text = !self.input.read(cx).text().is_empty();
        let style = theme::with(|t| state(variant, status, size, t, false, focused, disabled));
        let hover_style = theme::with(|t| state(variant, status, size, t, true, false, disabled));
        let (m, clear_idle, clear_hover, clear_active) = theme::with(|t| {
            (
                input_affix_metric(t),
                clear_icon_color(t, false, false),
                clear_icon_color(t, true, false),
                clear_icon_color(t, true, true),
            )
        });

        let mut row = div()
            .flex()
            .items_center()
            .gap(px(m.gap))
            .w_full()
            .px(px(style.padding_inline))
            .py(px(style.padding_block))
            .line_height(px(style.line_height))
            .text_size(px(style.font_size))
            .text_color(style.fg)
            .on_key_down(cx.listener(Self::on_key_down))
            .child(affix_icon(ICON_SEARCH, m.icon_font, style.affix))
            .child(div().flex_1().min_w_0().child(self.input.clone()));

        if has_text && !disabled {
            let group = SharedString::from(format!("search-clear-{:?}", cx.entity_id()));
            let clear = div()
                .id(group.clone())
                .group(group.clone())
                .relative()
                .w(px(m.clear_font))
                .h(px(m.clear_font))
                .flex_shrink_0()
                .cursor(CursorStyle::PointingHand)
                .child(
                    div()
                        .group_hover(group.clone(), |s| s.opacity(0.))
                        .child(affix_icon(ICON_CLEAR, m.clear_font, clear_idle)),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .opacity(0.)
                        .group_hover(group.clone(), |s| s.opacity(1.))
                        .child(affix_icon(ICON_CLEAR, m.clear_font, clear_hover)),
                )
                .child(
                    div()
                        .id(SharedString::from(format!(
                            "search-clear-active-{:?}",
                            cx.entity_id()
                        )))
                        .absolute()
                        .inset_0()
                        .opacity(0.)
                        .active(|s| s.opacity(1.))
                        .child(affix_icon(ICON_CLEAR, m.clear_font, clear_active)),
                );
            let clear = Semantics::new(Role::Button, "清除搜索内容")
                .id(format!("search-clear-{:?}", cx.entity_id()))
                .focusable(true)
                .apply(clear);
            let entity = cx.entity();
            row = row.child(a11y::click_void(clear, move |window, cx| {
                let _ = entity.update(cx, |input, cx| input.clear(window, cx));
            }));
        }

        let semantics = Semantics::new(Role::SearchInput, "搜索")
            .id(format!("search-input-{:?}", cx.entity_id()))
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

impl Focusable for SearchInput {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}
