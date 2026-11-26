// 生成 Avatar 状态基准。
// 规则来源：antdv-next Avatar 样式与 computed style（harness/_avatar_probe.html 核对）：
//   - 尺寸 24 / 32 / 40，圆形半径 = 边长一半；
//   - 方形圆角 borderRadiusSM / borderRadius / borderRadiusLG（4 / 6 / 8）；
//   - 底色 colorTextPlaceholder、文字 colorTextLightSolid，字号始终 fontSize、行高 fontHeight。
//
//   node harness/antd_baseline/gen_avatar_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  const out = [];
  for (const [ix, size] of [24, 32, 40].entries()) {
    for (const square of [false, true]) {
      const radius = square
        ? ix === 0
          ? token.borderRadiusSM
          : ix === 1
            ? token.borderRadius
            : token.borderRadiusLG
        : size / 2;
      out.push(
        [
          String(ix),
          square ? '1' : '0',
          token.colorTextPlaceholder,
          token.colorTextLightSolid,
          size,
          radius,
          token.fontSize,
          token.fontHeight,
        ].join('|'),
      );
    }
  }
  return out.join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `avatar_matrix_${name}.txt`), rows(token));
  console.log(`wrote avatar_matrix_${name}.txt`);
}
