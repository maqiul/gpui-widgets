// 从 @ant-design/colors 的固定预置色阶生成 src/theme/preset.rs。
// antdv-next 的 Tag/FloatButton 预置色取 `${family}1 / 3 / 6 / 7`，
// 这些色阶与主色种子无关，可直接固化成令牌数据。
//
//   node harness/antd_baseline/gen_tag_presets.mjs
import fs from 'node:fs';
import path from 'node:path';
import { presetPalettes } from '@ant-design/colors';

const families = [
  ['Blue', 'blue'],
  ['Green', 'green'],
  ['Orange', 'orange'],
  ['Red', 'red'],
  ['Purple', 'purple'],
  ['Cyan', 'cyan'],
  ['Magenta', 'magenta'],
];

const arms = ['light', 'border', 'solid', 'text'];
const indexes = [0, 2, 5, 6];

const rows = families.map(([rust, key]) => {
  const p = presetPalettes[key];
  const values = indexes.map((i) => `0x${p[i].slice(1)}`);
  return `        PresetColor::${rust} => PresetPalette { ${arms
    .map((arm, i) => `${arm}: Color::from_hex(${values[i]})`)
    .join(', ')} },`;
});

const source = `//! antd 预置色板（固定色阶，与主色种子无关）。
//!
//! 生成脚本：\`harness/antd_baseline/gen_tag_presets.mjs\`，数据源 \`@ant-design/colors\`。
//! 取 \`family1 / family3 / family6 / family7\`：浅底 / 浅描边 / 实心底 / 文本色。

use crate::theme::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetColor {
${families.map(([rust]) => `    ${rust},`).join('\n')}
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresetPalette {
    /// 浅色底（\`family1\`）
    pub light: Color,
    /// 浅色描边（\`family3\`）
    pub border: Color,
    /// 实心底（\`family6\`）
    pub solid: Color,
    /// 文本色（\`family7\`）
    pub text: Color,
}

pub fn preset_palette(color: PresetColor) -> PresetPalette {
    match color {
${rows.join('\n')}
    }
}
`;

const out = path.resolve(import.meta.dirname, '../../src/theme/preset.rs');
fs.writeFileSync(out, source);
console.log(`wrote ${out} (${families.length} families)`);
