//! 多行文本输入框：硬换行（Enter）、跨行选区、上下行光标移动、IME。
//!
//! 复用 [`crate::text_input::bind_keys`] 注册的编辑快捷键，
//! 额外注册 Enter / Up / Down 等本模块动作。
//!
//! 说明：v1 按 `\n` 分行整形，不做可视宽度软换行；
//! 超出高度的内容通过光标自动滚动保持可见。

use std::{ops::Range, rc::Rc};

use gpui::{
    AccessibleAction, App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, Element,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, FocusHandle, Focusable, Font,
    FontWeight, GlobalElementId, Hsla, InspectorElementId, IntoElement, KeyBinding, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point, Role,
    ScrollDelta, ScrollEvent, ScrollHandle, ShapedLine, SharedString, StatefulInteractiveElement,
    Style, TextRun, UTF16Selection, UnderlineStyle, Window, accesskit, div, fill, point,
    prelude::*, px, relative, size,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::Scrollbar;
use crate::a11y::Semantics;
use crate::text_input::{self, InputSize, InputStatus, InputStyle, InputVariant};
use crate::theme::{self, Tokens, metric};

gpui::actions!(
    widgets_text_area,
    [
        Enter, Newline, Up, Down, SelectUp, SelectDown, SelectHome, SelectEnd, Indent, Outdent,
    ]
);

/// 注册多行编辑快捷键（在应用启动时调用一次，与 text_input::bind_keys 并存）。
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", Enter, Some("TextArea")),
        KeyBinding::new("shift-enter", Newline, Some("TextArea")),
        KeyBinding::new("up", Up, Some("TextArea")),
        KeyBinding::new("down", Down, Some("TextArea")),
        KeyBinding::new("shift-up", SelectUp, Some("TextArea")),
        KeyBinding::new("shift-down", SelectDown, Some("TextArea")),
        KeyBinding::new("shift-home", SelectHome, Some("TextArea")),
        KeyBinding::new("shift-end", SelectEnd, Some("TextArea")),
        // Tab 在代码编辑器中缩进（不再做焦点切换）
        KeyBinding::new("tab", Indent, Some("TextArea")),
        KeyBinding::new("shift-tab", Outdent, Some("TextArea")),
    ]);
}

/// 多行输入解析后的度量（纯数据，可直接和浏览器基准比对）。
///
/// 外框与单行输入共用 [`text_input::state`]；这里只补多行特有的内容度量：
/// antd 的 textarea 高度由内容决定（`height: auto`），行高是无单位 `line-height` × 字号。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextAreaMetrics {
    pub pad_x: f32,
    pub pad_y: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub min_height: f32,
    pub border: f32,
}

/// TextArea 滚动度量；距离均从内容顶部起算，单位为像素。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextAreaScrollMetrics {
    pub content_height: f32,
    pub viewport_height: f32,
    pub max_scroll: f32,
}

/// 计算多行内容度量；`pad` / `text_size` 是包装控件（代码编辑器）的显式覆盖。
pub fn metrics(
    size: InputSize,
    tokens: &Tokens,
    pad: Option<(f32, f32)>,
    text_size: Option<f32>,
) -> TextAreaMetrics {
    let resolved = metric::InputMetric::resolve(tokens);
    let index = size.index();
    let font_size = text_size.unwrap_or(resolved.font_size[index]);
    // antd 的 textarea 规则把 `line-height` 固定成基础 `token.lineHeight`（大号也不切 LG），
    // 且 `min-height: controlHeight` 与尺寸档无关 —— 两处都和单行输入不同。
    let line_height = font_size * tokens.line_height;
    let (pad_x, pad_y) = pad.unwrap_or((
        resolved.padding_inline[index],
        resolved.padding_block[index],
    ));
    TextAreaMetrics {
        pad_x,
        pad_y,
        font_size,
        line_height,
        min_height: tokens.control_height,
        border: resolved.line_width,
    }
}

/// TextArea 外框状态矩阵，语义与 [`text_input::state`] 完全一致（同一套 Input 令牌）。
pub fn state(
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    tokens: &Tokens,
    hover: bool,
    focused: bool,
    disabled: bool,
) -> InputStyle {
    text_input::state(variant, status, size, tokens, hover, focused, disabled)
}

/// 字节偏移 → 行/列（1 起），口径照原版 FileEditor.updateCaret：
/// 行 = 偏移前 `\n` 数 + 1，列 = 偏移 - 上一个换行符位置（首行从 1 起）。
/// 越界偏移夹到内容末尾；按字节计（ASCII 内容与原版 UTF-16 计数一致）。
pub fn caret_line_col(content: &str, pos: usize) -> (usize, usize) {
    let pos = pos.min(content.len());
    let mut line = 1;
    let mut line_start = 0;
    for (i, b) in content.bytes().enumerate().take(pos) {
        if b == b'\n' {
            line += 1;
            line_start = i + 1;
        }
    }
    (line, pos - line_start + 1)
}

/// 高亮 token：行内字节区间 + 颜色 + 是否加粗。
///
/// 高亮器收到的行文本不含 `\n`，返回的区间需完整、连续地覆盖整行，
/// 以便组成一行的 [`TextRun`] 序列。
#[derive(Clone)]
pub struct CodeToken {
    pub range: Range<usize>,
    pub color: Hsla,
    pub bold: bool,
}

impl CodeToken {
    pub fn new(range: Range<usize>, color: Hsla) -> Self {
        Self {
            range,
            color,
            bold: false,
        }
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }
}

/// 按行高亮器
pub type Highlighter = Rc<dyn Fn(&str) -> Vec<CodeToken>>;

/// 多行文本输入框视图。
///
/// ```ignore
/// let area = cx.new(|cx| TextArea::new(cx).placeholder("请输入内容").height(px(120.)));
/// // render 中：div().child(area.clone())
/// ```
pub struct TextArea {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    height: Pixels,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    /// 每行的整形结果与起始字节偏移
    last_lines: Vec<(usize, ShapedLine)>,
    last_bounds: Option<Bounds<Pixels>>,
    scroll_y: Pixels,
    scroll_handle: ScrollHandle,
    /// 上次「光标带入视口」时的光标偏移。滚轮不改光标 ⇒ 与本值相等 ⇒ render 不再纠正滚动。
    scroll_anchor: Option<usize>,
    is_selecting: bool,
    /// 上下移动时保持的目标列（字节列）
    preferred_col: Option<usize>,
    on_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    /// 提交模式：Enter 触发 on_submit 而非换行（Shift+Enter 仍换行）。
    on_submit: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    /// 代码模式：左侧行号栏
    line_numbers: bool,
    /// 代码模式：当前行背景高亮
    current_line: bool,
    /// 按行语法高亮器
    highlighter: Option<Highlighter>,
    /// Tab 缩进空格数（0 表示插入制表符）
    tab_spaces: usize,
    /// 行号栏宽度（由 prepaint 计算，供命中测试使用）
    gutter: Pixels,
    /// 内容内边距覆盖 (x, y)；`None` 时取 Input 令牌
    pad: Option<(f32, f32)>,
    /// 字号覆盖；`None` 时取 Input 令牌
    text_size: Option<f32>,
    /// 字体族（代码模式用等宽字体）
    font_family: Option<SharedString>,
    variant: InputVariant,
    status: InputStatus,
    size: InputSize,
    disabled: bool,
    a11y_semantics: Option<crate::a11y::Semantics>,
}

impl TextArea {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: "".into(),
            height: px(120.),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_lines: Vec::new(),
            last_bounds: None,
            scroll_y: px(0.),
            scroll_handle: ScrollHandle::new(),
            scroll_anchor: None,
            is_selecting: false,
            preferred_col: None,
            on_change: None,
            on_submit: None,
            line_numbers: false,
            current_line: false,
            highlighter: None,
            tab_spaces: 0,
            gutter: px(0.),
            pad: None,
            text_size: None,
            font_family: None,
            variant: InputVariant::default(),
            status: InputStatus::default(),
            size: InputSize::default(),
            disabled: false,
            a11y_semantics: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 初始内容（构造时使用，光标置于末尾，不触发变更回调）
    pub fn initial_text(mut self, text: impl Into<SharedString>) -> Self {
        self.content = text.into();
        self.selected_range = self.content.len()..self.content.len();
        self.selection_reversed = false;
        self
    }

    /// 运行时设置占位文字（供包装控件每帧同步）。
    pub fn set_placeholder(&mut self, placeholder: impl Into<SharedString>) {
        self.placeholder = placeholder.into();
    }

    /// 可视高度（默认 120px）
    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    /// 左侧显示行号栏（代码模式）
    pub fn line_numbers(mut self, line_numbers: bool) -> Self {
        self.line_numbers = line_numbers;
        self
    }

    /// 当前行背景高亮（代码模式）
    pub fn current_line(mut self, current_line: bool) -> Self {
        self.current_line = current_line;
        self
    }

    /// 按行语法高亮器（见 [`crate::code_editor`]）
    pub fn highlighter(mut self, highlighter: Highlighter) -> Self {
        self.highlighter = Some(highlighter);
        self
    }

    /// 运行时替换 / 关闭高亮器
    pub fn set_highlighter(&mut self, highlighter: Option<Highlighter>, cx: &mut Context<Self>) {
        self.highlighter = highlighter;
        cx.notify();
    }

    /// Tab 缩进空格数（默认 0，插入制表符）
    pub fn tab_spaces(mut self, spaces: usize) -> Self {
        self.tab_spaces = spaces;
        self
    }

    /// 内容内边距覆盖（默认取 Input 令牌）
    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.pad = Some((x, y));
        self
    }

    /// 字号覆盖（默认取 Input 令牌）
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = Some(size);
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

    /// 禁用态：灰底、不可聚焦 / 编辑
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 包装控件替换多行输入的可访问性标签与稳定 id。
    pub(crate) fn set_a11y_semantics(&mut self, semantics: crate::a11y::Semantics) {
        self.a11y_semantics = Some(semantics);
    }

    fn semantics(&self, default_id: SharedString) -> Semantics {
        let label: SharedString = if self.placeholder.is_empty() {
            "多行文本输入".into()
        } else {
            self.placeholder.clone()
        };
        self.a11y_semantics
            .clone()
            .unwrap_or_else(|| Semantics::new(Role::MultilineTextInput, label).id(default_id))
            .value(self.content.clone())
            .placeholder(self.placeholder.clone())
            .disabled(self.disabled)
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
        if self.disabled {
            return;
        }
        let value = match data {
            Some(accesskit::ActionData::Value(value)) => value.to_string(),
            Some(accesskit::ActionData::NumericValue(value)) => value.to_string(),
            _ => return,
        };
        self.set_text(value, window, cx);
    }

    fn replace_selected_from_a11y(
        &mut self,
        data: Option<&accesskit::ActionData>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let Some(accesskit::ActionData::Value(value)) = data else {
            return;
        };
        self.replace_text_in_range(None, value, window, cx);
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
        self.preferred_col = None;
        cx.notify();
    }

    fn scroll_from_a11y(
        &mut self,
        down: bool,
        data: Option<&accesskit::ActionData>,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let m = self.metrics();
        let amount = match data {
            Some(accesskit::ActionData::ScrollUnit(accesskit::ScrollUnit::Page)) => {
                f32::from(self.viewport_height(&m))
            }
            _ => m.line_height,
        };
        let delta = if down { -amount } else { amount };
        let next = Self::resolve_wheel_scroll(
            f32::from(self.scroll_y),
            delta,
            f32::from(self.max_scroll(&m)),
        );
        self.set_scroll_offset(px(next), cx);
    }

    fn set_scroll_from_a11y(
        &mut self,
        data: Option<&accesskit::ActionData>,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        if let Some(accesskit::ActionData::SetScrollOffset(offset)) = data {
            self.set_scroll_offset(px(-(offset.y as f32)), cx);
        }
    }

    /// 在承载焦点与 action 的 Div 上发布多行输入语义，绘制元素不重复暴露。
    pub(crate) fn with_a11y<T: StatefulInteractiveElement>(
        &self,
        element: T,
        cx: &Context<Self>,
    ) -> T {
        let entity = cx.entity();
        let semantics = self.semantics(format!("text-area-{:?}", entity.entity_id()).into());
        let mut element = semantics.apply(element);
        if !self.disabled {
            element = element
                .on_a11y_action(AccessibleAction::SetValue, {
                    let entity = entity.clone();
                    move |data, window, cx| {
                        let _ = entity
                            .update(cx, |input, cx| input.set_value_from_a11y(data, window, cx));
                    }
                })
                .on_a11y_action(AccessibleAction::ReplaceSelectedText, {
                    let entity = entity.clone();
                    move |data, window, cx| {
                        let _ = entity.update(cx, |input, cx| {
                            input.replace_selected_from_a11y(data, window, cx)
                        });
                    }
                })
                .on_a11y_action(AccessibleAction::SetTextSelection, {
                    let entity = entity.clone();
                    move |data, window, cx| {
                        let _ = entity.update(cx, |input, cx| {
                            input.set_selection_from_a11y(data, window, cx)
                        });
                    }
                });
        }

        let text = self.content.clone();
        let disabled = self.disabled;
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

    /// 当前尺寸对应的内容度量（行高 / 内边距 / 字号 / 边框宽）
    pub fn metrics(&self) -> TextAreaMetrics {
        theme::with(|tokens| metrics(self.size, tokens, self.pad, self.text_size))
    }

    /// 使用平台等宽字体（代码模式）；宿主全局等宽覆盖优先。
    pub fn monospace(mut self) -> Self {
        let family = crate::theme::mono_family_or(if cfg!(target_os = "windows") {
            "Consolas"
        } else if cfg!(target_os = "macos") {
            "Menlo"
        } else {
            "monospace"
        });
        self.font_family = Some(family.into());
        self
    }

    /// 指定字体族（宿主设置里的终端/编辑器字体优先于等宽回退）。
    pub fn font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    /// 运行期换字体族（设置保存后活更新）；`None` 回到正文字体。
    pub fn set_font_family(&mut self, family: Option<SharedString>, cx: &mut Context<Self>) {
        self.font_family = family;
        cx.notify();
    }

    /// 运行期改字号（设置保存后活更新）；同值不 notify，避免渲染期自激。
    /// 行高由字号派生，且 prepaint 每帧重排 shaped-line 缓存，无需额外失效。
    pub fn set_text_size(&mut self, size: f32, cx: &mut Context<Self>) {
        if self.text_size == Some(size) {
            return;
        }
        self.text_size = Some(size);
        cx.notify();
    }

    pub fn on_change(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    /// 提交模式：Enter 触发回调（不插换行），Shift+Enter 仍换行。
    /// 回调延迟到本效果周期结束执行，避免 TextArea 租约未释放时外层读取自身。
    pub fn on_submit(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(Rc::new(listener));
        self
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    /// 光标所在行/列（1 起）。宿主状态栏（编辑器 行 x，列 y）用。
    pub fn caret(&self) -> (usize, usize) {
        caret_line_col(&self.content, self.selected_range.start)
    }

    /// 程序化摆光标（加载文件后回到 1,1 口径）；越界夹到内容末尾。
    pub fn set_caret(&mut self, pos: usize, cx: &mut Context<Self>) {
        let pos = pos.min(self.content.len());
        self.selected_range = pos..pos;
        self.selection_reversed = false;
        self.preferred_col = None;
        cx.notify();
    }

    /// 可视高度改设（宿主按窗口尺寸铺满编辑区）；同值不 notify，避免渲染期自激。
    pub fn set_height(&mut self, height: Pixels, cx: &mut Context<Self>) {
        if self.height == height {
            return;
        }
        self.height = height;
        cx.notify();
    }

    /// 与内置滚动条共享的滚动句柄；offset.y 为负值，0 表示顶部。
    pub fn scroll_handle(&self) -> ScrollHandle {
        self.scroll_handle.clone()
    }

    /// 当前距顶部的滚动距离。
    pub fn scroll_offset(&self) -> Pixels {
        self.scroll_y
    }

    /// 内容 / 视口 / 最大滚动距离。
    pub fn scroll_metrics(&self) -> TextAreaScrollMetrics {
        let m = self.metrics();
        TextAreaScrollMetrics {
            content_height: m.line_height * self.line_count() as f32,
            viewport_height: f32::from(self.viewport_height(&m)),
            max_scroll: f32::from(self.max_scroll(&m)),
        }
    }

    /// 滚动到指定行顶部；行号从 0 起，越界夹到末行。
    pub fn scroll_to_line(&mut self, line: usize, cx: &mut Context<Self>) {
        let line = line.min(self.line_count().saturating_sub(1));
        let m = self.metrics();
        self.set_scroll_offset(px(m.line_height * line as f32), cx);
    }

    /// 设置距顶部的滚动距离；夹到 `[0, max_scroll]`，不移动光标。
    pub fn set_scroll_offset(&mut self, offset: Pixels, cx: &mut Context<Self>) {
        let m = self.metrics();
        let offset = offset.clamp(px(0.), self.max_scroll(&m));
        self.scroll_y = offset;
        self.scroll_anchor = Some(self.cursor_offset());
        self.scroll_handle.set_offset(point(px(0.), -offset));
        cx.notify();
    }

    pub fn set_text(
        &mut self,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_text_silent(text, cx);
        self.notify_change(window, cx);
    }

    /// 程序化写入但不触发 on_change：供包装控件（Sender 等）在自身 update
    /// 期间写入内部文本框，避免回调反向 update 包装实体导致双重租约 panic。
    pub fn set_text_silent(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = text.into();
        self.selected_range = self.content.len()..self.content.len();
        self.selection_reversed = false;
        self.marked_range = None;
        self.preferred_col = None;
        cx.notify();
    }

    fn notify_change(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.content, window, cx);
        }
    }

    // ------------------------------------------------------------------
    // 行 / 偏移工具
    // ------------------------------------------------------------------

    /// 总行数（含末尾虚拟行）
    fn line_count(&self) -> usize {
        self.content.bytes().filter(|b| *b == b'\n').count() + 1
    }

    /// 可视内容高度：扣除上下内边距与边框，并向下取整到整行，
    /// 避免末行被裁掉一截
    fn viewport_height(&self, m: &TextAreaMetrics) -> Pixels {
        let raw = self.height - px(m.pad_y * 2. + m.border * 2.);
        let lines = (f32::from(raw) / m.line_height).floor().max(1.);
        px(lines * m.line_height)
    }

    /// 最大滚动量（对齐到整行，保证首末行都完整可见）
    fn max_scroll(&self, m: &TextAreaMetrics) -> Pixels {
        let content_h = px(m.line_height * self.line_count() as f32);
        let raw = content_h - self.viewport_height(m);
        if raw <= px(0.) {
            return px(0.);
        }
        px((f32::from(raw) / m.line_height).floor() * m.line_height)
    }

    /// 滚轮解算（纯函数）：GPUI 向下 delta 为负，结果始终夹在 `[0, max_scroll]`。
    fn resolve_wheel_scroll(scroll: f32, delta: f32, max_scroll: f32) -> f32 {
        (scroll - delta).clamp(0., max_scroll)
    }

    /// 解算下一帧滚动位置（纯函数）：**只有光标偏移变化时**才把光标行带入视口。
    ///
    /// `anchor` 是上次对齐过的光标偏移。滚轮只改 `scroll`、不改光标 ⇒ 两者相等 ⇒
    /// 原样返回，用户的滚动位置得以跨帧保持。clamp 则始终执行（视口变小时仍需收敛）。
    /// 返回 `(新滚动位置, 新锚点)`。
    fn resolve_scroll(
        anchor: Option<usize>,
        cursor: usize,
        cursor_line: usize,
        line_height: f32,
        scroll: f32,
        viewport: f32,
        max_scroll: f32,
    ) -> (f32, usize) {
        let mut scroll = scroll;
        if anchor != Some(cursor) {
            let cursor_y = line_height * cursor_line as f32;
            if cursor_y < scroll {
                scroll = cursor_y;
            }
            if cursor_y + line_height > scroll + viewport {
                scroll = cursor_y + line_height - viewport;
            }
        }
        (scroll.clamp(0., max_scroll), cursor)
    }

    /// offset 所在行号（offset 非字符边界时向左对齐）
    fn line_of(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.content.len());
        while offset > 0 && !self.content.is_char_boundary(offset) {
            offset -= 1;
        }
        self.content[..offset]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
    }

    /// 行首偏移
    fn line_start(&self, line: usize) -> usize {
        if line == 0 {
            return 0;
        }
        self.content
            .match_indices('\n')
            .nth(line - 1)
            .map(|(i, _)| i + 1)
            .unwrap_or(self.content.len())
    }

    /// 行尾偏移（不含换行符）
    fn line_end(&self, line: usize) -> usize {
        let start = self.line_start(line);
        self.content[start..]
            .find('\n')
            .map(|i| start + i)
            .unwrap_or(self.content.len())
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
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

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.preferred_col = None;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    /// 上下移动：保持目标列
    fn move_vertical(&mut self, down: bool, select: bool, cx: &mut Context<Self>) {
        let cursor = self.cursor_offset();
        let line = self.line_of(cursor);
        let col = self.preferred_col.unwrap_or(cursor - self.line_start(line));

        let target_line = if down {
            if line + 1 >= self.line_count() {
                // 已在末行：移到文末
                let end = self.content.len();
                if select {
                    self.select_to(end, cx);
                } else {
                    self.move_to(end, cx);
                }
                return;
            }
            line + 1
        } else {
            if line == 0 {
                if select {
                    self.select_to(0, cx);
                } else {
                    self.move_to(0, cx);
                }
                return;
            }
            line - 1
        };

        let start = self.line_start(target_line);
        let end = self.line_end(target_line);
        // 保持字节列时可能落在多字节字符中间，向左对齐到字符边界
        let mut target = (start + col).min(end);
        while target > start && !self.content.is_char_boundary(target) {
            target -= 1;
        }
        self.preferred_col = Some(col);
        if select {
            self.select_to(target, cx);
        } else {
            let reversed = self.selection_reversed;
            self.selected_range = target..target;
            self.selection_reversed = reversed;
            cx.notify();
        }
    }

    // ------------------------------------------------------------------
    // 编辑动作（复用 text_input 的动作类型）
    // ------------------------------------------------------------------

    fn enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(on_submit) = self.on_submit.clone() {
            // 提交模式：不插换行；延迟回调，释放 TextArea 租约后再让外层读取/清空
            window.defer(cx, move |window, cx| on_submit(window, cx));
            return;
        }
        self.replace_text_in_range(None, "\n", window, cx);
    }

    fn newline(&mut self, _: &Newline, window: &mut Window, cx: &mut Context<Self>) {
        self.replace_text_in_range(None, "\n", window, cx);
    }

    /// Tab：插入缩进（text_area::bind_keys 的 tab 绑定）
    fn indent(&mut self, _: &Indent, window: &mut Window, cx: &mut Context<Self>) {
        let spaces = if self.tab_spaces == 0 {
            "\t".to_string()
        } else {
            " ".repeat(self.tab_spaces)
        };
        self.replace_text_in_range(None, &spaces, window, cx);
    }

    /// Shift+Tab：删去光标前最多 tab_spaces 个空格（或一个制表符）
    fn outdent(&mut self, _: &Outdent, window: &mut Window, cx: &mut Context<Self>) {
        let cursor = self.cursor_offset();
        let start = self.line_start(self.line_of(cursor));
        let before = &self.content[start..cursor];
        let remove = if before.ends_with('\t') {
            1
        } else {
            before
                .chars()
                .rev()
                .take(self.tab_spaces.max(1))
                .take_while(|c| *c == ' ')
                .count()
        };
        if remove == 0 {
            return;
        }
        self.selected_range = cursor - remove..cursor;
        self.selection_reversed = false;
        self.replace_text_in_range(None, "", window, cx);
    }

    fn up(&mut self, _: &Up, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_vertical(false, false, cx);
    }

    fn down(&mut self, _: &Down, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_vertical(true, false, cx);
    }

    fn select_up(&mut self, _: &SelectUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_vertical(false, true, cx);
    }

    fn select_down(&mut self, _: &SelectDown, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_vertical(true, true, cx);
    }

    fn select_home(&mut self, _: &SelectHome, _window: &mut Window, cx: &mut Context<Self>) {
        let start = self.line_start(self.line_of(self.cursor_offset()));
        self.select_to(start, cx);
    }

    fn select_end(&mut self, _: &SelectEnd, _window: &mut Window, cx: &mut Context<Self>) {
        let end = self.line_end(self.line_of(self.cursor_offset()));
        self.select_to(end, cx);
    }

    fn left(&mut self, _: &crate::text_input::Left, _window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            self.move_to(prev, cx);
        } else {
            let start = self.selected_range.start;
            self.move_to(start, cx);
        }
    }

    fn right(
        &mut self,
        _: &crate::text_input::Right,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            self.move_to(next, cx);
        } else {
            let end = self.selected_range.end;
            self.move_to(end, cx);
        }
    }

    fn select_left(
        &mut self,
        _: &crate::text_input::SelectLeft,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let prev = self.previous_boundary(self.cursor_offset());
        self.select_to(prev, cx);
    }

    fn select_right(
        &mut self,
        _: &crate::text_input::SelectRight,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let next = self.next_boundary(self.cursor_offset());
        self.select_to(next, cx);
    }

    fn select_all(
        &mut self,
        _: &crate::text_input::SelectAll,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_range = 0..self.content.len();
        self.selection_reversed = false;
        cx.notify();
    }

    fn home(&mut self, _: &crate::text_input::Home, _window: &mut Window, cx: &mut Context<Self>) {
        let start = self.line_start(self.line_of(self.cursor_offset()));
        self.move_to(start, cx);
    }

    fn end(&mut self, _: &crate::text_input::End, _window: &mut Window, cx: &mut Context<Self>) {
        let end = self.line_end(self.line_of(self.cursor_offset()));
        self.move_to(end, cx);
    }

    fn backspace(
        &mut self,
        _: &crate::text_input::Backspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            if self.cursor_offset() == prev {
                return;
            }
            self.select_to(prev, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn delete(
        &mut self,
        _: &crate::text_input::Delete,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if self.cursor_offset() == next {
                return;
            }
            self.select_to(next, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn paste(&mut self, _: &crate::text_input::Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text, window, cx);
        }
    }

    fn copy(&mut self, _: &crate::text_input::Copy, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &crate::text_input::Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    // ------------------------------------------------------------------
    // 鼠标
    // ------------------------------------------------------------------

    fn index_for_position(&self, position: Point<Pixels>) -> usize {
        let Some(bounds) = self.last_bounds.as_ref() else {
            return 0;
        };
        let m = self.metrics();
        let local_y = position.y - bounds.top() + self.scroll_y;
        let line = (f32::from(local_y) / m.line_height).floor().max(0.) as usize;
        let line = line.min(self.line_count() - 1);
        let start = self.line_start(line);
        let end = self.line_end(line);

        let local_x = position.x - bounds.left() - self.gutter;
        let shaped = self
            .last_lines
            .iter()
            .find(|(offset, _)| *offset == start)
            .map(|(_, shaped)| shaped);
        let Some(shaped) = shaped else {
            return start;
        };
        let index = shaped.closest_index_for_x(local_x);
        (start + index).min(end)
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        let index = self.index_for_position(event.position);
        if event.modifiers.shift {
            self.select_to(index, cx);
        } else {
            self.move_to(index, cx);
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _cx: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            let index = self.index_for_position(event.position);
            self.select_to(index, cx);
        }
    }

    /// 滚轮在内框内滚动内容（不传递给外层页面）
    fn on_scroll(&mut self, event: &ScrollEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let m = self.metrics();
        let delta = match event.delta {
            ScrollDelta::Pixels(point) => f32::from(point.y),
            ScrollDelta::Lines(point) => point.y * m.line_height,
        };
        if delta == 0. {
            return;
        }
        let max_scroll = self.max_scroll(&m);
        if max_scroll <= px(0.) {
            return;
        }
        let before = -self.scroll_handle.offset().y;
        let after = px(Self::resolve_wheel_scroll(
            f32::from(before),
            delta,
            f32::from(max_scroll),
        ));
        if after != before {
            self.scroll_handle.set_offset(point(px(0.), -after));
            self.scroll_y = after;
            self.scroll_anchor = Some(self.cursor_offset());
            // 编辑器内已消费滚轮：不再传递给外层页面
            cx.stop_propagation();
            cx.notify();
        }
    }

    // ------------------------------------------------------------------
    // UTF-16 换算（IME）
    // ------------------------------------------------------------------

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
}
impl EntityInputHandler for TextArea {
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
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.selection_reversed = false;
        self.marked_range.take();
        self.preferred_col = None;
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
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

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
        self.preferred_col = None;

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
        let range = self.range_from_utf16(&range_utf16);
        let m = self.metrics();
        let line = self.line_of(range.start);
        let line_start = self.line_start(line);
        let shaped = self
            .last_lines
            .iter()
            .find(|(offset, _)| *offset == line_start)
            .map(|(_, shaped)| shaped)?;

        let y = bounds.top() + px(m.line_height * line as f32) - self.scroll_y;
        Some(Bounds::from_corners(
            point(
                bounds.left() + self.gutter + shaped.x_for_index(range.start - line_start),
                y,
            ),
            point(
                bounds.left()
                    + self.gutter
                    + shaped.x_for_index(range.end.saturating_sub(line_start)),
                y + px(m.line_height),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let local = self.last_bounds?.localize(&point)?;
        let m = self.metrics();
        let line = (f32::from(local.y + self.scroll_y) / m.line_height)
            .floor()
            .max(0.) as usize;
        let line = line.min(self.line_count() - 1);
        let line_start = self.line_start(line);
        let shaped = self
            .last_lines
            .iter()
            .find(|(offset, _)| *offset == line_start)
            .map(|(_, shaped)| shaped)?;
        let index = line_start + shaped.closest_index_for_x(local.x - self.gutter);
        Some(self.offset_to_utf16(index.min(self.content.len())))
    }
}

/// 负责多行整形与绘制的内部元素
struct TextAreaElement {
    input: Entity<TextArea>,
    a11y: Semantics,
}

struct TextAreaPrepaint {
    lines: Vec<(usize, ShapedLine)>,
    gutter_lines: Vec<ShapedLine>,
    gutter: Pixels,
    current_line: Option<PaintQuad>,
    selection: Vec<PaintQuad>,
    cursor: Option<PaintQuad>,
}

impl Element for TextAreaElement {
    type RequestLayoutState = ();
    type PrepaintState = TextAreaPrepaint;

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
        let input = self.input.read(cx);
        let m = input.metrics();
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = px(m.line_height * input.line_count() as f32).into();
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
        let scroll_y = input.scroll_y;
        let m = input.metrics();
        let style = window.text_style();
        let font_size = px(m.font_size);
        let line_height = px(m.line_height);

        let show_placeholder = content.is_empty();
        let mut lines: Vec<(usize, ShapedLine)> = Vec::new();
        let mut gutter_lines: Vec<ShapedLine> = Vec::new();
        let mut selection = Vec::new();
        let mut cursor_quad = None;
        let mut current_line_quad = None;

        // 行号栏宽度：等宽数字宽度按字号估算
        let gutter = if input.line_numbers && !show_placeholder {
            let digits = input.line_count().to_string().len() as f32;
            px(10. + digits * f32::from(font_size) * 0.62 + 10.)
        } else {
            px(0.)
        };
        let text_left = bounds.left() + gutter;
        let cursor_line = input.line_of(cursor);

        let mut line_start = 0usize;
        let mut line_ix = 0usize;
        let raw_lines: Vec<&str> = if show_placeholder {
            vec![input.placeholder.as_ref()]
        } else {
            content.split('\n').collect()
        };

        for raw in raw_lines {
            let line_end = line_start + raw.len();
            let text_color = if show_placeholder {
                theme::text_placeholder()
            } else {
                style.color
            };
            let display: SharedString = raw.to_string().into();

            let run = TextRun {
                len: display.len(),
                font: style.font(),
                color: text_color,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            // IME 组合下划线（按行截取）；组合期间不做语法着色
            let runs = if let Some(marked) = input.marked_range.as_ref() {
                let m_start = marked.start.max(line_start);
                let m_end = marked.end.min(line_end);
                if m_start < m_end {
                    vec![
                        TextRun {
                            len: m_start - line_start,
                            ..run.clone()
                        },
                        TextRun {
                            len: m_end - m_start,
                            underline: Some(UnderlineStyle {
                                color: Some(run.color),
                                thickness: px(1.0),
                                wavy: false,
                            }),
                            ..run.clone()
                        },
                        TextRun {
                            len: line_end - m_end,
                            ..run
                        },
                    ]
                    .into_iter()
                    .filter(|r| r.len > 0)
                    .collect()
                } else {
                    vec![run]
                }
            } else if let Some(highlighter) = input.highlighter.as_ref()
                && !show_placeholder
            {
                let tokens = highlighter(raw);
                if tokens.is_empty() {
                    vec![run]
                } else {
                    tokens
                        .into_iter()
                        .map(|token| TextRun {
                            len: token.range.end - token.range.start,
                            font: if token.bold {
                                Font {
                                    weight: FontWeight::SEMIBOLD,
                                    ..style.font()
                                }
                            } else {
                                style.font()
                            },
                            color: token.color,
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        })
                        .collect()
                }
            } else {
                vec![run]
            };

            let shaped = window
                .text_system()
                .shape_line(display, font_size, &runs, None);

            let y = bounds.top() + line_height * line_ix as f32 - scroll_y;

            // 当前行背景
            if input.current_line && !show_placeholder && line_ix == cursor_line {
                current_line_quad = Some(fill(
                    Bounds::new(
                        point(bounds.left(), y),
                        size(bounds.size.width, line_height),
                    ),
                    theme::accent_soft(),
                ));
            }

            // 行号
            if gutter > px(0.) {
                let label = (line_ix + 1).to_string();
                let color = if line_ix == cursor_line {
                    theme::accent()
                } else {
                    theme::text_disabled()
                };
                let num_run = TextRun {
                    len: label.len(),
                    font: style.font(),
                    color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                gutter_lines.push(window.text_system().shape_line(
                    label.into(),
                    font_size,
                    &[num_run],
                    None,
                ));
            }

            // 选区（跨行逐段绘制）
            if !selected_range.is_empty() && !show_placeholder {
                let s = selected_range.start.max(line_start);
                let e = selected_range.end.min(line_end);
                if s < e || (selected_range.start <= line_start && selected_range.end > line_end) {
                    let x1 = shaped.x_for_index(s - line_start);
                    let mut x2 = shaped.x_for_index(e - line_start);
                    // 选区越过行尾（含换行符）时补一小段示意
                    if selected_range.end > line_end {
                        x2 += px(4.);
                    }
                    if x2 > x1 {
                        selection.push(fill(
                            Bounds::from_corners(
                                point(text_left + x1, y),
                                point(text_left + x2, y + line_height),
                            ),
                            theme::primary_bg(),
                        ));
                    }
                }
            }

            // 光标
            if selected_range.is_empty() && cursor >= line_start && cursor <= line_end {
                let x = shaped.x_for_index(cursor - line_start);
                cursor_quad = Some(fill(
                    Bounds::new(point(text_left + x, y), size(px(1.5), line_height)),
                    theme::accent(),
                ));
            }

            lines.push((line_start, shaped));
            line_start = line_end + 1;
            line_ix += 1;
        }

        TextAreaPrepaint {
            lines,
            gutter_lines,
            gutter,
            current_line: current_line_quad,
            selection,
            cursor: cursor_quad,
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
        let m = self.input.read(cx).metrics();
        let scroll_y = self.input.read(cx).scroll_y;
        let viewport = self.input.read(cx).viewport_height(&m);
        let line_height = px(m.line_height);
        let gutter = prepaint.gutter;
        // 内容视口：不含上下内边距，避免上一行渗到内边距里
        let content_bounds = Bounds::new(bounds.origin, size(bounds.size.width, viewport));
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(content_bounds, self.input.clone()),
            cx,
        );

        // 行号栏底色 + 右分隔线（先于文字与行号）
        if gutter > px(0.) {
            window.paint_quad(fill(
                Bounds::new(point(bounds.left(), bounds.top()), size(gutter, viewport)),
                theme::code_bg(),
            ));
        }

        // 当前行背景（覆盖到行号栏，形成整行高亮）
        if let Some(quad) = prepaint.current_line.take() {
            window.paint_quad(quad);
        }

        if gutter > px(0.) {
            window.paint_quad(fill(
                Bounds::new(
                    point(bounds.left() + gutter - px(1.), bounds.top()),
                    size(px(1.), viewport),
                ),
                theme::border(),
            ));
        }

        for quad in prepaint.selection.drain(..) {
            window.paint_quad(quad);
        }

        window.with_content_mask(
            Some(ContentMask {
                bounds: content_bounds,
            }),
            |window| {
                for (ix, (_, line)) in prepaint.lines.iter().enumerate() {
                    let y = bounds.top() + line_height * ix as f32 - scroll_y;
                    // 视口外的行跳过绘制
                    if y + line_height < bounds.top() || y > bounds.top() + viewport {
                        continue;
                    }
                    let _ = line.paint(
                        point(bounds.left() + gutter, y),
                        line_height,
                        gpui::TextAlign::default(),
                        None,
                        window,
                        cx,
                    );
                }

                // 行号（右对齐到距离分隔线 10px 处）
                for (ix, number) in prepaint.gutter_lines.iter().enumerate() {
                    let y = bounds.top() + line_height * ix as f32 - scroll_y;
                    if y + line_height < bounds.top() || y > bounds.top() + viewport {
                        continue;
                    }
                    let x = bounds.left() + gutter - px(10.) - number.width;
                    let _ = number.paint(
                        point(x, y),
                        line_height,
                        gpui::TextAlign::default(),
                        None,
                        window,
                        cx,
                    );
                }
            },
        );

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        let lines = std::mem::take(&mut prepaint.lines);
        self.input.update(cx, |input, _| {
            input.last_lines = lines;
            input.last_bounds = Some(content_bounds);
            input.gutter = gutter;
        });
    }
}

impl Render for TextArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let disabled = self.disabled;
        let a11y = self.semantics(format!("text-area-{:?}", cx.entity_id()).into());
        let (variant, status, size) = (self.variant, self.status, self.size);
        let style = theme::with(|t| state(variant, status, size, t, false, focused, disabled));
        let hover_style = theme::with(|t| state(variant, status, size, t, true, false, disabled));
        let m = self.metrics();
        let handle_offset = self.scroll_handle.offset().y;
        let cursor = self.cursor_offset();
        let (scroll, anchor) = Self::resolve_scroll(
            self.scroll_anchor,
            cursor,
            self.line_of(cursor),
            m.line_height,
            (-handle_offset).max(px(0.)).into(),
            self.viewport_height(&m).into(),
            self.max_scroll(&m).into(),
        );
        self.scroll_y = px(scroll);
        self.scroll_anchor = Some(anchor);
        if handle_offset != px(-scroll) {
            self.scroll_handle.set_offset(point(px(0.), px(-scroll)));
        }
        let viewport = self.viewport_height(&m);
        let max_scroll = self.max_scroll(&m);

        let mut root = div()
            .id(a11y.element_id())
            .key_context("TextArea")
            .flex()
            .flex_col()
            .w_full()
            .px(px(m.pad_x))
            .py(px(m.pad_y))
            .overflow_hidden()
            .line_height(px(m.line_height))
            .text_size(px(m.font_size))
            .text_color(style.fg);

        // antd 的 textarea 是 `height: auto; min-height: controlHeight`，
        // 所以外框 helper 给的高度在这里降级为 min-height，可视高度仍由控件自己决定。
        root = text_input::apply_frame(root, &style)
            .h(self.height)
            .min_h(px(m.min_height));
        root = text_input::hover_frame(root, &hover_style, focused);

        if disabled {
            root = root.cursor(CursorStyle::Arrow);
        } else {
            root = root
                .track_focus(&self.focus_handle(cx))
                .cursor(CursorStyle::IBeam)
                .on_action(cx.listener(Self::enter))
                .on_action(cx.listener(Self::newline))
                .on_action(cx.listener(Self::indent))
                .on_action(cx.listener(Self::outdent))
                .on_action(cx.listener(Self::up))
                .on_action(cx.listener(Self::down))
                .on_action(cx.listener(Self::select_up))
                .on_action(cx.listener(Self::select_down))
                .on_action(cx.listener(Self::select_home))
                .on_action(cx.listener(Self::select_end))
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

        if let Some(family) = self.font_family.clone() {
            root = root.font_family(family);
        }
        let handle = self.scroll_handle.clone();
        let entity_tag = format!("{:?}", cx.entity_id());
        let mut scroll_view = div()
            .id(ElementId::Name(
                format!("textarea-scroll-{entity_tag}").into(),
            ))
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .overflow_hidden();
        if !disabled {
            scroll_view = scroll_view.on_scroll(cx.listener(Self::on_scroll));
        }
        let mut row = div()
            .flex()
            .flex_row()
            .w_full()
            .h(viewport)
            .child(scroll_view.child(TextAreaElement {
                input: cx.entity(),
                a11y: a11y.clone(),
            }));
        if !disabled && max_scroll > px(0.) {
            row = row.child(Scrollbar::vertical_with_metrics(
                ElementId::Name(format!("textarea-scrollbar-{entity_tag}").into()),
                handle,
                viewport,
                max_scroll,
            ));
        }
        if !disabled && max_scroll > px(0.) {
            let entity = cx.entity();
            root = root
                .on_a11y_action(AccessibleAction::ScrollUp, {
                    let entity = entity.clone();
                    move |data, _, cx| {
                        let _ =
                            entity.update(cx, |input, cx| input.scroll_from_a11y(false, data, cx));
                    }
                })
                .on_a11y_action(AccessibleAction::ScrollDown, {
                    let entity = entity.clone();
                    move |data, _, cx| {
                        let _ =
                            entity.update(cx, |input, cx| input.scroll_from_a11y(true, data, cx));
                    }
                })
                .on_a11y_action(AccessibleAction::SetScrollOffset, {
                    let entity = entity.clone();
                    move |data, _, cx| {
                        let _ = entity.update(cx, |input, cx| input.set_scroll_from_a11y(data, cx));
                    }
                });
        }
        self.with_a11y(root.child(row), cx)
    }
}

impl Focusable for TextArea {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::TextArea;

    fn close(actual: f32, expected: f32) -> bool {
        (actual - expected).abs() < 1e-4
    }

    #[test]
    fn text_area_wheel_accumulates_and_survives_repeated_renders() {
        let mut scroll = 0.;
        for _ in 0..12 {
            let next = TextArea::resolve_wheel_scroll(scroll, -20., 400.);
            assert!(next > scroll);
            let (rendered, anchor) = TextArea::resolve_scroll(Some(0), 0, 0, 20., next, 100., 400.);
            assert!(close(rendered, next));
            assert_eq!(anchor, 0);
            scroll = rendered;
        }
        assert!(close(scroll, 240.));
    }

    #[test]
    fn text_area_wheel_clamps_at_both_ends() {
        assert!(close(TextArea::resolve_wheel_scroll(0., 20., 400.), 0.));
        assert!(close(
            TextArea::resolve_wheel_scroll(380., -40., 400.),
            400.
        ));
        assert!(close(TextArea::resolve_wheel_scroll(120., 20., 400.), 100.));
    }

    #[test]
    fn text_area_cursor_change_reveals_lower_line() {
        let (scroll, anchor) = TextArea::resolve_scroll(Some(0), 200, 10, 20., 0., 100., 400.);
        assert!(close(scroll, 120.));
        assert_eq!(anchor, 200);
    }

    #[test]
    fn text_area_cursor_change_reveals_upper_line() {
        let (scroll, anchor) = TextArea::resolve_scroll(Some(200), 20, 1, 20., 200., 100., 400.);
        assert!(close(scroll, 20.));
        assert_eq!(anchor, 20);
    }

    #[test]
    fn text_area_unchanged_cursor_never_yanks_manual_scroll() {
        for scroll in [0., 20., 120., 240., 400.] {
            let (rendered, anchor) =
                TextArea::resolve_scroll(Some(0), 0, 0, 20., scroll, 100., 400.);
            assert!(close(rendered, scroll));
            assert_eq!(anchor, 0);
        }
    }

    #[test]
    fn text_area_manual_scroll_then_cursor_move_reveals_new_position() {
        let manual = TextArea::resolve_wheel_scroll(0., -200., 400.);
        let (scroll, anchor) = TextArea::resolve_scroll(Some(0), 60, 3, 20., manual, 100., 400.);
        assert!(close(scroll, 60.));
        assert_eq!(anchor, 60);
    }

    #[test]
    fn text_area_non_overflowing_content_cannot_scroll() {
        assert!(close(TextArea::resolve_wheel_scroll(0., 20., 0.), 0.));
        assert!(close(TextArea::resolve_wheel_scroll(0., -20., 0.), 0.));
        let (scroll, anchor) = TextArea::resolve_scroll(Some(0), 0, 0, 20., 50., 100., 0.);
        assert!(close(scroll, 0.));
        assert_eq!(anchor, 0);
    }

    #[test]
    fn text_area_viewport_shrink_still_clamps_without_cursor_follow() {
        let (scroll, anchor) = TextArea::resolve_scroll(Some(0), 0, 0, 20., 300., 80., 200.);
        assert!(close(scroll, 200.));
        assert_eq!(anchor, 0);
    }

    #[test]
    fn text_area_initial_render_reveals_caret_at_content_end() {
        let (scroll, anchor) = TextArea::resolve_scroll(None, 240, 12, 20., 0., 100., 400.);
        assert!(close(scroll, 160.));
        assert_eq!(anchor, 240);
    }

    #[test]
    fn text_area_resolve_scroll_is_pure_for_repeated_inputs() {
        for _ in 0..3 {
            assert_eq!(
                TextArea::resolve_scroll(Some(0), 0, 0, 20., 200., 100., 400.),
                TextArea::resolve_scroll(Some(0), 0, 0, 20., 200., 100., 400.)
            );
        }
        assert_eq!(TextArea::resolve_wheel_scroll(20., 5., 100.), 15.);
    }
}
