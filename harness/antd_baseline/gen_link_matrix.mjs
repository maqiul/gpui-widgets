// 生成 Link 状态基准。
// 规则来源：antdv-next `typography/style/mixins.js` operationUnit + `getLinkStyles`：
//   常态 colorLink；悬停 colorLinkHover + linkHoverDecoration（下划线）；
//   按下 colorLinkActive + 下划线（focus 同 hover）；禁用 colorTextDisabled 且悬停不变。
//
//   node harness/antd_baseline/gen_link_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  const states = [
    ['normal', token.colorLink, 0],
    ['hover', token.colorLinkHover, 1],
    ['active', token.colorLinkActive, 1],
    ['disabled', token.colorTextDisabled, 0],
  ];
  return states.map((r) => r.join('|')).join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `link_matrix_${name}.txt`), rows(token));
  console.log(`wrote link_matrix_${name}.txt`);
}
