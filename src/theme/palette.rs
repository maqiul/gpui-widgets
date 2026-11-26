//! `@ant-design/fast-color` + `@ant-design/colors` 的 Rust 移植。
//!
//! antdv-next 的全部色值都是从这里派生的：种子色 → `generate()` 十级色板 → 别名令牌。
//! 因此这里的取整顺序必须和 JS 逐行对齐（`Math.round` 的调用位置、hsv/hsl 两套饱和度的混用），
//! 否则换个品牌主色就会和 antdv-next 差出 1 个色阶。
//!
//! 颜色以 0~255 的 RGB 分量存储，而不是 gpui 的 Hsla：派生过程中间要反复做 hsv/hsl 换算，
//! 每次换算都会取整，用 Hsla 承载会把误差累积到最终色值上。

use gpui::{Hsla, Rgba};

/// JS 的 `Math.round`：正数半数向上取整
fn round(v: f32) -> f32 {
    v.round()
}

/// 一个 rgb + alpha 的颜色，分量沿用 fast-color 的取值范围（rgb 为 0~255，alpha 为 0~1）
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// 从 `0xRRGGBB` 构造不透明颜色
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xff) as f32,
            g: ((hex >> 8) & 0xff) as f32,
            b: (hex & 0xff) as f32,
            a: 1.0,
        }
    }

    /// 带 alpha 的构造，对应 fast-color 的 `setA`
    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    pub fn rgba(self) -> Rgba {
        Rgba {
            r: self.r / 255.0,
            g: self.g / 255.0,
            b: self.b / 255.0,
            a: self.a,
        }
    }

    pub fn hsla(self) -> Hsla {
        self.rgba().into()
    }

    fn max(&self) -> f32 {
        self.r.max(self.g).max(self.b)
    }

    fn min(&self) -> f32 {
        self.r.min(self.g).min(self.b)
    }

    /// 色相，整数（fast-color 的 `getHue` 会 round）
    pub fn hue(&self) -> f32 {
        let max = self.max();
        let min = self.min();
        let delta = max - min;
        if delta == 0.0 {
            return 0.0;
        }
        let v = if self.r == max {
            (self.g - self.b) / delta + if self.g < self.b { 6.0 } else { 0.0 }
        } else if self.g == max {
            (self.b - self.r) / delta + 2.0
        } else {
            (self.r - self.g) / delta + 4.0
        };
        round(60.0 * v)
    }

    /// hsv 饱和度 0~1
    pub fn hsv_s(&self) -> f32 {
        let max = self.max();
        let delta = max - self.min();
        if delta == 0.0 || max == 0.0 {
            0.0
        } else {
            delta / max
        }
    }

    /// hsv 明度 0~1
    pub fn hsv_v(&self) -> f32 {
        self.max() / 255.0
    }

    /// hsl 亮度 0~1
    pub fn hsl_l(&self) -> f32 {
        (self.max() + self.min()) / 510.0
    }

    /// hsv → rgb，取整位置与 fast-color `fromHsv` 一致
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let h = ((h % 360.0) + 360.0) % 360.0;
        let vv = round(v * 255.0);
        let mut out = Self {
            r: vv,
            g: vv,
            b: vv,
            a: 1.0,
        };
        if s <= 0.0 {
            return out;
        }
        let hh = h / 60.0;
        let i = hh.floor();
        let ff = hh - i;
        let p = round(v * (1.0 - s) * 255.0);
        let q = round(v * (1.0 - s * ff) * 255.0);
        let t = round(v * (1.0 - s * (1.0 - ff)) * 255.0);
        match i as i32 {
            0 => {
                out.g = t;
                out.b = p;
            }
            1 => {
                out.r = q;
                out.b = p;
            }
            2 => {
                out.r = p;
                out.b = t;
            }
            3 => {
                out.r = p;
                out.g = q;
            }
            4 => {
                out.r = t;
                out.g = p;
            }
            _ => {
                out.g = p;
                out.b = q;
            }
        }
        out
    }

    /// hsl → rgb，取整位置与 fast-color `fromHsl` 一致
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let h = ((h % 360.0) + 360.0) % 360.0;
        if s <= 0.0 {
            let rgb = round(l * 255.0);
            return Self {
                r: rgb,
                g: rgb,
                b: rgb,
                a: 1.0,
            };
        }
        let hue_prime = h / 60.0;
        let chroma = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let second = chroma * (1.0 - (hue_prime % 2.0 - 1.0).abs());
        let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);
        if hue_prime >= 0.0 && hue_prime < 1.0 {
            r = chroma;
            g = second;
        } else if hue_prime < 2.0 {
            r = second;
            g = chroma;
        } else if hue_prime < 3.0 {
            g = chroma;
            b = second;
        } else if hue_prime < 4.0 {
            g = second;
            b = chroma;
        } else if hue_prime < 5.0 {
            r = second;
            b = chroma;
        } else {
            r = chroma;
            b = second;
        }
        let m = l - chroma / 2.0;
        Self {
            r: round((r + m) * 255.0),
            g: round((g + m) * 255.0),
            b: round((b + m) * 255.0),
            a: 1.0,
        }
    }

    /// 按 amount（0~100）向 other 靠拢，对应 fast-color `mix`
    pub fn mix(&self, other: Color, amount: f32) -> Color {
        let p = amount / 100.0;
        let calc = |a: f32, b: f32| (b - a) * p + a;
        Color {
            r: round(calc(self.r, other.r)),
            g: round(calc(self.g, other.g)),
            b: round(calc(self.b, other.b)),
            a: round(calc(self.a, other.a) * 100.0) / 100.0,
        }
    }

    /// 变暗 amount 个亮度百分点，对应 fast-color `darken`。
    /// 注意：fast-color 在这里取的是 **hsv** 饱和度却当作 **hsl** 饱和度用，
    /// 这是它自己的历史包袱，ant-design 的 `colorBorder` / `colorBgLayout` 等实色都靠它，
    /// 所以必须照抄，不能“修正”。
    pub fn darken(&self, amount: f32) -> Color {
        let l = (self.hsl_l() - amount / 100.0).max(0.0);
        Color::from_hsl(self.hue(), self.hsv_s(), l)
    }

    /// 与白色混和提亮，对应 fast-color `lighten`（同 darken 的饱和度包袱）
    pub fn lighten(&self, amount: f32) -> Color {
        let l = (self.hsl_l() + amount / 100.0).min(1.0);
        Color::from_hsl(self.hue(), self.hsv_s(), l)
    }
}

impl From<Hsla> for Color {
    fn from(c: Hsla) -> Self {
        Color::from(c.to_rgb())
    }
}

impl From<Rgba> for Color {
    fn from(c: Rgba) -> Self {
        Color {
            r: c.r * 255.0,
            g: c.g * 255.0,
            b: c.b * 255.0,
            a: c.a,
        }
    }
}

impl From<u32> for Color {
    fn from(hex: u32) -> Self {
        Color::from_hex(hex)
    }
}

const HUE_STEP: f32 = 2.0;
const SATURATION_STEP: f32 = 0.16;
const SATURATION_STEP_2: f32 = 0.05;
const BRIGHTNESS_STEP_1: f32 = 0.05;
const BRIGHTNESS_STEP_2: f32 = 0.15;
const LIGHT_COLOR_COUNT: f32 = 5.0;
const DARK_COLOR_COUNT: f32 = 4.0;

/// 深色主题色板映射表：`generate(color, { theme: 'dark' })` 用的 (色板下标, 混入比例)
const DARK_COLOR_MAP: [(usize, f32); 10] = [
    (7, 15.0),
    (6, 25.0),
    (5, 30.0),
    (5, 45.0),
    (5, 65.0),
    (5, 85.0),
    (4, 90.0),
    (3, 95.0),
    (2, 97.0),
    (1, 98.0),
];

fn get_hue(hsv_h: f32, i: f32, light: bool) -> f32 {
    let h = hsv_h.round();
    let mut hue = if h >= 60.0 && h <= 240.0 {
        if light {
            h - HUE_STEP * i
        } else {
            h + HUE_STEP * i
        }
    } else if light {
        h + HUE_STEP * i
    } else {
        h - HUE_STEP * i
    };
    if hue < 0.0 {
        hue += 360.0;
    } else if hue >= 360.0 {
        hue -= 360.0;
    }
    hue
}

fn get_saturation(base: Color, i: f32, light: bool) -> f32 {
    // 灰色不参与饱和度阶梯
    if base.hue() == 0.0 && base.hsv_s() == 0.0 {
        return base.hsv_s();
    }
    let mut s = if light {
        base.hsv_s() - SATURATION_STEP * i
    } else if i == DARK_COLOR_COUNT {
        base.hsv_s() + SATURATION_STEP
    } else {
        base.hsv_s() + SATURATION_STEP_2 * i
    };
    if s > 1.0 {
        s = 1.0;
    }
    if light && i == LIGHT_COLOR_COUNT && s > 0.1 {
        s = 0.1;
    }
    if s < 0.06 {
        s = 0.06;
    }
    (s * 100.0).round() / 100.0
}

fn get_value(base: Color, i: f32, light: bool) -> f32 {
    let v = if light {
        base.hsv_v() + BRIGHTNESS_STEP_1 * i
    } else {
        base.hsv_v() - BRIGHTNESS_STEP_2 * i
    };
    (v.clamp(0.0, 1.0) * 100.0).round() / 100.0
}

/// `generate(color)`：以 `base` 为第 6 档（下标 5）的十级色板，1~5 档更浅、7~10 档更深。
fn palette_steps(base: Color) -> [Color; 10] {
    let mut out = [base; 10];
    let mut i = LIGHT_COLOR_COUNT;
    let mut slot = 0usize;
    while i > 0.0 {
        out[slot] = Color::from_hsv(
            get_hue(base.hue(), i, true),
            get_saturation(base, i, true),
            get_value(base, i, true),
        );
        slot += 1;
        i -= 1.0;
    }
    out[5] = base;
    let mut i = 1.0;
    while i <= DARK_COLOR_COUNT {
        out[(5.0 + i) as usize] = Color::from_hsv(
            get_hue(base.hue(), i, false),
            get_saturation(base, i, false),
            get_value(base, i, false),
        );
        i += 1.0;
    }
    out
}

/// `generate(color, { theme: 'dark', backgroundColor })`
fn palette_steps_dark(base: Color, background: Color) -> [Color; 10] {
    let steps = palette_steps(base);
    DARK_COLOR_MAP.map(|(index, amount)| background.mix(steps[index], amount))
}

/// antd 别名取色用的 1~10 档色板（返回下标 0~9 对应 1~10 档）。
///
/// 浅色：`{1..7: steps[0..6], 8: steps[4], 9: steps[5], 10: steps[6]}`
/// 深色：`{1..4: steps[0..3], 5: steps[6], 6: steps[5], 7: steps[4], 8: steps[6], 9: steps[5], 10: steps[4]}`
pub fn palette(base: Color, dark: bool, background: Color) -> [Color; 10] {
    let steps = if dark {
        palette_steps_dark(base, background)
    } else {
        palette_steps(base)
    };
    if dark {
        [
            steps[0], steps[1], steps[2], steps[3], steps[6], steps[5], steps[4], steps[6],
            steps[5], steps[4],
        ]
    } else {
        [
            steps[0], steps[1], steps[2], steps[3], steps[4], steps[5], steps[6], steps[4],
            steps[5], steps[6],
        ]
    }
}

/// `util/getAlphaColor.ts`：反解「前景色叠在背景色上需要多少 alpha」。
/// 用于 `colorSplit` / `controlOutline` / `colorErrorOutline` 这类「实色转半透明」的令牌。
pub fn alpha_over(front: Color, background: Color) -> Color {
    if front.a < 1.0 {
        return front;
    }
    for i in 1..=100 {
        let fa = (i as f32 / 100.0 * 100.0).round() / 100.0;
        let calc = |f: f32, b: f32| round((f - b * (1.0 - fa)) / fa);
        let (r, g, b) = (
            calc(front.r, background.r),
            calc(front.g, background.g),
            calc(front.b, background.b),
        );
        let stable = |v: f32| v >= 0.0 && v <= 255.0;
        if stable(r) && stable(g) && stable(b) {
            return Color { r, g, b, a: fa };
        }
    }
    Color {
        r: front.r,
        g: front.g,
        b: front.b,
        a: 1.0,
    }
}

/// `themes/shared/genColorMapToken` 里 errorBgFilledHover 的构造：1 档与 3 档各混一半。
pub fn error_bg_filled_hover(palette: &[Color; 10]) -> Color {
    palette[0].mix(palette[2], 50.0)
}
