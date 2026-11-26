// 生成 Radio 状态基准。
// 规则来源：antdv-next Radio 样式与 computed style（harness/_radio_probe.html 核对）：
//   - 圆 16×16、1px 描边、border-radius 50%；
//   - 未选中 colorBgContainer + colorBorder；悬停边框主色；
//   - 选中主色填充/描边 + 6×6 colorTextLightSolid 内点；悬停边框 colorPrimaryHover；
//   - 禁用底 colorBgContainerDisabled、描边 colorBorder、内点/标签 colorTextDisabled；
//   - 标签字号 fontSize、行高 fontHeight、与圆间距 paddingXS(8)。
//
//   node harness/antd_baseline/gen_radio_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  const out = [];
  for (const disabled of [false, true]) {
    for (const selected of [false, true]) {
      const bg = selected
        ? disabled
          ? token.colorBgContainerDisabled
          : token.colorPrimary
        : disabled
          ? token.colorBgContainerDisabled
          : token.colorBgContainer;
      const border = selected && !disabled ? token.colorPrimary : token.colorBorder;
      const hover = disabled
        ? token.colorBorder
        : selected
          ? token.colorPrimaryHover
          : token.colorPrimary;
      const dot = !selected
        ? 'transparent'
        : disabled
          ? token.colorTextDisabled
          : token.colorTextLightSolid;
      out.push(
        [
          disabled ? '1' : '0',
          selected ? '1' : '0',
          bg,
          border,
          hover,
          dot,
          disabled ? token.colorTextDisabled : token.colorText,
          16,
          6,
          8,
          token.lineWidth,
          token.paddingXS,
          token.fontSize,
          token.fontHeight,
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
  fs.writeFileSync(path.join(dir, `radio_matrix_${name}.txt`), rows(token));
  console.log(`wrote radio_matrix_${name}.txt`);
}
