//! antdv-next 令牌体系的 Rust 版：Seed（种子）→ 派生 → Tokens（别名，控件层取色用）。
//!
//! 三层结构和 antdv-next 一一对应：
//! - Seed：`theme/themes/seed.js` 的 `seedToken`
//! - 派生：`theme/themes/{default,dark}/colors.js` + `themes/shared/*` + `theme/util/alias.js`
//! - Tokens：控件直接消费的别名令牌，字段名 = antd 令牌名（驼峰转蛇形），方便逐个对照官方文档
//!
//! 之所以全量派生而不是抄一份数值表：用户只改一个 `color_primary`，
//! antdv-next 会连带把 hover / active / 染底 / 描边 / focus ring 全部换掉，
//! 抄表就只能支持默认主色。

use gpui::{BoxShadow, Hsla, point, px};

use super::palette::{Color, alpha_over, error_bg_filled_hover, palette};

/// 明暗形态，对应 antdv-next 的 defaultAlgorithm / darkAlgorithm
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Appearance {
    Light,
    Dark,
}

/// 种子令牌：控件层不应该直接读它，改完要重新派生 Tokens。
#[derive(Clone, Copy, Debug)]
pub struct Seed {
    pub color_primary: Color,
    pub color_success: Color,
    pub color_warning: Color,
    pub color_error: Color,
    pub color_info: Color,
    /// `None` 时跟随 `color_info`，和 antd 的 `seed.colorLink || seed.colorInfo` 一致
    pub color_link: Option<Color>,
    /// `None` 时按明暗形态取 `#000` / `#fff`
    pub color_text_base: Option<Color>,
    /// `None` 时按明暗形态取 `#fff` / `#000`
    pub color_bg_base: Option<Color>,
    pub font_size: f32,
    pub line_width: f32,
    pub border_radius: f32,
    pub control_height: f32,
    pub size_step: f32,
    pub size_unit: f32,
    /// `motion: false` 时三档动画时长归零（antd 同名开关）
    pub motion: bool,
    /// `focus_outline: false` 时 lineWidthFocus 归零
    pub focus_outline: bool,
}

impl Default for Seed {
    fn default() -> Self {
        Self {
            color_primary: Color::from_hex(0x1677ff),
            color_success: Color::from_hex(0x52c41a),
            color_warning: Color::from_hex(0xfaad14),
            color_error: Color::from_hex(0xff4d4f),
            color_info: Color::from_hex(0x1677ff),
            color_link: None,
            color_text_base: None,
            color_bg_base: None,
            font_size: 14.0,
            line_width: 1.0,
            border_radius: 6.0,
            control_height: 32.0,
            size_step: 4.0,
            size_unit: 4.0,
            motion: true,
            focus_outline: true,
        }
    }
}

/// 控件可覆盖的阴影层级，对应 antd 的 boxShadow / boxShadowTertiary 等
#[derive(Clone, Debug)]
pub struct Shadows(pub Vec<BoxShadow>);

/// 派生后的别名令牌。字段全部公开：`theme::update` 允许在不重新派生的情况下微调个别令牌。
#[derive(Clone, Debug)]
pub struct Tokens {
    pub appearance: Appearance,

    // ---- 主色族 ----
    pub color_primary: Hsla,
    pub color_primary_hover: Hsla,
    pub color_primary_active: Hsla,
    pub color_primary_bg: Hsla,
    pub color_primary_bg_hover: Hsla,
    pub color_primary_border: Hsla,
    pub color_primary_border_hover: Hsla,
    pub color_primary_text: Hsla,
    pub color_primary_text_hover: Hsla,
    pub color_primary_text_active: Hsla,

    // ---- 功能色族 ----
    pub color_success: Hsla,
    pub color_success_hover: Hsla,
    pub color_success_active: Hsla,
    pub color_success_bg: Hsla,
    pub color_success_bg_hover: Hsla,
    pub color_success_border: Hsla,
    pub color_success_border_hover: Hsla,
    pub color_success_text: Hsla,
    pub color_success_text_hover: Hsla,
    pub color_success_text_active: Hsla,
    pub color_warning: Hsla,
    pub color_warning_hover: Hsla,
    pub color_warning_active: Hsla,
    pub color_warning_bg: Hsla,
    pub color_warning_bg_hover: Hsla,
    pub color_warning_border: Hsla,
    pub color_warning_border_hover: Hsla,
    pub color_warning_text: Hsla,
    pub color_warning_text_hover: Hsla,
    pub color_warning_text_active: Hsla,
    pub color_error: Hsla,
    pub color_error_hover: Hsla,
    pub color_error_active: Hsla,
    pub color_error_bg: Hsla,
    pub color_error_bg_hover: Hsla,
    pub color_error_bg_active: Hsla,
    pub color_error_bg_filled_hover: Hsla,
    pub color_error_border: Hsla,
    pub color_error_border_hover: Hsla,
    pub color_error_text: Hsla,
    pub color_error_text_hover: Hsla,
    pub color_error_text_active: Hsla,
    pub color_info: Hsla,
    pub color_info_hover: Hsla,
    pub color_info_active: Hsla,
    pub color_info_bg: Hsla,
    pub color_info_bg_hover: Hsla,
    pub color_info_border: Hsla,
    pub color_info_border_hover: Hsla,
    pub color_info_text: Hsla,
    pub color_info_text_hover: Hsla,
    pub color_info_text_active: Hsla,
    pub color_link: Hsla,
    pub color_link_hover: Hsla,
    pub color_link_active: Hsla,

    // ---- 中性色 ----
    pub color_text_base: Hsla,
    pub color_bg_base: Hsla,
    pub color_text: Hsla,
    pub color_text_secondary: Hsla,
    pub color_text_tertiary: Hsla,
    pub color_text_quaternary: Hsla,
    pub color_fill: Hsla,
    pub color_fill_secondary: Hsla,
    pub color_fill_tertiary: Hsla,
    pub color_fill_quaternary: Hsla,
    pub color_bg_layout: Hsla,
    pub color_bg_container: Hsla,
    pub color_bg_elevated: Hsla,
    pub color_bg_spotlight: Hsla,
    pub color_bg_mask: Hsla,
    pub color_bg_solid: Hsla,
    pub color_bg_solid_hover: Hsla,
    pub color_bg_solid_active: Hsla,
    pub color_border: Hsla,
    pub color_border_secondary: Hsla,
    pub color_white: Hsla,

    // ---- 中性色别名 ----
    pub color_text_placeholder: Hsla,
    pub color_text_disabled: Hsla,
    pub color_text_heading: Hsla,
    pub color_text_label: Hsla,
    pub color_text_description: Hsla,
    pub color_text_light_solid: Hsla,
    pub color_split: Hsla,
    pub color_border_bg: Hsla,
    pub color_bg_container_disabled: Hsla,
    pub color_bg_text_hover: Hsla,
    pub color_bg_text_active: Hsla,
    pub color_fill_content: Hsla,
    pub color_fill_content_hover: Hsla,
    pub color_fill_alter: Hsla,
    pub color_icon: Hsla,
    pub color_icon_hover: Hsla,
    pub color_highlight: Hsla,
    pub color_error_outline: Hsla,
    pub color_warning_outline: Hsla,
    pub color_primary_outline: Hsla,

    // ---- 交互态 ----
    pub control_item_bg_hover: Hsla,
    pub control_item_bg_active: Hsla,
    pub control_item_bg_active_hover: Hsla,
    pub control_item_bg_active_disabled: Hsla,
    pub control_tmp_outline: Hsla,

    // ---- 字体 ----
    pub font_size: f32,
    pub font_size_sm: f32,
    pub font_size_lg: f32,
    pub font_size_xl: f32,
    pub font_size_icon: f32,
    pub font_size_heading1: f32,
    pub font_size_heading2: f32,
    pub font_size_heading3: f32,
    pub font_size_heading4: f32,
    pub font_size_heading5: f32,
    pub line_height: f32,
    pub line_height_sm: f32,
    pub line_height_lg: f32,
    pub font_height: f32,
    pub font_height_sm: f32,
    pub font_height_lg: f32,
    pub font_weight_strong: f32,

    // ---- 控件尺寸 ----
    pub control_height: f32,
    pub control_height_sm: f32,
    pub control_height_xs: f32,
    pub control_height_lg: f32,
    pub control_padding_horizontal: f32,
    pub control_padding_horizontal_sm: f32,
    pub control_interactive_size: f32,
    pub control_outline_width: f32,

    // ---- 圆角 ----
    pub border_radius: f32,
    pub border_radius_xs: f32,
    pub border_radius_sm: f32,
    pub border_radius_lg: f32,
    pub border_radius_outer: f32,

    // ---- 栅格间距 ----
    pub size_xxs: f32,
    pub size_xs: f32,
    pub size_sm: f32,
    pub size_ms: f32,
    pub size: f32,
    pub size_md: f32,
    pub size_lg: f32,
    pub size_xl: f32,
    pub size_xxl: f32,
    pub padding: f32,
    pub padding_xxs: f32,
    pub padding_xs: f32,
    pub padding_sm: f32,
    pub padding_md: f32,
    pub padding_lg: f32,
    pub padding_xl: f32,
    pub margin: f32,
    pub margin_xxs: f32,
    pub margin_xs: f32,
    pub margin_sm: f32,
    pub margin_md: f32,
    pub margin_lg: f32,
    pub margin_xl: f32,
    pub margin_xxl: f32,

    // ---- 描边 ----
    pub line_width: f32,
    pub line_width_bold: f32,
    pub line_width_focus: f32,

    // ---- 动效 / 其它 ----
    pub motion_duration_fast: f32,
    pub motion_duration_mid: f32,
    pub motion_duration_slow: f32,
    pub opacity_loading: f32,
    pub z_index_popup_base: f32,

    // ---- 阴影 ----
    pub shadow: Shadows,
    pub shadow_secondary: Shadows,
    pub shadow_tertiary: Shadows,
    pub shadow_card: Shadows,
}

/// 字号阶梯，对应 `themes/shared/genFontSizes.ts`：返回 (10 档字号, 10 档行高)
fn gen_font_sizes(base: f32) -> ([f32; 10], [f32; 10]) {
    let mut sizes = [base; 10];
    for (index, size) in sizes.iter_mut().enumerate() {
        let i = index as f32 - 1.0;
        let base_size = base * std::f32::consts::E.powf(i / 5.0);
        *size = if index > 1 {
            (base_size.floor() / 2.0).floor() * 2.0
        } else {
            (base_size.ceil() / 2.0).floor() * 2.0
        };
    }
    sizes[1] = base;
    let heights = sizes.map(|size| (size + 8.0) / size);
    (sizes, heights)
}

/// 圆角阶梯，对应 `themes/shared/genRadius.ts`
fn gen_radius(base: f32) -> (f32, f32, f32, f32, f32) {
    let lg = if base < 6.0 && base >= 5.0 {
        base + 1.0
    } else if base < 16.0 && base >= 6.0 {
        base + 2.0
    } else if base >= 16.0 {
        16.0
    } else {
        base
    };
    let sm = if base < 7.0 && base >= 5.0 {
        4.0
    } else if base < 8.0 && base >= 7.0 {
        5.0
    } else if base < 14.0 && base >= 8.0 {
        6.0
    } else if base < 16.0 && base >= 14.0 {
        7.0
    } else if base >= 16.0 {
        8.0
    } else {
        base
    };
    let xs = if base < 6.0 && base >= 2.0 {
        1.0
    } else if base >= 6.0 {
        2.0
    } else {
        base
    };
    let outer = if base > 4.0 && base < 8.0 {
        4.0
    } else if base >= 8.0 {
        6.0
    } else {
        base
    };
    (base, xs, sm, lg, outer)
}

/// `getSolidColor(base, brightness)`：实色阶梯，antd 用它算 border / layout 背景。
/// 浅色主题用 `darken(#fff)`，深色主题用 `lighten(#000)`，两个方向不可混用。
fn solid(base: Color, brightness: f32, dark: bool) -> Color {
    if dark {
        base.lighten(brightness)
    } else {
        base.darken(brightness)
    }
}

/// `setA` 版 alpha 色：直接给基色加透明度
fn alpha(base: Color, a: f32) -> Color {
    base.with_alpha(a)
}

/// 阴影层定义：(offset_x, offset_y, blur, spread, alpha)
type ShadowLayer = (f32, f32, f32, f32, f32);

const SHADOW_PRIMARY: [ShadowLayer; 3] = [
    (0.0, 6.0, 16.0, 0.0, 0.08),
    (0.0, 3.0, 6.0, -4.0, 0.12),
    (0.0, 9.0, 28.0, 8.0, 0.05),
];
const SHADOW_TERTIARY: [ShadowLayer; 3] = [
    (0.0, 1.0, 2.0, 0.0, 0.05),
    (0.0, 1.0, 6.0, -1.0, 0.03),
    (0.0, 2.0, 4.0, 0.0, 0.03),
];
const SHADOW_CARD: [ShadowLayer; 3] = [
    (0.0, 1.0, 2.0, -2.0, 0.16),
    (0.0, 3.0, 6.0, 0.0, 0.12),
    (0.0, 5.0, 12.0, 4.0, 0.09),
];

fn build_shadows(base: Color, layers: &[ShadowLayer]) -> Shadows {
    Shadows(
        layers
            .iter()
            .map(|(x, y, blur, spread, a)| BoxShadow {
                color: base.with_alpha(base.a * a).hsla(),
                offset: point(px(*x), px(*y)),
                blur_radius: px(*blur),
                spread_radius: px(*spread),
                inset: false,
            })
            .collect(),
    )
}

/// 一族功能色的别名取色：`palette` 已是 1~10 档（下标 0~9）。
#[derive(Clone, Copy)]
struct Family {
    bg: usize,
    bg_hover: usize,
    border: usize,
    border_hover: usize,
    hover: usize,
    base: usize,
    active: usize,
    text_hover: usize,
    text: usize,
    text_active: usize,
}

/// primary / error 族：hover 用独立的第 5 档
const FAMILY_HARD: Family = Family {
    bg: 0,
    bg_hover: 1,
    border: 2,
    border_hover: 3,
    hover: 4,
    base: 5,
    active: 6,
    text_hover: 7,
    text: 8,
    text_active: 9,
};

/// success / warning / info 族：antd 让 hover 复用 borderHover（`xColors[4]`）
const FAMILY_SOFT: Family = Family {
    bg: 0,
    bg_hover: 1,
    border: 2,
    border_hover: 3,
    hover: 3,
    base: 5,
    active: 6,
    text_hover: 7,
    text: 8,
    text_active: 9,
};

/// error 族独有的第 3 档复用（bgActive 与 border 同值）
const ERROR_BG_ACTIVE: usize = 2;

/// 由种子 + 明暗形态派生全部别名令牌。
pub fn derive(seed: &Seed, appearance: Appearance) -> Tokens {
    let dark = appearance == Appearance::Dark;
    let bg_base =
        seed.color_bg_base
            .unwrap_or(Color::from_hex(if dark { 0x000000 } else { 0xffffff }));
    let text_base =
        seed.color_text_base
            .unwrap_or(Color::from_hex(if dark { 0xffffff } else { 0x000000 }));
    let shadow_base = if dark {
        Color::from_hex(0xffffff).with_alpha(0.2)
    } else {
        Color::from_hex(0x000000)
    };
    // generate(color, { theme: 'dark' }) 的混色基线固定是 #141414
    let palette_bg = Color::from_hex(0x141414);

    let primary = palette(seed.color_primary, dark, palette_bg);
    let success = palette(seed.color_success, dark, palette_bg);
    let warning = palette(seed.color_warning, dark, palette_bg);
    let error = palette(seed.color_error, dark, palette_bg);
    let info = palette(seed.color_info, dark, palette_bg);
    let link = palette(seed.color_link.unwrap_or(seed.color_info), dark, palette_bg);

    // 按别名档位取出色板中的 Color（调用处统一 `.hsla()`）
    let pick = |pal: &[Color; 10], f: &Family| {
        (
            pal[f.bg],
            pal[f.bg_hover],
            pal[f.border],
            pal[f.border_hover],
            pal[f.hover],
            pal[f.base],
            pal[f.active],
            pal[f.text],
            pal[f.text_hover],
            pal[f.text_active],
        )
    };

    let (
        primary_bg,
        primary_bg_hover,
        primary_border,
        primary_border_hover,
        primary_hover,
        primary_color,
        primary_active,
        primary_text,
        primary_text_hover,
        primary_text_active,
    ) = pick(&primary, &FAMILY_HARD);
    // 深色下 antd 用描边档当染底档（themes/dark/index.js 末尾的两行覆盖）
    let (primary_bg, primary_bg_hover) = if dark {
        (primary_border, primary_border_hover)
    } else {
        (primary_bg, primary_bg_hover)
    };

    let (
        success_bg,
        success_bg_hover,
        success_border,
        success_border_hover,
        success_hover,
        success_color,
        success_active,
        success_text,
        success_text_hover,
        success_text_active,
    ) = pick(&success, &FAMILY_SOFT);
    let (
        warning_bg,
        warning_bg_hover,
        warning_border,
        warning_border_hover,
        warning_hover,
        warning_color,
        warning_active,
        warning_text,
        warning_text_hover,
        warning_text_active,
    ) = pick(&warning, &FAMILY_SOFT);
    let (
        info_bg,
        info_bg_hover,
        info_border,
        info_border_hover,
        info_hover,
        info_color,
        info_active,
        info_text,
        info_text_hover,
        info_text_active,
    ) = pick(&info, &FAMILY_SOFT);
    let (
        error_bg,
        error_bg_hover,
        error_border,
        error_border_hover,
        error_hover,
        error_color,
        error_active,
        error_text,
        error_text_hover,
        error_text_active,
    ) = pick(&error, &FAMILY_HARD);
    let error_bg_active = error[ERROR_BG_ACTIVE];
    let error_bg_filled_hover = error_bg_filled_hover(&error);

    // ---- 中性色 ----
    let (text_a, text_a2, text_a3, text_a4) = if dark {
        (0.85, 0.65, 0.45, 0.25)
    } else {
        (0.88, 0.65, 0.45, 0.25)
    };
    let (fill_a, fill_a2, fill_a3, fill_a4) = if dark {
        (0.18, 0.12, 0.08, 0.04)
    } else {
        (0.15, 0.06, 0.04, 0.02)
    };
    let color_text = alpha(text_base, text_a);
    let color_text_secondary = alpha(text_base, text_a2);
    let color_text_tertiary = alpha(text_base, text_a3);
    let color_text_quaternary = alpha(text_base, text_a4);
    let color_fill = alpha(text_base, fill_a);
    let color_fill_secondary = alpha(text_base, fill_a2);
    let color_fill_tertiary = alpha(text_base, fill_a3);
    let color_fill_quaternary = alpha(text_base, fill_a4);
    let color_bg_layout = solid(bg_base, if dark { 0.0 } else { 4.0 }, dark);
    let color_bg_container = solid(bg_base, if dark { 8.0 } else { 0.0 }, dark);
    let color_bg_elevated = solid(bg_base, if dark { 12.0 } else { 0.0 }, dark);
    let color_bg_spotlight = if dark {
        solid(bg_base, 26.0, true)
    } else {
        alpha(text_base, 0.85)
    };
    let color_border = solid(bg_base, if dark { 26.0 } else { 15.0 }, dark);
    let color_border_secondary = solid(bg_base, if dark { 19.0 } else { 6.0 }, dark);
    let (bg_solid, bg_solid_hover, bg_solid_active) = if dark {
        (
            alpha(text_base, 0.95),
            alpha(text_base, 1.0),
            alpha(text_base, 0.9),
        )
    } else {
        (
            alpha(text_base, 1.0),
            alpha(text_base, 0.75),
            alpha(text_base, 0.95),
        )
    };

    // ---- 中性色别名 ----
    let color_split = alpha_over(color_border_secondary, color_bg_container);
    // outline 取各自色板的第 1 档（= colorXxxBg），与 antd util/alias.js 一致
    let color_error_outline = alpha_over(error[0], color_bg_container);
    let color_warning_outline = alpha_over(warning[0], color_bg_container);
    let color_primary_outline = alpha_over(primary_bg, color_bg_container);

    // ---- 字号 / 行高 ----
    let (sizes, heights) = gen_font_sizes(seed.font_size);
    let (radius, radius_xs, radius_sm, radius_lg, radius_outer) = gen_radius(seed.border_radius);

    // ---- 尺寸阶梯 ----
    let size_xxl = seed.size_unit * (seed.size_step + 8.0);
    let size_xl = seed.size_unit * (seed.size_step + 4.0);
    let size_lg = seed.size_unit * (seed.size_step + 2.0);
    let size_md = seed.size_unit * (seed.size_step + 1.0);
    let size_ms = seed.size_unit * seed.size_step;
    let size_base = seed.size_unit * seed.size_step;
    let size_sm = seed.size_unit * (seed.size_step - 1.0);
    let size_xs = seed.size_unit * (seed.size_step - 2.0);
    let size_xxs = seed.size_unit * (seed.size_step - 3.0);

    let (duration_fast, duration_mid, duration_slow) = if seed.motion {
        (0.1, 0.2, 0.3)
    } else {
        (0.0, 0.0, 0.0)
    };

    Tokens {
        appearance,

        color_primary: primary_color.hsla(),
        color_primary_hover: primary_hover.hsla(),
        color_primary_active: primary_active.hsla(),
        color_primary_bg: primary_bg.hsla(),
        color_primary_bg_hover: primary_bg_hover.hsla(),
        color_primary_border: primary_border.hsla(),
        color_primary_border_hover: primary_border_hover.hsla(),
        color_primary_text: primary_text.hsla(),
        color_primary_text_hover: primary_text_hover.hsla(),
        color_primary_text_active: primary_text_active.hsla(),

        color_success: success_color.hsla(),
        color_success_hover: success_hover.hsla(),
        color_success_active: success_active.hsla(),
        color_success_bg: success_bg.hsla(),
        color_success_bg_hover: success_bg_hover.hsla(),
        color_success_border: success_border.hsla(),
        color_success_border_hover: success_border_hover.hsla(),
        color_success_text: success_text.hsla(),
        color_success_text_hover: success_text_hover.hsla(),
        color_success_text_active: success_text_active.hsla(),
        color_warning: warning_color.hsla(),
        color_warning_hover: warning_hover.hsla(),
        color_warning_active: warning_active.hsla(),
        color_warning_bg: warning_bg.hsla(),
        color_warning_bg_hover: warning_bg_hover.hsla(),
        color_warning_border: warning_border.hsla(),
        color_warning_border_hover: warning_border_hover.hsla(),
        color_warning_text: warning_text.hsla(),
        color_warning_text_hover: warning_text_hover.hsla(),
        color_warning_text_active: warning_text_active.hsla(),
        color_error: error_color.hsla(),
        color_error_hover: error_hover.hsla(),
        color_error_active: error_active.hsla(),
        color_error_bg: error_bg.hsla(),
        color_error_bg_hover: error_bg_hover.hsla(),
        color_error_bg_active: error_bg_active.hsla(),
        color_error_bg_filled_hover: error_bg_filled_hover.hsla(),
        color_error_border: error_border.hsla(),
        color_error_border_hover: error_border_hover.hsla(),
        color_error_text: error_text.hsla(),
        color_error_text_hover: error_text_hover.hsla(),
        color_error_text_active: error_text_active.hsla(),
        color_info: info_color.hsla(),
        color_info_hover: info_hover.hsla(),
        color_info_active: info_active.hsla(),
        color_info_bg: info_bg.hsla(),
        color_info_bg_hover: info_bg_hover.hsla(),
        color_info_border: info_border.hsla(),
        color_info_border_hover: info_border_hover.hsla(),
        color_info_text: info_text.hsla(),
        color_info_text_hover: info_text_hover.hsla(),
        color_info_text_active: info_text_active.hsla(),
        color_link: link[5].hsla(),
        color_link_hover: link[3].hsla(),
        color_link_active: link[6].hsla(),

        color_text_base: text_base.hsla(),
        color_bg_base: bg_base.hsla(),
        color_text: color_text.hsla(),
        color_text_secondary: color_text_secondary.hsla(),
        color_text_tertiary: color_text_tertiary.hsla(),
        color_text_quaternary: color_text_quaternary.hsla(),
        color_fill: color_fill.hsla(),
        color_fill_secondary: color_fill_secondary.hsla(),
        color_fill_tertiary: color_fill_tertiary.hsla(),
        color_fill_quaternary: color_fill_quaternary.hsla(),
        color_bg_layout: color_bg_layout.hsla(),
        color_bg_container: color_bg_container.hsla(),
        color_bg_elevated: color_bg_elevated.hsla(),
        color_bg_spotlight: color_bg_spotlight.hsla(),
        color_bg_mask: Color::from_hex(0x000000).with_alpha(0.45).hsla(),
        color_bg_solid: bg_solid.hsla(),
        color_bg_solid_hover: bg_solid_hover.hsla(),
        color_bg_solid_active: bg_solid_active.hsla(),
        color_border: color_border.hsla(),
        color_border_secondary: color_border_secondary.hsla(),
        color_white: Color::from_hex(0xffffff).hsla(),

        color_text_placeholder: color_text_quaternary.hsla(),
        color_text_disabled: color_text_quaternary.hsla(),
        color_text_heading: color_text.hsla(),
        color_text_label: color_text_secondary.hsla(),
        color_text_description: color_text_tertiary.hsla(),
        color_text_light_solid: Color::from_hex(0xffffff).hsla(),
        color_split: color_split.hsla(),
        color_border_bg: color_bg_container.hsla(),
        color_bg_container_disabled: color_fill_tertiary.hsla(),
        color_bg_text_hover: color_fill_secondary.hsla(),
        color_bg_text_active: color_fill.hsla(),
        color_fill_content: color_fill_secondary.hsla(),
        color_fill_content_hover: color_fill.hsla(),
        color_fill_alter: color_fill_quaternary.hsla(),
        color_icon: color_text_tertiary.hsla(),
        color_icon_hover: color_text.hsla(),
        color_highlight: error_color.hsla(),
        color_error_outline: color_error_outline.hsla(),
        color_warning_outline: color_warning_outline.hsla(),
        color_primary_outline: color_primary_outline.hsla(),

        control_item_bg_hover: color_fill_tertiary.hsla(),
        control_item_bg_active: primary_bg.hsla(),
        control_item_bg_active_hover: primary_bg_hover.hsla(),
        control_item_bg_active_disabled: color_fill.hsla(),
        control_tmp_outline: color_fill_quaternary.hsla(),

        font_size: sizes[1],
        font_size_sm: sizes[0],
        font_size_lg: sizes[2],
        font_size_xl: sizes[3],
        font_size_icon: sizes[0],
        font_size_heading1: sizes[6],
        font_size_heading2: sizes[5],
        font_size_heading3: sizes[4],
        font_size_heading4: sizes[3],
        font_size_heading5: sizes[2],
        line_height: heights[1],
        line_height_sm: heights[0],
        line_height_lg: heights[2],
        font_height: (heights[1] * sizes[1]).round(),
        font_height_sm: (heights[0] * sizes[0]).round(),
        font_height_lg: (heights[2] * sizes[2]).round(),
        font_weight_strong: 600.0,

        control_height: seed.control_height,
        control_height_sm: seed.control_height * 0.75,
        control_height_xs: seed.control_height * 0.5,
        control_height_lg: seed.control_height * 1.25,
        control_padding_horizontal: 12.0,
        control_padding_horizontal_sm: 8.0,
        control_interactive_size: seed.control_height / 2.0,
        control_outline_width: seed.line_width * 2.0,

        border_radius: radius,
        border_radius_xs: radius_xs,
        border_radius_sm: radius_sm,
        border_radius_lg: radius_lg,
        border_radius_outer: radius_outer,

        size_xxs,
        size_xs,
        size_sm,
        size_ms,
        size: size_base,
        size_md,
        size_lg,
        size_xl,
        size_xxl,
        padding: size_base,
        padding_xxs: size_xxs,
        padding_xs: size_xs,
        padding_sm: size_sm,
        padding_md: size_md,
        padding_lg: size_lg,
        padding_xl: size_xl,
        margin: size_base,
        margin_xxs: size_xxs,
        margin_xs: size_xs,
        margin_sm: size_sm,
        margin_md: size_md,
        margin_lg: size_lg,
        margin_xl: size_xl,
        margin_xxl: size_xxl,

        line_width: seed.line_width,
        line_width_bold: seed.line_width + 1.0,
        line_width_focus: if seed.focus_outline {
            seed.line_width * 3.0
        } else {
            0.0
        },

        motion_duration_fast: duration_fast,
        motion_duration_mid: duration_mid,
        motion_duration_slow: duration_slow,
        opacity_loading: 0.65,
        z_index_popup_base: 1000.0,

        shadow: build_shadows(shadow_base, &SHADOW_PRIMARY),
        shadow_secondary: build_shadows(shadow_base, &SHADOW_PRIMARY),
        shadow_tertiary: build_shadows(shadow_base, &SHADOW_TERTIARY),
        shadow_card: build_shadows(shadow_base, &SHADOW_CARD),
    }
}
