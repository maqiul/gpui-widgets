// 生成 Breadcrumb 状态基准。
// 规则来源：antdv-next Breadcrumb 样式与 computed style（harness/_breadcrumb_probe.html 核对）：
//   - 非末项与分隔符 colorTextDescription，末项 colorText；
//   - 字号 fontSize 14、行高 fontHeight 22、字重 400；
//   - 默认分隔符 "/"，两侧各 8px。
//
//   node harness/antd_baseline/gen_breadcrumb_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  return [
    [
      token.colorTextDescription,
      token.colorText,
      token.colorTextDescription,
      token.fontSize,
      token.fontHeight,
      8,
    ].join('|'),
  ].join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `breadcrumb_matrix_${name}.txt`), rows(token));
  console.log(`wrote breadcrumb_matrix_${name}.txt`);
}
