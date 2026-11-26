// 生成 Tabs 状态基准。
// 规则来源：antdv-next `tabs/style/index.js` prepareComponentToken + genTabStyle + genCardStyle：
//   - 下划线：项内边距块 8/12/16（paddingXS/paddingSM/padding），字号 14/14/16、
//     行高 22/22/24；文字 colorText / colorPrimaryHover / colorPrimaryActive / colorPrimary /
//     colorTextDisabled；指示条 lineWidthBold，分隔线 colorBorderSecondary；
//   - 项间距 32、导航底部外边距 16；
//   - 卡片：底色 colorFillAlter、选中 colorBgContainer、边框 colorBorderSecondary、
//     高 32/40/48、内边距块 4/8/11、内边距行 8/16/16、圆角 6/8/8、卡片间隙 marginXXS/2；
//   - 关闭按钮 colorIcon、左间距 marginXS、字号 fontSizeSM。
//
//   node harness/antd_baseline/gen_tabs_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;
const STATES = ['normal', 'hover', 'active', 'selected', 'disabled', 'selected-disabled'];

function textColor(token, state) {
  switch (state) {
    case 'normal':
      return token.colorText;
    case 'hover':
      return token.colorPrimaryHover;
    case 'active':
      return token.colorPrimaryActive;
    case 'selected':
      return token.colorPrimary;
    case 'disabled':
    case 'selected-disabled':
      return token.colorTextDisabled;
    default:
      throw new Error(`unknown state ${state}`);
  }
}

function rows(token) {
  const line = token.lineWidth;
  const fontHeight = [token.fontHeight, token.fontHeight, token.fontHeightLG];
  const cardHeight = [token.controlHeight, token.controlHeightLG, token.controlHeightLG + 8];
  const metrics = [
    32, // horizontalItemGutter
    token.margin, // horizontalMargin
    token.paddingXS,
    token.paddingSM,
    token.padding,
    token.fontSize,
    token.fontSize,
    token.fontSizeLG,
    fontHeight[0],
    fontHeight[1],
    fontHeight[2],
    token.lineWidthBold,
    line,
    cardHeight[0],
    cardHeight[1],
    cardHeight[2],
    (cardHeight[0] - fontHeight[0]) / 2 - line,
    (cardHeight[1] - fontHeight[1]) / 2 - line,
    (cardHeight[2] - fontHeight[2]) / 2 - line,
    token.paddingXS,
    token.padding,
    token.padding,
    token.borderRadius,
    token.borderRadiusLG,
    token.borderRadiusLG,
    token.marginXXS / 2,
    token.marginXS,
    token.fontSizeSM,
  ];
  return (
    ['line', 'card']
      .flatMap((kind) =>
        STATES.map((state) => {
          const selected = state === 'selected' || state === 'selected-disabled';
          const text = textColor(token, state);
          const bg = kind === 'card'
            ? selected ? token.colorBgContainer : token.colorFillAlter
            : 'rgba(0,0,0,0)';
          const border = kind === 'card' ? token.colorBorderSecondary : 'rgba(0,0,0,0)';
          const ink = selected ? 1 : 0;
          return [kind, state, text, bg, border, ink, ...metrics].join('|');
        }),
      )
      .join('\n') + '\n'
  );
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `tabs_matrix_${name}.txt`), rows(token));
  console.log(`wrote tabs_matrix_${name}.txt`);
}
