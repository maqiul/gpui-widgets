// 生成 Result 基准。
// 规则来源：antdv-next `result/style/index.js` prepareComponentToken：
//   titleFontSize=fontSizeHeading3、subtitleFontSize=fontSize、iconFontSize=fontSizeHeading3*3、
//   extraMargin=paddingLG 0 0 0；根容器 padding=`paddingLG*2 paddingXL`；
//   正文块 marginTop=paddingLG、padding=`paddingLG (padding*2.5)`。
//
//   node harness/antd_baseline/gen_result_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  return [
    token.colorSuccess,
    token.colorError,
    token.colorWarning,
    token.colorInfo,
    token.colorTextHeading,
    token.colorTextDescription,
    token.colorFillAlter,
    token.fontSizeHeading3 * 3,
    token.fontSizeHeading3,
    Math.round(token.fontSizeHeading3 * 1.3333334),
    token.marginXS,
    token.fontSize,
    Math.round(token.fontSize * token.lineHeight),
    token.paddingLG,
    token.padding * 2.5,
    token.paddingLG,
    token.paddingXL,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `result_matrix_${name}.txt`), line(token));
  console.log(`wrote result_matrix_${name}.txt`);
}
