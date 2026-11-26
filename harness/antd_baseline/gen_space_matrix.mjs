// 生成 Space 基准。
// 规则来源：antdv-next `space/style/index.js` mergeToken：
//   spaceGapSmallSize=paddingXS、spaceGapMiddleSize=padding、spaceGapLargeSize=paddingLG。
//
//   node harness/antd_baseline/gen_space_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  return (
    [token.paddingXS, token.padding, token.paddingLG].join('|') + '\n'
  );
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `space_matrix_${name}.txt`), rows(token));
  console.log(`wrote space_matrix_${name}.txt`);
}
