//! 按钮控件：对齐 antdv-next 的 `color` × `variant` × 交互态模型。
//!
//! 外观完全由 [`state`] 这个纯函数从设计令牌推导，渲染层不含色值字面量或魔法像素：
//!
//! ```ignore
//! Button::new("保存")
//!     .color(ButtonColor::Primary)
//!     .variant(ButtonVariant::Solid)
//!     .on_click(cx.listener(|this, _, _window, cx| { /* ... */ }))
//! ```
//!
//! 旧的 `.primary()` / `.secondary()` / `.danger()` / `.ghost()` 保留为等价的快捷方式。

use std::rc::Rc;

use gpui::{
    AnyElement, App, BoxShadow, ClickEvent, CursorStyle, Div, ElementId, Hsla, IntoElement, Role,
    SharedString, Stateful, Window, div, prelude::*, px, transparent_black,
};

use crate::theme::Tokens;
use crate::theme::metric::{self, ButtonMetric};
use crate::{Spinner, a11y, motion, theme};

/// 样式变体，对应 antd 的 `variant`
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonVariant {
    /// 实心：品牌色底 + 反色文字
    Solid,
    /// 描边（默认）
    #[default]
    Outlined,
    /// 虚线描边
    Dashed,
    /// 浅色填充
    Filled,
    /// 无底文字按钮
    Text,
    /// 链接
    Link,
}

/// 色彩族，对应 antd 的 `color`
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonColor {
    /// 中性：文字 `colorText`、底 `colorBgContainer`、边 `colorBorder`
    #[default]
    Default,
    /// 品牌主色
    Primary,
    /// 危险 / 错误色
    Danger,
}

/// 按钮尺寸
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// 按钮形状
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonShape {
    /// 默认圆角矩形
    #[default]
    Default,
    /// 胶囊形（全圆角）
    Round,
    /// 正圆（宽高相等，适合单个图标字符）
    Circle,
}

/// `icon_element` 图标元素的悬停组名：svg 不继承父级文字色，宿主可用
/// `group_hover(ICON_GROUP, …)` 让图标跟随按钮悬停换色。
pub const ICON_GROUP: &str = "gpui-button";

/// 某个（色彩 × 变体 × 态）组合解析出的外观，取值全部来自令牌。
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonStyle {
    pub fg: Hsla,
    pub bg: Hsla,
    pub border: Hsla,
    /// antd 按钮静止态的 `0 ${controlOutlineWidth}px 0` 那条投影
    pub shadow: Option<BoxShadow>,
    pub dashed: bool,
}

impl ButtonStyle {
    /// gpui 的 `shadow` 收 Vec，无投影时传空表
    fn shadows(&self) -> Vec<BoxShadow> {
        self.shadow.clone().into_iter().collect()
    }
}

impl ButtonSize {
    /// small / medium / large 在度量数组里的下标
    fn index(self) -> usize {
        match self {
            ButtonSize::Small => 0,
            ButtonSize::Medium => 1,
            ButtonSize::Large => 2,
        }
    }
}

/// 交互态在「静止 / hover / active」三组派生色里的下标。
/// antd 的 CSS 变量（`--ant-btn-color-hover` / `-active`）就是按 active > hover > 静止级联的。
fn state_index(hover: bool, pressed: bool) -> usize {
    match (pressed, hover) {
        (true, _) => 2,
        (false, true) => 1,
        (false, false) => 0,
    }
}

/// `--ant-btn-color-base`：主色族本身，用作描边与实心底。
/// `default` 的描边走 `colorBorder`（见 antd `genDefaultButtonStyle`）。
fn color_base(color: ButtonColor, t: &Tokens) -> [Hsla; 3] {
    match color {
        ButtonColor::Default => [
            t.color_border,
            t.color_primary_hover,
            t.color_primary_active,
        ],
        ButtonColor::Primary => [
            t.color_primary,
            t.color_primary_hover,
            t.color_primary_active,
        ],
        ButtonColor::Danger => [t.color_error, t.color_error_hover, t.color_error_active],
    }
}

/// `--ant-btn-text-color`：文字按钮 / 链接的基础色。
/// `default` 的文字静止态是 `colorText`（antd `textTextColor = colorText`）。
fn text_base(color: ButtonColor, t: &Tokens) -> [Hsla; 3] {
    match color {
        ButtonColor::Default => [t.color_text, t.color_primary_hover, t.color_primary_active],
        ButtonColor::Primary => [
            t.color_primary,
            t.color_primary_hover,
            t.color_primary_active,
        ],
        ButtonColor::Danger => [t.color_error, t.color_error_hover, t.color_error_active],
    }
}

/// `--ant-btn-color-light*`：浅底填充，hover / active 逐级加深。
fn light_trio(color: ButtonColor, t: &Tokens) -> [Hsla; 3] {
    match color {
        ButtonColor::Default => [t.color_fill_tertiary, t.color_fill_secondary, t.color_fill],
        ButtonColor::Primary => [
            t.color_primary_bg,
            t.color_primary_bg_hover,
            t.color_primary_border,
        ],
        ButtonColor::Danger => [
            t.color_error_bg,
            t.color_error_bg_filled_hover,
            t.color_error_bg_active,
        ],
    }
}

/// `--ant-btn-solid-bg-color*`：实心按钮底色；非 default 直接复用主色族。
fn solid_bg_trio(color: ButtonColor, t: &Tokens) -> [Hsla; 3] {
    match color {
        ButtonColor::Default => [
            t.color_bg_solid,
            t.color_bg_solid_hover,
            t.color_bg_solid_active,
        ],
        _ => color_base(color, t),
    }
}

/// `--ant-btn-shadow` 的着色令牌（宽度是 `controlOutlineWidth`）。
fn shadow_color(color: ButtonColor, t: &Tokens) -> Hsla {
    match color {
        ButtonColor::Default => t.control_tmp_outline,
        ButtonColor::Primary => t.color_primary_outline,
        ButtonColor::Danger => t.color_error_outline,
    }
}

/// 解析（色彩 × 变体 × 交互态）的完整外观。纯函数：不读全局、不含字面量。
pub fn state(
    color: ButtonColor,
    variant: ButtonVariant,
    tokens: &Tokens,
    ghost: bool,
    hover: bool,
    pressed: bool,
    disabled: bool,
) -> ButtonStyle {
    if disabled {
        // 文字 / 链接按钮禁用时保持透明底，只灰字。
        // antd 的 colorBorderDisabled 在明暗两套算法里都等于 colorBorder，故直接取描边令牌。
        let flat = matches!(variant, ButtonVariant::Text | ButtonVariant::Link);
        return ButtonStyle {
            fg: tokens.color_text_disabled,
            bg: if flat {
                transparent_black()
            } else {
                tokens.color_bg_container_disabled
            },
            border: if flat {
                transparent_black()
            } else {
                tokens.color_border
            },
            shadow: None,
            dashed: variant == ButtonVariant::Dashed,
        };
    }

    let i = state_index(hover, pressed);
    let (fg, bg, border, has_shadow) = match variant {
        ButtonVariant::Solid => {
            let fg = match color {
                ButtonColor::Default => metric::solid_text_color(tokens.color_bg_solid),
                _ => tokens.color_text_light_solid,
            };
            (
                fg,
                solid_bg_trio(color, tokens)[i],
                transparent_black(),
                !ghost,
            )
        }
        ButtonVariant::Outlined | ButtonVariant::Dashed => {
            // ghost 把底色换成透明；default 的描边与文字改为跟随容器底色
            let ghosted_default = ghost && color == ButtonColor::Default;
            (
                if ghosted_default {
                    tokens.color_bg_container
                } else {
                    text_base(color, tokens)[i]
                },
                if ghost {
                    transparent_black()
                } else {
                    tokens.color_bg_container
                },
                if ghosted_default {
                    tokens.color_bg_container
                } else {
                    color_base(color, tokens)[i]
                },
                !ghost,
            )
        }
        ButtonVariant::Filled => (
            text_base(color, tokens)[0],
            light_trio(color, tokens)[i],
            transparent_black(),
            false,
        ),
        ButtonVariant::Text => {
            // hover 用 color-light、active 用 color-light-active；静止无底
            let bg = match i {
                0 => transparent_black(),
                1 => light_trio(color, tokens)[0],
                _ => light_trio(color, tokens)[2],
            };
            let fg = if color == ButtonColor::Default {
                tokens.color_text
            } else {
                text_base(color, tokens)[i]
            };
            (fg, bg, transparent_black(), false)
        }
        // linkHoverBg 在 antd 里是 transparent：链接按钮任何时候都不铺底
        ButtonVariant::Link => (
            text_base(color, tokens)[i],
            transparent_black(),
            transparent_black(),
            false,
        ),
    };

    ButtonStyle {
        fg,
        bg,
        border,
        shadow: has_shadow.then(|| {
            metric::button_shadow(shadow_color(color, tokens), tokens.control_outline_width)
        }),
        dashed: variant == ButtonVariant::Dashed,
    }
}

/// 聚焦环：antd 用 `outline: lineWidthFocus solid colorPrimaryBorder` + `outline-offset: 1`。
/// 本控件库的按钮还不接键盘焦点，先把令牌换算暴露出来，供自绘焦点态的容器复用。
pub fn focus_ring(tokens: &Tokens, metric: &ButtonMetric) -> Vec<BoxShadow> {
    metric::focus_ring(tokens.color_primary_border, metric.focus_ring_width)
}

/// 通用按钮。
///
/// ```ignore
/// Button::new("保存")
///     .primary()
///     .on_click(cx.listener(|this, _, _window, cx| { /* ... */ }))
/// ```
pub struct Button {
    label: SharedString,
    id: Option<SharedString>,
    origin: SharedString,
    color: ButtonColor,
    variant: ButtonVariant,
    size: ButtonSize,
    shape: ButtonShape,
    ghost: bool,
    disabled: bool,
    loading: bool,
    block: bool,
    icon: Option<SharedString>,
    icon_element: Option<AnyElement>,
    on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
}

impl Button {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            label: label.into(),
            id: None,
            origin: format!(
                "button-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            color: ButtonColor::default(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            shape: ButtonShape::default(),
            ghost: false,
            disabled: false,
            loading: false,
            block: false,
            icon: None,
            icon_element: None,
            on_click: None,
        }
    }

    /// 交互态唯一标识。gpui 按 `ElementId` 跨帧 persist 按下/聚焦态
    /// （`element_states` 以 `GlobalElementId` 为键）；**同名按钮若共用一个 id，
    /// 会一起进入 active 态**。默认以构造点区分；循环或动态列表中反复在同一处
    /// 创建的按钮，必须用本旋钮传入彼此不同的 id（如按行 key/索引派生）。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 色彩族
    pub fn color(mut self, color: ButtonColor) -> Self {
        self.color = color;
        self
    }

    /// 样式变体
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 透明底（antd 的 `ghost` 修饰符，与色彩族正交）
    pub fn ghost_bg(mut self, ghost: bool) -> Self {
        self.ghost = ghost;
        self
    }

    /// 尺寸（antd 的 `size`）
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn small(self) -> Self {
        self.size(ButtonSize::Small)
    }

    pub fn large(self) -> Self {
        self.size(ButtonSize::Large)
    }

    /// 胶囊形按钮（全圆角）
    pub fn round(mut self) -> Self {
        self.shape = ButtonShape::Round;
        self
    }

    /// 正圆按钮：宽高相等，标签放单个图标字符（如 "＋"）
    pub fn circle(mut self) -> Self {
        self.shape = ButtonShape::Circle;
        self
    }

    /// 加载态：显示旋转指示器并阻止点击，配色保持正常态
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// 撑满父容器宽度
    pub fn block(mut self) -> Self {
        self.block = true;
        self
    }

    /// 标签前的图标字符（如 "⬇" / "＋"）
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 标签前的图标元素（如 svg）：与 `icon` 字符槽互斥，设置后优先渲染元素。
    /// 元素自身的着色由宿主负责；按钮容器带 [`ICON_GROUP`] 组名，可用
    /// `group_hover` 跟随悬停换色。
    pub fn icon_element(mut self, icon: impl IntoElement) -> Self {
        self.icon_element = Some(icon.into_any_element());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(listener));
        self
    }

    // ---- 旧 API 兼容：语义快捷方式 ----

    /// 主按钮：`primary` 色彩 + 实心
    pub fn primary(self) -> Self {
        self.color(ButtonColor::Primary)
            .variant(ButtonVariant::Solid)
    }

    /// 次按钮：默认色彩 + 描边
    pub fn secondary(self) -> Self {
        self.color(ButtonColor::Default)
            .variant(ButtonVariant::Outlined)
    }

    /// 危险按钮：`danger` 色彩 + 实心
    pub fn danger(self) -> Self {
        self.color(ButtonColor::Danger)
            .variant(ButtonVariant::Solid)
    }

    /// 幽灵按钮：透明底的描边按钮
    pub fn ghost(self) -> Self {
        self.variant(ButtonVariant::Outlined).ghost_bg(true)
    }
}

impl IntoElement for Button {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let size = self.size.index();
        let circle_side = self.shape == ButtonShape::Circle;
        let icon = self.icon.clone();
        let with_children = self.loading || icon.is_some() || self.icon_element.is_some();

        let (metric, normal, hover, active) = theme::with(|tokens| {
            let restyle = |hover: bool, pressed: bool| {
                state(
                    self.color,
                    self.variant,
                    tokens,
                    self.ghost,
                    hover,
                    pressed,
                    self.disabled,
                )
            };
            (
                ButtonMetric::resolve(tokens),
                restyle(false, false),
                restyle(true, false),
                restyle(false, true),
            )
        });
        let interactive = !self.disabled && !self.loading && self.on_click.is_some();
        let animated = !self.disabled && !self.loading;
        let element_id = ElementId::Name(
            self.id
                .clone()
                .unwrap_or_else(|| format!("{}-{}", self.origin, self.label).into()),
        );
        let aria_label = if self.label.is_empty() {
            SharedString::from("Button")
        } else {
            self.label.clone()
        };
        let dashed = normal.dashed;

        let mut element = div()
            .id(element_id.clone())
            .group(ICON_GROUP)
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .text_size(px(metric.font_size[size]))
            .font_weight(metric.font_weight)
            .h(px(metric.height[size]))
            .border(px(metric.border_width))
            .border_color(normal.border)
            .bg(normal.bg)
            .text_color(normal.fg)
            .shadow(normal.shadows());

        element = match self.shape {
            ButtonShape::Default => element.rounded(px(metric.radius[size])),
            ButtonShape::Round | ButtonShape::Circle => element.rounded_full(),
        };
        if circle_side {
            element = element.w(px(metric.height[size])).px_0();
        } else {
            element = element.px(px(metric.padding_inline[size]));
        }
        if dashed {
            element = element.border_dashed();
        }
        if self.block {
            element = element.w_full();
        }

        element = if self.disabled {
            element.cursor(CursorStyle::Arrow)
        } else {
            element.cursor(CursorStyle::PointingHand)
        };

        if with_children {
            element = element.gap(px(metric.icon_gap));
        }
        if self.loading {
            element = element.child(
                Spinner::new()
                    .size(px(metric.font_size[size]))
                    .color(normal.fg),
            );
        }
        if let Some(icon_el) = self.icon_element {
            element = element.child(icon_el);
        } else if let Some(icon) = icon {
            element = element.child(icon);
        }
        element = element.child(self.label);

        if interactive {
            if let Some(on_click) = self.on_click {
                element = a11y::click_shared(element, on_click);
            }
        }
        let semantics = a11y::Semantics::new(Role::Button, aria_label)
            .id(format!("{:?}", element_id))
            .disabled(self.disabled || self.loading)
            .focusable(interactive);
        let element = semantics.apply(element);
        let motion_id = format!("button-motion-{element_id:?}");
        let motion_value =
            motion::AnimatedValue::new(motion_id, element, 0.0, move |mut element, value| {
                let fg = motion::interactive_color(normal.fg, hover.fg, active.fg, value);
                let bg = motion::interactive_color(normal.bg, hover.bg, active.bg, value);
                let border =
                    motion::interactive_color(normal.border, hover.border, active.border, value);
                element = element
                    .bg(bg)
                    .text_color(fg)
                    .border_color(border)
                    .shadow(normal.shadows())
                    .text_size(px(metric.font_size[size]))
                    .font_weight(metric.font_weight);
                element
            })
            .duration(motion::fast_duration());
        let motion_value = if animated {
            motion_value.hover_refresh(motion::bind_hover_refresh)
        } else {
            motion_value
        };
        let root_id = ElementId::Name(format!("{element_id:?}-motion-root").into());
        div().id(root_id).child(motion_value)
    }
}
