// 生成 Transfer 基准。
// 规则来源：antdv-next `transfer/style/index.js`：
//   section border=colorBorder/radius=borderRadiusLG、header 高 controlHeightLG/
//   底 colorBgContainer/底线 colorSplit、item 高 controlHeight/内边距 (controlHeight-fontHeight)/2 + paddingSM、
//   选中 controlItemBgActive、悬停 controlItemBgHover、选中悬停 controlItemBgActiveHover、
//   禁用 colorTextDisabled/colorBgContainerDisabled、actions margin=marginXS/gap=marginXXS、
//   按钮=默认尺寸 primary、勾选框 controlInteractiveSize/borderRadiusSM。
//   listWidth/listHeight 是组件令牌默认 180/200；header 间距 4 为 antd 硬编码。
//
//   node harness/antd_baseline/gen_transfer_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  const fontHeight = Math.round(token.fontSize * token.lineHeight);
  const headerV = Math.ceil((token.controlHeightLG - token.lineWidth - fontHeight) / 2);
  return [
    token.colorBorder,
    token.colorSplit,
    token.colorBgContainer,
    token.colorText,
    token.controlItemBgActive,
    token.controlItemBgHover,
    token.controlItemBgActiveHover,
    token.colorTextDisabled,
    token.colorBgContainerDisabled,
    token.colorPrimary,
    token.colorTextLightSolid,
    token.colorPrimaryHover,
    180,
    200,
    token.controlHeightLG,
    headerV - token.lineWidth,
    headerV,
    token.paddingSM,
    4,
    token.controlHeight,
    (token.controlHeight - fontHeight) / 2,
    token.paddingSM,
    token.marginXS,
    token.borderRadiusLG,
    token.borderRadiusLG - token.lineWidth,
    token.controlInteractiveSize,
    token.borderRadiusSM,
    token.marginXS,
    token.marginXXS,
    token.controlHeight,
    token.borderRadius,
    token.fontSizeSM,
    token.fontSize,
    fontHeight,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `transfer_matrix_${name}.txt`), line(token));
  console.log(`wrote transfer_matrix_${name}.txt`);
}