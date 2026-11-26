// 生成 Progress 状态基准。
// 规则来源：antdv-next Progress 样式与 computed style（harness/_progress_probe.html 核对）：
//   - body 行盒 flex、gap 8；轨道高 8（small 6）、圆角 100、rail = colorFillSecondary；
//   - 正常 / 进行中 colorInfo，成功 colorSuccess，异常 colorError；
//   - 成功 / 异常的指示器换成同色 14px 圆形图标，正常为百分比文字 colorText；
//   - 分段项默认宽 14、small 宽 2，段间距 2；指示器左边距 8。
//
//   node harness/antd_baseline/gen_progress_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

const STATUS = [
  ['normal', 0, 'colorInfo'],
  ['active', 1, 'colorInfo'],
  ['success', 2, 'colorSuccess'],
  ['exception', 3, 'colorError'],
];

function rows(token) {
  const out = [];
  const push = (statusIx, small, showInfo, colorKey) => {
    const icon = statusIx >= 2;
    const stroke = small ? 6 : 8;
    const font = small ? token.fontSizeSM : token.fontSize;
    out.push(
      [
        String(statusIx),
        small ? '1' : '0',
        showInfo ? '1' : '0',
        token[colorKey],
        token.colorFillSecondary,
        icon ? token[colorKey] : token.colorText,
        token.colorTextLightSolid,
        stroke,
        100,
        8,
        font,
        showInfo ? font : stroke,
        small ? 2 : 14,
        2,
      ].join('|'),
    );
  };

  push(0, false, true, 'colorInfo');
  push(0, true, true, 'colorInfo');
  push(0, false, false, 'colorInfo');
  push(2, false, true, 'colorSuccess');
  push(3, false, true, 'colorError');
  push(1, false, true, 'colorInfo');
  push(0, false, true, 'colorInfo');
  push(0, true, true, 'colorInfo');
  return out.join('\n') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `progress_matrix_${name}.txt`), rows(token));
  console.log(`wrote progress_matrix_${name}.txt`);
}
