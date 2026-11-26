// 生成 Alert 状态基准。
// 规则来源：antdv-next `alert/style/index.js` prepareComponentToken：
//   - 圆角 borderRadiusLG；
//   - 默认内边距 8×12，带描述 20×24；
//   - 图标间距 8 / 12，图标 14 / fontSizeHeading3；
//   - 带描述标题 fontSizeLG 且下边距 8；
//   - banner 去边框、去圆角。
//
//   node harness/antd_baseline/gen_alert_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function row(token, type, { description = false, banner = false } = {}) {
  const palette = {
    info: [token.colorInfoBg, token.colorInfoBorder, token.colorInfo],
    success: [token.colorSuccessBg, token.colorSuccessBorder, token.colorSuccess],
    warning: [token.colorWarningBg, token.colorWarningBorder, token.colorWarning],
    error: [token.colorErrorBg, token.colorErrorBorder, token.colorError],
  }[type];
  return [
    type,
    description ? 1 : 0,
    banner ? 1 : 0,
    palette[0],
    palette[1],
    palette[2],
    banner ? 0 : token.borderRadiusLG,
    description ? 20 : 8,
    description ? 24 : 12,
    description ? 12 : 8,
    description ? token.fontSizeHeading3 : 14,
    description ? token.fontSizeLG : token.fontSize,
    token.marginXS,
    token.fontHeight,
    banner ? 0 : token.lineWidth,
  ].join('|');
}

function rows(token) {
  return [
    row(token, 'info'),
    row(token, 'success'),
    row(token, 'warning'),
    row(token, 'error'),
    row(token, 'info', { description: true }),
    row(token, 'info', { banner: true }),
  ].join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `alert_matrix_${name}.txt`), rows(token));
  console.log(`wrote alert_matrix_${name}.txt`);
}
