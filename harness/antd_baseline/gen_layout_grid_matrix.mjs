// 生成 Layout / Grid 基准。
// 规则来源：antdv-next `layout/style/index.js` prepareComponentToken
// （headerBg/siderBg '#001529'、triggerBg '#002140' 字面量；headerHeight controlHeight×2；
// headerPadding 0 controlHeightLG×1.25；footerPadding controlHeightSM × 同横向；
// bodyBg/footerBg colorBgLayout；headerColor/footerColor colorText；
// triggerHeight controlHeightLG+marginXXS×2；lightSiderBg/lightTriggerBg colorBgContainer）+
// `layout/style/sider.js`（light trigger color colorText）+
// `grid/style/index.js`（gridColumns 24；flex 0 0 span/24%；offset marginInlineStart offset/24%；
// gutter 拆半：Row 负外边距 -g/2、Col 内边距 g/2，见 grid/Row.js 的 gutter 处理）。
//
//   node harness/antd_baseline/gen_layout_grid_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function layoutLine(token) {
  const paddingInline = token.controlHeightLG * 1.25;
  return [
    token.controlHeight * 2,
    paddingInline,
    '#001529',
    token.colorText,
    token.controlHeightSM,
    paddingInline,
    token.colorBgLayout,
    token.colorText,
    token.fontSize,
    token.colorBgLayout,
    '#001529',
    token.colorBgContainer,
    token.controlHeightLG + token.marginXXS * 2,
    '#002140',
    token.colorTextLightSolid,
    token.colorBgContainer,
    token.colorText,
  ].join('|') + '\n';
}

function gridLine(token) {
  const columns = 24;
  const gutter = 16;
  return [
    columns,
    (12 / columns) * 100,
    (8 / columns) * 100,
    (6 / columns) * 100,
    -gutter / 2,
    gutter / 2,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `layout_matrix_${name}.txt`), layoutLine(token));
  fs.writeFileSync(path.join(dir, `grid_matrix_${name}.txt`), gridLine(token));
  console.log(`wrote layout/grid_matrix_${name}.txt`);
}