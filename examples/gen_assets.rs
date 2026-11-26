//! 生成演示视频资产：`examples/assets/sample.avi`（MJPEG AVI，256x144 @30fps，3 秒）。
//!
//! 运行：`cargo run --example gen_assets`
//! 追加 `--dib` 参数可生成未压缩 DIB 版本（体积大，但不依赖 MJPEG 解码器）。
//!
//! 画面内容：渐变背景 + 弹跳圆球 + 底部进度条，用于验证 VideoPlayer 的
//! 解码、播放节奏与进度控制。

use std::io::Write;
use std::path::PathBuf;

const W: u32 = 256;
const H: u32 = 144;
const FPS: u32 = 30;
const FRAMES: u32 = 90;

fn main() {
    let dib = std::env::args().any(|a| a == "--dib");
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/assets");
    std::fs::create_dir_all(&dir).expect("创建 assets 目录失败");
    let out = dir.join("sample.avi");

    let mut frames: Vec<Vec<u8>> = Vec::new();
    for i in 0..FRAMES {
        let rgb = draw_frame(i);
        let data = if dib {
            // DIB：自底向上 BGR 行序
            let mut bgr = vec![0u8; (W * H * 3) as usize];
            for y in 0..H as usize {
                for x in 0..W as usize {
                    let s = (y * W as usize + x) * 3;
                    let d = ((H as usize - 1 - y) * W as usize + x) * 3;
                    bgr[d] = rgb[s + 2];
                    bgr[d + 1] = rgb[s + 1];
                    bgr[d + 2] = rgb[s];
                }
            }
            bgr
        } else {
            let img = image::RgbImage::from_raw(W, H, rgb).expect("帧缓冲尺寸");
            let mut buf = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 85)
                .encode_image(&img)
                .expect("JPEG 编码失败");
            buf
        };
        frames.push(data);
    }

    let avi = build_avi(&frames, dib);
    std::fs::File::create(&out)
        .and_then(|mut f| f.write_all(&avi))
        .expect("写入 avi 失败");
    println!(
        "已生成 {}（{} 字节，{} 帧）",
        out.display(),
        avi.len(),
        FRAMES
    );
}

/// 绘制第 i 帧（RGB24）
fn draw_frame(i: u32) -> Vec<u8> {
    let t = i as f32 / FRAMES as f32;
    let mut px = vec![0u8; (W * H * 3) as usize];
    // 渐变背景：色相随时间旋转
    let (r1, g1, b1) = hue(t);
    let (r2, g2, b2) = hue(t + 0.35);
    for y in 0..H as usize {
        let k = y as f32 / H as f32;
        for x in 0..W as usize {
            let d = (y * W as usize + x) * 3;
            px[d] = (r1 + (r2 - r1) * k) as u8;
            px[d + 1] = (g1 + (g2 - g1) * k) as u8;
            px[d + 2] = (b1 + (b2 - b1) * k) as u8;
        }
    }
    // 弹跳圆球
    let cx = W as f32 / 2. + (i as f32 * 0.13).sin() * (W as f32 / 2. - 30.);
    let cy = H as f32 / 2. + (i as f32 * 0.21).cos() * (H as f32 / 2. - 26.);
    for y in 0..H as usize {
        for x in 0..W as usize {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 18. {
                let d = (y * W as usize + x) * 3;
                let shade = 1. - dist / 18. * 0.4;
                px[d] = (255. * shade) as u8;
                px[d + 1] = (214. * shade) as u8;
                px[d + 2] = (90. * shade) as u8;
            }
        }
    }
    // 底部进度条
    let filled = ((i + 1) as f32 / FRAMES as f32 * W as f32) as usize;
    for y in (H as usize - 6)..H as usize {
        for x in 0..W as usize {
            let d = (y * W as usize + x) * 3;
            let v = if x < filled { 255 } else { 40 };
            px[d] = v;
            px[d + 1] = v;
            px[d + 2] = v;
        }
    }
    px
}

/// 简易色相 -> RGB（0..1 循环）
fn hue(h: f32) -> (f32, f32, f32) {
    let h = h.rem_euclid(1.) * 6.;
    let x = 1. - (h % 2. - 1.).abs();
    let (r, g, b) = match h as u32 {
        0 => (1., x, 0.),
        1 => (x, 1., 0.),
        2 => (0., 1., x),
        3 => (0., x, 1.),
        4 => (x, 0., 1.),
        _ => (1., 0., x),
    };
    (r * 120. + 40., g * 120. + 40., b * 120. + 40.)
}

fn chunk(out: &mut Vec<u8>, fourcc: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(fourcc);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
    if data.len() % 2 == 1 {
        out.push(0);
    }
}

fn list(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
    let mut inner = Vec::with_capacity(body.len() + 4);
    inner.extend_from_slice(kind);
    inner.extend_from_slice(body);
    chunk(out, b"LIST", &inner);
}

fn u32s(out: &mut Vec<u8>, vals: &[u32]) {
    for v in vals {
        out.extend_from_slice(&v.to_le_bytes());
    }
}

/// 组装 AVI（hdrl + movi + idx1）
fn build_avi(frames: &[Vec<u8>], dib: bool) -> Vec<u8> {
    let handler: &[u8; 4] = if dib { b"DIB " } else { b"MJPG" };
    let max_frame = frames.iter().map(|f| f.len()).max().unwrap_or(0) as u32;

    // avih（MainAVIHeader，56 字节）
    let mut avih = Vec::new();
    u32s(
        &mut avih,
        &[
            1_000_000 / FPS, // dwMicroSecPerFrame
            max_frame * FPS, // dwMaxBytesPerSec
            0,               // dwPaddingGranularity
            0x10,            // dwFlags = AVIF_HASINDEX
            FRAMES,          // dwTotalFrames
            0,               // dwInitialFrames
            1,               // dwStreams
            max_frame,       // dwSuggestedBufferSize
            W,
            H,
            0,
            0,
            0,
            0, // dwReserved[4]
        ],
    );
    let mut avih_chunk = Vec::new();
    chunk(&mut avih_chunk, b"avih", &avih);

    // strh（AVIStreamHeader，56 字节）
    let mut strh = Vec::new();
    strh.extend_from_slice(b"vids");
    strh.extend_from_slice(handler);
    u32s(&mut strh, &[0]); // dwFlags
    strh.extend_from_slice(&0u16.to_le_bytes()); // wPriority
    strh.extend_from_slice(&0u16.to_le_bytes()); // wLanguage
    u32s(
        &mut strh,
        &[
            0,          // dwInitialFrames
            1,          // dwScale
            FPS,        // dwRate
            0,          // dwStart
            FRAMES,     // dwLength
            max_frame,  // dwSuggestedBufferSize
            0xFFFFFFFF, // dwQuality
            0,          // dwSampleSize
        ],
    );
    strh.extend_from_slice(&0i16.to_le_bytes()); // rcFrame.left
    strh.extend_from_slice(&0i16.to_le_bytes()); // rcFrame.top
    strh.extend_from_slice(&(W as i16).to_le_bytes());
    strh.extend_from_slice(&(H as i16).to_le_bytes());
    let mut strh_chunk = Vec::new();
    chunk(&mut strh_chunk, b"strh", &strh);

    // strf（BITMAPINFOHEADER，40 字节）
    let mut strf = Vec::new();
    u32s(
        &mut strf,
        &[
            40,
            W,
            H,              // 正高度：自顶向下（DIB 数据内部已按自底向上存放）
            1 | (24 << 16), // biPlanes | biBitCount
            u32::from_le_bytes(*handler),
            W * H * 3,
            0,
            0,
            0,
            0,
        ],
    );
    let mut strf_chunk = Vec::new();
    chunk(&mut strf_chunk, b"strf", &strf);

    let mut strl = Vec::new();
    strl.extend_from_slice(&strh_chunk);
    strl.extend_from_slice(&strf_chunk);
    let mut strl_list = Vec::new();
    list(&mut strl_list, b"strl", &strl);

    let mut hdrl = Vec::new();
    hdrl.extend_from_slice(&avih_chunk);
    hdrl.extend_from_slice(&strl_list);
    let mut hdrl_list = Vec::new();
    list(&mut hdrl_list, b"hdrl", &hdrl);

    // movi：idx1 偏移以 'movi' fourcc 本身为基准（含 4 字节 tag）
    let mut movi = Vec::new();
    let mut offsets = Vec::new();
    for f in frames {
        offsets.push((movi.len() as u32 + 4, f.len() as u32));
        chunk(&mut movi, b"00dc", f);
    }
    let mut movi_list = Vec::new();
    list(&mut movi_list, b"movi", &movi);

    // idx1
    let mut idx = Vec::new();
    for (off, size) in offsets {
        idx.extend_from_slice(b"00dc");
        u32s(&mut idx, &[0x10, off, size]); // AVIIF_KEYFRAME
    }
    let mut idx_chunk = Vec::new();
    chunk(&mut idx_chunk, b"idx1", &idx);

    let mut body = Vec::new();
    body.extend_from_slice(b"AVI ");
    body.extend_from_slice(&hdrl_list);
    body.extend_from_slice(&movi_list);
    body.extend_from_slice(&idx_chunk);

    let mut avi = Vec::with_capacity(body.len() + 8);
    avi.extend_from_slice(b"RIFF");
    avi.extend_from_slice(&(body.len() as u32).to_le_bytes());
    avi.extend_from_slice(&body);
    avi
}
