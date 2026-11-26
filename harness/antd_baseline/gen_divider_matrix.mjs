// 生成 Divider 基准。
// 规则来源：antdv-next `divider/style/index.js` + prepareComponentToken：
//   线条 colorSplit / lineWidth；带文字标题色 colorTextHeading、fontSizeLG、字重 500，
//   plain 时 colorText / fontSize、字重常规；文字两侧 1em；
//   无文字上下 marginLG、带文字 margin；朝向留白 orientationMargin=0.05；
//   竖线左右 marginXS、高 0.9em。
//
//   node harness/antd_baseline/gen_divider_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  const fields = [
    token.colorSplit,
    token.colorTextHeading,
    token.colorText,
    token.lineWidth,
    token.marginLG,
    token.margin,
    token.fontSizeLG,
    token.fontSize,
    token.fontSizeLG,
    token.fontSize,
    0.05,
    token.marginXS,
    0.9,
    500,
  ];
  return fields.join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `divider_matrix_${name}.txt`), rows(token));
  console.log(`wrote divider_matrix_${name}.txt`);
}
