// 生成 Upload Dragger 基准。
// 规则来源：antdv-next `upload/style/dragger.js`：
//   bg=colorFillAlter、border=colorBorder、hover=colorPrimaryHover、
//   icon=colorPrimary/fontSizeHeading3*2、text=colorTextHeading/fontSizeLG、
//   hint=colorTextDescription/fontSize、disabled=colorTextDisabled。
//
//   node harness/antd_baseline/gen_upload_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;

function line(token) {
  return [
    token.colorFillAlter,
    token.colorBorder,
    token.colorPrimaryHover,
    token.colorPrimary,
    token.colorTextHeading,
    token.colorTextDescription,
    token.colorTextDisabled,
    token.padding,
    token.fontSizeHeading3 * 2,
    Math.max(token.margin, token.fontSizeLG),
    token.fontSize,
    token.fontSizeLG,
    Math.round(token.fontSizeLG * token.lineHeightLG),
    token.marginXXS,
    token.fontSize,
    Math.round(token.fontSize * token.lineHeight),
    token.fontSize,
    token.marginXXS,
    token.borderRadiusLG,
  ].join('|') + '\n';
}

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `upload_matrix_${name}.txt`), line(token));
  console.log(`wrote upload_matrix_${name}.txt`);
}
