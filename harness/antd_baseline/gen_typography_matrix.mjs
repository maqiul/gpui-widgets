// 生成 Typography 基准。
// 规则来源：antdv-next `typography/style/index.js` + `mixins.js`：
//   标题 fontSizeHeading1-5 / lineHeightHeading1-5 / fontWeightStrong 600，
//   标题下距 fontSizeSM*lineHeightSM；正文 fontSize/lineHeight；
//   语义色 colorText / colorTextDescription / colorSuccessText / colorWarningText /
//   colorErrorText / colorTextDisabled；行内代码 85%、圆角 3、
//   底 rgba(150,150,150,.1)、边 rgba(100,100,100,.2)；mark 底 gold2（#ffe58f）。
//
//   node harness/antd_baseline/gen_typography_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  const size = [1, 2, 3, 4, 5].map((n) => token[`fontSizeHeading${n}`]);
  const line = [1, 2, 3, 4, 5].map((n) =>
    Math.round(token[`fontSizeHeading${n}`] * token[`lineHeightHeading${n}`]),
  );
  return (
    [
      ...size,
      ...line,
      Math.round(token.fontSizeSM * token.lineHeightSM),
      token.fontSize,
      token.lineHeight,
      token.colorText,
      token.colorTextDescription,
      token.colorSuccessText,
      token.colorWarningText,
      token.colorErrorText,
      token.colorTextDisabled,
      token.fontWeightStrong,
      token.fontWeightStrong,
      0.85,
      3,
      'rgba(150,150,150,0.10196)',
      'rgba(100,100,100,0.2)',
      '#ffe58f',
    ].join('|') + '\n'
  );
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `typography_matrix_${name}.txt`), rows(token));
  console.log(`wrote typography_matrix_${name}.txt`);
}
