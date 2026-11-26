//! OSC 7 工作目录解析纯函数矩阵（0.1.8 ⑦）。
//! 只测 `parse_osc7`：不建 Window / 不渲染，OSC 分派沿用既有跨包状态机。
//! 语义照原版 niceshell `registerOscHandler(7)`：`file://host/path` → cwd 驱动 SFTP 跟随。

use gpui_widgets::terminal::parse_osc7;

#[test]
fn terminal_osc7_host_and_path() {
    assert_eq!(
        parse_osc7("file://myhost/home/user/proj").as_deref(),
        Some("/home/user/proj")
    );
    // 空 host（三斜杠）是最常见的 shell 集成写法
    assert_eq!(
        parse_osc7("file:///home/user").as_deref(),
        Some("/home/user")
    );
}

#[test]
fn terminal_osc7_prefix_case_insensitive() {
    assert_eq!(parse_osc7("FILE://host/tmp").as_deref(), Some("/tmp"));
    assert_eq!(parse_osc7("File://host/tmp").as_deref(), Some("/tmp"));
}

#[test]
fn terminal_osc7_path_case_preserved() {
    // 前缀大小写不敏感，但路径本身大小写敏感（Linux），不能被一起 lowercase
    assert_eq!(
        parse_osc7("file://host/Home/User/Proj").as_deref(),
        Some("/Home/User/Proj")
    );
}

#[test]
fn terminal_osc7_percent_decode() {
    assert_eq!(
        parse_osc7("file://host/home/user%20name").as_deref(),
        Some("/home/user name")
    );
    // 多字节 UTF-8 按字节收集后整体解码，不能逐字节截断
    assert_eq!(
        parse_osc7("file://host/tmp/%E4%B8%AD%E6%96%87").as_deref(),
        Some("/tmp/中文")
    );
    // 解码出的斜杠参与路径归一
    assert_eq!(parse_osc7("file://host/a%2Fb").as_deref(), Some("/a/b"));
}

#[test]
fn terminal_osc7_bad_percent_kept_literal() {
    assert_eq!(
        parse_osc7("file://host/tmp/%zz").as_deref(),
        Some("/tmp/%zz")
    );
    assert_eq!(parse_osc7("file://host/tmp/%2").as_deref(), Some("/tmp/%2"));
    assert_eq!(parse_osc7("file://host/tmp/%").as_deref(), Some("/tmp/%"));
    // "+1" 能被 from_str_radix 接受，但不是合法百分号转义
    assert_eq!(
        parse_osc7("file://host/tmp/%+1").as_deref(),
        Some("/tmp/%+1")
    );
}

#[test]
fn terminal_osc7_trailing_slash() {
    assert_eq!(parse_osc7("file://host/tmp/").as_deref(), Some("/tmp"));
    assert_eq!(parse_osc7("file://host/tmp///").as_deref(), Some("/tmp"));
    // 根目录的尾斜杠就是路径本身，不能削成空串
    assert_eq!(parse_osc7("file://host/").as_deref(), Some("/"));
    assert_eq!(parse_osc7("file:///").as_deref(), Some("/"));
}

#[test]
fn terminal_osc7_rejects_invalid() {
    assert_eq!(parse_osc7(""), None);
    assert_eq!(parse_osc7("file://host"), None);
    assert_eq!(parse_osc7("http://host/path"), None);
    assert_eq!(parse_osc7("file:/host/path"), None);
    // 尾斜杠削完只剩空串（"//"）时不是绝对路径
    assert_eq!(parse_osc7("file://host//"), None);
}

#[test]
fn terminal_osc7_short_multibyte_no_panic() {
    // 前缀判定按字节切片，多字节短串必须走 get(..7) 而不是直接索引
    assert_eq!(parse_osc7("中"), None);
    assert_eq!(parse_osc7("file://"), None);
    assert_eq!(parse_osc7("文件://host/tmp"), None);
}
