// 生成 Switch 状态基准。
// 规则来源：antdv-next Switch 样式与 computed style（harness/_switch_probe.html 核对）：
//   - 轨道 44x22（small 28x16），滑块 18（small 12），内边距 2，圆角 100；
//   - 未选中 colorTextQuaternary，悬停 colorTextTertiary；选中主色，悬停 colorPrimaryHover；
//   - 滑块 colorBgContainer；禁用 / 加载 opacity 0.65；
//   - 内文字 12px，选中侧内边距 9（small 6）。
//
//   node harness/antd_baseline/gen_switch_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

const parse = (text) => {
  text = text.trim();
  if (text.startsWith('#')) {
    const hex = text.slice(1);
    const s = hex.length === 3 ? [...hex].map((c) => c + c).join('') : hex;
    return [...[0, 2, 4].map((i) => parseInt(s.slice(i, i + 2), 16)), 1];
  }
  const parts = text
    .slice(text.indexOf('(') + 1, -1)
    .split(',')
    .map((v) => parseFloat(v));
  return [parts[0], parts[1], parts[2], parts[3] ?? 1];
};

const fmt = (rgb) =>
  rgb.every((v) => Math.abs(v - Math.round(v)) < 1e-6)
    ? `#${rgb.map((v) => Math.round(v).toString(16).padStart(2, '0')).join('')}`
    : `rgb(${rgb.map((v) => Math.round(v)).join(',')})`;

// 与 Rust 侧一致：先把（可能半透明的）颜色合成到容器底色，再按 0.65 整体混淡
const fade = (color, backdrop) => {
  const f = parse(color);
  const b = parse(backdrop);
  const over = [0, 1, 2].map((i) => f[i] * f[3] + b[i] * (1 - f[3]));
  return fmt(over.map((v, i) => v * 0.65 + b[i] * 0.35));
};

function rows(token) {
  const out = [];
  for (const small of [false, true]) {
    for (const on of [false, true]) {
      for (const disabled of [false, true]) {
        for (const loading of [false, true]) {
          const inactive = disabled || loading;
          const raw = on ? token.colorPrimary : token.colorTextQuaternary;
          const rawHover = inactive
            ? raw
            : on
              ? token.colorPrimaryHover
              : token.colorTextTertiary;
          const cv = (color) => (inactive ? fade(color, token.colorBgContainer) : color);
          out.push(
            [
              small ? '1' : '0',
              on ? '1' : '0',
              disabled ? '1' : '0',
              loading ? '1' : '0',
              cv(raw),
              cv(rawHover),
              token.colorBgContainer,
              cv(token.colorTextTertiary),
              cv(token.colorTextLightSolid),
              cv(token.colorPrimary),
              inactive ? '0.65' : '1',
              small ? 28 : 44,
              small ? 16 : 22,
              small ? 12 : 18,
              2,
              100,
              small ? 6 : 9,
              token.fontSizeSM,
              small ? 16 : 22,
            ].join('|'),
          );
        }
      }
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
  fs.writeFileSync(path.join(dir, `switch_matrix_${name}.txt`), rows(token));
  console.log(`wrote switch_matrix_${name}.txt`);
}
