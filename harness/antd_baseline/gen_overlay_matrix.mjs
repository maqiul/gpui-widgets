// 生成 Tooltip / Popover 浮层基准。
// 规则来源：antdv-next Tooltip / Popover 的 CSS 与 computed style（harness/_tooltip_probe.html 核对）：
//   - tooltip 容器：padding calc(paddingSM / 2) paddingXS、min-height controlHeight、
//     圆角 borderRadius、最大宽 250、底色 colorBgSpotlight、文字 colorTextLightSolid；
//   - popover 容器：paddingSM、圆角 borderRadiusLG、底色 colorBgElevated、文字 colorText、无描边。
//
//   node harness/antd_baseline/gen_overlay_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

const tooltip = (token) =>
  [
    token.colorBgSpotlight,
    token.colorTextLightSolid,
    token.paddingSM / 2,
    token.paddingXS,
    token.borderRadius,
    token.controlHeight,
    250,
    token.fontSize,
    token.fontHeight,
  ].join('|') + '\n';

const popover = (token) =>
  [
    token.colorBgElevated,
    token.colorText,
    token.paddingSM,
    token.borderRadiusLG,
    token.fontSize,
    token.fontHeight,
  ].join('|') + '\n';

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `tooltip_matrix_${name}.txt`), tooltip(token));
  fs.writeFileSync(path.join(dir, `popover_matrix_${name}.txt`), popover(token));
  console.log(`wrote tooltip/popover_matrix_${name}.txt`);
}
