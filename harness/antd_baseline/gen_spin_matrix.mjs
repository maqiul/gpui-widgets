// 生成 Spin 基准。
// 规则来源：antdv-next `spin/style/index.js` prepareComponentToken/genIndicatorStyle：
//   section color colorPrimary / gap paddingSM / description fontSize lineHeight 1；
//   dotSize controlHeightLG/2 / SM controlHeightLG*.35 / LG controlHeight；
//   dot-item (dotSize - marginXXS/2)/2，scale(0.75)，基 opacity 0.3。
//
//   node harness/antd_baseline/gen_spin_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  const dot = token.controlHeightLG / 2;
  const dotSM = token.controlHeightLG * 0.35;
  const dotLG = token.controlHeight;
  const item = (d) => (d - token.marginXXS / 2) / 2;
  return [
    token.colorPrimary,
    dot,
    dotSM,
    dotLG,
    item(dot),
    item(dotSM),
    item(dotLG),
    0.75,
    0.3,
    token.paddingSM,
    token.fontSize,
    token.fontSize,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `spin_matrix_${name}.txt`), line(token));
  console.log(`wrote spin_matrix_${name}.txt`);
}