// 生成 Descriptions 基准。
// 规则来源：antdv-next `descriptions/style/index.js` prepareComponentToken：
//   标签 colorTextTertiary / 带边框时 colorTextSecondary + colorFillAlter 底；
//   标题 colorText / fontSizeLG / fontWeightStrong；标题下距 fontSizeSM*lineHeightSM；
//   行间距 padding；带边框单元格 padding / paddingLG；冒号 marginXXS/2 与 marginXS；
//   外框圆角 borderRadiusLG、分隔线 colorSplit。
//
//   node harness/antd_baseline/gen_descriptions_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  const fields = [
    token.colorFillAlter,
    token.colorTextTertiary,
    token.colorTextSecondary,
    token.colorText,
    token.colorText,
    token.colorSplit,
    token.fontSizeLG,
    token.fontSizeLG * token.lineHeightLG,
    token.fontSizeSM * token.lineHeightSM,
    token.padding,
    token.padding,
    token.padding,
    token.paddingLG,
    token.marginXXS / 2,
    token.marginXS,
    token.borderRadiusLG,
    token.fontSize,
    token.lineHeight,
    token.fontWeightStrong,
  ];
  return fields.join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `descriptions_matrix_${name}.txt`), rows(token));
  console.log(`wrote descriptions_matrix_${name}.txt`);
}
