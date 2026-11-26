//! 单行文本输入框：支持光标移动、选区、剪贴板与系统输入法（IME）。
//!
//! 实现参考 GPUI 官方 `examples/input.rs`：
//! 通过 `EntityInputHandler` 接入平台原生文本输入，
//! 通过自定义 `Element` 完成文本整形与绘制。
//!
//! 使用前需调用 [`bind_keys`] 注册编辑快捷键。

use std::{ops::Range, rc::Rc};

use gpui::{
    AccessibleAction, Action, App, Bounds, BoxShadow, ClipboardItem, Context, CursorStyle, Element,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, FocusHandle, Focusable,
    GlobalElementId, Hsla, InspectorElementId, InteractiveElement, IntoElement, KeyBinding,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    Role, ShapedLine, SharedString, StatefulInteractiveElement, Style, Subscription, TextRun,
    UTF16Selection, UnderlineStyle, Window, accesskit, div, fill, point, prelude::*, px, relative,
    size, transparent_black,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::a11y::{self, Semantics};
use crate::theme::{self, Tokens, metric};

gpui::actions!(
    widgets_text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        Paste,
        Cut,
        Copy,
    ]
);

/// 注册文本编辑快捷键（在应用启动时调用一次）。
///
/// `secondary` 在 macOS 映射 Cmd、其余平台映射 Ctrl；gpui 0.3.14 起
/// `KeyBinding::new` 不再解析该别名，需走按宿主 OS 解析的 `KeyBinding::load`。
pub fn bind_keys(cx: &mut App) {
    let mut bindings = vec![
        KeyBinding::new("backspace", Backspace, None),
        KeyBinding::new("delete", Delete, None),
        KeyBinding::new("left", Left, None),
        KeyBinding::new("right", Right, None),
        KeyBinding::new("shift-left", SelectLeft, None),
        KeyBinding::new("shift-right", SelectRight, None),
        KeyBinding::new("home", Home, None),
        KeyBinding::new("end", End, None),
    ];
    for (keys, action) in [
        ("secondary-a", Box::new(SelectAll) as Box<dyn Action>),
        ("secondary-v", Box::new(Paste)),
        ("secondary-c", Box::new(Copy)),
        ("secondary-x", Box::new(Cut)),
    ] {
        bindings
            .push(KeyBinding::load(keys, action, None, false, None, cx).expect("valid keystroke"));
    }
    cx.bind_keys(bindings);
}

/// 输入框外观变体（antdv-next Input `variant`）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputVariant {
    #[default]
    Outlined,
    Filled,
    Borderless,
    Underlined,
}

/// 校验状态（antdv-next Input `status`）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputStatus {
    #[default]
    Default,
    Error,
    Warning,
}

/// 三档尺寸（small / middle / large）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputSize {
    Small,
    #[default]
    Middle,
    Large,
}

impl InputSize {
    pub(crate) fn index(self) -> usize {
        match self {
            InputSize::Small => 0,
            InputSize::Middle => 1,
            InputSize::Large => 2,
        }
    }
}

/// 文本水平对齐（默认左对齐；设置步进器数值一类需要居中）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputAlign {
    #[default]
    Left,
    Center,
}

/// 对齐产生的横向偏移（纯函数，便于逐格断言）。
///
/// 左对齐恒为 0（与未引入对齐前逐字一致）；居中把行宽与容器宽的差平分，
/// 行比容器宽时夹回 0，避免负偏移把文本推出左边界。
pub fn align_offset(align: InputAlign, container_width: f32, line_width: f32) -> f32 {
    match align {
        InputAlign::Left => 0.0,
        InputAlign::Center => ((container_width - line_width) / 2.0).max(0.0),
    }
}

/// 某一交互态下输入框的最终外观（纯数据，可直接和浏览器基准比对）。
#[derive(Clone, Debug, PartialEq)]
pub struct InputStyle {
    pub fg: Hsla,
    pub placeholder: Hsla,
    /// 前后缀（prefix / suffix）颜色：状态色优先，否则跟随正文
    pub affix: Hsla,
    pub bg: Hsla,
    /// 上 / 右 / 下 / 左，顺序与 CSS 一致
    pub border: [Hsla; 4],
    pub border_width: [f32; 4],
    pub shadow: Option<BoxShadow>,
    pub height: f32,
    pub padding_block: f32,
    pub padding_inline: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
}

impl InputStyle {
    /// antd 用透明描边表达"无边框"，同一控件四边只会有一种可见色。
    pub(crate) fn border_color(&self) -> Hsla {
        self.border
            .into_iter()
            .find(|color| color.a > 0.0)
            .unwrap_or_else(transparent_black)
    }
}

/// 输入框状态矩阵：变体 × 状态 × 尺寸 × (hover, focus, disabled) → 外观。
///
/// 与 `button::state` 一样是纯函数，只依赖 `Tokens`，便于逐格对照 antd 基准。
pub fn state(
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    tokens: &Tokens,
    hover: bool,
    focused: bool,
    disabled: bool,
) -> InputStyle {
    let resolved = metric::InputMetric::resolve(tokens);
    let index = size.index();
    let transparent = transparent_black();
    let line = resolved.line_width;

    let status_border = match status {
        InputStatus::Default => tokens.color_border,
        InputStatus::Error => tokens.color_error,
        InputStatus::Warning => tokens.color_warning,
    };
    let status_hover_border = match status {
        InputStatus::Default => tokens.color_primary_hover,
        InputStatus::Error => tokens.color_error_border_hover,
        InputStatus::Warning => tokens.color_warning_border_hover,
    };
    let status_focus_border = match status {
        InputStatus::Default => tokens.color_primary,
        InputStatus::Error | InputStatus::Warning => status_border,
    };
    let status_outline = match status {
        InputStatus::Default => tokens.color_primary_outline,
        InputStatus::Error => tokens.color_error_outline,
        InputStatus::Warning => tokens.color_warning_outline,
    };

    // 填充 / 无边框变体会用状态色改正文色；其余变体的状态只影响描边
    let fg = match (variant, status, disabled) {
        (InputVariant::Filled, InputStatus::Error, false) => tokens.color_error_text,
        (InputVariant::Filled, InputStatus::Warning, false) => tokens.color_warning_text,
        (InputVariant::Borderless, InputStatus::Error, _) => tokens.color_error,
        (InputVariant::Borderless, InputStatus::Warning, _) => tokens.color_warning,
        (_, _, true) => tokens.color_text_disabled,
        _ => tokens.color_text,
    };

    let bg = match variant {
        InputVariant::Outlined => {
            if disabled {
                tokens.color_bg_container_disabled
            } else {
                tokens.color_bg_container
            }
        }
        InputVariant::Filled => {
            if disabled {
                tokens.color_bg_container_disabled
            } else if focused {
                tokens.color_bg_container
            } else if hover {
                match status {
                    InputStatus::Default => tokens.color_fill_secondary,
                    InputStatus::Error => tokens.color_error_bg_hover,
                    InputStatus::Warning => tokens.color_warning_bg_hover,
                }
            } else {
                match status {
                    InputStatus::Default => tokens.color_fill_tertiary,
                    InputStatus::Error => tokens.color_error_bg,
                    InputStatus::Warning => tokens.color_warning_bg,
                }
            }
        }
        InputVariant::Borderless => transparent,
        InputVariant::Underlined => tokens.color_bg_container,
    };

    let border_color = if disabled {
        match variant {
            InputVariant::Borderless => transparent,
            InputVariant::Filled => tokens.color_border,
            InputVariant::Outlined | InputVariant::Underlined => status_border,
        }
    } else if focused {
        match variant {
            InputVariant::Borderless => transparent,
            _ => status_focus_border,
        }
    } else if hover {
        match variant {
            InputVariant::Outlined | InputVariant::Underlined => status_hover_border,
            InputVariant::Filled | InputVariant::Borderless => transparent,
        }
    } else {
        match variant {
            InputVariant::Outlined | InputVariant::Underlined => status_border,
            InputVariant::Filled | InputVariant::Borderless => transparent,
        }
    };

    // underlined 只留一条底边（antd `border-width: 1px 0`），其余变体四边同色
    let border = match variant {
        InputVariant::Borderless => [transparent; 4],
        InputVariant::Underlined => [transparent, transparent, border_color, transparent],
        _ => [border_color; 4],
    };
    let border_width = match variant {
        InputVariant::Underlined => [line, 0.0, line, 0.0],
        _ => [line; 4],
    };

    // 只有 outlined / filled 有聚焦环；filled 的聚焦环被 antd 显式清掉（`outline: 0`）
    let shadow = if focused && !disabled && variant == InputVariant::Outlined {
        Some(metric::input_focus_shadow(
            status_outline,
            resolved.outline_width,
        ))
    } else {
        None
    };

    let affix = if !disabled {
        match status {
            InputStatus::Error => tokens.color_error,
            InputStatus::Warning => tokens.color_warning,
            InputStatus::Default => fg,
        }
    } else {
        fg
    };

    InputStyle {
        fg,
        placeholder: tokens.color_text_placeholder,
        affix,
        bg,
        border,
        border_width,
        shadow,
        height: resolved.height[index],
        padding_block: resolved.padding_block[index],
        padding_inline: resolved.padding_inline[index],
        radius: if variant == InputVariant::Underlined {
            0.0
        } else {
            resolved.radius[index]
        },
        font_size: resolved.font_size[index],
        line_height: resolved.line_height[index],
    }
}

/// 输入族共用外框（不含高度）：底色 / 描边 / 圆角 / 聚焦环。
///
/// 自动高度的控件（TextArea / TagInput / Autocomplete）走这里再自行设高度。
pub(crate) fn apply_frame_auto<T: InteractiveElement + Styled>(
    mut div: T,
    style: &InputStyle,
) -> T {
    // gpui 的 Div 只有一个 border_color，透明边必须同时把宽度归零，
    // 否则会被统一色刷上（underlined 的透明顶边就会画出整条上边框）。
    let mut widths = style.border_width;
    if style.border.iter().any(|color| color.a > 0.) {
        for (width, color) in widths.iter_mut().zip(style.border.iter()) {
            if color.a == 0. {
                *width = 0.;
            }
        }
    }
    div = div
        .bg(style.bg)
        .rounded(px(style.radius))
        .border_t(px(widths[0]))
        .border_r(px(widths[1]))
        .border_b(px(widths[2]))
        .border_l(px(widths[3]))
        .border_color(style.border_color());
    if let Some(shadow) = style.shadow.clone() {
        div = div.shadow(vec![shadow]);
    }
    div
}

/// 输入族共用外框：固定高度 + 底色 / 描边 / 圆角 / 聚焦环。
pub(crate) fn apply_frame<T: InteractiveElement + Styled>(div: T, style: &InputStyle) -> T {
    apply_frame_auto(div, style).h(px(style.height))
}

/// 悬停态只换底色与描边色；已聚焦时不覆盖焦点外观（antd 的 `:focus` 规则在 `:hover` 之后）。
pub(crate) fn hover_frame<T: InteractiveElement>(div: T, hover: &InputStyle, focused: bool) -> T {
    let hover = hover.clone();
    div.hover(move |style| {
        if focused {
            return style;
        }
        style.bg(hover.bg).border_color(hover.border_color())
    })
}

/// 单行文本输入框视图。
///
/// ```ignore
/// let input = cx.new(|cx| TextInput::new(cx).placeholder("请输入姓名"));
/// // render 中：div().child(input.clone())
/// ```
pub struct TextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
    masked: bool,
    bare: bool,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    disabled: bool,
    readonly: bool,
    allow_clear: bool,
    max_length: Option<usize>,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
    /// 覆盖令牌算出的字号（命令条这类比正文小一档的宿主输入位）。
    font_size: Option<f32>,
    /// 等宽字体（`monospace()` 打开）。
    monospace: bool,
    /// 文本水平对齐（默认左对齐）。
    align: InputAlign,
    on_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    on_blur: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    blur_subscription: Option<Subscription>,
    /// 上一帧对齐偏移，供鼠标命中换算（左对齐恒 0）。
    last_align_offset: f32,
    /// 包装控件为 bare 输入指定的语义；未指定时 bare 输入不进入可访问性树。
    a11y_semantics: Option<Semantics>,
    a11y_set_open: Option<Rc<dyn Fn(bool, &mut Window, &mut App)>>,
}

impl TextInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: "".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
            masked: false,
            bare: false,
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            disabled: false,
            readonly: false,
            allow_clear: false,
            max_length: None,
            prefix: None,
            suffix: None,
            font_size: None,
            monospace: false,
            align: InputAlign::default(),
            on_change: None,
            on_blur: None,
            blur_subscription: None,
            last_align_offset: 0.0,
            a11y_semantics: None,
            a11y_set_open: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 密码掩码模式：显示为圆点（IME 组合期间临时显示原文）
    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    /// 运行时切换掩码模式
    pub fn set_masked(&mut self, masked: bool, cx: &mut Context<Self>) {
        self.masked = masked;
        cx.notify();
    }

    /// 包装控件同步外观：外层已画输入框时，内部 bare 输入只需要跟随
    /// 字号 / 状态色 / 禁用态（TagInput / Autocomplete 等）。
    pub(crate) fn set_form_state(
        &mut self,
        variant: InputVariant,
        status: InputStatus,
        size: InputSize,
        disabled: bool,
        cx: &mut Context<Self>,
    ) {
        if (self.variant, self.status, self.size, self.disabled)
            != (variant, status, size, disabled)
        {
            self.variant = variant;
            self.status = status;
            self.size = size;
            self.disabled = disabled;
            cx.notify();
        }
    }

    /// 裸模式：去掉边框 / 背景 / 内边距，供包装控件（密码框 / 搜索框等）嵌入使用
    pub fn bare(mut self, bare: bool) -> Self {
        self.bare = bare;
        self
    }

    /// 包装控件替换 bare 输入的可访问性角色与稳定 id。
    pub(crate) fn set_a11y_semantics(&mut self, semantics: Semantics) {
        self.a11y_semantics = Some(semantics);
    }

    pub(crate) fn set_a11y_set_open(
        &mut self,
        listener: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) {
        self.a11y_set_open = Some(Rc::new(listener));
    }

    pub(crate) fn a11y_value(&self) -> SharedString {
        if self.masked && !self.content.is_empty() {
            "•".repeat(self.content.chars().count()).into()
        } else {
            self.content.clone()
        }
    }

    fn semantics(&self, default_id: SharedString) -> Semantics {
        let label: SharedString = if self.placeholder.is_empty() {
            "文本输入".into()
        } else {
            self.placeholder.clone()
        };
        self.a11y_semantics
            .clone()
            .unwrap_or_else(|| Semantics::new(Role::TextInput, label).id(default_id))
            .value(self.a11y_value())
            .placeholder(self.placeholder.clone())
            .disabled(self.disabled)
            .read_only(self.readonly)
            .invalid(self.status == InputStatus::Error)
            .focusable(!self.disabled)
    }

    fn offset_from_a11y_index(&self, index: usize) -> usize {
        self.content
            .char_indices()
            .nth(index)
            .map(|(offset, _)| offset)
            .unwrap_or(self.content.len())
    }

    fn set_value_from_a11y(
        &mut self,
        data: Option<&accesskit::ActionData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || self.readonly {
            return;
        }
        let value = match data {
            Some(accesskit::ActionData::Value(value)) => value.to_string(),
            Some(accesskit::ActionData::NumericValue(value)) => value.to_string(),
            _ => return,
        };
        self.set_text(value.replace('\n', " "), window, cx);
    }

    fn replace_selected_from_a11y(
        &mut self,
        data: Option<&accesskit::ActionData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(accesskit::ActionData::Value(value)) = data else {
            return;
        };
        self.replace_text_in_range(None, &value.replace('\n', " "), window, cx);
    }

    fn set_selection_from_a11y(
        &mut self,
        data: Option<&accesskit::ActionData>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let Some(accesskit::ActionData::SetTextSelection(selection)) = data else {
            return;
        };
        let anchor = self.offset_from_a11y_index(selection.anchor.character_index);
        let focus = self.offset_from_a11y_index(selection.focus.character_index);
        self.selection_reversed = anchor > focus;
        self.selected_range = anchor.min(focus)..anchor.max(focus);
        self.marked_range = None;
        cx.notify();
    }

    /// 在承载焦点与 action 的 Div 上发布输入语义，bare 绘制节点不重复暴露。
    pub(crate) fn with_a11y<T: StatefulInteractiveElement>(
        &self,
        element: T,
        cx: &Context<Self>,
    ) -> T {
        let entity = cx.entity();
        let semantics = self.semantics(format!("text-input-{:?}", entity.entity_id()).into());
        self.with_a11y_actions(semantics.apply(element), cx)
    }

    /// 在包装控件已发布的角色 / 名称 / 状态上补充文本 action 与 TextRun。
    pub(crate) fn with_a11y_actions<T: StatefulInteractiveElement>(
        &self,
        element: T,
        cx: &Context<Self>,
    ) -> T {
        let entity = cx.entity();
        let mut element = element;
        if !self.disabled {
            element = element.on_a11y_action(AccessibleAction::SetTextSelection, {
                let entity = entity.clone();
                move |data, window, cx| {
                    let _ = entity.update(cx, |input, cx| {
                        input.set_selection_from_a11y(data, window, cx)
                    });
                }
            });
            if !self.readonly {
                element = element
                    .on_a11y_action(AccessibleAction::SetValue, {
                        let entity = entity.clone();
                        move |data, window, cx| {
                            let _ = entity.update(cx, |input, cx| {
                                input.set_value_from_a11y(data, window, cx)
                            });
                        }
                    })
                    .on_a11y_action(AccessibleAction::ReplaceSelectedText, {
                        let entity = entity.clone();
                        move |data, window, cx| {
                            let _ = entity.update(cx, |input, cx| {
                                input.replace_selected_from_a11y(data, window, cx)
                            });
                        }
                    });
            }
        }
        if !self.disabled
            && let Some(listener) = self.a11y_set_open.clone()
        {
            element = element.on_a11y_action(AccessibleAction::Expand, {
                let listener = listener.clone();
                move |_, window, cx| listener(true, window, cx)
            });
            element = element.on_a11y_action(AccessibleAction::Collapse, {
                move |_, window, cx| listener(false, window, cx)
            });
        }
        let text = self.a11y_value();
        let disabled = self.disabled;
        let read_only = self.readonly;
        let invalid = self.status == InputStatus::Error;
        let start = self.content[..self.selected_range.start.min(self.content.len())]
            .chars()
            .count();
        let end = self.content[..self.selected_range.end.min(self.content.len())]
            .chars()
            .count();
        let (anchor, focus) = if self.selection_reversed {
            (end, start)
        } else {
            (start, end)
        };
        element.a11y_synthetic_children(move |builder| {
            let parent = builder.parent_node();
            if disabled {
                parent.set_disabled();
            }
            if read_only {
                parent.set_read_only();
            }
            if invalid {
                parent.set_invalid(accesskit::Invalid::True);
            }
            let mut run = accesskit::Node::new(Role::TextRun);
            run.set_value(text.to_string());
            run.set_character_lengths(
                text.chars()
                    .map(|ch| ch.len_utf8() as u8)
                    .collect::<Vec<u8>>(),
            );
            let run_id = builder.synthetic_node_id(0);
            builder.push_child(run_id, run);
            builder
                .parent_node()
                .set_text_selection(accesskit::TextSelection {
                    anchor: accesskit::TextPosition {
                        node: run_id,
                        character_index: anchor,
                    },
                    focus: accesskit::TextPosition {
                        node: run_id,
                        character_index: focus,
                    },
                });
        })
    }

    /// 禁用态：灰底、不可聚焦 / 编辑
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 只读：可选中 / 复制，不可编辑
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// 有内容时显示 ✕ 一键清空
    pub fn allow_clear(mut self, allow_clear: bool) -> Self {
        self.allow_clear = allow_clear;
        self
    }

    /// 最大字符数（仅限制用户输入，程序化写入不受限）
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// 框内前缀文本（如图标字符 / 单位）
    pub fn prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// 框内后缀文本
    pub fn suffix(mut self, suffix: impl Into<SharedString>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// 外观变体（默认 outlined）
    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    /// 校验状态（默认 / 错误 / 警告）
    pub fn status(mut self, status: InputStatus) -> Self {
        self.status = status;
        self
    }

    /// 尺寸（small / middle / large）
    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    /// 错误态简写：等价于 `status(InputStatus::Error)`
    pub fn error(mut self, error: bool) -> Self {
        self.status = if error {
            InputStatus::Error
        } else {
            InputStatus::Default
        };
        self
    }

    /// 等宽字体（终端命令、路径一类输入位）。默认关闭，字号仍跟令牌。
    pub fn monospace(mut self) -> Self {
        self.monospace = true;
        self
    }

    /// 覆盖令牌字号（如命令条输入框的 12px）。不设则跟 `size` 派生值。
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    /// 运行时切换等宽与字号覆盖（包装控件按形态重排时用）。
    pub fn set_typography(&mut self, monospace: bool, font_size: Option<f32>) {
        self.monospace = monospace;
        self.font_size = font_size;
    }

    /// 运行时设置占位文本（不触发 notify，由调用方决定）
    pub fn set_placeholder(&mut self, placeholder: impl Into<SharedString>) {
        self.placeholder = placeholder.into();
    }

    /// 内容变化回调（参数为最新文本）
    pub fn on_change(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    /// 文本水平对齐（默认左对齐；居中用于步进器数值一类）。
    pub fn align(mut self, align: InputAlign) -> Self {
        self.align = align;
        self
    }

    /// 失焦回调（参数为失焦时的文本）。仅在真正失去焦点时触发一次，
    /// 供「失焦才提交」的宿主输入位（设置文本卡）使用。
    pub fn on_blur(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_blur = Some(Rc::new(listener));
        self
    }

    /// 当前文本
    pub fn text(&self) -> &str {
        &self.content
    }

    /// 程序化设置文本
    pub fn set_text(
        &mut self,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_text_silent(text, cx);
        self.notify_change(window, cx);
    }

    /// 程序化写入但不触发 on_change：供包装控件（密码框 / 搜索框等）在自身
    /// update 期间写入内部输入框，避免回调反向 update 包装实体导致双重租约 panic
    pub fn set_text_silent(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = text.into();
        self.selected_range = self.content.len()..self.content.len();
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
    }

    fn notify_change(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.content, window, cx);
        }
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_text_silent("", cx);
        // 静默写入不触发 on_change，手动补通知
        self.notify_change(window, cx);
        self.focus_handle.focus(window, cx);
    }

    /// max_length 截断插入文本（按字符数）
    fn clamp_insert(&self, range: &Range<usize>, new_text: &str) -> String {
        let Some(max) = self.max_length else {
            return new_text.to_string();
        };
        let kept = self.content.chars().count() - self.content[range.clone()].chars().count();
        let allow = max.saturating_sub(kept);
        new_text.chars().take(allow).collect()
    }

    fn left(&mut self, _: &Left, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), window, cx);
        } else {
            self.move_to(self.selected_range.start, window, cx)
        }
    }

    fn right(&mut self, _: &Right, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), window, cx);
        } else {
            self.move_to(self.selected_range.end, window, cx)
        }
    }

    fn select_left(&mut self, _: &SelectLeft, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), window, cx);
    }

    fn select_right(&mut self, _: &SelectRight, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), window, cx);
    }

    fn select_all(&mut self, _: &SelectAll, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
        self.select_to(self.content.len(), window, cx)
    }

    fn home(&mut self, _: &Home, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
    }

    fn end(&mut self, _: &End, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), window, cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            if self.cursor_offset() == prev {
                return;
            }
            self.select_to(prev, window, cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if self.cursor_offset() == next {
                return;
            }
            self.select_to(next, window, cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), window, cx)
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        }
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text.replace('\n', " "), window, cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn move_to(&mut self, offset: usize, _window: &Window, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        cx.notify()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        let index =
            line.closest_index_for_x(position.x - bounds.left() - px(self.last_align_offset));
        if self.masked_display().is_some() {
            self.from_masked_offset(index)
        } else {
            index
        }
    }

    fn select_to(&mut self, offset: usize, _window: &Window, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    /// 掩码显示文本（IME 组合期间不掩码，临时显示原文）
    fn masked_display(&self) -> Option<SharedString> {
        if self.masked && self.marked_range.is_none() && !self.content.is_empty() {
            Some("•".repeat(self.content.chars().count()).into())
        } else {
            None
        }
    }

    /// 原文字节偏移 -> 掩码文本字节偏移（每个字符对应一个 3 字节的 •）
    fn to_masked_offset(&self, offset: usize) -> usize {
        3 * self.content[..offset].chars().count()
    }

    /// 掩码文本字节偏移 -> 原文字节偏移
    fn from_masked_offset(&self, masked: usize) -> usize {
        let chars = masked / 3;
        self.content
            .char_indices()
            .nth(chars)
            .map(|(i, _)| i)
            .unwrap_or(self.content.len())
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || self.readonly {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let inserted = self.clamp_insert(&range, new_text);

        self.content =
            (self.content[0..range.start].to_owned() + &inserted + &self.content[range.end..])
                .into();
        self.selected_range = range.start + inserted.len()..range.start + inserted.len();
        self.selection_reversed = false;
        self.marked_range.take();
        self.notify_change(window, cx);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || self.readonly {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let inserted = self.clamp_insert(&range, new_text);
        let new_text = inserted.as_str();

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        let insertion_start_utf16 = self.offset_to_utf16(range.start);
        let inserted_length_utf16 = new_text.encode_utf16().count();
        self.selected_range = new_selected_range_utf16
            .map(|selected| {
                self.range_from_utf16(
                    &(insertion_start_utf16 + selected.start.min(inserted_length_utf16)
                        ..insertion_start_utf16 + selected.end.min(inserted_length_utf16)),
                )
            })
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        self.selection_reversed = false;

        self.notify_change(window, cx);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let masked = self.masked_display();
        let expected: &str = masked.as_ref().map(|m| m.as_ref()).unwrap_or(&self.content);
        if last_layout.text.as_ref() != expected {
            return None;
        }
        let range = self.range_from_utf16(&range_utf16);
        let (start, end) = if masked.is_some() {
            (
                self.to_masked_offset(range.start),
                self.to_masked_offset(range.end),
            )
        } else {
            (range.start, range.end)
        };
        Some(Bounds::from_corners(
            point(bounds.left() + last_layout.x_for_index(start), bounds.top()),
            point(
                bounds.left() + last_layout.x_for_index(end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;

        let masked = self.masked_display();
        let expected: &str = masked.as_ref().map(|m| m.as_ref()).unwrap_or(&self.content);
        if last_layout.text.as_ref() != expected {
            return None;
        }
        let mut utf8_index = last_layout.closest_index_for_x(line_point.x);
        if masked.is_some() {
            utf8_index = self.from_masked_offset(utf8_index);
        }
        Some(self.offset_to_utf16(utf8_index))
    }
}

/// 负责文本整形与绘制的内部元素
struct TextElement {
    input: Entity<TextInput>,
    a11y: Semantics,
}

struct TextPrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
    align_offset: f32,
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = TextPrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn a11y_role(&self) -> Option<Role> {
        Some(self.a11y.role())
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        self.a11y.write(node);
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();

        let masked = input.masked_display();
        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), theme::text_placeholder())
        } else if let Some(masked) = masked.clone() {
            (masked, style.color)
        } else {
            (content, style.color)
        };
        // 掩码模式下，光标 / 选区偏移需映射到掩码文本
        let map_offset = |offset: usize| {
            if masked.is_some() {
                input.to_masked_offset(offset)
            } else {
                offset
            }
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let offset = px(align_offset(
            input.align,
            f32::from(bounds.size.width),
            f32::from(line.width),
        ));
        let cursor_pos = line.x_for_index(map_offset(cursor));
        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + offset + cursor_pos, bounds.top()),
                        size(px(1.5), bounds.bottom() - bounds.top()),
                    ),
                    theme::primary(),
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left()
                                + offset
                                + line.x_for_index(map_offset(selected_range.start)),
                            bounds.top(),
                        ),
                        point(
                            bounds.left()
                                + offset
                                + line.x_for_index(map_offset(selected_range.end)),
                            bounds.bottom(),
                        ),
                    ),
                    theme::primary_bg(),
                )),
                None,
            )
        };
        TextPrepaintState {
            line: Some(line),
            cursor,
            selection,
            align_offset: f32::from(offset),
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        let interactive = {
            let input = self.input.read(cx);
            !input.disabled && !input.readonly
        };
        if interactive {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
                cx,
            );
        }
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection)
        }
        let line = prepaint.line.take().unwrap();
        let align_offset = prepaint.align_offset;
        line.paint(
            point(bounds.left() + px(align_offset), bounds.top()),
            window.line_height(),
            gpui::TextAlign::default(),
            None,
            window,
            cx,
        )
        .unwrap();

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
            input.last_align_offset = align_offset;
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.on_blur.is_some() && self.blur_subscription.is_none() {
            let cb = self.on_blur.clone().unwrap();
            let handle = self.focus_handle.clone();
            self.blur_subscription = Some(cx.on_blur(&handle, window, move |input, window, cx| {
                cb(&input.content, window, cx);
            }));
        }
        let focused = self.focus_handle.is_focused(window);
        let disabled = self.disabled;
        let bare = self.bare;
        let a11y = self.semantics(format!("text-input-{:?}", cx.entity_id()).into());
        let (variant, status, size) = (self.variant, self.status, self.size);
        let style = theme::with(|t| state(variant, status, size, t, false, focused, disabled));
        let hover_style = theme::with(|t| state(variant, status, size, t, true, false, disabled));
        let gap = theme::with(|t| t.padding_xxs);
        let (font_size, mono_family) =
            metric::input_typography(style.font_size, self.font_size, self.monospace);

        let mut root = div()
            .id(a11y.element_id())
            .key_context("TextInput")
            .flex()
            .flex_row()
            .items_center()
            .gap(px(gap))
            .w_full()
            .overflow_hidden()
            .line_height(px(style.line_height))
            .text_size(px(font_size))
            .text_color(style.fg);
        if let Some(family) = mono_family {
            root = root.font_family(family);
        }

        if !bare {
            root = root
                .px(px(style.padding_inline))
                .py(px(style.padding_block));
            root = apply_frame(root, &style);
            root = hover_frame(root, &hover_style, focused);
        }

        if disabled {
            root = root.cursor(CursorStyle::Arrow);
        } else {
            root = root
                .track_focus(&self.focus_handle(cx))
                .cursor(CursorStyle::IBeam)
                .on_action(cx.listener(Self::backspace))
                .on_action(cx.listener(Self::delete))
                .on_action(cx.listener(Self::left))
                .on_action(cx.listener(Self::right))
                .on_action(cx.listener(Self::select_left))
                .on_action(cx.listener(Self::select_right))
                .on_action(cx.listener(Self::select_all))
                .on_action(cx.listener(Self::home))
                .on_action(cx.listener(Self::end))
                .on_action(cx.listener(Self::paste))
                .on_action(cx.listener(Self::cut))
                .on_action(cx.listener(Self::copy))
                .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
                .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
                .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
                .on_mouse_move(cx.listener(Self::on_mouse_move));
        }

        if let Some(prefix) = self.prefix.clone() {
            root = root.child(div().text_color(style.affix).child(prefix));
        }
        root = root.child(div().flex_1().min_w(px(0.)).child(TextElement {
            input: cx.entity(),
            a11y: a11y.clone(),
        }));
        if let Some(suffix) = self.suffix.clone() {
            root = root.child(div().text_color(style.affix).child(suffix));
        }
        if self.allow_clear && !disabled && !self.readonly && !self.content.is_empty() {
            let entity = cx.entity();
            let clear = div()
                .id(ElementId::Name(
                    format!("text-input-clear-{:?}", cx.entity_id()).into(),
                ))
                .cursor(CursorStyle::PointingHand)
                .text_size(px(12.))
                .text_color(theme::text_quaternary())
                .hover(|style| style.text_color(theme::text()))
                .child("✕");
            let clear = Semantics::new(Role::Button, "清除文本")
                .id(format!("text-input-clear-{:?}", cx.entity_id()))
                .focusable(true)
                .apply(clear);
            root = root.child(a11y::click_void(clear, move |window, cx| {
                let _ = entity.update(cx, |input, cx| input.clear(window, cx));
            }));
        }
        if !bare || self.a11y_semantics.is_some() {
            root = self.with_a11y(root, cx);
        }
        root
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
