// 生成 Calendar 面板基准。
// 规则来源：antdv-next `calendar/style/index.js`：
//   fullBg/fullPanelBg = colorBgContainer / itemActiveBg = controlItemBgActive /
//   yearControlWidth 80 / monthControlWidth 70 / miniContentHeight 256（下拉 listHeight 同源）/
//   header padding paddingSM 0 / 头部控件间距 marginXS /
//   weekHeight = controlHeightSM*0.75 / th paddingInlineEnd paddingSM paddingBottom paddingXXS /
//   body padding paddingXS 0 / date margin 0 marginXS/2、padding paddingXS/2 paddingXS 0 /
//   dateValueHeight = controlHeightSM /
//   dateContentHeight = (fontHeightSM+marginXS)*3 + lineWidth*2 /
//   顶描边 lineWidthBold colorSplit（today 换 colorPrimary）/
//   cell 高 = dateValueHeight+dateContentHeight+paddingXS/2+lineWidthBold /
//   hover controlItemBgHover / 选中底 itemActiveBg + 值 colorPrimary / 补位 colorTextDisabled /
//   mode-switch = radio-button：高 controlHeight、paddingInline padding-lineWidth、
//   lineHeight controlHeight-2*lineWidth、外圆角 borderRadius（探针实测）、
//   选中字/边 colorPrimary、未选中悬停字 colorPrimary、选中悬停 colorPrimaryHover。
//
//   node harness/antd_baseline/gen_calendar_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  const dateValueHeight = token.controlHeightSM;
  const dateContentHeight =
    (token.fontHeightSM + token.marginXS) * 3 + token.lineWidth * 2;
  const datePadTop = token.paddingXS / 2;
  const dateBorder = token.lineWidthBold;
  return [
    token.colorBgContainer,
    token.colorSplit,
    token.colorText,
    token.colorTextDisabled,
    token.controlItemBgHover,
    token.controlItemBgActive,
    token.colorPrimary,
    token.colorPrimaryHover,
    token.colorBorder,
    token.colorTextQuaternary,
    token.colorBgElevated,
    token.paddingSM,
    80,
    70,
    token.marginXS,
    token.controlHeightSM * 0.75,
    token.paddingXXS,
    token.paddingXS,
    token.marginXS / 2,
    datePadTop,
    dateValueHeight,
    dateContentHeight,
    dateBorder,
    dateValueHeight + dateContentHeight + datePadTop + dateBorder,
    token.fontSize,
    token.fontHeight,
    token.controlHeight,
    token.padding - token.lineWidth,
    token.borderRadius,
    token.controlHeight - token.lineWidth * 2,
    256,
    token.fontSizeIcon,
    token.fontWeightStrong,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `calendar_matrix_${name}.txt`), line(token));
  console.log(`wrote calendar_matrix_${name}.txt`);
}