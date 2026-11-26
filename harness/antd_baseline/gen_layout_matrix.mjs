// 生成 Collapse / Anchor / Form 面板基准。
// 规则来源：antdv-next `collapse/style/index.js` prepareComponentToken + genBaseStyle 尺寸档、
// `anchor/style/index.js` prepareComponentToken + mergeToken、
// `form/style/index.js` prepareComponentToken + genFormSize：
//   Collapse：headerBg = colorFillAlter 叠容器底；contentBg = colorBgContainer；border = colorBorder；
//     头字 colorTextHeading / 禁用 colorTextDisabled；radius = borderRadiusLG；
//     headerPadding 三档 12/16、8/12/8/8、16/24/16/16；body 16、12、24；箭头 fontSizeIcon、间距 marginSM。
//   Anchor：轨 lineWidthBold / colorSplit；ink colorPrimary；标题 colorText / 激活 colorPrimary /
//     悬停 colorPrimaryHover；link paddingBlock paddingXXS（子级减半）、paddingInlineStart padding；
//     标题下间距 fontSize/14×3；ink 高 fontHeight。
//   Form：标签 colorTextHeading；星号 colorError；提示 colorError / colorTextDescription；
//     标签高 controlHeight 三档（纵向 middle 为行高 auto）；星号间距 marginXXS；冒号 marginXXS/2 与 marginXS；
//     项间距 marginLG；纵向标签下内边距 paddingXS；提示字号 fontSize；extra 最小高 controlHeightSM。
//
//   node harness/antd_baseline/gen_layout_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';
import { FastColor } from '@ant-design/fast-color';

const dir = import.meta.dirname;

function collapseLine(t) {
  const headerBg = new FastColor(t.colorFillAlter).onBackground(t.colorBgContainer).toHexString();
  return [
    headerBg,
    t.colorBgContainer,
    t.colorBorder,
    t.colorTextHeading,
    t.colorTextDisabled,
    t.borderRadiusLG,
    t.paddingSM, t.padding, t.padding, t.padding,
    t.marginSM, t.fontSizeSM, t.fontSize, t.fontSize * t.lineHeight,
    t.paddingXS, t.paddingSM, t.paddingXS, t.paddingSM,
    t.padding, t.paddingLG, t.padding, t.paddingLG,
    t.fontSizeLG, t.fontSizeLG * t.lineHeightLG,
  ].join('|') + '\n';
}

function anchorLine(t) {
  return [
    t.lineWidthBold,
    t.colorSplit,
    t.colorPrimary,
    t.colorText,
    t.colorPrimary,
    t.colorPrimaryHover,
    t.paddingXXS,
    t.paddingXXS / 2,
    t.padding,
    (t.fontSize / 14) * 3,
    t.fontSize * t.lineHeight,
  ].join('|') + '\n';
}

function formLine(t) {
  return [
    t.colorTextHeading,
    t.colorError,
    t.colorError,
    t.colorTextDescription,
    t.controlHeight,
    t.fontSize * t.lineHeight,
    t.controlHeightSM,
    t.controlHeightSM,
    t.controlHeightLG,
    t.marginXXS,
    t.marginXXS / 2,
    t.marginXS,
    t.marginLG,
    t.paddingXS,
    t.fontSize,
    t.controlHeightSM,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const t = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `collapse_matrix_${name}.txt`), collapseLine(t));
  fs.writeFileSync(path.join(dir, `anchor_matrix_${name}.txt`), anchorLine(t));
  fs.writeFileSync(path.join(dir, `form_matrix_${name}.txt`), formLine(t));
  console.log(`wrote collapse/anchor/form_matrix_${name}.txt`);
}