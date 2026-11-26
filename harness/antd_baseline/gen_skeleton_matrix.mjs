// 生成 Skeleton 状态基准。
// 规则来源：antdv-next `skeleton/style/index.js` prepareComponentToken：
//   - 标题 / 段落条高 controlHeight / 2；
//   - 圆角 borderRadiusSM；
//   - 行间距 controlHeightXS；
//   - 底色 colorFillContent（gradientFromColor）；
//   - 三行及以上末行宽 61%。
//
//   node harness/antd_baseline/gen_skeleton_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  return [
    [
      token.colorFillContent,
      token.borderRadiusSM,
      token.controlHeight / 2,
      token.controlHeightXS,
      0.61,
    ].join('|'),
  ].join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `skeleton_matrix_${name}.txt`), rows(token));
  console.log(`wrote skeleton_matrix_${name}.txt`);
}
