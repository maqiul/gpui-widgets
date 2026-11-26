// 生成 NavMenu / Countdown 基准。
// 规则来源：antdv-next `menu/style/index.js` prepareComponentToken
// （itemHeight controlHeightLG；itemMarginInline/Block marginXXS；itemPaddingInline padding；
// itemBorderRadius borderRadiusLG；subMenuItemBorderRadius borderRadiusSM；
// iconSize fontSize；iconMarginInlineEnd controlHeightSM-fontSize；
// menuArrowSize fontSize/7*5；arrow insetInlineEnd margin；
// 父标题 paddingInlineEnd = arrowSize+padding+marginXS；subMenuItemBg colorFillAlter；
// itemColor colorText；itemDisabledColor colorTextDisabled；itemSelectedColor colorPrimary；
// itemHoverBg colorBgTextHover；itemSelectedBg controlItemBgActive；
// `theme.js` root borderInlineEnd = activeBarBorderWidth(lineWidth) solid colorSplit）+
// `@v-c/menu/dist/Menu.js`（inlineIndent 默认 24，paddingLeft = indent×层级）+
// `statistic/style/index.js` prepareComponentToken（titleFontSize fontSize；
// contentFontSize fontSizeHeading3；content colorTextHeading；title colorTextDescription；
// header paddingBottom marginXXS）。
//
//   node harness/antd_baseline/gen_navmenu_countdown_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function navmenuLine(token) {
  const arrowSize = (token.fontSize / 7) * 5;
  return [
    token.controlHeightLG,
    token.marginXXS,
    token.marginXXS,
    token.padding,
    token.borderRadiusLG,
    token.borderRadiusSM,
    24,
    token.fontSize,
    token.controlHeightSM - token.fontSize,
    arrowSize,
    token.margin,
    arrowSize + token.padding + token.marginXS,
    token.fontSize,
    token.colorFillAlter,
    token.colorText,
    token.colorTextDisabled,
    token.colorPrimary,
    token.colorBgTextHover,
    token.controlItemBgActive,
    token.lineWidth,
    token.colorSplit,
  ].join('|') + '\n';
}

function countdownLine(token) {
  return [
    token.fontSizeHeading3,
    token.colorTextHeading,
    token.fontSize,
    token.colorTextDescription,
    token.marginXXS,
    token.lineHeight,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `navmenu_matrix_${name}.txt`), navmenuLine(token));
  fs.writeFileSync(path.join(dir, `countdown_matrix_${name}.txt`), countdownLine(token));
  console.log(`wrote navmenu/countdown_matrix_${name}.txt`);
}