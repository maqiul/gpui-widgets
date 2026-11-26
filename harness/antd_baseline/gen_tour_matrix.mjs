// 生成 Tour 基准。
// 规则来源：antdv-next `tour/style/index.js` + `panelRender.js` + `@v-c/tour/dist/Mask.js`：
//   section width 520 / radius borderRadiusLG / shadow boxShadowTertiary / bg colorBgElevated；
//   close top/end padding、尺寸 fontSize*lineHeight、radius borderRadiusSM、
//     colorIcon / hover colorIconHover+colorBgTextHover / active colorBgTextActive；
//   header padding padding padding paddingXS（右再留 closeBtnSize）；title fontWeightStrong；
//   description padding 0 padding；footer paddingXS padding padding；
//   indicator 6×6 colorFill / active colorPrimary / 间距 indicatorHeight；
//   actions 按钮间距 marginXS；mask 为 rc-tour 默认 fill rgba(0,0,0,0.5)。
//
//   node harness/antd_baseline/gen_tour_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  return [
    token.colorBgElevated,
    'rgba(0,0,0,0.5)',
    token.colorIcon,
    token.colorIconHover,
    token.colorBgTextHover,
    token.colorBgTextActive,
    token.colorText,
    token.colorFill,
    token.colorPrimary,
    520,
    token.borderRadiusLG,
    token.padding,
    token.paddingXS,
    token.fontSize * token.lineHeight,
    token.borderRadiusSM,
    token.fontSize,
    6,
    6,
    token.marginXS,
    token.fontWeightStrong,
    token.fontSize,
    token.fontSize * token.lineHeight,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `tour_matrix_${name}.txt`), line(token));
  console.log(`wrote tour_matrix_${name}.txt`);
}