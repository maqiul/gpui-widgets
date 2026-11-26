// 生成 Carousel 基准。
// 规则来源：antdv-next `carousel/style/index.js`：
//   arrowSize 16 / arrowOffset marginXS / 箭头 #fff opacity .4（硬编码，用 colorTextLightSolid 等价）；
//   dotWidth 16 / dotHeight 3 / dotGap marginXXS / dotOffset 12 / dotActiveWidth 24 /
//   dot 底 colorBgContainer opacity .2、hover .75（硬编码）。
//
//   node harness/antd_baseline/gen_carousel_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  return [
    token.colorTextLightSolid,
    token.colorBgContainer,
    16,
    token.marginXS,
    0.4,
    16,
    3,
    token.marginXXS,
    12,
    24,
    0.2,
    0.75,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `carousel_matrix_${name}.txt`), line(token));
  console.log(`wrote carousel_matrix_${name}.txt`);
}