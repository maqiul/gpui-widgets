//! 控件库主题：antdv-next 的三层 Design Token 体系（Seed → 派生 → 别名）。
//!
//! 默认浅色主题，主色 `#1677ff`，所有色值 / 尺寸 / 交互态都由种子派生，
//! 因此改主色或切深色不需要知道任何一个具体色值：
//!
//! ```ignore
//! use gpui::{rgb, App};
//! use gpui_widgets::theme::{self, Theme};
//!
//! theme::set_theme(Theme::dark());        // 切深色：整套色板重新派生
//! theme::set_accent(rgb(0x8b5cf6));       // 换品牌主色：hover/active/border/bg 全部跟着算
//! theme::update(|t| t.color_border = rgb(0xff0000).into()); // 微调单个别名令牌
//! cx.refresh_windows();                     // 切换后触发重绘
//! ```
//!
//! 控件取色优先用具体令牌函数（`theme::primary()`、`theme::color_fill_tertiary()` …），
//! 需要成组令牌时用 `theme::with(|t| …)` 一次读完，避免反复抢锁。

pub mod metric;
mod palette;
mod preset;
mod token;

use std::sync::{LazyLock, RwLock};

use gpui::{BoxShadow, Hsla, point, px, rgb};

pub use palette::Color;
pub use preset::{PresetColor, PresetPalette, preset_palette};
pub use token::{Appearance, Seed, Shadows, Tokens, derive};

static THEME: LazyLock<RwLock<Theme>> = LazyLock::new(|| RwLock::new(Theme::light()));

/// 当前主题 = 种子 + 明暗形态 + 派生出的别名令牌。
/// `tokens` 是控件唯一该读的东西；`seed` / `appearance` 只在重新派生时用。
#[derive(Clone, Debug)]
pub struct Theme {
    pub seed: Seed,
    pub appearance: Appearance,
    pub tokens: Tokens,
}

impl Theme {
    /// antdv-next defaultAlgorithm 对应的浅色主题
    pub fn light() -> Self {
        Self::new(Seed::default(), Appearance::Light)
    }

    /// antdv-next darkAlgorithm 对应的深色主题
    pub fn dark() -> Self {
        Self::new(Seed::default(), Appearance::Dark)
    }

    pub fn new(seed: Seed, appearance: Appearance) -> Self {
        let tokens = derive(&seed, appearance);
        Self {
            seed,
            appearance,
            tokens,
        }
    }

    pub fn is_dark(&self) -> bool {
        self.appearance == Appearance::Dark
    }

    /// 换品牌主色：重新派生整套主色族（hover / active / border / bg / outline）
    pub fn with_accent(mut self, color: impl Into<Hsla>) -> Self {
        self.seed.color_primary = Color::from(color.into());
        Self::new(self.seed, self.appearance)
    }

    /// 换明暗形态，保留已改过的种子
    pub fn with_appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = appearance;
        Self::new(self.seed, appearance)
    }

    /// 按 antdv-next 的种子约定改功能色：`color_info` 独立、`color_link` 为 `None` 时跟随 info
    pub fn with_functional(mut self, name: impl FnOnce(&mut Seed)) -> Self {
        name(&mut self.seed);
        Self::new(self.seed, self.appearance)
    }
}

/// 整体替换当前主题（之后调用 `cx.refresh_windows()` 或 `cx.notify()` 重绘）
pub fn set_theme(theme: Theme) {
    *THEME.write().unwrap() = theme;
}

/// 读取当前主题副本（含种子与令牌）
pub fn current() -> Theme {
    THEME.read().unwrap().clone()
}

/// 在当前主题上微调别名令牌：`theme::update(|t| t.color_border = …)`。
/// 不会重新派生，改动可能被后续 `set_theme` / `set_accent` 覆盖。
pub fn update(f: impl FnOnce(&mut Tokens)) {
    f(&mut THEME.write().unwrap().tokens);
}

/// 只换品牌主色（重新派生主色族）
pub fn set_accent(color: impl Into<Hsla>) {
    update_seed_and_rederive(|seed| seed.color_primary = Color::from(color.into()));
}

/// 切明暗形态（保留种子）
pub fn set_appearance(appearance: Appearance) {
    let mut guard = THEME.write().unwrap();
    guard.appearance = appearance;
    guard.tokens = derive(&guard.seed, appearance);
}

/// 改种子并重新派生全部令牌
pub fn update_seed_and_rederive(f: impl FnOnce(&mut Seed)) {
    let mut guard = THEME.write().unwrap();
    f(&mut guard.seed);
    guard.tokens = derive(&guard.seed, guard.appearance);
}

/// 一次读取多个令牌，避免每取一个色就抢一次锁：`theme::with(|t| (t.color_primary, t.font_size))`
pub fn with<R>(f: impl FnOnce(&Tokens) -> R) -> R {
    f(&THEME.read().unwrap().tokens)
}

static MONO_FAMILY: LazyLock<RwLock<Option<&'static str>>> = LazyLock::new(|| RwLock::new(None));

/// 宿主随包等宽字体（如 JetBrains Mono）后全局覆盖平台回退等宽族；
/// `None` 保持各控件原有平台回退（gallery 基线零变化）。
pub fn set_mono_family(family: Option<&'static str>) {
    *MONO_FAMILY.write().unwrap() = family;
}

/// 取全局等宽覆盖族，未设置时回退 `default`。
pub fn mono_family_or(default: &'static str) -> &'static str {
    MONO_FAMILY.read().unwrap().unwrap_or(default)
}

static SCROLLBAR_THUMB: LazyLock<RwLock<Option<Hsla>>> = LazyLock::new(|| RwLock::new(None));

/// 全局覆盖滚动条滑块颜色（含 alpha）；`None` 保持 antd 实测回退 `rgba(0,0,0,0.5)`。
///
/// 一次管住库内所有自绘滚动条；控件级 `Scrollbar::thumb_color` 优先级更高。
/// 切明暗 / `set_theme` 不会重算它，需要换色时宿主自己再调一次。
pub fn set_scrollbar_thumb(color: Option<Hsla>) {
    *SCROLLBAR_THUMB.write().unwrap() = color;
}

/// 当前滑块全局覆盖色，未设置时为 `None`。
pub fn scrollbar_thumb_override() -> Option<Hsla> {
    *SCROLLBAR_THUMB.read().unwrap()
}

/// 取滑块颜色，未覆盖时回退 `default`。
pub fn scrollbar_thumb_or(default: Hsla) -> Hsla {
    scrollbar_thumb_override().unwrap_or(default)
}

static SCROLLBAR_TRACK: LazyLock<RwLock<Option<Hsla>>> = LazyLock::new(|| RwLock::new(None));

/// 全局覆盖滚动条轨道底色（含 alpha）。rc-virtual-list 常态不画轨道、只在悬停时
/// 画 `colorFillQuaternary`，所以本覆盖同样只在悬停时可见；`None` 恢复令牌值。
pub fn set_scrollbar_track(color: Option<Hsla>) {
    *SCROLLBAR_TRACK.write().unwrap() = color;
}

/// 当前轨道全局覆盖色，未设置时为 `None`。
pub fn scrollbar_track_override() -> Option<Hsla> {
    *SCROLLBAR_TRACK.read().unwrap()
}

/// 取轨道颜色，未覆盖时回退 `default`。
pub fn scrollbar_track_or(default: Hsla) -> Hsla {
    scrollbar_track_override().unwrap_or(default)
}

/// 把半透明色叠加到背景上（antd 的 `FastColor.onBackground`）：
/// 用于 `colorFillTertiary` 之类的 alpha 填充合成实色。
pub fn blend(front: Hsla, back: Hsla) -> Hsla {
    Hsla::from(back.to_rgb().blend(front.to_rgb()))
}

/// 派生 `Tokens` 上的取值函数，同时保留旧语义名（bg / surface / accent …）作为别名
macro_rules! getters {
    ($($name:ident => $field:ident),* $(,)?) => {
        $(
            #[doc = concat!("当前主题的 `", stringify!($field), "` 令牌")]
            pub fn $name() -> Hsla {
                THEME.read().unwrap().tokens.$field
            }
        )*
    };
}

// ---- 主色族 ----
getters! {
    primary => color_primary,
    primary_hover => color_primary_hover,
    primary_active => color_primary_active,
    primary_bg => color_primary_bg,
    primary_bg_hover => color_primary_bg_hover,
    primary_border => color_primary_border,
    primary_border_hover => color_primary_border_hover,
    primary_text => color_primary_text,
    primary_text_hover => color_primary_text_hover,
    primary_text_active => color_primary_text_active,
    primary_outline => color_primary_outline,
}

// ---- 功能色族 ----
getters! {
    success => color_success,
    success_hover => color_success_hover,
    success_active => color_success_active,
    success_bg => color_success_bg,
    success_border => color_success_border,
    warning => color_warning,
    warning_hover => color_warning_hover,
    warning_active => color_warning_active,
    warning_bg => color_warning_bg,
    warning_border => color_warning_border,
    danger => color_error,
    danger_hover => color_error_hover,
    danger_active => color_error_active,
    danger_bg => color_error_bg,
    danger_bg_hover => color_error_bg_hover,
    danger_border => color_error_border,
    danger_outline => color_error_outline,
    info => color_info,
    info_bg => color_info_bg,
    link => color_link,
    link_hover => color_link_hover,
    link_active => color_link_active,
}

// ---- 中性色 ----
getters! {
    text => color_text,
    text_secondary => color_text_secondary,
    text_tertiary => color_text_tertiary,
    text_quaternary => color_text_quaternary,
    text_disabled => color_text_disabled,
    text_placeholder => color_text_placeholder,
    text_heading => color_text_heading,
    text_light_solid => color_text_light_solid,
    white => color_white,
    fill => color_fill,
    fill_secondary => color_fill_secondary,
    fill_tertiary => color_fill_tertiary,
    fill_quaternary => color_fill_quaternary,
    bg_layout => color_bg_layout,
    bg_container => color_bg_container,
    bg_elevated => color_bg_elevated,
    bg_spotlight => color_bg_spotlight,
    bg_mask => color_bg_mask,
    bg_solid => color_bg_solid,
    bg_solid_hover => color_bg_solid_hover,
    bg_solid_active => color_bg_solid_active,
    border => color_border,
    border_secondary => color_border_secondary,
    split => color_split,
    icon => color_icon,
    icon_hover => color_icon_hover,
}

// ---- 交互态 ----
getters! {
    control_item_bg_hover => control_item_bg_hover,
    control_item_bg_active => control_item_bg_active,
    control_item_bg_active_hover => control_item_bg_active_hover,
    control_item_bg_active_disabled => control_item_bg_active_disabled,
}

// ---- 旧 21 语义色：保留为别名，未迁移的控件继续可编译 ----
getters! {
    bg => color_bg_layout,
    surface => color_bg_container,
    text_muted => color_text_secondary,
    hover_bg => control_item_bg_hover,
    active_bg => color_fill_secondary,
    accent => color_primary,
    accent_hover => color_primary_hover,
    accent_active => color_primary_active,
    accent_soft => color_primary_bg,
    danger_soft => color_error_bg,
    success_soft => color_success_bg,
    warning_soft => color_warning_bg,
    track => color_fill_secondary,
    code_bg => color_bg_layout,
}

/// 图表分类色板：8 个色相 + 明度双重区分的类别色，取自 antd 预置色系，明暗主题自适应。
/// 用于饼图 / 雷达 / 散点 / 热力等多系列图表，保证相邻类别可分辨。
/// 深色主题下按 dark 色板的做法提亮，避免在暗底上刺眼。
pub fn chart_palette() -> [Hsla; 8] {
    let raw: [u32; 8] = [
        0x1677ff, // 蓝（colorPrimary 基色）
        0x13a8a8, // 青
        0xfaad14, // 琥珀（colorWarning 基色）
        0xff4d4f, // 红（colorError 基色）
        0x722ed1, // 紫
        0x52c41a, // 绿（colorSuccess 基色）
        0xeb2f96, // 品红
        0xa0d911, // 柠绿
    ];
    let dark = with(|t| t.appearance == Appearance::Dark);
    raw.map(|c| {
        let h: Hsla = rgb(c).into();
        if dark {
            Hsla {
                l: (h.l + 0.10).min(0.74),
                s: (h.s * 0.92).min(1.),
                ..h
            }
        } else {
            h
        }
    })
}

/// 取分类色板第 index 色（自动取模循环）
pub fn chart_color(index: usize) -> Hsla {
    chart_palette()[index % 8]
}

/// AntD 聚焦环：`0 0 0 lineWidthFocus colorPrimaryOutline`。
/// 输入框 / 下拉框获得焦点时与 `primary` 描边一起使用。
pub fn focus_ring() -> Vec<BoxShadow> {
    let (width, color) = with(|t| (t.line_width_focus, t.color_primary_outline));
    vec![BoxShadow {
        color,
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(width),
        inset: false,
    }]
}

/// 错误态聚焦环（`colorErrorOutline`）
pub fn focus_ring_error() -> Vec<BoxShadow> {
    let (width, color) = with(|t| (t.line_width_focus, t.color_error_outline));
    vec![BoxShadow {
        color,
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(width),
        inset: false,
    }]
}

/// 终端 ANSI 16 色：浅色底与深色底各一套。
///
/// 这两份是终端自己的配色板，不跟界面中性色派生：协议默认的亮白（`#e5e5e5`）
/// 压在白底上等于隐形，白底必须换成深一档的同族色。索引 0~7 常规、8~15 高亮，
/// 256 色扩展由终端控件自行换算。
const ANSI_LIGHT: [u32; 16] = [
    0x24292f, 0xcf222e, 0x116329, 0x7d4e00, 0x0969da, 0x8250df, 0x1b7c83, 0x6e7781, 0x57606a,
    0xa40e26, 0x1a7f37, 0x633c01, 0x218bff, 0xa475f9, 0x3192aa, 0x8c959f,
];

const ANSI_DARK: [u32; 16] = [
    0x15161a, 0xf7768e, 0x9ece6a, 0xe0af68, 0x7aa2f7, 0xbb9af7, 0x7dcfff, 0xc0caf5, 0x414868,
    0xff7a93, 0xb9f27c, 0xff9e64, 0x7da6ff, 0xc7a9ff, 0xa4daff, 0xe6e9ef,
];

pub fn terminal_ansi() -> [Hsla; 16] {
    terminal_ansi_for(THEME.read().unwrap().appearance)
}

/// 终端默认背景：浅色是白底（终端跟着界面一起翻亮），深色是近黑的蓝黑。
pub fn terminal_bg() -> Hsla {
    terminal_bg_for(THEME.read().unwrap().appearance)
}

/// 终端默认前景，与 `terminal_bg` 同一套明暗档。
pub fn terminal_fg() -> Hsla {
    terminal_fg_for(THEME.read().unwrap().appearance)
}

/// 按明暗形态取 ANSI 16 色，纯函数，供测试逐通道断言。
pub fn terminal_ansi_for(appearance: Appearance) -> [Hsla; 16] {
    let table = if appearance == Appearance::Dark {
        ANSI_DARK
    } else {
        ANSI_LIGHT
    };
    table.map(|value| rgb(value).into())
}

/// 按明暗形态取终端底色。
pub fn terminal_bg_for(appearance: Appearance) -> Hsla {
    rgb(if appearance == Appearance::Dark {
        0x101216
    } else {
        0xffffff
    })
    .into()
}

/// 按明暗形态取终端前景。
pub fn terminal_fg_for(appearance: Appearance) -> Hsla {
    rgb(if appearance == Appearance::Dark {
        0xd7dbe3
    } else {
        0x24292f
    })
    .into()
}
