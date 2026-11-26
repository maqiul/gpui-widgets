// 生成 Steps 状态基准。
// 规则来源：antdv-next `steps/style/status.js` + `prepareComponentToken`：
//   - filled：待办 colorFillTertiary/colorTextLabel；进行、完成 colorPrimaryBg/colorPrimary；
//     失败 colorErrorBg/colorError；当前节点主色（失败错误色）实底 + 白字；
//   - outlined：容器底；待办灰描边/字，进行、完成主色，失败错误色；当前节点加浅色底；
//   - 连接线：待办 colorTextDisabled，进行、完成 colorPrimary，失败 colorError；
//   - 节点 controlHeight；标题 fontSizeLG/fontHeightLG；点状节点 controlHeight/4，当前 controlHeightLG/4。
//
//   node harness/antd_baseline/gen_steps_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;
const STATUSES = ['wait', 'process', 'finish', 'error'];

function colors(token, variant, status, active, dot) {
  const title =
    status === 'wait'
      ? token.colorTextDescription
      : status === 'error'
        ? token.colorError
        : token.colorText;
  const content =
    status === 'wait'
      ? active
        ? token.colorText
        : token.colorTextDescription
      : status === 'error'
        ? token.colorError
        : active
          ? token.colorText
          : token.colorTextDescription;

  if (dot) {
    const bg = active
      ? status === 'error'
        ? token.colorError
        : token.colorPrimary
      : status === 'wait'
        ? token.colorTextDisabled
        : status === 'error'
          ? token.colorError
          : token.colorPrimary;
    return [bg, 'rgba(0,0,0,0)', active ? token.colorTextLightSolid : bg, title, content];
  }

  if (active && variant === 'filled') {
    const solid = status === 'error' ? token.colorError : token.colorPrimary;
    return [solid, 'rgba(0,0,0,0)', token.colorTextLightSolid, title, content];
  }

  let bg;
  let border;
  let text;
  if (variant === 'filled') {
    bg =
      status === 'wait'
        ? token.colorFillTertiary
        : status === 'error'
          ? token.colorErrorBg
          : token.colorPrimaryBg;
    border = 'rgba(0,0,0,0)';
    text =
      status === 'wait'
        ? token.colorTextLabel
        : status === 'error'
          ? token.colorError
          : token.colorPrimary;
  } else {
    bg = token.colorBgContainer;
    border =
      status === 'wait'
        ? token.colorTextDisabled
        : status === 'error'
          ? token.colorError
          : token.colorPrimary;
    text = border;
  }
  if (active && variant === 'outlined') {
    bg =
      status === 'wait'
        ? token.colorFillTertiary
        : status === 'error'
          ? token.colorErrorBg
          : token.colorPrimaryBg;
  }
  return [bg, border, text, title, content];
}

function rows(token) {
  const rail = {
    wait: token.colorTextDisabled,
    process: token.colorPrimary,
    finish: token.colorPrimary,
    error: token.colorError,
  };
  const metrics = [
    token.controlHeight,
    token.fontSize,
    token.fontSizeLG,
    token.fontHeightLG,
    token.fontSize,
    token.fontHeight,
    token.lineWidth,
    token.controlHeight / 4,
    token.controlHeightLG / 4,
  ];
  const out = [];
  for (const variant of ['filled', 'outlined']) {
    for (const status of STATUSES) {
      for (const active of [0, 1]) {
        const [bg, border, text, title, content] = colors(token, variant, status, active === 1, false);
        out.push([variant, status, active, 0, bg, border, text, title, content, rail[status], ...metrics].join('|'));
      }
    }
  }
  for (const status of STATUSES) {
    for (const active of [0, 1]) {
      const [bg, border, text, title, content] = colors(token, 'filled', status, active === 1, true);
      out.push(['filled', status, active, 1, bg, border, text, title, content, rail[status], ...metrics].join('|'));
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
  fs.writeFileSync(path.join(dir, `steps_matrix_${name}.txt`), rows(token));
  console.log(`wrote steps_matrix_${name}.txt`);
}
