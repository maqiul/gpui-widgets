// 生成 Statistic 基准。
// 规则来源：antdv-next `statistic/style/index.js` prepareComponentToken：
//   标题 colorTextDescription / fontSize；内容 colorTextHeading / fontSizeHeading3；
//   标题下距 marginXXS；前缀/后缀间距 marginXXS。趋势是本项目扩展（功能色）。
//
//   node harness/antd_baseline/gen_statistic_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  return (
    [
      token.colorTextDescription,
      token.colorTextHeading,
      token.colorSuccess,
      token.colorError,
      token.fontSize,
      token.fontSizeHeading3,
      token.marginXXS,
      token.marginXXS,
      token.fontSizeSM,
      token.marginXXS,
    ].join('|') + '\n'
  );
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `statistic_matrix_${name}.txt`), rows(token));
  console.log(`wrote statistic_matrix_${name}.txt`);
}
