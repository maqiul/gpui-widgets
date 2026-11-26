// 生成 Tree 基准。
// 规则来源：antdv-next `tree/style/index.js` prepareComponentToken：
//   titleHeight / switcherSize / indentSize = controlHeightSM；
//   nodeHoverBg = controlItemBgHover、nodeSelectedBg = controlItemBgActive；
//   内容横向内边距 paddingXS、圆角 borderRadius；勾选框 controlInteractiveSize。
//
//   node harness/antd_baseline/gen_tree_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  return [
    token.colorText,
    token.colorTextQuaternary,
    token.colorPrimary,
    token.colorTextLightSolid,
    token.controlItemBgActive,
    token.controlItemBgHover,
    token.controlHeightSM,
    token.fontSizeSM,
    token.paddingXS,
    token.borderRadius,
    token.controlInteractiveSize,
    token.borderRadiusSM,
    token.marginXS,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `tree_matrix_${name}.txt`), line(token));
  console.log(`wrote tree_matrix_${name}.txt`);
}
