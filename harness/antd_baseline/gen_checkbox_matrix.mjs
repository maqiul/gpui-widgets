// 生成 Checkbox 状态基准。
// 规则来源：antdv-next Checkbox 样式与 computed style（harness/_checkbox_probe.html 核对）：
//   - 方框 16×16、圆角 borderRadiusSM(4)、1px 描边；
//   - 未选中 colorBgContainer + colorBorder，悬停边框主色；
//   - 选中主色填充 + colorTextLightSolid 勾，悬停边框 colorPrimaryHover；
//   - 半选保持容器底，内部 8×8 主色方块（antdv-next 实测，不是横杠）；
//   - 禁用底 colorBgContainerDisabled、描边 colorBorder、勾/方块 colorTextDisabled；
//   - 标签字号 fontSize、行高 fontHeight，与方框间距 paddingXS(8)。
//
//   node harness/antd_baseline/gen_checkbox_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function rows(token) {
  const out = [];
  for (const disabled of [false, true]) {
    for (const mode of [0, 1, 2]) {
      const checked = mode === 1;
      const indet = mode === 2;
      const marked = checked || indet;
      const bg = checked
        ? disabled
          ? token.colorBgContainerDisabled
          : token.colorPrimary
        : disabled
          ? token.colorBgContainerDisabled
          : token.colorBgContainer;
      const border = checked && !disabled ? token.colorPrimary : token.colorBorder;
      const hover = disabled
        ? token.colorBorder
        : checked
          ? token.colorPrimaryHover
          : token.colorPrimary;
      const mark = !marked
        ? 'transparent'
        : disabled
          ? token.colorTextDisabled
          : checked
            ? token.colorTextLightSolid
            : token.colorPrimary;
      out.push(
        [
          disabled ? '1' : '0',
          String(mode),
          bg,
          border,
          hover,
          mark,
          disabled ? token.colorTextDisabled : token.colorText,
          16,
          token.borderRadiusSM,
          token.lineWidth,
          8,
          11,
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
  fs.writeFileSync(path.join(dir, `checkbox_matrix_${name}.txt`), rows(token));
  console.log(`wrote checkbox_matrix_${name}.txt`);
}
