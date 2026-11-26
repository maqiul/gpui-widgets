// 生成 DatePicker 面板基准。
// 规则来源：antdv-next `date-picker/style/token.js` + `panel.js` + `index.js`：
//   cellWidth = controlHeightSM*1.5 / cellHeight = controlHeightSM /
//   pickerCellPaddingVertical = paddingXXS*1.5 / pickerDatePanelPaddingHorizontal = padding+paddingXXS/2 /
//   panelWidth = cellWidth*7 + padH*2 / textHeight = controlHeightLG /
//   th height = cellHeight+2*cellPadV / date-panel body padding paddingXS padH /
//   header borderBottom colorSplit、button colorIcon hover colorIconHover、view fontWeightStrong colorTextHeading /
//   cell inner radius borderRadiusSM、selected colorPrimary+colorTextLightSolid、today 描边 colorPrimary、
//   hover controlItemBgHover、补位 colorTextDisabled /
//   panel-container bg colorBgElevated radius borderRadiusLG shadow boxShadowSecondary /
//   footer borderTop colorSplit、ranges li lineHeight textHeight-2*lineWidth、paddingInline paddingSM /
//   suffix color colorTextQuaternary、marginInlineStart paddingXS/2。
//
//   node harness/antd_baseline/gen_date_picker_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  const cellWidth = token.controlHeightSM * 1.5;
  const cellHeight = token.controlHeightSM;
  const cellPadV = token.paddingXXS * 1.5;
  const padH = token.padding + token.paddingXXS / 2;
  return [
    token.colorBgElevated,
    token.colorSplit,
    token.colorTextHeading,
    token.colorIcon,
    token.colorIconHover,
    token.colorPrimary,
    token.colorText,
    token.colorTextDisabled,
    token.controlItemBgHover,
    token.colorTextLightSolid,
    token.colorTextQuaternary,
    cellWidth,
    cellHeight,
    cellPadV,
    padH,
    cellWidth * 7 + padH * 2,
    token.controlHeightLG,
    cellHeight + cellPadV * 2,
    token.borderRadiusLG,
    token.borderRadiusSM,
    token.paddingXS,
    token.paddingSM,
    token.controlHeightLG - token.lineWidth * 2,
    token.paddingXS / 2,
    token.fontWeightStrong,
    token.fontSize,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `date_picker_matrix_${name}.txt`), line(token));
  console.log(`wrote date_picker_matrix_${name}.txt`);
}