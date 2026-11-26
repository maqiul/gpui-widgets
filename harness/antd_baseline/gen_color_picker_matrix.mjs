// 生成 ColorPicker 基准。
// 规则来源：antdv-next `color-picker/style/index.js` + `color-block.js` + `presets.js`：
//   trigger min controlHeight / radius borderRadius / border lineWidth colorBorder / bg colorBgElevated /
//   padding paddingXXS-lineWidth；hover colorPrimaryHover；active colorPrimary + controlOutline 宽 controlOutlineWidth；
//   disabled colorTextDisabled + colorBgContainerDisabled；
//   color-block controlHeightSM / radius borderRadiusSM / 内描边 colorFillSecondary / inset colorTextQuaternary；
//   sm: trigger controlHeightSM radius SM block controlHeightXS radius XS；lg: controlHeightLG radius LG block controlHeight radius；
//   presets color 24 / gap marginXXS*1.5 / 勾 lineWidthBold colorWhite（bright 时 rgba(0,0,0,0.45)）/ hover 环 colorFill；
//   panel width 234 / popover padding paddingSM；trigger-text fontSize marginXS。
//
//   node harness/antd_baseline/gen_color_picker_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  return [
    token.colorBgElevated,
    token.colorBorder,
    token.colorPrimaryHover,
    token.colorPrimary,
    token.controlOutline,
    token.colorTextDisabled,
    token.colorBgContainerDisabled,
    token.colorFillSecondary,
    token.colorTextQuaternary,
    token.colorWhite,
    'rgba(0,0,0,0.45)',
    token.colorText,
    token.colorFill,
    token.controlHeight,
    token.controlHeightSM,
    token.controlHeightLG,
    token.borderRadius,
    token.borderRadiusSM,
    token.borderRadiusLG,
    token.paddingXXS - token.lineWidth,
    token.controlHeightSM,
    token.controlHeightXS,
    token.controlHeight,
    token.borderRadiusSM,
    token.borderRadiusXS,
    token.borderRadius,
    24,
    token.marginXXS * 1.5,
    token.lineWidthBold,
    234,
    token.paddingSM,
    token.fontSize,
    token.marginXS,
    token.controlOutlineWidth,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `color_picker_matrix_${name}.txt`), line(token));
  console.log(`wrote color_picker_matrix_${name}.txt`);
}