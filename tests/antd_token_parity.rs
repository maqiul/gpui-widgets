//! 与 antdv-next 1.7.1 实测令牌对齐的回归测试。
//!
//! 基准值来自 `.ref/tokens-*.json`（由 `.ref/dump_tokens.mjs` 在真实 antdv-next 运行时里导出）。
//! `tests/data/*.rs` 是生成物：改完本文件的 `derived()` 名单后重新生成——
//!
//! ```ignore
//! cd widgets/.ref && node gen_baseline_rs.mjs
//! ```
//!
//! 名单与本文件里的令牌名一一对应，漏名 / 改名都会在这里显式失败，而不是静默跳过。

use gpui::Hsla;
use gpui_widgets::theme::{Appearance, Color, Seed, Tokens, derive};

#[derive(Clone, Copy, Debug)]
pub enum Value {
    /// rgba 分量：r/g/b 为 0~255，a 为 0~1
    C(u8, u8, u8, f32),
    N(f64),
}

#[path = "data/accent_green.rs"]
mod accent_green;
#[path = "data/accent_violet.rs"]
mod accent_violet;
#[path = "data/accent_violet_dark.rs"]
mod accent_violet_dark;
#[path = "data/dark.rs"]
mod dark;
#[path = "data/light.rs"]
mod light;

fn c(color: Hsla) -> Value {
    let color = Color::from(color);
    let q = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    Value::C(q(color.r), q(color.g), q(color.b), color.a)
}

fn n(v: f32) -> Value {
    Value::N(v as f64)
}

/// 派生令牌 → antdv-next 令牌名。名字必须与 JSON 基准里的 key 完全一致。
fn derived(t: &Tokens) -> Vec<(&'static str, Value)> {
    let out: Vec<(&'static str, Value)> = vec![
        // ---- 主色族 ----
        ("colorPrimaryBg", c(t.color_primary_bg)),
        ("colorPrimaryBgHover", c(t.color_primary_bg_hover)),
        ("colorPrimaryBorder", c(t.color_primary_border)),
        ("colorPrimaryBorderHover", c(t.color_primary_border_hover)),
        ("colorPrimaryHover", c(t.color_primary_hover)),
        ("colorPrimary", c(t.color_primary)),
        ("colorPrimaryActive", c(t.color_primary_active)),
        ("colorPrimaryTextHover", c(t.color_primary_text_hover)),
        ("colorPrimaryText", c(t.color_primary_text)),
        ("colorPrimaryTextActive", c(t.color_primary_text_active)),
        // ---- 功能色族 ----
        ("colorSuccessBg", c(t.color_success_bg)),
        ("colorSuccessBgHover", c(t.color_success_bg_hover)),
        ("colorSuccessBorder", c(t.color_success_border)),
        ("colorSuccessBorderHover", c(t.color_success_border_hover)),
        ("colorSuccessHover", c(t.color_success_hover)),
        ("colorSuccess", c(t.color_success)),
        ("colorSuccessActive", c(t.color_success_active)),
        ("colorSuccessTextHover", c(t.color_success_text_hover)),
        ("colorSuccessText", c(t.color_success_text)),
        ("colorSuccessTextActive", c(t.color_success_text_active)),
        ("colorWarningBg", c(t.color_warning_bg)),
        ("colorWarningBgHover", c(t.color_warning_bg_hover)),
        ("colorWarningBorder", c(t.color_warning_border)),
        ("colorWarningBorderHover", c(t.color_warning_border_hover)),
        ("colorWarningHover", c(t.color_warning_hover)),
        ("colorWarning", c(t.color_warning)),
        ("colorWarningActive", c(t.color_warning_active)),
        ("colorWarningTextHover", c(t.color_warning_text_hover)),
        ("colorWarningText", c(t.color_warning_text)),
        ("colorWarningTextActive", c(t.color_warning_text_active)),
        ("colorErrorBg", c(t.color_error_bg)),
        ("colorErrorBgHover", c(t.color_error_bg_hover)),
        ("colorErrorBgActive", c(t.color_error_bg_active)),
        ("colorErrorBgFilledHover", c(t.color_error_bg_filled_hover)),
        ("colorErrorBorder", c(t.color_error_border)),
        ("colorErrorBorderHover", c(t.color_error_border_hover)),
        ("colorErrorHover", c(t.color_error_hover)),
        ("colorError", c(t.color_error)),
        ("colorErrorActive", c(t.color_error_active)),
        ("colorErrorTextHover", c(t.color_error_text_hover)),
        ("colorErrorText", c(t.color_error_text)),
        ("colorErrorTextActive", c(t.color_error_text_active)),
        ("colorInfoBg", c(t.color_info_bg)),
        ("colorInfoBgHover", c(t.color_info_bg_hover)),
        ("colorInfoBorder", c(t.color_info_border)),
        ("colorInfoBorderHover", c(t.color_info_border_hover)),
        ("colorInfoHover", c(t.color_info_hover)),
        ("colorInfo", c(t.color_info)),
        ("colorInfoActive", c(t.color_info_active)),
        ("colorInfoTextHover", c(t.color_info_text_hover)),
        ("colorInfoText", c(t.color_info_text)),
        ("colorInfoTextActive", c(t.color_info_text_active)),
        ("colorLinkHover", c(t.color_link_hover)),
        ("colorLink", c(t.color_link)),
        ("colorLinkActive", c(t.color_link_active)),
        // ---- 中性色 ----
        ("colorTextBase", c(t.color_text_base)),
        ("colorBgBase", c(t.color_bg_base)),
        ("colorText", c(t.color_text)),
        ("colorTextSecondary", c(t.color_text_secondary)),
        ("colorTextTertiary", c(t.color_text_tertiary)),
        ("colorTextQuaternary", c(t.color_text_quaternary)),
        ("colorFill", c(t.color_fill)),
        ("colorFillSecondary", c(t.color_fill_secondary)),
        ("colorFillTertiary", c(t.color_fill_tertiary)),
        ("colorFillQuaternary", c(t.color_fill_quaternary)),
        ("colorBgLayout", c(t.color_bg_layout)),
        ("colorBgContainer", c(t.color_bg_container)),
        ("colorBgElevated", c(t.color_bg_elevated)),
        ("colorBgSpotlight", c(t.color_bg_spotlight)),
        ("colorBgMask", c(t.color_bg_mask)),
        ("colorBgSolid", c(t.color_bg_solid)),
        ("colorBgSolidHover", c(t.color_bg_solid_hover)),
        ("colorBgSolidActive", c(t.color_bg_solid_active)),
        ("colorBorder", c(t.color_border)),
        ("colorBorderSecondary", c(t.color_border_secondary)),
        ("colorWhite", c(t.color_white)),
        // ---- 中性色别名 ----
        ("colorTextPlaceholder", c(t.color_text_placeholder)),
        ("colorTextDisabled", c(t.color_text_disabled)),
        ("colorTextHeading", c(t.color_text_heading)),
        ("colorTextLabel", c(t.color_text_label)),
        ("colorTextDescription", c(t.color_text_description)),
        ("colorTextLightSolid", c(t.color_text_light_solid)),
        ("colorSplit", c(t.color_split)),
        ("colorBorderBg", c(t.color_border_bg)),
        ("colorBgContainerDisabled", c(t.color_bg_container_disabled)),
        ("colorBgTextHover", c(t.color_bg_text_hover)),
        ("colorBgTextActive", c(t.color_bg_text_active)),
        ("colorFillContent", c(t.color_fill_content)),
        ("colorFillContentHover", c(t.color_fill_content_hover)),
        ("colorFillAlter", c(t.color_fill_alter)),
        ("colorIcon", c(t.color_icon)),
        ("colorIconHover", c(t.color_icon_hover)),
        ("colorHighlight", c(t.color_highlight)),
        ("colorErrorOutline", c(t.color_error_outline)),
        ("colorWarningOutline", c(t.color_warning_outline)),
        ("controlOutline", c(t.color_primary_outline)),
        ("controlItemBgHover", c(t.control_item_bg_hover)),
        ("controlItemBgActive", c(t.control_item_bg_active)),
        (
            "controlItemBgActiveHover",
            c(t.control_item_bg_active_hover),
        ),
        (
            "controlItemBgActiveDisabled",
            c(t.control_item_bg_active_disabled),
        ),
        ("controlTmpOutline", c(t.control_tmp_outline)),
        // ---- 字体 ----
        ("fontSize", n(t.font_size)),
        ("fontSizeSM", n(t.font_size_sm)),
        ("fontSizeLG", n(t.font_size_lg)),
        ("fontSizeXL", n(t.font_size_xl)),
        ("fontSizeIcon", n(t.font_size_icon)),
        ("fontSizeHeading1", n(t.font_size_heading1)),
        ("fontSizeHeading2", n(t.font_size_heading2)),
        ("fontSizeHeading3", n(t.font_size_heading3)),
        ("fontSizeHeading4", n(t.font_size_heading4)),
        ("fontSizeHeading5", n(t.font_size_heading5)),
        ("lineHeight", n(t.line_height)),
        ("lineHeightSM", n(t.line_height_sm)),
        ("lineHeightLG", n(t.line_height_lg)),
        ("fontHeight", n(t.font_height)),
        ("fontHeightSM", n(t.font_height_sm)),
        ("fontHeightLG", n(t.font_height_lg)),
        ("fontWeightStrong", n(t.font_weight_strong)),
        // ---- 控件尺寸 / 圆角 ----
        ("controlHeight", n(t.control_height)),
        ("controlHeightSM", n(t.control_height_sm)),
        ("controlHeightXS", n(t.control_height_xs)),
        ("controlHeightLG", n(t.control_height_lg)),
        ("controlPaddingHorizontal", n(t.control_padding_horizontal)),
        (
            "controlPaddingHorizontalSM",
            n(t.control_padding_horizontal_sm),
        ),
        ("controlInteractiveSize", n(t.control_interactive_size)),
        ("controlOutlineWidth", n(t.control_outline_width)),
        ("borderRadius", n(t.border_radius)),
        ("borderRadiusXS", n(t.border_radius_xs)),
        ("borderRadiusSM", n(t.border_radius_sm)),
        ("borderRadiusLG", n(t.border_radius_lg)),
        ("borderRadiusOuter", n(t.border_radius_outer)),
        // ---- 间距阶梯 ----
        ("sizeXXS", n(t.size_xxs)),
        ("sizeXS", n(t.size_xs)),
        ("sizeSM", n(t.size_sm)),
        ("sizeMS", n(t.size_ms)),
        ("size", n(t.size)),
        ("sizeMD", n(t.size_md)),
        ("sizeLG", n(t.size_lg)),
        ("sizeXL", n(t.size_xl)),
        ("sizeXXL", n(t.size_xxl)),
        ("paddingXXS", n(t.padding_xxs)),
        ("paddingXS", n(t.padding_xs)),
        ("paddingSM", n(t.padding_sm)),
        ("padding", n(t.padding)),
        ("paddingMD", n(t.padding_md)),
        ("paddingLG", n(t.padding_lg)),
        ("paddingXL", n(t.padding_xl)),
        ("marginXXS", n(t.margin_xxs)),
        ("marginXS", n(t.margin_xs)),
        ("marginSM", n(t.margin_sm)),
        ("margin", n(t.margin)),
        ("marginMD", n(t.margin_md)),
        ("marginLG", n(t.margin_lg)),
        ("marginXL", n(t.margin_xl)),
        ("marginXXL", n(t.margin_xxl)),
        // ---- 描边 / 动效 ----
        ("lineWidth", n(t.line_width)),
        ("lineWidthBold", n(t.line_width_bold)),
        ("lineWidthFocus", n(t.line_width_focus)),
        ("motionDurationFast", n(t.motion_duration_fast)),
        ("motionDurationMid", n(t.motion_duration_mid)),
        ("motionDurationSlow", n(t.motion_duration_slow)),
        ("opacityLoading", n(t.opacity_loading)),
        ("zIndexPopupBase", n(t.z_index_popup_base)),
    ];
    out
}

fn compare(scenario: &str, expected: &[(&str, Value)], actual: &[(&'static str, Value)]) {
    let mut diffs = Vec::new();
    for (name, want) in expected {
        let Some((_, got)) = actual.iter().find(|(n, _)| n == name) else {
            diffs.push(format!("{scenario}: {name} 未在 derived() 中覆盖"));
            continue;
        };
        match (want, got) {
            (Value::C(wr, wg, wb, wa), Value::C(gr, gg, gb, ga)) => {
                // Hsla 往返会带来 1 个色阶的取整误差，透明度允许 0.005 偏差
                let ch = |a: u8, b: u8| (a as i32 - b as i32).abs();
                if ch(*wr, *gr) > 1
                    || ch(*wg, *gg) > 1
                    || ch(*wb, *gb) > 1
                    || (wa - ga).abs() > 0.005
                {
                    diffs.push(format!(
                        "{scenario}: {name} 期望 rgba({wr},{wg},{wb},{wa})，实际 rgba({gr},{gg},{gb},{ga})"
                    ));
                }
            }
            (Value::N(w), Value::N(g)) => {
                if (w - g).abs() > 1e-3 {
                    diffs.push(format!("{scenario}: {name} 期望 {w}，实际 {g}"));
                }
            }
            _ => diffs.push(format!("{scenario}: {name} 类型不匹配（颜色 vs 数值）")),
        }
    }
    // 反向检查：derived() 里不能有基准里不存在的名字（拼错 / 改名）
    for (name, _) in actual {
        if !expected.iter().any(|(n, _)| n == name) {
            diffs.push(format!("{scenario}: derived() 中的 {name} 在基准里不存在"));
        }
    }
    assert!(diffs.is_empty(), "{}\n", diffs.join("\n"));
}

fn violet_seed() -> Seed {
    let mut seed = Seed::default();
    seed.color_primary = Color::from_hex(0x8b5cf6);
    seed
}

fn green_seed() -> Seed {
    let mut seed = Seed::default();
    seed.color_primary = Color::from_hex(0x22c55e);
    seed
}

#[test]
fn light_matches_antdv_next() {
    let t = derive(&Seed::default(), Appearance::Light);
    compare("light", light::EXPECTED, &derived(&t));
}

#[test]
fn dark_matches_antdv_next() {
    let t = derive(&Seed::default(), Appearance::Dark);
    compare("dark", dark::EXPECTED, &derived(&t));
}

#[test]
fn accent_violet_light_matches_antdv_next() {
    let t = derive(&violet_seed(), Appearance::Light);
    compare("violet-light", accent_violet::EXPECTED, &derived(&t));
}

#[test]
fn accent_violet_dark_matches_antdv_next() {
    let t = derive(&violet_seed(), Appearance::Dark);
    compare("violet-dark", accent_violet_dark::EXPECTED, &derived(&t));
}

#[test]
fn accent_green_light_matches_antdv_next() {
    let t = derive(&green_seed(), Appearance::Light);
    compare("green-light", accent_green::EXPECTED, &derived(&t));
}
