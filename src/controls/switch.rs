//! 开关控件：antdv-next Switch（22px 轨道，18px 滑块）。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, Hsla, IntoElement, Rgba, Role, SharedString, Stateful,
    Toggled, Window, div, prelude::*, px,
};

use crate::Spinner;
use crate::a11y;
use crate::theme::{self, Tokens, metric};

/// 轨道 / 滑块外观（antdv-next Switch 实测）：
/// 未选中轨道 `colorTextQuaternary`、悬停 `colorTextTertiary`；选中轨道主色、
/// 悬停 `colorPrimaryHover`；滑块 `colorBgContainer`；禁用 / 加载整体 opacity 0.65。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchStyle {
    pub bg: Hsla,
    pub hover_bg: Hsla,
    pub handle_bg: Hsla,
    pub off_text: Hsla,
    pub on_text: Hsla,
    pub accent: Hsla,
    pub opacity: f32,
    pub track_width: f32,
    pub track_height: f32,
    pub handle_size: f32,
    pub padding: f32,
    pub radius: f32,
    pub text_padding_near: f32,
    pub font_size: f32,
    pub line_height: f32,
}

pub fn switch_state(
    tokens: &Tokens,
    small: bool,
    on: bool,
    disabled: bool,
    loading: bool,
) -> SwitchStyle {
    let m = metric::SwitchMetric::resolve(tokens);
    let ix = usize::from(small);
    let inactive = disabled || loading;
    let bg = if on {
        tokens.color_primary
    } else {
        tokens.color_text_quaternary
    };
    let hover_bg = if inactive {
        bg
    } else if on {
        tokens.color_primary_hover
    } else {
        tokens.color_text_tertiary
    };
    // gpui 的 opacity 逐元素生效，不会像 CSS 那样先拍平整棵子树；禁用/加载的
    // 观感改为把颜色按 0.65 预混到面板底色上，滑块自身保持纯色。
    let fade = |color: Hsla| {
        if inactive {
            blend_over(color, tokens.color_bg_container, m.disabled_opacity)
        } else {
            color
        }
    };
    SwitchStyle {
        bg: fade(bg),
        hover_bg: fade(hover_bg),
        handle_bg: tokens.color_bg_container,
        off_text: fade(tokens.color_text_tertiary),
        on_text: fade(tokens.color_text_light_solid),
        accent: fade(tokens.color_primary),
        opacity: if inactive { m.disabled_opacity } else { 1.0 },
        track_width: m.track_min_width[ix],
        track_height: m.track_height[ix],
        handle_size: m.handle_size[ix],
        padding: m.track_padding,
        radius: m.radius,
        text_padding_near: m.text_padding_near[ix],
        font_size: m.font_size,
        line_height: m.track_height[ix],
    }
}

fn blend_over(color: Hsla, backdrop: Hsla, alpha: f32) -> Hsla {
    let fg = Rgba::from(color);
    let bg = Rgba::from(backdrop);
    let over = |f: f32, b: f32| f * fg.a + b * (1.0 - fg.a);
    let mix = |f: f32, b: f32| f * alpha + b * (1.0 - alpha);
    Rgba {
        r: mix(over(fg.r, bg.r), bg.r),
        g: mix(over(fg.g, bg.g), bg.g),
        b: mix(over(fg.b, bg.b), bg.b),
        a: 1.0,
    }
    .into()
}

/// 通用开关。
///
/// ```ignore
/// Switch::new(self.notifications)
///     .on_change(cx.listener(|this, on, _window, cx| {
///         this.notifications = *on;
///         cx.notify();
///     }))
/// ```
pub struct Switch {
    label: Option<SharedString>,
    id: Option<SharedString>,
    origin: SharedString,
    on: bool,
    disabled: bool,
    loading: bool,
    small: bool,
    text: Option<(SharedString, SharedString)>,
    on_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
}

impl Switch {
    #[track_caller]
    pub fn new(on: bool) -> Self {
        let location = core::panic::Location::caller();
        Self {
            label: None,
            id: None,
            origin: format!(
                "switch-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            on,
            disabled: false,
            loading: false,
            small: false,
            text: None,
            on_change: None,
        }
    }

    /// 动态列表中反复在同一构造点创建时应显式传入唯一 id。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 加载态：滑块内显示旋转指示器并阻止切换
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// 小尺寸轨道（28x16）
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    /// 轨道内文字：开启时显示 on_text，关闭时显示 off_text（如 "开" / "关"）
    pub fn text(
        mut self,
        on_text: impl Into<SharedString>,
        off_text: impl Into<SharedString>,
    ) -> Self {
        self.text = Some((on_text.into(), off_text.into()));
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Switch {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let on = self.on;
        let style = theme::with(|t| switch_state(t, self.small, on, self.disabled, self.loading));

        let mut knob = div()
            .flex_shrink_0()
            .size(px(style.handle_size))
            .rounded(px(style.handle_size / 2.0))
            .bg(style.handle_bg)
            .shadow(metric::switch_handle_shadow(style.opacity, self.small));
        if self.loading {
            knob = knob.flex().items_center().justify_center().child(
                Spinner::new()
                    .size(px(style.handle_size - 4.0))
                    .color(style.accent),
            );
        }

        let mut track = div()
            .flex()
            .flex_row()
            .items_center()
            .h(px(style.track_height))
            .rounded(px(style.radius))
            .text_size(px(style.font_size))
            .line_height(px(style.line_height))
            .bg(style.bg);

        if let Some((on_text, off_text)) = self.text {
            track = track
                .min_w(px(style.track_width))
                .justify_between()
                .pl(px(if on {
                    style.text_padding_near
                } else {
                    style.padding
                }))
                .pr(px(if on {
                    style.padding
                } else {
                    style.text_padding_near
                }));
            let label = div().text_color(if on { style.on_text } else { style.off_text });
            track = if on {
                track.child(label.child(on_text)).child(knob)
            } else {
                track.child(knob).child(label.child(off_text))
            };
        } else {
            track = track
                .w(px(style.track_width))
                .p(px(style.padding))
                .when(on, |track| track.justify_end())
                .child(knob);
        }

        if !self.disabled && !self.loading {
            track = track.hover(move |s| s.bg(style.hover_bg));
        }

        let id = ElementId::Name(self.id.clone().unwrap_or_else(|| {
            format!(
                "{}-{}",
                self.origin,
                self.label.as_deref().unwrap_or("switch")
            )
            .into()
        }));
        let aria_label = self
            .label
            .clone()
            .unwrap_or_else(|| SharedString::from("Switch"));
        let mut element = div()
            .id(id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(track);

        if let Some(label) = self.label {
            element = element.child(div().text_sm().text_color(theme::text()).child(label));
        }

        let semantics = a11y::Semantics::new(Role::Switch, aria_label)
            .id(format!("{:?}", id))
            .toggled(if on { Toggled::True } else { Toggled::False })
            .disabled(self.disabled || self.loading)
            .focusable(!self.disabled && !self.loading && self.on_change.is_some());
        element = semantics.apply(element);

        if self.disabled || self.loading {
            return element.cursor(CursorStyle::Arrow);
        }

        element = element.cursor(CursorStyle::PointingHand);

        if let Some(on_change) = self.on_change {
            element = a11y::click(element, move |_, window, cx| on_change(&!on, window, cx));
        }

        element
    }
}
