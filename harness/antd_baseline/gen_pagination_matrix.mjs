// 生成 Pagination 页码项 / 翻页按钮基准。
// 规则来源：antdv-next Pagination 样式与 computed style（harness/_pagination_probe.html 核对）：
//   - 项 32（small 24）、间距 8（small 4）、圆角 6，描边默认透明；
//   - 选中项主色文字 + 主色描边，底色仍是 colorBgContainer，字重 600；
//   - 普通项悬停 colorBgTextHover；翻页按钮无底色，禁用 colorTextDisabled（opacity 1）。
//
//   node harness/antd_baseline/gen_pagination_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

const round = (v) => (Math.round(v * 10) / 10).toString();

function itemRows(token) {
  const rows = [];
  const pad = token.marginXXS * 1.5;
  for (const small of [false, true]) {
    for (const active of [false, true]) {
      for (const disabled of [false, true]) {
        const fg = disabled
          ? token.colorTextDisabled
          : active
            ? token.colorPrimary
            : token.colorText;
        rows.push(
          [
            small ? '1' : '0',
            active ? '1' : '0',
            disabled ? '1' : '0',
            token.colorBgContainer,
            fg,
            active && !disabled ? token.colorPrimary : 'transparent',
            disabled || active ? token.colorBgContainer : token.colorBgTextHover,
            disabled ? token.colorTextDisabled : active ? token.colorPrimaryHover : token.colorText,
            active && !disabled ? token.colorPrimaryHover : 'transparent',
            small ? token.controlHeightSM : token.controlHeight,
            small ? token.paddingXXS : token.marginXS,
            token.borderRadius,
            token.fontSize,
            active ? token.fontWeightStrong : 400,
            token.lineWidth,
            round(pad),
          ].join('|'),
        );
      }
    }
  }
  return rows.join('\n') + '\n';
}

function navRows(token) {
  const rows = [];
  const pad = token.marginXXS * 1.5;
  for (const small of [false, true]) {
    for (const disabled of [false, true]) {
      rows.push(
        [
          small ? '1' : '0',
          disabled ? '1' : '0',
          'transparent',
          disabled ? token.colorTextDisabled : token.colorText,
          'transparent',
          disabled ? 'transparent' : token.colorBgTextHover,
          disabled ? token.colorTextDisabled : token.colorText,
          'transparent',
          small ? token.controlHeightSM : token.controlHeight,
          small ? token.paddingXXS : token.marginXS,
          token.borderRadius,
          token.fontSize,
          400,
          token.lineWidth,
          round(pad),
        ].join('|'),
      );
    }
  }
  return rows.join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `pagination_item_matrix_${name}.txt`), itemRows(token));
  fs.writeFileSync(path.join(dir, `pagination_nav_matrix_${name}.txt`), navRows(token));
  console.log(`wrote pagination_*_matrix_${name}.txt`);
}
