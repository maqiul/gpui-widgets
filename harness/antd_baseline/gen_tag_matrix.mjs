// 生成 Tag 状态矩阵基准：tokens-*.json（antd 实测导出）+ @ant-design/colors 预置色 + antd Tag 规则。
// 规则来源：antdv-next dist/tag/style/index.js / presetCmp.js / statusCmp.js。
//
//   node harness/antd_baseline/gen_tag_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';
import { presetPalettes } from '@ant-design/colors';

const dir = import.meta.dirname;

const parse = (css) => {
  const text = css.trim();
  if (text === 'transparent') return { r: 0, g: 0, b: 0, a: 0 };
  const rgb = /rgba?\(([^)]+)\)/.exec(text);
  if (rgb) {
    const [r, g, b, a = 1] = rgb[1].split(',').map((v) => parseFloat(v));
    return { r, g, b, a };
  }
  const h = /^#([0-9a-f]{3,8})$/i.exec(text);
  if (h) {
    let s = h[1];
    if (s.length === 3) s = s.split('').map((ch) => ch + ch).join('');
    return {
      r: parseInt(s.slice(0, 2), 16),
      g: parseInt(s.slice(2, 4), 16),
      b: parseInt(s.slice(4, 6), 16),
      a: s.length === 8 ? parseInt(s.slice(6, 8), 16) / 255 : 1,
    };
  }
  throw new Error(`bad color ${css}`);
};
const over = (front, back) => ({
  r: back.r * (1 - front.a) + front.r * front.a,
  g: back.g * (1 - front.a) + front.g * front.a,
  b: back.b * (1 - front.a) + front.b * front.a,
  a: 1,
});
const fmt = (c) => {
  if (c.a === 0) return 'transparent';
  const r = Math.round(c.r);
  const g = Math.round(c.g);
  const b = Math.round(c.b);
  const a = Math.round(c.a * 1000) / 1000;
  return a === 1 ? `rgb(${r},${g},${b})` : `rgba(${r},${g},${b},${a})`;
};

const colors = [
  ['default', null],
  ['blue', 'blue'],
  ['green', 'green'],
  ['orange', 'orange'],
  ['red', 'red'],
  ['purple', 'purple'],
  ['cyan', 'cyan'],
  ['magenta', 'magenta'],
  ['success', 'status'],
  ['processing', 'status'],
  ['error', 'status'],
  ['warning', 'status'],
];
const variants = ['outlined', 'filled', 'solid'];

function matrix(tokens) {
  const defaultBg = over(parse(tokens.colorFillTertiary), parse(tokens.colorBgContainer));
  const statusOf = {
    success: ['colorSuccess', 'colorSuccessBg', 'colorSuccessBorder'],
    processing: ['colorInfo', 'colorInfoBg', 'colorInfoBorder'],
    error: ['colorError', 'colorErrorBg', 'colorErrorBorder'],
    warning: ['colorWarning', 'colorWarningBg', 'colorWarningBorder'],
  };
  const lines = [];
  for (const [name, kind] of colors) {
    for (const variant of variants) {
      for (const disabled of [false, true]) {
        let bg, fg, border;
        if (kind === 'status') {
          const [solidKey, bgKey, borderKey] = statusOf[name];
          if (variant === 'solid') [bg, fg, border] = [tokens[solidKey], '#ffffff', tokens[solidKey]];
          else if (variant === 'filled') [bg, fg, border] = [tokens[bgKey], tokens[solidKey], 'transparent'];
          else [bg, fg, border] = [tokens[bgKey], tokens[solidKey], tokens[borderKey]];
        } else if (kind) {
          const p = presetPalettes[kind];
          if (variant === 'solid') [bg, fg, border] = [p[5], '#ffffff', p[5]];
          else if (variant === 'filled') [bg, fg, border] = [p[0], p[6], 'transparent'];
          else [bg, fg, border] = [p[0], p[6], p[2]];
        } else if (variant === 'solid') {
          const solid = parse(tokens.colorBgSolid);
          const bright =
            solid.a > 0.5 && 0.299 * solid.r + 0.587 * solid.g + 0.114 * solid.b > 192;
          bg = tokens.colorBgSolid;
          fg = bright ? '#000000' : '#ffffff';
          border = 'transparent';
        } else if (variant === 'filled') {
          [bg, fg, border] = [fmt(defaultBg), tokens.colorText, 'transparent'];
        } else {
          [bg, fg, border] = [fmt(defaultBg), tokens.colorText, tokens.colorBorder];
        }
        if (disabled) {
          bg = tokens.colorBgContainerDisabled;
          fg = tokens.colorTextDisabled;
          border = variant === 'outlined' ? tokens.colorBorder : 'transparent';
        }
        lines.push([name, variant, disabled ? '1' : '0', bg, fg, border].join('|'));
      }
    }
  }
  return lines.join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const tokens = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  const out = path.join(dir, `tag_matrix_${name}.txt`);
  fs.writeFileSync(out, matrix(tokens));
  console.log(`wrote ${out}`);
}
