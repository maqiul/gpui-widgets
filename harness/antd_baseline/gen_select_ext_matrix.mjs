// 生成 Cascader / TreeSelect 基准。
// 规则来源：antdv-next `cascader/style/index.js` prepareComponentToken +
// `cascader/style/columns.js` getColumnsStyle + `tree-select/style/index.js` genBaseStyle：
//   Cascader：controlWidth 184 / controlItemWidth 111 / dropdownHeight 180；
//   optionPadding = round((controlHeight − fontSize×lineHeight)/2) × paddingSM；
//   menuPadding = paddingXXS；选项圆角 borderRadiusSM、hover controlItemBgHover、
//   active controlItemBgActive + colorText + fontWeightStrong；禁用 colorTextDisabled；
//   展开图标 marginInlineStart paddingXXS / colorIcon / fontSizeIcon；
//   列分隔 lineWidth colorSplit；浮层底 colorBgElevated、圆角 borderRadiusLG。
//   TreeSelect：浮层 padding paddingXS × paddingXS/2、圆角 borderRadiusLG、底 colorBgElevated；
//   内部树沿用 tree/style（titleHeight/indent controlHeightSM、内容 paddingXS、圆角 borderRadius）。
//
//   node harness/antd_baseline/gen_select_ext_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function cascaderLine(token) {
  return [
    token.colorText,
    token.controlItemBgActive,
    token.controlItemBgHover,
    token.colorTextDisabled,
    token.fontWeightStrong,
    Math.round((token.controlHeight - token.fontSize * token.lineHeight) / 2),
    token.paddingSM,
    token.paddingXXS,
    token.borderRadiusSM,
    token.fontSize,
    token.fontSize * token.lineHeight,
    token.paddingXXS,
    token.fontSizeIcon,
    111,
    180,
    184,
    token.colorSplit,
    token.lineWidth,
    token.borderRadiusLG,
    token.colorIcon,
    token.colorBgElevated,
  ].join('|') + '\n';
}

function treeSelectLine(token) {
  return [
    token.paddingXS,
    token.paddingXS / 2,
    token.borderRadiusLG,
    token.colorBgElevated,
    token.controlHeightSM,
    token.controlHeightSM,
    token.paddingXS,
    token.borderRadius,
    token.controlItemBgActive,
    token.controlItemBgHover,
    token.colorText,
    token.colorTextQuaternary,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `cascader_matrix_${name}.txt`), cascaderLine(token));
  fs.writeFileSync(path.join(dir, `treeselect_matrix_${name}.txt`), treeSelectLine(token));
  console.log(`wrote cascader/treeselect_matrix_${name}.txt`);
}