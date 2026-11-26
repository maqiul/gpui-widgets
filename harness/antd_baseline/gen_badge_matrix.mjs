// 生成 Badge 状态基准。
// 规则来源：antdv-next Badge 样式与 computed style（harness/_badge_probe.html 核对）：
//   - 计数 20 高、最小宽 20、圆角 10、字号 fontSizeSM(12)、行高 20；
//   - 计数宽度 = max(20, 16 + 字符数 × 7)（antd padding 0 8 + 12px 数字近似字宽）；
//   - 圆点 6×6；计数与圆点都压在右上角，外圈 1px colorBgContainer；
//   - 默认底 colorError、文字 colorTextLightSolid；自定义 color 直接作为底色。
//
//   node harness/antd_baseline/gen_badge_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  const out = [];
  const push = (mode, bg, chars) => {
    const dot = mode === 1;
    const width = dot ? 6 : chars <= 1 ? 20 : Math.max(20, 16 + chars * 7);
    out.push(
      [
        String(mode),
        bg,
        token.colorTextLightSolid,
        20,
        10,
        token.fontSizeSM,
        20,
        6,
        width,
        token.colorBgContainer,
        1,
      ].join('|'),
    );
  };

  push(0, token.colorError, 1);
  push(0, token.colorError, 3);
  push(1, token.colorError, 0);
  push(0, token.colorError, 1);
  push(0, '#1677ff', 1);
  return out.join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `badge_matrix_${name}.txt`), rows(token));
  console.log(`wrote badge_matrix_${name}.txt`);
}
