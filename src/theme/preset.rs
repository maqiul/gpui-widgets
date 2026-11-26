//! antd 预置色板（固定色阶，与主色种子无关）。
//!
//! 生成脚本：`harness/antd_baseline/gen_tag_presets.mjs`，数据源 `@ant-design/colors`。
//! 取 `family1 / family3 / family6 / family7`：浅底 / 浅描边 / 实心底 / 文本色。

use crate::theme::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetColor {
    Blue,
    Green,
    Orange,
    Red,
    Purple,
    Cyan,
    Magenta,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresetPalette {
    /// 浅色底（`family1`）
    pub light: Color,
    /// 浅色描边（`family3`）
    pub border: Color,
    /// 实心底（`family6`）
    pub solid: Color,
    /// 文本色（`family7`）
    pub text: Color,
}

pub fn preset_palette(color: PresetColor) -> PresetPalette {
    match color {
        PresetColor::Blue => PresetPalette { light: Color::from_hex(0xe6f4ff), border: Color::from_hex(0x91caff), solid: Color::from_hex(0x1677ff), text: Color::from_hex(0x0958d9) },
        PresetColor::Green => PresetPalette { light: Color::from_hex(0xf6ffed), border: Color::from_hex(0xb7eb8f), solid: Color::from_hex(0x52c41a), text: Color::from_hex(0x389e0d) },
        PresetColor::Orange => PresetPalette { light: Color::from_hex(0xfff7e6), border: Color::from_hex(0xffd591), solid: Color::from_hex(0xfa8c16), text: Color::from_hex(0xd46b08) },
        PresetColor::Red => PresetPalette { light: Color::from_hex(0xfff1f0), border: Color::from_hex(0xffa39e), solid: Color::from_hex(0xf5222d), text: Color::from_hex(0xcf1322) },
        PresetColor::Purple => PresetPalette { light: Color::from_hex(0xf9f0ff), border: Color::from_hex(0xd3adf7), solid: Color::from_hex(0x722ed1), text: Color::from_hex(0x531dab) },
        PresetColor::Cyan => PresetPalette { light: Color::from_hex(0xe6fffb), border: Color::from_hex(0x87e8de), solid: Color::from_hex(0x13c2c2), text: Color::from_hex(0x08979c) },
        PresetColor::Magenta => PresetPalette { light: Color::from_hex(0xfff0f6), border: Color::from_hex(0xffadd2), solid: Color::from_hex(0xeb2f96), text: Color::from_hex(0xc41d7f) },
    }
}
