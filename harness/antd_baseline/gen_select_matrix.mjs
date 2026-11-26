// 生成 Select 触发框 / 菜单项 / 菜单容器基准。
// 规则来源：antdv-next dist/select/style/{token,select-input,dropdown}.js，
// 禁用与状态叠加已用浏览器 computed style 核对（见 harness/_select_probe.html）：
//   - 禁用不改正文色，error/warning 仍取状态色；
//   - filled 的 error/warning 正文取 colorError/colorWarning；
//   - 禁用 outlined/filled 描边取 colorBorder（underlined 保留状态描边，borderless 透明）。
//
//   node harness/antd_baseline/gen_select_matrix.mjs
import fs from 'node:fs';
import path from 'node:path';

const dir = import.meta.dirname;
const VARIANTS = ['outlined', 'filled', 'borderless', 'underlined'];
const STATUSES = ['default', 'error', 'warning'];
const SIZES = ['small', 'middle', 'large'];
const STATES = [
  ['normal', false, false],
  ['hover', true, false],
  ['focused', false, true],
];

const round = (v) => (Math.round(v * 10) / 10).toString();

function paddingBlock(height, fontSize, lineHeight, line, ceil) {
  const raw = (height - fontSize * lineHeight) / 2;
  const rounded = ceil ? Math.ceil(raw * 10) / 10 : Math.round(raw * 10) / 10;
  return Math.max(rounded - line, 0);
}

function trigger(token) {
  const line = token.lineWidth;
  const heights = [token.controlHeightSM, token.controlHeight, token.controlHeightLG];
  const fonts = [token.fontSize, token.fontSize, token.fontSizeLG];
  const ratios = [token.lineHeight, token.lineHeight, token.lineHeightLG];
  const lines = [token.fontHeight, token.fontHeight, token.fontHeightLG];
  const radii = [token.borderRadiusSM, token.borderRadius, token.borderRadiusLG];
  const paddings = [token.paddingXS, token.paddingSM, token.paddingSM];
  const rows = [];
  for (const variant of VARIANTS) {
    for (const status of STATUSES) {
      for (let size = 0; size < 3; size += 1) {
        for (const [stateName, hover, focused] of STATES) {
          for (const disabled of [false, true]) {
            const fg =
              status === 'error'
                ? token.colorError
                : status === 'warning'
                  ? token.colorWarning
                  : token.colorText;
            const statusBorder =
              status === 'error'
                ? token.colorError
                : status === 'warning'
                  ? token.colorWarning
                  : token.colorBorder;
            const hoverBorder =
              status === 'error'
                ? token.colorErrorBorderHover
                : status === 'warning'
                  ? token.colorWarningBorderHover
                  : token.colorPrimaryHover;
            const focusBorder = status === 'default' ? token.colorPrimary : statusBorder;
            const outline =
              status === 'error'
                ? token.colorErrorOutline
                : status === 'warning'
                  ? token.colorWarningOutline
                  : token.controlOutline;

            let bg;
            if (disabled) {
              bg = variant === 'borderless' ? 'transparent' : token.colorBgContainerDisabled;
            } else if (variant === 'outlined' || variant === 'underlined') {
              bg = token.colorBgContainer;
            } else if (variant === 'borderless') {
              bg = 'transparent';
            } else if (focused) {
              bg = token.colorBgContainer;
            } else if (hover) {
              bg =
                status === 'error'
                  ? token.colorErrorBgHover
                  : status === 'warning'
                    ? token.colorWarningBgHover
                    : token.colorFillSecondary;
            } else {
              bg =
                status === 'error'
                  ? token.colorErrorBg
                  : status === 'warning'
                    ? token.colorWarningBg
                    : token.colorFillTertiary;
            }

            let border;
            if (disabled) {
              border =
                variant === 'borderless'
                  ? 'transparent'
                  : variant === 'underlined'
                    ? statusBorder
                    : token.colorBorder;
            } else if (focused) {
              border = variant === 'borderless' ? 'transparent' : focusBorder;
            } else if (hover) {
              border =
                variant === 'outlined' || variant === 'underlined' ? hoverBorder : 'transparent';
            } else {
              border =
                variant === 'outlined' || variant === 'underlined'
                  ? statusBorder
                  : 'transparent';
            }

            const border4 =
              variant === 'borderless'
                ? ['transparent', 'transparent', 'transparent', 'transparent']
                : variant === 'underlined'
                  ? ['transparent', 'transparent', border, 'transparent']
                  : [border, border, border, border];
            const width4 =
              variant === 'underlined' ? [line, 0, line, 0] : [line, line, line, line];

            let shadowW = 0;
            let shadowColor = 'transparent';
            if (focused && !disabled) {
              if (variant === 'outlined') {
                shadowW = token.controlOutlineWidth;
                shadowColor = outline;
              } else if (variant === 'borderless') {
                shadowW = token.lineWidthFocus;
                shadowColor = focusBorder;
              }
            }

            rows.push(
              [
                variant,
                status,
                SIZES[size],
                disabled ? '1' : '0',
                stateName,
                fg,
                bg,
                border4.join(','),
                width4.join(','),
                `${shadowW};${shadowColor}`,
                round(heights[size]),
                round(paddingBlock(heights[size], fonts[size], ratios[size], line, size === 2)),
                round(paddings[size] - line),
                round(variant === 'underlined' ? 0 : radii[size]),
                round(fonts[size]),
                round(lines[size]),
              ].join('|'),
            );
          }
        }
      }
    }
  }
  return rows.join('\n') + '\n';
}

function options(token) {
  const selectedBg = token.controlItemBgActive;
  const selectedActiveBg = token.controlItemBgActiveHover;
  const activeBg = token.controlItemBgHover;
  const rows = [];
  for (const selected of [false, true]) {
    for (const active of [false, true]) {
      for (const disabled of [false, true]) {
        let bg = 'transparent';
        let fg = token.colorText;
        let weight = 400;
        if (disabled) {
          bg = selected ? token.colorBgContainerDisabled : 'transparent';
          fg = token.colorTextDisabled;
        } else if (selected) {
          bg = active ? selectedActiveBg : selectedBg;
          weight = 600;
        } else if (active) {
          bg = activeBg;
        }
        rows.push(
          [
            selected ? '1' : '0',
            active ? '1' : '0',
            disabled ? '1' : '0',
            bg,
            fg,
            weight,
            token.controlHeight,
            (token.controlHeight - token.fontSize * token.lineHeight) / 2,
            token.controlPaddingHorizontal,
            token.borderRadiusSM,
            token.fontSize,
            token.fontHeight,
          ].join('|'),
        );
      }
    }
  }
  return rows.join('\n') + '\n';
}

function chips(token) {
  const line = token.lineWidth;
  const heights = [token.controlHeightSM, token.controlHeight, token.controlHeightLG];
  const fonts = [token.fontSize, token.fontSize, token.fontSizeLG];
  const radii = [token.borderRadiusSM, token.borderRadius, token.borderRadiusLG];
  const rows = [];
  for (const variant of VARIANTS) {
    for (const status of STATUSES) {
      for (let size = 0; size < 3; size += 1) {
        for (const disabled of [false, true]) {
          const fg =
            status === 'error'
              ? token.colorError
              : status === 'warning'
                ? token.colorWarning
                : token.colorText;
          const bg = variant === 'filled' ? token.colorBgContainer : token.colorFillSecondary;
          const height = heights[size] - 8;
          rows.push(
            [
              variant,
              status,
              SIZES[size],
              disabled ? '1' : '0',
              bg,
              fg,
              round(height),
              round(height - 2),
              round(radii[size] - 2),
              round(token.paddingXS),
              round(token.paddingXXS),
              round((heights[size] - height - 4 * line) / 2),
              round(token.paddingXXS),
              round(fonts[size]),
              token.colorTextTertiary,
              round(token.fontSizeIcon - 2),
            ].join('|'),
          );
        }
      }
    }
  }
  return rows.join('\n') + '\n';
}

const menu = (token) =>
  [token.colorBgElevated, token.borderRadiusLG, token.paddingXXS].join('|') + '\n';

for (const [name, file] of [
  ['light', 'tokens-light.json'],
  ['dark', 'tokens-dark.json'],
  ['violet', 'tokens-accent-violet.json'],
]) {
  const token = JSON.parse(fs.readFileSync(path.join(dir, file), 'utf8'));
  fs.writeFileSync(path.join(dir, `select_trigger_matrix_${name}.txt`), trigger(token));
  fs.writeFileSync(path.join(dir, `select_option_matrix_${name}.txt`), options(token));
  fs.writeFileSync(path.join(dir, `select_chip_matrix_${name}.txt`), chips(token));
  fs.writeFileSync(path.join(dir, `select_menu_matrix_${name}.txt`), menu(token));
  console.log(`wrote select_*_matrix_${name}.txt`);
}
