// 生成 Card 状态基准。
// 规则来源：antdv-next `card/style/index.js` prepareComponentToken：
//   - 圆角 borderRadiusLG；
//   - 头高 fontHeightLG + padding×2 = 56、左右 24、字号 fontSizeLG(16)、strong；
//   - body 内边距 paddingLG(24)；
//   - actions 项上下 12、字号 fontSize(14)、文字 colorTextDescription；
//   - 有边框 colorBorderSecondary，无边框改用 boxShadowTertiary。
//
//   node harness/antd_baseline/gen_card_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  const base = [
    token.colorBgContainer,
    token.colorBorderSecondary,
    token.colorTextHeading,
    token.colorText,
    token.colorText,
    token.colorTextDescription,
    token.colorPrimary,
    token.borderRadiusLG,
    token.fontHeightLG + token.padding * 2,
    token.paddingLG,
    token.fontSizeLG,
    token.paddingLG,
    token.paddingSM,
    token.fontSize,
    token.fontHeight,
  ];
  return [0, 1]
    .map((bordered) =>
      [bordered ? 1 : 0, ...base, bordered ? token.lineWidth : 0, token.lineWidth].join('|'),
    )
    .join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `card_matrix_${name}.txt`), rows(token));
  console.log(`wrote card_matrix_${name}.txt`);
}
