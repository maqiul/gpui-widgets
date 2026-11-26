// 生成 Kbd / Scrollbar / Stepper 基准。
// 规则来源：antdv-next `typography/style/mixins.js` getResetStyles kbd
// （margin 0 0.2em；paddingInline 0.4em；paddingBlock 0.15em/0.1em；fontSize 90%；
// fontFamilyCode；bg rgba(150,150,150,0.06) 字面量；border lineWidth solid rgba(100,100,100,0.2)；
// borderBottomWidth 2；borderRadius 3 字面量；em 基准 = kbd 自身 fontSize）+
// `@v-c/virtual-list/dist/ScrollBar.js`（容器 8px；滑块 borderRadius 99 药丸；
// background var(--vc-virtual-list-scrollbar-bg, rgba(0,0,0,0.5))；
// antd table/style/index.js:28 与 tree:46 写 --rc-… 变量名失配，覆写失效 → 实测回退默认）+
// `scrollbarUtil.js` getSpinSize（container/scrollRange*container，MIN_SIZE 20，floor）+
// `table/style/virtual.js:9-12` 轨道 hover colorFillQuaternary +
// Stepper 无 antd 横向对应 → 映射 InputNumber 容器令牌
// （controlHeight/borderRadius/lineWidth/colorBorder/colorBgContainer/
// colorBgContainerDisabled/colorTextDisabled）+ 通用交互态
// （hover colorBgTextHover；active controlItemBgActive）；button_size = controlHeight-2*lineWidth。
//
//   node harness/antd_baseline/gen_kbd_scrollbar_stepper_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function kbdLine(token) {
  const fontSize = token.fontSize * 0.9;
  return [
    fontSize,
    fontSize * 0.4,
    fontSize * 0.15,
    fontSize * 0.1,
    fontSize * 0.2,
    3,
    token.lineWidth,
    2,
    'rgba(150,150,150,0.06)',
    'rgba(100,100,100,0.2)',
    token.colorText,
  ].join('|') + '\n';
}

function scrollbarLine(token) {
  return [
    8,
    99,
    20,
    'rgba(0,0,0,0.5)',
    token.colorFillQuaternary,
  ].join('|') + '\n';
}

function stepperLine(token) {
  return [
    token.controlHeight,
    token.borderRadius,
    token.lineWidth,
    token.colorBorder,
    token.colorBgContainer,
    token.colorBgContainerDisabled,
    token.colorTextDisabled,
    token.controlHeight - 2 * token.lineWidth,
    token.fontSize,
    token.colorText,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `kbd_matrix_${name}.txt`), kbdLine(token));
  fs.writeFileSync(path.join(dir, `scrollbar_matrix_${name}.txt`), scrollbarLine(token));
  fs.writeFileSync(path.join(dir, `stepper_matrix_${name}.txt`), stepperLine(token));
  console.log(`wrote kbd/scrollbar/stepper_matrix_${name}.txt`);
}