//! 粘贴换行归一纯函数矩阵（0.1.9 ⑩）。
//! 只测 `normalize_paste`：不建 Window / 不渲染，bracketed paste 包装沿用既有 paste_text 路径。
//! 语义照原版 xterm.js `Terminal.paste` 的 `data.replace(/\r?\n/g, '\r')`：
//! 终端里换行就是回车，LF 不归一会当独立字符发给远端，多行粘贴错行。

use gpui_widgets::terminal::normalize_paste;

#[test]
fn terminal_paste_crlf_to_single_cr() {
    assert_eq!(normalize_paste("a\r\nb"), "a\rb");
    // CRLF 不能变成两个 CR，否则远端多收到一次回车
    assert_eq!(normalize_paste("\r\n"), "\r");
    assert_eq!(normalize_paste("line\r\n"), "line\r");
}

#[test]
fn terminal_paste_lone_lf_to_cr() {
    assert_eq!(normalize_paste("a\nb"), "a\rb");
    assert_eq!(normalize_paste("\n"), "\r");
    assert_eq!(normalize_paste("line\n"), "line\r");
}

#[test]
fn terminal_paste_lone_cr_preserved() {
    // 已经是 CR 的不动；连续 CR 是远端语义（多次回车），不能合并
    assert_eq!(normalize_paste("a\rb"), "a\rb");
    assert_eq!(normalize_paste("a\r\rb"), "a\r\rb");
    assert_eq!(normalize_paste("\r\r\r"), "\r\r\r");
}

#[test]
fn terminal_paste_lf_then_cr_not_merged() {
    // `\n\r` 不是 CRLF：LF 归一成 CR，原有的 CR 保留 ⇒ 两个 CR
    assert_eq!(normalize_paste("a\n\rb"), "a\r\rb");
}

#[test]
fn terminal_paste_mixed_sequence() {
    assert_eq!(normalize_paste("a\r\n\nb\r\rc\n"), "a\r\rb\r\rc\r");
    assert_eq!(normalize_paste("\n\n\n"), "\r\r\r");
    assert_eq!(normalize_paste("\r\n\r\n"), "\r\r");
}

#[test]
fn terminal_paste_empty_and_plain_unchanged() {
    assert_eq!(normalize_paste(""), "");
    assert_eq!(normalize_paste("plain text 123"), "plain text 123");
    // 制表符 / 空格 / ESC 序列都不该被碰
    assert_eq!(normalize_paste("a\tb c"), "a\tb c");
    assert_eq!(
        normalize_paste("\x1b[200~x\x1b[201~"),
        "\x1b[200~x\x1b[201~"
    );
}

#[test]
fn terminal_paste_multibyte_utf8_intact() {
    assert_eq!(
        normalize_paste("中文\r\n日本語\nemoji🚀"),
        "中文\r日本語\remoji🚀"
    );
    // 归一只碰 ASCII 的 CR/LF，多字节字符不能被按字节截断
    assert_eq!(normalize_paste("🚀\n"), "🚀\r");
}

#[test]
fn terminal_paste_idempotent() {
    for input in [
        "",
        "a\r\nb",
        "a\nb",
        "a\r\rb",
        "a\n\rb",
        "\r\n\r\n\n",
        "中文\r\nx\n",
    ] {
        let once = normalize_paste(input);
        assert_eq!(normalize_paste(&once), once, "二次归一不应再变：{input:?}");
    }
}

#[test]
fn terminal_paste_never_emits_lf_and_never_grows() {
    for input in [
        "a\r\n\nb\r\rc\n",
        "\n\r\n\r",
        "line1\nline2\r\nline3\r",
        "中文\r\n日本語\n",
        "\n\n\n\n",
    ] {
        let out = normalize_paste(input);
        assert!(!out.contains('\n'), "归一后不得残留 LF：{input:?}");
        assert!(out.len() <= input.len(), "归一只会缩短不会变长：{input:?}");
        // 非换行字符逐个保留、顺序不变
        let keep = |s: &str| {
            s.chars()
                .filter(|c| *c != '\r' && *c != '\n')
                .collect::<String>()
        };
        assert_eq!(keep(&out), keep(input), "非换行内容不得丢失：{input:?}");
    }
}

#[test]
fn terminal_paste_cr_count_equals_line_break_count() {
    // 每个「换行事件」（CRLF 或 lone LF 或 lone CR）恰好产出一个 CR
    for (input, expected_cr) in [
        ("a\r\nb", 1),
        ("a\nb", 1),
        ("a\rb", 1),
        ("a\r\n\nb", 2),
        ("a\n\rb", 2),
        ("\r\n\r\n\r\n", 3),
        ("no break", 0),
    ] {
        let out = normalize_paste(input);
        assert_eq!(
            out.chars().filter(|c| *c == '\r').count(),
            expected_cr,
            "CR 计数不符：{input:?} → {out:?}"
        );
    }
}
