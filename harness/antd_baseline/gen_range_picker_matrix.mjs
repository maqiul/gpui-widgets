// 生成 RangePicker 面板基准。
// 规则来源：antdv-next `date-picker/style/token.js` + `panel.js` + `index.js`（与 DatePicker 同源）：
//   cellWidth = controlHeightSM*1.5 / cellHeight = controlHeightSM /
//   pickerCellPaddingVertical = paddingXXS*1.5 / pickerDatePanelPaddingHorizontal = padding+paddingXXS/2 /
//   panelWidth = cellWidth*7 + padH*2（双面板并排，panel_gap=0）/ textHeight = controlHeightLG /
//   th height = cellHeight+2*cellPadV /
//   端点 inner colorPrimary+colorTextLightSolid、外侧半侧连续底 controlItemBgActive（cellActiveWithRangeBg）/
//   区间内整格连续底 controlItemBgActive / today 描边 colorPrimary /
//   hover controlItemBgHover、补位 colorTextDisabled /
//   panel-container bg colorBgElevated radius borderRadiusLG shadow boxShadowSecondary /
//   separator width 1em fontSize fontSizeLG color colorTextQuaternary（SwapRightOutlined）。
//
//   node harness/antd_baseline/gen_range_picker_matrix.mjs
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
    token.controlItemBgActive,
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
    token.fontSize,
    token.fontWeightStrong,
    token.fontSizeLG,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `range_picker_matrix_${name}.txt`), line(token));
  console.log(`wrote range_picker_matrix_${name}.txt`);
}