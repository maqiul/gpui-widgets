// 从 antdv-next 实测导出的 tokens-*.json 生成 widgets/tests/data/*.rs 基准表。
// 生成的名字列表取自 widgets/tests/antd_token_parity.rs 里的 derived()，
// 因此“加了令牌却忘了重新生成”会在测试里显式失败。
import fs from 'node:fs';
import path from 'node:path';

const refDir = import.meta.dirname;
const widgets = path.resolve(refDir, '../..');
const parity = fs.readFileSync(path.join(widgets, 'tests/antd_token_parity.rs'), 'utf8');

const names = [...parity.matchAll(/"((?:color|control|font|line|size|padding|margin|border|motion|opacity|zIndex)[A-Za-z0-9]*)"/g)].map(
  (m) => m[1],
);
const unique = [...new Set(names)];
console.log(`derived() 中声明了 ${unique.length} 个令牌名`);

const scenarios = [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['accent_violet', 'tokens-accent-violet.json'],
  ['accent_violet_dark', 'tokens-accent-violet-dark.json'],
  ['accent_green', 'tokens-accent-green.json'],
];

const outDir = path.join(widgets, 'tests/data');
fs.mkdirSync(outDir, { recursive: true });

// Rust 的 f64/f32 字面量必须带小数点，JSON 里的 3、1000 之类整数要补上
const fnum = (v) => (Number.isInteger(v) ? `${v}.0` : String(v));

function toValue(name, raw) {
  const wrap = (expr) => `    ("${name}", ${expr}),`;
  if (typeof raw === 'number') return wrap(`crate::Value::N(${fnum(raw)})`);
  if (typeof raw !== 'string') return null;
  const hex = /^#([0-9a-f]{3,6})$/i.exec(raw.trim());
  if (hex) {
    let s = hex[1];
    if (s.length === 3) s = s.split('').map((ch) => ch + ch).join('');
    const v = parseInt(s, 16);
    return wrap(`crate::Value::C(${(v >> 16) & 255},${(v >> 8) & 255},${v & 255},1.0)`);
  }
  const secs = /^([0-9.]+)s$/.exec(raw.trim());
  if (secs) return wrap(`crate::Value::N(${fnum(parseFloat(secs[1]))})`);
  const rgba = /^rgba?\(([^)]+)\)$/.exec(raw.trim());
  if (rgba) {
    const parts = rgba[1].split(',').map((s) => parseFloat(s));
    const a = parts.length > 3 ? parts[3] : 1;
    return wrap(`crate::Value::C(${Math.round(parts[0])},${Math.round(parts[1])},${Math.round(parts[2])},${fnum(a)})`);
  }
  return null;
}

for (const [slug, file] of scenarios) {
  const tokens = JSON.parse(fs.readFileSync(path.join(refDir, file), 'utf8'));
  const lines = [];
  const missing = [];
  for (const name of unique) {
    if (!(name in tokens)) {
      missing.push(name);
      continue;
    }
    const value = toValue(name, tokens[name]);
    if (value) lines.push(value);
    else missing.push(`${name}=${JSON.stringify(tokens[name])}`);
  }
  const body = `// 由 .ref/gen_baseline_rs.mjs 从 ${file} 生成，请勿手改。
#![allow(clippy::all)]

pub const EXPECTED: &[(&str, crate::Value)] = &[
${lines.join('\n')}
];
`;
  fs.writeFileSync(path.join(outDir, `${slug}.rs`), body);
  console.log(`${slug}: ${lines.length} 项${missing.length ? `，跳过 ${missing.length}：${missing.slice(0, 8).join(', ')}` : ''}`);
}
