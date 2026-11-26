// 生成 InputNumber / Password / Search 面板基准。
// 规则来源：antdv-next `input-number/style/token.js` prepareComponentToken +
// `input-number/style/index.js` genInputNumberStyles +
// `input/style/index.js` genAffixStyle / genAllowClearStyle：
//   InputNumber：controlWidth 90；handleWidth = controlHeightSM − lineWidth×2；
//   handleFontSize = fontSize/2；handleHoverColor = colorPrimary；handleActiveBg = colorFillAlter；
//   handleBg = colorBgContainer；handleBorderColor = colorBorder；
//   filledHandleBg = colorFillSecondary.onBackground(colorBgContainer)。
//   Password：眼睛图标 1em = fontSize，colorIcon / hover colorIconHover；间距 inputAffixPadding = paddingXXS。
//   Search：前缀 1em = fontSize 继承正文色；清空 fontSizeIcon = fontSizeSM，
//   colorTextQuaternary / hover colorIcon / active colorText；间距 paddingXXS。
//
//   node harness/antd_baseline/gen_input_compound_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';
import { FastColor } from '@ant-design/fast-color';

const dir = import.meta.dirname;

function numberLine(token) {
  const filledHandleBg = new FastColor(token.colorFillSecondary)
    .onBackground(token.colorBgContainer)
    .toHexString();
  return [
    token.colorIcon,
    token.colorPrimary,
    token.colorTextDisabled,
    token.colorFillAlter,
    token.colorBgContainer,
    token.colorBorder,
    filledHandleBg,
    token.controlHeightSM - token.lineWidth * 2,
    token.fontSize / 2,
    90,
  ].join('|') + '\n';
}

function passwordLine(token) {
  return [
    token.colorIcon,
    token.colorIconHover,
    token.fontSize,
    token.paddingXXS,
  ].join('|') + '\n';
}

function searchLine(token) {
  return [
    token.colorTextQuaternary,
    token.colorIcon,
    token.colorText,
    token.fontSizeSM,
    token.fontSize,
    token.paddingXXS,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `number_input_matrix_${name}.txt`), numberLine(token));
  fs.writeFileSync(path.join(dir, `password_input_matrix_${name}.txt`), passwordLine(token));
  fs.writeFileSync(path.join(dir, `search_input_matrix_${name}.txt`), searchLine(token));
  console.log(`wrote number_input/password_input/search_input_matrix_${name}.txt`);
}