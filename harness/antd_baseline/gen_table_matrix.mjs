// 生成 Table 基准。
// 规则来源：antdv-next `table/style/index.js` prepareComponentToken：
//   headerBg/rowHoverBg/footerBg = colorFillAlter.onBackground(colorBgContainer)；
//   borderColor = colorBorderSecondary；rowSelectedBg = controlItemBgActive；
//   cellPaddingBlock/Inline = padding；selectionColumnWidth = controlHeight；
//   tableExpandColumnWidth = controlInteractiveSize + padding*2。
//
//   node harness/antd_baseline/gen_table_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function parseColor(text) {
  const t = String(text).trim();
  if (t.startsWith('#')) {
    const h = t.slice(1);
    const s = h.length === 3 ? h.split('').map((c) => c + c).join('') : h;
    return [parseInt(s.slice(0, 2), 16), parseInt(s.slice(2, 4), 16), parseInt(s.slice(4, 6), 16), 1];
  }
  const body = t.replace(/^rgba?\(/, '').replace(/\)$/, '');
  const p = body.split(',').map((v) => parseFloat(v.trim()));
  return [p[0], p[1], p[2], p.length > 3 ? p[3] : 1];
}

function onBackground(front, back) {
  const f = parseColor(front);
  const b = parseColor(back);
  const a = f[3];
  const mix = (i) => Math.round(f[i] * a + b[i] * (1 - a));
  return `rgb(${mix(0)},${mix(1)},${mix(2)})`;
}

function line(token) {
  const alterSolid = onBackground(token.colorFillAlter, token.colorBgContainer);
  return [
    alterSolid,
    token.colorTextHeading,
    token.colorBorderSecondary,
    alterSolid,
    token.controlItemBgActive,
    token.controlItemBgActiveHover,
    alterSolid,
    token.colorTextHeading,
    token.padding,
    token.padding,
    token.fontSize,
    Math.round(token.fontSize * token.lineHeight),
    token.controlHeight,
    token.controlInteractiveSize + token.padding * 2,
    token.borderRadiusLG,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `table_matrix_${name}.txt`), line(token));
  console.log(`wrote table_matrix_${name}.txt`);
}
