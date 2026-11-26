// 生成 Notification / Message 面板基准。
// 规则来源：antdv-next `notification/style/index.js` prepareNotificationToken +
// `notification/style/notification.js` genListItemSharedStyle / genNotificationItemStyle +
// `message/style/index.js` prepareMessageToken / genMessageItemStyle：
//   Notification：宽 384（组件默认 token）；padding paddingMD / paddingContentHorizontalLG(=sizeLG=paddingLG)；
//   bg colorBgElevated；radius borderRadiusLG；图标 fontSizeLG*lineHeightLG(=fontHeightLG) 四态色；
//   wrapper gap marginSM；section gap marginXS；标题 fontSizeLG/fontHeightLG colorTextHeading；
//   描述 fontSize colorText；关闭钮 controlHeightLG*0.55、top paddingMD、right paddingLG、radius borderRadiusSM、
//   colorIcon / hover colorIconHover+colorBgTextHover / active colorBgTextActive；
//   closable 右留 paddingLG；actions marginTop marginSM；堆栈 gap margin、边距 marginLG。
//   Message：padding (controlHeightLG-fontHeight)/2 / paddingSM；图标 fontSizeLG；gap marginXS；
//   正文 fontSize/fontHeight；radius borderRadiusLG；堆栈 gap margin、边距 marginLG。
//
//   node harness/antd_baseline/gen_notice_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function notificationLine(token) {
  return [
    token.colorBgElevated,
    token.colorInfo,
    token.colorSuccess,
    token.colorWarning,
    token.colorError,
    token.colorTextHeading,
    token.colorText,
    token.colorIcon,
    token.colorIconHover,
    token.colorBgTextHover,
    token.colorBgTextActive,
    token.paddingMD,
    token.paddingLG,
    token.borderRadiusLG,
    token.fontSize,
    token.fontHeight,
    token.fontSizeLG,
    token.fontHeightLG,
    token.marginSM,
    token.marginXS,
    token.controlHeightLG * 0.55,
    token.borderRadiusSM,
    token.margin,
    token.marginLG,
    384,
  ].join('|') + '\n';
}

function messageLine(token) {
  return [
    (token.controlHeightLG - token.fontHeight) / 2,
    token.paddingSM,
    token.borderRadiusLG,
    token.fontSize,
    token.fontHeight,
    token.fontSizeLG,
    token.marginXS,
    token.margin,
    token.marginLG,
    token.colorBgElevated,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `notification_matrix_${name}.txt`), notificationLine(token));
  fs.writeFileSync(path.join(dir, `message_matrix_${name}.txt`), messageLine(token));
  console.log(`wrote notification_matrix_${name}.txt / message_matrix_${name}.txt`);
}