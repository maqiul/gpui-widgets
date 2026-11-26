// 生成 Modal 基准。
// 规则来源：antdv-next `modal/style/index.js` prepareComponentToken/prepareToken：
//   contentBg colorBgElevated / titleColor colorTextHeading / 遮罩 colorBgMask；
//   close colorIcon、hover colorIconHover + colorBgTextHover、active colorBgTextActive；
//   contentPadding paddingMD paddingContentHorizontalLG(=paddingLG) / radius borderRadiusLG；
//   headerMarginBottom marginXS / titleFontSize fontSizeHeading5 / titleLineHeight lineHeightHeading5(1.5)；
//   modalHeaderHeight = titleLineHeight*fontSize + 2*padding / closeBtnSize controlHeight；
//   close offset = (headerHeight-closeSize)/2 / close radius borderRadiusSM / close-x fontSizeLG；
//   footerMarginTop marginSM / btn+btn marginXS / body fontSize lineHeight。
//
//   node harness/antd_baseline/gen_modal_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  const titleLineHeight = token.fontSizeHeading5 * token.lineHeightHeading5;
  const headerHeight = titleLineHeight + token.padding * 2;
  return [
    token.colorBgMask,
    token.colorBgElevated,
    token.colorTextHeading,
    token.colorIcon,
    token.colorIconHover,
    token.colorBgTextHover,
    token.colorBgTextActive,
    520,
    token.paddingMD,
    token.paddingLG,
    token.borderRadiusLG,
    token.marginXS,
    token.fontSizeHeading5,
    titleLineHeight,
    headerHeight,
    token.controlHeight,
    (headerHeight - token.controlHeight) / 2,
    token.borderRadiusSM,
    token.fontSizeLG,
    token.marginSM,
    token.marginXS,
    token.fontSize,
    token.fontHeight,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `modal_matrix_${name}.txt`), line(token));
  console.log(`wrote modal_matrix_${name}.txt`);
}