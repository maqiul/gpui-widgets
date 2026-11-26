//! 终端控件：VT/ANSI 解析 + 字符网格 + 回滚缓冲 + 选区复制 + 搜索 + 输入回调。
//!
//! 面向 SSH / 本地 shell 等场景的纯 GPUI 终端视图（无第三方终端依赖）：
//! - **VT/ANSI 状态机**：Ground / Escape / CSI / OSC / DCS，支持 SGR 16/256/真彩、
//!   光标定位、擦除、插入/删除、滚动区域、OSC 标题与超链接（OSC 8）。
//! - **字符网格**：等宽字体按 `col * cell_w` 定位，色段批量整形保证网格对齐。
//! - **回滚缓冲**：`history`（上限可配）+ `screen`（固定 rows），滚轮 / Shift+PgUp 翻阅；
//!   resize 按软/硬换行重排内容，并同步映射光标、选区与搜索。
//! - **选区复制**：鼠标拖拽选区，Ctrl+C（有选区时）/ Ctrl+Shift+C 复制，Ctrl+V 粘贴。
//! - **搜索**：`set_search` 高亮全部匹配，`search_next/prev` 跳转并自动滚动到可见。
//! - **输入回调**：键盘转字节序列经 `on_input` 投递给调用方（PTY / SSH 通道）。
//!
//! 控件本身是 `Entity`，`cx.new(|cx| Terminal::new(cx))` 创建后作为子视图渲染。
//! 键盘走原始 `on_key_down`（无需 `bind_keys`）；`on_input` / `on_title` 均经
//! `window.defer` 延迟触发，避免在控件租约内反向 `update` 自身导致双重租约 panic。
//!
//! ```ignore
//! let term = cx.new(|cx| Terminal::new(cx).cols(100).rows(28)
//!     .on_input(|bytes, _window, _cx| { /* 写入 PTY / SSH 通道 */ }));
//! // 收到远端输出：term.update(cx, |t, cx| t.write(data, cx));
//! // render 中：div().child(term.clone())
//! ```

use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, DispatchPhase, Element, ElementId,
    ElementInputHandler, Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId,
    Hitbox, HitboxBehavior, Hsla, InspectorElementId, IntoElement, KeyDownEvent, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, ScrollEvent,
    ScrollHandle, StrikethroughStyle, Style, TextRun, UTF16Selection, UnderlineStyle, Window, div,
    fill, point, prelude::*, px, relative, rgb, size,
};

use crate::a11y;
use crate::theme;
use crate::theme::metric;

const FONT_SIZE: f32 = 14.;
const CELL_H: f32 = 18.;
const DEFAULT_COLS: usize = 80;
const DEFAULT_ROWS: usize = 24;
const DEFAULT_HISTORY: usize = 1000;
/// 光标闪烁半周期（xterm 默认口径）。
const CURSOR_BLINK_MS: u64 = 530;

/// 绝对坐标：(合并缓冲行号, 列)。合并缓冲 = history ++ screen。
type Pos = (usize, usize);

/// IME 预编辑只属于输入层，不写入终端网格；选中范围统一使用 UTF-16 偏移。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ImeComposition {
    text: String,
    selected_range_utf16: Range<usize>,
}

impl ImeComposition {
    fn new(text: &str, selected_range_utf16: Option<Range<usize>>) -> Option<Self> {
        if text.is_empty() {
            return None;
        }
        let len = terminal_ime_utf16_len(text);
        let selected = selected_range_utf16.unwrap_or_else(|| len..len);
        Some(Self {
            text: text.to_owned(),
            selected_range_utf16: terminal_ime_clamp_range(text, selected),
        })
    }

    fn selected_text_range(&self) -> UTF16Selection {
        UTF16Selection {
            range: self.selected_range_utf16.clone(),
            reversed: false,
        }
    }

    fn marked_text_range(&self) -> Range<usize> {
        0..terminal_ime_utf16_len(&self.text)
    }

    fn text_for_range(
        &self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
    ) -> String {
        let range = terminal_ime_clamp_range(&self.text, range_utf16);
        actual_range.replace(range.clone());
        terminal_ime_text_for_range(&self.text, range)
    }
}

fn terminal_ime_utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// 夹到预编辑文本范围并归一化，避免平台传入反向 range 导致切片 panic。
fn terminal_ime_clamp_range(text: &str, range: Range<usize>) -> Range<usize> {
    let len = terminal_ime_utf16_len(text);
    let start = range.start.min(len);
    let end = range.end.min(len);
    if start <= end { start..end } else { end..start }
}

/// UTF-16 偏移 → UTF-8 字节偏移；落在代理对内部时回到该字符起点。
fn terminal_ime_utf16_to_byte(text: &str, offset: usize) -> usize {
    let target = offset.min(terminal_ime_utf16_len(text));
    let mut utf16_offset = 0;
    for (byte_offset, ch) in text.char_indices() {
        if target <= utf16_offset {
            return byte_offset;
        }
        utf16_offset += ch.len_utf16();
        if target < utf16_offset {
            return byte_offset;
        }
    }
    text.len()
}

fn terminal_ime_text_for_range(text: &str, range_utf16: Range<usize>) -> String {
    let range = terminal_ime_clamp_range(text, range_utf16);
    let start = terminal_ime_utf16_to_byte(text, range.start);
    let end = terminal_ime_utf16_to_byte(text, range.end);
    text.get(start..end).unwrap_or_default().to_owned()
}

/// 预编辑按终端网格宽度定位；宽字符占两格，其余字符占一格。
fn terminal_ime_columns_for_utf16(text: &str, offset: usize) -> usize {
    let byte_offset = terminal_ime_utf16_to_byte(text, offset);
    text[..byte_offset]
        .chars()
        .map(|ch| if is_wide(ch) { 2 } else { 1 })
        .sum()
}

fn terminal_ime_utf16_for_column(text: &str, column: usize) -> usize {
    let mut current_column = 0;
    let mut utf16_offset = 0;
    for ch in text.chars() {
        let width = if is_wide(ch) { 2 } else { 1 };
        if column < current_column + width {
            return utf16_offset;
        }
        current_column += width;
        utf16_offset += ch.len_utf16();
    }
    utf16_offset
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TermColor {
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// 当前画笔（SGR 属性）。
#[derive(Clone, Copy, PartialEq, Eq)]
struct Pen {
    fg: TermColor,
    bg: TermColor,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    inverse: bool,
    hidden: bool,
    strike: bool,
}

impl Default for Pen {
    fn default() -> Self {
        Self {
            fg: TermColor::Default,
            bg: TermColor::Default,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            inverse: false,
            hidden: false,
            strike: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Cell {
    ch: char,
    pen: Pen,
    link: Option<u32>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            pen: Pen::default(),
            link: None,
        }
    }
}

#[derive(Clone)]
struct Line {
    cells: Vec<Cell>,
    /// 本行是上一行因自动换行产生的软续行；false 表示显式硬换行起点。
    wrapped: bool,
}

impl Line {
    fn blank(cols: usize) -> Self {
        Self {
            cells: vec![Cell::default(); cols],
            wrapped: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ParseState {
    Ground,
    Escape,
    Csi,
    Osc,
    Dcs,
    Charset,
}

fn norm(a: Pos, b: Pos) -> (Pos, Pos) {
    if a <= b { (a, b) } else { (b, a) }
}

/// ANSI 索引色 → Hsla（0~15 取主题色板，16~231 为 6×6×6 色立方，232~255 灰阶）。
fn ansi_index(i: u8) -> Hsla {
    let pal = theme::terminal_ansi();
    if i < 16 {
        pal[i as usize]
    } else if i < 232 {
        let n = i - 16;
        let lv = [0u32, 95, 135, 175, 215, 255];
        let r = lv[(n / 36) as usize];
        let g = lv[((n % 36) / 6) as usize];
        let b = lv[(n % 6) as usize];
        rgb((r << 16) | (g << 8) | b).into()
    } else {
        let v = 8u32 + (i as u32 - 232) * 10;
        rgb((v << 16) | (v << 8) | v).into()
    }
}

fn resolve_fg(c: TermColor, bold: bool) -> Hsla {
    match c {
        TermColor::Default => theme::terminal_fg(),
        TermColor::Indexed(i) => {
            if bold && i < 8 {
                ansi_index(i + 8)
            } else {
                ansi_index(i)
            }
        }
        TermColor::Rgb(r, g, b) => rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32).into(),
    }
}

fn resolve_bg(c: TermColor) -> Option<Hsla> {
    match c {
        TermColor::Default => None,
        TermColor::Indexed(i) => Some(ansi_index(i)),
        TermColor::Rgb(r, g, b) => {
            Some(rgb(((r as u32) << 16) | ((g as u32) << 8) | b as u32).into())
        }
    }
}

/// 计算单元格有效前景 / 背景（处理反显、隐藏、暗淡、超链接）。
fn cell_colors(cell: &Cell) -> (Hsla, Option<Hsla>) {
    let mut fg = resolve_fg(cell.pen.fg, cell.pen.bold);
    let mut bg = resolve_bg(cell.pen.bg);
    if cell.pen.inverse {
        let f = fg;
        fg = bg.unwrap_or_else(theme::terminal_bg);
        bg = Some(f);
    }
    if cell.pen.hidden {
        fg = bg.unwrap_or_else(theme::terminal_bg);
    }
    if cell.pen.dim {
        fg.a *= 0.6;
    }
    if cell.link.is_some() {
        fg = theme::primary();
    }
    (fg, bg)
}

/// 粗略判定宽字符（CJK / 全角），占两个单元格。
fn is_wide(ch: char) -> bool {
    let c = ch as u32;
    (0x1100..=0x115F).contains(&c)
        || (0x2E80..=0xA4CF).contains(&c)
        || (0xAC00..=0xD7A3).contains(&c)
        || (0xF900..=0xFAFF).contains(&c)
        || (0xFE30..=0xFE4F).contains(&c)
        || (0xFF00..=0xFF60).contains(&c)
        || (0xFFE0..=0xFFE6).contains(&c)
        || (0x20000..=0x3FFFD).contains(&c)
}

/// 光标形状（xterm `cursorStyle`）。默认 `Bar`，与既有 2px 竖条一致。
/// 命名避开 gpui 的 `CursorStyle`（那是鼠标指针样式，本控件根节点在用）。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CursorShape {
    /// 实心块：铺满单元格，字符反色为终端底色
    Block,
    /// 竖条
    #[default]
    Bar,
    /// 下划线
    Underline,
}

/// 终端滚动度量；`max_scroll` 为回滚缓冲可滚动的像素高度。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerminalScrollMetrics {
    pub history_lines: usize,
    pub visible_rows: usize,
    pub viewport_height: Pixels,
    pub max_scroll: Pixels,
}

/// 终端视图。
pub struct Terminal {
    focus_handle: FocusHandle,
    id: ElementId,
    cols: usize,
    rows: usize,
    max_history: usize,
    font_size: Pixels,
    /// 行高倍数（相对 font_size）；None = 默认 CELL_H/FONT_SIZE 比例（零回归）
    line_height: Option<f32>,
    history: VecDeque<Line>,
    screen: Vec<Line>,
    cur_row: usize,
    cur_col: usize,
    pen: Pen,
    scroll_top: usize,
    scroll_bottom: usize,
    scroll_offset: usize,
    scroll_handle: ScrollHandle,
    last_scroll_handle_offset: Pixels,
    cursor_visible: bool,
    cursor_shape: CursorShape,
    cursor_blink: bool,
    blink_on: bool,
    blink_started: bool,
    saved_cursor: (usize, usize, Pen),
    state: ParseState,
    csi_buf: Vec<u8>,
    csi_private: Option<u8>,
    osc_buf: Vec<u8>,
    utf8: Vec<u8>,
    title: String,
    title_changed: bool,
    links: HashMap<u32, String>,
    next_link: u32,
    cur_link: Option<u32>,
    /// (锚点, 头部)，未归一化；读取时用 norm 归一。
    sel: Option<(Pos, Pos)>,
    selecting: bool,
    /// 平台 IME 预编辑；仅保存文本与 UTF-16 选中范围，不进入 PTY/网格。
    composition: Option<ImeComposition>,
    /// 最近一帧满足聚焦、实时底部且光标可见条件时的输入几何。
    last_input_bounds: Option<Bounds<Pixels>>,
    last_input_origin: Option<Point<Pixels>>,
    last_cell_w: Pixels,
    last_cell_h: Pixels,
    search: String,
    matches: Vec<(Pos, Pos)>,
    match_idx: usize,
    bracketed_paste: bool,
    application_cursor: bool,
    /// 外框描边开关：嵌在面板里的终端（如宿主应用的分屏格）不要那圈输入框式描边。
    bordered: bool,
    /// 宿主指定的字体族（设置里的终端字体）；None 保持平台回退 Consolas。
    font_family: Option<gpui::SharedString>,
    /// 缺字回退字体链（按顺序尝试）：主字体缺框线/块元素/符号时落到等宽兜底字体，
    /// 汉字再落到系统 CJK 字体——对齐原版 CSS 的逗号栈。
    font_fallbacks: Vec<gpui::SharedString>,
    on_input: Option<Rc<dyn Fn(&[u8], &mut Window, &mut App)>>,
    on_title: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    /// pane-fit 旋钮：设了即高度撑满父容器剩余空间，paint 期按实际
    /// bounds 换算网格并回调 (cols, rows)，宿主据此 resize PTY。
    on_fit: Option<Rc<dyn Fn(usize, usize, &mut Window, &mut App)>>,
    fit_min_cols: usize,
    fit_min_rows: usize,
    on_search_open: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_search_results: Option<Rc<dyn Fn(usize, Option<usize>, &mut Window, &mut App)>>,
    on_cwd: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    on_selection_change: Option<Rc<dyn Fn(Option<String>, &mut Window, &mut App)>>,
    selection_changed: bool,
    /// 上次已上报的选区文本：拖动期每帧都标脏，靠它去重后再投递。
    selection_reported: Option<String>,
    cwd: String,
    cwd_changed: bool,
    search_changed: bool,
    /// 上次已上报的 (总数, 当前下标)：搜索态下每次 write 都会重算匹配，
    /// 靠它去重，对齐 xterm onDidChangeResults 只在结果真变化时触发。
    search_reported: (usize, Option<usize>),
}

impl Terminal {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let cols = DEFAULT_COLS;
        let rows = DEFAULT_ROWS;
        Self {
            focus_handle: cx.focus_handle(),
            id: ElementId::Name("terminal".into()),
            cols,
            rows,
            max_history: DEFAULT_HISTORY,
            font_size: px(FONT_SIZE),
            line_height: None,
            history: VecDeque::new(),
            screen: (0..rows).map(|_| Line::blank(cols)).collect(),
            cur_row: 0,
            cur_col: 0,
            pen: Pen::default(),
            scroll_top: 0,
            scroll_bottom: rows - 1,
            scroll_offset: 0,
            scroll_handle: ScrollHandle::new(),
            last_scroll_handle_offset: px(0.),
            cursor_visible: true,
            cursor_shape: CursorShape::default(),
            cursor_blink: false,
            blink_on: true,
            blink_started: false,
            saved_cursor: (0, 0, Pen::default()),
            state: ParseState::Ground,
            csi_buf: Vec::new(),
            csi_private: None,
            osc_buf: Vec::new(),
            utf8: Vec::new(),
            title: String::new(),
            title_changed: false,
            links: HashMap::new(),
            next_link: 1,
            cur_link: None,
            sel: None,
            selecting: false,
            composition: None,
            last_input_bounds: None,
            last_input_origin: None,
            last_cell_w: px(0.),
            last_cell_h: px(0.),
            search: String::new(),
            matches: Vec::new(),
            match_idx: 0,
            bracketed_paste: false,
            application_cursor: false,
            bordered: true,
            font_family: None,
            font_fallbacks: Vec::new(),
            on_input: None,
            on_title: None,
            on_fit: None,
            fit_min_cols: 1,
            fit_min_rows: 1,
            on_search_open: None,
            on_search_results: None,
            on_cwd: None,
            on_selection_change: None,
            selection_changed: false,
            selection_reported: None,
            cwd: String::new(),
            cwd_changed: false,
            search_changed: false,
            search_reported: (0, None),
        }
    }

    /// 唯一元素 id（同窗口多实例须区分，避免 hover 串扰）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn cols(mut self, n: usize) -> Self {
        self.cols = n.max(1);
        self.screen = (0..self.rows).map(|_| Line::blank(self.cols)).collect();
        self
    }

    pub fn rows(mut self, n: usize) -> Self {
        self.rows = n.max(1);
        self.screen = (0..self.rows).map(|_| Line::blank(self.cols)).collect();
        self.scroll_bottom = self.rows - 1;
        self
    }

    pub fn font_size(mut self, size: Pixels) -> Self {
        self.font_size = size;
        self
    }

    /// 行高倍数旋钮（宿主设置项 line_height，语义为倍数，1.0~1.8）
    pub fn line_height(mut self, ratio: f32) -> Self {
        self.line_height = Some(ratio);
        self
    }

    /// 运行期换字号（设置保存后活更新）
    pub fn set_font_size(&mut self, size: Pixels, cx: &mut Context<Self>) {
        self.font_size = size;
        cx.notify();
    }

    /// 运行期换行高倍数（设置保存后活更新）
    pub fn set_line_height(&mut self, ratio: f32, cx: &mut Context<Self>) {
        self.line_height = Some(ratio);
        cx.notify();
    }

    /// 单元格高：随 font_size 缩放；设了倍数则 font_size × 倍数
    fn cell_h(&self) -> Pixels {
        match self.line_height {
            Some(r) => self.font_size * r,
            None => self.font_size * (CELL_H / FONT_SIZE),
        }
    }

    /// 指定终端字体族（宿主随包字体 / 设置里的终端字体）。
    pub fn font_family(mut self, family: impl Into<gpui::SharedString>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    /// 运行期换字体族（设置保存后活更新）；`None` 回到平台回退。
    pub fn set_font_family(&mut self, family: Option<gpui::SharedString>, cx: &mut Context<Self>) {
        self.font_family = family;
        cx.notify();
    }

    /// 指定缺字回退字体链（按顺序尝试）。
    pub fn font_fallbacks(mut self, families: Vec<gpui::SharedString>) -> Self {
        self.font_fallbacks = families;
        self
    }

    /// 运行期换回退字体链（设置保存后活更新）。
    pub fn set_font_fallbacks(
        &mut self,
        families: Vec<gpui::SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.font_fallbacks = families;
        cx.notify();
    }

    /// 是否画那圈输入框式外框（默认 `true`）。嵌进宿主面板格子里时关掉，
    /// 让终端底色直接铺满格子，与相邻区域无缝相接。
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// 回滚缓冲上限（默认 1000 行）。
    pub fn max_history(mut self, n: usize) -> Self {
        self.max_history = n;
        self
    }

    /// 运行期改回滚上限（设置保存后活更新），立即按新上限裁剪，
    /// 口径与 `scroll_up` / `resize` 的裁剪一致。
    pub fn set_max_history(&mut self, n: usize, cx: &mut Context<Self>) {
        self.max_history = n;
        let before = self.history.len();
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
        if self.history.len() != before {
            // 裁掉的是最老的行，绝对行号整体前移：选区与匹配下标随之失效。
            self.set_sel(None);
            self.set_scroll_lines(self.scroll_offset.min(self.history.len()));
            if !self.search.is_empty() {
                self.recompute_matches();
            }
        }
        cx.notify();
    }

    /// 光标形状（xterm `cursorStyle`），默认竖条。
    pub fn cursor_style(mut self, shape: CursorShape) -> Self {
        self.cursor_shape = shape;
        self
    }

    /// 运行期改光标形状（设置保存后活更新）。
    pub fn set_cursor_style(&mut self, shape: CursorShape, cx: &mut Context<Self>) {
        self.cursor_shape = shape;
        cx.notify();
    }

    /// 光标闪烁（xterm `cursorBlink`），默认关。开启后半周期约 530ms。
    pub fn cursor_blink(mut self, blink: bool) -> Self {
        self.cursor_blink = blink;
        self
    }

    /// 运行期开关闪烁（设置保存后活更新）；关掉即恢复常亮。
    pub fn set_cursor_blink(&mut self, blink: bool, cx: &mut Context<Self>) {
        self.cursor_blink = blink;
        self.blink_on = true;
        cx.notify();
    }

    /// 键盘输入回调：字节序列投递给调用方（PTY / SSH）。经 defer 延迟触发。
    pub fn on_input(mut self, f: impl Fn(&[u8], &mut Window, &mut App) + 'static) -> Self {
        self.on_input = Some(Rc::new(f));
        self
    }

    /// 标题变化回调（OSC 0/2）。经 defer 延迟触发。
    pub fn on_title(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_title = Some(Rc::new(f));
        self
    }

    /// pane-fit 旋钮（默认关，零回归）：开启后终端高度不再固定 `rows * CELL_H`，
    /// 而是撑满父容器剩余空间；每帧按 bounds / 单元格度量算出可容纳网格，
    /// 与当前网格不同时 resize 并回调 `(cols, rows)`。宿主在回调里 resize PTY。
    /// 父容器须是定高 flex 列（终端根节点以 `flex_1` 占剩余高度）。
    pub fn on_fit(mut self, f: impl Fn(usize, usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_fit = Some(Rc::new(f));
        self
    }

    /// pane-fit 尺寸闸门（默认 1×1，保持既有行为）：`on_fit` 换算出的网格
    /// 小于此门槛时，既不执行本地 `resize`，也不回调宿主。宿主可用它在
    /// 窗口拖动期间拦住 5×3 之类瞬态尺寸，避免本地 reflow 被中间态污染。
    pub fn fit_min(mut self, cols: usize, rows: usize) -> Self {
        self.fit_min_cols = cols.max(1);
        self.fit_min_rows = rows.max(1);
        self
    }

    /// 运行期调整 pane-fit 尺寸闸门。
    pub fn set_fit_min(&mut self, cols: usize, rows: usize, cx: &mut Context<Self>) {
        self.fit_min_cols = cols.max(1);
        self.fit_min_rows = rows.max(1);
        cx.notify();
    }

    /// Ctrl+Shift+F 触发：请求宿主打开搜索面板（库内不含面板 UI）。
    /// 该组合被本地拦截，不会作为 0x06 发往远端。
    pub fn on_search_open(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_search_open = Some(Rc::new(f));
        self
    }

    /// 搜索结果变化回调 `(总数, 当前下标)`，无匹配时当前下标为 `None`。
    /// 对齐原版 SearchAddon.onDidChangeResults；经 defer 延迟触发。
    pub fn on_search_results(
        mut self,
        f: impl Fn(usize, Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_search_results = Some(Rc::new(f));
        self
    }

    /// 工作目录变化回调（OSC 7 `file://host/path`）。经 defer 延迟触发。
    /// 宿主据此让 SFTP 跟随远端 cwd，不必再自行解析原始字节流。
    pub fn on_cwd(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_cwd = Some(Rc::new(f));
        self
    }

    /// 选区建立 / 变化 / 清除回调（`None` = 无选区）。经 defer 延迟触发。
    /// 宿主 copyOnSelect 直接接这里，不必再 observe 实体自行 diff。
    pub fn on_selection_change(
        mut self,
        f: impl Fn(Option<String>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(Rc::new(f));
        self
    }

    // ------------------------------------------------------------------
    // 公开 API
    // ------------------------------------------------------------------

    /// 写入远端输出字节（VT/ANSI 解析）。会清除选区并回到底部。
    pub fn write(&mut self, data: &[u8], cx: &mut Context<Self>) {
        self.set_sel(None);
        self.set_scroll_lines(0);
        // 有输出时光标常亮（xterm 同口径），避免刷屏期间光标随机消失。
        self.blink_on = true;
        for &b in data {
            self.feed(b);
        }
        self.clamp_cursor();
        if !self.search.is_empty() {
            self.recompute_matches();
        }
        cx.notify();
    }

    /// 写入字符串输出（便捷方法）。
    pub fn write_str(&mut self, s: &str, cx: &mut Context<Self>) {
        self.write(s.as_bytes(), cx);
    }

    /// 清屏（保留回滚缓冲以外的当前屏清空，光标归零）。
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.history.clear();
        self.screen = (0..self.rows).map(|_| Line::blank(self.cols)).collect();
        self.cur_row = 0;
        self.cur_col = 0;
        self.set_scroll_lines(0);
        self.set_sel(None);
        cx.notify();
    }

    /// 完全重置（缓冲、画笔、解析状态、标题、链接、搜索）。
    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.reset_state();
        self.title.clear();
        self.search.clear();
        self.matches.clear();
        self.search_changed = true;
        cx.notify();
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    /// 最近一次 OSC 7 报告的工作目录（未报告过为空串）。
    pub fn cwd(&self) -> &str {
        &self.cwd
    }

    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// 设置搜索词并高亮全部匹配（大小写不敏感）。
    pub fn set_search(&mut self, q: &str, cx: &mut Context<Self>) {
        self.search = q.to_string();
        self.recompute_matches();
        self.match_idx = 0;
        if !self.matches.is_empty() {
            self.goto_match(0);
        }
        cx.notify();
    }

    pub fn search_next(&mut self, cx: &mut Context<Self>) {
        if self.matches.is_empty() {
            return;
        }
        self.match_idx = (self.match_idx + 1) % self.matches.len();
        self.goto_match(self.match_idx);
        cx.notify();
    }

    pub fn search_prev(&mut self, cx: &mut Context<Self>) {
        if self.matches.is_empty() {
            return;
        }
        self.match_idx = (self.match_idx + self.matches.len() - 1) % self.matches.len();
        self.goto_match(self.match_idx);
        cx.notify();
    }

    pub fn search_matches(&self) -> usize {
        self.matches.len()
    }

    /// 当前命中序号（0 基）；无匹配返回 `None`。搜索条显示 `idx+1/count` 用。
    pub fn search_index(&self) -> Option<usize> {
        (!self.matches.is_empty()).then(|| self.match_idx.min(self.matches.len() - 1))
    }

    /// 全选合并缓冲（回滚 + 屏幕），对齐原版 `term.selectAll()`。
    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        let last = self.combined_len().saturating_sub(1);
        self.set_sel(Some(((0, 0), (last, self.cols))));
        cx.notify();
    }

    fn set_scroll_lines(&mut self, lines: usize) {
        self.scroll_offset = lines.min(self.history.len());
        let offset_y = terminal_handle_y(self.scroll_offset, self.history.len(), self.cell_h());
        self.scroll_handle.set_offset(point(px(0.), offset_y));
        self.last_scroll_handle_offset = offset_y;
    }

    fn sync_scroll_from_handle(&mut self) {
        let offset_y = self.scroll_handle.offset().y;
        if offset_y != self.last_scroll_handle_offset {
            self.set_scroll_lines(terminal_lines_from_handle(
                offset_y,
                self.cell_h(),
                self.history.len(),
            ));
        } else if self.scroll_offset == 0 {
            // 新输出会拉长 history；实时底部必须重新锚到新的 -max。
            self.set_scroll_lines(0);
        }
    }

    /// 与内置滚动条共享的滚动句柄；0 表示最旧顶部，`-max` 表示实时底部。
    pub fn scroll_handle(&self) -> ScrollHandle {
        self.scroll_handle.clone()
    }

    /// 当前回滚行数；0 表示实时底部。
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn scroll_metrics(&self) -> TerminalScrollMetrics {
        TerminalScrollMetrics {
            history_lines: self.history.len(),
            visible_rows: self.rows,
            viewport_height: self.cell_h() * self.rows as f32,
            max_scroll: self.cell_h() * self.history.len() as f32,
        }
    }

    /// 设置回滚行数；0 表示实时底部，越界夹到最旧一行。
    pub fn set_scroll_offset(&mut self, lines: usize, cx: &mut Context<Self>) {
        self.set_scroll_lines(lines);
        cx.notify();
    }

    pub fn scroll_to_bottom(&mut self, cx: &mut Context<Self>) {
        self.set_scroll_lines(0);
        cx.notify();
    }

    /// 当前选区文本（无选区返回 None）。
    pub fn selection_text(&self) -> Option<String> {
        let (s, e) = self.sel.map(|(a, b)| norm(a, b))?;
        if s == e {
            return None;
        }
        let mut out = String::new();
        for line in s.0..=e.0 {
            if let Some(l) = self.line_at(line) {
                let a = if line == s.0 { s.1 } else { 0 };
                let b = if line == e.0 { e.1 } else { self.cols };
                let seg: String = l
                    .cells
                    .iter()
                    .skip(a)
                    .take(b.saturating_sub(a))
                    .map(|c| c.ch)
                    .collect();
                out.push_str(seg.trim_end());
            }
            if line < e.0 {
                out.push('\n');
            }
        }
        Some(out)
    }

    pub fn copy_selection(&self, cx: &mut App) {
        if let Some(t) = self.selection_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(t));
        }
    }

    /// 当前视口文本摘要：仅暴露屏幕上实际可见的非空行，并限制长度，
    /// 避免大尺寸终端在无障碍树中携带整个网格。
    fn visible_text_summary(&self) -> String {
        const MAX_CHARS: usize = 4096;
        let visible_start = self
            .combined_len()
            .saturating_sub(self.rows + self.scroll_offset);
        let mut out = String::new();
        let mut chars = 0;
        'rows: for row in visible_start..visible_start + self.rows {
            let Some(line) = self.line_at(row) else {
                continue;
            };
            let text: String = line
                .cells
                .iter()
                .map(|cell| if cell.pen.hidden { ' ' } else { cell.ch })
                .collect();
            let text = text.trim_end();
            if text.is_empty() {
                continue;
            }
            for ch in text.chars() {
                if chars == MAX_CHARS {
                    out.push('…');
                    break 'rows;
                }
                out.push(ch);
                chars += 1;
            }
            out.push('\n');
        }
        if out.ends_with('\n') {
            out.pop();
        }
        out
    }

    fn a11y_semantics(&self, id: impl Into<gpui::SharedString>) -> a11y::Semantics {
        let label = if self.title.is_empty() {
            "终端".to_owned()
        } else {
            format!("终端：{}", self.title)
        };
        let mut semantics = a11y::Semantics::new(gpui::Role::Terminal, label)
            .id(id)
            .value(self.visible_text_summary())
            .focusable(true);
        if !self.cwd.is_empty() {
            semantics = semantics.description(format!("工作目录：{}", self.cwd));
        }
        semantics
    }

    /// 全缓冲文本（回滚 + 屏幕）：每行去尾空白后以 `\n` 连接。
    /// 对齐原版 bufferText（`translateToString(true)`）——与 `buffer_tail` 不同，
    /// **不**掐掉首尾空行，空行也占一行。
    pub fn buffer_text(&self) -> String {
        (0..self.combined_len())
            .filter_map(|row| self.line_at(row))
            .map(|l| {
                let s: String = l.cells.iter().map(|c| c.ch).collect();
                s.trim_end().to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 复制全缓冲（原版右键「复制全部」）。
    pub fn copy_all(&self, cx: &mut App) {
        cx.write_to_clipboard(ClipboardItem::new_string(self.buffer_text()));
    }

    /// 合并缓冲（回滚 + 屏幕）末尾 `max_lines` 行文本，供宿主「携带终端输出」。
    /// 语义照原版 bufferTail：每行去尾空白，再掐掉首尾空行；无内容返回空串。
    pub fn buffer_tail(&self, max_lines: usize) -> String {
        let total = self.history.len() + self.screen.len();
        let start = total.saturating_sub(max_lines.max(1));
        let lines: Vec<String> = (start..total)
            .filter_map(|row| self.line_at(row))
            .map(|l| l.cells.iter().map(|c| c.ch).collect())
            .collect();
        tail_join(&lines, max_lines)
    }

    /// 合并缓冲中屏幕首行的绝对行号（= 回滚行数）。
    pub fn screen_top(&self) -> usize {
        self.history.len()
    }

    /// 光标所在屏幕行（0 基）。
    pub fn cursor_row(&self) -> usize {
        self.cur_row
    }

    /// 程序化设置选区（绝对坐标 (行, 列)，锚点 + 头部）。
    pub fn set_selection(
        &mut self,
        anchor: (usize, usize),
        head: (usize, usize),
        cx: &mut Context<Self>,
    ) {
        self.set_sel(Some((anchor, head)));
        cx.notify();
    }

    fn reflow_columns(&mut self, new_cols: usize) {
        let old_cols = self.cols;
        if new_cols == old_cols {
            return;
        }
        let old_lines: Vec<Line> = self
            .history
            .iter()
            .cloned()
            .chain(self.screen.iter().cloned())
            .collect();
        let old_history = self.history.len();
        let old_scroll = self.scroll_offset;
        let old_view_top = old_lines.len().saturating_sub(self.rows + old_scroll);
        let old_cursor = (old_history + self.cur_row, self.cur_col);
        let old_saved = (old_history + self.saved_cursor.0, self.saved_cursor.1);
        let old_sel = self.sel;
        let reflow = reflow_lines(&old_lines, old_cols, new_cols);
        let cursor = map_reflow_pos(&reflow.positions, old_cursor);
        let saved = map_reflow_pos(&reflow.positions, old_saved);
        let view_top = map_reflow_pos(&reflow.positions, (old_view_top, 0)).0;

        let mut lines = reflow.lines;
        while lines.len() < self.rows {
            lines.push(Line::blank(new_cols));
        }
        let screen_start = lines.len().saturating_sub(self.rows);
        self.history = lines[..screen_start].iter().cloned().collect();
        self.screen = lines[screen_start..].to_vec();
        let mut removed = 0;
        while self.history.len() > self.max_history {
            self.history.pop_front();
            removed += 1;
        }

        self.cols = new_cols;
        let (cur_row, cur_col) = terminal_reflow_cursor(cursor, screen_start, self.rows, new_cols);
        self.cur_row = cur_row;
        self.cur_col = cur_col;
        let (saved_row, saved_col) =
            terminal_reflow_cursor(saved, screen_start, self.rows, new_cols);
        self.saved_cursor.0 = saved_row;
        self.saved_cursor.1 = saved_col;

        if let Some((a, b)) = old_sel {
            let a = map_reflow_pos(&reflow.positions, a);
            let b = map_reflow_pos(&reflow.positions, b);
            if a.0 < removed || b.0 < removed {
                self.set_sel(None);
            } else {
                self.set_sel(Some(((a.0 - removed, a.1), (b.0 - removed, b.1))));
            }
        }

        let match_idx = self.match_idx;
        self.recompute_matches();
        if self.matches.is_empty() {
            self.match_idx = 0;
        } else {
            self.match_idx = match_idx.min(self.matches.len() - 1);
        }

        self.scroll_top = 0;
        self.scroll_bottom = self.rows - 1;
        let final_len = self.history.len() + self.screen.len();
        let new_top = view_top.saturating_sub(removed);
        let scroll = if old_scroll == 0 {
            0
        } else {
            final_len
                .saturating_sub(self.rows + new_top)
                .min(self.history.len())
        };
        self.set_scroll_lines(scroll);
    }

    /// 调整网格尺寸（保留内容，裁剪 / 补空白）。
    pub fn resize(&mut self, cols: usize, rows: usize, cx: &mut Context<Self>) {
        let cols = cols.max(1);
        let rows = rows.max(1);
        if cols != self.cols {
            self.reflow_columns(cols);
        }
        let old_rows = self.screen.len();
        let (top_remove, bottom_remove, next_cursor) =
            terminal_resize_viewport(old_rows, rows, self.cur_row);
        for _ in 0..top_remove {
            if let Some(line) = (!self.screen.is_empty()).then(|| self.screen.remove(0)) {
                self.history.push_back(line);
            }
        }
        for _ in 0..bottom_remove {
            self.screen.pop();
        }
        self.cur_row = next_cursor;
        while self.screen.len() < rows {
            self.screen.push(Line::blank(cols));
        }
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
        self.cols = cols;
        self.rows = rows;
        self.scroll_top = 0;
        self.scroll_bottom = rows - 1;
        self.clamp_cursor();
        self.set_scroll_lines(self.scroll_offset.min(self.history.len()));
        cx.notify();
    }

    // ------------------------------------------------------------------
    // 缓冲工具
    // ------------------------------------------------------------------

    /// 选区唯一写入口：顺带标记待上报，避免漏掉某个变更点。
    fn set_sel(&mut self, sel: Option<(Pos, Pos)>) {
        self.sel = sel;
        self.selection_changed = true;
    }

    fn combined_len(&self) -> usize {
        self.history.len() + self.screen.len()
    }

    fn line_at(&self, abs: usize) -> Option<&Line> {
        if abs < self.history.len() {
            self.history.get(abs)
        } else {
            self.screen.get(abs - self.history.len())
        }
    }

    fn blank_cell(&self) -> Cell {
        let mut c = Cell::default();
        c.pen.bg = self.pen.bg;
        c
    }

    fn clamp_cursor(&mut self) {
        self.cur_row = self.cur_row.min(self.rows - 1);
        self.cur_col = self.cur_col.min(self.cols);
    }

    fn clear_row(&mut self, r: usize) {
        let blank = self.blank_cell();
        if let Some(l) = self.screen.get_mut(r) {
            for c in l.cells.iter_mut() {
                *c = blank;
            }
        }
    }

    fn erase_line_range(&mut self, r: usize, a: usize, b: usize) {
        let blank = self.blank_cell();
        if let Some(l) = self.screen.get_mut(r) {
            let a = a.min(self.cols);
            let b = b.min(self.cols);
            for c in l.cells[a..b].iter_mut() {
                *c = blank;
            }
        }
    }

    // ------------------------------------------------------------------
    // VT / ANSI 解析
    // ------------------------------------------------------------------

    fn reset_state(&mut self) {
        self.history.clear();
        self.screen = (0..self.rows).map(|_| Line::blank(self.cols)).collect();
        self.cur_row = 0;
        self.cur_col = 0;
        self.pen = Pen::default();
        self.scroll_top = 0;
        self.scroll_bottom = self.rows - 1;
        self.set_scroll_lines(0);
        self.cursor_visible = true;
        self.saved_cursor = (0, 0, Pen::default());
        self.state = ParseState::Ground;
        self.csi_buf.clear();
        self.csi_private = None;
        self.osc_buf.clear();
        self.utf8.clear();
        self.set_sel(None);
        self.links.clear();
        self.cur_link = None;
    }

    fn feed(&mut self, b: u8) {
        match self.state {
            ParseState::Ground => self.ground(b),
            ParseState::Escape => self.escape(b),
            ParseState::Csi => self.csi(b),
            ParseState::Osc => self.osc(b),
            ParseState::Dcs => {
                if b == 0x07 {
                    self.state = ParseState::Ground;
                } else if b == 0x1b {
                    self.state = ParseState::Escape;
                }
            }
            ParseState::Charset => self.state = ParseState::Ground,
        }
    }

    fn ground(&mut self, b: u8) {
        match b {
            0x1b => {
                self.utf8.clear();
                self.state = ParseState::Escape;
            }
            0x07 => {}
            0x08 => self.cur_col = self.cur_col.saturating_sub(1),
            0x09 => {
                self.cur_col = ((self.cur_col / 8) + 1) * 8;
                if self.cur_col >= self.cols {
                    self.cur_col = self.cols - 1;
                }
            }
            0x0a | 0x0b | 0x0c => self.line_feed(false),
            0x0d => self.cur_col = 0,
            0x0e | 0x0f => {}
            _ if b < 0x20 => {}
            _ => self.push_utf8(b),
        }
    }

    fn push_utf8(&mut self, b: u8) {
        self.utf8.push(b);
        let decoded = match std::str::from_utf8(&self.utf8) {
            Ok(s) => Some(s.to_string()),
            Err(e) => {
                if e.error_len().is_some() {
                    Some("\u{fffd}".to_string())
                } else {
                    None
                }
            }
        };
        if let Some(s) = decoded {
            self.utf8.clear();
            for ch in s.chars() {
                self.put_char(ch);
            }
        }
    }

    fn put_char(&mut self, ch: char) {
        if self.cur_col >= self.cols {
            self.cur_col = 0;
            self.line_feed(true);
        }
        let wide = is_wide(ch);
        if wide && self.cur_col + 1 >= self.cols {
            self.cur_col = 0;
            self.line_feed(true);
        }
        let row = self.cur_row;
        let col = self.cur_col;
        if let Some(line) = self.screen.get_mut(row) {
            if let Some(cell) = line.cells.get_mut(col) {
                cell.ch = ch;
                cell.pen = self.pen;
                cell.link = self.cur_link;
            }
            if wide {
                if let Some(cell) = line.cells.get_mut(col + 1) {
                    cell.ch = ' ';
                    cell.pen = self.pen;
                    cell.link = None;
                }
            }
        }
        self.cur_col += if wide { 2 } else { 1 };
    }

    fn line_feed(&mut self, soft: bool) {
        if self.cur_row == self.scroll_bottom {
            self.scroll_up(1);
            if let Some(line) = self.screen.get_mut(self.scroll_bottom) {
                line.wrapped = soft;
            }
        } else if self.cur_row < self.rows - 1 {
            self.cur_row += 1;
            if let Some(line) = self.screen.get_mut(self.cur_row) {
                line.wrapped = soft;
            }
        }
    }

    fn reverse_index(&mut self) {
        if self.cur_row == self.scroll_top {
            self.scroll_down(1);
        } else {
            self.cur_row = self.cur_row.saturating_sub(1);
        }
    }

    fn scroll_up(&mut self, n: usize) {
        let top = self.scroll_top;
        let bottom = self.scroll_bottom;
        for _ in 0..n {
            if top >= self.screen.len() {
                break;
            }
            let removed = self.screen.remove(top);
            let at = bottom.min(self.screen.len());
            self.screen.insert(at, Line::blank(self.cols));
            if top == 0 && bottom == self.rows - 1 {
                self.history.push_back(removed);
                while self.history.len() > self.max_history {
                    self.history.pop_front();
                }
            }
        }
    }

    fn scroll_down(&mut self, n: usize) {
        let top = self.scroll_top;
        let bottom = self.scroll_bottom;
        for _ in 0..n {
            if bottom >= self.screen.len() {
                break;
            }
            self.screen.remove(bottom);
            self.screen.insert(top, Line::blank(self.cols));
        }
    }

    fn escape(&mut self, b: u8) {
        self.utf8.clear();
        match b {
            b'[' => {
                self.state = ParseState::Csi;
                self.csi_buf.clear();
                self.csi_private = None;
            }
            b']' => {
                self.state = ParseState::Osc;
                self.osc_buf.clear();
            }
            b'P' => self.state = ParseState::Dcs,
            b'(' | b')' | b'*' | b'+' => self.state = ParseState::Charset,
            b'7' => {
                self.saved_cursor = (self.cur_row, self.cur_col, self.pen);
                self.state = ParseState::Ground;
            }
            b'8' => {
                let (r, c, p) = self.saved_cursor;
                self.cur_row = r;
                self.cur_col = c;
                self.pen = p;
                self.state = ParseState::Ground;
            }
            b'M' => {
                self.reverse_index();
                self.state = ParseState::Ground;
            }
            b'D' => {
                self.line_feed(false);
                self.state = ParseState::Ground;
            }
            b'E' => {
                self.cur_col = 0;
                self.line_feed(false);
                self.state = ParseState::Ground;
            }
            b'c' => {
                self.reset_state();
                self.state = ParseState::Ground;
            }
            _ => self.state = ParseState::Ground,
        }
    }

    fn csi(&mut self, b: u8) {
        match b {
            0x30..=0x3f => {
                if self.csi_buf.is_empty() && matches!(b, b'?' | b'>' | b'<') {
                    self.csi_private = Some(b);
                } else {
                    self.csi_buf.push(b);
                }
            }
            0x20..=0x2f => {}
            0x40..=0x7e => {
                self.dispatch_csi(b as char);
                self.state = ParseState::Ground;
            }
            0x1b => self.state = ParseState::Escape,
            _ => {}
        }
    }

    fn dispatch_csi(&mut self, f: char) {
        let raw = String::from_utf8_lossy(&self.csi_buf).to_string();
        let params: Vec<usize> = raw
            .split(';')
            .map(|s| s.parse::<usize>().unwrap_or(0))
            .collect();
        let p0 = params.first().copied().unwrap_or(0);
        let priv_q = self.csi_private == Some(b'?');
        let n = if p0 == 0 { 1 } else { p0 };
        match f {
            'A' => self.cur_row = self.cur_row.saturating_sub(n),
            'B' => self.cur_row = (self.cur_row + n).min(self.rows - 1),
            'C' => self.cur_col = (self.cur_col + n).min(self.cols - 1),
            'D' => self.cur_col = self.cur_col.saturating_sub(n),
            'E' => {
                self.cur_row = (self.cur_row + n).min(self.rows - 1);
                self.cur_col = 0;
            }
            'F' => {
                self.cur_row = self.cur_row.saturating_sub(n);
                self.cur_col = 0;
            }
            'G' | '`' => self.cur_col = (p0.max(1) - 1).min(self.cols - 1),
            'H' | 'f' => {
                let r = params.first().copied().unwrap_or(1).max(1) - 1;
                let c = params.get(1).copied().unwrap_or(1).max(1) - 1;
                self.cur_row = r.min(self.rows - 1);
                self.cur_col = c.min(self.cols - 1);
            }
            'd' => self.cur_row = (p0.max(1) - 1).min(self.rows - 1),
            'J' => self.erase_display(p0),
            'K' => self.erase_line(p0),
            'L' => self.insert_lines(n),
            'M' => self.delete_lines(n),
            'P' => self.delete_chars(n),
            '@' => self.insert_chars(n),
            'X' => self.erase_chars(n),
            'S' => self.scroll_up(n),
            'T' => self.scroll_down(n),
            'm' => self.sgr(&params),
            'h' => self.set_mode(true, priv_q, &params),
            'l' => self.set_mode(false, priv_q, &params),
            'r' => {
                let t = params.first().copied().unwrap_or(1).max(1) - 1;
                let b = params.get(1).copied().unwrap_or(self.rows).max(1) - 1;
                self.scroll_top = t.min(self.rows - 1);
                self.scroll_bottom = b.min(self.rows - 1).max(self.scroll_top);
                self.cur_row = self.scroll_top;
                self.cur_col = 0;
            }
            's' => self.saved_cursor = (self.cur_row, self.cur_col, self.pen),
            'u' => {
                let (r, c, p) = self.saved_cursor;
                self.cur_row = r;
                self.cur_col = c;
                self.pen = p;
            }
            _ => {}
        }
        self.clamp_cursor();
    }

    fn sgr(&mut self, params: &[usize]) {
        if params.is_empty() {
            self.pen = Pen::default();
            return;
        }
        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => self.pen = Pen::default(),
                1 => self.pen.bold = true,
                2 => self.pen.dim = true,
                3 => self.pen.italic = true,
                4 => self.pen.underline = true,
                7 => self.pen.inverse = true,
                8 => self.pen.hidden = true,
                9 => self.pen.strike = true,
                22 => {
                    self.pen.bold = false;
                    self.pen.dim = false;
                }
                23 => self.pen.italic = false,
                24 => self.pen.underline = false,
                27 => self.pen.inverse = false,
                28 => self.pen.hidden = false,
                29 => self.pen.strike = false,
                30..=37 => self.pen.fg = TermColor::Indexed((params[i] - 30) as u8),
                39 => self.pen.fg = TermColor::Default,
                40..=47 => self.pen.bg = TermColor::Indexed((params[i] - 40) as u8),
                49 => self.pen.bg = TermColor::Default,
                90..=97 => self.pen.fg = TermColor::Indexed((params[i] - 90 + 8) as u8),
                100..=107 => self.pen.bg = TermColor::Indexed((params[i] - 100 + 8) as u8),
                38 | 48 => {
                    let fg = params[i] == 38;
                    if params.get(i + 1) == Some(&5) {
                        let idx = params.get(i + 2).copied().unwrap_or(0) as u8;
                        if fg {
                            self.pen.fg = TermColor::Indexed(idx);
                        } else {
                            self.pen.bg = TermColor::Indexed(idx);
                        }
                        i += 2;
                    } else if params.get(i + 1) == Some(&2) {
                        let r = params.get(i + 2).copied().unwrap_or(0) as u8;
                        let g = params.get(i + 3).copied().unwrap_or(0) as u8;
                        let b = params.get(i + 4).copied().unwrap_or(0) as u8;
                        if fg {
                            self.pen.fg = TermColor::Rgb(r, g, b);
                        } else {
                            self.pen.bg = TermColor::Rgb(r, g, b);
                        }
                        i += 4;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    fn erase_display(&mut self, mode: usize) {
        match mode {
            0 => {
                self.erase_line_range(self.cur_row, self.cur_col, self.cols);
                for r in self.cur_row + 1..self.rows {
                    self.clear_row(r);
                }
            }
            1 => {
                for r in 0..self.cur_row {
                    self.clear_row(r);
                }
                self.erase_line_range(self.cur_row, 0, self.cur_col + 1);
            }
            2 | 3 => {
                for r in 0..self.rows {
                    self.clear_row(r);
                }
                if mode == 3 {
                    self.history.clear();
                }
            }
            _ => {}
        }
    }

    fn erase_line(&mut self, mode: usize) {
        match mode {
            0 => self.erase_line_range(self.cur_row, self.cur_col, self.cols),
            1 => self.erase_line_range(self.cur_row, 0, self.cur_col + 1),
            2 => self.clear_row(self.cur_row),
            _ => {}
        }
    }

    fn insert_lines(&mut self, n: usize) {
        let top = self.cur_row;
        let bottom = self.scroll_bottom;
        for _ in 0..n {
            if bottom >= self.screen.len() {
                break;
            }
            self.screen.remove(bottom);
            self.screen.insert(top, Line::blank(self.cols));
        }
    }

    fn delete_lines(&mut self, n: usize) {
        let top = self.cur_row;
        let bottom = self.scroll_bottom;
        for _ in 0..n {
            if top >= self.screen.len() {
                break;
            }
            self.screen.remove(top);
            let at = bottom.min(self.screen.len());
            self.screen.insert(at, Line::blank(self.cols));
        }
    }

    fn delete_chars(&mut self, n: usize) {
        let blank = self.blank_cell();
        let cols = self.cols;
        let row = self.cur_row;
        let start = self.cur_col.min(cols);
        let end = (start + n).min(cols);
        if let Some(line) = self.screen.get_mut(row) {
            for _ in start..end {
                line.cells.remove(start);
            }
            while line.cells.len() < cols {
                line.cells.push(blank);
            }
        }
    }

    fn insert_chars(&mut self, n: usize) {
        let blank = self.blank_cell();
        let cols = self.cols;
        let row = self.cur_row;
        let start = self.cur_col.min(cols);
        if let Some(line) = self.screen.get_mut(row) {
            for _ in 0..n {
                line.cells.insert(start, blank);
            }
            line.cells.truncate(cols);
        }
    }

    fn erase_chars(&mut self, n: usize) {
        let r = self.cur_row;
        let a = self.cur_col;
        let b = (a + n).min(self.cols);
        self.erase_line_range(r, a, b);
    }

    fn set_mode(&mut self, set: bool, priv_q: bool, params: &[usize]) {
        for &p in params {
            if priv_q {
                match p {
                    25 => self.cursor_visible = set,
                    2004 => self.bracketed_paste = set,
                    1 => self.application_cursor = set,
                    _ => {}
                }
            }
        }
    }

    fn osc(&mut self, b: u8) {
        match b {
            0x07 => {
                self.dispatch_osc();
                self.state = ParseState::Ground;
            }
            0x1b => {
                self.dispatch_osc();
                self.state = ParseState::Escape;
            }
            _ => self.osc_buf.push(b),
        }
    }

    fn dispatch_osc(&mut self) {
        let s = String::from_utf8_lossy(&self.osc_buf).to_string();
        self.osc_buf.clear();
        if let Some((cmd, rest)) = s.split_once(';') {
            match cmd {
                "0" | "2" => {
                    self.title = rest.to_string();
                    self.title_changed = true;
                }
                "7" => {
                    if let Some(path) = parse_osc7(rest) {
                        self.cwd = path;
                        self.cwd_changed = true;
                    }
                }
                "8" => {
                    let mut it = rest.splitn(2, ';');
                    let _params = it.next().unwrap_or("");
                    let url = it.next().unwrap_or("");
                    if url.is_empty() {
                        self.cur_link = None;
                    } else {
                        let id = self.next_link;
                        self.next_link += 1;
                        self.links.insert(id, url.to_string());
                        self.cur_link = Some(id);
                    }
                }
                _ => {}
            }
        }
    }

    // ------------------------------------------------------------------
    // 搜索
    // ------------------------------------------------------------------

    fn recompute_matches(&mut self) {
        self.matches.clear();
        self.search_changed = true;
        if self.search.is_empty() {
            return;
        }
        let q = self.search.to_lowercase();
        let total = self.combined_len();
        for line in 0..total {
            if let Some(l) = self.line_at(line) {
                let text: String = l.cells.iter().map(|c| c.ch).collect();
                let low = text.to_lowercase();
                let mut start = 0;
                while let Some(pos) = low[start..].find(&q) {
                    let a = start + pos;
                    let end = (a + q.len()).min(text.len());
                    let col_a = text[..a].chars().count();
                    let col_b = text[..end].chars().count();
                    self.matches.push(((line, col_a), (line, col_b)));
                    start = a + q.len().max(1);
                    if start >= low.len() {
                        break;
                    }
                }
            }
        }
    }

    fn goto_match(&mut self, idx: usize) {
        self.search_changed = true;
        if let Some(m) = self.matches.get(idx) {
            let line = m.0.0;
            let h = self.history.len();
            if line < h {
                let target_start = line.saturating_sub(self.rows / 2);
                let off = self.combined_len().saturating_sub(self.rows + target_start);
                self.set_scroll_lines(off.min(h));
            } else {
                self.set_scroll_lines(0);
            }
        }
    }

    // ------------------------------------------------------------------
    // 键盘输入
    // ------------------------------------------------------------------

    fn emit(&mut self, data: &[u8], window: &mut Window, cx: &mut Context<Self>) {
        if let Some(f) = self.on_input.clone() {
            let data = data.to_vec();
            window.defer(cx, move |window, cx| f(&data, window, cx));
        }
    }

    fn arrow(&self, c: char) -> Vec<u8> {
        if self.application_cursor {
            vec![0x1b, b'O', c as u8]
        } else {
            vec![0x1b, b'[', c as u8]
        }
    }

    fn paste(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.paste_text(&text, window, cx);
        }
    }

    /// 程序化粘贴（右键菜单用），口径与 Ctrl+V 完全一致：先做换行归一
    /// （`\r\n` / `\n` → `\r`，见 [`normalize_paste`]），远端开了 bracketed paste
    /// （mode 2004）再包 `ESC[200~` / `ESC[201~`，宿主不必自扫也不必自行归一。
    pub fn paste_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let text = normalize_paste(text);
        let bytes = if self.bracketed_paste {
            let mut v = b"\x1b[200~".to_vec();
            v.extend_from_slice(text.as_bytes());
            v.extend_from_slice(b"\x1b[201~");
            v
        } else {
            text.into_bytes()
        };
        self.emit(&bytes, window, cx);
    }

    fn on_key_down(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let k = &ev.keystroke;
        let mods = k.modifiers;
        let key = k.key.as_str();

        // Ctrl+C：有选区复制，否则发送 ^C；Ctrl+Shift+C 始终复制。
        if mods.control && !mods.shift && key == "c" {
            if self.selection_text().is_some() {
                self.copy_selection(cx);
            } else {
                self.emit(&[0x03], window, cx);
            }
            return;
        }
        if mods.control && mods.shift && (key == "c" || key == "C") {
            self.copy_selection(cx);
            return;
        }
        if mods.control && !mods.shift && key == "v" {
            self.paste(window, cx);
            return;
        }
        if mods.control && mods.shift && (key == "v" || key == "V") {
            self.paste(window, cx);
            return;
        }
        // Ctrl+Shift+A/L/F 是本地快捷键（原版 attachCustomKeyEventHandler）。
        // 必须抢在下面的控制码分支之前：那条分支只看 control 不看 shift，
        // 会把它们当成 Ctrl+字母 发出 0x01 / 0x0C / 0x06 到远端。
        if mods.control && mods.shift && !mods.alt {
            match key {
                "a" | "A" => {
                    self.select_all(cx);
                    return;
                }
                "l" | "L" => {
                    // 原版 clearScreen 即本地写这串：清屏 + 清回滚 + 光标归位。
                    self.write(b"\x1b[2J\x1b[3J\x1b[H", cx);
                    return;
                }
                "f" | "F" => {
                    if let Some(f) = self.on_search_open.clone() {
                        window.defer(cx, move |window, cx| f(window, cx));
                    }
                    return;
                }
                _ => {}
            }
        }
        // Shift+PgUp/PgDn 本地翻阅回滚。
        if mods.shift && !mods.control {
            if key == "pageup" {
                self.set_scroll_lines((self.scroll_offset + self.rows).min(self.history.len()));
                cx.notify();
                return;
            }
            if key == "pagedown" {
                self.set_scroll_lines(self.scroll_offset.saturating_sub(self.rows));
                cx.notify();
                return;
            }
        }

        // IME 已接管字符/候选导航时，raw keydown 不再向 PTY 重复发送。
        // 快捷键与控制码分支保留在上方；Esc 只取消本控件预编辑，不发给远端。
        if self.composition.is_some() {
            if key == "escape" {
                self.composition = None;
                cx.notify();
                return;
            }
            let candidate_navigation = matches!(
                key,
                "up" | "down" | "left" | "right" | "pageup" | "pagedown"
            );
            let printable = !mods.control && !mods.alt && (key == "space" || k.key_char.is_some());
            if candidate_navigation || printable {
                return;
            }
        }

        let seq: Option<Vec<u8>> = match key {
            "enter" => Some(vec![b'\r']),
            "backspace" => Some(vec![0x7f]),
            "tab" => Some(vec![b'\t']),
            "escape" => Some(vec![0x1b]),
            "up" => Some(self.arrow('A')),
            "down" => Some(self.arrow('B')),
            "right" => Some(self.arrow('C')),
            "left" => Some(self.arrow('D')),
            "home" => Some(vec![0x1b, b'[', b'H']),
            "end" => Some(vec![0x1b, b'[', b'F']),
            "pageup" => Some(vec![0x1b, b'[', b'5', b'~']),
            "pagedown" => Some(vec![0x1b, b'[', b'6', b'~']),
            "delete" => Some(vec![0x1b, b'[', b'3', b'~']),
            "insert" => Some(vec![0x1b, b'[', b'2', b'~']),
            "space" => Some(vec![b' ']),
            _ => None,
        };
        if let Some(s) = seq {
            self.emit(&s, window, cx);
            return;
        }

        // Ctrl+字母 → 控制码。
        if mods.control && !mods.alt {
            let ch = k
                .key_char
                .as_deref()
                .and_then(|s| s.chars().next())
                .or_else(|| key.chars().next());
            if let Some(ch) = ch {
                if ch.is_ascii_alphabetic() {
                    let code = (ch.to_ascii_lowercase() as u8) & 0x1f;
                    self.emit(&[code], window, cx);
                    return;
                }
            }
        }

        // 可打印字符。
        if !mods.control && !mods.alt {
            if let Some(ch) = k.key_char.as_deref() {
                self.emit(ch.as_bytes(), window, cx);
                return;
            }
        }

        // Alt+键 → ESC 前缀。
        if mods.alt {
            if let Some(ch) = k.key_char.as_deref() {
                let mut v = vec![0x1b];
                v.extend_from_slice(ch.as_bytes());
                self.emit(&v, window, cx);
            }
        }
    }
}

impl EntityInputHandler for Terminal {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let Some(composition) = self.composition.as_ref() else {
            actual_range.take();
            return None;
        };
        Some(composition.text_for_range(range_utf16, actual_range))
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(
            self.composition
                .as_ref()
                .map(ImeComposition::selected_text_range)
                .unwrap_or(UTF16Selection {
                    range: 0..0,
                    reversed: false,
                }),
        )
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.composition
            .as_ref()
            .map(ImeComposition::marked_text_range)
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.composition.take().is_some() {
            cx.notify();
        }
    }

    fn replace_text_in_range(
        &mut self,
        _range_utf16: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 终端没有可编辑网格 range；平台提交的最终文本整体写入现有输入通道。
        self.composition = None;
        if !text.is_empty() {
            self.emit(text.as_bytes(), window, cx);
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        _range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 预编辑只影响覆盖层；空串视为取消，绝不向 PTY 发字节。
        self.composition = ImeComposition::new(new_text, new_selected_range_utf16);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let composition = self.composition.as_ref()?;
        let input_bounds = self.last_input_bounds?;
        let origin = self.last_input_origin?;
        if element_bounds != input_bounds
            || self.last_cell_w <= px(0.)
            || self.last_cell_h <= px(0.)
        {
            return None;
        }
        let range = terminal_ime_clamp_range(&composition.text, range_utf16);
        let start_x = origin.x
            + self.last_cell_w
                * terminal_ime_columns_for_utf16(&composition.text, range.start) as f32;
        let end_x = origin.x
            + self.last_cell_w
                * terminal_ime_columns_for_utf16(&composition.text, range.end) as f32;
        Some(Bounds::new(
            point(start_x, origin.y),
            size(end_x - start_x, self.last_cell_h),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let composition = self.composition.as_ref()?;
        let input_bounds = self.last_input_bounds?;
        let origin = self.last_input_origin?;
        if self.last_cell_w <= px(0.)
            || self.last_cell_h <= px(0.)
            || point.y < origin.y
            || point.y >= origin.y + self.last_cell_h
            || point.y < input_bounds.top()
            || point.y >= input_bounds.bottom()
        {
            return None;
        }
        let column = ((point.x - origin.x) / self.last_cell_w).round().max(0.) as usize;
        Some(terminal_ime_utf16_for_column(&composition.text, column))
    }
}

/// 渲染快照：在 paint 开头一次性克隆，释放实体租约后再绘制（避免与 &mut App 冲突）。
struct Snapshot {
    cols: usize,
    rows: usize,
    visible_start: usize,
    history_len: usize,
    cur_row: usize,
    cur_col: usize,
    cursor_visible: bool,
    cursor_shape: CursorShape,
    /// 闪烁关闭时恒 true；开启时按半周期翻转
    blink_lit: bool,
    focused: bool,
    live: bool,
    font_size: Pixels,
    visible: Vec<Vec<Cell>>,
    sel: Option<(Pos, Pos)>,
    composition: Option<ImeComposition>,
    search_map: HashMap<usize, Vec<(usize, usize, bool)>>,
}

struct TerminalElement {
    terminal: Entity<Terminal>,
    id: ElementId,
}

struct TerminalPrepaint {
    hitbox: Hitbox,
}

impl Element for TerminalElement {
    type RequestLayoutState = ();
    type PrepaintState = TerminalPrepaint;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let t = self.terminal.read(cx);
        let rows = t.rows;
        let fit = t.on_fit.is_some();
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = if fit {
            relative(1.).into()
        } else {
            (t.cell_h() * rows as f32).into()
        };
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
        TerminalPrepaint { hitbox }
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
        // 单元格度量前置：pane-fit 要在取快照前按 bounds 换算网格，
        // 度量结果本帧绘制复用，避免重复 shape。
        let (font_size, font_family, font_fallbacks, on_fit, fit_min) = {
            let t = self.terminal.read(cx);
            (
                t.font_size,
                t.font_family.clone().unwrap_or_else(|| "Consolas".into()),
                t.font_fallbacks.clone(),
                t.on_fit.clone(),
                (t.fit_min_cols, t.fit_min_rows),
            )
        };
        let cell_h = self.terminal.read(cx).cell_h();
        let mut base_font = gpui::font(font_family.clone());
        if !font_fallbacks.is_empty() {
            base_font.fallbacks = Some(gpui::FontFallbacks::from_fonts(
                font_fallbacks.iter().map(|f| f.to_string()).collect(),
            ));
        }
        let probe_run = TextRun {
            len: 10,
            font: base_font.clone(),
            color: theme::terminal_fg(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let probe =
            window
                .text_system()
                .shape_line("0000000000".into(), font_size, &[probe_run], None);
        let cell_w = (probe.width / 10.).max(px(1.));
        if let Some(f) = on_fit {
            if let Some((fc, fr)) = terminal_fit_gate(
                bounds.size.width,
                bounds.size.height,
                cell_w,
                cell_h,
                fit_min,
            ) {
                let cur = {
                    let t = self.terminal.read(cx);
                    (t.cols, t.rows)
                };
                if (fc, fr) != cur {
                    self.terminal.update(cx, |t, cx| t.resize(fc, fr, cx));
                    f(fc, fr, window, cx);
                }
            }
        }

        let snap = {
            let t = self.terminal.read(cx);
            let cols = t.cols;
            let rows = t.rows;
            let visible_start = t.combined_len().saturating_sub(rows + t.scroll_offset);
            let visible = (0..rows)
                .map(|r| {
                    t.line_at(visible_start + r)
                        .map(|l| l.cells.clone())
                        .unwrap_or_else(|| vec![Cell::default(); cols])
                })
                .collect::<Vec<_>>();
            let mut search_map: HashMap<usize, Vec<(usize, usize, bool)>> = HashMap::new();
            for (i, m) in t.matches.iter().enumerate() {
                search_map
                    .entry(m.0.0)
                    .or_default()
                    .push((m.0.1, m.1.1, i == t.match_idx));
            }
            Snapshot {
                cols,
                rows,
                visible_start,
                history_len: t.history.len(),
                cur_row: t.cur_row,
                cur_col: t.cur_col,
                cursor_visible: t.cursor_visible,
                cursor_shape: t.cursor_shape,
                blink_lit: !t.cursor_blink || t.blink_on,
                focused: t.focus_handle.is_focused(window),
                live: t.scroll_offset == 0,
                font_size: t.font_size,
                visible,
                sel: t.sel.map(|(a, b)| norm(a, b)),
                composition: t.composition.clone(),
                search_map,
            }
        };

        let left = bounds.left();
        let top = bounds.top();

        // 只在可输入位置安装平台 IME handler；滚动离开实时底部/隐藏光标时不接管文本输入。
        if snap.focused && snap.live && snap.cursor_visible {
            let focus_handle = self.terminal.read(cx).focus_handle.clone();
            let origin = point(
                left + cell_w * snap.cur_col as f32,
                top + cell_h * snap.cur_row as f32,
            );
            self.terminal.update(cx, |terminal, _| {
                terminal.last_input_bounds = Some(bounds);
                terminal.last_input_origin = Some(origin);
                terminal.last_cell_w = cell_w;
                terminal.last_cell_h = cell_h;
            });
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.terminal.clone()),
                cx,
            );
        }

        // 背景
        window.paint_quad(fill(bounds, theme::terminal_bg()));

        let empty: Vec<(usize, usize, bool)> = Vec::new();

        // 光标几何。block 必须铺在字之前并把该格字符反色，否则会盖住字形；
        // bar / underline 仍按老口径画在字之后。
        let caret = if snap.live
            && snap.cursor_visible
            && snap.focused
            && snap.blink_lit
            && snap.cur_col < snap.cols
        {
            let crow = (snap.history_len + snap.cur_row).saturating_sub(snap.visible_start);
            (crow < snap.rows).then(|| {
                let thick = px(theme::with(metric::TerminalMetric::resolve).caret_width);
                let x = left + cell_w * snap.cur_col as f32;
                let y = top + cell_h * crow as f32;
                let quad = match snap.cursor_shape {
                    CursorShape::Block => Bounds::new(point(x, y), size(cell_w, cell_h)),
                    CursorShape::Bar => Bounds::new(point(x, y), size(thick, cell_h)),
                    CursorShape::Underline => {
                        Bounds::new(point(x, y + cell_h - thick), size(cell_w, thick))
                    }
                };
                (crow, quad)
            })
        } else {
            None
        };
        let block_caret = caret.filter(|_| snap.cursor_shape == CursorShape::Block);

        // 单元格背景 / 选区 / 搜索高亮
        for row in 0..snap.rows {
            let y = top + cell_h * row as f32;
            if y > bounds.bottom() {
                break;
            }
            let abs = snap.visible_start + row;
            let cells = &snap.visible[row];
            let ranges = snap.search_map.get(&abs).unwrap_or(&empty);
            for col in 0..snap.cols {
                let x = left + cell_w * col as f32;
                if x > bounds.right() {
                    break;
                }
                let mut bg = cell_colors(&cells[col]).1;
                if let Some((s, e)) = snap.sel {
                    if (abs, col) >= s && (abs, col) < e {
                        let mut c = theme::primary();
                        c.a = 0.30;
                        bg = Some(c);
                    }
                }
                for &(a, b, cur) in ranges {
                    if col >= a && col < b {
                        let mut c = theme::warning();
                        c.a = if cur { 0.55 } else { 0.30 };
                        bg = Some(c);
                    }
                }
                if let Some(bg) = bg {
                    window.paint_quad(fill(Bounds::new(point(x, y), size(cell_w, cell_h)), bg));
                }
            }
        }

        // block 光标铺在单元格背景之上、文本之下（该格字符随后反色绘制）
        if let Some((_, quad)) = block_caret {
            window.paint_quad(fill(quad, theme::primary()));
        }

        // 文本（按 画笔+链接 批量整形，等宽保证网格对齐）
        for row in 0..snap.rows {
            let y = top + cell_h * row as f32;
            if y > bounds.bottom() {
                break;
            }
            let cells = &snap.visible[row];
            let caret_col = block_caret.and_then(|(r, _)| (r == row).then_some(snap.cur_col));
            let mut col = 0;
            while col < snap.cols {
                let pen = cells[col].pen;
                let link = cells[col].link;
                let start = col;
                let mut s = String::new();
                while col < snap.cols
                    && cells[col].pen == pen
                    && cells[col].link == link
                    && caret_col.map_or(true, |c| (col == c) == (start == c))
                {
                    s.push(cells[col].ch);
                    col += 1;
                }
                if s.trim().is_empty() {
                    continue;
                }
                let x = left + cell_w * start as f32;
                if x > bounds.right() {
                    continue;
                }
                let (fg, _) = cell_colors(&cells[start]);
                let fg = if caret_col == Some(start) {
                    theme::terminal_bg()
                } else {
                    fg
                };
                let mut font = base_font.clone();
                if pen.bold {
                    font = font.bold();
                }
                if pen.italic {
                    font = font.italic();
                }
                let run = TextRun {
                    len: s.len(),
                    font,
                    color: fg,
                    background_color: None,
                    underline: if pen.underline || link.is_some() {
                        Some(UnderlineStyle {
                            thickness: px(1.),
                            color: Some(fg),
                            wavy: false,
                        })
                    } else {
                        None
                    },
                    strikethrough: if pen.strike {
                        Some(StrikethroughStyle {
                            thickness: px(1.),
                            color: Some(fg),
                        })
                    } else {
                        None
                    },
                };
                let shaped =
                    window
                        .text_system()
                        .shape_line(s.into(), snap.font_size, &[run], None);
                let _ = shaped.paint(
                    point(x, y),
                    cell_h,
                    gpui::TextAlign::default(),
                    None,
                    window,
                    cx,
                );
            }
        }

        // IME 预编辑覆盖层：遮住光标处原网格文字并原位绘制，绝不改网格内容。
        // 即使光标处于闪烁灭半周期，预编辑仍须可见。
        if snap.focused
            && snap.live
            && snap.cursor_visible
            && let Some(composition) = snap.composition.as_ref()
        {
            let x = left + cell_w * snap.cur_col as f32;
            let y = top + cell_h * snap.cur_row as f32;
            let run = TextRun {
                len: composition.text.len(),
                font: base_font.clone(),
                color: theme::terminal_fg(),
                background_color: None,
                underline: Some(UnderlineStyle {
                    thickness: px(1.),
                    color: Some(theme::primary()),
                    wavy: false,
                }),
                strikethrough: None,
            };
            let shaped = window.text_system().shape_line(
                composition.text.clone().into(),
                snap.font_size,
                &[run],
                None,
            );
            let width = shaped.width.max(px(0.));
            window.paint_quad(fill(
                Bounds::new(point(x, y), size(width, cell_h)),
                theme::terminal_bg(),
            ));
            let mut highlight = theme::primary();
            highlight.a = 0.22;
            window.paint_quad(fill(
                Bounds::new(point(x, y), size(width, cell_h)),
                highlight,
            ));
            let _ = shaped.paint(
                point(x, y),
                cell_h,
                gpui::TextAlign::default(),
                None,
                window,
                cx,
            );
        }

        // 光标（bar / underline；block 已在字前铺底）
        if let Some((_, quad)) = caret.filter(|_| snap.cursor_shape != CursorShape::Block) {
            window.paint_quad(fill(quad, theme::primary()));
        }

        // ---------------- 鼠标交互 ----------------
        let hitbox = prepaint.hitbox.clone();
        let entity = self.terminal.clone();
        let cols = snap.cols;
        let visible_start = snap.visible_start;

        let cell_from = move |pos: Point<Pixels>| -> Pos {
            let col = ((pos.x - left) / cell_w).floor().max(0.) as usize;
            let row = ((pos.y - top) / cell_h).floor().max(0.) as usize;
            (visible_start + row, col.min(cols.saturating_sub(1)))
        };

        // 按下：聚焦 + 设置选区锚点
        window.on_mouse_event({
            let entity = entity.clone();
            let hitbox = hitbox.clone();
            let cell_from = cell_from.clone();
            move |ev: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || ev.button != MouseButton::Left
                    || !hitbox.is_hovered(window)
                {
                    return;
                }
                let pos = cell_from(ev.position);
                let shift = ev.modifiers.shift;
                entity.update(cx, |t, cx| {
                    t.focus_handle.focus(window, cx);
                    if shift {
                        if let Some((a, _)) = t.sel {
                            t.set_sel(Some((a, pos)));
                        } else {
                            t.set_sel(Some((pos, pos)));
                        }
                    } else {
                        t.set_sel(Some((pos, pos)));
                    }
                    t.selecting = true;
                    cx.notify();
                });
            }
        });

        // 拖动：更新选区头部
        window.on_mouse_event({
            let entity = entity.clone();
            let cell_from = cell_from.clone();
            move |ev: &MouseMoveEvent, phase, _window, cx| {
                if phase != DispatchPhase::Bubble {
                    return;
                }
                if !entity.read(cx).selecting {
                    return;
                }
                if ev.pressed_button != Some(MouseButton::Left) {
                    entity.update(cx, |t, _| t.selecting = false);
                    return;
                }
                let pos = cell_from(ev.position);
                entity.update(cx, |t, cx| {
                    if let Some((a, _)) = t.sel {
                        t.set_sel(Some((a, pos)));
                    }
                    cx.notify();
                });
            }
        });

        // 松开：结束拖拽；单击链接则打开
        window.on_mouse_event({
            let entity = entity.clone();
            move |ev: &MouseUpEvent, phase, _window, cx| {
                if phase != DispatchPhase::Bubble || ev.button != MouseButton::Left {
                    return;
                }
                entity.update(cx, |t, cx| {
                    t.selecting = false;
                    if let Some((a, h)) = t.sel {
                        if a == h {
                            let url = t
                                .line_at(a.0)
                                .and_then(|l| l.cells.get(a.1))
                                .and_then(|c| c.link)
                                .and_then(|id| t.links.get(&id))
                                .cloned();
                            if let Some(url) = url {
                                cx.open_url(&url);
                            }
                            t.set_sel(None);
                        }
                    }
                    cx.notify();
                });
            }
        });

        // 滚轮：翻阅回滚缓冲
        window.on_mouse_event({
            let entity = entity.clone();
            let hitbox = hitbox.clone();
            move |ev: &ScrollEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !hitbox.is_hovered(window) {
                    return;
                }
                let delta = ev.delta.pixel_delta(window.line_height());
                let lines = (delta.y / cell_h).round() as i32;
                entity.update(cx, |t, cx| {
                    let max = t.history.len() as i32;
                    t.set_scroll_lines((t.scroll_offset as i32 + lines).clamp(0, max) as usize);
                    cx.notify();
                });
            }
        });
    }
}

/// 终端外框描边色：`bordered(false)` 时返回 `None` 不描边；聚焦走主色、
/// 未聚焦走 `colorBorder`，与输入类控件同一套。
pub fn terminal_frame(bordered: bool, focused: bool) -> Option<Hsla> {
    bordered.then(|| {
        if focused {
            theme::primary()
        } else {
            theme::border()
        }
    })
}

#[derive(Clone)]
struct ReflowResult {
    lines: Vec<Line>,
    /// 旧物理行/列 → 新物理行/列；每行长度 old_cols + 1，包含行尾。
    positions: Vec<Vec<(usize, usize)>>,
}

fn trim_trailing_blanks(cells: &[Cell]) -> &[Cell] {
    let mut end = cells.len();
    while end > 0 && cells[end - 1] == Cell::default() {
        end -= 1;
    }
    &cells[..end]
}

fn wrap_cells(cells: Vec<Cell>, cols: usize) -> (Vec<Line>, Vec<(usize, usize)>, Vec<usize>) {
    let mut lines = Vec::new();
    let mut positions = Vec::with_capacity(cells.len());
    let mut content_lens = Vec::new();
    let mut current = Vec::with_capacity(cols);
    let mut i = 0;
    while i < cells.len() {
        let wide = is_wide(cells[i].ch);
        let width = if wide { 2 } else { 1 };
        if !current.is_empty() && current.len() + width > cols {
            content_lens.push(current.len());
            lines.push(Line {
                cells: current,
                wrapped: !lines.is_empty(),
            });
            current = Vec::with_capacity(cols);
        }
        let row = lines.len();
        let col = current.len();
        positions.push((row, col));
        current.push(cells[i].clone());
        if wide && i + 1 < cells.len() {
            if cols >= 2 {
                positions.push((row, current.len()));
                current.push(cells[i + 1].clone());
            } else {
                positions.push((row, col.min(cols)));
            }
            i += 1;
        }
        i += 1;
    }
    if !current.is_empty() || lines.is_empty() {
        content_lens.push(current.len());
        lines.push(Line {
            cells: current,
            wrapped: !lines.is_empty(),
        });
    }
    for line in &mut lines {
        line.cells.resize(cols, Cell::default());
    }
    (lines, positions, content_lens)
}

fn reflow_lines(old_lines: &[Line], old_cols: usize, new_cols: usize) -> ReflowResult {
    let new_cols = new_cols.max(1);
    let mut positions = vec![vec![(0, 0); old_cols + 1]; old_lines.len()];
    let mut lines = Vec::new();
    let mut start = 0;
    while start < old_lines.len() {
        let mut end = start + 1;
        while end < old_lines.len() && old_lines[end].wrapped {
            end += 1;
        }
        let mut cells = Vec::new();
        let mut segments = Vec::new();
        for row in start..end {
            let trimmed = trim_trailing_blanks(&old_lines[row].cells);
            segments.push((row, cells.len(), trimmed.len()));
            cells.extend_from_slice(trimmed);
        }
        let new_start = lines.len();
        let (wrapped_lines, cell_positions, content_lens) = wrap_cells(cells, new_cols);
        let end_pos = (wrapped_lines.len() - 1, *content_lens.last().unwrap_or(&0));
        for (row, offset, len) in segments {
            for col in 0..=old_cols {
                let pos = cell_positions
                    .get(offset + col.min(len))
                    .copied()
                    .unwrap_or(end_pos);
                positions[row][col] = (new_start + pos.0, pos.1);
            }
        }
        lines.extend(wrapped_lines);
        start = end;
    }
    ReflowResult { lines, positions }
}

fn map_reflow_pos(positions: &[Vec<(usize, usize)>], pos: Pos) -> Pos {
    positions
        .get(pos.0)
        .and_then(|row| row.get(pos.1))
        .copied()
        .unwrap_or(pos)
}

/// resize 后的光标屏幕行（纯函数）：缩小高度时减去顶部移除行数，扩大高度时保持不变。
fn terminal_resize_cursor_row(cur_row: usize, old_rows: usize, new_rows: usize) -> usize {
    let removed = old_rows.saturating_sub(new_rows);
    cur_row
        .saturating_sub(removed)
        .min(new_rows.saturating_sub(1))
}

/// resize 缩小行数的视口裁剪（纯函数）：优先把顶部行滚入 history，
/// 但保证光标行留在屏幕内；光标上方行不够时再从底部丢弃空白/后续行。
/// 返回 `(顶部移除数, 底部移除数, 新光标行)`。
fn terminal_resize_viewport(
    old_rows: usize,
    new_rows: usize,
    cur_row: usize,
) -> (usize, usize, usize) {
    if new_rows >= old_rows {
        return (0, 0, cur_row.min(new_rows.saturating_sub(1)));
    }
    let remove = old_rows - new_rows;
    let top_remove = remove.min(cur_row.min(old_rows.saturating_sub(1)));
    (
        top_remove,
        remove - top_remove,
        terminal_resize_cursor_row(cur_row, old_rows, new_rows),
    )
}

/// pane-fit 网格换算（纯函数）：可用宽高除以单元格度量，向下取整。
/// 任一输入非正返回 `(0, 0)`（容器尚未布局完成，调用方应跳过）；
/// 否则每维至少 1 格，避免零尺寸网格把 PTY 弄坏。
pub fn fit_grid(width: Pixels, height: Pixels, cell_w: Pixels, cell_h: Pixels) -> (usize, usize) {
    if width <= px(0.) || height <= px(0.) || cell_w <= px(0.) || cell_h <= px(0.) {
        return (0, 0);
    }
    (
        ((width / cell_w).floor() as usize).max(1),
        ((height / cell_h).floor() as usize).max(1),
    )
}

/// pane-fit 尺寸闸门（纯函数）：网格可容纳且不小于 `min` 才放行。
/// 返回 `None` 时调用方既不 resize 本地终端，也不通知宿主 resize PTY。
pub fn terminal_fit_gate(
    width: Pixels,
    height: Pixels,
    cell_w: Pixels,
    cell_h: Pixels,
    min: (usize, usize),
) -> Option<(usize, usize)> {
    let (cols, rows) = fit_grid(width, height, cell_w, cell_h);
    (cols >= min.0.max(1) && rows >= min.1.max(1)).then_some((cols, rows))
}

/// reflow 后光标落到屏幕区的哪一个位置（纯函数）。
fn terminal_reflow_cursor(
    pos: Pos,
    screen_start: usize,
    rows: usize,
    cols: usize,
) -> (usize, usize) {
    if pos.0 >= screen_start {
        (
            (pos.0 - screen_start).min(rows.saturating_sub(1)),
            pos.1.min(cols),
        )
    } else {
        // 光标被中间态推进 history 时，贴回屏幕顶行；绝不能甩到底行让远端重绘落空。
        (0, pos.1.min(cols))
    }
}

/// `ScrollHandle` 负偏移 → 距顶部行数（纯函数）；吸附到最近整行并夹到缓冲上限。
pub fn terminal_scroll_lines(offset_y: Pixels, cell_h: Pixels, max_lines: usize) -> usize {
    if cell_h <= px(0.) {
        return 0;
    }
    ((-offset_y / cell_h).round().max(0.) as usize).min(max_lines)
}

/// 终端「距底部行数」→ `ScrollHandle` 负偏移（纯函数）。
fn terminal_handle_y(lines_from_bottom: usize, history_lines: usize, cell_h: Pixels) -> Pixels {
    let from_top = history_lines.saturating_sub(lines_from_bottom.min(history_lines));
    -cell_h * from_top as f32
}

/// `ScrollHandle` 负偏移 → 终端「距底部行数」（纯函数）。
fn terminal_lines_from_handle(offset_y: Pixels, cell_h: Pixels, history_lines: usize) -> usize {
    if cell_h <= px(0.) {
        return 0;
    }
    let from_top = terminal_scroll_lines(offset_y, cell_h, history_lines);
    history_lines.saturating_sub(from_top)
}

/// 取末尾 max(1, max_lines) 行、每行去尾空白、掐掉首尾空行后拼接（纯函数）。
/// 行内容已由调用方从网格取出；窗口不足时全取，空输入返回空串。
pub fn tail_join(lines: &[String], max_lines: usize) -> String {
    let start = lines.len().saturating_sub(max_lines.max(1));
    let mut picked: Vec<&str> = lines[start..].iter().map(|s| s.trim_end()).collect();
    while picked.first().is_some_and(|s| s.trim().is_empty()) {
        picked.remove(0);
    }
    while picked.last().is_some_and(|s| s.trim().is_empty()) {
        picked.pop();
    }
    picked.join("\n")
}

/// 解析 OSC 7 载荷 `file://host/path` 为工作目录（纯函数）。
/// 前缀大小写不敏感，但路径本身保留原大小写；百分号解码后去掉尾斜杠
/// （根 `/` 除外），非绝对路径一律丢弃。
pub fn parse_osc7(rest: &str) -> Option<String> {
    let head = rest.get(..7)?;
    if !head.eq_ignore_ascii_case("file://") {
        return None;
    }
    let host_path = &rest[7..];
    let path = percent_decode(&host_path[host_path.find('/')?..]);
    let path = if path.len() > 1 {
        path.trim_end_matches('/')
    } else {
        path.as_str()
    };
    if !path.starts_with('/') {
        return None;
    }
    Some(path.to_string())
}

/// 百分号解码：按字节收集再整体转 UTF-8，避免多字节字符被逐字节截断。
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = s.get(i + 1..i + 3)
            && hex.bytes().all(|b| b.is_ascii_hexdigit())
            && let Ok(v) = u8::from_str_radix(hex, 16)
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// 粘贴换行归一（纯函数）：`\r\n` 与单独的 `\n` 一律成 `\r`，单独的 `\r` 原样保留。
/// 口径同原版 xterm.js `Terminal.paste` 的 `data.replace(/\r?\n/g, '\r')`——
/// 终端里换行就是回车，不归一会把 LF 当独立字符发给远端，多行粘贴会错行。
pub fn normalize_paste(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' && chars.peek() == Some(&'\n') {
            chars.next();
        }
        out.push(if ch == '\n' { '\r' } else { ch });
    }
    out
}

impl Render for Terminal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 闪烁定时器惰性起：关闭时零唤醒，不影响未开该旋钮的宿主。
        if self.cursor_blink && !self.blink_started {
            self.blink_started = true;
            let this = cx.weak_entity();
            let mut app = cx.to_async();
            cx.foreground_executor()
                .spawn(async move {
                    loop {
                        app.background_executor()
                            .timer(Duration::from_millis(CURSOR_BLINK_MS))
                            .await;
                        let keep = this.update(&mut app, |t, cx| {
                            if !t.cursor_blink {
                                // 关掉后退出循环并复位，便于再次开启时重起定时器。
                                t.blink_started = false;
                                return false;
                            }
                            t.blink_on = !t.blink_on;
                            cx.notify();
                            true
                        });
                        if !matches!(keep, Ok(true)) {
                            break;
                        }
                    }
                })
                .detach();
        }
        // 标题回调延迟触发，避免在 write 租约内反向 update。
        if self.title_changed {
            self.title_changed = false;
            if let Some(f) = self.on_title.clone() {
                let title = self.title.clone();
                window.defer(cx, move |window, cx| f(&title, window, cx));
            }
        }
        if self.selection_changed {
            self.selection_changed = false;
            let text = self.selection_text();
            if text != self.selection_reported {
                self.selection_reported = text.clone();
                if let Some(f) = self.on_selection_change.clone() {
                    window.defer(cx, move |window, cx| f(text, window, cx));
                }
            }
        }
        if self.cwd_changed {
            self.cwd_changed = false;
            if let Some(f) = self.on_cwd.clone() {
                let cwd = self.cwd.clone();
                window.defer(cx, move |window, cx| f(&cwd, window, cx));
            }
        }
        if self.search_changed {
            self.search_changed = false;
            let count = self.matches.len();
            let cur = self.search_index();
            if (count, cur) != self.search_reported {
                self.search_reported = (count, cur);
                if let Some(f) = self.on_search_results.clone() {
                    window.defer(cx, move |window, cx| f(count, cur, window, cx));
                }
            }
        }
        let focused = self.focus_handle.is_focused(window);
        self.sync_scroll_from_handle();
        let scroll_metrics = self.scroll_metrics();
        let frame = terminal_frame(self.bordered, focused);
        let root = div()
            .id(self.id.clone())
            .key_context("Terminal")
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .on_key_down(cx.listener(Self::on_key_down))
            .flex()
            .flex_row()
            .w_full()
            .overflow_hidden()
            .bg(theme::terminal_bg());
        let root = match frame {
            Some(color) => root.rounded_md().border_1().border_color(color),
            None => root,
        };
        // pane-fit：根节点以 flex_1 吃掉父 flex 列的剩余高度（命令条等
        // 固定兄弟先占位），元素高度 relative(1.) 才有定值可解析。
        let root = if self.on_fit.is_some() {
            root.flex_1().min_h(px(0.))
        } else {
            root
        };
        let terminal_a11y_id = format!("terminal-content-{:?}-{:?}", self.id, cx.entity_id());
        let terminal_a11y = self.a11y_semantics(terminal_a11y_id.clone());
        let mut row = div().flex().flex_row().flex_1().min_w(px(0.)).child(
            div().flex_1().min_w(px(0.)).child(TerminalElement {
                terminal: cx.entity(),
                id: ElementId::Name(terminal_a11y_id.into()),
            }),
        );
        if scroll_metrics.max_scroll > px(0.) {
            row = row.child(crate::Scrollbar::vertical_with_metrics(
                ElementId::Name(
                    format!("terminal-scrollbar-{:?}-{:?}", self.id, cx.entity_id()).into(),
                ),
                self.scroll_handle.clone(),
                scroll_metrics.viewport_height,
                scroll_metrics.max_scroll,
            ));
        }
        terminal_a11y.apply(root.child(row))
    }
}

impl Focusable for Terminal {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Cell, ImeComposition, Line, TermColor, map_reflow_pos, reflow_lines, terminal_fit_gate,
        terminal_handle_y, terminal_ime_clamp_range, terminal_ime_columns_for_utf16,
        terminal_ime_text_for_range, terminal_ime_utf16_for_column, terminal_ime_utf16_len,
        terminal_ime_utf16_to_byte, terminal_lines_from_handle, terminal_reflow_cursor,
        terminal_resize_cursor_row, terminal_resize_viewport,
    };
    use gpui::px;

    #[test]
    fn terminal_ime_utf16_len_counts_ascii_cjk_and_emoji() {
        assert_eq!(terminal_ime_utf16_len(""), 0);
        assert_eq!(terminal_ime_utf16_len("ascii"), 5);
        assert_eq!(terminal_ime_utf16_len("中文"), 2);
        assert_eq!(terminal_ime_utf16_len("😀"), 2);
        assert_eq!(terminal_ime_utf16_len("a中😀z"), 5);
    }

    #[test]
    fn terminal_ime_selection_clamp_normalizes_and_bounds() {
        let text = "a中😀z";
        assert_eq!(terminal_ime_clamp_range(text, 0..99), 0..5);
        assert_eq!(terminal_ime_clamp_range(text, 99..0), 0..5);
        assert_eq!(terminal_ime_clamp_range(text, 4..1), 1..4);
        assert_eq!(terminal_ime_clamp_range(text, 2..4), 2..4);

        let composition = ImeComposition::new(text, Some(4..99)).unwrap();
        assert_eq!(composition.selected_text_range().range, 4..5);
        assert_eq!(composition.marked_text_range(), 0..5);
        let mut actual_range = None;
        assert_eq!(
            composition.text_for_range(1..99, &mut actual_range),
            "中😀z"
        );
        assert_eq!(actual_range, Some(1..5));
        assert_eq!(
            ImeComposition::new(text, None)
                .unwrap()
                .selected_text_range()
                .range,
            5..5
        );
    }

    #[test]
    fn terminal_ime_utf16_conversion_keeps_character_boundaries() {
        let text = "a中😀z";
        let expected = [(0, 0), (1, 1), (2, 4), (3, 4), (4, 8), (5, 9), (99, 9)];
        for (utf16, byte) in expected {
            assert_eq!(terminal_ime_utf16_to_byte(text, utf16), byte);
        }
        assert_eq!(terminal_ime_utf16_to_byte("😀", 1), 0);
        assert_eq!(terminal_ime_utf16_to_byte("😀", 2), 4);
    }

    #[test]
    fn terminal_ime_text_for_range_handles_boundaries_and_overflow() {
        let text = "A中😀Z";
        for (range, expected) in [
            (0..1, "A"),
            (1..2, "中"),
            (2..4, "😀"),
            (4..5, "Z"),
            (1..4, "中😀"),
            (0..99, text),
            (3..3, ""),
            (2..3, ""),
        ] {
            assert_eq!(terminal_ime_text_for_range(text, range), expected);
        }
    }

    #[test]
    fn terminal_ime_grid_geometry_uses_terminal_cell_widths() {
        let text = "a中😀z";
        let expected = [(0, 0), (1, 1), (2, 3), (3, 3), (4, 4), (5, 5)];
        for (utf16, columns) in expected {
            assert_eq!(terminal_ime_columns_for_utf16(text, utf16), columns);
        }
        for (columns, utf16) in [(0, 0), (1, 1), (2, 1), (3, 2), (4, 4), (5, 5)] {
            assert_eq!(terminal_ime_utf16_for_column(text, columns), utf16);
        }
    }

    #[test]
    fn terminal_ime_empty_composition_is_unmarked_and_empty() {
        assert!(ImeComposition::new("", None).is_none());
        assert!(ImeComposition::new("", Some(0..0)).is_none());
        assert_eq!(terminal_ime_clamp_range("", 9..1), 0..0);
        assert_eq!(terminal_ime_text_for_range("", 0..99), "");
        assert_eq!(terminal_ime_utf16_len(""), 0);
    }

    #[test]
    fn terminal_direction_bottom_maps_to_handle_bottom() {
        assert_eq!(terminal_handle_y(0, 10, px(18.)), px(-180.));
    }

    #[test]
    fn terminal_direction_top_maps_to_handle_top() {
        assert_eq!(terminal_handle_y(10, 10, px(18.)), px(0.));
        assert_eq!(terminal_handle_y(99, 10, px(18.)), px(0.));
    }

    #[test]
    fn terminal_direction_roundtrip_preserves_lines_from_bottom() {
        for lines in 0..=10 {
            let offset = terminal_handle_y(lines, 10, px(18.));
            assert_eq!(terminal_lines_from_handle(offset, px(18.), 10), lines);
        }
    }

    #[test]
    fn terminal_direction_handle_top_and_bottom_map_to_content() {
        assert_eq!(terminal_lines_from_handle(px(0.), px(18.), 10), 10);
        assert_eq!(terminal_lines_from_handle(px(-180.), px(18.), 10), 0);
    }

    #[test]
    fn terminal_direction_history_growth_requires_reanchor() {
        let old_bottom = terminal_handle_y(0, 10, px(18.));
        let new_bottom = terminal_handle_y(0, 20, px(18.));
        assert_eq!(old_bottom, px(-180.));
        assert_eq!(new_bottom, px(-360.));
        assert_eq!(terminal_lines_from_handle(old_bottom, px(18.), 20), 10);
    }

    #[test]
    fn terminal_direction_bad_cell_height_returns_bottom() {
        assert_eq!(terminal_lines_from_handle(px(-180.), px(0.), 10), 0);
        assert_eq!(terminal_lines_from_handle(px(-180.), px(-18.), 10), 0);
    }

    #[test]
    fn terminal_resize_cursor_shrifts_by_removed_top_rows() {
        assert_eq!(terminal_resize_cursor_row(8, 10, 5), 3);
        assert_eq!(terminal_resize_cursor_row(2, 10, 5), 0);
        assert_eq!(terminal_resize_cursor_row(9, 10, 3), 2);
    }

    #[test]
    fn terminal_resize_cursor_growth_keeps_content_row() {
        assert_eq!(terminal_resize_cursor_row(3, 5, 10), 3);
        assert_eq!(terminal_resize_cursor_row(0, 5, 10), 0);
    }

    #[test]
    fn terminal_resize_cursor_sequence_stays_on_content() {
        let row = terminal_resize_cursor_row(8, 10, 5);
        assert_eq!(row, 3);
        let row = terminal_resize_cursor_row(row, 5, 8);
        assert_eq!(row, 3);
        let row = terminal_resize_cursor_row(row, 8, 4);
        assert_eq!(row, 0);
        let row = terminal_resize_cursor_row(row, 4, 6);
        assert_eq!(row, 0);
    }

    #[test]
    fn terminal_resize_cursor_zero_rows_is_safe() {
        assert_eq!(terminal_resize_cursor_row(8, 10, 0), 0);
    }

    #[test]
    fn terminal_resize_cursor_preserves_absolute_row_when_not_removed() {
        for old_rows in 1usize..=12 {
            for new_rows in 1usize..=12 {
                let removed = old_rows.saturating_sub(new_rows);
                for cur_row in removed..old_rows {
                    let resized = terminal_resize_cursor_row(cur_row, old_rows, new_rows);
                    assert_eq!(removed + resized, cur_row);
                }
            }
        }
    }

    #[test]
    fn terminal_resize_viewport_keeps_cursor_visible() {
        // 光标上方行足够：保持原有「顶部滚入 history」口径。
        assert_eq!(terminal_resize_viewport(10, 5, 8), (5, 0, 3));
        // 光标上方行不足：顶部只移到光标行，其余从底部裁掉，光标留在顶行。
        assert_eq!(terminal_resize_viewport(40, 3, 28), (28, 9, 0));
        assert_eq!(terminal_resize_viewport(10, 2, 1), (1, 7, 0));
        // 扩大：不裁行，只把光标夹回合法范围。
        assert_eq!(terminal_resize_viewport(3, 40, 2), (0, 0, 2));
        assert_eq!(terminal_resize_viewport(3, 40, 99), (0, 0, 39));
    }

    #[test]
    fn terminal_resize_viewport_matrix_keeps_cursor_inside_screen() {
        for old_rows in 1usize..=16 {
            for new_rows in 1usize..=16 {
                for cur_row in 0..old_rows {
                    let (top_remove, bottom_remove, next_row) =
                        terminal_resize_viewport(old_rows, new_rows, cur_row);
                    assert_eq!(
                        top_remove + bottom_remove,
                        old_rows.saturating_sub(new_rows)
                    );
                    assert!(next_row < new_rows.max(1));
                    assert!(top_remove <= cur_row);
                    assert_eq!(cur_row - top_remove, next_row);
                }
            }
        }
    }

    #[test]
    fn terminal_fit_gate_blocks_drag_transient() {
        // 5×3 的拖动瞬态被 20×5 闸门拦住，既不本地 reflow，也不通知宿主。
        assert_eq!(
            terminal_fit_gate(px(40.), px(54.), px(8.), px(18.), (20, 5)),
            None
        );
        assert_eq!(
            terminal_fit_gate(px(1280.), px(720.), px(8.), px(18.), (20, 5)),
            Some((160, 40))
        );
        // 默认 1×1 只挡未布局完成的零尺寸。
        assert_eq!(
            terminal_fit_gate(px(4.), px(4.), px(8.), px(18.), (1, 1)),
            Some((1, 1))
        );
        assert_eq!(
            terminal_fit_gate(px(0.), px(720.), px(8.), px(18.), (1, 1)),
            None
        );
    }

    #[test]
    fn terminal_reflow_cursor_history_maps_to_screen_top() {
        assert_eq!(terminal_reflow_cursor((720, 4), 722, 40, 160), (0, 4));
        assert_eq!(terminal_reflow_cursor((730, 4), 722, 40, 160), (8, 4));
        assert_eq!(terminal_reflow_cursor((1000, 999), 900, 40, 160), (39, 160));
    }

    #[test]
    fn terminal_reflow_roundtrip_preserves_trailing_blank_pen() {
        let mut line = test_line("abc", 8, false);
        for cell in &mut line.cells[3..] {
            cell.pen.bg = TermColor::Indexed(4);
        }
        let before = line.clone();

        let shrunk = reflow_lines(&[line], 8, 4);
        let expanded = reflow_lines(&shrunk.lines, 4, 8);

        assert_eq!(expanded.lines.len(), 1);
        assert_eq!(expanded.lines[0].wrapped, before.wrapped);
        assert_eq!(expanded.lines[0].cells.len(), before.cells.len());
        for (actual, expected) in expanded.lines[0].cells.iter().zip(before.cells.iter()) {
            assert_eq!(actual.ch, expected.ch);
            assert!(actual.pen == expected.pen, "pen mismatch: {:?}", actual.ch);
            assert_eq!(actual.link, expected.link);
        }
    }

    fn test_line(text: &str, cols: usize, wrapped: bool) -> Line {
        let mut cells: Vec<Cell> = text
            .chars()
            .map(|ch| Cell {
                ch,
                ..Cell::default()
            })
            .collect();
        cells.resize(cols, Cell::default());
        Line { cells, wrapped }
    }

    fn line_text(line: &Line) -> String {
        line.cells
            .iter()
            .map(|cell| cell.ch)
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    #[test]
    fn terminal_reflow_shrink_and_expand_soft_wrap() {
        let old = vec![test_line("abc", 6, false), test_line("def", 6, true)];
        let shrunk = reflow_lines(&old, 6, 4);
        assert_eq!(
            shrunk.lines.iter().map(line_text).collect::<Vec<_>>(),
            ["abcd", "ef"]
        );
        assert!(!shrunk.lines[0].wrapped);
        assert!(shrunk.lines[1].wrapped);
        let expanded = reflow_lines(&shrunk.lines, 4, 6);
        assert_eq!(
            expanded.lines.iter().map(line_text).collect::<Vec<_>>(),
            ["abcdef"]
        );
    }

    #[test]
    fn terminal_reflow_hard_break_is_not_merged() {
        let old = vec![test_line("abc", 6, false), test_line("def", 6, false)];
        let expanded = reflow_lines(&old, 6, 10);
        assert_eq!(
            expanded.lines.iter().map(line_text).collect::<Vec<_>>(),
            ["abc", "def"]
        );
    }

    #[test]
    fn terminal_reflow_preserves_cell_style() {
        let mut styled = Cell::default();
        styled.ch = 'x';
        styled.pen.bold = true;
        styled.pen.fg = super::TermColor::Rgb(1, 2, 3);
        styled.link = Some(7);
        let old = vec![Line {
            cells: vec![styled.clone(), Cell::default()],
            wrapped: false,
        }];
        let result = reflow_lines(&old, 2, 1);
        let actual = &result.lines[0].cells[0];
        assert_eq!(actual.ch, styled.ch);
        assert!(actual.pen == styled.pen);
        assert_eq!(actual.link, styled.link);
    }

    #[test]
    fn terminal_reflow_wide_char_is_not_split() {
        let old = vec![Line {
            cells: vec![
                Cell {
                    ch: 'a',
                    ..Cell::default()
                },
                Cell {
                    ch: 'b',
                    ..Cell::default()
                },
                Cell {
                    ch: '界',
                    ..Cell::default()
                },
                Cell::default(),
                Cell {
                    ch: 'c',
                    ..Cell::default()
                },
            ],
            wrapped: false,
        }];
        let result = reflow_lines(&old, 5, 3);
        assert_eq!(result.lines.len(), 2);
        assert_eq!(line_text(&result.lines[0]), "ab");
        assert_eq!(result.lines[1].cells[0].ch, '界');
        assert_eq!(result.lines[1].cells[2].ch, 'c');
    }

    #[test]
    fn terminal_reflow_maps_cursor_and_selection_positions() {
        let old = vec![test_line("abc", 6, false), test_line("def", 6, true)];
        let result = reflow_lines(&old, 6, 4);
        assert_eq!(map_reflow_pos(&result.positions, (0, 2)), (0, 2));
        assert_eq!(map_reflow_pos(&result.positions, (1, 1)), (1, 0));
        assert_eq!(map_reflow_pos(&result.positions, (1, 3)), (1, 2));
    }

    #[test]
    fn terminal_reflow_blank_hard_line_is_preserved() {
        let old = vec![test_line("", 4, false), test_line("a", 4, false)];
        let result = reflow_lines(&old, 4, 8);
        assert_eq!(result.lines.len(), 2);
        assert_eq!(line_text(&result.lines[0]), "");
        assert_eq!(line_text(&result.lines[1]), "a");
    }

    #[test]
    fn terminal_reflow_maps_blank_rows_after_wrapped_content() {
        let old = vec![
            test_line("abcdefgh", 10, false),
            test_line("", 10, false),
            test_line("", 10, false),
        ];
        let result = reflow_lines(&old, 10, 4);
        assert_eq!(
            result.lines.iter().map(line_text).collect::<Vec<_>>(),
            ["abcd", "efgh", "", ""]
        );
        assert_eq!(map_reflow_pos(&result.positions, (1, 1)), (2, 0));
        assert_eq!(map_reflow_pos(&result.positions, (2, 1)), (3, 0));
    }
}
