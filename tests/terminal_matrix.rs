//! 终端配色矩阵：明暗两套 ANSI 16 色 + 底/前景，锁定参照值并验证随主题翻转。
//!
//! 参照来自 niceshell 原版 xterm 配置（浅色 GitHub Light、深色 Tokyo Night）。
//! 终端控件只读 `theme::terminal_*()`，这套值一回归终端外观即失配。

use std::sync::{Mutex, MutexGuard};

use gpui::Hsla;
use gpui_widgets::terminal::terminal_frame;
use gpui_widgets::theme::{
    self, Appearance, Color, Seed, Theme, terminal_ansi_for, terminal_bg_for, terminal_fg_for, with,
};

#[test]
fn terminal_frame_matrix() {
    // 外框是纯状态函数：关掉就没有描边，开着则聚焦走主色、未聚焦走边框色。
    let _guard: MutexGuard<'_, ()> = THEME_LOCK.lock().unwrap();
    let original = with(|t| t.appearance);
    theme::set_theme(Theme::new(Seed::default(), Appearance::Light));

    assert!(
        terminal_frame(false, false).is_none(),
        "bordered=false 不该描边"
    );
    assert!(
        terminal_frame(false, true).is_none(),
        "bordered=false 聚焦也不该描边"
    );
    assert_hex(
        terminal_frame(true, false).unwrap(),
        with(|t| u32_hex(t.color_border)),
        "未聚焦描边",
    );
    assert_hex(
        terminal_frame(true, true).unwrap(),
        with(|t| u32_hex(t.color_primary)),
        "聚焦描边",
    );

    theme::set_appearance(original);
}

fn u32_hex(c: gpui::Hsla) -> u32 {
    let c = Color::from(c);
    ((c.r.round().clamp(0.0, 255.0) as u32) << 16)
        | ((c.g.round().clamp(0.0, 255.0) as u32) << 8)
        | (c.b.round().clamp(0.0, 255.0) as u32)
}

static THEME_LOCK: Mutex<()> = Mutex::new(());

/// 浅色 ANSI 16 色（0~7 常规、8~15 高亮）
const LIGHT_ANSI: [u32; 16] = [
    0x24292f, 0xcf222e, 0x116329, 0x7d4e00, 0x0969da, 0x8250df, 0x1b7c83, 0x6e7781, 0x57606a,
    0xa40e26, 0x1a7f37, 0x633c01, 0x218bff, 0xa475f9, 0x3192aa, 0x8c959f,
];

/// 深色 ANSI 16 色（Tokyo Night）
const DARK_ANSI: [u32; 16] = [
    0x15161a, 0xf7768e, 0x9ece6a, 0xe0af68, 0x7aa2f7, 0xbb9af7, 0x7dcfff, 0xc0caf5, 0x414868,
    0xff7a93, 0xb9f27c, 0xff9e64, 0x7da6ff, 0xc7a9ff, 0xa4daff, 0xe6e9ef,
];

fn channels(c: Hsla) -> [u8; 4] {
    let c = Color::from(c);
    [
        c.r.round().clamp(0.0, 255.0) as u8,
        c.g.round().clamp(0.0, 255.0) as u8,
        c.b.round().clamp(0.0, 255.0) as u8,
        (c.a * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

fn assert_hex(actual: Hsla, expected: u32, label: &str) {
    let a = channels(actual);
    let e = [
        ((expected >> 16) & 0xff) as u8,
        ((expected >> 8) & 0xff) as u8,
        (expected & 0xff) as u8,
        255,
    ];
    let diff = |x: u8, y: u8| (x as i16 - y as i16).abs();
    assert!(
        (0..4).all(|i| diff(a[i], e[i]) <= 1),
        "{label}: actual {a:?}, expected {e:?}"
    );
}

#[test]
fn terminal_light_matrix() {
    assert_hex(terminal_bg_for(Appearance::Light), 0xffffff, "light bg");
    assert_hex(terminal_fg_for(Appearance::Light), 0x24292f, "light fg");
    let palette = terminal_ansi_for(Appearance::Light);
    for (i, expected) in LIGHT_ANSI.iter().copied().enumerate() {
        assert_hex(palette[i], expected, &format!("light ansi[{i}]"));
    }
}

#[test]
fn terminal_dark_matrix() {
    assert_hex(terminal_bg_for(Appearance::Dark), 0x101216, "dark bg");
    assert_hex(terminal_fg_for(Appearance::Dark), 0xd7dbe3, "dark fg");
    let palette = terminal_ansi_for(Appearance::Dark);
    for (i, expected) in DARK_ANSI.iter().copied().enumerate() {
        assert_hex(palette[i], expected, &format!("dark ansi[{i}]"));
    }
}

#[test]
fn terminal_contrast_matrix() {
    // 底与正文必须拉开足够亮度差，否则终端文字不可读；两套配色同向校验。
    for appearance in [Appearance::Light, Appearance::Dark] {
        let delta = (terminal_fg_for(appearance).l - terminal_bg_for(appearance).l).abs();
        assert!(
            delta > 0.6,
            "{appearance:?}: 终端底/正文亮度差过小（{delta}）"
        );
    }
    // ANSI 的黑 → 白 → 高亮白必须逐级变亮，两套配色都成立。
    for appearance in [Appearance::Light, Appearance::Dark] {
        let p = terminal_ansi_for(appearance);
        assert!(
            p[0].l < p[7].l && p[7].l < p[15].l,
            "{appearance:?}: ANSI 灰阶顺序颠倒"
        );
        // 高亮档只在深色配色整体变亮；浅色参照（GitHub Light）的高亮红比常规红更沉，
        // 靠饱和度而非亮度区分，所以这里只校验灰阶。
        if appearance == Appearance::Dark {
            assert!(p[9].l > p[1].l, "Dark: 红色高亮档未变亮");
        }
    }
}

#[test]
fn terminal_theme_matrix() {
    // 终端控件读的是全局 terminal_*()，验证它们随明暗形态整体翻转。
    let _guard: MutexGuard<'_, ()> = THEME_LOCK.lock().unwrap();
    let original = with(|t| t.appearance);

    theme::set_theme(Theme::new(Seed::default(), Appearance::Light));
    assert_hex(theme::terminal_bg(), 0xffffff, "global light bg");
    assert_hex(theme::terminal_fg(), 0x24292f, "global light fg");
    assert_hex(
        theme::terminal_ansi()[4],
        LIGHT_ANSI[4],
        "global light ansi[4]",
    );

    theme::set_appearance(Appearance::Dark);
    assert_hex(theme::terminal_bg(), 0x101216, "global dark bg");
    assert_hex(theme::terminal_fg(), 0xd7dbe3, "global dark fg");
    assert_hex(
        theme::terminal_ansi()[4],
        DARK_ANSI[4],
        "global dark ansi[4]",
    );

    theme::set_appearance(original);
}
