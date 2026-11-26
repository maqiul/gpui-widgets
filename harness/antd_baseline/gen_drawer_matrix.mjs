// 生成 Drawer 基准。
// 规则来源：antdv-next `drawer/style/index.js`：
//   遮罩 colorBgMask / section colorBgElevated / header+footer 分隔线 colorSplit；
//   header padding padding paddingLG / fontSize fontSizeLG / lineHeight lineHeightLG(1.5)；
//   close 尺寸 fontSizeLG+paddingXS / radius borderRadiusSM / colorIcon、hover colorIconHover +
//   colorBgTextHover、active colorBgTextActive / close 与标题间距 marginXS；
//   body padding paddingLG / footer padding footerPaddingBlock(paddingXS) footerPaddingInline(padding)。
//
//   node harness/antd_baseline/gen_drawer_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  return [
    token.colorBgMask,
    token.colorBgElevated,
    token.colorSplit,
    token.colorText,
    token.colorIcon,
    token.colorIconHover,
    token.colorBgTextHover,
    token.colorBgTextActive,
    378,
    token.padding,
    token.paddingLG,
    token.fontSizeLG,
    token.fontSizeLG * token.lineHeightLG,
    token.lineWidth,
    token.fontSizeLG + token.paddingXS,
    token.marginXS,
    token.borderRadiusSM,
    token.paddingLG,
    token.paddingXS,
    token.padding,
    token.marginXS,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `drawer_matrix_${name}.txt`), line(token));
  console.log(`wrote drawer_matrix_${name}.txt`);
}