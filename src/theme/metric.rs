//! Button 的度量与少量派生色：antdv-next 的 `button/style/token.ts` 在 Rust 侧的对应物。
//!
//! 这里的数字全部来自 `Tokens`，渲染代码（`src/button.rs`）只允许读这个模块，
//! 不允许再出现色值字面量或魔法 px。antd 自己写死的常量（`paddingInlineSM` 的 8、
//! `fontWeight` 的 400、focus outline-offset 的 1）单独命名并注明出处。

use gpui::{BoxShadow, FontWeight, Hsla, black, point, px, rgba, white};

use crate::theme::Tokens;

/// antd `paddingInlineSM: 8 - lineWidth`。这个 8 不跟随 size 令牌，
/// 换成 `sizeXS` 在 compact 算法下会得到 3px 而不是 7px。
const PADDING_INLINE_SM_BASE: f32 = 8.0;
/// antd Button token `fontWeight: 400`（比正文的 500 更轻）。
const FONT_WEIGHT: f32 = 400.0;
/// antd `:focus-visible` 的 `outline-offset: 1`。gpui 没有 outline，
/// 焦点环用同宽的 spread shadow 近似，所以宽度要把这 1px 算进去。
const FOCUS_OUTLINE_OFFSET: f32 = 1.0;
/// gpui 0.2.2 的 shadow shader 只在 `[-3σ, +3σ]` 上取样（`shaders.wgsl` 的 `fs_shadow`），
/// `blur_radius = 0` 时积分区间退化成一个点，alpha 恒为 0，硬边投影画不出来。
/// antd 的 `0 2px 0` / focus ring 都是零模糊，这里用最小模糊代替硬边。
pub const MIN_SHADOW_BLUR: f32 = 0.5;

/// 三档尺寸（small / medium / large）的按钮度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonMetric {
    pub height: [f32; 3],
    pub padding_inline: [f32; 3],
    pub font_size: [f32; 3],
    pub radius: [f32; 3],
    pub border_width: f32,
    pub icon_gap: f32,
    pub font_weight: FontWeight,
    /// `0 ${controlOutlineWidth}px 0` 阴影的纵向偏移
    pub shadow_offset_y: f32,
    /// 焦点环宽度：`lineWidthFocus` + `outlineOffset`
    pub focus_ring_width: f32,
}

impl ButtonMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        let line = tokens.line_width;
        Self {
            height: [
                tokens.control_height_sm,
                tokens.control_height,
                tokens.control_height_lg,
            ],
            padding_inline: [
                PADDING_INLINE_SM_BASE - line,
                tokens.size_ms - line,
                tokens.size_ms - line,
            ],
            font_size: [tokens.font_size, tokens.font_size, tokens.font_size_lg],
            radius: [
                tokens.border_radius_sm,
                tokens.border_radius,
                tokens.border_radius_lg,
            ],
            border_width: line,
            icon_gap: tokens.margin_xs,
            font_weight: FontWeight(FONT_WEIGHT),
            shadow_offset_y: tokens.control_outline_width,
            focus_ring_width: tokens.line_width_focus + FOCUS_OUTLINE_OFFSET,
        }
    }
}

/// 三档尺寸的输入框度量（antdv-next `input/style/token.ts` 的 `initComponentToken`）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputMetric {
    pub height: [f32; 3],
    pub padding_block: [f32; 3],
    pub padding_inline: [f32; 3],
    pub font_size: [f32; 3],
    pub line_height: [f32; 3],
    pub radius: [f32; 3],
    pub line_width: f32,
    /// 聚焦环宽度：`controlOutlineWidth`（2），比按钮的 `lineWidthFocus`（4）窄
    pub outline_width: f32,
}

fn round_to_tenth(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

fn ceil_to_tenth(value: f32) -> f32 {
    (value * 10.0).ceil() / 10.0
}

/// `paddingBlock = round((controlHeight − fontSize × lineHeight) / 2 × 10) / 10 − lineWidth`，
/// 下限 0；大尺寸用 ceil 而不是 round（与 antd 源码一致）。
fn padding_block(
    height: f32,
    font_size: f32,
    line_height: f32,
    line_width: f32,
    ceil: bool,
) -> f32 {
    let raw = (height - font_size * line_height) / 2.0;
    let rounded = if ceil {
        ceil_to_tenth(raw)
    } else {
        round_to_tenth(raw)
    };
    (rounded - line_width).max(0.0)
}

impl InputMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        let line = tokens.line_width;
        let height = [
            tokens.control_height_sm,
            tokens.control_height,
            tokens.control_height_lg,
        ];
        let font_size = [tokens.font_size, tokens.font_size, tokens.font_size_lg];
        // 纵向内边距用的是字号 × 行高倍数，与 antd 公式同源
        let ratio = [
            tokens.line_height,
            tokens.line_height,
            tokens.line_height_lg,
        ];
        Self {
            height,
            padding_block: [
                padding_block(height[0], font_size[0], ratio[0], line, false),
                padding_block(height[1], font_size[1], ratio[1], line, false),
                padding_block(height[2], font_size[2], ratio[2], line, true),
            ],
            padding_inline: [
                tokens.control_padding_horizontal_sm - line,
                tokens.padding_sm - line,
                tokens.control_padding_horizontal - line,
            ],
            font_size,
            line_height: [
                tokens.font_height,
                tokens.font_height,
                tokens.font_height_lg,
            ],
            radius: [
                tokens.border_radius_sm,
                tokens.border_radius,
                tokens.border_radius_lg,
            ],
            line_width: line,
            outline_width: tokens.control_outline_width,
        }
    }
}

/// Select 触发框度量（antdv-next `select/style/select-input.js` 的变量与尺寸档）。
///
/// 与 `InputMetric` 同公式，但大尺寸的水平内边距仍是 `paddingSM - lineWidth`
/// （antd 只给 `-lg` 改高度 / 字号 / 圆角，不改水平内边距）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectMetric {
    pub height: [f32; 3],
    pub padding_block: [f32; 3],
    pub padding_inline: [f32; 3],
    pub font_size: [f32; 3],
    pub line_height: [f32; 3],
    pub radius: [f32; 3],
    pub line_width: f32,
    pub outline_width: f32,
    /// 菜单项最小高度 = `controlHeight`
    pub option_height: f32,
    /// 菜单项纵向内边距 = `(controlHeight - fontSize × lineHeight) / 2`（不减描边）
    pub option_padding_block: f32,
    /// 菜单项横向内边距 = `controlPaddingHorizontal`
    pub option_padding_inline: f32,
    /// 菜单容器内边距 = `paddingXXS`
    pub menu_padding: f32,
    pub menu_radius: f32,
    /// 多选触发器：纵向内边距 = `lineWidth`，左侧 = `paddingXXS - lineWidth`
    pub multi_padding_block: f32,
    /// 多选触发器左侧内边距 = `paddingXXS - lineWidth`
    pub multi_padding_start: f32,
    /// 多选触发器右内边距（与单选 `padding_inline` 一致）
    pub multi_padding_inline: [f32; 3],
    /// 芯片高度 / 行高 / 圆角（antd 多选 selection-item）
    pub chip_height: [f32; 3],
    pub chip_line_height: [f32; 3],
    pub chip_radius: [f32; 3],
    /// 芯片横向内边距：左 `paddingXS`、右 `paddingXXS`
    pub chip_padding_start: f32,
    pub chip_padding_end: f32,
    /// 芯片纵向外边距 / 右外边距
    pub chip_margin_block: [f32; 3],
    pub chip_margin_end: f32,
    /// 芯片移除图标字号 = `fontSizeIcon - 2`
    pub chip_remove_size: f32,
}

impl SelectMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        let line = tokens.line_width;
        let height = [
            tokens.control_height_sm,
            tokens.control_height,
            tokens.control_height_lg,
        ];
        let font_size = [tokens.font_size, tokens.font_size, tokens.font_size_lg];
        let ratio = [
            tokens.line_height,
            tokens.line_height,
            tokens.line_height_lg,
        ];
        let option_font = tokens.font_size;
        let option_ratio = tokens.line_height;
        // 多选芯片高度 = controlHeight - 2×(描边 + 触发器纵向内边距 + 芯片纵向外边距)
        let chip_height = [height[0] - 8.0, height[1] - 8.0, height[2] - 8.0];
        Self {
            height,
            padding_block: [
                padding_block(height[0], font_size[0], ratio[0], line, false),
                padding_block(height[1], font_size[1], ratio[1], line, false),
                padding_block(height[2], font_size[2], ratio[2], line, true),
            ],
            padding_inline: [
                tokens.padding_xs - line,
                tokens.padding_sm - line,
                tokens.padding_sm - line,
            ],
            font_size,
            line_height: [
                tokens.font_height,
                tokens.font_height,
                tokens.font_height_lg,
            ],
            radius: [
                tokens.border_radius_sm,
                tokens.border_radius,
                tokens.border_radius_lg,
            ],
            line_width: line,
            outline_width: tokens.control_outline_width,
            option_height: tokens.control_height,
            option_padding_block: (tokens.control_height - option_font * option_ratio) / 2.0,
            option_padding_inline: tokens.control_padding_horizontal,
            menu_padding: tokens.padding_xxs,
            menu_radius: tokens.border_radius_lg,
            multi_padding_block: line,
            multi_padding_start: tokens.padding_xxs - line,
            multi_padding_inline: [
                tokens.padding_xs - line,
                tokens.padding_sm - line,
                tokens.padding_sm - line,
            ],
            chip_height,
            chip_line_height: [
                chip_height[0] - 2.0,
                chip_height[1] - 2.0,
                chip_height[2] - 2.0,
            ],
            chip_radius: [
                tokens.border_radius_sm - 2.0,
                tokens.border_radius - 2.0,
                tokens.border_radius_lg - 2.0,
            ],
            chip_padding_start: tokens.padding_xs,
            chip_padding_end: tokens.padding_xxs,
            chip_margin_block: [
                (height[0] - chip_height[0] - 2.0 * line - 2.0 * line) / 2.0,
                (height[1] - chip_height[1] - 2.0 * line - 2.0 * line) / 2.0,
                (height[2] - chip_height[2] - 2.0 * line - 2.0 * line) / 2.0,
            ],
            chip_margin_end: tokens.padding_xxs,
            chip_remove_size: tokens.font_size_icon - 2.0,
        }
    }
}

/// Tag 度量（antdv-next `tag/style/index.ts` 的 `prepareToken`）。
///
/// Tag 高度由行高 + 描边决定（`height: auto`），没有纵向内边距和尺寸档。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TagMetric {
    /// `tagPaddingHorizontal - lineWidth`
    pub padding_inline: f32,
    pub font_size: f32,
    /// `lineHeightSM × tagFontSize`
    pub line_height: f32,
    pub radius: f32,
    pub border_width: f32,
    /// 关闭图标左边距：`paddingXXS - lineWidth`
    pub icon_margin: f32,
    /// 关闭图标字号：`fontSizeIcon - 2 × lineWidth`
    pub icon_size: f32,
}

/// antd Tag `tagPaddingHorizontal: 8`，不跟随 size 令牌。
const TAG_PADDING_HORIZONTAL_BASE: f32 = 8.0;

impl TagMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        let line = tokens.line_width;
        Self {
            padding_inline: TAG_PADDING_HORIZONTAL_BASE - line,
            font_size: tokens.font_size_sm,
            line_height: tokens.line_height_sm * tokens.font_size_sm,
            radius: tokens.border_radius_sm,
            border_width: line,
            icon_margin: tokens.padding_xxs - line,
            icon_size: tokens.font_size_icon - line * 2.0,
        }
    }
}

/// antd `tooltipMaxWidth: 250`，不随种子变化。
const TOOLTIP_MAX_WIDTH_BASE: f32 = 250.0;

/// Tooltip 气泡度量（antdv-next `.ant-tooltip-container` 实测）：
/// `padding: calc(paddingSM / 2) paddingXS`，`min-height: controlHeight`，
/// 圆角 `borderRadius`，最大宽 250。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooltipMetric {
    pub padding_block: f32,
    pub padding_inline: f32,
    pub radius: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub font_size: f32,
    pub line_height: f32,
}

impl TooltipMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            padding_block: tokens.padding_sm / 2.0,
            padding_inline: tokens.padding_xs,
            radius: tokens.border_radius,
            min_height: tokens.control_height,
            max_width: TOOLTIP_MAX_WIDTH_BASE,
            font_size: tokens.font_size,
            line_height: tokens.font_height,
        }
    }
}

/// Popover 容器度量（antdv-next `.ant-popover-container` 实测）：
/// `padding: paddingSM`、圆角 `borderRadiusLG`、无描边。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopoverMetric {
    pub padding: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
}

impl PopoverMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            padding: tokens.padding_sm,
            radius: tokens.border_radius_lg,
            font_size: tokens.font_size,
            line_height: tokens.font_height,
        }
    }
}

/// antd Pagination 度量：页码项 32（small 24），项间距 8（small 4），
/// 文字左右 `marginXXS × 1.5`（实测 6px），圆角 `borderRadius`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaginationMetric {
    pub size: [f32; 2],
    pub spacing: [f32; 2],
    pub radius: f32,
    pub line_width: f32,
    pub font_size: f32,
    pub padding_inline: f32,
    pub font_weight: FontWeight,
    pub active_font_weight: FontWeight,
}

impl PaginationMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            size: [tokens.control_height, tokens.control_height_sm],
            spacing: [tokens.margin_xs, tokens.padding_xxs],
            radius: tokens.border_radius,
            line_width: tokens.line_width,
            font_size: tokens.font_size,
            padding_inline: tokens.margin_xxs * 1.5,
            font_weight: FontWeight(FONT_WEIGHT),
            active_font_weight: FontWeight(tokens.font_weight_strong),
        }
    }
}

/// antd Switch 组件级常量（`switch/style/index.ts`）：轨道 22/16，最小宽 44/28，
/// 滑块 18/12，内边距 2，文字侧内边距 9/6，禁用/加载整体透明度 0.65。
const SWITCH_TRACK_HEIGHT: f32 = 22.0;
const SWITCH_TRACK_HEIGHT_SM: f32 = 16.0;
const SWITCH_TRACK_MIN_WIDTH: f32 = 44.0;
const SWITCH_TRACK_MIN_WIDTH_SM: f32 = 28.0;
const SWITCH_HANDLE_SIZE: f32 = 18.0;
const SWITCH_HANDLE_SIZE_SM: f32 = 12.0;
const SWITCH_TRACK_PADDING: f32 = 2.0;
const SWITCH_RADIUS: f32 = 100.0;
const SWITCH_DISABLED_OPACITY: f32 = 0.65;
const SWITCH_TEXT_PADDING_NEAR: f32 = 9.0;
const SWITCH_TEXT_PADDING_NEAR_SM: f32 = 6.0;

/// Switch 的度量（`harness/_switch_probe.html` computed style 核对）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchMetric {
    pub track_height: [f32; 2],
    pub track_min_width: [f32; 2],
    pub handle_size: [f32; 2],
    pub track_padding: f32,
    pub radius: f32,
    pub disabled_opacity: f32,
    pub text_padding_near: [f32; 2],
    pub font_size: f32,
}

impl SwitchMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            track_height: [SWITCH_TRACK_HEIGHT, SWITCH_TRACK_HEIGHT_SM],
            track_min_width: [SWITCH_TRACK_MIN_WIDTH, SWITCH_TRACK_MIN_WIDTH_SM],
            handle_size: [SWITCH_HANDLE_SIZE, SWITCH_HANDLE_SIZE_SM],
            track_padding: SWITCH_TRACK_PADDING,
            radius: SWITCH_RADIUS,
            disabled_opacity: SWITCH_DISABLED_OPACITY,
            text_padding_near: [SWITCH_TEXT_PADDING_NEAR, SWITCH_TEXT_PADDING_NEAR_SM],
            font_size: tokens.font_size_sm,
        }
    }
}

/// antd Checkbox 组件常量（`checkbox/style/index.ts`）：方框 16、半选方块 8、
/// 勾选字形 11（参照图 5.7×9.1 的 2px 折线用同字号字符近似）。
const CHECKBOX_SIZE: f32 = 16.0;
const CHECKBOX_MARK_BLOCK: f32 = 8.0;
const CHECKBOX_MARK_GLYPH: f32 = 11.0;

/// Checkbox 的度量（`harness/_checkbox_probe.html` computed style 核对）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckboxMetric {
    pub size: f32,
    pub radius: f32,
    pub line_width: f32,
    pub mark_block: f32,
    pub mark_glyph: f32,
    pub label_gap: f32,
    pub font_size: f32,
    pub line_height: f32,
}

impl CheckboxMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            size: CHECKBOX_SIZE,
            radius: tokens.border_radius_sm,
            line_width: tokens.line_width,
            mark_block: CHECKBOX_MARK_BLOCK,
            mark_glyph: CHECKBOX_MARK_GLYPH,
            label_gap: tokens.padding_xs,
            font_size: tokens.font_size,
            line_height: tokens.font_height,
        }
    }
}

/// antd Radio 组件常量：圆 16、内点 6（computed `::after` 实测）。
const RADIO_SIZE: f32 = 16.0;
const RADIO_DOT: f32 = 6.0;

/// Radio 的度量（`harness/_radio_probe.html` computed style 核对）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadioMetric {
    pub size: f32,
    pub dot: f32,
    pub radius: f32,
    pub line_width: f32,
    pub label_gap: f32,
    pub font_size: f32,
    pub line_height: f32,
}

impl RadioMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            size: RADIO_SIZE,
            dot: RADIO_DOT,
            radius: RADIO_SIZE / 2.0,
            line_width: tokens.line_width,
            label_gap: tokens.padding_xs,
            font_size: tokens.font_size,
            line_height: tokens.font_height,
        }
    }
}

/// antd Breadcrumb 组件常量（`harness/_breadcrumb_probe.html` computed style 核对）：
/// 分隔符两侧各 8px、字号 14、行高 `fontHeight` 22。
const BREADCRUMB_GAP: f32 = 8.0;

/// Breadcrumb 的度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreadcrumbMetric {
    pub font_size: f32,
    pub line_height: f32,
    pub gap: f32,
}

impl BreadcrumbMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            font_size: tokens.font_size,
            line_height: tokens.font_height,
            gap: BREADCRUMB_GAP,
        }
    }
}

/// antd Card 组件常量（源码 `card/style/index.js` prepareComponentToken）：
/// 圆角 `borderRadiusLG` 8；头高 `fontHeightLG + padding*2` = 56，左右内边距 24；
/// 头字号 `fontSizeLG` 16、字重 strong；body 内边距 `paddingLG` 24；
/// actions 项上下 12、字号 14。
const CARD_ACTIONS_PADDING: f32 = 12.0;

/// Card 的度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardMetric {
    pub radius: f32,
    pub header_height: f32,
    pub header_padding: f32,
    pub header_font_size: f32,
    pub body_padding: f32,
    pub actions_padding: f32,
    pub actions_font_size: f32,
    pub font_size: f32,
    pub line_height: f32,
}

impl CardMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            radius: tokens.border_radius_lg,
            header_height: tokens.font_height_lg + tokens.padding * 2.0,
            header_padding: tokens.padding_lg,
            header_font_size: tokens.font_size_lg,
            body_padding: tokens.padding_lg,
            actions_padding: CARD_ACTIONS_PADDING,
            actions_font_size: tokens.font_size,
            font_size: tokens.font_size,
            line_height: tokens.font_height,
        }
    }
}

/// antd Steps 组件常量（源码 `steps/style/index.js` prepareComponentToken）：
/// 图标 `controlHeight` 32（small `fontSizeHeading3` 24）、字号 `fontSize` 14；
/// 标题 `fontSizeLG` 16 / `fontHeightLG` 24（small 14 / 22）；
/// 水平项间距与节点后线间距都是 `margin` 16（small `marginXS` 12）；
/// 图标与文字列间距水平 8、垂直 16；垂直项下内边距 `paddingSM` 12；
/// 点状节点 `controlHeight/4` 8、当前 10，线粗 `lineWidthBold` 2。
const STEPS_ICON_SIZE_SMALL: f32 = 24.0;
const STEPS_ITEM_GAP: f32 = 16.0;
const STEPS_ITEM_GAP_SMALL: f32 = 12.0;
const STEPS_RAIL_GAP: f32 = 16.0;
const STEPS_RAIL_GAP_SMALL: f32 = 12.0;
const STEPS_ICON_GAP: f32 = 8.0;
const STEPS_ICON_GAP_VERTICAL: f32 = 16.0;
const STEPS_VERTICAL_PADDING: f32 = 12.0;
const STEPS_VERTICAL_RAIL_MARGIN: f32 = 6.0;
const STEPS_DOT_WRAPPER_GAP: f32 = 12.0;
const STEPS_DOT_RAIL_GAP: f32 = 8.0;
const STEPS_TITLE_FONT_SMALL: f32 = 14.0;
const STEPS_TITLE_LINE_SMALL: f32 = 22.0;

/// Steps 的度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepsMetric {
    pub icon_size: f32,
    pub icon_size_small: f32,
    pub icon_font_size: f32,
    pub title_font_size: f32,
    pub title_line_height: f32,
    pub title_font_size_small: f32,
    pub title_line_height_small: f32,
    pub content_font_size: f32,
    pub content_line_height: f32,
    pub item_gap: f32,
    pub item_gap_small: f32,
    pub rail_gap: f32,
    pub rail_gap_small: f32,
    pub icon_gap: f32,
    pub icon_gap_vertical: f32,
    pub heading_height: f32,
    pub heading_height_small: f32,
    pub vertical_padding: f32,
    pub vertical_rail_margin: f32,
    pub dot_size: f32,
    pub dot_active_size: f32,
    pub dot_wrapper_gap: f32,
    pub dot_rail_gap: f32,
    pub rail_width: f32,
    pub dot_rail_width: f32,
}

impl StepsMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            icon_size: tokens.control_height,
            icon_size_small: STEPS_ICON_SIZE_SMALL,
            icon_font_size: tokens.font_size,
            title_font_size: tokens.font_size_lg,
            title_line_height: tokens.font_height_lg,
            title_font_size_small: STEPS_TITLE_FONT_SMALL,
            title_line_height_small: STEPS_TITLE_LINE_SMALL,
            content_font_size: tokens.font_size,
            content_line_height: tokens.font_height,
            item_gap: STEPS_ITEM_GAP,
            item_gap_small: STEPS_ITEM_GAP_SMALL,
            rail_gap: STEPS_RAIL_GAP,
            rail_gap_small: STEPS_RAIL_GAP_SMALL,
            icon_gap: STEPS_ICON_GAP,
            icon_gap_vertical: STEPS_ICON_GAP_VERTICAL,
            heading_height: tokens.control_height.max(tokens.font_height_lg),
            heading_height_small: STEPS_ICON_SIZE_SMALL.max(STEPS_TITLE_LINE_SMALL),
            vertical_padding: STEPS_VERTICAL_PADDING,
            vertical_rail_margin: STEPS_VERTICAL_RAIL_MARGIN,
            dot_size: tokens.control_height / 4.0,
            dot_active_size: tokens.control_height_lg / 4.0,
            dot_wrapper_gap: STEPS_DOT_WRAPPER_GAP,
            dot_rail_gap: STEPS_DOT_RAIL_GAP,
            rail_width: tokens.line_width,
            dot_rail_width: 2.0,
        }
    }
}

/// antd Timeline 组件常量（源码 `timeline/style/index.js` + `style/horizontal.js`）：
/// 节点外径 10（内径 8）、描边 `lineWidthBold` 2；节点距项顶 7、节点与内容间距 16；
/// 有标题时切左右两栏：间距 `margin*2 + 8` 40，项下内边距 `padding*1.25` 20；
/// 无标题时左轨：项最小高 48、项下内边距 `paddingSM` 12；
/// 连接线 `colorSplit`、粗 `lineWidthBold`，从节点下方 17 起、越过项底 7；
/// 标题与内容 `fontSize/fontHeight` 14/22，辅助文字 `fontSizeSM/fontHeightSM` 12/20。
const TIMELINE_DOT_SIZE: f32 = 10.0;
const TIMELINE_DOT_INNER: f32 = 8.0;
const TIMELINE_DOT_BORDER: f32 = 2.0;
const TIMELINE_DOT_TOP: f32 = 7.0;
const TIMELINE_ICON_GAP: f32 = 16.0;
const TIMELINE_CONTENT_GAP: f32 = 40.0;
const TIMELINE_ITEM_MIN_HEIGHT: f32 = 48.0;
const TIMELINE_PADDING_PLAIN: f32 = 12.0;
const TIMELINE_PADDING_ALTERNATE: f32 = 20.0;
const TIMELINE_RAIL_TOP: f32 = 17.0;
const TIMELINE_RAIL_BOTTOM: f32 = 7.0;
const TIMELINE_HEADER_MIN_HEIGHT: f32 = 24.0;
const TIMELINE_META_FONT_SIZE: f32 = 12.0;
const TIMELINE_META_LINE_HEIGHT: f32 = 20.0;
const TIMELINE_CONTENT_TOP: f32 = 1.0;

/// Timeline 的度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimelineMetric {
    pub dot_size: f32,
    pub dot_inner: f32,
    pub dot_border: f32,
    pub dot_top: f32,
    pub icon_gap: f32,
    pub content_gap: f32,
    pub item_min_height: f32,
    pub padding_plain: f32,
    pub padding_alternate: f32,
    pub rail_top: f32,
    pub rail_bottom: f32,
    pub rail_width: f32,
    pub header_min_height: f32,
    pub title_size: f32,
    pub title_line: f32,
    pub meta_size: f32,
    pub meta_line: f32,
    pub content_top: f32,
}

impl TimelineMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            dot_size: TIMELINE_DOT_SIZE,
            dot_inner: TIMELINE_DOT_INNER,
            dot_border: TIMELINE_DOT_BORDER,
            dot_top: TIMELINE_DOT_TOP,
            icon_gap: TIMELINE_ICON_GAP,
            content_gap: TIMELINE_CONTENT_GAP,
            item_min_height: TIMELINE_ITEM_MIN_HEIGHT,
            padding_plain: TIMELINE_PADDING_PLAIN,
            padding_alternate: TIMELINE_PADDING_ALTERNATE,
            rail_top: TIMELINE_RAIL_TOP,
            rail_bottom: TIMELINE_RAIL_BOTTOM,
            rail_width: tokens.line_width_bold,
            header_min_height: TIMELINE_HEADER_MIN_HEIGHT,
            title_size: tokens.font_size,
            title_line: tokens.font_height,
            meta_size: TIMELINE_META_FONT_SIZE,
            meta_line: TIMELINE_META_LINE_HEIGHT,
            content_top: TIMELINE_CONTENT_TOP,
        }
    }
}

/// antd Segmented 组件常量（源码 `segmented/style/index.js` prepareComponentToken）：
/// 轨道内边距 `lineWidthBold` 2；容器圆角 `borderRadius` 6（sm 4 / lg 8）；
/// 选项高 `controlHeight - 2*trackPadding` 28（sm 20 / lg 36），圆角 `borderRadiusSM` 4；
/// 选项水平内边距 `controlPaddingHorizontal - lineWidth` 11（sm 7）；
/// 字号默认 14、large 16；轨道底 `colorBgLayout`，选中底 `colorBgElevated` + 阴影。

/// Segmented 的度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentedMetric {
    pub track_padding: f32,
    pub container_radius: [f32; 3],
    pub item_radius: [f32; 3],
    pub label_height: [f32; 3],
    pub padding_inline: [f32; 3],
    pub font_size: [f32; 3],
}

impl SegmentedMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        let track_padding = tokens.line_width_bold;
        Self {
            track_padding,
            container_radius: [
                tokens.border_radius_sm,
                tokens.border_radius,
                tokens.border_radius_lg,
            ],
            item_radius: [
                tokens.border_radius_xs,
                tokens.border_radius_sm,
                tokens.border_radius,
            ],
            label_height: [
                tokens.control_height_sm - track_padding * 2.0,
                tokens.control_height - track_padding * 2.0,
                tokens.control_height_lg - track_padding * 2.0,
            ],
            padding_inline: [
                tokens.control_padding_horizontal_sm - tokens.line_width,
                tokens.control_padding_horizontal - tokens.line_width,
                tokens.control_padding_horizontal - tokens.line_width,
            ],
            font_size: [tokens.font_size, tokens.font_size, tokens.font_size_lg],
        }
    }
}

/// Tabs 的度量（antdv-next `tabs/style/index.js` prepareComponentToken + genSizeStyle）。
///
/// 下划线风格：项内边距块向 `paddingXS / paddingSM / padding`（8/12/16），
/// 字号 14/14/16、行高 22/22/24；项间距 32；导航底部外边距 16；
/// 指示条高 `lineWidthBold`、分隔线 `lineWidth`。
/// 卡片风格：卡片高 32/40/48、内边距块 4/8/11、内边距行 8/16/16、
/// 圆角 6/8/8、卡片间隙 `marginXXS / 2`（2）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsMetric {
    pub item_gap: f32,
    pub nav_margin_bottom: f32,
    pub padding_block: [f32; 3],
    pub font_size: [f32; 3],
    pub line_height: [f32; 3],
    pub ink_height: f32,
    pub divider_width: f32,
    pub card_height: [f32; 3],
    pub card_padding_block: [f32; 3],
    pub card_padding_inline: [f32; 3],
    pub card_radius: [f32; 3],
    pub card_gutter: f32,
    pub close_gap: f32,
    pub close_size: f32,
}

impl TabsMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        let line = tokens.line_width;
        let font_size = [tokens.font_size, tokens.font_size, tokens.font_size_lg];
        // antd 在 prepareComponentToken 中把水平项间距写成常量 32
        let item_gap = 32.0;
        let card_height = [
            tokens.control_height,
            tokens.control_height_lg,
            tokens.control_height_lg + 8.0,
        ];
        let font_height = [
            tokens.font_height,
            tokens.font_height,
            tokens.font_height_lg,
        ];
        Self {
            item_gap,
            nav_margin_bottom: tokens.margin,
            padding_block: [tokens.padding_xs, tokens.padding_sm, tokens.padding],
            font_size,
            line_height: font_height,
            ink_height: tokens.line_width_bold,
            divider_width: line,
            card_height,
            card_padding_block: [
                (card_height[0] - font_height[0]) / 2.0 - line,
                (card_height[1] - font_height[1]) / 2.0 - line,
                (card_height[2] - font_height[2]) / 2.0 - line,
            ],
            card_padding_inline: [tokens.padding_xs, tokens.padding, tokens.padding],
            card_radius: [
                tokens.border_radius,
                tokens.border_radius_lg,
                tokens.border_radius_lg,
            ],
            card_gutter: tokens.margin_xxs / 2.0,
            close_gap: tokens.margin_xs,
            close_size: tokens.font_size_sm,
        }
    }
}

/// antd Alert 组件常量（源码 `alert/style/index.js` prepareComponentToken）：
/// 圆角 `borderRadiusLG` 8；默认内边距 8×12；带描述 20×24；
/// 图标与标题间距默认 8、带描述 12；图标尺寸默认 14、带描述 `fontSizeHeading3` 24；
/// 带描述标题 `fontSizeLG` 16 且下边距 8；关闭图标 `fontSizeIcon` 12。
const ALERT_RADIUS: f32 = 8.0;
const ALERT_PADDING: [(f32, f32); 2] = [(8.0, 12.0), (20.0, 24.0)];
const ALERT_ICON_GAP: [f32; 2] = [8.0, 12.0];
const ALERT_ICON_SIZE: [f32; 2] = [14.0, 24.0];
const ALERT_TITLE_MARGIN: f32 = 8.0;

/// Alert 的度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlertMetric {
    pub radius: f32,
    pub padding: [(f32, f32); 2],
    pub icon_gap: [f32; 2],
    pub icon_size: [f32; 2],
    pub title_size: f32,
    pub description_title_size: f32,
    pub title_margin: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub close_size: f32,
}

impl AlertMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            radius: ALERT_RADIUS,
            padding: ALERT_PADDING,
            icon_gap: ALERT_ICON_GAP,
            icon_size: ALERT_ICON_SIZE,
            title_size: tokens.font_size,
            description_title_size: tokens.font_size_lg,
            title_margin: ALERT_TITLE_MARGIN,
            font_size: tokens.font_size,
            line_height: tokens.font_height,
            close_size: tokens.font_size_icon,
        }
    }
}

/// antd Skeleton 组件常量（源码 `skeleton/style/index.js` prepareComponentToken）：
/// 标题 / 段落条高 = `controlHeight / 2` 16、圆角 `borderRadiusSM` 4；
/// 行间距 `controlHeightXS` 16；末行宽度 61%（三行及以上才收窄）。
const SKELETON_LAST_LINE_RATIO: f32 = 0.61;

/// Skeleton 的度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkeletonMetric {
    pub height: f32,
    pub radius: f32,
    pub gap: f32,
    pub last_line_ratio: f32,
}

impl SkeletonMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            height: tokens.control_height / 2.0,
            radius: tokens.border_radius_sm,
            gap: tokens.control_height_xs,
            last_line_ratio: SKELETON_LAST_LINE_RATIO,
        }
    }
}

/// antd Badge 组件常量（`harness/_badge_probe.html` computed style 核对）：
/// 计数高 20、最小宽 20、圆角 10、字号 12；圆点 6×6；
/// 两者都用 `translate(50%,-50%)` 压在右上角，外圈 1px 容器色描边。
const BADGE_HEIGHT: f32 = 20.0;
const BADGE_MIN_WIDTH: f32 = 20.0;
const BADGE_RADIUS: f32 = 10.0;
const BADGE_DOT: f32 = 6.0;
const BADGE_RING: f32 = 1.0;
/// 多字符计数的水平内边距（antd `padding: 0 8px`）与 12px 数字的近似字宽。
const BADGE_TEXT_PADDING: f32 = 16.0;
const BADGE_CHAR_WIDTH: f32 = 7.0;

/// Badge 的度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadgeMetric {
    pub height: f32,
    pub min_width: f32,
    pub radius: f32,
    pub dot: f32,
    pub font_size: f32,
    pub ring: f32,
}

impl BadgeMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            height: BADGE_HEIGHT,
            min_width: BADGE_MIN_WIDTH,
            radius: BADGE_RADIUS,
            dot: BADGE_DOT,
            font_size: tokens.font_size_sm,
            ring: BADGE_RING,
        }
    }

    /// 计数角标宽度：单字符固定 20；多字符按 `8px` 内边距 + 12px 数字近似字宽。
    pub fn count_width(&self, chars: usize) -> f32 {
        if chars <= 1 {
            self.min_width
        } else {
            self.min_width
                .max(BADGE_TEXT_PADDING + chars as f32 * BADGE_CHAR_WIDTH)
        }
    }
}

/// Badge 外圈：antd `box-shadow: 0 0 0 1px colorBgContainer`。
pub fn badge_ring_shadow(color: Hsla) -> BoxShadow {
    BoxShadow {
        color,
        offset: point(px(0.0), px(0.0)),
        blur_radius: px(0.0),
        spread_radius: px(BADGE_RING),
        inset: false,
    }
}

/// antd Avatar 组件常量（`harness/_avatar_probe.html` computed style 核对）：
/// 尺寸 24 / 32 / 40，方形圆角 4 / 6 / 8；文字始终 14px、行高 `fontHeight`。
const AVATAR_SIZE_SM: f32 = 24.0;
const AVATAR_SIZE: f32 = 32.0;
const AVATAR_SIZE_LG: f32 = 40.0;

/// Avatar 的度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarMetric {
    pub size: [f32; 3],
    pub radius: [f32; 3],
    pub font_size: f32,
    pub line_height: f32,
}

impl AvatarMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            size: [AVATAR_SIZE_SM, AVATAR_SIZE, AVATAR_SIZE_LG],
            radius: [
                tokens.border_radius_sm,
                tokens.border_radius,
                tokens.border_radius_lg,
            ],
            font_size: tokens.font_size,
            line_height: tokens.font_height,
        }
    }
}

/// antd Progress 组件常量（`harness/_progress_probe.html` computed style 核对）：
/// 默认轨高 8、small 6，圆角 100，指示文字与轨道间距 8；
/// 分段项默认宽 14、small 宽 2（antdv-next 实测固定 px），段间距 2。
const PROGRESS_STROKE: f32 = 8.0;
const PROGRESS_STROKE_SM: f32 = 6.0;
const PROGRESS_RADIUS: f32 = 100.0;
const PROGRESS_GAP: f32 = 8.0;
const PROGRESS_STEPS_ITEM: f32 = 14.0;
const PROGRESS_STEPS_ITEM_SM: f32 = 2.0;
const PROGRESS_STEPS_GAP: f32 = 2.0;

/// Progress 的度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressMetric {
    pub stroke: [f32; 2],
    pub radius: f32,
    pub gap: f32,
    pub font_size: [f32; 2],
    pub steps_item: [f32; 2],
    pub steps_gap: f32,
}

impl ProgressMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            stroke: [PROGRESS_STROKE, PROGRESS_STROKE_SM],
            radius: PROGRESS_RADIUS,
            gap: PROGRESS_GAP,
            font_size: [tokens.font_size, tokens.font_size_sm],
            steps_item: [PROGRESS_STEPS_ITEM, PROGRESS_STEPS_ITEM_SM],
            steps_gap: PROGRESS_STEPS_GAP,
        }
    }
}

/// Switch 滑块投影：antd `0 2px 4px 0 rgba(0,35,11,0.2)`。
/// `factor` 用于禁用 / 加载态把投影一并压淡；小尺寸滑块的模糊半径收窄，
/// 补偿 gpui 阴影采样比 Chromium 更大的扩散（实测 12px 滑块下方过暗）。
pub fn switch_handle_shadow(factor: f32, small: bool) -> Vec<BoxShadow> {
    let mut color: Hsla = rgba(0x00230b33).into();
    color.a *= factor;
    vec![BoxShadow {
        color,
        offset: point(px(0.0), px(2.0)),
        blur_radius: px(if small { 1.5 } else { 4.0 }),
        spread_radius: px(0.0),
        inset: false,
    }]
}

/// 输入框聚焦环：`0 0 0 controlOutlineWidth <outline>`，零偏移、零模糊。
pub fn input_focus_shadow(color: Hsla, width: f32) -> BoxShadow {
    BoxShadow {
        color,
        offset: point(px(0.0), px(0.0)),
        blur_radius: px(MIN_SHADOW_BLUR),
        spread_radius: px(width),
        inset: false,
    }
}

/// `0 ${controlOutlineWidth}px 0 <color>`：antd 按钮静止态的那条投影。
pub fn button_shadow(color: Hsla, offset_y: f32) -> BoxShadow {
    BoxShadow {
        color,
        offset: point(px(0.0), px(offset_y)),
        blur_radius: px(MIN_SHADOW_BLUR),
        spread_radius: px(0.0),
        inset: false,
    }
}

/// 焦点环：`outline: lineWidthFocus solid colorPrimaryBorder` + `outline-offset: 1`，
/// gpui 用 spread shadow 表达，因此扩散半径是两者之和。
pub fn focus_ring(color: Hsla, width: f32) -> Vec<BoxShadow> {
    vec![BoxShadow {
        color,
        offset: point(px(0.0), px(0.0)),
        blur_radius: px(MIN_SHADOW_BLUR),
        spread_radius: px(width),
        inset: false,
    }]
}

/// antd `isLightBg`：透明度 ≤ 0.5 时先压到白底上比 HSV 明度，否则比 RGB 亮度。
/// 阈值（0.5 / 192）与 antd 源码一致。
fn is_light_bg(color: Hsla) -> bool {
    let [r, g, b] = hsl_to_rgb(color);
    if color.a <= 0.5 {
        let over_white = |c: f32| c * color.a + 1.0 * (1.0 - color.a);
        let max = over_white(r).max(over_white(g)).max(over_white(b));
        max > 0.5
    } else {
        // fast-color getLuminance 的权重（0..255 通道），阈值 192
        (r * 0.299 + g * 0.587 + b * 0.114) * 255.0 > 192.0
    }
}

/// gpui 的 Hsla（通道归一化）转 RGB（0..1）。
fn hsl_to_rgb(color: Hsla) -> [f32; 3] {
    let (h, s, l) = (color.h, color.s, color.l);
    if s == 0. {
        return [l, l, l];
    }
    let q = if l < 0.5 { l * (1. + s) } else { l + s - l * s };
    let p = 2. * l - q;
    let channel = |mut t: f32| {
        if t < 0. {
            t += 1.
        } else if t > 1. {
            t -= 1.
        }
        if t < 1. / 6. {
            p + (q - p) * 6. * t
        } else if t < 1. / 2. {
            q
        } else if t < 2. / 3. {
            p + (q - p) * (2. / 3. - t) * 6.
        } else {
            p
        }
    };
    let h = ((h % 1.0) + 1.0) % 1.0;
    [channel(h + 1. / 3.), channel(h), channel(h - 1. / 3.)]
}

/// antd Button token `solidTextColor`：实心按钮文字色由 `colorBgSolid` 的亮度决定，
/// 亮底配黑字、暗底配白字（源码 `isLightBg(colorBgSolid) ? '#000' : '#fff'`）。
pub fn solid_text_color(bg_solid: Hsla) -> Hsla {
    if is_light_bg(bg_solid) {
        black()
    } else {
        white()
    }
}

/// antd Slider 组件常量（源码 `slider/style/index.js` prepareComponentToken）：
/// 容器高 `controlHeightLG / 4` 10、轨道 4、滑块同容器高、悬停滑块 `controlHeightSM / 2` 12、
/// 刻度点 8、滑块环 `lineWidth + 1` 2（悬停 2.5）、悬停外圈 6；
/// 轨道内边距 `railSize` 4，容器外边距块向 `(controlHeight - controlSize) / 2` 11、
/// 行向 `controlSize / 2` 5；带刻度额外下边距 `controlHeightLG - controlSize` 30；
/// 刻度文字 `fontSize` 14，顶距 `railSize * 3` 12。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderMetric {
    pub control_size: f32,
    pub rail_size: f32,
    pub handle_size: f32,
    pub handle_size_hover: f32,
    pub dot_size: f32,
    pub ring_width: f32,
    pub ring_width_hover: f32,
    pub outline_width: f32,
    pub handle_pos: f32,
    pub dot_offset: f32,
    pub mark_top: f32,
    pub mark_font: f32,
    pub margin_part: f32,
    pub margin_full: f32,
    pub margin_part_with_mark: f32,
}

impl SliderMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        let control_size = tokens.control_height_lg / 4.0;
        let rail_size = 4.0;
        let handle_size = control_size;
        let ring_width = tokens.line_width + 1.0;
        let dot_size = 8.0;
        Self {
            control_size,
            rail_size,
            handle_size,
            handle_size_hover: tokens.control_height_sm / 2.0,
            dot_size,
            ring_width,
            ring_width_hover: tokens.line_width + 1.5,
            outline_width: 6.0,
            handle_pos: (rail_size * 3.0 - handle_size) / 2.0,
            dot_offset: (rail_size - dot_size) / 2.0,
            mark_top: rail_size * 3.0,
            mark_font: tokens.font_size,
            margin_part: (tokens.control_height - control_size) / 2.0,
            margin_full: control_size / 2.0,
            margin_part_with_mark: tokens.control_height_lg - control_size,
        }
    }
}

/// antd Statistic 组件常量（源码 `statistic/style/index.js` prepareComponentToken）：
/// 标题字号 `fontSize` 14、颜色 `colorTextDescription`，标题下距 `marginXXS` 4；
/// 内容字号 `fontSizeHeading3` 24、颜色 `colorTextHeading`；前缀/后缀间距 `marginXXS`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatisticMetric {
    pub title_size: f32,
    pub content_size: f32,
    pub header_gap: f32,
    pub affix_gap: f32,
    pub trend_size: f32,
    pub trend_gap: f32,
}

impl StatisticMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            title_size: tokens.font_size,
            content_size: tokens.font_size_heading3,
            header_gap: tokens.margin_xxs,
            affix_gap: tokens.margin_xxs,
            trend_size: tokens.font_size_sm,
            trend_gap: tokens.margin_xxs,
        }
    }
}

/// antd Typography 常量（源码 `typography/style/index.js` + `mixins.js`）：
/// 标题字号 38/30/24/20/16、行高 46/38/32/28/24（lineHeightHeading 1.21–1.5），
/// 字重 `fontWeightStrong` 600，标题下距 `fontSizeSM * lineHeightSM` 20；
/// 正文 14/22；行内代码 85% 字号、内边距 0.4em / 0.2em×0.1em、圆角 3、
/// 底 `rgba(150,150,150,.1)`、边 `rgba(100,100,100,.2)`；mark 底 gold2 `#ffe58f`。
pub const TYPOGRAPHY_TITLE_WEIGHT: f32 = 600.0;
pub const TYPOGRAPHY_STRONG_WEIGHT: f32 = 600.0;
pub const TYPOGRAPHY_CODE_SIZE_RATIO: f32 = 0.85;
pub const TYPOGRAPHY_CODE_RADIUS: f32 = 3.0;
pub const TYPOGRAPHY_CODE_BG: u32 = 0x9696961a;
pub const TYPOGRAPHY_CODE_BORDER: u32 = 0x64646433;
pub const TYPOGRAPHY_MARK_BG: u32 = 0xffe58fff;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypographyMetric {
    pub title_size: [f32; 5],
    pub title_line_height: [f32; 5],
    pub title_margin_bottom: f32,
    pub body_size: f32,
    pub body_line_height: f32,
}

impl TypographyMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            title_size: [
                tokens.font_size_heading1,
                tokens.font_size_heading2,
                tokens.font_size_heading3,
                tokens.font_size_heading4,
                tokens.font_size_heading5,
            ],
            // antd 的行高是相对字号的比例，这里取整成像素
            title_line_height: [
                (tokens.font_size_heading1 * 1.2105263).round(),
                (tokens.font_size_heading2 * 1.2666667).round(),
                (tokens.font_size_heading3 * 1.3333334).round(),
                (tokens.font_size_heading4 * 1.4).round(),
                (tokens.font_size_heading5 * 1.5).round(),
            ],
            title_margin_bottom: tokens.font_height_sm,
            body_size: tokens.font_size,
            body_line_height: tokens.font_height,
        }
    }
}

/// 描述列表：标题 16/24、标题下距 20；表格项下/右间距 16；带边框单元格 16/24；
/// 冒号左右 2/8（marginXXS/2、marginXS）。
pub const DESCRIPTIONS_TITLE_WEIGHT: f32 = 600.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DescriptionsMetric {
    pub title_size: f32,
    pub title_line_height: f32,
    pub title_margin_bottom: f32,
    pub item_padding_bottom: f32,
    pub item_padding_end: f32,
    pub bordered_padding_block: f32,
    pub bordered_padding_inline: f32,
    pub colon_margin_left: f32,
    pub colon_margin_right: f32,
    pub border_radius: f32,
    pub item_size: f32,
    pub line_height: f32,
}

impl DescriptionsMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            title_size: tokens.font_size_lg,
            title_line_height: tokens.font_height_lg,
            title_margin_bottom: tokens.font_height_sm,
            item_padding_bottom: tokens.padding,
            item_padding_end: tokens.padding,
            bordered_padding_block: tokens.padding,
            bordered_padding_inline: tokens.padding_lg,
            colon_margin_left: tokens.margin_xxs / 2.0,
            colon_margin_right: tokens.margin_xs,
            border_radius: tokens.border_radius_lg,
            item_size: tokens.font_size,
            line_height: tokens.font_height,
        }
    }
}

/// 分隔线：线宽 1、无文字上下 24、带文字上下 16；文字 16/字重 500，
/// plain 时 14/常规；文字两侧内边距 1em=16；左右朝向留白 5%；竖线高 0.9em、左右 8。
pub const DIVIDER_TEXT_WEIGHT: f32 = 500.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DividerMetric {
    pub width: f32,
    pub margin: f32,
    pub margin_with_text: f32,
    pub text_size: f32,
    pub plain_text_size: f32,
    pub text_padding_inline: f32,
    pub plain_text_padding_inline: f32,
    pub orientation_margin: f32,
    pub vertical_margin_inline: f32,
    pub vertical_height: f32,
}

impl DividerMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            width: tokens.line_width,
            margin: tokens.margin_lg,
            margin_with_text: tokens.margin,
            text_size: tokens.font_size_lg,
            plain_text_size: tokens.font_size,
            text_padding_inline: tokens.font_size_lg,
            plain_text_padding_inline: tokens.font_size,
            orientation_margin: 0.05,
            vertical_margin_inline: tokens.margin_xs,
            vertical_height: tokens.font_size * 0.9,
        }
    }
}

/// 间距容器：small=marginXS/paddingXS 8、middle=padding 16、large=paddingLG 24。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpaceMetric {
    pub small: f32,
    pub middle: f32,
    pub large: f32,
}

impl SpaceMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            small: tokens.padding_xs,
            middle: tokens.padding,
            large: tokens.padding_lg,
        }
    }
}

/// Result 的度量：对齐 antdv-next `result/style/index.js`。
///
/// 图标圆 72（`fontSizeHeading3 * 3`）、下距 24（`paddingLG`）；
/// 标题 24/32；副标题 14/22；正文块上距与内边距同 `paddingLG`，
/// 横向内边距是 `padding * 2.5`（40），根容器上下 `paddingLG * 2`、左右 `paddingXL`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResultMetric {
    pub icon_size: f32,
    pub icon_glyph_ratio: f32,
    pub icon_margin_bottom: f32,
    pub title_size: f32,
    pub title_line_height: f32,
    pub title_margin: f32,
    pub subtitle_size: f32,
    pub subtitle_line_height: f32,
    pub body_margin_top: f32,
    pub body_padding_block: f32,
    pub body_padding_inline: f32,
    pub extra_margin: f32,
    pub root_padding_block: f32,
    pub root_padding_inline: f32,
}

impl ResultMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            icon_size: tokens.font_size_heading3 * 3.,
            // 实心圆图标里的对勾 / 叉号约占总框的一半
            icon_glyph_ratio: 0.55,
            icon_margin_bottom: tokens.padding_lg,
            title_size: tokens.font_size_heading3,
            title_line_height: (tokens.font_size_heading3 * 1.3333334).round(),
            title_margin: tokens.margin_xs,
            subtitle_size: tokens.font_size,
            subtitle_line_height: tokens.font_height,
            body_margin_top: tokens.padding_lg,
            body_padding_block: tokens.padding_lg,
            body_padding_inline: tokens.padding * 2.5,
            extra_margin: tokens.padding_lg,
            root_padding_block: tokens.padding_lg * 2.,
            root_padding_inline: tokens.padding_xl,
        }
    }
}

/// Tree 的度量：对齐 antdv-next `tree/style/index.js`。
///
/// `titleHeight = switcherSize = indentSize = controlHeightSM (24)`；
/// 节点内容横向内边距 `paddingXS (8)`、圆角 `borderRadius`；
/// 勾选框 `controlInteractiveSize (16)`、圆角 `borderRadiusSM`、右距 `marginXS (8)`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeMetric {
    pub title_height: f32,
    pub indent: f32,
    pub switcher_size: f32,
    pub switcher_icon_size: f32,
    pub content_padding_inline: f32,
    pub radius: f32,
    pub checkbox_size: f32,
    pub checkbox_radius: f32,
    pub checkbox_gap: f32,
}

impl TreeMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            title_height: tokens.control_height_sm,
            indent: tokens.control_height_sm,
            switcher_size: tokens.control_height_sm,
            switcher_icon_size: tokens.font_size_sm,
            content_padding_inline: tokens.padding_xs,
            radius: tokens.border_radius,
            checkbox_size: tokens.control_interactive_size,
            checkbox_radius: tokens.border_radius_sm,
            checkbox_gap: tokens.margin_xs,
        }
    }
}

/// Tree 的取色。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeStyle {
    pub text: Hsla,
    pub switcher: Hsla,
    pub checkbox_bg: Hsla,
    pub checkbox_border: Hsla,
    pub check_fg: Hsla,
    pub selected_bg: Hsla,
    pub selected_text: Hsla,
    pub hover_bg: Hsla,
}

pub fn tree_style(tokens: &Tokens) -> TreeStyle {
    TreeStyle {
        text: tokens.color_text,
        switcher: tokens.color_text_quaternary,
        checkbox_bg: tokens.color_primary,
        checkbox_border: tokens.color_primary,
        check_fg: tokens.color_text_light_solid,
        selected_bg: tokens.control_item_bg_active,
        selected_text: tokens.color_text,
        hover_bg: tokens.control_item_bg_hover,
    }
}

/// Table 的度量：对齐 antdv-next `table/style/index.js` prepareComponentToken。
///
/// 单元格内边距 `padding / padding`（默认档）、字号 `fontSize`、行高 `lineHeight`；
/// 勾选列宽 `controlHeight`；表头圆角 `borderRadiusLG`。
/// `default_col_width` 是库内默认列宽，antd 没有对应令牌。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableMetric {
    pub cell_padding_block: f32,
    pub cell_padding_inline: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub selection_column_width: f32,
    pub expand_column_width: f32,
    pub radius: f32,
    pub default_col_width: f32,
}

impl TableMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            cell_padding_block: tokens.padding,
            cell_padding_inline: tokens.padding,
            font_size: tokens.font_size,
            line_height: tokens.font_height,
            selection_column_width: tokens.control_height,
            expand_column_width: tokens.control_interactive_size + tokens.padding * 2.,
            radius: tokens.border_radius_lg,
            default_col_width: 120.,
        }
    }
}

/// Table 的取色：实心填充 = 半透明填充叠在容器底色上。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableStyle {
    pub header_bg: Hsla,
    pub header_color: Hsla,
    pub border_color: Hsla,
    pub row_hover_bg: Hsla,
    pub row_selected_bg: Hsla,
    pub row_selected_hover_bg: Hsla,
    pub row_expanded_bg: Hsla,
    pub footer_bg: Hsla,
    pub footer_color: Hsla,
}

pub fn table_style(tokens: &Tokens) -> TableStyle {
    let alter_solid = crate::theme::blend(tokens.color_fill_alter, tokens.color_bg_container);
    TableStyle {
        header_bg: alter_solid,
        header_color: tokens.color_text_heading,
        border_color: tokens.color_border_secondary,
        row_hover_bg: alter_solid,
        row_selected_bg: tokens.control_item_bg_active,
        row_selected_hover_bg: tokens.control_item_bg_active_hover,
        row_expanded_bg: alter_solid,
        footer_bg: alter_solid,
        footer_color: tokens.color_text_heading,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuBarMetric {
    pub height: f32,
    pub padding_inline: f32,
    pub gap: f32,
    pub item_padding_inline: f32,
    pub item_radius: f32,
    pub indicator_width: f32,
    pub indicator_height: f32,
    pub font_size: f32,
    pub line_height: f32,
}

impl MenuBarMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            height: tokens.control_height,
            padding_inline: tokens.padding_xs,
            gap: tokens.margin_xxs / 2.,
            item_padding_inline: tokens.padding,
            item_radius: tokens.border_radius_sm,
            indicator_width: tokens.control_height * 0.45,
            indicator_height: tokens.line_width * 2.,
            font_size: tokens.font_size,
            line_height: tokens.font_height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolbarMetric {
    pub height: f32,
    pub padding: f32,
    pub gap: f32,
    pub item_padding_inline: f32,
    pub item_radius: f32,
    pub icon_size: f32,
    pub separator_length: f32,
    pub separator_margin: f32,
    pub font_size: f32,
    pub line_height: f32,
}

impl ToolbarMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            height: tokens.control_height,
            padding: tokens.padding_xs,
            gap: tokens.margin_xxs,
            item_padding_inline: tokens.padding_sm,
            item_radius: tokens.border_radius_sm,
            icon_size: tokens.font_size_lg,
            separator_length: tokens.font_height_lg,
            separator_margin: tokens.margin_xxs,
            font_size: tokens.font_size,
            line_height: tokens.font_height,
        }
    }
}

/// Upload Dragger 的度量：对齐 antdv-next `upload/style/dragger.js`。
///
/// 内边距 `padding`；图标 `fontSizeHeading3 * 2 = 48`、下距 `margin`；
/// 主文案 `fontSizeLG`、下距 `marginXXS`；提示 `fontSize`；圆角 `borderRadiusLG`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UploadMetric {
    pub padding: f32,
    pub icon_size: f32,
    pub icon_margin_top: f32,
    pub icon_margin_bottom: f32,
    pub text_size: f32,
    pub text_line_height: f32,
    pub text_margin_bottom: f32,
    pub hint_size: f32,
    pub hint_line_height: f32,
    pub hint_margin_top: f32,
    pub hint_margin_bottom: f32,
    pub radius: f32,
}

impl UploadMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            padding: tokens.padding,
            icon_size: tokens.font_size_heading3 * 2.,
            // dragger 子节点是 p：UA 默认 1em 上下外边距与 antd token 边距折叠取大者
            icon_margin_top: tokens.font_size,
            icon_margin_bottom: tokens.margin.max(tokens.font_size_lg),
            text_size: tokens.font_size_lg,
            text_line_height: tokens.font_height_lg,
            text_margin_bottom: tokens.margin_xxs,
            hint_size: tokens.font_size,
            hint_line_height: tokens.font_height,
            hint_margin_top: tokens.font_size,
            hint_margin_bottom: tokens.margin_xxs,
            radius: tokens.border_radius_lg,
        }
    }
}

/// Upload Dragger 的取色。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UploadStyle {
    pub bg: Hsla,
    pub border: Hsla,
    pub border_hover: Hsla,
    pub icon: Hsla,
    pub text: Hsla,
    pub hint: Hsla,
    pub disabled: Hsla,
}

pub fn upload_style(tokens: &Tokens) -> UploadStyle {
    UploadStyle {
        bg: tokens.color_fill_alter,
        border: tokens.color_border,
        border_hover: tokens.color_primary_hover,
        icon: tokens.color_primary,
        text: tokens.color_text_heading,
        hint: tokens.color_text_description,
        disabled: tokens.color_text_disabled,
    }
}

/// Transfer 的几何（antdv-next `transfer/style/index.js` prepareComponentToken）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransferMetric {
    pub list_width: f32,
    pub list_height: f32,
    pub header_height: f32,
    pub header_padding_top: f32,
    pub header_padding_bottom: f32,
    pub header_padding_inline: f32,
    pub header_gap: f32,
    pub item_height: f32,
    pub item_padding_block: f32,
    pub item_padding_inline: f32,
    pub item_gap: f32,
    pub radius: f32,
    pub content_radius: f32,
    pub checkbox_size: f32,
    pub checkbox_radius: f32,
    pub actions_margin: f32,
    pub actions_gap: f32,
    pub button_size: f32,
    pub button_radius: f32,
    pub icon_size: f32,
    pub font_size: f32,
    pub line_height: f32,
}

impl TransferMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        let header_v =
            ((tokens.control_height_lg - tokens.line_width - tokens.font_height) / 2.).ceil();
        Self {
            list_width: 180.,
            list_height: 200.,
            header_height: tokens.control_height_lg,
            header_padding_top: header_v - tokens.line_width,
            header_padding_bottom: header_v,
            header_padding_inline: tokens.padding_sm,
            header_gap: 4.,
            item_height: tokens.control_height,
            item_padding_block: (tokens.control_height - tokens.font_height) / 2.,
            item_padding_inline: tokens.padding_sm,
            item_gap: tokens.margin_xs,
            radius: tokens.border_radius_lg,
            content_radius: tokens.border_radius_lg - tokens.line_width,
            checkbox_size: tokens.control_interactive_size,
            checkbox_radius: tokens.border_radius_sm,
            actions_margin: tokens.margin_xs,
            actions_gap: tokens.margin_xxs,
            button_size: tokens.control_height,
            button_radius: tokens.border_radius,
            icon_size: tokens.font_size_sm,
            font_size: tokens.font_size,
            line_height: tokens.font_height,
        }
    }
}

/// Transfer 的取色。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransferStyle {
    pub border: Hsla,
    pub split: Hsla,
    pub header_bg: Hsla,
    pub header_text: Hsla,
    pub item_checked: Hsla,
    pub item_hover: Hsla,
    pub item_checked_hover: Hsla,
    pub disabled: Hsla,
    pub disabled_bg: Hsla,
    pub checkbox_bg: Hsla,
    pub checkbox_border: Hsla,
    pub check_fg: Hsla,
    pub button_bg: Hsla,
    pub button_fg: Hsla,
    pub button_hover: Hsla,
}

pub fn transfer_style(tokens: &Tokens) -> TransferStyle {
    TransferStyle {
        border: tokens.color_border,
        split: tokens.color_split,
        header_bg: tokens.color_bg_container,
        header_text: tokens.color_text,
        item_checked: tokens.control_item_bg_active,
        item_hover: tokens.control_item_bg_hover,
        item_checked_hover: tokens.control_item_bg_active_hover,
        disabled: tokens.color_text_disabled,
        disabled_bg: tokens.color_bg_container_disabled,
        checkbox_bg: tokens.color_primary,
        checkbox_border: tokens.color_border,
        check_fg: tokens.color_text_light_solid,
        button_bg: tokens.color_primary,
        button_fg: tokens.color_text_light_solid,
        button_hover: tokens.color_primary_hover,
    }
}

/// Carousel 的几何（antdv-next `carousel/style/index.js` prepareComponentToken）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarouselMetric {
    pub arrow_size: f32,
    pub arrow_offset: f32,
    pub arrow_opacity: f32,
    pub dot_width: f32,
    pub dot_height: f32,
    pub dot_gap: f32,
    pub dot_offset: f32,
    pub dot_active_width: f32,
    pub dot_opacity: f32,
    pub dot_hover_opacity: f32,
}

impl CarouselMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            arrow_size: 16.,
            arrow_offset: tokens.margin_xs,
            // antd 硬编码 opacity .4 / .2 / .75，非令牌
            arrow_opacity: 0.4,
            dot_width: 16.,
            dot_height: 3.,
            dot_gap: tokens.margin_xxs,
            dot_offset: 12.,
            dot_active_width: 24.,
            dot_opacity: 0.2,
            dot_hover_opacity: 0.75,
        }
    }
}

/// Carousel 的取色（箭头 antd 硬编码 #fff，用 colorTextLightSolid 等价表达）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarouselStyle {
    pub arrow: Hsla,
    pub dot: Hsla,
}

pub fn carousel_style(tokens: &Tokens) -> CarouselStyle {
    CarouselStyle {
        arrow: tokens.color_text_light_solid,
        dot: tokens.color_bg_container,
    }
}

/// Modal 的几何（antdv-next `modal/style/index.js` prepareComponentToken/prepareToken）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModalMetric {
    pub width: f32,
    pub content_padding_block: f32,
    pub content_padding_inline: f32,
    pub radius: f32,
    pub header_margin_bottom: f32,
    pub title_size: f32,
    pub title_line_height: f32,
    pub header_height: f32,
    pub close_size: f32,
    pub close_offset: f32,
    pub close_radius: f32,
    pub close_icon_size: f32,
    pub footer_margin_top: f32,
    pub footer_gap: f32,
    pub body_size: f32,
    pub body_line_height: f32,
}

impl ModalMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        // lineHeightHeading5 与 lineHeightLG 同为 antd 无单位 1.5
        let title_line_height = tokens.font_size_heading5 * tokens.line_height_lg;
        let header_height = title_line_height + tokens.padding * 2.;
        let close_size = tokens.control_height;
        Self {
            width: 520.,
            content_padding_block: tokens.padding_md,
            content_padding_inline: tokens.padding_lg,
            radius: tokens.border_radius_lg,
            header_margin_bottom: tokens.margin_xs,
            title_size: tokens.font_size_heading5,
            title_line_height,
            header_height,
            close_size,
            close_offset: (header_height - close_size) / 2.,
            close_radius: tokens.border_radius_sm,
            close_icon_size: tokens.font_size_lg,
            footer_margin_top: tokens.margin_sm,
            footer_gap: tokens.margin_xs,
            body_size: tokens.font_size,
            body_line_height: tokens.font_height,
        }
    }
}

/// Modal 的取色（非 wireframe 默认：header/footer 透明、无分隔线）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModalStyle {
    pub mask: Hsla,
    pub content_bg: Hsla,
    pub title: Hsla,
    pub close: Hsla,
    pub close_hover: Hsla,
    pub close_hover_bg: Hsla,
    pub close_active_bg: Hsla,
}

pub fn modal_style(tokens: &Tokens) -> ModalStyle {
    ModalStyle {
        mask: tokens.color_bg_mask,
        content_bg: tokens.color_bg_elevated,
        title: tokens.color_text_heading,
        close: tokens.color_icon,
        close_hover: tokens.color_icon_hover,
        close_hover_bg: tokens.color_bg_text_hover,
        close_active_bg: tokens.color_bg_text_active,
    }
}

/// Drawer 的几何（antdv-next `drawer/style/index.js`）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawerMetric {
    pub width: f32,
    pub header_padding_block: f32,
    pub header_padding_inline: f32,
    pub header_font_size: f32,
    pub header_line_height: f32,
    pub border_width: f32,
    pub close_size: f32,
    pub close_gap: f32,
    pub close_radius: f32,
    pub body_padding: f32,
    pub footer_padding_block: f32,
    pub footer_padding_inline: f32,
    pub footer_gap: f32,
}

impl DrawerMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            width: 378.,
            header_padding_block: tokens.padding,
            header_padding_inline: tokens.padding_lg,
            header_font_size: tokens.font_size_lg,
            header_line_height: tokens.font_size_lg * tokens.line_height_lg,
            border_width: tokens.line_width,
            close_size: tokens.font_size_lg + tokens.padding_xs,
            close_gap: tokens.margin_xs,
            close_radius: tokens.border_radius_sm,
            body_padding: tokens.padding_lg,
            footer_padding_block: tokens.padding_xs,
            footer_padding_inline: tokens.padding,
            footer_gap: tokens.margin_xs,
        }
    }
}

/// Drawer 的取色（header/footer 分隔线 colorSplit，面板 colorBgElevated）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawerStyle {
    pub mask: Hsla,
    pub bg: Hsla,
    pub border: Hsla,
    pub text: Hsla,
    pub close: Hsla,
    pub close_hover: Hsla,
    pub close_hover_bg: Hsla,
    pub close_active_bg: Hsla,
}

pub fn drawer_style(tokens: &Tokens) -> DrawerStyle {
    DrawerStyle {
        mask: tokens.color_bg_mask,
        bg: tokens.color_bg_elevated,
        border: tokens.color_split,
        text: tokens.color_text,
        close: tokens.color_icon,
        close_hover: tokens.color_icon_hover,
        close_hover_bg: tokens.color_bg_text_hover,
        close_active_bg: tokens.color_bg_text_active,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RatingMetric {
    pub star_size: f32,
    pub star_size_sm: f32,
    pub star_size_lg: f32,
    pub gap: f32,
}

pub fn rating_metric(tokens: &Tokens) -> RatingMetric {
    RatingMetric {
        star_size: tokens.control_height * 0.625,
        star_size_sm: tokens.control_height_sm * 0.625,
        star_size_lg: tokens.control_height_lg * 0.625,
        gap: tokens.margin_xs,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RatingStyle {
    pub star: Hsla,
    pub star_bg: Hsla,
}

pub fn rating_style(tokens: &Tokens) -> RatingStyle {
    RatingStyle {
        // antd yellow-6（starColor），tokens 无 yellow 系字段；暗色算法产出另一档
        star: match tokens.appearance {
            crate::theme::Appearance::Dark => rgba(0xd8bd14ff).into(),
            _ => rgba(0xfadb14ff).into(),
        },
        star_bg: tokens.color_fill_content,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpinMetric {
    pub dot_size: f32,
    pub dot_size_sm: f32,
    pub dot_size_lg: f32,
    pub item_size: f32,
    pub item_size_sm: f32,
    pub item_size_lg: f32,
    pub dot_scale: f32,
    pub base_opacity: f32,
    pub gap: f32,
    pub desc_size: f32,
    pub desc_line_height: f32,
}

pub fn spin_metric(tokens: &Tokens) -> SpinMetric {
    let item = |dot: f32| (dot - tokens.margin_xxs / 2.) / 2.;
    let dot_size = tokens.control_height_lg / 2.;
    let dot_size_sm = tokens.control_height_lg * 0.35;
    let dot_size_lg = tokens.control_height;
    SpinMetric {
        dot_size,
        dot_size_sm,
        dot_size_lg,
        item_size: item(dot_size),
        item_size_sm: item(dot_size_sm),
        item_size_lg: item(dot_size_lg),
        dot_scale: 0.75,
        base_opacity: 0.3,
        gap: tokens.padding_sm,
        desc_size: tokens.font_size,
        desc_line_height: tokens.font_size,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpinStyle {
    pub dot: Hsla,
}

pub fn spin_style(tokens: &Tokens) -> SpinStyle {
    SpinStyle {
        dot: tokens.color_primary,
    }
}

/// antd Spin 动画帧：holder 旋转角（antRotate 1.2s 线性，起始 45°）与四点亮度
/// （antSpinMove 1s alternate，延迟 0/0.4/0.8/1.2s，基 opacity 0.3 → 1）。
pub fn spin_frame(t: f32) -> (f32, [f32; 4]) {
    let angle = 45. + 360. * t.rem_euclid(1.2) / 1.2;
    let mut op = [0.3; 4];
    for (i, delay) in [0., 0.4, 0.8, 1.2].iter().enumerate() {
        if t >= *delay {
            let p = (t - delay).rem_euclid(2.);
            op[i] = 0.3 + 0.7 * p.min(2. - p);
        }
    }
    (angle, op)
}

/// 四颗圆点中心：item 盒贴 holder 四角、各自 scale 后整体绕中心旋转 angle 度。
pub fn spin_dot_centers(holder: f32, item: f32, angle_deg: f32) -> [(f32, f32); 4] {
    let c = holder / 2.;
    let half = item / 2.;
    let corners = [
        (half, half),
        (holder - half, half),
        (holder - half, holder - half),
        (half, holder - half),
    ];
    let rad = angle_deg.to_radians();
    let (sin, cos) = rad.sin_cos();
    corners.map(|(x, y)| {
        let (dx, dy) = (x - c, y - c);
        (c + dx * cos - dy * sin, c + dx * sin + dy * cos)
    })
}

/// antd empty/utils getAsSolidColor：半透明色叠在容器底上的实色。
fn solid_over(fg: Hsla, bg: Hsla) -> Hsla {
    let f = crate::theme::Color::from(fg);
    let b = crate::theme::Color::from(bg);
    let a = f.a + b.a * (1. - f.a);
    let mix = |fc: f32, bc: f32| (fc * f.a + bc * b.a * (1. - f.a)) / a;
    crate::theme::Color {
        r: mix(f.r, b.r),
        g: mix(f.g, b.g),
        b: mix(f.b, b.b),
        a,
    }
    .hsla()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmptyMetric {
    pub img_height: f32,
    pub img_height_md: f32,
    pub img_height_sm: f32,
    pub img_margin_bottom: f32,
    pub margin_inline: f32,
    pub margin_block_normal: f32,
    pub margin_block_small: f32,
    pub shadow_opacity: f32,
    pub desc_size: f32,
    pub desc_line_height: f32,
    pub footer_margin_top: f32,
}

pub fn empty_metric(tokens: &Tokens) -> EmptyMetric {
    EmptyMetric {
        img_height: tokens.control_height_lg * 2.5,
        img_height_md: tokens.control_height_lg,
        img_height_sm: tokens.control_height_lg * 0.875,
        img_margin_bottom: tokens.margin_xs,
        margin_inline: tokens.margin_xs,
        margin_block_normal: tokens.margin_xl,
        margin_block_small: tokens.margin_xs,
        shadow_opacity: 0.8,
        desc_size: tokens.font_size,
        desc_line_height: tokens.font_height,
        footer_margin_top: tokens.margin,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmptyStyle {
    pub panel_bg: Hsla,
    pub border: Hsla,
    pub detail: Hsla,
    pub shadow: Hsla,
    pub icon: Hsla,
    pub simple_border: Hsla,
    pub simple_shadow: Hsla,
    pub simple_content: Hsla,
    pub desc: Hsla,
}

pub fn empty_style(tokens: &Tokens) -> EmptyStyle {
    let bg = tokens.color_bg_container;
    EmptyStyle {
        panel_bg: solid_over(tokens.color_fill_tertiary, bg),
        border: solid_over(tokens.color_text_quaternary, bg),
        detail: solid_over(tokens.color_fill, bg),
        shadow: solid_over(tokens.color_fill_secondary, bg),
        icon: bg,
        simple_border: solid_over(tokens.color_fill, bg),
        simple_shadow: solid_over(tokens.color_fill_tertiary, bg),
        simple_content: solid_over(tokens.color_fill_quaternary, bg),
        desc: tokens.color_text_description,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorPickerMetric {
    pub trigger_size: f32,
    pub trigger_size_sm: f32,
    pub trigger_size_lg: f32,
    pub trigger_radius: f32,
    pub trigger_radius_sm: f32,
    pub trigger_radius_lg: f32,
    pub trigger_padding: f32,
    pub block_size: f32,
    pub block_size_sm: f32,
    pub block_size_lg: f32,
    pub block_radius: f32,
    pub block_radius_sm: f32,
    pub block_radius_lg: f32,
    pub preset_size: f32,
    pub preset_gap: f32,
    pub check_stroke: f32,
    pub panel_width: f32,
    pub panel_padding: f32,
    pub text_size: f32,
    pub text_gap: f32,
}

pub fn color_picker_metric(tokens: &Tokens) -> ColorPickerMetric {
    ColorPickerMetric {
        trigger_size: tokens.control_height,
        trigger_size_sm: tokens.control_height_sm,
        trigger_size_lg: tokens.control_height_lg,
        trigger_radius: tokens.border_radius,
        trigger_radius_sm: tokens.border_radius_sm,
        trigger_radius_lg: tokens.border_radius_lg,
        trigger_padding: tokens.padding_xxs - tokens.line_width,
        block_size: tokens.control_height_sm,
        block_size_sm: tokens.control_height_xs,
        block_size_lg: tokens.control_height,
        block_radius: tokens.border_radius_sm,
        block_radius_sm: tokens.border_radius_xs,
        block_radius_lg: tokens.border_radius,
        preset_size: 24.,
        preset_gap: tokens.margin_xxs * 1.5,
        check_stroke: tokens.line_width_bold,
        panel_width: 234.,
        panel_padding: tokens.padding_sm,
        text_size: tokens.font_size,
        text_gap: tokens.margin_xs,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorPickerStyle {
    pub trigger_bg: Hsla,
    pub border: Hsla,
    pub hover: Hsla,
    pub active: Hsla,
    pub outline: Hsla,
    pub outline_width: f32,
    pub disabled_text: Hsla,
    pub disabled_bg: Hsla,
    pub inner: Hsla,
    pub inset: Hsla,
    pub check: Hsla,
    pub check_bright: Hsla,
    pub text: Hsla,
    pub preset_hover: Hsla,
}

pub fn color_picker_style(tokens: &Tokens) -> ColorPickerStyle {
    ColorPickerStyle {
        trigger_bg: tokens.color_bg_elevated,
        border: tokens.color_border,
        hover: tokens.color_primary_hover,
        active: tokens.color_primary,
        outline: tokens.color_primary_outline,
        outline_width: tokens.control_outline_width,
        disabled_text: tokens.color_text_disabled,
        disabled_bg: tokens.color_bg_container_disabled,
        inner: tokens.color_fill_secondary,
        inset: tokens.color_text_quaternary,
        check: tokens.color_white,
        check_bright: rgba(0x00000073).into(),
        text: tokens.color_text,
        preset_hover: tokens.color_fill,
    }
}

/// antd ColorPresets isBright：alpha<=.5 看叠底后 HSV 的 v，否则看亮度加权 >192。
pub fn color_preset_bright(r: f32, g: f32, b: f32, a: f32, bg: Hsla) -> bool {
    if a <= 0.5 {
        let bg = crate::theme::Color::from(bg);
        let mix = |c: f32, bc: f32| c * a + bc * (1. - a);
        let v = mix(r, bg.r).max(mix(g, bg.g)).max(mix(b, bg.b)) / 255.;
        v > 0.5
    } else {
        r * 0.299 + g * 0.587 + b * 0.114 > 192.
    }
}

/// 预设选中勾：antd ::after 的右+下边框盒，left 21.5% / top 50%，
/// transform-origin 中心 + rotate(45deg) translate(-50%,-50%)。返回折线三点（预设盒坐标）。
pub fn color_check_points(size: f32) -> [(f32, f32); 3] {
    let w = size / 13. * 5.;
    let h = size / 13. * 8.;
    let corners = [(w, 0.), (w, h), (0., h)];
    let (sin, cos) = 45f32.to_radians().sin_cos();
    corners.map(|(x, y)| {
        let (dx, dy) = (x - w, y - h);
        (
            size * 0.215 + w / 2. + dx * cos - dy * sin,
            size * 0.5 + h / 2. + dx * sin + dy * cos,
        )
    })
}

/// antd Tour 组件常量（源码 `tour/style/index.js`）：卡片宽 520、指示点 6×6、
/// 点间距 = indicatorHeight；关闭钮 `fontSize * lineHeight`；其余走全局令牌。
pub struct TourMetric {
    pub width: f32,
    pub radius: f32,
    pub padding: f32,
    pub padding_xs: f32,
    pub close_size: f32,
    pub close_radius: f32,
    pub close_icon: f32,
    pub indicator: f32,
    pub indicator_gap: f32,
    pub btn_gap: f32,
    pub title_weight: f32,
    pub text_size: f32,
    pub text_line_height: f32,
}

pub fn tour_metric(tokens: &Tokens) -> TourMetric {
    TourMetric {
        width: 520.0,
        radius: tokens.border_radius_lg,
        padding: tokens.padding,
        padding_xs: tokens.padding_xs,
        close_size: tokens.font_size * tokens.line_height,
        close_radius: tokens.border_radius_sm,
        close_icon: tokens.font_size,
        indicator: 6.0,
        indicator_gap: 6.0,
        btn_gap: tokens.margin_xs,
        title_weight: tokens.font_weight_strong,
        text_size: tokens.font_size,
        text_line_height: tokens.font_size * tokens.line_height,
    }
}

pub struct TourStyle {
    pub bg: Hsla,
    pub shadow: Vec<gpui::BoxShadow>,
    /// rc-tour Mask 的默认 fill（源码 `@v-c/tour/dist/Mask.js`），非 colorBgMask
    pub mask: Hsla,
    pub close: Hsla,
    pub close_hover: Hsla,
    pub close_hover_bg: Hsla,
    pub close_active_bg: Hsla,
    pub text: Hsla,
    pub indicator: Hsla,
    pub indicator_active: Hsla,
}

pub fn tour_style(tokens: &Tokens) -> TourStyle {
    TourStyle {
        bg: tokens.color_bg_elevated,
        shadow: tokens.shadow_tertiary.0.clone(),
        mask: gpui::rgba(0x00000080).into(),
        close: tokens.color_icon,
        close_hover: tokens.color_icon_hover,
        close_hover_bg: tokens.color_bg_text_hover,
        close_active_bg: tokens.color_bg_text_active,
        text: tokens.color_text,
        indicator: tokens.color_fill,
        indicator_active: tokens.color_primary,
    }
}

/// DatePicker 日历面板度量（antdv-next `date-picker/style/token.js` + `panel.js`）。
pub struct DatePickerMetric {
    pub panel_width: f32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub cell_inner: f32,
    pub cell_pad_v: f32,
    pub pad_h: f32,
    pub body_pad_v: f32,
    pub header_height: f32,
    pub th_height: f32,
    pub radius: f32,
    pub cell_radius: f32,
    pub footer_line: f32,
    pub footer_pad_h: f32,
    pub header_pad_h: f32,
    pub suffix_gap: f32,
    pub icon_size: f32,
    /// pickerControlIconSize 7 的旋转对角线 ≈ 9.9，取整 10 作 chevron 绘制盒
    pub nav_icon: f32,
    pub font_size: f32,
    pub title_weight: f32,
}

pub fn date_picker_metric(tokens: &Tokens) -> DatePickerMetric {
    let cell_width = tokens.control_height_sm * 1.5;
    let cell_height = tokens.control_height_sm;
    let cell_pad_v = tokens.padding_xxs * 1.5;
    let pad_h = tokens.padding + tokens.padding_xxs / 2.0;
    DatePickerMetric {
        panel_width: cell_width * 7.0 + pad_h * 2.0,
        cell_width,
        cell_height,
        cell_inner: cell_height,
        cell_pad_v,
        pad_h,
        body_pad_v: tokens.padding_xs,
        header_height: tokens.control_height_lg,
        th_height: cell_height + cell_pad_v * 2.0,
        radius: tokens.border_radius_lg,
        cell_radius: tokens.border_radius_sm,
        footer_line: tokens.control_height_lg - tokens.line_width * 2.0,
        footer_pad_h: tokens.padding_sm,
        header_pad_h: tokens.padding_xs,
        suffix_gap: tokens.padding_xs / 2.0,
        icon_size: tokens.font_size,
        nav_icon: 10.0,
        font_size: tokens.font_size,
        title_weight: tokens.font_weight_strong,
    }
}

pub struct DatePickerStyle {
    pub bg: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub split: Hsla,
    pub header_fg: Hsla,
    pub icon: Hsla,
    pub icon_hover: Hsla,
    pub view_hover: Hsla,
    pub cell_fg: Hsla,
    pub cell_out_fg: Hsla,
    pub cell_hover_bg: Hsla,
    pub selected_bg: Hsla,
    pub selected_fg: Hsla,
    pub link: Hsla,
    pub suffix: Hsla,
}

pub fn date_picker_style(tokens: &Tokens) -> DatePickerStyle {
    DatePickerStyle {
        bg: tokens.color_bg_elevated,
        shadow: tokens.shadow_secondary.0.clone(),
        split: tokens.color_split,
        header_fg: tokens.color_text_heading,
        icon: tokens.color_icon,
        icon_hover: tokens.color_icon_hover,
        view_hover: tokens.color_primary,
        cell_fg: tokens.color_text,
        cell_out_fg: tokens.color_text_disabled,
        cell_hover_bg: tokens.control_item_bg_hover,
        selected_bg: tokens.color_primary,
        selected_fg: tokens.color_text_light_solid,
        link: tokens.color_primary,
        suffix: tokens.color_text_quaternary,
    }
}

/// 日期格状态：选中 > 今日描边 > 本月 / 补位灰字。
pub struct DateCellStyle {
    pub fg: Hsla,
    pub bg: Option<Hsla>,
    pub border: Option<Hsla>,
}

pub fn date_cell_state(
    tokens: &Tokens,
    in_view: bool,
    today: bool,
    selected: bool,
) -> DateCellStyle {
    if !in_view {
        return DateCellStyle {
            fg: tokens.color_text_disabled,
            bg: None,
            border: None,
        };
    }
    if selected {
        return DateCellStyle {
            fg: tokens.color_text_light_solid,
            bg: Some(tokens.color_primary),
            border: None,
        };
    }
    DateCellStyle {
        fg: tokens.color_text,
        bg: None,
        border: if today {
            Some(tokens.color_primary)
        } else {
            None
        },
    }
}

/// TimePicker 面板度量（antdv-next `date-picker/style/token.js` + `panel.js` time-panel 段）。
pub struct TimePickerMetric {
    pub column_width: f32,
    pub column_height: f32,
    pub cell_height: f32,
    pub column_margin_v: f32,
    pub cell_margin_h: f32,
    pub inner_width: f32,
    pub inner_pad_start: f32,
    pub cell_radius: f32,
    pub radius: f32,
    pub footer_line: f32,
    pub footer_pad_h: f32,
    pub ok_pad_block: f32,
    pub font_size: f32,
    pub suffix_gap: f32,
    pub icon_size: f32,
}

pub fn time_picker_metric(tokens: &Tokens) -> TimePickerMetric {
    let column_width = tokens.control_height_lg * 1.4;
    let cell_height = 28.0;
    TimePickerMetric {
        column_width,
        column_height: 224.0,
        cell_height,
        column_margin_v: tokens.padding_xxs,
        cell_margin_h: tokens.margin_xxs,
        inner_width: column_width - tokens.margin_xxs * 2.0,
        inner_pad_start: (column_width - cell_height) / 2.0,
        cell_radius: tokens.border_radius_sm,
        radius: tokens.border_radius_lg,
        footer_line: tokens.control_height_lg - tokens.line_width * 2.0,
        footer_pad_h: tokens.padding_sm,
        ok_pad_block: tokens.line_width * 2.0,
        font_size: tokens.font_size,
        suffix_gap: tokens.padding_xs / 2.0,
        icon_size: tokens.font_size,
    }
}

pub struct TimePickerStyle {
    pub bg: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub split: Hsla,
    pub fg: Hsla,
    pub disabled_fg: Hsla,
    pub selected_bg: Hsla,
    pub hover_bg: Hsla,
    pub link: Hsla,
    pub suffix: Hsla,
}

pub fn time_picker_style(tokens: &Tokens) -> TimePickerStyle {
    TimePickerStyle {
        bg: tokens.color_bg_elevated,
        shadow: tokens.shadow_secondary.0.clone(),
        split: tokens.color_split,
        fg: tokens.color_text,
        disabled_fg: tokens.color_text_disabled,
        selected_bg: tokens.control_item_bg_active,
        hover_bg: tokens.control_item_bg_hover,
        link: tokens.color_primary,
        suffix: tokens.color_text_quaternary,
    }
}

/// 时间格状态：选中底 controlItemBgActive（非主色填充），字色不变。
pub struct TimeCellStyle {
    pub fg: Hsla,
    pub bg: Option<Hsla>,
}

pub fn time_cell_state(tokens: &Tokens, selected: bool) -> TimeCellStyle {
    TimeCellStyle {
        fg: tokens.color_text,
        bg: if selected {
            Some(tokens.control_item_bg_active)
        } else {
            None
        },
    }
}

/// RangePicker 附加度量（antdv-next `date-picker/style/index.js` separator 段）。
pub struct RangePickerMetric {
    pub separator_size: f32,
    pub panel_gap: f32,
}

pub fn range_picker_metric(tokens: &Tokens) -> RangePickerMetric {
    RangePickerMetric {
        separator_size: tokens.font_size_lg,
        panel_gap: 0.0,
    }
}

/// 范围格状态：端点主色填充 + 半侧连续底；区间内整格连续底 controlItemBgActive。
pub struct RangeCellStyle {
    pub fg: Hsla,
    pub inner_bg: Option<Hsla>,
    /// 端点内格圆角只保留外侧两角
    pub end: Option<RangeEnd>,
    pub band: Option<Hsla>,
    /// 端点连续底只铺半侧（Start 铺右半、End 铺左半）
    pub band_half: Option<RangeEnd>,
    pub border: Option<Hsla>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RangeEnd {
    Start,
    End,
}

pub fn range_cell_state(
    tokens: &Tokens,
    in_view: bool,
    today: bool,
    is_start: bool,
    is_end: bool,
    in_range: bool,
) -> RangeCellStyle {
    if !in_view {
        return RangeCellStyle {
            fg: tokens.color_text_disabled,
            inner_bg: None,
            end: None,
            band: None,
            band_half: None,
            border: None,
        };
    }
    if is_start || is_end {
        let end = if is_start {
            RangeEnd::Start
        } else {
            RangeEnd::End
        };
        return RangeCellStyle {
            fg: tokens.color_text_light_solid,
            inner_bg: Some(tokens.color_primary),
            end: Some(end),
            band: if is_start && is_end {
                None
            } else {
                Some(tokens.control_item_bg_active)
            },
            band_half: if is_start && is_end { None } else { Some(end) },
            border: None,
        };
    }
    RangeCellStyle {
        fg: tokens.color_text,
        inner_bg: None,
        end: None,
        band: if in_range {
            Some(tokens.control_item_bg_active)
        } else {
            None
        },
        band_half: None,
        border: if today {
            Some(tokens.color_primary)
        } else {
            None
        },
    }
}

/// Calendar 度量（antdv-next `calendar/style/index.js` + `date-picker/style/panel.js`/`token.js`）。
/// 全尺寸月视图：date 格高 = dateValueHeight + dateContentHeight + paddingXS/2 + lineWidthBold。
pub struct CalendarMetric {
    pub header_pad_v: f32,
    pub year_width: f32,
    pub month_width: f32,
    pub control_gap: f32,
    pub week_height: f32,
    pub week_pad_end: f32,
    pub week_pad_bottom: f32,
    pub body_pad_v: f32,
    pub date_margin_h: f32,
    pub date_pad_top: f32,
    pub date_pad_h: f32,
    pub date_value_height: f32,
    pub date_content_height: f32,
    pub date_border: f32,
    pub cell_height: f32,
    pub font_size: f32,
    pub content_line_height: f32,
    pub mode_height: f32,
    pub mode_pad_inline: f32,
    pub mode_radius: f32,
    pub mode_line_height: f32,
    pub menu_max_height: f32,
    pub year_offset: i32,
    pub year_total: i32,
    pub select_height: f32,
    pub select_pad_inline: f32,
    pub select_radius: f32,
    pub icon_size: f32,
    pub option_height: f32,
    pub option_pad_inline: f32,
    pub menu_padding: f32,
    pub menu_radius: f32,
}

pub fn calendar_metric(tokens: &Tokens) -> CalendarMetric {
    let sm = SelectMetric::resolve(tokens);
    let date_value_height = tokens.control_height_sm;
    let date_content_height =
        (tokens.font_height_sm + tokens.margin_xs) * 3.0 + tokens.line_width * 2.0;
    let date_pad_top = tokens.padding_xs / 2.0;
    let date_border = tokens.line_width_bold;
    CalendarMetric {
        header_pad_v: tokens.padding_sm,
        year_width: 80.0,
        month_width: 70.0,
        control_gap: tokens.margin_xs,
        week_height: tokens.control_height_sm * 0.75,
        week_pad_end: tokens.padding_sm,
        week_pad_bottom: tokens.padding_xxs,
        body_pad_v: tokens.padding_xs,
        date_margin_h: tokens.margin_xs / 2.0,
        date_pad_top,
        date_pad_h: tokens.padding_xs,
        date_value_height,
        date_content_height,
        date_border,
        cell_height: date_value_height + date_content_height + date_pad_top + date_border,
        font_size: tokens.font_size,
        content_line_height: tokens.font_height,
        mode_height: tokens.control_height,
        mode_pad_inline: tokens.padding - tokens.line_width,
        // 探针实测：radio-button 组外侧圆角为 borderRadius（6），非 SM
        mode_radius: tokens.border_radius,
        mode_line_height: tokens.control_height - tokens.line_width * 2.0,
        menu_max_height: 256.0,
        year_offset: 10,
        year_total: 20,
        select_height: sm.height[1],
        select_pad_inline: sm.padding_inline[1],
        select_radius: sm.radius[1],
        icon_size: tokens.font_size_icon,
        option_height: sm.option_height,
        option_pad_inline: sm.option_padding_inline,
        menu_padding: sm.menu_padding,
        menu_radius: sm.menu_radius,
    }
}

/// Calendar 配色（fullBg/itemActiveBg 为组件令牌，其余走全局令牌）。
pub struct CalendarStyle {
    pub bg: Hsla,
    pub split: Hsla,
    pub fg: Hsla,
    pub out_fg: Hsla,
    pub hover_bg: Hsla,
    pub selected_bg: Hsla,
    pub selected_fg: Hsla,
    pub today_border: Hsla,
    pub mode_border: Hsla,
    pub mode_fg: Hsla,
    pub mode_bg: Hsla,
    pub mode_checked_fg: Hsla,
    pub mode_hover_fg: Hsla,
    pub mode_checked_hover_fg: Hsla,
    pub suffix: Hsla,
    pub option_selected_bg: Hsla,
    pub option_hover_bg: Hsla,
    pub option_fg: Hsla,
    pub option_weight: f32,
    pub menu_bg: Hsla,
    pub menu_shadow: Vec<gpui::BoxShadow>,
}

pub fn calendar_style(tokens: &Tokens) -> CalendarStyle {
    CalendarStyle {
        bg: tokens.color_bg_container,
        split: tokens.color_split,
        fg: tokens.color_text,
        out_fg: tokens.color_text_disabled,
        hover_bg: tokens.control_item_bg_hover,
        selected_bg: tokens.control_item_bg_active,
        selected_fg: tokens.color_primary,
        today_border: tokens.color_primary,
        mode_border: tokens.color_border,
        mode_fg: tokens.color_text,
        mode_bg: tokens.color_bg_container,
        mode_checked_fg: tokens.color_primary,
        mode_hover_fg: tokens.color_primary,
        mode_checked_hover_fg: tokens.color_primary_hover,
        suffix: tokens.color_text_quaternary,
        option_selected_bg: tokens.control_item_bg_active,
        option_hover_bg: tokens.control_item_bg_hover,
        option_fg: tokens.color_text,
        option_weight: tokens.font_weight_strong,
        menu_bg: tokens.color_bg_elevated,
        menu_shadow: tokens.shadow_secondary.0.clone(),
    }
}

/// 全尺寸月视图 date 格：选中底 controlItemBgActive + 值主色；今日顶描边主色；补位值禁用色。
pub struct CalendarCellStyle {
    pub value_fg: Hsla,
    pub bg: Option<Hsla>,
    pub border_top: Hsla,
}

pub fn calendar_cell_state(
    tokens: &Tokens,
    in_view: bool,
    today: bool,
    selected: bool,
) -> CalendarCellStyle {
    if !in_view {
        return CalendarCellStyle {
            value_fg: tokens.color_text_disabled,
            bg: None,
            border_top: tokens.color_split,
        };
    }
    CalendarCellStyle {
        value_fg: if selected {
            tokens.color_primary
        } else {
            tokens.color_text
        },
        bg: if selected {
            Some(tokens.control_item_bg_active)
        } else {
            None
        },
        border_top: if today {
            tokens.color_primary
        } else {
            tokens.color_split
        },
    }
}

/// Popconfirm 度量（antdv-next `popconfirm/style/index.js`，外壳同 Popover）。
/// message 底距 marginXS / 图标字号 fontSize 右距 marginXS / 描述顶距 marginXXS /
/// 按钮间距 marginXS（marginInlineStart）/ 标题字重 fontWeightStrong。
pub struct PopconfirmMetric {
    pub message_margin_bottom: f32,
    pub icon_gap: f32,
    pub icon_size: f32,
    pub desc_margin_top: f32,
    pub button_gap: f32,
    pub title_weight: f32,
}

pub fn popconfirm_metric(tokens: &Tokens) -> PopconfirmMetric {
    PopconfirmMetric {
        message_margin_bottom: tokens.margin_xs,
        icon_gap: tokens.margin_xs,
        icon_size: tokens.font_size,
        desc_margin_top: tokens.margin_xxs,
        button_gap: tokens.margin_xs,
        title_weight: tokens.font_weight_strong,
    }
}

/// Popconfirm 配色：图标 colorWarning / 标题 colorTextHeading / 描述 colorText。
pub struct PopconfirmStyle {
    pub icon: Hsla,
    pub title_fg: Hsla,
    pub desc_fg: Hsla,
}

pub fn popconfirm_style(tokens: &Tokens) -> PopconfirmStyle {
    PopconfirmStyle {
        icon: tokens.color_warning,
        title_fg: tokens.color_text_heading,
        desc_fg: tokens.color_text,
    }
}

/// Notification / Message 共享配色（antdv-next `notification/style/notification.js`
/// genListItemSharedStyle）：卡片 colorBgElevated；图标四态色 colorInfo/Success/Warning/Error；
/// 标题 colorTextHeading；描述与 Message 正文 colorText；关闭按钮 colorIcon，
/// hover colorIconHover + colorBgTextHover，active colorBgTextActive。
pub struct NoticeStyle {
    pub bg: Hsla,
    pub icon_info: Hsla,
    pub icon_success: Hsla,
    pub icon_warning: Hsla,
    pub icon_error: Hsla,
    pub title_fg: Hsla,
    pub text_fg: Hsla,
    pub close_fg: Hsla,
    pub close_fg_hover: Hsla,
    pub close_bg_hover: Hsla,
    pub close_bg_active: Hsla,
}

pub fn notice_style(tokens: &Tokens) -> NoticeStyle {
    NoticeStyle {
        bg: tokens.color_bg_elevated,
        icon_info: tokens.color_info,
        icon_success: tokens.color_success,
        icon_warning: tokens.color_warning,
        icon_error: tokens.color_error,
        title_fg: tokens.color_text_heading,
        text_fg: tokens.color_text,
        close_fg: tokens.color_icon,
        close_fg_hover: tokens.color_icon_hover,
        close_bg_hover: tokens.color_bg_text_hover,
        close_bg_active: tokens.color_bg_text_active,
    }
}

/// 关闭按钮交互态：idle 无底色，hover/active 换底色（colorBgTextHover/Active）；
/// active 以 hover 为前提（指针不在钮上按下不生效）。
pub struct NoticeCloseState {
    pub fg: Hsla,
    pub bg: Option<Hsla>,
}

pub fn notice_close_state(tokens: &Tokens, hovered: bool, pressed: bool) -> NoticeCloseState {
    let cs = notice_style(tokens);
    if hovered && pressed {
        NoticeCloseState {
            fg: cs.close_fg_hover,
            bg: Some(cs.close_bg_active),
        }
    } else if hovered {
        NoticeCloseState {
            fg: cs.close_fg_hover,
            bg: Some(cs.close_bg_hover),
        }
    } else {
        NoticeCloseState {
            fg: cs.close_fg,
            bg: None,
        }
    }
}

/// Notification 度量（antdv-next `notification/style/index.js` prepareNotificationToken +
/// `notification/style/notification.js` genNotificationItemStyle）：
/// 宽 384（组件默认 token）；内边距 paddingMD / paddingContentHorizontalLG(=sizeLG)；
/// 图标字号 fontSizeLG*lineHeightLG；wrapper gap marginSM；section gap marginXS；
/// 标题 fontSizeLG/lineHeightLG；关闭钮 controlHeightLG*0.55，top paddingMD、
/// inlineEnd paddingContentHorizontalLG，圆角 borderRadiusSM；closable 文本右留 paddingLG；
/// actions 顶距 marginSM；堆栈 gap margin、边距 marginLG；圆角 borderRadiusLG。
pub struct NotificationMetric {
    pub width: f32,
    pub padding_vertical: f32,
    pub padding_horizontal: f32,
    pub radius: f32,
    pub font: f32,
    pub line: f32,
    pub icon_size: f32,
    pub icon_gap: f32,
    pub section_gap: f32,
    pub title_font: f32,
    pub title_line: f32,
    pub desc_font: f32,
    pub close_size: f32,
    pub close_top: f32,
    pub close_right: f32,
    pub close_radius: f32,
    pub closable_pad_end: f32,
    pub actions_margin_top: f32,
    pub stack_gap: f32,
    pub edge: f32,
}

pub fn notification_metric(tokens: &Tokens) -> NotificationMetric {
    NotificationMetric {
        width: 384.0,
        padding_vertical: tokens.padding_md,
        padding_horizontal: tokens.padding_lg,
        radius: tokens.border_radius_lg,
        font: tokens.font_size,
        line: tokens.font_height,
        icon_size: tokens.font_height_lg,
        icon_gap: tokens.margin_sm,
        section_gap: tokens.margin_xs,
        title_font: tokens.font_size_lg,
        title_line: tokens.font_height_lg,
        desc_font: tokens.font_size,
        close_size: tokens.control_height_lg * 0.55,
        close_top: tokens.padding_md,
        close_right: tokens.padding_lg,
        close_radius: tokens.border_radius_sm,
        closable_pad_end: tokens.padding_lg,
        actions_margin_top: tokens.margin_sm,
        stack_gap: tokens.margin,
        edge: tokens.margin_lg,
    }
}

/// Message 度量（antdv-next `message/style/index.js` prepareMessageToken +
/// genMessageItemStyle）：内边距 (controlHeightLG-fontSize*lineHeight)/2 / paddingSM；
/// 图标字号 fontSizeLG；wrapper 居中 gap marginXS；正文 fontSize/lineHeight；
/// 圆角 borderRadiusLG；堆栈 gap margin、边距 marginLG。
pub struct MessageMetric {
    pub padding_vertical: f32,
    pub padding_horizontal: f32,
    pub radius: f32,
    pub font: f32,
    pub line: f32,
    pub icon_size: f32,
    pub icon_gap: f32,
    pub stack_gap: f32,
    pub edge: f32,
}

pub fn message_metric(tokens: &Tokens) -> MessageMetric {
    MessageMetric {
        padding_vertical: (tokens.control_height_lg - tokens.font_height) / 2.0,
        padding_horizontal: tokens.padding_sm,
        radius: tokens.border_radius_lg,
        font: tokens.font_size,
        line: tokens.font_height,
        icon_size: tokens.font_size_lg,
        icon_gap: tokens.margin_xs,
        stack_gap: tokens.margin,
        edge: tokens.margin_lg,
    }
}

/// InputNumber 度量（antdv-next `input-number/style/token.js` prepareComponentToken）：
/// 默认宽 controlWidth 90；步进列宽 handleWidth = controlHeightSM − lineWidth×2；
/// 箭头字号 handleFontSize = fontSize/2。
pub struct InputNumberMetric {
    pub control_width: f32,
    pub handle_width: f32,
    pub handle_font: f32,
}

pub fn input_number_metric(tokens: &Tokens) -> InputNumberMetric {
    InputNumberMetric {
        control_width: 90.0,
        handle_width: tokens.control_height_sm - tokens.line_width * 2.0,
        handle_font: tokens.font_size / 2.0,
    }
}

/// 步进按钮交互态（`input-number/style/index.js` genInputNumberStyles）：
/// 常态 colorIcon，悬停 handleHoverColor(=colorPrimary)，按下 handleActiveBg(=colorFillAlter)，
/// 禁用 colorTextDisabled 且无按下底色。
pub struct NumberHandleState {
    pub fg: Hsla,
    pub bg: Option<Hsla>,
}

pub fn number_handle_state(
    tokens: &Tokens,
    hovered: bool,
    pressed: bool,
    disabled: bool,
) -> NumberHandleState {
    if disabled {
        return NumberHandleState {
            fg: tokens.color_text_disabled,
            bg: None,
        };
    }
    NumberHandleState {
        fg: if hovered {
            tokens.color_primary
        } else {
            tokens.color_icon
        },
        bg: if hovered && pressed {
            Some(tokens.color_fill_alter)
        } else {
            None
        },
    }
}

/// 步进列底色：outlined/underlined 恒 handleBg(=colorBgContainer)；
/// filled 常态 filledHandleBg = colorFillSecondary 叠 colorBgContainer，
/// focus-within 换回 handleBg。
pub fn number_actions_bg(tokens: &Tokens, filled: bool, focused: bool) -> Hsla {
    if filled && !focused {
        crate::theme::blend(tokens.color_fill_secondary, tokens.color_bg_container)
    } else {
        tokens.color_bg_container
    }
}

/// 输入框前后缀度量（`input/style/index.js` genAffixStyle / genAllowClearStyle）：
/// 密码眼睛 / 搜索前缀图标 1em = fontSize；清空图标 fontSizeIcon(=fontSizeSM)；
/// 前后缀与正文间距 inputAffixPadding(=paddingXXS)。
pub struct InputAffixMetric {
    pub icon_font: f32,
    pub clear_font: f32,
    pub gap: f32,
}

pub fn input_affix_metric(tokens: &Tokens) -> InputAffixMetric {
    InputAffixMetric {
        icon_font: tokens.font_size,
        clear_font: tokens.font_size_sm,
        gap: tokens.padding_xxs,
    }
}

/// 密码显隐图标颜色（genAffixStyle `-password-icon`）：colorIcon，悬停 colorIconHover；
/// 禁用恒 colorIcon（悬停不变色）。
pub fn password_icon_color(tokens: &Tokens, hovered: bool, disabled: bool) -> Hsla {
    if hovered && !disabled {
        tokens.color_icon_hover
    } else {
        tokens.color_icon
    }
}

/// 清空图标颜色（genAllowClearStyle `-clear-icon`）：
/// colorTextQuaternary → 悬停 colorIcon → 按下 colorText。
pub fn clear_icon_color(tokens: &Tokens, hovered: bool, pressed: bool) -> Hsla {
    if hovered && pressed {
        tokens.color_text
    } else if hovered {
        tokens.color_icon
    } else {
        tokens.color_text_quaternary
    }
}

/// Collapse 度量（`collapse/style/index.js` prepareComponentToken + genBaseStyle 尺寸档）。
/// headerPadding 三档 `12 16` / `8 12 8 8` / `16 24 16 16`；body `16 16` / 12 / 24；
/// 箭头 fontSizeIcon 12、与标题间距 marginSM（icon-placement-end 的 marginInlineStart）。
pub struct CollapseMetric {
    pub header_pad_v: f32,
    pub header_pad_h: f32,
    pub header_pad_left: f32,
    pub body_pad_v: f32,
    pub body_pad_h: f32,
    pub icon_gap: f32,
    pub icon_font: f32,
    pub font: f32,
    pub font_height: f32,
    pub radius: f32,
}

pub fn collapse_metric(tokens: &Tokens, size: crate::text_input::InputSize) -> CollapseMetric {
    let (hv, hh, hl, bv, bh, font, fh) = match size {
        crate::text_input::InputSize::Small => (
            tokens.padding_xs,
            tokens.padding_sm,
            tokens.padding_xs,
            tokens.padding_sm,
            tokens.padding_sm,
            tokens.font_size,
            tokens.font_size * tokens.line_height,
        ),
        crate::text_input::InputSize::Middle => (
            tokens.padding_sm,
            tokens.padding,
            tokens.padding,
            tokens.padding,
            tokens.padding,
            tokens.font_size,
            tokens.font_size * tokens.line_height,
        ),
        crate::text_input::InputSize::Large => (
            tokens.padding,
            tokens.padding_lg,
            tokens.padding,
            tokens.padding_lg,
            tokens.padding_lg,
            tokens.font_size_lg,
            tokens.font_size_lg * tokens.line_height_lg,
        ),
    };
    CollapseMetric {
        header_pad_v: hv,
        header_pad_h: hh,
        header_pad_left: hl,
        body_pad_v: bv,
        body_pad_h: bh,
        icon_gap: tokens.margin_sm,
        icon_font: tokens.font_size_sm,
        font,
        font_height: fh,
        radius: tokens.border_radius_lg,
    }
}

/// Collapse 头底 = headerBg(colorFillAlter) 叠在容器底上（浏览器半透明合成等价）。
pub fn collapse_header_bg(tokens: &Tokens) -> Hsla {
    crate::theme::blend(tokens.color_fill_alter, tokens.color_bg_container)
}

/// Collapse 头文字 = colorTextHeading；禁用 colorTextDisabled。
pub fn collapse_header_fg(tokens: &Tokens, disabled: bool) -> Hsla {
    if disabled {
        tokens.color_text_disabled
    } else {
        tokens.color_text_heading
    }
}

/// Collapse 内容底 = contentBg(colorBgContainer)；边框/分隔 = colorBorder。
pub fn collapse_content_bg(tokens: &Tokens) -> Hsla {
    tokens.color_bg_container
}

pub fn collapse_border(tokens: &Tokens) -> Hsla {
    tokens.color_border
}

/// Anchor 度量（`anchor/style/index.js` prepareComponentToken + mergeToken）：
/// 轨宽 lineWidthBold；link paddingBlock paddingXXS（子级减半）；paddingInlineStart padding；
/// 标题下间距 fontSize/14×3；ink 高 = 标题行高 fontHeight。
pub struct AnchorMetric {
    pub rail_width: f32,
    pub link_pad_block: f32,
    pub nested_pad_block: f32,
    pub link_pad_left: f32,
    pub title_gap: f32,
    pub font_height: f32,
}

pub fn anchor_metric(tokens: &Tokens) -> AnchorMetric {
    AnchorMetric {
        rail_width: tokens.line_width_bold,
        link_pad_block: tokens.padding_xxs,
        nested_pad_block: tokens.padding_xxs / 2.0,
        link_pad_left: tokens.padding,
        title_gap: tokens.font_size / 14.0 * 3.0,
        font_height: tokens.font_size * tokens.line_height,
    }
}

/// Anchor 标题色：激活 colorPrimary；悬停 colorPrimaryHover（全局 a:hover）；其余 colorText。
pub fn anchor_title_color(tokens: &Tokens, active: bool, hovered: bool) -> Hsla {
    if active {
        tokens.color_primary
    } else if hovered {
        tokens.color_primary_hover
    } else {
        tokens.color_text
    }
}

/// Anchor 轨 = colorSplit；ink = colorPrimary。
pub fn anchor_rail_color(tokens: &Tokens) -> Hsla {
    tokens.color_split
}

pub fn anchor_ink_color(tokens: &Tokens) -> Hsla {
    tokens.color_primary
}

/// Form 度量（`form/style/index.js` prepareComponentToken + genFormSize）：
/// 标签高 controlHeight 三档；必填星号间距 marginXXS；冒号间距 marginXXS/2 与 marginXS；
/// 项间距 marginLG；纵向标签下内边距 paddingXS；提示字号 fontSize；extra 最小高 controlHeightSM。
pub struct FormMetric {
    pub label_height: f32,
    pub vertical_label_height: f32,
    pub label_font: f32,
    pub required_gap: f32,
    pub colon_start: f32,
    pub colon_end: f32,
    pub item_gap: f32,
    pub label_pad_bottom: f32,
    pub msg_font: f32,
    pub extra_min_height: f32,
}

pub fn form_metric(tokens: &Tokens, size: crate::text_input::InputSize) -> FormMetric {
    let label_height = match size {
        crate::text_input::InputSize::Small => tokens.control_height_sm,
        crate::text_input::InputSize::Middle => tokens.control_height,
        crate::text_input::InputSize::Large => tokens.control_height_lg,
    };
    FormMetric {
        label_height,
        // 纵向标签高：middle 为 auto(行高)，small/large 被 genFormSize 钉到 controlHeight 档
        vertical_label_height: match size {
            crate::text_input::InputSize::Middle => tokens.font_size * tokens.line_height,
            _ => label_height,
        },
        label_font: tokens.font_size,
        required_gap: tokens.margin_xxs,
        colon_start: tokens.margin_xxs / 2.0,
        colon_end: tokens.margin_xs,
        item_gap: tokens.margin_lg,
        label_pad_bottom: tokens.padding_xs,
        msg_font: tokens.font_size,
        extra_min_height: tokens.control_height_sm,
    }
}

/// Form 标签色 = labelColor(colorTextHeading)；必填星号 = labelRequiredMarkColor(colorError)。
pub fn form_label_color(tokens: &Tokens) -> Hsla {
    tokens.color_text_heading
}

pub fn form_required_color(tokens: &Tokens) -> Hsla {
    tokens.color_error
}

/// Form 提示色：错误 colorError，其余（help/extra）colorTextDescription。
pub fn form_msg_color(tokens: &Tokens, is_error: bool) -> Hsla {
    if is_error {
        tokens.color_error
    } else {
        tokens.color_text_description
    }
}

/// Cascader 度量（`cascader/style/index.js` prepareComponentToken + `columns.js`）：
/// 触发框默认宽 controlWidth 184；列最小宽 controlItemWidth 111、列高 dropdownHeight 180；
/// 选项内边距 `round((controlHeight − fontSize×lineHeight)/2)` × paddingSM；
/// 列内边距 menuPadding(paddingXXS)；选项圆角 borderRadiusSM；
/// 展开图标间距 paddingXXS、字号 fontSizeIcon；列分隔 lineWidth colorSplit。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CascaderMetric {
    pub control_width: f32,
    pub item_min_width: f32,
    pub dropdown_height: f32,
    pub option_pad_block: f32,
    pub option_pad_inline: f32,
    pub menu_pad: f32,
    pub item_radius: f32,
    pub icon_gap: f32,
    pub icon_font: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub column_border: f32,
    pub panel_radius: f32,
}

pub fn cascader_metric(tokens: &Tokens) -> CascaderMetric {
    CascaderMetric {
        control_width: 184.0,
        item_min_width: 111.0,
        dropdown_height: 180.0,
        option_pad_block: ((tokens.control_height - tokens.font_size * tokens.line_height) / 2.0)
            .round(),
        option_pad_inline: tokens.padding_sm,
        menu_pad: tokens.padding_xxs,
        item_radius: tokens.border_radius_sm,
        icon_gap: tokens.padding_xxs,
        icon_font: tokens.font_size_icon,
        font_size: tokens.font_size,
        line_height: tokens.font_height,
        column_border: tokens.line_width,
        panel_radius: tokens.border_radius_lg,
    }
}

/// Cascader 选项三态：路径上的项（active）选中底 + 加粗，悬停 controlItemBgHover，
/// 禁用 colorTextDisabled 且悬停不换底。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CascaderItemStyle {
    pub bg: Hsla,
    pub fg: Hsla,
    pub font_weight: FontWeight,
}

pub fn cascader_item_state(
    tokens: &Tokens,
    selected: bool,
    hovered: bool,
    disabled: bool,
) -> CascaderItemStyle {
    if disabled {
        return CascaderItemStyle {
            bg: gpui::transparent_black(),
            fg: tokens.color_text_disabled,
            font_weight: FontWeight::NORMAL,
        };
    }
    if selected {
        return CascaderItemStyle {
            bg: tokens.control_item_bg_active,
            fg: tokens.color_text,
            font_weight: FontWeight(tokens.font_weight_strong),
        };
    }
    CascaderItemStyle {
        bg: if hovered {
            tokens.control_item_bg_hover
        } else {
            gpui::transparent_black()
        },
        fg: tokens.color_text,
        font_weight: FontWeight::NORMAL,
    }
}

/// 展开箭头色：常态 colorIcon，禁用 colorTextDisabled。
pub fn cascader_icon_color(tokens: &Tokens, disabled: bool) -> Hsla {
    if disabled {
        tokens.color_text_disabled
    } else {
        tokens.color_icon
    }
}

/// 列分隔线色 = colorSplit。
pub fn cascader_column_border(tokens: &Tokens) -> Hsla {
    tokens.color_split
}

/// 浮层底 = colorBgElevated（与 Select 菜单同源）。
pub fn cascader_menu_bg(tokens: &Tokens) -> Hsla {
    tokens.color_bg_elevated
}

/// 展开时触发框已选值的透明度：antdv-next `select/style/select-input.js`
/// `.ant-select-open .ant-select-content-has-value { opacity: .25 }`（字面量，非令牌）。
pub fn select_content_open_opacity() -> f32 {
    0.25
}

/// TreeSelect 浮层度量（`tree-select/style/index.js`）：
/// 内边距 paddingXS × paddingXS/2，圆角 borderRadiusLG，底 colorBgElevated；
/// 内部树沿用 TreeMetric / tree_style（colorBgContainer 换成 elevated）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeSelectMetric {
    pub pad_block: f32,
    pub pad_inline: f32,
    pub radius: f32,
}

pub fn tree_select_metric(tokens: &Tokens) -> TreeSelectMetric {
    TreeSelectMetric {
        pad_block: tokens.padding_xs,
        pad_inline: tokens.padding_xs / 2.0,
        radius: tokens.border_radius_lg,
    }
}

pub fn tree_select_menu_bg(tokens: &Tokens) -> Hsla {
    tokens.color_bg_elevated
}

/// FloatButton 度量（antdv-next `float-button/style/index.js`）：
/// 尺寸 controlHeightLG，图标 fontSizeIcon × 1.5，纵向内边距 paddingXXS、间隔 paddingXXS/2，
/// 独立按钮偏移 bottom marginXXL / right marginLG；内部 Button 为 size=large，
/// 方形圆角走 borderRadiusLG。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatButtonMetric {
    pub size: f32,
    pub icon_size: f32,
    pub padding_block: f32,
    pub gap: f32,
    pub inset_block_end: f32,
    pub inset_inline_end: f32,
    pub radius_square: f32,
    pub border_width: f32,
}

pub fn float_button_metric(tokens: &Tokens) -> FloatButtonMetric {
    FloatButtonMetric {
        size: tokens.control_height_lg,
        icon_size: tokens.font_size_icon * 1.5,
        padding_block: tokens.padding_xxs,
        gap: tokens.padding_xxs / 2.0,
        inset_block_end: tokens.margin_xxl,
        inset_inline_end: tokens.margin_lg,
        radius_square: tokens.border_radius_lg,
        border_width: tokens.line_width,
    }
}

/// Splitter 度量（antdv-next `splitter/style/index.js` prepareComponentToken）：
/// 可见条 2、命中区 6、中心指示段 20（resizeSpinnerSize）；面板横向内边距 1（CSS `0 1px`）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitterMetric {
    pub bar_size: f32,
    pub trigger_size: f32,
    pub draggable_size: f32,
    pub panel_padding_inline: f32,
    pub collapse_width: f32,
    pub collapse_height: f32,
    pub collapse_radius: f32,
    pub collapse_font: f32,
}

pub fn splitter_metric(tokens: &Tokens) -> SplitterMetric {
    SplitterMetric {
        bar_size: 2.0,
        trigger_size: 6.0,
        draggable_size: 20.0,
        panel_padding_inline: 1.0,
        collapse_width: tokens.font_size_sm,
        collapse_height: tokens.control_height_sm,
        collapse_radius: tokens.border_radius_xs,
        collapse_font: tokens.font_size_sm,
    }
}

/// 分割条 / 折叠钮底色三态：静止 controlItemBgHover，hover controlItemBgActive，
/// 拖拽中 controlItemBgActiveHover；禁用回到静止色（antd `-disabled` 规则）。
pub fn splitter_bar_state(tokens: &Tokens, hover: bool, active: bool, disabled: bool) -> Hsla {
    if disabled || !hover && !active {
        return tokens.control_item_bg_hover;
    }
    if active {
        tokens.control_item_bg_active_hover
    } else {
        tokens.control_item_bg_active
    }
}

/// 分割条中心指示段颜色：colorFill（禁用时隐藏，由渲染层决定）。
pub fn splitter_knob_color(tokens: &Tokens) -> Hsla {
    tokens.color_fill
}

/// antd Layout 写死的深navy常量（`layout/style/index.js` prepareComponentToken）：
/// headerBg / siderBg `#001529`、triggerBg `#002140`，不随种子变化。
const LAYOUT_HEADER_BG: u32 = 0x001529;
const LAYOUT_TRIGGER_BG: u32 = 0x002140;

/// Layout 度量（antdv-next `layout/style/index.js` + `sider.js`）：
/// header 高 controlHeight×2、横向内边距 controlHeightLG×1.25、行高同高；
/// footer 纵向 controlHeightSM、横向同 header；body/footer 底 colorBgLayout；
/// sider 默认深navy，light 主题 colorBgContainer；trigger 高 controlHeightLG+marginXXS×2。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutMetric {
    pub header_height: f32,
    pub header_padding_inline: f32,
    pub header_bg: Hsla,
    pub header_color: Hsla,
    pub footer_padding_block: f32,
    pub footer_padding_inline: f32,
    pub footer_bg: Hsla,
    pub footer_color: Hsla,
    pub footer_font_size: f32,
    pub font_size: f32,
    pub body_bg: Hsla,
    pub sider_bg: Hsla,
    pub light_sider_bg: Hsla,
    pub trigger_height: f32,
    pub trigger_bg: Hsla,
    pub trigger_color: Hsla,
    pub light_trigger_bg: Hsla,
    pub light_trigger_color: Hsla,
}

pub fn layout_metric(tokens: &Tokens) -> LayoutMetric {
    let padding_inline = tokens.control_height_lg * 1.25;
    LayoutMetric {
        header_height: tokens.control_height * 2.0,
        header_padding_inline: padding_inline,
        header_bg: gpui::rgb(LAYOUT_HEADER_BG).into(),
        header_color: tokens.color_text,
        footer_padding_block: tokens.control_height_sm,
        footer_padding_inline: padding_inline,
        footer_bg: tokens.color_bg_layout,
        footer_color: tokens.color_text,
        footer_font_size: tokens.font_size,
        font_size: tokens.font_size,
        body_bg: tokens.color_bg_layout,
        sider_bg: gpui::rgb(LAYOUT_HEADER_BG).into(),
        light_sider_bg: tokens.color_bg_container,
        trigger_height: tokens.control_height_lg + tokens.margin_xxs * 2.0,
        trigger_bg: gpui::rgb(LAYOUT_TRIGGER_BG).into(),
        trigger_color: tokens.color_text_light_solid,
        light_trigger_bg: tokens.color_bg_container,
        light_trigger_color: tokens.color_text,
    }
}

/// 状态栏度量：高 controlHeightSM + marginXXS，字号 fontSizeSM，顶分隔线 colorSplit。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusBarMetric {
    pub height: f32,
    pub padding_inline: f32,
    pub gap: f32,
    pub font_size: f32,
    pub separator_height: f32,
    pub border_width: f32,
    pub bg: Hsla,
    pub fg: Hsla,
    pub border_color: Hsla,
}

pub fn status_bar_metric(tokens: &Tokens) -> StatusBarMetric {
    StatusBarMetric {
        height: tokens.control_height_sm + tokens.margin_xxs,
        padding_inline: tokens.padding_xs,
        gap: tokens.margin_sm,
        font_size: tokens.font_size_sm,
        separator_height: tokens.font_size_sm,
        border_width: tokens.line_width,
        bg: tokens.color_bg_container,
        fg: tokens.color_text_secondary,
        border_color: tokens.color_split,
    }
}

/// antd Grid 列数（`grid/style/index.js` gridColumns，字面量 24）。
pub const GRID_COLUMNS: u32 = 24;

/// Col 的 flex-basis / maxWidth 占比：span / 24。
pub fn grid_col_basis(span: u32) -> f32 {
    span as f32 / GRID_COLUMNS as f32
}

/// Col offset 的 marginInlineStart 占比：offset / 24。
pub fn grid_offset_margin(offset: u32) -> f32 {
    offset as f32 / GRID_COLUMNS as f32
}

/// Row gutter 的负外边距：-gutter/2（antd 把 gutter 拆成两侧 Col 的内边距）。
pub fn grid_row_gutter_margin(gutter: f32) -> f32 {
    -gutter / 2.0
}

/// Col 在 gutter 下的横向内边距：gutter/2。
pub fn grid_col_gutter_padding(gutter: f32) -> f32 {
    gutter / 2.0
}

// ---- NavMenu（antd Menu inline 模式） ----

/// rc-menu `inlineIndent` 默认值（`@v-c/menu/dist/Menu.js` defaults，字面量 24）；
/// 行 paddingLeft = inlineIndent × 层级（顶层为 1）。
pub const NAV_INLINE_INDENT: f32 = 24.0;

/// Menu inline 度量（`menu/style/index.js` prepareComponentToken + `vertical.js`
/// + `theme.js` root borderInlineEnd = activeBarBorderWidth(lineWidth) colorSplit）。
pub struct NavMenuMetric {
    pub item_height: f32,
    pub item_margin_inline: f32,
    pub item_margin_block: f32,
    pub item_padding_inline: f32,
    pub item_radius: f32,
    pub sub_item_radius: f32,
    pub inline_indent: f32,
    pub icon_size: f32,
    pub icon_margin_inline_end: f32,
    pub arrow_size: f32,
    pub arrow_inset_end: f32,
    pub title_padding_end_with_arrow: f32,
    pub font_size: f32,
    pub sub_item_bg: Hsla,
    pub border_width: f32,
    pub border_color: Hsla,
}

pub fn nav_menu_metric(tokens: &Tokens) -> NavMenuMetric {
    let arrow_size = tokens.font_size / 7.0 * 5.0;
    NavMenuMetric {
        item_height: tokens.control_height_lg,
        item_margin_inline: tokens.margin_xxs,
        item_margin_block: tokens.margin_xxs,
        item_padding_inline: tokens.padding,
        item_radius: tokens.border_radius_lg,
        sub_item_radius: tokens.border_radius_sm,
        inline_indent: NAV_INLINE_INDENT,
        icon_size: tokens.font_size,
        icon_margin_inline_end: tokens.control_height_sm - tokens.font_size,
        arrow_size,
        arrow_inset_end: tokens.margin,
        title_padding_end_with_arrow: arrow_size + tokens.padding + tokens.margin_xs,
        font_size: tokens.font_size,
        sub_item_bg: tokens.color_fill_alter,
        border_width: tokens.line_width,
        border_color: tokens.color_split,
    }
}

pub struct NavMenuState {
    pub bg: Hsla,
    pub fg: Hsla,
}

/// 叶子条目状态：选中底 controlItemBgActive + 主色字；hover 底 colorBgTextHover。
pub fn nav_menu_item_state(
    tokens: &Tokens,
    selected: bool,
    hover: bool,
    disabled: bool,
) -> NavMenuState {
    if disabled {
        return NavMenuState {
            bg: gpui::transparent_black(),
            fg: tokens.color_text_disabled,
        };
    }
    if selected {
        return NavMenuState {
            bg: tokens.control_item_bg_active,
            fg: tokens.color_primary,
        };
    }
    NavMenuState {
        bg: if hover {
            tokens.color_bg_text_hover
        } else {
            gpui::transparent_black()
        },
        fg: tokens.color_text,
    }
}

/// 父级 submenu 标题：后代选中时仅字变主色（subMenuItemSelectedColor），底不变。
pub fn nav_menu_parent_state(
    tokens: &Tokens,
    child_selected: bool,
    hover: bool,
    disabled: bool,
) -> NavMenuState {
    NavMenuState {
        bg: if !disabled && hover {
            tokens.color_bg_text_hover
        } else {
            gpui::transparent_black()
        },
        fg: if disabled {
            tokens.color_text_disabled
        } else if child_selected {
            tokens.color_primary
        } else {
            tokens.color_text
        },
    }
}

// ---- Countdown（antd Statistic.Countdown / StatisticTimer） ----

/// Statistic 度量（`statistic/style/index.js` prepareComponentToken）：
/// 内容字号 fontSizeHeading3、字色 colorTextHeading；标题 fontSize / colorTextDescription，
/// 标题下边距 marginXXS。
pub struct CountdownMetric {
    pub content_font_size: f32,
    pub content_color: Hsla,
    pub title_font_size: f32,
    pub title_color: Hsla,
    pub title_padding_bottom: f32,
    pub line_height: f32,
}

pub fn countdown_metric(tokens: &Tokens) -> CountdownMetric {
    CountdownMetric {
        content_font_size: tokens.font_size_heading3,
        content_color: tokens.color_text_heading,
        title_font_size: tokens.font_size,
        title_color: tokens.color_text_description,
        title_padding_bottom: tokens.margin_xxs,
        line_height: tokens.line_height,
    }
}

// ---- Kbd（antd Typography kbd，typography/style/mixins.js getResetStyles） ----

/// kbd 字面量：圆角 3、下边框 2、底 rgba(150,150,150,0.06)、描边 rgba(100,100,100,0.2)，
/// 不随种子 / 明暗变化（mixins.js 写死）。
pub const KBD_RADIUS: f32 = 3.0;
pub const KBD_BORDER_BOTTOM: f32 = 2.0;

/// em 基准为 kbd 自身字号（fontSize 90%）：paddingInline 0.4em、
/// paddingBlock 0.15em/0.1em、marginInline 0.2em。
/// 行高不取令牌：antd kbd 重置样式不设 lineHeight，实测为字体自然行高（Consolas ≈1.18em）。
pub struct KbdMetric {
    pub font_size: f32,
    pub padding_inline: f32,
    pub padding_block_start: f32,
    pub padding_block_end: f32,
    pub margin_inline: f32,
    pub radius: f32,
    pub border_width: f32,
    pub border_bottom_width: f32,
    pub bg: Hsla,
    pub border_color: Hsla,
    pub fg: Hsla,
}

pub fn kbd_metric(tokens: &Tokens) -> KbdMetric {
    let font_size = tokens.font_size * 0.9;
    KbdMetric {
        font_size,
        padding_inline: font_size * 0.4,
        padding_block_start: font_size * 0.15,
        padding_block_end: font_size * 0.1,
        margin_inline: font_size * 0.2,
        radius: KBD_RADIUS,
        border_width: tokens.line_width,
        border_bottom_width: KBD_BORDER_BOTTOM,
        bg: rgba(0x9696960f).into(),
        border_color: rgba(0x64646433).into(),
        fg: tokens.color_text,
    }
}

// ---- Scrollbar（rc-virtual-list ScrollBar.js；table/tree 的 colorSplit 覆写因变量名失配失效） ----

/// ScrollBar.js 字面量：槽宽 8、滑块圆角 99px（药丸）、MIN_SIZE 20。
pub const SCROLLBAR_SLOT: f32 = 8.0;
pub const SCROLLBAR_THUMB_RADIUS: f32 = 99.0;
pub const SCROLLBAR_THUMB_MIN: f32 = 20.0;

pub struct ScrollbarMetric {
    pub slot: f32,
    pub thumb_radius: f32,
    pub thumb_min: f32,
    pub thumb_color: Hsla,
    pub track_hover_bg: Hsla,
}

/// 滑块底 = --rc-virtual-list-scrollbar-bg（antd tableScrollBg / tree 均覆写为 colorSplit）；
/// 轨道常态透明，hover 才 colorFillQuaternary（table/style/virtual.js）。
pub fn scrollbar_metric(tokens: &Tokens) -> ScrollbarMetric {
    ScrollbarMetric {
        slot: SCROLLBAR_SLOT,
        thumb_radius: SCROLLBAR_THUMB_RADIUS,
        thumb_min: SCROLLBAR_THUMB_MIN,
        // 上游变量名失配：antd 写 --rc-virtual-list-scrollbar-bg，ScrollBar.js 读
        // --vc-virtual-list-scrollbar-bg，覆写失效 → 实际回退 rgba(0,0,0,0.5)。
        thumb_color: rgba(0x00000080).into(),
        track_hover_bg: tokens.color_fill_quaternary,
    }
}

pub fn scrollbar_track_bg(tokens: &Tokens, hovered: bool) -> Hsla {
    if hovered {
        tokens.color_fill_quaternary
    } else {
        gpui::transparent_black()
    }
}

/// 滑块长度：container/scrollRange×container，下限 MIN_SIZE（scrollbarUtil.getSpinSize）。
pub fn scrollbar_thumb_len(track_len: f32, visible: f32, max: f32) -> f32 {
    let total = visible + max;
    let base = if total > 0.0 {
        track_len * (visible / total)
    } else {
        track_len
    };
    base.max(SCROLLBAR_THUMB_MIN).min(track_len)
}

// ---- Stepper（无 antd 横排对应；容器取 InputNumber 令牌，按钮态取通用交互令牌） ----

pub struct StepperMetric {
    pub height: f32,
    pub radius: f32,
    pub border_width: f32,
    pub border_color: Hsla,
    pub bg: Hsla,
    pub disabled_bg: Hsla,
    pub disabled_fg: Hsla,
    pub button_size: f32,
    /// 中间数值盒宽度（本库横向步进器自定；antd 无对应组件）
    pub value_width: f32,
    pub font_size: f32,
    pub fg: Hsla,
}

pub fn stepper_metric(tokens: &Tokens) -> StepperMetric {
    StepperMetric {
        height: tokens.control_height,
        radius: tokens.border_radius,
        border_width: tokens.line_width,
        border_color: tokens.color_border,
        bg: tokens.color_bg_container,
        disabled_bg: tokens.color_bg_container_disabled,
        disabled_fg: tokens.color_text_disabled,
        button_size: tokens.control_height - tokens.line_width * 2.0,
        value_width: 56.0,
        font_size: tokens.font_size,
        fg: tokens.color_text,
    }
}

pub struct StepperState {
    pub bg: Hsla,
    pub fg: Hsla,
}

/// 按钮态：整体禁用或到边界 → 禁用字色无底；active/hover 取通用交互底。
pub fn stepper_button_state(
    tokens: &Tokens,
    disabled: bool,
    at_bound: bool,
    hover: bool,
    active: bool,
) -> StepperState {
    if disabled || at_bound {
        return StepperState {
            bg: gpui::transparent_black(),
            fg: tokens.color_text_disabled,
        };
    }
    StepperState {
        bg: if active {
            tokens.control_item_bg_active
        } else if hover {
            tokens.color_bg_text_hover
        } else {
            gpui::transparent_black()
        },
        fg: tokens.color_text,
    }
}

// ---------------------------------------------------------------------------
// 非 antd 自研控件（antd X 系列或无对应）：尺寸为本库自定，集中在此以便
// 渲染层不出现魔法 px；颜色仍来自 `Tokens`。
// ---------------------------------------------------------------------------

/// 对话气泡度量。
pub struct BubbleMetric {
    pub avatar_size: f32,
    pub max_width: f32,
    pub spinner_size: f32,
}

pub fn bubble_metric(_tokens: &Tokens) -> BubbleMetric {
    BubbleMetric {
        avatar_size: 32.0,
        max_width: 420.0,
        spinner_size: 16.0,
    }
}

/// 代码块度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CodeBlockMetric {
    pub font_size: f32,
    pub line_height: f32,
    /// 右上角动作按钮之间的水平间距
    pub action_gap: f32,
}

pub fn code_block_metric(_tokens: &Tokens) -> CodeBlockMetric {
    CodeBlockMetric {
        font_size: 13.0,
        line_height: 20.0,
        action_gap: 4.0,
    }
}

impl CodeBlockMetric {
    /// 覆盖代码正文字号；额外行距（`line_height - font_size`）保持不变。
    /// 按钮间距 `action_gap` 是控件几何，不跟随字号。
    pub fn with_font_size(mut self, size: f32) -> Self {
        let extra_leading = self.line_height - self.font_size;
        self.font_size = size;
        self.line_height = size + extra_leading;
        self
    }

    /// 跟随 Markdown 正文字号覆盖：按同一比例缩放，而非直接取覆盖值，
    /// 以保住「代码比正文小一档」的既有关系（本库正文 14 / 代码 13，原版 12 / 11）。
    pub fn following_markdown(self, markdown_base: f32, markdown_size: f32) -> Self {
        let size = self.font_size * markdown_size / markdown_base;
        self.with_font_size(size)
    }
}

/// 命令条度量。
///
/// 分两套几何：`framed`（默认，整条自带输入框外框）与 `bare`
/// （宿主画 chrome 底 + 上边线、输入框自己描边）。
pub struct CommandBarMetric {
    pub bar_padding_block: f32,
    pub bar_padding_inline: f32,
    pub bar_gap: f32,
    pub bar_radius: f32,
    /// `bare`：整条行高 / 左右内边距 / 子项间距 / 圆角。
    pub bar_height: f32,
    pub bare_padding_inline: f32,
    pub bare_gap: f32,
    pub bare_radius: f32,
    /// `bare`：输入框描边盒的行高与左右内边距。
    pub input_height: f32,
    pub input_padding_inline: f32,
    /// 提示符字号（`bare`）与动作按钮行高 / 内边距 / 字号。
    pub prompt_font_size: f32,
    pub action_height: f32,
    pub action_padding_inline: f32,
    pub action_font_size: f32,
    /// `bare`：输入框字号（原版 12px 等宽，比主题正文小一档）。
    pub input_font_size: f32,
    /// 浮层（片段 / 历史）尺寸与行内边距。
    pub pop_min_width: f32,
    pub pop_max_height: f32,
    pub pop_padding: f32,
    pub pop_radius: f32,
    pub pop_row_padding_block: f32,
    pub pop_row_padding_inline: f32,
    pub pop_row_font_size: f32,
    /// 历史空态：居中段落，四边等距内边距。
    pub pop_empty_padding: f32,
    /// 浮层底边与条顶边的间距。
    pub pop_gap: f32,
    pub chip_gap: f32,
    pub chip_padding_block: f32,
    pub chip_radius: f32,
    /// 片段面板（原版 SnippetPanel 形态）：定宽定高，自带内边距。
    pub snip_width: f32,
    pub snip_max_height: f32,
    pub snip_padding: f32,
    /// 面板头部：标题 + 「新建」按钮，`padding: 2px 4px 6px`。
    pub snip_head_padding_top: f32,
    pub snip_head_padding_inline: f32,
    pub snip_head_padding_bottom: f32,
    pub snip_title_font_size: f32,
    pub snip_new_height: f32,
    pub snip_new_padding_inline: f32,
    pub snip_new_gap: f32,
    /// 分组之间 4px；组名 `padding: 4px 6px 2px`、10px 三级字。
    pub snip_group_gap: f32,
    pub snip_group_name_padding_top: f32,
    pub snip_group_name_padding_inline: f32,
    pub snip_group_name_padding_bottom: f32,
    pub snip_group_name_font_size: f32,
    /// 片段行：名称 11px + 等宽命令 10px，行高由 5px 上下内边距撑出。
    pub snip_row_padding_block: f32,
    pub snip_row_padding_inline: f32,
    pub snip_row_gap: f32,
    pub snip_name_font_size: f32,
    pub snip_cmd_font_size: f32,
    /// 空态：`padding: 12px 8px`、11px 居中。
    pub snip_empty_padding_block: f32,
    pub snip_empty_padding_inline: f32,
    /// 内联新建/编辑表单（原版 `.snip__form`）：gap6 padding4 mb6 描边圆角盒。
    pub snip_form_gap: f32,
    pub snip_form_padding: f32,
    pub snip_form_margin_bottom: f32,
    pub snip_form_radius: f32,
    /// 命令 TextArea rows=2 的整高（2 行行高 + 上下内边距 + 描边）。
    pub snip_form_command_height: f32,
    /// sendEnter 勾选行：13px 方框 + 11px 标签，gap6。
    pub snip_check_gap: f32,
    pub snip_check_font_size: f32,
    pub snip_check_box: f32,
    pub snip_check_radius: f32,
    pub snip_check_glyph: f32,
    /// 表单脚部：右对齐 gap6；取消/保存 22px 高、11px 字。
    pub snip_foot_gap: f32,
    pub snip_btn_height: f32,
    pub snip_btn_padding_inline: f32,
    pub snip_btn_font_size: f32,
    /// 行悬停才显形的编辑/删除图标槽：22×22、圆角 6。
    pub snip_icon_size: f32,
    pub snip_icon_radius: f32,
}

/// 等宽字体族：命令条输入框、历史行、片段命令列共用。
///
/// 宿主可用 `theme::set_mono_family` 全局覆盖（随包 JetBrains Mono 等）；
/// 未覆盖时退到系统自带等宽字体。
pub fn mono_font_family() -> &'static str {
    crate::theme::mono_family_or(if cfg!(target_os = "windows") {
        "Consolas"
    } else if cfg!(target_os = "macos") {
        "Menlo"
    } else {
        "monospace"
    })
}

/// 输入框排版：宿主覆盖值优先，否则跟令牌字号 + 正文字体。
///
/// 返回 `(字号, 等宽族)`；族为 `None` 时不覆盖字体。命令条这类
/// 「比正文小一档且等宽」的输入位走覆盖分支。
pub fn input_typography(
    token_font_size: f32,
    font_size: Option<f32>,
    monospace: bool,
) -> (f32, Option<&'static str>) {
    (
        font_size.unwrap_or(token_font_size),
        monospace.then(mono_font_family),
    )
}

pub fn command_bar_metric(_tokens: &Tokens) -> CommandBarMetric {
    CommandBarMetric {
        bar_padding_block: 7.0,
        bar_padding_inline: 12.0,
        bar_gap: 8.0,
        bar_radius: 6.0,
        bar_height: 34.0,
        bare_padding_inline: 10.0,
        bare_gap: 6.0,
        bare_radius: 6.0,
        input_height: 24.0,
        input_padding_inline: 8.0,
        prompt_font_size: 12.0,
        action_height: 22.0,
        action_padding_inline: 8.0,
        action_font_size: 11.0,
        input_font_size: 12.0,
        pop_min_width: 220.0,
        pop_max_height: 240.0,
        pop_padding: 4.0,
        pop_radius: 8.0,
        pop_row_padding_block: 5.0,
        pop_row_padding_inline: 8.0,
        pop_row_font_size: 11.0,
        pop_empty_padding: 8.0,
        pop_gap: 2.0,
        chip_gap: 6.0,
        chip_padding_block: 2.0,
        chip_radius: 4.0,
        snip_width: 320.0,
        snip_max_height: 340.0,
        snip_padding: 6.0,
        snip_head_padding_top: 2.0,
        snip_head_padding_inline: 4.0,
        snip_head_padding_bottom: 6.0,
        snip_title_font_size: 11.0,
        snip_new_height: 20.0,
        snip_new_padding_inline: 6.0,
        snip_new_gap: 3.0,
        snip_group_gap: 4.0,
        snip_group_name_padding_top: 4.0,
        snip_group_name_padding_inline: 6.0,
        snip_group_name_padding_bottom: 2.0,
        snip_group_name_font_size: 10.0,
        snip_row_padding_block: 5.0,
        snip_row_padding_inline: 6.0,
        snip_row_gap: 6.0,
        snip_name_font_size: 11.0,
        snip_cmd_font_size: 10.0,
        snip_empty_padding_block: 12.0,
        snip_empty_padding_inline: 8.0,
        snip_form_gap: 6.0,
        snip_form_padding: 4.0,
        snip_form_margin_bottom: 6.0,
        snip_form_radius: 8.0,
        snip_form_command_height: 54.0,
        snip_check_gap: 6.0,
        snip_check_font_size: 11.0,
        snip_check_box: 13.0,
        snip_check_radius: 2.0,
        snip_check_glyph: 9.0,
        snip_foot_gap: 6.0,
        snip_btn_height: 22.0,
        snip_btn_padding_inline: 10.0,
        snip_btn_font_size: 11.0,
        snip_icon_size: 22.0,
        snip_icon_radius: 6.0,
    }
}

/// 命令条外观色。
pub struct CommandBarStyle {
    /// 整条底色：`framed` 走容器底；`bare` 不画底，由宿主铺自己的 chrome。
    pub bar_bg: Option<Hsla>,
    /// 整条描边：仅 `framed` 有（聚焦主色 / 常态 `colorBorder`）。
    pub bar_border: Option<Hsla>,
    /// 输入框描边：仅 `bare` 有，聚焦主色。
    pub input_border: Option<Hsla>,
    pub input_bg: Hsla,
    pub prompt_fg: Hsla,
    pub hint_fg: Hsla,
    /// 浮层行悬停：底色 + 文字色。
    pub pop_row_hover_bg: Hsla,
    pub pop_row_hover_fg: Hsla,
    /// 「清空历史」行悬停换成错误色。
    pub pop_danger_hover_fg: Hsla,
    pub pop_empty_fg: Hsla,
    /// 片段面板：标题与组名降到三级字，名称用正文色，命令列三级字。
    /// 悬停只换底色（`pop_row_hover_bg`），文字色不跟随——与历史行不同。
    pub snip_title_fg: Hsla,
    pub snip_group_fg: Hsla,
    pub snip_name_fg: Hsla,
    pub snip_cmd_fg: Hsla,
    /// 内联表单：盒底比浮层底深一档（原版 `--ns-bg`），勾选标签二级字。
    pub snip_form_bg: Hsla,
    pub snip_check_fg: Hsla,
    pub snip_check_box_fg: Hsla,
    /// 取消/保存按钮：常态二级字 + 描边；悬停换正文色 + 三级字描边（主色描边款也跟随）。
    pub snip_btn_fg: Hsla,
    pub snip_btn_hover_fg: Hsla,
    pub snip_btn_hover_border: Hsla,
    pub snip_save_fg: Hsla,
    /// 行悬停图标：常态三级字；编辑悬停正文色、删除悬停错误色。
    pub snip_icon_fg: Hsla,
    pub snip_icon_hover_fg: Hsla,
    pub snip_icon_danger_fg: Hsla,
}

/// 命令条外观：`framed` = 控件自带整条外框；`bare` = 底色与上边线都归宿主，
/// 描边收到输入框上（嵌在终端面板下沿的命令条形态）。
pub fn command_bar_style(tokens: &Tokens, framed: bool, focused: bool) -> CommandBarStyle {
    CommandBarStyle {
        bar_bg: framed.then_some(tokens.color_bg_container),
        bar_border: framed.then(|| {
            if focused {
                tokens.color_primary
            } else {
                tokens.color_border
            }
        }),
        input_border: (!framed).then(|| {
            if focused {
                tokens.color_primary
            } else {
                tokens.color_border
            }
        }),
        input_bg: tokens.color_bg_container,
        prompt_fg: tokens.color_primary,
        hint_fg: tokens.color_text_secondary,
        pop_row_hover_bg: tokens.color_primary_bg,
        pop_row_hover_fg: tokens.color_primary,
        pop_danger_hover_fg: tokens.color_error,
        pop_empty_fg: tokens.color_text_tertiary,
        snip_title_fg: tokens.color_text_tertiary,
        snip_group_fg: tokens.color_text_tertiary,
        snip_name_fg: tokens.color_text,
        snip_cmd_fg: tokens.color_text_tertiary,
        snip_form_bg: tokens.color_bg_layout,
        snip_check_fg: tokens.color_text_secondary,
        snip_check_box_fg: tokens.color_white,
        snip_btn_fg: tokens.color_text_secondary,
        snip_btn_hover_fg: tokens.color_text,
        snip_btn_hover_border: tokens.color_text_tertiary,
        snip_save_fg: tokens.color_primary,
        snip_icon_fg: tokens.color_text_tertiary,
        snip_icon_hover_fg: tokens.color_text,
        snip_icon_danger_fg: tokens.color_error,
    }
}

/// 命令条动作按钮的四种形态：普通 / 面板展开 / 主动作（执行）/ 禁用。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBarAction {
    Default,
    /// 对应浮层已展开（片段 / 历史）。
    Active,
    /// 「执行」——实心底，是这一行唯一的主动作。
    Go,
    Disabled,
    GoDisabled,
}

pub struct CommandBarActionStyle {
    pub bg: Option<Hsla>,
    pub border: Hsla,
    pub fg: Hsla,
    pub hover_bg: Option<Hsla>,
    pub hover_border: Hsla,
    pub hover_fg: Hsla,
}

/// 命令条动作按钮三态色。禁用态不画底、只把文字降到 `colorTextTertiary`；
/// 「执行」的悬停取 `colorPrimaryActive`（比常态更深一档），与其余描边按钮相反。
pub fn command_bar_action_style(tokens: &Tokens, kind: CommandBarAction) -> CommandBarActionStyle {
    use CommandBarAction as A;
    match kind {
        A::Default => CommandBarActionStyle {
            bg: None,
            border: tokens.color_border,
            fg: tokens.color_text_secondary,
            hover_bg: Some(tokens.color_primary_bg),
            hover_border: tokens.color_primary,
            hover_fg: tokens.color_primary,
        },
        A::Active => CommandBarActionStyle {
            bg: Some(tokens.color_primary_bg),
            border: tokens.color_primary,
            fg: tokens.color_primary,
            hover_bg: Some(tokens.color_primary_bg),
            hover_border: tokens.color_primary,
            hover_fg: tokens.color_primary,
        },
        A::Go => CommandBarActionStyle {
            bg: Some(tokens.color_primary),
            border: tokens.color_primary,
            fg: tokens.color_text_light_solid,
            hover_bg: Some(tokens.color_primary_active),
            hover_border: tokens.color_primary_active,
            hover_fg: tokens.color_text_light_solid,
        },
        A::Disabled | A::GoDisabled => CommandBarActionStyle {
            bg: None,
            border: tokens.color_border,
            fg: tokens.color_text_tertiary,
            hover_bg: None,
            hover_border: tokens.color_border,
            hover_fg: tokens.color_text_tertiary,
        },
    }
}

/// 命令面板度量。
pub struct CommandPaletteMetric {
    pub width: f32,
    pub margin_top: f32,
    pub header_padding_block: f32,
    pub list_max_height: f32,
    pub item_padding_block: f32,
    pub mask: Hsla,
}

pub fn command_palette_metric(tokens: &Tokens) -> CommandPaletteMetric {
    CommandPaletteMetric {
        width: 480.0,
        margin_top: 80.0,
        header_padding_block: 12.0,
        list_max_height: 320.0,
        item_padding_block: 8.0,
        mask: tokens.color_bg_mask,
    }
}

/// 拖放区度量。
pub struct DropZoneMetric {
    pub icon_size: f32,
}

pub fn drop_zone_metric(_tokens: &Tokens) -> DropZoneMetric {
    DropZoneMetric { icon_size: 24.0 }
}

/// FPS 面板度量。
pub struct FpsMetric {
    pub value_font_size: f32,
    pub unit_padding_bottom: f32,
    pub graph_width: f32,
    pub graph_height: f32,
    pub dot_size: f32,
}

pub fn fps_metric(_tokens: &Tokens) -> FpsMetric {
    FpsMetric {
        value_font_size: 30.0,
        unit_padding_bottom: 4.0,
        graph_width: 180.0,
        graph_height: 44.0,
        dot_size: 8.0,
    }
}

/// 提示卡片度量。
pub struct PromptsMetric {
    pub card_width: f32,
}

pub fn prompts_metric(_tokens: &Tokens) -> PromptsMetric {
    PromptsMetric { card_width: 200.0 }
}

/// 发送框度量。
pub struct SenderMetric {
    pub input_height: f32,
}

pub fn sender_metric(_tokens: &Tokens) -> SenderMetric {
    SenderMetric { input_height: 56.0 }
}

/// 发送按钮三态色。常态实心底 + 悬停加深一档（`colorPrimaryHover`），与 `Button` 主动作
/// 同一口径；加载态退到中性底、不响应悬停。
pub struct SenderSendStyle {
    pub bg: Hsla,
    pub fg: Hsla,
    pub hover_bg: Option<Hsla>,
}

pub fn sender_send_style(tokens: &Tokens, loading: bool) -> SenderSendStyle {
    if loading {
        SenderSendStyle {
            bg: tokens.control_item_bg_hover,
            fg: tokens.color_text_disabled,
            hover_bg: None,
        }
    } else {
        SenderSendStyle {
            bg: tokens.color_primary,
            fg: tokens.color_text_light_solid,
            hover_bg: Some(tokens.color_primary_hover),
        }
    }
}

/// 思维链度量。
pub struct ThoughtChainMetric {
    pub icon_size: f32,
    pub icon_font_size: f32,
    pub indicator_width: f32,
    pub line_width: f32,
}

pub fn thought_chain_metric(_tokens: &Tokens) -> ThoughtChainMetric {
    ThoughtChainMetric {
        icon_size: 18.0,
        icon_font_size: 11.0,
        indicator_width: 20.0,
        line_width: 2.0,
    }
}

/// 思维链步骤外观：Pending 只有主色描边；Success/Error 实心底 + 白色字形。
pub struct ThoughtStatusStyle {
    pub border: Option<Hsla>,
    pub fill: Option<Hsla>,
    pub fg: Hsla,
    pub glyph: Option<&'static str>,
}

pub fn thought_status_style(tokens: &Tokens, status: crate::ThoughtStatus) -> ThoughtStatusStyle {
    use crate::ThoughtStatus;
    match status {
        ThoughtStatus::Pending => ThoughtStatusStyle {
            border: Some(tokens.color_primary),
            fill: None,
            fg: tokens.color_text_light_solid,
            glyph: None,
        },
        ThoughtStatus::Success => ThoughtStatusStyle {
            border: None,
            fill: Some(tokens.color_success),
            fg: tokens.color_text_light_solid,
            glyph: Some("✓"),
        },
        ThoughtStatus::Error => ThoughtStatusStyle {
            border: None,
            fill: Some(tokens.color_error),
            fg: tokens.color_text_light_solid,
            glyph: Some("✕"),
        },
    }
}

/// 标题栏度量。
pub struct TitleBarMetric {
    pub control_glyph_size: f32,
}

pub fn title_bar_metric(_tokens: &Tokens) -> TitleBarMetric {
    TitleBarMetric {
        control_glyph_size: 14.0,
    }
}

/// 视频播放器度量。`backdrop` 是画面外的信箱底，固定深色以保证任何主题下边框不抢画面。
pub struct VideoMetric {
    pub height: f32,
    pub radius: f32,
    pub spinner_size: f32,
    pub backdrop: Hsla,
}

pub fn video_metric(_tokens: &Tokens) -> VideoMetric {
    VideoMetric {
        height: 220.0,
        radius: 8.0,
        spinner_size: 24.0,
        backdrop: gpui::rgb(0x14141a).into(),
    }
}

/// 欢迎面板度量。
pub struct WelcomeMetric {
    pub icon_size: f32,
    pub icon_font_size: f32,
}

pub fn welcome_metric(_tokens: &Tokens) -> WelcomeMetric {
    WelcomeMetric {
        icon_size: 56.0,
        icon_font_size: 28.0,
    }
}

/// 二维码度量：白底黑模块是扫描语义，固定取 `colorWhite` / `colorText`。
pub struct QrCodeMetric {
    pub default_size: f32,
    pub bg: Hsla,
    pub fg: Hsla,
}

pub fn qrcode_metric(tokens: &Tokens) -> QrCodeMetric {
    QrCodeMetric {
        default_size: 160.0,
        bg: tokens.color_white,
        fg: tokens.color_text,
    }
}

/// 上传列表度量。
pub struct UploadListMetric {
    pub bar_height: f32,
    pub row_padding_block: f32,
}

pub fn upload_list_metric(_tokens: &Tokens) -> UploadListMetric {
    UploadListMetric {
        bar_height: 3.0,
        row_padding_block: 6.0,
    }
}

/// 上传状态外观：上传中主色 / 完成成功色 / 失败错误色；失败时文件名也用错误色。
pub struct UploadStatusStyle {
    pub icon: &'static str,
    pub color: Hsla,
    pub label: &'static str,
    pub name: Hsla,
}

pub fn upload_status_style(tokens: &Tokens, status: crate::UploadStatus) -> UploadStatusStyle {
    use crate::UploadStatus;
    match status {
        UploadStatus::Uploading => UploadStatusStyle {
            icon: "📄",
            color: tokens.color_primary,
            label: "上传中",
            name: tokens.color_text,
        },
        UploadStatus::Done => UploadStatusStyle {
            icon: "✓",
            color: tokens.color_success,
            label: "已完成",
            name: tokens.color_text,
        },
        UploadStatus::Error => UploadStatusStyle {
            icon: "✕",
            color: tokens.color_error,
            label: "失败",
            name: tokens.color_error,
        },
    }
}

/// 传输任务度量。
pub struct TransferManagerMetric {
    pub meta_gap: f32,
    pub empty_padding_block: f32,
    pub tag_padding_inline: f32,
    pub tag_padding_block: f32,
    pub actions_padding_top: f32,
    pub direction_icon_size: f32,
    pub action_button_size: f32,
    pub bar_height: f32,
}

pub fn transfer_manager_metric(_tokens: &Tokens) -> TransferManagerMetric {
    TransferManagerMetric {
        meta_gap: 2.0,
        empty_padding_block: 28.0,
        tag_padding_inline: 6.0,
        tag_padding_block: 1.0,
        actions_padding_top: 2.0,
        direction_icon_size: 26.0,
        action_button_size: 22.0,
        bar_height: 4.0,
    }
}

/// 传输状态外观（字色 + 软底色）。
pub struct TransferStatusStyle {
    pub fg: Hsla,
    pub bg: Hsla,
}

pub fn transfer_status_style(
    tokens: &Tokens,
    status: crate::TransferStatus,
) -> TransferStatusStyle {
    use crate::TransferStatus;
    let (fg, bg) = match status {
        TransferStatus::Queued => (tokens.color_text_secondary, tokens.control_item_bg_hover),
        TransferStatus::Transferring => (tokens.color_primary, tokens.color_primary_bg),
        TransferStatus::Paused => (tokens.color_warning, tokens.color_warning_bg),
        TransferStatus::Done => (tokens.color_success, tokens.color_success_bg),
        TransferStatus::Failed => (tokens.color_error, tokens.color_error_bg),
        TransferStatus::Canceled => (tokens.color_text_disabled, tokens.control_item_bg_hover),
    };
    TransferStatusStyle { fg, bg }
}

/// 列表度量（antd List：条目内边距 paddingSM / padding，头像与页脚为本库自定）。
pub struct ListMetric {
    pub avatar_size: f32,
    pub footer_padding_block: f32,
}

pub fn list_metric(_tokens: &Tokens) -> ListMetric {
    ListMetric {
        avatar_size: 32.0,
        footer_padding_block: 10.0,
    }
}

/// 水印度量：默认值对齐 antdv-next Watermark（字号 fontSizeLG、颜色 colorFill、
/// 间距 100×100）；旋转 -22° 未实现，见偏差。
pub struct WatermarkMetric {
    pub font_size: f32,
    pub color: Hsla,
    pub gap_x: f32,
    pub gap_y: f32,
}

pub fn watermark_metric(tokens: &Tokens) -> WatermarkMetric {
    WatermarkMetric {
        font_size: tokens.font_size_lg,
        color: tokens.color_fill,
        gap_x: 100.0,
        gap_y: 100.0,
    }
}

/// Markdown 渲染度量：标题字号取 antd 的 heading 令牌，其余间距对齐正文栅格。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarkdownMetric {
    pub font_size: f32,
    pub line_height: f32,
    /// h1..h6 字号（h6 与正文同号）
    pub heading_sizes: [f32; 6],
    /// h1..h6 行高：保持正文的额外行距不随字号放大
    pub heading_line_heights: [f32; 6],
    /// 块与块之间的垂直间距
    pub block_gap: f32,
    /// 列表项内部块、引用内部块的间距
    pub list_gap: f32,
    /// 列表符号列宽
    pub bullet_width: f32,
    /// 引用块左侧内边距
    pub quote_padding: f32,
    /// 引用块左侧竖线宽度
    pub quote_border: f32,
    /// 分隔线粗细
    pub rule_thickness: f32,
    /// 链接下划线 / 删除线粗细
    pub link_thickness: f32,
    /// 表格单元格内边距
    pub table_cell_padding: f32,
    /// 表格边框宽度
    pub table_border: f32,
    /// 任务列表勾选框边长（antd checkbox 为 16px，取 font_size_lg）
    pub task_box_size: f32,
    /// 任务列表勾号字号
    pub task_check_size: f32,
}

pub fn markdown_metric(tokens: &Tokens) -> MarkdownMetric {
    let extra_leading = tokens.font_height - tokens.font_size;
    let heading_sizes = [
        tokens.font_size_heading1,
        tokens.font_size_heading2,
        tokens.font_size_heading3,
        tokens.font_size_heading4,
        tokens.font_size_heading5,
        tokens.font_size,
    ];
    MarkdownMetric {
        font_size: tokens.font_size,
        line_height: tokens.font_height,
        heading_sizes,
        heading_line_heights: heading_sizes.map(|size| size + extra_leading),
        block_gap: tokens.margin_sm,
        list_gap: tokens.margin_xxs,
        bullet_width: tokens.size_ms,
        quote_padding: tokens.padding_sm,
        quote_border: tokens.line_width_bold,
        rule_thickness: tokens.line_width,
        link_thickness: tokens.line_width,
        table_cell_padding: tokens.padding_sm,
        table_border: tokens.line_width,
        task_box_size: tokens.font_size_lg,
        task_check_size: tokens.font_size_sm,
    }
}

/// 聊天面板用的紧凑 Markdown 度量：正文不变，标题压到气泡尺度、块间距收紧。
pub fn markdown_compact_metric(tokens: &Tokens) -> MarkdownMetric {
    let mut m = markdown_metric(tokens);
    let fs = m.font_size;
    let extra_leading = m.line_height - fs;
    m.heading_sizes = [fs * 1.45, fs * 1.3, fs * 1.18, fs * 1.08, fs, fs];
    m.heading_line_heights = m.heading_sizes.map(|size| size + extra_leading);
    m.block_gap = tokens.margin_xs;
    m.list_gap = tokens.margin_xxs;
    m.table_cell_padding = tokens.padding_xs;
    m
}

impl MarkdownMetric {
    /// 覆盖正文字号：标题按同比例缩放，额外行距（`line_height - font_size`）保持不变，
    /// 与 `markdown_compact_metric` 的既有约定一致。
    /// 块间距 / 列表符号列宽 / 勾选框等控件几何不跟随字号。
    pub fn with_font_size(mut self, size: f32) -> Self {
        let extra_leading = self.line_height - self.font_size;
        let ratio = size / self.font_size;
        self.font_size = size;
        self.line_height = size + extra_leading;
        self.heading_sizes = self.heading_sizes.map(|s| s * ratio);
        self.heading_line_heights = self.heading_sizes.map(|s| s + extra_leading);
        self
    }
}

/// 终端光标度量：bar / underline 的厚度。block 直接用单元格宽高，无需令牌。
/// 取 `line_width_bold`（默认 seed 下 = 2），与既有 2px 竖条光标一致。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerminalMetric {
    pub caret_width: f32,
}

impl TerminalMetric {
    pub fn resolve(tokens: &Tokens) -> Self {
        Self {
            caret_width: tokens.line_width_bold,
        }
    }
}
