//! 二维码：纯 Rust 编码 + GPU 网格绘制（对应 antd QRCode）。
//!
//! 支持 byte 模式、ECC L、version 1-6（最长 134 字节，超出截断）、固定 mask 0。
//! 没有外部依赖，适合 URL / 短文本场景。

use gpui::{
    App, Bounds, Element, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels, Role,
    SharedString, Style, Window, fill, point, px, size,
};

use crate::{a11y::Semantics, theme};

/// 二维码。
///
/// ```ignore
/// QRCode::new("https://example.com").size(px(160.))
/// ```
pub struct QRCode {
    data: Vec<u8>,
    size: Pixels,
}

impl QRCode {
    pub fn new(data: impl AsRef<str>) -> Self {
        let mut bytes = data.as_ref().as_bytes().to_vec();
        bytes.truncate(134);
        Self {
            data: bytes,
            size: px(theme::with(crate::theme::metric::qrcode_metric).default_size),
        }
    }

    /// 渲染边长（默认 160px）
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }
}

impl IntoElement for QRCode {
    type Element = QrElement;

    fn into_element(self) -> Self::Element {
        let text = String::from_utf8_lossy(&self.data).into_owned();
        let semantics = Semantics::new(Role::Image, SharedString::from(format!("二维码：{text}")))
            .id(format!("qrcode-{text}"))
            .value(SharedString::from(text));
        QrElement {
            matrix: encode(&self.data),
            size: self.size,
            semantics,
        }
    }
}

/// 二维码绘制元素
pub struct QrElement {
    matrix: Vec<Vec<bool>>,
    size: Pixels,
    semantics: Semantics,
}

impl Element for QrElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        Some(self.semantics.element_id())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn a11y_role(&self) -> Option<Role> {
        Some(Role::Image)
    }

    fn write_a11y_info(&self, node: &mut gpui::accesskit::Node) {
        self.semantics.write(node);
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = self.size.into();
        style.size.height = self.size.into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let n = self.matrix.len();
        if n == 0 {
            return;
        }
        // 白底（含 4 模块静区）
        let m = theme::with(crate::theme::metric::qrcode_metric);
        window.paint_quad(fill(bounds, m.bg));

        let module = f32::from(self.size) / (n as f32 + 8.);
        let module = px(module);
        let offset = module * 4.;
        let dark = m.fg;
        for (row, line) in self.matrix.iter().enumerate() {
            for (col, &on) in line.iter().enumerate() {
                if !on {
                    continue;
                }
                let cell = Bounds::new(
                    point(
                        bounds.left() + offset + module * col as f32,
                        bounds.top() + offset + module * row as f32,
                    ),
                    size(module, module),
                );
                window.paint_quad(fill(cell, dark));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 编码实现（byte 模式 / ECC L / version 1-6 / mask 0）
// ---------------------------------------------------------------------------

/// 各版本（1-6）ECC L 的数据码字数
const DATA_CODEWORDS: [usize; 6] = [19, 34, 55, 80, 108, 136];
/// 各版本每块纠错码字数
const ECC_PER_BLOCK: [usize; 6] = [7, 10, 15, 20, 26, 18];
/// 各版本块数
const NUM_BLOCKS: [usize; 6] = [1, 1, 1, 1, 1, 2];

/// 编码为模块矩阵（true = 黑）
fn encode(data: &[u8]) -> Vec<Vec<bool>> {
    // 选择最小可容纳版本
    let version = (0..6)
        .find(|&v| data.len() <= DATA_CODEWORDS[v] - 2)
        .unwrap_or(5);
    let data_cw = DATA_CODEWORDS[version];

    // 位流：模式 0100 + 8bit 长度 + 数据 + 终止符 + 补齐
    let mut bits: Vec<bool> = Vec::new();
    push_bits(&mut bits, 0b0100, 4);
    push_bits(&mut bits, data.len() as u32, 8);
    for &b in data {
        push_bits(&mut bits, b as u32, 8);
    }
    let capacity = data_cw * 8;
    let terminator = (capacity - bits.len()).min(4);
    push_bits(&mut bits, 0, terminator);
    while bits.len() % 8 != 0 {
        bits.push(false);
    }
    let mut codewords: Vec<u8> = bits
        .chunks(8)
        .map(|chunk| {
            let mut b = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    b |= 1 << (7 - i);
                }
            }
            b
        })
        .collect();
    let mut pad = 0xEC;
    while codewords.len() < data_cw {
        codewords.push(pad);
        pad = if pad == 0xEC { 0x11 } else { 0xEC };
    }

    // 分块 + RS 纠错 + 交织
    let blocks = NUM_BLOCKS[version];
    let ecc_len = ECC_PER_BLOCK[version];
    let short_len = data_cw / blocks;
    let divisor = rs_divisor(ecc_len);
    let mut block_data: Vec<Vec<u8>> = Vec::new();
    let mut block_ecc: Vec<Vec<u8>> = Vec::new();
    let mut offset = 0;
    for i in 0..blocks {
        // 前面的块可能多 1 个数据码字
        let len = short_len + if i < data_cw % blocks { 1 } else { 0 };
        let dat = &codewords[offset..offset + len];
        offset += len;
        block_ecc.push(rs_remainder(dat, &divisor));
        block_data.push(dat.to_vec());
    }
    let mut final_cw: Vec<u8> = Vec::new();
    let max_data = block_data.iter().map(|b| b.len()).max().unwrap_or(0);
    for i in 0..max_data {
        for block in &block_data {
            if i < block.len() {
                final_cw.push(block[i]);
            }
        }
    }
    for i in 0..ecc_len {
        for block in &block_ecc {
            final_cw.push(block[i]);
        }
    }

    // 构建矩阵
    let n = 17 + 4 * (version + 1);
    let mut modules = vec![vec![false; n]; n];
    let mut is_function = vec![vec![false; n]; n];

    // 时序图案
    for i in 0..n {
        set_fn(&mut modules, &mut is_function, 6, i, i % 2 == 0);
        set_fn(&mut modules, &mut is_function, i, 6, i % 2 == 0);
    }
    // 三个定位图案（含分隔带）
    draw_finder(&mut modules, &mut is_function, 3, 3);
    draw_finder(&mut modules, &mut is_function, n - 4, 3);
    draw_finder(&mut modules, &mut is_function, 3, n - 4);
    // 对齐图案（v2-6 仅右下角一个）
    if version >= 1 {
        let c = n - 7;
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let dist = dx.abs().max(dy.abs());
                set_fn(
                    &mut modules,
                    &mut is_function,
                    (c as i32 + dx) as usize,
                    (c as i32 + dy) as usize,
                    dist != 1,
                );
            }
        }
    }
    // 格式信息（ECC L + mask 0）
    draw_format(&mut modules, &mut is_function, n);

    // 数据蛇形放置
    let mut i = 0usize;
    let mut right = n as i32 - 1;
    while right >= 1 {
        if right == 6 {
            right = 5;
        }
        for vert in 0..n {
            for j in 0..2 {
                let x = (right - j) as usize;
                let upward = ((right + 1) & 2) == 0;
                let y = if upward { n - 1 - vert } else { vert };
                if !is_function[y][x] && i < final_cw.len() * 8 {
                    let byte = final_cw[i / 8];
                    modules[y][x] = (byte >> (7 - (i % 8))) & 1 == 1;
                    i += 1;
                }
            }
        }
        right -= 2;
    }

    // mask 0：(x + y) % 2 == 0 翻转
    for y in 0..n {
        for x in 0..n {
            if !is_function[y][x] && (x + y) % 2 == 0 {
                modules[y][x] = !modules[y][x];
            }
        }
    }

    modules
}

fn push_bits(bits: &mut Vec<bool>, value: u32, count: usize) {
    for i in (0..count).rev() {
        bits.push((value >> i) & 1 == 1);
    }
}

/// 设置功能模块（x = 列，y = 行）
fn set_fn(modules: &mut [Vec<bool>], is_fn: &mut [Vec<bool>], x: usize, y: usize, dark: bool) {
    modules[y][x] = dark;
    is_fn[y][x] = true;
}

/// 定位图案 + 分隔带（中心 cx, cy）
fn draw_finder(modules: &mut [Vec<bool>], is_fn: &mut [Vec<bool>], cx: usize, cy: usize) {
    let n = modules.len() as i32;
    for dy in -4i32..=4 {
        for dx in -4i32..=4 {
            let x = cx as i32 + dx;
            let y = cy as i32 + dy;
            if x < 0 || x >= n || y < 0 || y >= n {
                continue;
            }
            let dist = dx.abs().max(dy.abs());
            set_fn(
                modules,
                is_fn,
                x as usize,
                y as usize,
                dist != 2 && dist != 4,
            );
        }
    }
}

/// 格式信息（L + mask 0，BCH(15,5) + 掩码 0x5412）
fn draw_format(modules: &mut [Vec<bool>], is_fn: &mut [Vec<bool>], n: usize) {
    let data = 1u32 << 3; // L 的格式位 = 1，mask = 0
    let mut rem = data;
    for _ in 0..10 {
        rem = (rem << 1) ^ ((rem >> 9) * 0x537);
    }
    let bits = ((data << 10) | rem) ^ 0x5412;
    let bit = |i: usize| (bits >> i) & 1 == 1;

    // 第一份（左上角周围）
    for i in 0..=5 {
        set_fn(modules, is_fn, 8, i, bit(i));
    }
    set_fn(modules, is_fn, 8, 7, bit(6));
    set_fn(modules, is_fn, 8, 8, bit(7));
    set_fn(modules, is_fn, 7, 8, bit(8));
    for i in 9..=14 {
        set_fn(modules, is_fn, 14 - i, 8, bit(i));
    }
    // 第二份
    for i in 0..=7 {
        set_fn(modules, is_fn, n - 1 - i, 8, bit(i));
    }
    for i in 8..=14 {
        set_fn(modules, is_fn, 8, n - 15 + i, bit(i));
    }
    // 恒暗模块
    set_fn(modules, is_fn, 8, n - 8, true);
}

/// GF(256) 乘法（约化多项式 0x11D）
fn gf_mul(x: u8, y: u8) -> u8 {
    let mut z = 0u8;
    for i in (0..8).rev() {
        z = (z << 1) ^ ((z >> 7) * 0x1D);
        z ^= ((y >> i) & 1) * x;
    }
    z
}

/// RS 生成多项式（degree 个纠错码字）
fn rs_divisor(degree: usize) -> Vec<u8> {
    let mut result = vec![0u8; degree];
    result[degree - 1] = 1;
    let mut root = 1u8;
    for _ in 0..degree {
        for j in 0..degree {
            result[j] = gf_mul(result[j], root);
            if j + 1 < degree {
                result[j] ^= result[j + 1];
            }
        }
        root = gf_mul(root, 2);
    }
    result
}

/// RS 余数（纠错码字）
fn rs_remainder(data: &[u8], divisor: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; divisor.len()];
    for &b in data {
        let factor = b ^ result[0];
        result.remove(0);
        result.push(0);
        for (i, &coef) in divisor.iter().enumerate() {
            result[i] ^= gf_mul(coef, factor);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 完整码字（数据 + 纠错）应能被生成多项式整除
    #[test]
    fn rs_codeword_divisible() {
        for v in 0..6 {
            let data: Vec<u8> = (0..DATA_CODEWORDS[v])
                .map(|i| (i * 7 % 256) as u8)
                .collect();
            let divisor = rs_divisor(ECC_PER_BLOCK[v]);
            let ecc = rs_remainder(&data, &divisor);
            let mut full = data.clone();
            full.extend_from_slice(&ecc);
            let rem = rs_remainder(&full, &divisor);
            assert!(rem.iter().all(|&b| b == 0), "version {}", v + 1);
        }
    }

    /// L + mask 0 的格式信息标准值
    #[test]
    fn format_bits_known_value() {
        let data = 1u32 << 3;
        let mut rem = data;
        for _ in 0..10 {
            rem = (rem << 1) ^ ((rem >> 9) * 0x537);
        }
        let bits = ((data << 10) | rem) ^ 0x5412;
        assert_eq!(bits, 0x77C4);
    }

    /// 矩阵尺寸与关键功能图案
    #[test]
    fn matrix_size_and_finder() {
        let m = encode(b"test");
        assert_eq!(m.len(), 21); // version 1
        assert!(m[3][3]); // 左上定位图案中心
        assert!(!m[7][7]); // 分隔带
        assert!(m[13][8]); // 恒暗模块 (n-8, 8)
        assert!(m[3][17]); // 右上定位图案中心
        assert!(m[17][3]); // 左下定位图案中心
    }

    /// 长文本自动选版本
    #[test]
    fn version_selection() {
        let m = encode(&[b'a'; 100]);
        assert_eq!(m.len(), 37); // version 5 (106 字节容量)
    }
}
