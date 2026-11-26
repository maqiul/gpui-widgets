// 生成 Popconfirm 面板基准。
// 规则来源：antdv-next `popconfirm/style/index.js` + `popover/style/index.js`：
//   message marginBottom marginXS / 图标 colorWarning fontSize lineHeight 1 marginInlineEnd marginXS /
//   标题 fontWeightStrong colorTextHeading（only-child 降 normal）/ 描述 marginTop marginXXS colorText /
//   按钮 marginInlineStart marginXS、均 size small（okType 走 Button type：primary 实心 / danger 描边）/
//   外壳 = Popover：bg colorBgElevated、innerPadding 12、radius borderRadiusLG、fontSize、lineHeight fontHeight。
//
//   node harness/antd_baseline/gen_popconfirm_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  return [
    token.colorWarning,
    token.colorTextHeading,
    token.colorText,
    token.colorBgElevated,
    token.marginXS,
    token.fontSize,
    token.marginXXS,
    token.fontWeightStrong,
    12,
    token.borderRadiusLG,
    token.fontHeight,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `popconfirm_matrix_${name}.txt`), line(token));
  console.log(`wrote popconfirm_matrix_${name}.txt`);
}