// 生成 Dropdown / ContextMenu 菜单基准。
// 规则来源：antdv-next dropdown 菜单 + antd 实测 tokens；菜单项与 Select 选项同源
// （高度 32 = 22 行高 + 5×2 内边距），已用 harness/_dropdown_probe.html 的 computed style 核对：
//   - 容器 paddingXXS / borderRadiusLG / colorBgElevated，无描边；
//   - 危险项 colorError、禁用项 colorTextDisabled（opacity 保持 1）；
//   - 分隔线 colorSplit、1px、上下 margin 4。
//
//   node harness/antd_baseline/gen_menu_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

const round = (v) => (Math.round(v * 10) / 10).toString();

function items(token) {
  const padBlock = (token.controlHeight - token.fontSize * token.lineHeight) / 2;
  const rows = [];
  for (const danger of [false, true]) {
    for (const disabled of [false, true]) {
      const fg = disabled
        ? token.colorTextDisabled
        : danger
          ? token.colorError
          : token.colorText;
      rows.push(
        [
          danger ? '1' : '0',
          disabled ? '1' : '0',
          'transparent',
          fg,
          token.controlItemBgHover,
          token.controlHeight,
          round(padBlock),
          token.controlPaddingHorizontal,
          token.borderRadiusSM,
          token.fontSize,
          token.fontHeight,
        ].join('|'),
      );
    }
  }
  return rows.join('\n') + '\n';
}

const menu = (token) =>
  [token.colorBgElevated, token.borderRadiusLG, token.paddingXXS].join('|') + '\n';

const divider = (token) =>
  [token.colorSplit, token.lineWidth, token.paddingXXS].join('|') + '\n';

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `menu_item_matrix_${name}.txt`), items(token));
  fs.writeFileSync(path.join(dir, `menu_panel_matrix_${name}.txt`), menu(token));
  fs.writeFileSync(path.join(dir, `menu_divider_matrix_${name}.txt`), divider(token));
  console.log(`wrote menu_*_matrix_${name}.txt`);
}
