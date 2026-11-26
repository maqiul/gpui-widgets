// 生成 Timeline 状态基准。
// 规则来源：antdv-next `timeline/style/index.js` + `Timeline.js`（dot 型 Steps）：
//   - 节点圆环 10×10、描边 lineWidthBold、透明填充；blue/red/green/gray 依次映射
//     colorPrimary/colorError/colorSuccess/colorTextDisabled；
//   - 进行中（pending/loading）节点主色，内侧主色点；其后的连接线 dotted；
//   - 连接线 colorSplit、lineWidthBold；从节点下 17 起、越过项底 7；
//   - 有标题两栏：间距 2*margin+8=40、项下 padding*1.25=20、栏最小高 24；
//   - 无标题左轨：项最小高 48、项下 paddingSM=12、节点距顶 7、节点与内容 16；
//   - 标题/内容 fontSize/fontHeight 14/22；辅助文字 12/20。
//
//   node harness/antd_baseline/gen_timeline_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;
const COLORS = ['blue', 'red', 'green', 'gray'];

function rows(token) {
  const colorOf = {
    blue: token.colorPrimary,
    red: token.colorError,
    green: token.colorSuccess,
    gray: token.colorTextDisabled,
  };
  const out = [];
  for (const color of COLORS) {
    for (const loading of [0, 1]) {
      const dotBorder = loading ? token.colorPrimary : colorOf[color];
      const innerSize = loading ? 4 : 0;
      out.push(
        [
          color,
          loading,
          dotBorder,
          'rgba(0,0,0,0)',
          10,
          2,
          innerSize,
          dotBorder,
          token.colorSplit,
          token.lineWidthBold,
          loading,
          token.colorText,
          token.colorText,
          token.colorTextDescription,
          token.fontSize,
          token.fontHeight,
          token.fontSizeSM,
          token.fontHeightSM,
          token.paddingSM,
          token.padding * 1.25,
          17,
          7,
          16,
          token.padding * 2 + 8,
          24,
          7,
          48,
          8,
        ].join('|'),
      );
    }
  }
  return out.join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `timeline_matrix_${name}.txt`), rows(token));
  console.log(`wrote timeline_matrix_${name}.txt`);
}
