//! Markdown 渲染：GFM 常用子集 → gpui 元素。
//!
//! 支持：段落 / 标题 / 有序与无序列表（可嵌套、含任务列表勾选框）/ 引用 / 分隔线 /
//! 表格（含 `:---:` 列对齐）/ 围栏代码块（交给 [`CodeBlock`]，可附带动作按钮），以及
//! 粗体 / 斜体 / 删除线 / 行内代码 / 链接（点击回调）/ 裸 URL 自动链接（GFM）。
//!
//! 解析器为零依赖的手写实现，只覆盖对话场景常见语法；未支持的语法按纯文本降级。

use std::ops::Range;
use std::rc::Rc;

use gpui::{
    AnyElement, App, Element, ElementId, Font, Hitbox, Hsla, InteractiveElement, InteractiveText,
    IntoElement, ParentElement, RenderOnce, Role, SharedString, StrikethroughStyle, Styled,
    StyledText, TextAlign, TextRun, UnderlineStyle, Window, accesskit, div, px,
};

use crate::{CodeAction, CodeBlock, a11y, theme};

/// Markdown 文本。
///
/// ```ignore
/// Markdown::new("# 标题\n\n正文 **加粗** 与 [链接](https://example.com)")
///     .on_link(|url, _, _| { /* 打开外部浏览器 */ })
///     .code_action(CodeAction::new("填入", |code, _, _| { /* ... */ }))
/// ```
#[derive(IntoElement)]
pub struct Markdown {
    id: Option<ElementId>,
    source: SharedString,
    on_link: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    code_actions: Vec<CodeAction>,
    code_copy_labels: Option<(SharedString, SharedString)>,
    compact: bool,
    text_size: Option<f32>,
}

impl Markdown {
    pub fn new(source: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            source: source.into(),
            on_link: None,
            code_actions: Vec::new(),
            code_copy_labels: None,
            compact: false,
            text_size: None,
        }
    }

    /// 文档及派生语义节点的稳定 id；未提供时按源文本分配标识。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 紧凑排版：标题降到聊天气泡尺度，适合聊天面板等窄容器。
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    /// 覆盖正文字号：标题同比缩放、额外行距不变，围栏代码块按同一比例跟随
    /// （保住「代码比正文小一档」的关系）；不调用则跟随主题度量。
    /// Markdown 是 RenderOnce：宿主每帧带新值重建即活更新，无需 setter。
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = Some(size);
        self
    }

    /// 链接被点击时回调（参数依次为 URL、窗口、应用）
    pub fn on_link(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_link = Some(Rc::new(listener));
        self
    }

    /// 为渲染出的每个围栏代码块追加一个动作按钮
    pub fn code_action(mut self, action: CodeAction) -> Self {
        self.code_actions.push(action);
        self
    }

    /// 围栏代码块复制按钮的常态 / 已复制态文案（多语言宿主传入译文；默认中文，零回归）。
    pub fn code_copy_labels(
        mut self,
        copy: impl Into<SharedString>,
        copied: impl Into<SharedString>,
    ) -> Self {
        self.code_copy_labels = Some((copy.into(), copied.into()));
        self
    }
}

impl RenderOnce for Markdown {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let base = theme::with(if self.compact {
            crate::theme::metric::markdown_compact_metric
        } else {
            crate::theme::metric::markdown_metric
        });
        let base_font_size = base.font_size;
        let m = match self.text_size {
            Some(size) => base.with_font_size(size),
            None => base,
        };
        let code_text_size = self.text_size.map(|size| {
            theme::with(crate::theme::metric::code_block_metric)
                .following_markdown(base_font_size, size)
                .font_size
        });
        let font = window.text_style().font();
        let colors = RunColors {
            text: theme::text(),
            link: theme::link(),
            code_bg: theme::fill_secondary(),
        };
        let blocks = parse(&self.source);
        let root_id: SharedString = self
            .id
            .as_ref()
            .map(|id| format!("markdown-document-{id}"))
            .unwrap_or_else(|| format!("markdown-document-{:p}", self.source.as_ptr()))
            .into();
        let mut ctx = Ctx {
            root_id: root_id.clone(),
            m: &m,
            font: &font,
            colors,
            on_link: self.on_link.as_ref(),
            actions: &self.code_actions,
            copy_labels: self.code_copy_labels.as_ref(),
            code_text_size,
            seq: 0,
        };

        let semantics =
            a11y::Semantics::new(Role::Document, document_label(&blocks)).id(root_id.clone());
        let mut container = semantics.apply(
            div()
                .id(ElementId::Name(root_id.into()))
                .flex()
                .flex_col()
                .w_full()
                .gap(px(m.block_gap))
                .text_size(px(m.font_size))
                .line_height(px(m.line_height))
                .text_color(theme::text()),
        );
        for block in &blocks {
            container = container.child(ctx.block(block, theme::text()));
        }
        container
    }
}

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Inline {
    text: String,
    bold: bool,
    italic: bool,
    strike: bool,
    code: bool,
    link: Option<String>,
}

#[derive(Debug)]
enum Block {
    Paragraph(Vec<Inline>),
    Heading {
        level: u8,
        inlines: Vec<Inline>,
    },
    Code {
        language: Option<SharedString>,
        code: SharedString,
    },
    List {
        ordered: bool,
        start: u64,
        items: Vec<ListItem>,
    },
    Quote(Vec<Block>),
    Rule,
    Table {
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
        aligns: Vec<TableAlign>,
    },
}

/// 列表项：`task` 为 GFM 任务列表勾选状态（`None` 表示普通项）
#[derive(Debug)]
struct ListItem {
    task: Option<bool>,
    blocks: Vec<Block>,
}

/// GFM 表格列对齐（分隔行 `:---` / `:---:` / `---:`）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TableAlign {
    #[default]
    Left,
    Center,
    Right,
}

// ---------------------------------------------------------------------------
// 块级解析
// ---------------------------------------------------------------------------

fn parse(source: &str) -> Vec<Block> {
    let lines: Vec<String> = source
        .lines()
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect();
    parse_blocks(&lines)
}

fn parse_blocks(lines: &[String]) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        if let Some(fence) = fence_open(line) {
            let mut code: Vec<&str> = Vec::new();
            let mut j = i + 1;
            while j < lines.len() {
                if is_fence_close(&lines[j], fence.ch, fence.count) {
                    j += 1;
                    break;
                }
                code.push(lines[j].as_str());
                j += 1;
            }
            let code = code.join("\n");
            let language = fence
                .info
                .split_whitespace()
                .next()
                .map(|info| SharedString::from(info.to_string()));
            blocks.push(Block::Code {
                language,
                code: SharedString::from(code.trim_end_matches('\n').to_string()),
            });
            i = j;
            continue;
        }

        if let Some(level) = heading_level(line) {
            let text = line
                .trim_start()
                .trim_start_matches('#')
                .trim()
                .trim_end_matches('#')
                .trim_end();
            let inlines = parse_inline(text);
            if !inlines.is_empty() {
                blocks.push(Block::Heading { level, inlines });
            }
            i += 1;
            continue;
        }

        if is_rule(line) {
            blocks.push(Block::Rule);
            i += 1;
            continue;
        }

        if line.trim_start().starts_with('>') {
            let mut inner = Vec::new();
            let mut j = i;
            while j < lines.len() {
                let trimmed = lines[j].trim_start();
                if let Some(rest) = trimmed.strip_prefix('>') {
                    inner.push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
                    j += 1;
                } else if lines[j].trim().is_empty()
                    && j + 1 < lines.len()
                    && lines[j + 1].trim_start().starts_with('>')
                {
                    inner.push(String::new());
                    j += 1;
                } else {
                    break;
                }
            }
            let inner_blocks = parse_blocks(&inner);
            if !inner_blocks.is_empty() {
                blocks.push(Block::Quote(inner_blocks));
            }
            i = j;
            continue;
        }

        if let Some(marker) = list_marker(line) {
            let base = marker.indent;
            let ordered = marker.ordered;
            let start = marker.start;
            let mut items: Vec<ListItem> = Vec::new();
            let mut j = i;
            while j < lines.len() {
                let Some(item_marker) = list_marker(&lines[j]) else {
                    break;
                };
                if item_marker.indent != base || item_marker.ordered != ordered {
                    break;
                }
                let content_col = item_marker.content_start;
                let mut item_lines = vec![lines[j][content_col..].to_string()];
                j += 1;
                while j < lines.len() {
                    let current = &lines[j];
                    if current.trim().is_empty() {
                        let mut k = j;
                        while k < lines.len() && lines[k].trim().is_empty() {
                            k += 1;
                        }
                        if k < lines.len() {
                            let sibling = list_marker(&lines[k])
                                .map(|nm| nm.indent == base && nm.ordered == ordered)
                                .unwrap_or(false);
                            if leading_spaces(&lines[k]) > base {
                                item_lines.push(String::new());
                                j += 1;
                                continue;
                            }
                            if sibling {
                                j = k;
                                break;
                            }
                        }
                        j = k;
                        break;
                    }
                    if let Some(nm) = list_marker(current)
                        && nm.indent == base
                    {
                        break;
                    }
                    let indent = leading_spaces(current);
                    if indent <= base {
                        break;
                    }
                    item_lines.push(current[content_col.min(indent)..].to_string());
                    j += 1;
                }
                let (task, rest) = task_marker(&item_lines[0]);
                if task.is_some() {
                    item_lines[0] = rest.to_string();
                }
                items.push(ListItem {
                    task,
                    blocks: parse_blocks(&item_lines),
                });
                if j >= lines.len() {
                    break;
                }
                match list_marker(&lines[j]) {
                    Some(nm) if nm.indent == base && nm.ordered == ordered => {}
                    _ => break,
                }
            }
            blocks.push(Block::List {
                ordered,
                start,
                items,
            });
            i = j;
            continue;
        }

        if line.contains('|') && i + 1 < lines.len() && is_table_delimiter(&lines[i + 1]) {
            let aligns = table_aligns(&lines[i + 1]);
            let headers = split_cells(line)
                .into_iter()
                .map(|cell| parse_inline(&cell))
                .collect();
            let mut rows = Vec::new();
            let mut j = i + 2;
            while j < lines.len() {
                let current = &lines[j];
                if current.trim().is_empty() || !current.contains('|') {
                    break;
                }
                rows.push(
                    split_cells(current)
                        .into_iter()
                        .map(|cell| parse_inline(&cell))
                        .collect(),
                );
                j += 1;
            }
            blocks.push(Block::Table {
                headers,
                rows,
                aligns,
            });
            i = j;
            continue;
        }

        let mut paragraph = vec![line.clone()];
        let mut j = i + 1;
        while j < lines.len() {
            let current = &lines[j];
            if current.trim().is_empty()
                || fence_open(current).is_some()
                || heading_level(current).is_some()
                || is_rule(current)
                || current.trim_start().starts_with('>')
                || list_marker(current).is_some()
            {
                break;
            }
            if current.contains('|') && j + 1 < lines.len() && is_table_delimiter(&lines[j + 1]) {
                break;
            }
            paragraph.push(current.clone());
            j += 1;
        }
        let inlines = parse_inline(&paragraph.join("\n"));
        if !inlines.is_empty() {
            blocks.push(Block::Paragraph(inlines));
        }
        i = j;
    }
    blocks
}

struct Fence {
    ch: char,
    count: usize,
    info: String,
}

fn fence_open(line: &str) -> Option<Fence> {
    let trimmed = line.trim_start();
    let ch = trimmed.chars().next()?;
    if ch != '`' && ch != '~' {
        return None;
    }
    let count = trimmed.chars().take_while(|&c| c == ch).count();
    if count < 3 {
        return None;
    }
    Some(Fence {
        ch,
        count,
        info: trimmed[count..].trim().to_string(),
    })
}

fn is_fence_close(line: &str, ch: char, count: usize) -> bool {
    let trimmed = line.trim_start();
    let run = trimmed.chars().take_while(|&c| c == ch).count();
    run >= count && trimmed[run..].trim().is_empty()
}

fn heading_level(line: &str) -> Option<u8> {
    let trimmed = line.trim_start();
    let hashes = trimmed.bytes().take_while(|&b| b == b'#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
        Some(hashes as u8)
    } else {
        None
    }
}

fn is_rule(line: &str) -> bool {
    let trimmed = line.trim();
    let Some(first) = trimmed.chars().next() else {
        return false;
    };
    if !matches!(first, '*' | '-' | '_') {
        return false;
    }
    let mut count = 0;
    for ch in trimmed.chars() {
        if ch == first {
            count += 1;
        } else if ch != ' ' && ch != '\t' {
            return false;
        }
    }
    count >= 3
}

struct ListMarker {
    indent: usize,
    ordered: bool,
    start: u64,
    content_start: usize,
}

fn leading_spaces(line: &str) -> usize {
    let mut columns = 0;
    for ch in line.chars() {
        match ch {
            ' ' => columns += 1,
            '\t' => columns += 4,
            _ => break,
        }
    }
    columns
}

fn list_marker(line: &str) -> Option<ListMarker> {
    let mut indent = 0;
    let mut start_byte = None;
    for (ix, ch) in line.char_indices() {
        match ch {
            ' ' => indent += 1,
            '\t' => indent += 4,
            _ => {
                start_byte = Some(ix);
                break;
            }
        }
    }
    let start_byte = start_byte?;
    let rest = &line[start_byte..];
    let first = rest.chars().next()?;
    let (ordered, start, marker_len) = match first {
        '-' | '*' | '+' => (false, 1, 1),
        '0'..='9' => {
            let digits = rest.chars().take_while(|c| c.is_ascii_digit()).count();
            let after = &rest[digits..];
            if !(after.starts_with('.') || after.starts_with(')')) {
                return None;
            }
            (true, rest[..digits].parse().ok()?, digits + 1)
        }
        _ => return None,
    };
    let after_marker = &rest[marker_len..];
    let spaces = after_marker
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .count();
    if spaces == 0 {
        return None;
    }
    Some(ListMarker {
        indent,
        ordered,
        start,
        content_start: start_byte + marker_len + spaces,
    })
}

/// GFM 任务列表标记：行首 `[ ]` / `[x]` / `[X]` 后须跟空白或行尾；
/// 命中时返回勾选状态与去掉标记后的剩余文本。
fn task_marker(line: &str) -> (Option<bool>, &str) {
    let trimmed = line.trim_start();
    let checked = match trimmed.get(..3) {
        Some("[ ]") => false,
        Some("[x]" | "[X]") => true,
        _ => return (None, line),
    };
    let rest = &trimmed[3..];
    if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') {
        (Some(checked), rest.trim_start())
    } else {
        (None, line)
    }
}

fn split_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let trimmed = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let trimmed = trimmed.strip_suffix('|').unwrap_or(trimmed);
    let mut cells = Vec::new();
    let mut buf = String::new();
    let mut escaped = false;
    for ch in trimmed.chars() {
        if escaped {
            buf.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '|' => {
                cells.push(buf.trim().to_string());
                buf.clear();
            }
            _ => buf.push(ch),
        }
    }
    cells.push(buf.trim().to_string());
    cells
}

fn is_table_delimiter(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.contains('-') {
        return false;
    }
    let cells = split_cells(trimmed);
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let cell = cell.trim();
            let cell = cell.trim_start_matches(':').trim_end_matches(':');
            !cell.is_empty() && cell.chars().all(|ch| ch == '-')
        })
}

fn table_aligns(line: &str) -> Vec<TableAlign> {
    split_cells(line)
        .into_iter()
        .map(|cell| match (cell.starts_with(':'), cell.ends_with(':')) {
            (true, true) => TableAlign::Center,
            (false, true) => TableAlign::Right,
            _ => TableAlign::Left,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 行内解析
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
struct InlineAttrs {
    bold: bool,
    italic: bool,
    strike: bool,
    link: Option<String>,
}

fn parse_inline(text: &str) -> Vec<Inline> {
    let mut out = Vec::new();
    parse_inline_into(&mut out, text, InlineAttrs::default());
    out
}

fn push_text(out: &mut Vec<Inline>, text: &str, attrs: &InlineAttrs, code: bool) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = out.last_mut()
        && last.code == code
        && last.bold == attrs.bold
        && last.italic == attrs.italic
        && last.strike == attrs.strike
        && last.link == attrs.link
    {
        last.text.push_str(text);
        return;
    }
    out.push(Inline {
        text: text.to_string(),
        bold: attrs.bold,
        italic: attrs.italic,
        strike: attrs.strike,
        code,
        link: attrs.link.clone(),
    });
}

fn parse_inline_into(out: &mut Vec<Inline>, text: &str, attrs: InlineAttrs) {
    let mut i = 0;
    let mut buf = String::new();
    while i < text.len() {
        let rest = &text[i..];

        if rest.starts_with('\\') && rest.len() > 1 {
            let ch = rest[1..].chars().next().unwrap();
            buf.push(ch);
            i += 1 + ch.len_utf8();
            continue;
        }

        if rest.starts_with('`') {
            let ticks = rest.bytes().take_while(|b| *b == b'`').count();
            let ticks = ticks.max(1);
            let fence = "`".repeat(ticks);
            if let Some(offset) = rest[ticks..].find(&fence) {
                let inner = &rest[ticks..ticks + offset];
                push_text(out, &buf, &attrs, false);
                buf.clear();
                push_text(out, inner.trim(), &attrs, true);
                i += ticks + offset + ticks;
                continue;
            }
        }

        if rest.starts_with("![") || rest.starts_with('[') {
            let label_start = if rest.starts_with("![") { i + 2 } else { i + 1 };
            if let Some((label, url, consumed)) = parse_link(&text[label_start..]) {
                push_text(out, &buf, &attrs, false);
                buf.clear();
                let mut inner_attrs = attrs.clone();
                inner_attrs.link = Some(url);
                parse_inline_into(out, label, inner_attrs);
                i = label_start + consumed;
                continue;
            }
        }

        if rest.starts_with('<')
            && let Some(end) = rest.find('>')
        {
            let inner = &rest[1..end];
            if inner.contains("://") {
                push_text(out, &buf, &attrs, false);
                buf.clear();
                let mut inner_attrs = attrs.clone();
                inner_attrs.link = Some(inner.to_string());
                parse_inline_into(out, inner, inner_attrs);
                i += end + 1;
                continue;
            }
        }

        if let Some((label, href, consumed)) = autolink_at(rest, text[..i].chars().next_back()) {
            push_text(out, &buf, &attrs, false);
            buf.clear();
            let mut link_attrs = attrs.clone();
            link_attrs.link = Some(href);
            push_text(out, &label, &link_attrs, false);
            i += consumed;
            continue;
        }

        if let Some((kind, inner_len, delim_len)) = emphasis_at(rest) {
            push_text(out, &buf, &attrs, false);
            buf.clear();
            let mut inner_attrs = attrs.clone();
            match kind {
                0 => inner_attrs.bold = true,
                1 => inner_attrs.italic = true,
                _ => inner_attrs.strike = true,
            }
            let inner = &rest[delim_len..delim_len + inner_len];
            parse_inline_into(out, inner, inner_attrs);
            i += delim_len + inner_len + delim_len;
            continue;
        }

        let ch = rest.chars().next().unwrap();
        buf.push(ch);
        i += ch.len_utf8();
    }
    push_text(out, &buf, &attrs, false);
}

/// 返回 (样式类型, 内容字节长度, 定界符长度)
/// GFM 扩展自动链接：裸 `http://` / `https://` / `ftp://` / `www.` URL。
/// 前一字符须为行首 / 空白 / `*_~(`；返回 (显示文本, href, 消耗字节数)。
fn autolink_at(rest: &str, prev: Option<char>) -> Option<(String, String, usize)> {
    if let Some(prev) = prev
        && !prev.is_whitespace()
        && !matches!(prev, '*' | '_' | '~' | '(')
    {
        return None;
    }
    let (prefix, bare_www) = if rest.starts_with("https://") {
        ("https://", false)
    } else if rest.starts_with("http://") {
        ("http://", false)
    } else if rest.starts_with("ftp://") {
        ("ftp://", false)
    } else if rest.starts_with("www.") {
        ("www.", true)
    } else {
        return None;
    };
    let body = &rest[prefix.len()..];
    let end = body
        .char_indices()
        .find(|(_, ch)| !is_url_char(*ch))
        .map(|(ix, _)| ix)
        .unwrap_or(body.len());
    let mut url =
        body[..end].trim_end_matches(['?', '!', '.', ',', ':', ';', '*', '_', '~', '"', '\'']);
    while url.ends_with(')') {
        let opens = url.chars().filter(|ch| *ch == '(').count();
        let closes = url.chars().filter(|ch| *ch == ')').count();
        if closes <= opens {
            break;
        }
        url = &url[..url.len() - 1];
    }
    let domain_end = url.find(['/', ':', '?', '#']).unwrap_or(url.len());
    let domain = &url[..domain_end];
    let segments: Vec<&str> = domain.split('.').collect();
    if segments.len() < 2
        || segments.iter().any(|seg| seg.is_empty())
        || segments[segments.len() - 2..]
            .iter()
            .any(|seg| seg.contains('_'))
    {
        return None;
    }
    let scheme = if bare_www { "http://" } else { "" };
    Some((
        format!("{prefix}{url}"),
        format!("{scheme}{prefix}{url}"),
        prefix.len() + url.len(),
    ))
}

/// GFM 自动链接 path 允许的 ASCII 字符（空白 / `<` / 非 ASCII 均终止）
fn is_url_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            '-' | '.'
                | '_'
                | '~'
                | ':'
                | '/'
                | '?'
                | '#'
                | '['
                | ']'
                | '@'
                | '!'
                | '$'
                | '&'
                | '\''
                | '('
                | ')'
                | '*'
                | '+'
                | ','
                | ';'
                | '='
                | '%'
        )
}

fn emphasis_at(rest: &str) -> Option<(u8, usize, usize)> {
    for (delim, kind) in [("**", 0u8), ("__", 0), ("~~", 2), ("*", 1), ("_", 1)] {
        if !rest.starts_with(delim) {
            continue;
        }
        if delim.len() == 1 && rest[delim.len()..].starts_with(delim) {
            continue;
        }
        let after = &rest[delim.len()..];
        if after.is_empty() || after.starts_with(char::is_whitespace) {
            continue;
        }
        if let Some(inner_len) = find_closing(after, delim) {
            return Some((kind, inner_len, delim.len()));
        }
    }
    None
}

fn find_closing(text: &str, delim: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(pos) = text[from..].find(delim) {
        let abs = from + pos;
        if abs > 0 {
            let prev = text[..abs].chars().next_back().unwrap();
            if !prev.is_whitespace() {
                return Some(abs);
            }
        }
        from = abs + delim.len();
    }
    None
}

/// 解析 `label](url)`，返回 (label, url, 已消费字节数)
fn parse_link(text: &str) -> Option<(&str, String, usize)> {
    let close = find_unescaped(text, ']')?;
    let after = &text[close + 1..];
    if !after.starts_with('(') {
        return None;
    }
    let end = find_unescaped(&after[1..], ')')?;
    let dest = after[1..1 + end].trim();
    let url = dest
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c| c == '"' || c == '\'');
    if url.is_empty() {
        return None;
    }
    Some((&text[..close], url.to_string(), close + 2 + end + 1))
}

fn find_unescaped(text: &str, target: char) -> Option<usize> {
    let mut escaped = false;
    for (ix, ch) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            _ if ch == target => return Some(ix),
            _ => {}
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 渲染
// ---------------------------------------------------------------------------

fn inline_text(inlines: &[Inline]) -> String {
    inlines.iter().map(|inline| inline.text.as_str()).collect()
}

fn block_text(block: &Block) -> String {
    match block {
        Block::Paragraph(inlines) | Block::Heading { inlines, .. } => inline_text(inlines),
        Block::Code { language, .. } => language
            .as_ref()
            .map(|language| format!("{language} 代码"))
            .unwrap_or_else(|| "代码".to_string()),
        Block::List { items, .. } => items
            .iter()
            .map(|item| blocks_text(&item.blocks))
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("；"),
        Block::Quote(inner) => blocks_text(inner),
        Block::Rule => String::new(),
        Block::Table { headers, rows, .. } => {
            let mut cells = headers
                .iter()
                .map(|cell| inline_text(cell))
                .collect::<Vec<_>>();
            cells.extend(
                rows.iter()
                    .flatten()
                    .map(|cell| inline_text(cell))
                    .filter(|text| !text.is_empty()),
            );
            cells.join("；")
        }
    }
}

fn blocks_text(blocks: &[Block]) -> String {
    blocks
        .iter()
        .map(block_text)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn document_label(blocks: &[Block]) -> SharedString {
    blocks
        .iter()
        .find_map(|block| match block {
            Block::Heading { inlines, .. } if !inlines.is_empty() => Some(inline_text(inlines)),
            _ => None,
        })
        .unwrap_or_else(|| "Markdown 文档".to_string())
        .into()
}

#[derive(Clone, Copy)]
struct RunColors {
    text: Hsla,
    link: Hsla,
    code_bg: Hsla,
}

fn build_runs(
    inlines: &[Inline],
    base_font: &Font,
    force_bold: bool,
    m: &crate::theme::metric::MarkdownMetric,
    colors: RunColors,
) -> (
    SharedString,
    Vec<TextRun>,
    Vec<(Range<usize>, SharedString)>,
) {
    let mut text = String::new();
    let mut runs = Vec::new();
    let mut links = Vec::new();
    for inline in inlines {
        if inline.text.is_empty() {
            continue;
        }
        let start = text.len();
        text.push_str(&inline.text);
        let end = text.len();

        let mut font = base_font.clone();
        if inline.code {
            font.family = crate::theme::mono_family_or("Consolas").into();
        }
        if inline.bold || force_bold {
            font = font.bold();
        }
        if inline.italic {
            font = font.italic();
        }
        let color = if inline.link.is_some() {
            colors.link
        } else {
            colors.text
        };
        let background_color = inline.code.then_some(colors.code_bg);
        let underline = inline.link.is_some().then_some(UnderlineStyle {
            thickness: px(m.link_thickness),
            color: Some(colors.link),
            wavy: false,
        });
        let strikethrough = inline.strike.then_some(StrikethroughStyle {
            thickness: px(m.link_thickness),
            color: Some(color),
        });
        runs.push(TextRun {
            len: end - start,
            font,
            color,
            background_color,
            underline,
            strikethrough,
        });
        if let Some(url) = &inline.link {
            links.push((start..end, SharedString::from(url.to_string())));
        }
    }
    (SharedString::from(text), runs, links)
}

struct Ctx<'a> {
    root_id: SharedString,
    m: &'a crate::theme::metric::MarkdownMetric,
    font: &'a Font,
    colors: RunColors,
    on_link: Option<&'a Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    actions: &'a [CodeAction],
    copy_labels: Option<&'a (SharedString, SharedString)>,
    code_text_size: Option<f32>,
    seq: usize,
}

impl Ctx<'_> {
    fn next_id(&mut self, kind: &str) -> SharedString {
        self.seq += 1;
        SharedString::from(format!("{}-{kind}-{}", self.root_id, self.seq))
    }

    fn text(&mut self, inlines: &[Inline], color: Hsla, force_bold: bool) -> AnyElement {
        let colors = RunColors {
            text: color,
            ..self.colors
        };
        let (text, runs, links) = build_runs(inlines, self.font, force_bold, self.m, colors);
        if text.is_empty() {
            return div().into_any_element();
        }
        let styled = StyledText::new(text.clone()).with_runs(runs);
        let Some(on_link) = self.on_link else {
            return styled.into_any_element();
        };
        if links.is_empty() {
            return styled.into_any_element();
        }
        let ranges: Vec<Range<usize>> = links.iter().map(|(range, _)| range.clone()).collect();
        let urls: Vec<SharedString> = links.iter().map(|(_, url)| url.clone()).collect();
        let id = self.next_id("text");
        let on_link = on_link.clone();
        let mouse_urls = urls.clone();
        let link_metadata = links
            .into_iter()
            .enumerate()
            .map(|(index, (range, url))| MarkdownLink {
                id: format!("{id}-link-{index}"),
                label: text[range.clone()].to_string(),
                url,
                on_link: on_link.clone(),
            })
            .collect();
        AccessibleMarkdownText {
            id: ElementId::Name(id.clone().into()),
            text: InteractiveText::new(ElementId::Name(id.into()), styled).on_click(
                ranges,
                move |ix, window, cx| {
                    if let Some(url) = mouse_urls.get(ix) {
                        on_link(url, window, cx);
                    }
                },
            ),
            links: link_metadata,
        }
        .into_any_element()
    }

    fn blocks(&mut self, blocks: &[Block], color: Hsla) -> Vec<AnyElement> {
        blocks
            .iter()
            .map(|block| self.block(block, color))
            .collect()
    }

    fn block(&mut self, block: &Block, color: Hsla) -> AnyElement {
        match block {
            Block::Paragraph(inlines) => {
                let id = self.next_id("paragraph");
                a11y::Semantics::new(Role::Paragraph, inline_text(inlines))
                    .id(id.clone())
                    .apply(
                        div()
                            .id(ElementId::Name(id.into()))
                            .w_full()
                            .child(self.text(inlines, color, false)),
                    )
                    .into_any_element()
            }
            Block::Heading { level, inlines } => {
                let ix = ((*level).clamp(1, 6) - 1) as usize;
                let id = self.next_id("heading");
                a11y::Semantics::new(Role::Heading, inline_text(inlines))
                    .id(id.clone())
                    .level(*level as usize)
                    .apply(
                        div()
                            .id(ElementId::Name(id.into()))
                            .w_full()
                            .text_size(px(self.m.heading_sizes[ix]))
                            .line_height(px(self.m.heading_line_heights[ix]))
                            .text_color(theme::text_heading())
                            .child(self.text(inlines, theme::text_heading(), true)),
                    )
                    .into_any_element()
            }
            Block::Code { language, code } => {
                let id = self.next_id("code");
                let mut block = CodeBlock::new(code.clone()).id(id);
                if let Some(language) = language {
                    block = block.language(language.clone());
                }
                for action in self.actions {
                    block = block.action(action.clone());
                }
                if let Some((copy, copied)) = self.copy_labels {
                    block = block.copy_labels(copy.clone(), copied.clone());
                }
                if let Some(size) = self.code_text_size {
                    block = block.text_size(size);
                }
                block.into_any_element()
            }
            Block::List {
                ordered,
                start,
                items,
            } => {
                let id = self.next_id("list");
                let list_semantics = a11y::Semantics::new(
                    Role::List,
                    if *ordered {
                        "有序列表"
                    } else {
                        "无序列表"
                    },
                )
                .id(id.clone());
                let mut list = div()
                    .id(ElementId::Name(id.into()))
                    .flex()
                    .flex_col()
                    .w_full()
                    .gap(px(self.m.list_gap));
                for (ix, item) in items.iter().enumerate() {
                    let marker = if *ordered {
                        format!("{}.", start + ix as u64)
                    } else {
                        "•".to_string()
                    };
                    let marker_element = match item.task {
                        Some(checked) => {
                            let mut box_element = div()
                                .flex_shrink_0()
                                .mt(px((self.m.line_height - self.m.task_box_size) / 2.0))
                                .size(px(self.m.task_box_size))
                                .rounded_sm()
                                .border_1()
                                .flex()
                                .items_center()
                                .justify_center()
                                .line_height(px(self.m.task_box_size));
                            if checked {
                                box_element = box_element
                                    .bg(theme::primary())
                                    .border_color(theme::primary())
                                    .text_size(px(self.m.task_check_size))
                                    .text_color(theme::white())
                                    .child("✓");
                            } else {
                                box_element = box_element
                                    .bg(theme::bg_container())
                                    .border_color(theme::border());
                            }
                            box_element.into_any_element()
                        }
                        None => div()
                            .flex_shrink_0()
                            .w(px(self.m.bullet_width))
                            .text_color(theme::text_secondary())
                            .child(marker)
                            .into_any_element(),
                    };
                    let children = self.blocks(&item.blocks, color);
                    let item_id = self.next_id("list-item");
                    let mut semantics =
                        a11y::Semantics::new(Role::ListItem, blocks_text(&item.blocks))
                            .id(item_id.clone())
                            .position(ix + 1, items.len());
                    if let Some(checked) = item.task {
                        semantics = semantics.description(if checked {
                            "任务已完成"
                        } else {
                            "任务未完成"
                        });
                    }
                    list = list.child(
                        semantics.apply(
                            div()
                                .id(ElementId::Name(item_id.into()))
                                .flex()
                                .flex_row()
                                .gap(px(self.m.list_gap))
                                .child(marker_element)
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w(px(0.))
                                        .flex()
                                        .flex_col()
                                        .gap(px(self.m.list_gap))
                                        .children(children),
                                ),
                        ),
                    );
                }
                list_semantics.apply(list).into_any_element()
            }
            Block::Quote(inner) => {
                let children = self.blocks(inner, theme::text_secondary());
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .gap(px(self.m.list_gap))
                    .border_l(px(self.m.quote_border))
                    .border_color(theme::border())
                    .pl(px(self.m.quote_padding))
                    .children(children)
                    .into_any_element()
            }
            Block::Rule => div()
                .w_full()
                .h(px(self.m.rule_thickness))
                .bg(theme::border())
                .into_any_element(),
            Block::Table {
                headers,
                rows,
                aligns,
            } => self.table(headers, rows, aligns, color),
        }
    }

    fn table(
        &mut self,
        headers: &[Vec<Inline>],
        rows: &[Vec<Vec<Inline>>],
        aligns: &[TableAlign],
        color: Hsla,
    ) -> AnyElement {
        let mut table = div()
            .flex()
            .flex_col()
            .w_full()
            .border(px(self.m.table_border))
            .border_color(theme::border())
            .rounded_md()
            .overflow_hidden();

        let mut header_row = div().flex().flex_row().bg(theme::fill_secondary());
        for (ix, cell) in headers.iter().enumerate() {
            let content = self.text(cell, color, true);
            header_row = header_row.child(
                div()
                    .flex_1()
                    .px(px(self.m.table_cell_padding))
                    .py(px(self.m.table_cell_padding))
                    .text_align(gpui_align(aligns.get(ix).copied().unwrap_or_default()))
                    .child(content),
            );
        }
        table = table.child(header_row);

        for row in rows {
            let mut row_element = div()
                .flex()
                .flex_row()
                .border_t(px(self.m.table_border))
                .border_color(theme::border());
            for (ix, cell) in row.iter().enumerate() {
                let content = self.text(cell, color, false);
                row_element = row_element.child(
                    div()
                        .flex_1()
                        .px(px(self.m.table_cell_padding))
                        .py(px(self.m.table_cell_padding))
                        .text_align(gpui_align(aligns.get(ix).copied().unwrap_or_default()))
                        .child(content),
                );
            }
            table = table.child(row_element);
        }
        table.into_any_element()
    }
}

struct MarkdownLink {
    id: String,
    label: String,
    url: SharedString,
    on_link: Rc<dyn Fn(&str, &mut Window, &mut App)>,
}

/// 保持 InteractiveText 的布局与鼠标命中不变，只把各链接区间暴露为
/// 可执行 Click 的无障碍子节点；容器本身不重复朗读正文。
struct AccessibleMarkdownText {
    id: ElementId,
    text: InteractiveText,
    links: Vec<MarkdownLink>,
}

struct MarkdownTextPrepaint {
    hitbox: Hitbox,
    link_nodes: Vec<accesskit::NodeId>,
}

impl Element for AccessibleMarkdownText {
    type RequestLayoutState = ();
    type PrepaintState = MarkdownTextPrepaint;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn a11y_role(&self) -> Option<Role> {
        Some(Role::GenericContainer)
    }

    fn write_a11y_info(&self, node: &mut accesskit::Node) {
        node.set_author_id(self.id.to_string());
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        self.text.request_layout(id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let hitbox = self
            .text
            .prepaint(id, inspector_id, bounds, request_layout, window, cx);
        MarkdownTextPrepaint {
            hitbox,
            link_nodes: Vec::new(),
        }
    }

    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut gpui::A11ySubtreeBuilder,
    ) {
        prepaint.link_nodes.clear();
        for (index, link) in self.links.iter().enumerate() {
            let mut node = accesskit::Node::new(Role::Link);
            a11y::Semantics::new(Role::Link, link.label.clone())
                .id(link.id.clone())
                .description(link.url.clone())
                .write(&mut node);
            node.add_action(accesskit::Action::Click);
            let node_id = builder.synthetic_node_id(index);
            if builder.push_child(node_id, node) {
                prepaint.link_nodes.push(node_id);
            }
        }
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        for (index, node_id) in prepaint.link_nodes.iter().copied().enumerate() {
            let Some(link) = self.links.get(index) else {
                continue;
            };
            let url = link.url.clone();
            let on_link = link.on_link.clone();
            window.on_a11y_action(node_id, accesskit::Action::Click, move |_, window, cx| {
                on_link(&url, window, cx)
            });
        }
        self.text.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            &mut prepaint.hitbox,
            window,
            cx,
        );
    }
}

fn gpui_align(align: TableAlign) -> TextAlign {
    match align {
        TableAlign::Left => TextAlign::Left,
        TableAlign::Center => TextAlign::Center,
        TableAlign::Right => TextAlign::Right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::metric::MarkdownMetric;

    fn metric() -> MarkdownMetric {
        MarkdownMetric {
            font_size: 14.0,
            line_height: 22.0,
            heading_sizes: [36.0, 30.0, 24.0, 20.0, 16.0, 14.0],
            heading_line_heights: [44.0, 38.0, 32.0, 28.0, 24.0, 22.0],
            block_gap: 8.0,
            list_gap: 4.0,
            bullet_width: 16.0,
            quote_padding: 12.0,
            quote_border: 2.0,
            rule_thickness: 1.0,
            link_thickness: 1.0,
            table_cell_padding: 8.0,
            table_border: 1.0,
            task_box_size: 16.0,
            task_check_size: 12.0,
        }
    }

    fn colors() -> RunColors {
        RunColors {
            text: gpui::black(),
            link: gpui::red(),
            code_bg: gpui::white(),
        }
    }

    #[test]
    fn markdown_parses_headings_and_paragraphs() {
        let blocks = parse("# 标题\n\n第一段\n第二行\n\n## 子标题");
        assert_eq!(blocks.len(), 3);
        match &blocks[0] {
            Block::Heading { level, inlines } => {
                assert_eq!(*level, 1);
                assert_eq!(inlines[0].text, "标题");
            }
            other => panic!("unexpected {other:?}"),
        }
        match &blocks[1] {
            Block::Paragraph(inlines) => assert_eq!(inlines[0].text, "第一段\n第二行"),
            other => panic!("unexpected {other:?}"),
        }
        match &blocks[2] {
            Block::Heading { level, .. } => assert_eq!(*level, 2),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn markdown_parses_fenced_code() {
        let blocks = parse("```rust\nfn main() {}\n```");
        match &blocks[0] {
            Block::Code { language, code } => {
                assert_eq!(
                    language.as_ref().map(|value| value.to_string()),
                    Some("rust".to_string())
                );
                assert_eq!(code.as_ref(), "fn main() {}");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn markdown_parses_nested_list() {
        let blocks = parse("- 一\n  - 子项\n- 二");
        match &blocks[0] {
            Block::List {
                ordered,
                start,
                items,
            } => {
                assert!(!*ordered);
                assert_eq!(*start, 1);
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0].blocks[1], Block::List { .. }));
                assert!(items.iter().all(|item| item.task.is_none()));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn markdown_parses_ordered_list_start() {
        let blocks = parse("3. 三\n4. 四");
        match &blocks[0] {
            Block::List {
                ordered,
                start,
                items,
            } => {
                assert!(*ordered);
                assert_eq!(*start, 3);
                assert_eq!(items.len(), 2);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn markdown_parses_quote_rule_and_table() {
        let blocks = parse("> 引用\n\n---\n\n| a | b |\n| --- | :-: |\n| 1 | 2 |");
        assert!(matches!(blocks[0], Block::Quote(_)));
        assert!(matches!(blocks[1], Block::Rule));
        match &blocks[2] {
            Block::Table {
                headers,
                rows,
                aligns,
            } => {
                assert_eq!(headers.len(), 2);
                assert_eq!(rows.len(), 1);
                assert_eq!(headers[0][0].text, "a");
                assert_eq!(rows[0][1][0].text, "2");
                assert_eq!(aligns, &[TableAlign::Left, TableAlign::Center]);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn markdown_table_aligns_follow_delimiter_colons() {
        let blocks = parse("| a | b | c | d |\n| :-- | :-: | --: | --- |\n| 1 | 2 | 3 | 4 |");
        match &blocks[0] {
            Block::Table { aligns, .. } => assert_eq!(
                aligns,
                &[
                    TableAlign::Left,
                    TableAlign::Center,
                    TableAlign::Right,
                    TableAlign::Left
                ]
            ),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn markdown_task_list_markers() {
        let blocks = parse("- [x] done\n- [ ] todo\n- plain\n- [x]nospace\n- [X] upper");
        match &blocks[0] {
            Block::List { items, .. } => {
                let tasks: Vec<Option<bool>> = items.iter().map(|item| item.task).collect();
                assert_eq!(tasks, vec![Some(true), Some(false), None, None, Some(true)]);
                match &items[0].blocks[0] {
                    Block::Paragraph(inlines) => assert_eq!(inlines[0].text, "done"),
                    other => panic!("unexpected {other:?}"),
                }
                match &items[3].blocks[0] {
                    Block::Paragraph(inlines) => assert_eq!(inlines[0].text, "[x]nospace"),
                    other => panic!("unexpected {other:?}"),
                }
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn markdown_task_marker_keeps_nested_content() {
        let blocks = parse("- [ ] 父项\n  - 子项\n- [x] 完成项");
        match &blocks[0] {
            Block::List { items, .. } => {
                assert_eq!(items[0].task, Some(false));
                assert!(matches!(items[0].blocks[1], Block::List { .. }));
                assert_eq!(items[1].task, Some(true));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn markdown_autolink_bare_urls() {
        let inlines = parse_inline("a https://x.y/p, b www.z.w c (https://q.r/(s)) d");
        let links: Vec<(String, String)> = inlines
            .iter()
            .filter_map(|inline| {
                inline
                    .link
                    .as_ref()
                    .map(|href| (inline.text.clone(), href.clone()))
            })
            .collect();
        assert_eq!(
            links,
            vec![
                ("https://x.y/p".to_string(), "https://x.y/p".to_string()),
                ("www.z.w".to_string(), "http://www.z.w".to_string()),
                ("https://q.r/(s)".to_string(), "https://q.r/(s)".to_string()),
            ]
        );
        let plain: String = inlines
            .iter()
            .filter(|inline| inline.link.is_none())
            .map(|inline| inline.text.as_str())
            .collect();
        assert_eq!(plain, "a , b  c () d");
    }

    #[test]
    fn markdown_autolink_boundary_and_non_ascii() {
        let inlines = parse_inline("见 https://a.b/path 尾。xhttps://c.d ftp://e.f/g");
        let links: Vec<&str> = inlines
            .iter()
            .filter_map(|inline| inline.link.as_deref())
            .collect();
        assert_eq!(links, vec!["https://a.b/path", "ftp://e.f/g"]);
        assert!(
            inlines
                .iter()
                .any(|inline| inline.link.is_none() && inline.text.contains("xhttps://c.d"))
        );
    }

    #[test]
    fn markdown_autolink_skips_code_and_bad_domain() {
        let code = parse_inline("`https://a.b`");
        assert_eq!(code.len(), 1);
        assert!(code[0].code && code[0].link.is_none());
        let bad = parse_inline("http://localhost/ https://a_b.c/");
        assert!(bad.iter().all(|inline| inline.link.is_none()));
    }

    #[test]
    fn markdown_inline_styles_and_links() {
        let inlines = parse_inline("普通 **粗** *斜* ~~删~~ `代码` [链接](https://example.com)");
        assert!(inlines.iter().any(|i| i.text == "粗" && i.bold));
        assert!(inlines.iter().any(|i| i.text == "斜" && i.italic));
        assert!(inlines.iter().any(|i| i.text == "删" && i.strike));
        assert!(inlines.iter().any(|i| i.text == "代码" && i.code));
        assert!(
            inlines
                .iter()
                .any(|i| i.text == "链接" && i.link.as_deref() == Some("https://example.com"))
        );
    }

    #[test]
    fn markdown_run_offsets_and_link_ranges() {
        let inlines = parse_inline("a **b** [c](https://x.y)");
        let (text, runs, links) =
            build_runs(&inlines, &gpui::font("X"), false, &metric(), colors());
        assert_eq!(text.as_ref(), "a b c");
        assert_eq!(
            runs.iter().map(|run| run.len).sum::<usize>(),
            text.as_ref().len()
        );
        assert_eq!(links.len(), 1);
        let (range, url) = &links[0];
        assert_eq!(&text.as_ref()[range.clone()], "c");
        assert_eq!(url.as_ref(), "https://x.y");
        assert!(runs.last().unwrap().underline.is_some());
    }
}
