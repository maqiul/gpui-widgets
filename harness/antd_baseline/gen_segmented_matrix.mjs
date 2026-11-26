// 生成 Segmented 状态基准。
// 规则来源：antdv-next `segmented/style/index.js` prepareComponentToken：
//   - 轨道：padding lineWidthBold 2、底 colorBgLayout、圆角 borderRadius（6）；
//     sm 4 / lg 8；
//   - 选项：高 controlHeight-2*trackPadding（28/20/36）、圆角 4/2/6、
//     水平 padding controlPaddingHorizontal-lineWidth（11/7/11）、字号 14/14/16；
//   - 常态 colorTextLabel；悬停 colorText + colorFillSecondary；按下 colorText + colorFill；
//     选中 colorText + colorBgElevated + boxShadowTertiary；禁用 colorTextDisabled。
//
//   node harness/antd_baseline/gen_segmented_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;
const STATES = ['normal', 'hover', 'active', 'selected', 'disabled', 'selected-disabled'];

function stateColors(token, state) {
  switch (state) {
    case 'normal':
      return [token.colorTextLabel, 'rgba(0,0,0,0)', 0];
    case 'hover':
      return [token.colorText, token.colorFillSecondary, 0];
    case 'active':
      return [token.colorText, token.colorFill, 0];
    case 'selected':
      return [token.colorText, token.colorBgElevated, 1];
    case 'disabled':
      return [token.colorTextDisabled, 'rgba(0,0,0,0)', 0];
    case 'selected-disabled':
      return [token.colorTextDisabled, token.colorBgElevated, 1];
    default:
      throw new Error(`unknown state ${state}`);
  }
}

function rows(token) {
  const trackPadding = token.lineWidthBold;
  const metrics = [
    trackPadding,
    token.borderRadiusSM,
    token.borderRadius,
    token.borderRadiusLG,
    token.borderRadiusXS,
    token.borderRadiusSM,
    token.borderRadius,
    token.controlHeightSM - trackPadding * 2,
    token.controlHeight - trackPadding * 2,
    token.controlHeightLG - trackPadding * 2,
    token.controlPaddingHorizontalSM - token.lineWidth,
    token.controlPaddingHorizontal - token.lineWidth,
    token.controlPaddingHorizontal - token.lineWidth,
    token.fontSize,
    token.fontSize,
    token.fontSizeLG,
  ];
  return (
    STATES.map((state) => {
      const [text, bg, shadow] = stateColors(token, state);
      return [state, text, bg, shadow, token.colorBgLayout, ...metrics].join('|');
    }).join('\n') + '\n'
  );
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `segmented_matrix_${name}.txt`), rows(token));
  console.log(`wrote segmented_matrix_${name}.txt`);
}
