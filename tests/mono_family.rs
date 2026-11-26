//! 阶段二十：全局等宽族覆盖旋钮（加性；默认 None 时各控件保持平台回退）。
//! 独立测试二进制，全局状态不与其它矩阵串扰；单例内顺序断言。

#[test]
fn mono_family_override_roundtrip() {
    let os_default = if cfg!(target_os = "windows") {
        "Consolas"
    } else if cfg!(target_os = "macos") {
        "Menlo"
    } else {
        "monospace"
    };

    assert_eq!(gpui_widgets::theme::mono_family_or("Consolas"), "Consolas");
    assert_eq!(gpui_widgets::theme::metric::mono_font_family(), os_default);
    let (size, fam) = gpui_widgets::theme::metric::input_typography(13., None, true);
    assert_eq!(size, 13.);
    assert_eq!(fam, Some(os_default));
    let (_, none) = gpui_widgets::theme::metric::input_typography(13., None, false);
    assert_eq!(none, None);

    gpui_widgets::theme::set_mono_family(Some("JetBrains Mono"));
    assert_eq!(
        gpui_widgets::theme::mono_family_or("Consolas"),
        "JetBrains Mono"
    );
    assert_eq!(
        gpui_widgets::theme::metric::mono_font_family(),
        "JetBrains Mono"
    );
    let (_, overridden) = gpui_widgets::theme::metric::input_typography(12., Some(11.), true);
    assert_eq!(overridden, Some("JetBrains Mono"));

    gpui_widgets::theme::set_mono_family(None);
    assert_eq!(gpui_widgets::theme::metric::mono_font_family(), os_default);
}
