//! 代码编辑器：在 [`TextArea`] 的代码模式（行号 / 当前行 / 等宽字体）之上，
//! 提供常用语言的轻量逐行语法着色预设。
//!
//! ```ignore
//! // 构建：行号 + 当前行高亮 + 等宽字体 + 4 空格缩进 + Rust 着色
//! let editor = cx.new(|cx| CodeEditor::new(cx, CodeLanguage::Rust));
//! // render 中：div().child(editor.clone())
//! ```
//!
//! 着色为单行词法级的近似实现（关键字 / 类型 / 字面量 / 字符串 / 数字 /
//! 行注释 / 函数名），足以让代码观感接近真实编辑器，不做跨行状态推导。

use std::rc::Rc;

use gpui::{Context, Hsla};

use crate::text_area::{CodeToken, Highlighter, TextArea};
use crate::theme;

/// 代码编辑器支持着色的语言
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeLanguage {
    Plain,
    Rust,
    JavaScript,
    Json,
    Toml,
    Python,
    Shell,
}

impl CodeLanguage {
    /// 展示名
    pub fn label(self) -> &'static str {
        match self {
            Self::Plain => "纯文本",
            Self::Rust => "Rust",
            Self::JavaScript => "JavaScript",
            Self::Json => "JSON",
            Self::Toml => "TOML",
            Self::Python => "Python",
            Self::Shell => "Shell",
        }
    }

    /// 全部语言（供语言切换器遍历）
    pub fn all() -> &'static [CodeLanguage] {
        &[
            Self::Plain,
            Self::Rust,
            Self::JavaScript,
            Self::Json,
            Self::Toml,
            Self::Python,
            Self::Shell,
        ]
    }

    /// 按语言名/别名解析（大小写不敏感）；无法识别时返回 `None`
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "plain" | "text" | "txt" | "" => Some(Self::Plain),
            "rust" | "rs" => Some(Self::Rust),
            "javascript" | "js" | "jsx" | "typescript" | "ts" | "tsx" => Some(Self::JavaScript),
            "json" | "jsonc" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            "python" | "py" => Some(Self::Python),
            "shell" | "sh" | "bash" | "zsh" | "console" => Some(Self::Shell),
            _ => None,
        }
    }

    /// 按行高亮器；`Plain` 返回 `None`（不改变文字颜色）
    pub fn highlighter(self) -> Option<Highlighter> {
        let spec = self.spec()?;
        Some(Rc::new(move |line: &str| tokenize(line, spec)))
    }

    fn spec(self) -> Option<&'static LangSpec> {
        match self {
            Self::Plain => None,
            Self::Rust => Some(&RUST),
            Self::JavaScript => Some(&JS),
            Self::Json => Some(&JSON),
            Self::Toml => Some(&TOML),
            Self::Python => Some(&PYTHON),
            Self::Shell => Some(&SHELL),
        }
    }
}

/// 代码编辑器预设
pub struct CodeEditor;

impl CodeEditor {
    /// 构建代码模式的 [`TextArea`]：行号栏 + 当前行高亮 + 等宽字体 +
    /// 4 空格缩进 + 13px 字号 + 指定语言着色。
    pub fn new(cx: &mut Context<TextArea>, language: CodeLanguage) -> TextArea {
        let mut area = TextArea::new(cx)
            // TextArea publishes MultilineTextInput semantics, including its live value,
            // placeholder, disabled state and focusability. Give the code preset a
            // meaningful accessible name without wrapping or replacing its focus node.
            .placeholder(format!("{} 代码编辑器", language.label()))
            .line_numbers(true)
            .current_line(true)
            .monospace()
            .padding(0., 7.)
            .text_size(13.)
            .tab_spaces(4);
        area.set_highlighter(language.highlighter(), cx);
        area
    }

    /// 运行时切换语言（`Plain` 会关闭着色）
    pub fn set_language(area: &mut TextArea, language: CodeLanguage, cx: &mut Context<TextArea>) {
        area.set_highlighter(language.highlighter(), cx);
    }
}

/// 单语言的词法规则
struct LangSpec {
    keywords: &'static [&'static str],
    types: &'static [&'static str],
    literals: &'static [&'static str],
    line_comment: Option<&'static str>,
}

static RUST: LangSpec = LangSpec {
    keywords: &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
        "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "type",
        "unsafe", "use", "where", "while",
    ],
    types: &[
        "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Rc", "Arc", "HashMap",
        "VecDeque", "HashSet", "RefCell", "Cell", "usize", "isize", "u8", "u16", "u32", "u64",
        "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64",
    ],
    literals: &["true", "false", "None", "Some"],
    line_comment: Some("//"),
};

static JS: LangSpec = LangSpec {
    keywords: &[
        "async",
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "export",
        "extends",
        "finally",
        "for",
        "from",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "let",
        "new",
        "of",
        "return",
        "static",
        "super",
        "switch",
        "throw",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "yield",
        "this",
    ],
    types: &[
        "String", "Number", "Boolean", "Object", "Array", "Promise", "Map", "Set", "console",
        "JSON", "Math",
    ],
    literals: &["true", "false", "null", "undefined"],
    line_comment: Some("//"),
};

static JSON: LangSpec = LangSpec {
    keywords: &[],
    types: &[],
    literals: &["true", "false", "null"],
    line_comment: Some("//"),
};

static TOML: LangSpec = LangSpec {
    keywords: &[],
    types: &[],
    literals: &["true", "false"],
    line_comment: Some("#"),
};

static PYTHON: LangSpec = LangSpec {
    keywords: &[
        "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
        "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is",
        "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "self", "try", "while",
        "with", "yield",
    ],
    types: &[
        "int", "float", "str", "bool", "list", "dict", "set", "tuple", "bytes", "len", "range",
        "print",
    ],
    literals: &["True", "False", "None"],
    line_comment: Some("#"),
};

static SHELL: LangSpec = LangSpec {
    keywords: &[
        "if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done", "case", "esac",
        "in", "function", "return", "export", "local", "exit", "source",
    ],
    types: &[],
    literals: &["true", "false"],
    line_comment: Some("#"),
};

/// 单行词法切分：返回完整覆盖该行的 token 序列
fn tokenize(line: &str, spec: &LangSpec) -> Vec<CodeToken> {
    let mut out: Vec<CodeToken> = Vec::new();
    let mut i = 0usize;

    while i < line.len() {
        let rest = &line[i..];
        let ch = rest.chars().next().unwrap();

        // 行注释：吞掉行尾
        if let Some(marker) = spec.line_comment
            && rest.starts_with(marker)
        {
            push(&mut out, i..line.len(), theme::text_secondary(), false);
            break;
        }

        // 字符串 / 字符字面量
        if ch == '"' || ch == '`' || (ch == '\'' && is_char_literal(rest)) {
            let len = scan_string(rest, ch);
            push(&mut out, i..i + len, theme::success(), false);
            i += len;
            continue;
        }

        // 数字（排除标识符尾部出现的数字）
        if ch.is_ascii_digit() && !prev_is_ident(line, i) {
            let len = scan_number(rest);
            push(&mut out, i..i + len, theme::warning(), false);
            i += len;
            continue;
        }

        // 标识符 / 关键字 / 类型 / 函数名
        if is_ident_start(ch) {
            let len = ident_len(rest);
            let word = &rest[..len];
            let next_is_call = rest[len..].trim_start().starts_with('(');
            if spec.keywords.contains(&word) {
                push(&mut out, i..i + len, theme::primary(), true);
            } else if spec.types.contains(&word) {
                push(&mut out, i..i + len, theme::primary_hover(), false);
            } else if spec.literals.contains(&word) {
                push(&mut out, i..i + len, theme::danger(), false);
            } else if next_is_call {
                push(&mut out, i..i + len, theme::chart_color(4), false);
            } else {
                push(&mut out, i..i + len, theme::text(), false);
            }
            i += len;
            continue;
        }

        // 其余单字符
        let len = ch.len_utf8();
        push(&mut out, i..i + len, theme::text(), false);
        i += len;
    }

    out
}

/// 追加 token，并合并相邻同色同粗细的片段，减少 run 数量
fn push(out: &mut Vec<CodeToken>, range: std::ops::Range<usize>, color: Hsla, bold: bool) {
    if range.is_empty() {
        return;
    }
    if let Some(last) = out.last_mut()
        && last.bold == bold
        && last.color == color
        && last.range.end == range.start
    {
        last.range.end = range.end;
        return;
    }
    out.push(CodeToken::new(range, color).bold(bold));
}

fn is_ident_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || !ch.is_ascii()
}

fn is_ident_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || !ch.is_ascii()
}

/// 标识符字节长度
fn ident_len(rest: &str) -> usize {
    let mut len = 0;
    for (idx, ch) in rest.char_indices() {
        if !is_ident_char(ch) {
            break;
        }
        len = idx + ch.len_utf8();
    }
    len
}

/// 判断 `'` 是否为字符字面量（而不是 Rust 生命周期）
fn is_char_literal(rest: &str) -> bool {
    let mut chars = rest.chars();
    chars.next(); // 跳过起始单引号
    match chars.next() {
        Some(marker) if marker != '\'' => {
            if marker == '\\' {
                // 转义字符：要求 8 字节内出现闭合引号
                rest[2..]
                    .char_indices()
                    .any(|(idx, c)| c == '\'' && idx <= 8)
            } else {
                chars.next() == Some('\'')
            }
        }
        _ => false,
    }
}

/// 字符串字节长度（含两端引号；未闭合则到行尾）
fn scan_string(rest: &str, quote: char) -> usize {
    let mut escaped = false;
    let mut end = quote.len_utf8();
    for (idx, ch) in rest.char_indices().skip(1) {
        end = idx + ch.len_utf8();
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            break;
        }
    }
    end
}

/// 数字字节长度（十六进制 / 小数 / 类型后缀；不吞掉 `..` 范围运算符）
fn scan_number(rest: &str) -> usize {
    let mut len = 0;
    for (idx, ch) in rest.char_indices() {
        let boundary = ch.is_ascii_alphanumeric()
            || ch == '_'
            || (ch == '.' && rest[idx + 1..].starts_with(|c: char| c.is_ascii_digit()));
        if !boundary {
            break;
        }
        len = idx + 1;
    }
    len.max(1)
}

/// 前一个字符是否为标识符字符（用于区分 `abc123` 与独立数字）
fn prev_is_ident(line: &str, i: usize) -> bool {
    line[..i]
        .chars()
        .next_back()
        .map(|c| is_ident_char(c) || c == '.')
        .unwrap_or(false)
}

/// 演示用纯文本（纯文本模式不着色）
pub const DEMO_PLAIN: &str = r#"发布检查清单

1. 运行 cargo test 确认全部用例通过
2. 检查 CHANGELOG 是否覆盖本次变更
3. 将版本号同步到 Cargo.toml 与文档
4. 打 tag 并推送：git tag v0.1.0 && git push --tags
5. 确认 CI 产物上传成功后再对外公告

备注：回滚方式为切回上一个 tag 并重新构建发布。"#;

/// 演示用 Rust 代码
pub const DEMO_RUST: &str = r#"use std::collections::HashMap;

/// 统计词频
fn word_count(text: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

fn main() {
    let text = "the quick brown fox jumps over the lazy dog";
    let counts = word_count(text);
    println!("共 {} 个不同的单词", counts.len());
    let hot = counts.iter().max_by_key(|(_, n)| **n);
    if let Some((word, n)) = hot {
        println!("出现最多：{} x {}", word, n);
    }
}"#;

/// 演示用 JavaScript 代码
pub const DEMO_JS: &str = r#"// 防抖工具
function debounce(fn, wait = 300) {
  let timer = null;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn.apply(this, args), wait);
  };
}

const search = debounce(async (keyword) => {
  const res = await fetch(`/api/search?q=${keyword}`);
  const data = await res.json();
  console.log(`找到 ${data.length} 条结果`);
}, 200);"#;

/// 演示用 JSON
pub const DEMO_JSON: &str = r#"{
  "name": "gpui-widgets",
  "version": "0.1.0",
  "private": false,
  "keywords": ["gpui", "ui", "widgets"],
  "dependencies": {
    "gpui": "0.2.2",
    "unicode-segmentation": "1.12"
  },
  "features": {
    "hot_reload": true,
    "max_workers": 4
  }
}"#;

/// 演示用 TOML
pub const DEMO_TOML: &str = r#"# 控件库构建配置
[package]
name = "gpui-widgets"
version = "0.1.0"
edition = "2024"

[dependencies]
gpui = "0.2.2"
unicode-segmentation = "1.12"

[profile.dev]
opt-level = 1
debug = true"#;

/// 演示用 Python
pub const DEMO_PYTHON: &str = r#"import json
from pathlib import Path


def load_config(path: Path) -> dict:
    """读取 JSON 配置"""
    if not path.exists():
        return {}
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


config = load_config(Path("config.json"))
print(f"加载 {len(config)} 项配置，调试模式：{config.get('debug', False)}")"#;

/// 演示用 Shell
pub const DEMO_SHELL: &str = r#"#!/usr/bin/env bash
set -euo pipefail

# 构建并发布示例
target="$1"
if [ -z "$target" ]; then
  echo "用法: ./build.sh <target>"
  exit 1
fi

for task in build test package; do
  echo "==> $task ($target)"
  cargo $task --release --target "$target"
done

echo "完成，产物位于 target/$target/release/""#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_editor_from_name_aliases() {
        for (name, expected) in [
            ("Rust", CodeLanguage::Rust),
            ("rs", CodeLanguage::Rust),
            ("JS", CodeLanguage::JavaScript),
            ("typescript", CodeLanguage::JavaScript),
            ("JSON", CodeLanguage::Json),
            ("toml", CodeLanguage::Toml),
            ("py", CodeLanguage::Python),
            ("bash", CodeLanguage::Shell),
            (" sh ", CodeLanguage::Shell),
            ("text", CodeLanguage::Plain),
        ] {
            assert_eq!(CodeLanguage::from_name(name), Some(expected), "{name}");
        }
        assert_eq!(CodeLanguage::from_name("cobol"), None);
    }

    #[test]
    fn code_editor_tokenizer_covers_line() {
        for language in CodeLanguage::all() {
            let Some(highlight) = language.highlighter() else {
                continue;
            };
            for line in ["let x = 1; // 注释", "\"字符串\" 123 abc", ""] {
                let tokens = highlight(line);
                let covered: usize = tokens.iter().map(|t| t.range.len()).sum();
                assert_eq!(covered, line.len(), "{language:?} {line:?}");
            }
        }
    }
}
