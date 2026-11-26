// 生成 Empty 基准。
// 规则来源：antdv-next `empty/style/index.js` + `empty/empty.js` / `simple.js` / `utils.js`：
//   插画色 = getAsSolidColor(fill, colorBgContainer)：
//     default panelBg colorFillTertiary / border colorTextQuaternary / detail colorFill /
//     shadow colorFillSecondary（ellipse 再乘 fill-opacity .8）/ icon colorBgContainer；
//     simple border colorFill / shadow colorFillTertiary / content colorFillQuaternary；
//   imgHeight controlHeightLG*2.5 / MD controlHeightLG / SM controlHeightLG*.875；
//   image marginBottom marginXS / marginInline marginXS；
//   -normal（simple 插画）marginBlock marginXL / -small marginBlock marginXS；
//   description fontSize fontHeight colorTextDescription / footer marginTop margin。
//
//   node harness/antd_baseline/gen_empty_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function parse(text) {
  text = text.trim();
  if (text.startsWith('#')) {
    const s = text.slice(1);
    const v = (i) => parseInt(s.slice(i, i + 2), 16);
    return [v(0), v(2), v(4), 1];
  }
  const body = text.replace(/^rgba?\(/, '').replace(/\)$/, '');
  const p = body.split(',').map((x) => parseFloat(x));
  return [p[0], p[1], p[2], p.length > 3 ? p[3] : 1];
}

function solid(fg, bg) {
  const f = parse(fg);
  const b = parse(bg);
  const mix = (i) => Math.round(f[i] * f[3] + b[i] * (1 - f[3]));
  return `#${[mix(0), mix(1), mix(2)].map((v) => v.toString(16).padStart(2, '0')).join('')}`;
}

function line(token) {
  const bg = token.colorBgContainer;
  return [
    solid(token.colorFillTertiary, bg),
    solid(token.colorTextQuaternary, bg),
    solid(token.colorFill, bg),
    solid(token.colorFillSecondary, bg),
    bg,
    solid(token.colorFill, bg),
    solid(token.colorFillTertiary, bg),
    solid(token.colorFillQuaternary, bg),
    token.colorTextDescription,
    token.controlHeightLG * 2.5,
    token.controlHeightLG,
    token.controlHeightLG * 0.875,
    token.marginXS,
    token.marginXS,
    token.marginXL,
    token.marginXS,
    0.8,
    token.fontSize,
    token.fontHeight,
    token.margin,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `empty_matrix_${name}.txt`), line(token));
  console.log(`wrote empty_matrix_${name}.txt`);
}