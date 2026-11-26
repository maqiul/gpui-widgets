// 生成 TimePicker 面板基准。
// 规则来源：antdv-next `date-picker/style/token.js` + `panel.js` time-panel 段：
//   timeColumnWidth = controlHeightLG*1.4 / timeColumnHeight 224 / timeCellHeight 28 /
//   column margin paddingXXS 0 / li marginInline marginXXS /
//   inner width = colW-2*marginXXS、paddingInlineStart = (colW-28)/2、radius borderRadiusSM /
//   selected controlItemBgActive / hover cellHoverBg(controlItemBgHover) / disabled colorTextDisabled /
//   column 分隔 borderInlineStart colorSplit / panel-container colorBgElevated+borderRadiusLG+boxShadowSecondary /
//   footer ranges paddingInline paddingSM、li lineHeight controlHeightLG-2*lineWidth、ok paddingBlock lineWidth*2 /
//   suffix colorTextQuaternary、marginInlineStart paddingXS/2。
//
//   node harness/antd_baseline/gen_time_picker_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  const colW = token.controlHeightLG * 1.4;
  return [
    token.colorBgElevated,
    token.colorSplit,
    token.colorText,
    token.colorTextDisabled,
    token.controlItemBgActive,
    token.controlItemBgHover,
    token.colorPrimary,
    token.colorTextQuaternary,
    colW,
    224,
    28,
    token.paddingXXS,
    token.marginXXS,
    colW - token.marginXXS * 2,
    (colW - 28) / 2,
    token.borderRadiusSM,
    token.borderRadiusLG,
    token.controlHeightLG - token.lineWidth * 2,
    token.paddingSM,
    token.lineWidth * 2,
    token.fontSize,
    token.paddingXS / 2,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `time_picker_matrix_${name}.txt`), line(token));
  console.log(`wrote time_picker_matrix_${name}.txt`);
}