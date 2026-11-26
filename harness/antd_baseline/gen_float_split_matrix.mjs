// 生成 FloatButton / Splitter 基准。
// 规则来源：antdv-next `float-button/style/index.js`（floatButtonSize = controlHeightLG、
// floatButtonIconSize = fontSizeIcon × 1.5、inset marginXXL / marginLG）+
// `float-button/style/button.js`（padding paddingXXS 0、gap paddingXXS/2、individual boxShadowSecondary）+
// `button/style/index.js`（circle 50%、size=large 故 square 走 borderRadiusLG；default/primary 三态色）+
// `splitter/style/index.js` prepareComponentToken（bar 2 / trigger 6 / draggable 20）与
// dragger ::before 三态（controlItemBgHover / controlItemBgActive / controlItemBgActiveHover）、
// ::after colorFill、panel padding 0 1px、collapse-bar fontSizeSM × controlHeightSM / borderRadiusXS。
//
//   node harness/antd_baseline/gen_float_split_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function floatButtonLine(token) {
  return [
    token.controlHeightLG,
    token.fontSizeIcon * 1.5,
    token.paddingXXS,
    token.paddingXXS / 2,
    token.marginXXL,
    token.marginLG,
    token.borderRadiusLG,
    token.lineWidth,
    token.colorBgContainer,
    token.colorBorder,
    token.colorText,
    token.colorPrimaryHover,
    token.colorPrimaryActive,
    token.colorPrimary,
    token.colorTextLightSolid,
    token.colorBgContainerDisabled,
    token.colorTextDisabled,
    token.colorBorder,
  ].join('|') + '\n';
}

function splitterLine(token) {
  return [
    token.splitBarSize || 2,
    token.splitTriggerSize || 6,
    token.splitBarDraggableSize ?? token.resizeSpinnerSize ?? 20,
    1,
    token.controlItemBgHover,
    token.controlItemBgActive,
    token.controlItemBgActiveHover,
    token.colorFill,
    token.fontSizeSM,
    token.controlHeightSM,
    token.borderRadiusXS,
    token.colorText,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `floatbutton_matrix_${name}.txt`), floatButtonLine(token));
  fs.writeFileSync(path.join(dir, `splitter_matrix_${name}.txt`), splitterLine(token));
  console.log(`wrote floatbutton/splitter_matrix_${name}.txt`);
}