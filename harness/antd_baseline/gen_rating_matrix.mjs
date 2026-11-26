// 生成 Rate 基准。
// 规则来源：antdv-next `rate/style/index.js` prepareComponentToken：
//   starColor yellow6 / starBg colorFillContent；
//   starSize controlHeight*.625 / SM controlHeightSM*.625 / LG controlHeightLG*.625；
//   星间距 marginXS（marginInlineEnd，末星除外）。
//
//   node harness/antd_baseline/gen_rating_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  return [
    token.yellow6,
    token.colorFillContent,
    token.controlHeight * 0.625,
    token.controlHeightSM * 0.625,
    token.controlHeightLG * 0.625,
    token.marginXS,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `rating_matrix_${name}.txt`), line(token));
  console.log(`wrote rating_matrix_${name}.txt`);
}