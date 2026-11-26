// 生成 Slider 状态基准。
// 规则来源：antdv-next `slider/style/index.js` prepareComponentToken：
//   - 容器高 controlHeightLG/4=10、轨道 4、滑块 10、悬停滑块 controlHeightSM/2=12、
//     刻度点 8、滑块环 lineWidth+1=2（悬停 +0.5）、悬停外圈 6；
//   - 轨道 colorFillTertiary → 悬停 colorFillSecondary；
//   - 已选轨道/滑块环 colorPrimaryBorder → 悬停 colorPrimaryBorderHover / colorPrimary；
//   - 禁用轨道 colorBgContainerDisabled，滑块环 colorTextDisabled 叠 colorBgContainer；
//   - 刻度点边框 colorBorderSecondary（悬停 colorFillContentHover），已过点 colorPrimaryBorder；
//   - 外边距块向 (controlHeight-controlSize)/2=11、行向 5，带刻度下边距 30，
//     刻度顶距 railSize*3=12、字号 fontSize。
//
//   node harness/antd_baseline/gen_slider_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;
const STATES = ['normal', 'hover', 'active', 'disabled'];

function parseColor(text) {
  const t = text.trim();
  if (t.startsWith('#')) {
    return {
      r: parseInt(t.slice(1, 3), 16),
      g: parseInt(t.slice(3, 5), 16),
      b: parseInt(t.slice(5, 7), 16),
      a: 1,
    };
  }
  const body = t.replace(/^rgba?\(|\)$/g, '');
  const [r, g, b, a = 1] = body.split(',').map((v) => parseFloat(v));
  return { r, g, b, a };
}

function fmt({ r, g, b, a }) {
  if (a >= 1) {
    const h = (v) => Math.round(v).toString(16).padStart(2, '0');
    return `#${h(r)}${h(g)}${h(b)}`;
  }
  const f = (v) => Number(v.toFixed(4));
  return `rgba(${f(r)},${f(g)},${f(b)},${f(a)})`;
}

// antd FastColor.onBackground：把半透明前景压在底色上
function onBackground(front, back) {
  const f = parseColor(front);
  const b = parseColor(back);
  return fmt({
    r: f.r * f.a + b.r * (1 - f.a),
    g: f.g * f.a + b.g * (1 - f.a),
    b: f.b * f.a + b.b * (1 - f.a),
    a: 1,
  });
}

function stateColors(token, state) {
  const disabledRing = onBackground(token.colorTextDisabled, token.colorBgContainer);
  switch (state) {
    case 'normal':
      return [
        token.colorFillTertiary,
        token.colorPrimaryBorder,
        token.colorBgElevated,
        token.colorPrimaryBorder,
        token.colorPrimaryBg,
        token.colorBorderSecondary,
        token.colorPrimaryBorder,
      ];
    case 'hover':
    case 'active':
      return [
        token.colorFillSecondary,
        token.colorPrimaryBorderHover,
        token.colorBgElevated,
        token.colorPrimary,
        token.colorPrimaryBg,
        token.colorFillContentHover,
        token.colorPrimaryBorder,
      ];
    case 'disabled':
      return [
        token.colorFillTertiary,
        token.colorBgContainerDisabled,
        token.colorBgElevated,
        disabledRing,
        token.colorPrimaryBg,
        token.colorBgContainerDisabled,
        token.colorBgContainerDisabled,
      ];
    default:
      throw new Error(`unknown state ${state}`);
  }
}

function rows(token) {
  const controlSize = token.controlHeightLG / 4;
  const railSize = 4;
  const dotSize = 8;
  const ringWidth = token.lineWidth + 1;
  const metrics = [
    controlSize,
    railSize,
    controlSize,
    token.controlHeightSM / 2,
    dotSize,
    ringWidth,
    token.lineWidth + 1.5,
    6,
    (railSize * 3 - controlSize) / 2,
    (railSize - dotSize) / 2,
    railSize * 3,
    token.fontSize,
    (token.controlHeight - controlSize) / 2,
    controlSize / 2,
    token.controlHeightLG - controlSize,
  ];
  return (
    STATES.map((state) => [state, ...stateColors(token, state), ...metrics].join('|')).join('\n') +
    '\n'
  );
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `slider_matrix_${name}.txt`), rows(token));
  console.log(`wrote slider_matrix_${name}.txt`);
}
