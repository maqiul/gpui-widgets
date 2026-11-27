//! 排版控件：标题 / 段落 / 行内文本（对应 antd Typography）。

use std::rc::Rc;
use std::time::Duration;

use gpui::{
    App, ClipboardItem, Context, CursorStyle, Div, ElementId, Entity, Focusable, FontWeight, Hsla,
    InteractiveElement, IntoElement, ParentElement, Render, RenderOnce, Role, SharedString,
    Stateful, StatefulInteractiveElement, Styled, Window, div, prelude::*, px, rgba,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    a11y::{self, Semantics},
    input::text_input::TextInput,
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

/// 按 Unicode 字素簇截断文本，并在实际发生截断时追加省略号。
///
/// `max_chars` 计算的是省略号之前的字素簇数量；传 0 时只返回 `symbol`。
pub fn truncate_text(text: &str, max_chars: usize, symbol: &str) -> String {
    let mut graphemes = UnicodeSegmentation::graphemes(text, true);
    let mut out = String::new();
    for grapheme in graphemes.by_ref().take(max_chars) {
        out.push_str(grapheme);
    }
    if graphemes.next().is_some() {
        out.push_str(symbol);
    }
    out
}

/// 受控编辑配置。
#[derive(Clone)]
pub struct EditableConfig {
    enabled: bool,
    editing: bool,
    on_editing_change: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    on_change: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    on_start: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_cancel: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_end: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    max_length: Option<usize>,
}

impl Default for EditableConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            editing: false,
            on_editing_change: None,
            on_change: None,
            on_start: None,
            on_cancel: None,
            on_end: None,
            max_length: None,
        }
    }
}

impl EditableConfig {
    pub fn editing(mut self, editing: bool) -> Self {
        self.editing = editing;
        self
    }

    pub fn on_editing_change(
        mut self,
        listener: impl Fn(&bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_editing_change = Some(Rc::new(listener));
        self
    }

    pub fn on_change(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }

    pub fn on_start(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_start = Some(Rc::new(listener));
        self
    }

    pub fn on_cancel(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(listener));
        self
    }

    pub fn on_end(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_end = Some(Rc::new(listener));
        self
    }

    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }
}

impl From<bool> for EditableConfig {
    fn from(enabled: bool) -> Self {
        Self {
            enabled,
            ..Default::default()
        }
    }
}

/// 复制配置。
#[derive(Clone)]
pub struct CopyConfig {
    enabled: bool,
    text: Option<SharedString>,
    tooltip: Option<SharedString>,
    on_copy: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl Default for CopyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            text: None,
            tooltip: None,
            on_copy: None,
        }
    }
}

impl CopyConfig {
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn on_copy(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_copy = Some(Rc::new(listener));
        self
    }
}

impl From<bool> for CopyConfig {
    fn from(enabled: bool) -> Self {
        Self {
            enabled,
            ..Default::default()
        }
    }
}

/// 截断 / 展开配置。
#[derive(Clone)]
pub struct EllipsisConfig {
    enabled: bool,
    max_chars: Option<usize>,
    expandable: bool,
    symbol: Option<SharedString>,
    expanded: bool,
    on_expand: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    tooltip: bool,
}

impl Default for EllipsisConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_chars: None,
            expandable: false,
            symbol: None,
            expanded: false,
            on_expand: None,
            tooltip: true,
        }
    }
}

impl EllipsisConfig {
    pub fn max_chars(mut self, max_chars: usize) -> Self {
        self.max_chars = Some(max_chars);
        self
    }

    pub fn expandable(mut self, expandable: bool) -> Self {
        self.expandable = expandable;
        self
    }

    pub fn symbol(mut self, symbol: impl Into<SharedString>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn on_expand(mut self, listener: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_expand = Some(Rc::new(listener));
        self
    }

    pub fn tooltip(mut self, tooltip: bool) -> Self {
        self.tooltip = tooltip;
        self
    }
}

impl From<bool> for EllipsisConfig {
    fn from(enabled: bool) -> Self {
        Self {
            enabled,
            tooltip: true,
            ..Default::default()
        }
    }
}

#[derive(Clone, Default)]
struct TypographyOptions {
    id: Option<SharedString>,
    title: Option<SharedString>,
    editable: Option<EditableConfig>,
    copyable: Option<CopyConfig>,
    ellipsis: Option<EllipsisConfig>,
}

#[derive(Clone, Copy, Default)]
struct TextModifiers {
    strong: bool,
    italic: bool,
    underline: bool,
    delete: bool,
    code: bool,
    mark: bool,
}

impl TextModifiers {
    fn role(self, fallback: Role) -> Role {
        if self.delete {
            Role::ContentDeletion
        } else if self.code {
            Role::Code
        } else if self.mark {
            Role::Mark
        } else if self.strong {
            Role::Strong
        } else {
            fallback
        }
    }

    fn apply(self, mut element: Stateful<Div>, body_size: f32) -> Stateful<Div> {
        if self.strong {
            element = element.font_weight(FontWeight(TYPOGRAPHY_STRONG_WEIGHT));
        }
        if self.italic {
            element = element.italic();
        }
        if self.underline {
            element = element.underline();
        }
        if self.delete {
            element = element.line_through();
        }
        if self.code {
            let code_size = body_size * TYPOGRAPHY_CODE_SIZE_RATIO;
            element = element
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
            element = element.bg(rgba(TYPOGRAPHY_MARK_BG));
        }
        element
    }
}

#[derive(Default)]
struct TypographyInteractionState {
    input: Option<Entity<TextInput>>,
    draft: SharedString,
    was_editing: bool,
    input_font_size: Option<f32>,
    input_max_length: Option<usize>,
    copied: bool,
    copy_seq: u64,
}

struct TypographyTitleView {
    text: SharedString,
}

impl Render for TypographyTitleView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        crate::overlay::tooltip::tooltip_bubble(self.text.clone())
    }
}

fn with_tooltip(element: Stateful<Div>, text: Option<SharedString>) -> Stateful<Div> {
    match text {
        Some(text) => element.tooltip(move |_, cx| {
            cx.new(|_| TypographyTitleView { text: text.clone() })
                .into()
        }),
        None => element,
    }
}

fn typography_action(
    id: SharedString,
    label: SharedString,
    glyph: &'static str,
    tooltip: Option<SharedString>,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let action = div()
        .id(ElementId::Name(id.clone().into()))
        .w(px(20.))
        .h(px(20.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .cursor(CursorStyle::PointingHand)
        .text_size(px(12.))
        .text_color(theme::text_tertiary())
        .hover(|style| {
            style
                .bg(theme::control_item_bg_hover())
                .text_color(theme::text())
        })
        .child(glyph);
    let action = Semantics::new(Role::Button, label)
        .id(id)
        .focusable(true)
        .apply(action);
    let action = with_tooltip(action, tooltip);
    a11y::click_void(action, on_click)
}

fn interaction_state(
    window: &mut Window,
    cx: &mut App,
    id: &SharedString,
) -> Entity<TypographyInteractionState> {
    window.use_keyed_state(
        ElementId::Name(format!("typography-state-{id}").into()),
        cx,
        |_, _| TypographyInteractionState::default(),
    )
}

fn ensure_edit_input(
    state: &Entity<TypographyInteractionState>,
    id: &SharedString,
    text: &SharedString,
    editing: bool,
    font_size: f32,
    max_length: Option<usize>,
    cx: &mut App,
) -> (Option<Entity<TextInput>>, bool) {
    if !editing {
        let was_editing = state.read(cx).was_editing;
        if was_editing {
            state.update(cx, |state, _| {
                state.was_editing = false;
                state.input = None;
            });
        }
        return (None, false);
    }

    let (was_editing, current_font_size, current_max_length, has_input, draft) = {
        let state = state.read(cx);
        (
            state.was_editing,
            state.input_font_size,
            state.input_max_length,
            state.input.is_some(),
            state.draft.clone(),
        )
    };
    if was_editing
        && has_input
        && current_font_size == Some(font_size)
        && current_max_length == max_length
    {
        return (state.read(cx).input.clone(), false);
    }

    let draft = if was_editing { draft } else { text.clone() };
    let input_id = format!("{id}-input");
    let weak = state.downgrade();
    let input = cx.new(move |cx| {
        let mut input = TextInput::new(cx).bare(true).font_size(font_size);
        input.set_a11y_semantics(Semantics::new(Role::TextInput, "编辑文本").id(input_id));
        if let Some(max_length) = max_length {
            input = input.max_length(max_length);
        }
        input.on_change(move |text, _, cx| {
            if let Some(state) = weak.upgrade() {
                state.update(cx, |state, cx| {
                    state.draft = text.into();
                    cx.notify();
                });
            }
        })
    });
    input.update(cx, |input, cx| {
        input.set_text_silent(draft.clone(), cx);
    });
    state.update(cx, |state, _| {
        state.input = Some(input.clone());
        state.draft = draft;
        state.was_editing = true;
        state.input_font_size = Some(font_size);
        state.input_max_length = max_length;
    });
    (Some(input), !was_editing)
}

#[allow(clippy::too_many_arguments)]
fn finish_typography(
    mut root: Stateful<Div>,
    text_element: Stateful<Div>,
    options: &TypographyOptions,
    id: &SharedString,
    state: Entity<TypographyInteractionState>,
    input: Option<Entity<TextInput>>,
    full_text: &SharedString,
    truncated: bool,
    cx: &mut App,
) -> Stateful<Div> {
    let editable = options.editable.as_ref().filter(|config| config.enabled);
    let copyable = options.copyable.as_ref().filter(|config| config.enabled);
    let ellipsis = options.ellipsis.as_ref().filter(|config| config.enabled);
    let editing = editable.is_some_and(|config| config.editing);
    let show_edit = editable.is_some() && !editing;
    let show_copy = copyable.is_some();
    let show_expand =
        ellipsis.is_some_and(|config| config.expandable && (truncated || config.expanded));
    let inline_actions = editing || show_edit || show_copy || show_expand;
    if inline_actions {
        root = root
            .flex()
            .flex_row()
            .items_center()
            .gap(px(theme::with(|tokens| tokens.margin_xs)));
    }

    if editing {
        if let Some(input) = input {
            let key_input = input.clone();
            let on_change = editable.and_then(|config| config.on_change.clone());
            let on_editing_change = editable.and_then(|config| config.on_editing_change.clone());
            let on_cancel = editable.and_then(|config| config.on_cancel.clone());
            let on_end = editable.and_then(|config| config.on_end.clone());
            root = root
                .child(div().flex_1().min_w_0().child(input))
                .on_key_down(move |event, window, cx| {
                    if event.keystroke.key == "enter" {
                        cx.stop_propagation();
                        let value = key_input.read(cx).text().to_string();
                        if let Some(listener) = &on_change {
                            listener(&value, window, cx);
                        }
                        if let Some(listener) = &on_editing_change {
                            listener(&false, window, cx);
                        }
                        if let Some(listener) = &on_end {
                            listener(window, cx);
                        }
                    } else if event.keystroke.key == "escape" {
                        cx.stop_propagation();
                        if let Some(listener) = &on_editing_change {
                            listener(&false, window, cx);
                        }
                        if let Some(listener) = &on_cancel {
                            listener(window, cx);
                        }
                        if let Some(listener) = &on_end {
                            listener(window, cx);
                        }
                    }
                });
        }
    } else {
        root = root.child(text_element);
    }

    if show_edit {
        let on_start = editable.and_then(|config| config.on_start.clone());
        let on_editing_change = editable.and_then(|config| config.on_editing_change.clone());
        let action = typography_action(
            format!("{id}-edit").into(),
            "编辑".into(),
            "✎",
            Some("编辑".into()),
            move |window, cx| {
                if let Some(listener) = &on_start {
                    listener(window, cx);
                }
                if let Some(listener) = &on_editing_change {
                    listener(&true, window, cx);
                }
            },
        );
        root = root.child(action);
    }

    if let Some(copyable) = copyable {
        let copied = state.read(cx).copied;
        let copy_text = copyable.text.clone().unwrap_or_else(|| full_text.clone());
        let on_copy = copyable.on_copy.clone();
        let tooltip = copyable.tooltip.clone().unwrap_or_else(|| "复制".into());
        let weak = state.downgrade();
        let action = typography_action(
            format!("{id}-copy").into(),
            if copied { "已复制" } else { "复制" }.into(),
            if copied { "✓" } else { "⧉" },
            Some(tooltip),
            move |window, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(copy_text.to_string()));
                let seq = if let Some(state) = weak.upgrade() {
                    state.update(cx, |state, cx| {
                        state.copy_seq += 1;
                        state.copied = true;
                        cx.notify();
                        state.copy_seq
                    })
                } else {
                    0
                };
                if let Some(listener) = &on_copy {
                    listener(window, cx);
                }
                let weak = weak.clone();
                let _ = cx.spawn(async move |cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(1200))
                        .await;
                    let _ = weak.update(cx, |state, cx| {
                        if state.copy_seq == seq {
                            state.copied = false;
                            cx.notify();
                        }
                    });
                });
            },
        );
        root = root.child(action);
    }

    if let Some(ellipsis) =
        ellipsis.filter(|config| config.expandable && (truncated || config.expanded))
    {
        let expanded = ellipsis.expanded;
        let on_expand = ellipsis.on_expand.clone();
        let action = typography_action(
            format!("{id}-expand").into(),
            if expanded { "收起" } else { "展开" }.into(),
            if expanded { "−" } else { "+" },
            Some("展开或收起".into()),
            move |window, cx| {
                if let Some(listener) = &on_expand {
                    listener(&!expanded, window, cx);
                }
            },
        );
        root = root.child(action);
    }

    let tooltip = options.title.clone().or_else(|| {
        if truncated && ellipsis.is_some_and(|config| config.tooltip) {
            Some(full_text.clone())
        } else {
            None
        }
    });
    with_tooltip(root, tooltip)
}

fn display_text(text: &SharedString, ellipsis: Option<&EllipsisConfig>) -> (SharedString, bool) {
    let Some(config) = ellipsis.filter(|config| config.enabled) else {
        return (text.clone(), false);
    };
    if config.expanded {
        return (text.clone(), false);
    }
    let Some(max_chars) = config.max_chars else {
        return (text.clone(), false);
    };
    let symbol = config.symbol.as_deref().unwrap_or("…");
    let display = truncate_text(text, max_chars, symbol);
    let truncated = display != *text;
    (display.into(), truncated)
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
    options: TypographyOptions,
}

impl Title {
    pub fn new(level: u8, text: impl Into<SharedString>) -> Self {
        Self {
            level: level.clamp(1, 5),
            text: text.into(),
            kind: TypographyType::Default,
            options: TypographyOptions::default(),
        }
    }

    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.options.id = Some(id.into());
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.options.title = Some(title.into());
        self
    }

    pub fn editable(mut self, config: impl Into<EditableConfig>) -> Self {
        self.options.editable = Some(config.into());
        self
    }

    pub fn copyable(mut self, config: impl Into<CopyConfig>) -> Self {
        self.options.copyable = Some(config.into());
        self
    }

    pub fn ellipsis(mut self, config: impl Into<EllipsisConfig>) -> Self {
        self.options.ellipsis = Some(config.into());
        self
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self
            .options
            .id
            .clone()
            .unwrap_or_else(|| format!("heading-{}-{}", self.level, self.text).into());
        let (display, truncated) = display_text(&self.text, self.options.ellipsis.as_ref());
        let (color, m) = theme::with(|tokens| {
            let color = if self.kind == TypographyType::Default {
                tokens.color_text_heading
            } else {
                typography_color(tokens, self.kind)
            };
            (color, TypographyMetric::resolve(tokens))
        });
        let ix = (self.level - 1) as usize;
        let font_size = m.title_size[ix];
        let semantics = Semantics::new(Role::Heading, display.clone())
            .id(id.clone())
            .level(self.level as usize);
        let mut text_element = semantics.apply(
            div()
                .id(ElementId::Name(SharedString::from(format!("{id}-text"))))
                .font_weight(FontWeight(TYPOGRAPHY_TITLE_WEIGHT))
                .text_size(px(font_size))
                .line_height(px(m.title_line_height[ix]))
                .text_color(color)
                .child(display),
        );
        if self
            .options
            .ellipsis
            .as_ref()
            .is_some_and(|config| config.enabled && !config.expanded)
        {
            text_element = text_element.min_w_0().whitespace_nowrap().text_ellipsis();
        }
        let root = div()
            .id(ElementId::Name(SharedString::from(format!("{id}-root"))))
            .mb(px(m.title_margin_bottom));
        let state = interaction_state(window, cx, &id);
        let editable = self
            .options
            .editable
            .as_ref()
            .filter(|config| config.enabled);
        let editing = editable.is_some_and(|config| config.editing);
        let max_length = editable.and_then(|config| config.max_length);
        let (input, started) =
            ensure_edit_input(&state, &id, &self.text, editing, font_size, max_length, cx);
        if started {
            if let Some(input) = input.clone() {
                window.defer(cx, move |window, cx| {
                    input.read(cx).focus_handle(cx).focus(window, cx);
                });
            }
        }
        finish_typography(
            root,
            text_element,
            &self.options,
            &id,
            state,
            input,
            &self.text,
            truncated,
            cx,
        )
    }
}

/// 段落文本
#[derive(IntoElement)]
pub struct Paragraph {
    text: SharedString,
    kind: TypographyType,
    modifiers: TextModifiers,
    options: TypographyOptions,
}

impl Paragraph {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            kind: TypographyType::Default,
            modifiers: TextModifiers::default(),
            options: TypographyOptions::default(),
        }
    }

    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.options.id = Some(id.into());
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.options.title = Some(title.into());
        self
    }

    pub fn editable(mut self, config: impl Into<EditableConfig>) -> Self {
        self.options.editable = Some(config.into());
        self
    }

    pub fn copyable(mut self, config: impl Into<CopyConfig>) -> Self {
        self.options.copyable = Some(config.into());
        self
    }

    pub fn ellipsis(mut self, config: impl Into<EllipsisConfig>) -> Self {
        self.options.ellipsis = Some(config.into());
        self
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
        self.modifiers.strong = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.modifiers.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.modifiers.underline = true;
        self
    }

    pub fn delete(mut self) -> Self {
        self.modifiers.delete = true;
        self
    }

    pub fn code(mut self) -> Self {
        self.modifiers.code = true;
        self
    }

    pub fn mark(mut self) -> Self {
        self.modifiers.mark = true;
        self
    }
}

impl RenderOnce for Paragraph {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self
            .options
            .id
            .clone()
            .unwrap_or_else(|| format!("paragraph-{}", self.text).into());
        let (display, truncated) = display_text(&self.text, self.options.ellipsis.as_ref());
        let (color, m) = theme::with(|tokens| {
            (
                typography_color(tokens, self.kind),
                TypographyMetric::resolve(tokens),
            )
        });
        let semantics = Semantics::new(Role::Paragraph, display.clone()).id(id.clone());
        let mut text_element = semantics.apply(
            div()
                .id(ElementId::Name(SharedString::from(format!("{id}-text"))))
                .text_size(px(m.body_size))
                .line_height(px(m.body_line_height))
                .text_color(color)
                .child(display),
        );
        text_element = self.modifiers.apply(text_element, m.body_size);
        if self
            .options
            .ellipsis
            .as_ref()
            .is_some_and(|config| config.enabled && !config.expanded)
        {
            text_element = text_element.min_w_0().whitespace_nowrap().text_ellipsis();
        }
        let root = div().id(ElementId::Name(SharedString::from(format!("{id}-root"))));
        let state = interaction_state(window, cx, &id);
        let editable = self
            .options
            .editable
            .as_ref()
            .filter(|config| config.enabled);
        let editing = editable.is_some_and(|config| config.editing);
        let max_length = editable.and_then(|config| config.max_length);
        let (input, started) = ensure_edit_input(
            &state,
            &id,
            &self.text,
            editing,
            m.body_size,
            max_length,
            cx,
        );
        if started {
            if let Some(input) = input.clone() {
                window.defer(cx, move |window, cx| {
                    input.read(cx).focus_handle(cx).focus(window, cx);
                });
            }
        }
        finish_typography(
            root,
            text_element,
            &self.options,
            &id,
            state,
            input,
            &self.text,
            truncated,
            cx,
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
    modifiers: TextModifiers,
    options: TypographyOptions,
}

impl Text {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            kind: TypographyType::Default,
            modifiers: TextModifiers::default(),
            options: TypographyOptions::default(),
        }
    }

    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.options.id = Some(id.into());
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.options.title = Some(title.into());
        self
    }

    pub fn editable(mut self, config: impl Into<EditableConfig>) -> Self {
        self.options.editable = Some(config.into());
        self
    }

    pub fn copyable(mut self, config: impl Into<CopyConfig>) -> Self {
        self.options.copyable = Some(config.into());
        self
    }

    pub fn ellipsis(mut self, config: impl Into<EllipsisConfig>) -> Self {
        self.options.ellipsis = Some(config.into());
        self
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
        self.modifiers.strong = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.modifiers.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.modifiers.underline = true;
        self
    }

    /// 删除线
    pub fn delete(mut self) -> Self {
        self.modifiers.delete = true;
        self
    }

    /// 行内代码样式
    pub fn code(mut self) -> Self {
        self.modifiers.code = true;
        self
    }

    /// 高亮标记（黄色底）
    pub fn mark(mut self) -> Self {
        self.modifiers.mark = true;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self
            .options
            .id
            .clone()
            .unwrap_or_else(|| format!("text-{}-{:?}", self.text, self.kind).into());
        let (display, truncated) = display_text(&self.text, self.options.ellipsis.as_ref());
        let (color, m) = theme::with(|tokens| {
            (
                typography_color(tokens, self.kind),
                TypographyMetric::resolve(tokens),
            )
        });
        let role = self.modifiers.role(Role::Label);
        let semantics = Semantics::new(role, display.clone()).id(id.clone());
        let mut text_element = semantics.apply(
            div()
                .id(ElementId::Name(SharedString::from(format!("{id}-text"))))
                .text_size(px(m.body_size))
                .line_height(px(m.body_line_height))
                .text_color(color)
                .child(display),
        );
        text_element = self.modifiers.apply(text_element, m.body_size);
        if self
            .options
            .ellipsis
            .as_ref()
            .is_some_and(|config| config.enabled && !config.expanded)
        {
            text_element = text_element.min_w_0().whitespace_nowrap().text_ellipsis();
        }
        let root = div().id(ElementId::Name(SharedString::from(format!("{id}-root"))));
        let state = interaction_state(window, cx, &id);
        let editable = self
            .options
            .editable
            .as_ref()
            .filter(|config| config.enabled);
        let editing = editable.is_some_and(|config| config.editing);
        let max_length = editable.and_then(|config| config.max_length);
        let (input, started) = ensure_edit_input(
            &state,
            &id,
            &self.text,
            editing,
            m.body_size,
            max_length,
            cx,
        );
        if started {
            if let Some(input) = input.clone() {
                window.defer(cx, move |window, cx| {
                    input.read(cx).focus_handle(cx).focus(window, cx);
                });
            }
        }
        finish_typography(
            root,
            text_element,
            &self.options,
            &id,
            state,
            input,
            &self.text,
            truncated,
            cx,
        )
    }
}
