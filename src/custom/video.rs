//! 视频播放控件（仅 Windows）。
//!
//! GPUI 本身不带视频解码能力，本控件在后台线程使用 **Windows Media Foundation**
//! 将视频解码为 RGBA 帧，经异步通道送到 UI 线程，用 `ImageSource::Render` 逐帧渲染。
//! 支持系统自带解码器覆盖的格式：MP4 (H.264)、AVI (MJPEG/未压缩)、WMV 等。
//!
//! 创建后自动播放；控制条提供 播放/暂停、进度拖动、时间显示。
//!
//! ```no_run
//! # use gpui::{App, Context};
//! # use gpui_widgets::VideoPlayer;
//! # fn demo(cx: &mut Context<VideoPlayer>) {}
//! // 在视图初始化中：
//! // let video = cx.new(|cx| VideoPlayer::new(cx, "demo.mp4"));
//! // render 中：div().child(video.clone())
//! ```
//!
//! 线程模型：解码线程持有 `Cmd` 接收端与帧发送端；控件 Drop 时发送 Stop，
//! 通道断开后线程自行退出，不会泄漏。

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use gpui::{
    Context, ElementId, InteractiveElement, IntoElement, ParentElement, Render, RenderImage, Role,
    SharedString, Styled, Window, div, px,
};
use image::{Frame, ImageFormat, RgbaImage};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};
use windows::Win32::System::Variant::VT_I8;
use windows::core::{GUID, HSTRING};

use crate::{Button, ImageView, Slider, Spinner, a11y, theme};

/// 解码线程控制命令
enum Cmd {
    Play,
    Pause,
    Seek(f32),
    Stop,
}

/// 解码线程 -> UI 线程消息
enum Msg {
    Meta { duration: f32 },
    Frame { image: Arc<RenderImage>, pts: f32 },
    Eos,
    Error(String),
}

/// 视频播放器。本身是一个 `Entity`，通过 `cx.new(|cx| VideoPlayer::new(cx, path))` 创建。
pub struct VideoPlayer {
    path: SharedString,
    frame: Option<Arc<RenderImage>>,
    duration: f32,
    position: f32,
    playing: bool,
    ended: bool,
    error: Option<SharedString>,
    cmd_tx: Sender<Cmd>,
}

impl VideoPlayer {
    /// 打开视频文件并自动播放（解码在后台线程进行）
    pub fn new(cx: &mut Context<Self>, path: impl Into<SharedString>) -> Self {
        let path = path.into();
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (msg_tx, msg_rx) = flume::unbounded();
        let thread_path = path.to_string();
        let _ = std::thread::Builder::new()
            .name("gpui-widgets-video".into())
            .spawn(move || decode_thread(thread_path, cmd_rx, msg_tx));

        // 将解码消息泵入实体状态（前台 executor，owned AsyncApp 避免借用生命周期问题）
        let this = cx.weak_entity();
        let mut app = cx.to_async();
        cx.foreground_executor()
            .spawn(async move {
                while let Ok(msg) = msg_rx.recv_async().await {
                    if this
                        .update(&mut app, |this, cx| this.apply_msg(msg, cx))
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .detach();

        Self {
            path,
            frame: None,
            duration: 0.,
            position: 0.,
            playing: true,
            ended: false,
            error: None,
            cmd_tx,
        }
    }

    /// 播放（播放到结尾后再次播放会从头开始）
    pub fn play(&mut self, cx: &mut Context<Self>) {
        if self.ended {
            self.seek(0., cx);
        }
        self.playing = true;
        let _ = self.cmd_tx.send(Cmd::Play);
        cx.notify();
    }

    /// 暂停
    pub fn pause(&mut self, cx: &mut Context<Self>) {
        self.playing = false;
        let _ = self.cmd_tx.send(Cmd::Pause);
        cx.notify();
    }

    /// 播放 / 暂停切换
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        if self.playing {
            self.pause(cx);
        } else {
            self.play(cx);
        }
    }

    /// 跳转到指定秒数
    pub fn seek(&mut self, secs: f32, cx: &mut Context<Self>) {
        let secs = secs.clamp(0., self.duration.max(0.));
        self.position = secs;
        self.ended = false;
        let _ = self.cmd_tx.send(Cmd::Seek(secs));
        cx.notify();
    }

    /// 当前播放位置（秒）
    pub fn position(&self) -> f32 {
        self.position
    }

    /// 视频总时长（秒）
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// 是否正在播放
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    fn apply_msg(&mut self, msg: Msg, cx: &mut Context<Self>) {
        match msg {
            Msg::Meta { duration } => self.duration = duration,
            Msg::Frame { image, pts } => {
                self.frame = Some(image);
                self.position = pts;
                self.ended = false;
            }
            Msg::Eos => {
                self.playing = false;
                self.ended = true;
                self.position = self.duration;
            }
            Msg::Error(err) => {
                self.error = Some(err.into());
                self.playing = false;
            }
        }
        cx.notify();
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(Cmd::Stop);
    }
}

impl Render for VideoPlayer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let m = theme::with(crate::theme::metric::video_metric);
        let screen = div()
            .w_full()
            .h(px(m.height))
            .flex()
            .items_center()
            .justify_center()
            .bg(m.backdrop)
            .rounded(px(m.radius))
            .overflow_hidden()
            .child(match (&self.error, &self.frame) {
                (Some(err), _) => div()
                    .w_full()
                    .px_4()
                    .text_center()
                    .text_xs()
                    .text_color(theme::text_secondary())
                    .child(err.clone())
                    .into_any_element(),
                (None, Some(frame)) => ImageView::from_render(frame.clone())
                    .fit_contain()
                    .size_full()
                    .into_any_element(),
                (None, None) => Spinner::new().size(px(m.spinner_size)).into_any_element(),
            });

        let label = if self.playing { "暂停" } else { "播放" };
        let time = format!("{} / {}", fmt_time(self.position), fmt_time(self.duration));
        let seek_id = format!("video-seek-{}", self.path);
        let player_id = format!("video-player-{}:{:?}", self.path, cx.entity_id());
        let status = if self.error.is_some() {
            "播放失败"
        } else if self.ended {
            "播放结束"
        } else if self.playing {
            "正在播放"
        } else if self.frame.is_none() {
            "正在载入"
        } else {
            "已暂停"
        };
        let mut semantics =
            a11y::Semantics::new(Role::Region, format!("视频播放器：{}", self.path))
                .id(player_id.clone())
                .value(format!("{time}；{status}"));
        if let Some(error) = &self.error {
            semantics = semantics.description(error.clone());
        }

        let controls = div().flex().flex_col().gap_2().child(screen).child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    Button::new(label)
                        .id(format!("{player_id}-play"))
                        .small()
                        .secondary()
                        .on_click(cx.listener(|this, _, _, cx| this.toggle(cx))),
                )
                .child(
                    div().flex_1().child(
                        Slider::new(ElementId::Name(seek_id.into()), self.position)
                            .label("播放进度")
                            .range(0., self.duration.max(0.1))
                            .on_change(cx.listener(|this, &v, _, cx| this.seek(v, cx))),
                    ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_secondary())
                        .child(time),
                ),
        );
        semantics.apply(controls.id(ElementId::Name(player_id.into())))
    }
}

fn fmt_time(secs: f32) -> String {
    let s = secs.max(0.) as u32;
    format!("{:02}:{:02}", s / 60, s % 60)
}

// ---------------------------------------------------------------------------
// 后台解码（Windows Media Foundation）
// ---------------------------------------------------------------------------

fn decode_thread(path: String, cmd_rx: Receiver<Cmd>, tx: flume::Sender<Msg>) {
    unsafe {
        if let Err(err) = CoInitializeEx(None, COINIT_MULTITHREADED).ok() {
            let _ = tx.send(Msg::Error(format!("COM 初始化失败: {err}")));
            return;
        }
        let result =
            MFStartup(MF_VERSION, MFSTARTUP_FULL).and_then(|_| run_reader(&path, &cmd_rx, &tx));
        let _ = MFShutdown();
        CoUninitialize();
        if let Err(err) = result {
            let _ = tx.send(Msg::Error(format!("视频解码失败: {err}")));
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn run_reader(
    path: &str,
    cmd_rx: &Receiver<Cmd>,
    tx: &flume::Sender<Msg>,
) -> windows::core::Result<()> {
    let url = HSTRING::from(path);
    let reader = MFCreateSourceReaderFromURL(&url, None)?;
    let stream = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;

    // 优先要求 RGB32；解码器不支持时回退原生格式（NV12/YUY2/RGB24 等）软转
    let mt = MFCreateMediaType()?;
    mt.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
    mt.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
    // 设置失败时保留原生输出类型，后续按 subtype 软转
    let _ = reader.SetCurrentMediaType(stream, None, &mt);

    let current = reader.GetCurrentMediaType(stream)?;
    let subtype = current.GetGUID(&MF_MT_SUBTYPE)?;
    let packed = current.GetUINT64(&MF_MT_FRAME_SIZE)?;
    let (w, h) = ((packed >> 32) as u32, packed as u32);
    let stride = current
        .GetUINT32(&MF_MT_DEFAULT_STRIDE)
        .map(|v| v as i32)
        .unwrap_or(w as i32 * 4);
    let stride = if stride == 0 { w as i32 * 4 } else { stride };
    let duration = reader
        .GetPresentationAttribute(MF_SOURCE_READER_MEDIASOURCE.0 as u32, &MF_PD_DURATION)
        .map(|pv| unsafe { pv.Anonymous.Anonymous.Anonymous.hVal } as f32 / 1e7)
        .unwrap_or(0.);
    let _ = tx.send(Msg::Meta { duration });

    // 播放时钟：clock 为当前应呈现的 pts（秒），暂停时冻结
    let mut clock = 0f32;
    let mut last = Instant::now();
    let mut paused = false;
    let mut eos = false;
    let mut pending: Option<(f32, Arc<RenderImage>)> = None;

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Cmd::Stop => return Ok(()),
                Cmd::Play => {
                    if eos {
                        seek_to(&reader, stream, 0.)?;
                        eos = false;
                        pending = None;
                        clock = 0.;
                    }
                    paused = false;
                }
                Cmd::Pause => paused = true,
                Cmd::Seek(secs) => {
                    seek_to(&reader, stream, secs)?;
                    clock = secs;
                    eos = false;
                    pending = None;
                }
            }
        }
        if tx.is_disconnected() {
            return Ok(());
        }

        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32();
        last = now;
        if !paused && !eos {
            clock += dt;
        }

        // 按 pts 节奏呈现上一帧
        if let Some((pts, image)) = pending.take() {
            if clock >= pts {
                if tx.send(Msg::Frame { image, pts }).is_err() {
                    return Ok(());
                }
            } else {
                pending = Some((pts, image));
                std::thread::sleep(Duration::from_millis(4));
                continue;
            }
        }
        if eos {
            std::thread::sleep(Duration::from_millis(20));
            continue;
        }

        let mut flags = 0u32;
        let mut ts = 0i64;
        let mut sample = None;
        reader.ReadSample(
            stream,
            0,
            None,
            Some(&mut flags),
            Some(&mut ts),
            Some(&mut sample),
        )?;
        if flags & (MF_SOURCE_READERF_ENDOFSTREAM.0 as u32) != 0 {
            eos = true;
            let _ = tx.send(Msg::Eos);
            continue;
        }
        let Some(sample) = sample else { continue };
        let pts = ts as f32 / 1e7;
        let image = sample_to_image(&sample, w, h, stride, &subtype)?;
        pending = Some((pts, image));
    }
}

/// 将一帧样本转换为 RGBA 的 `RenderImage`。
/// 支持 RGB32/RGB24/NV12/YUY2/IYUV/YV12，处理负 stride 的自底向上行序。
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn sample_to_image(
    sample: &IMFSample,
    w: u32,
    h: u32,
    stride: i32,
    subtype: &GUID,
) -> windows::core::Result<Arc<RenderImage>> {
    let buffer = sample.ConvertToContiguousBuffer()?;
    let mut ptr: *mut u8 = std::ptr::null_mut();
    let mut len = 0u32;
    buffer.Lock(&mut ptr, None, Some(&mut len))?;
    let result = unsafe { convert_frame(ptr, len as usize, w, h, stride, subtype) };
    buffer.Unlock()?;
    let rgba = result?;
    let image = RgbaImage::from_raw(w, h, rgba)
        .ok_or_else(|| windows::core::Error::new(E_INVALIDARG, "帧尺寸不匹配"))?;
    Ok(Arc::new(RenderImage::new(vec![Frame::new(image)])))
}

/// 行序归一：负 stride 表示自底向上存储
#[inline]
fn src_row(y: usize, height: usize, stride: i32) -> usize {
    if stride < 0 { height - 1 - y } else { y }
}

#[inline]
fn clamp8(v: f32) -> u8 {
    v.clamp(0., 255.) as u8
}

/// YUV (BT.601 limited range) -> RGB
#[inline]
fn yuv_to_rgb(y: u8, u: u8, v: u8) -> (u8, u8, u8) {
    let y = (y as f32 - 16.) * 1.164;
    let u = u as f32 - 128.;
    let v = v as f32 - 128.;
    (
        clamp8(y + 1.596 * v),
        clamp8(y - 0.391 * u - 0.813 * v),
        clamp8(y + 2.018 * u),
    )
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn convert_frame(
    ptr: *mut u8,
    len: usize,
    w: u32,
    h: u32,
    stride: i32,
    subtype: &GUID,
) -> windows::core::Result<Vec<u8>> {
    let (wu, hu) = (w as usize, h as usize);
    let row_bytes = stride.unsigned_abs() as usize;
    let mut rgba = vec![0u8; wu * hu * 4];
    let put = |rgba: &mut Vec<u8>, x: usize, y: usize, r: u8, g: u8, b: u8| {
        let d = (y * wu + x) * 4;
        rgba[d] = r;
        rgba[d + 1] = g;
        rgba[d + 2] = b;
        rgba[d + 3] = 255;
    };

    if *subtype == MFVideoFormat_MJPG {
        // 系统未插 MJPEG 解码器时样本为 JPEG 码流，直接用 image 解码
        let bytes = std::slice::from_raw_parts(ptr, len);
        let img = image::load_from_memory_with_format(bytes, ImageFormat::Jpeg).map_err(|e| {
            windows::core::Error::new(E_INVALIDARG, format!("JPEG 帧解码失败: {e}"))
        })?;
        return Ok(img.to_rgba8().into_raw());
    } else if *subtype == MFVideoFormat_RGB32 {
        if len < row_bytes * hu {
            return Err(windows::core::Error::new(E_INVALIDARG, "帧缓冲长度不足"));
        }
        for y in 0..hu {
            let row = ptr.add(src_row(y, hu, stride) * row_bytes);
            for x in 0..wu {
                let s = row.add(x * 4);
                put(&mut rgba, x, y, *s.add(2), *s.add(1), *s);
            }
        }
    } else if *subtype == MFVideoFormat_RGB24 {
        if len < row_bytes * hu {
            return Err(windows::core::Error::new(E_INVALIDARG, "帧缓冲长度不足"));
        }
        for y in 0..hu {
            let row = ptr.add(src_row(y, hu, stride) * row_bytes);
            for x in 0..wu {
                let s = row.add(x * 3);
                put(&mut rgba, x, y, *s.add(2), *s.add(1), *s);
            }
        }
    } else if *subtype == MFVideoFormat_NV12 {
        let uv_off = row_bytes * hu;
        if len < uv_off + row_bytes * (hu / 2) {
            return Err(windows::core::Error::new(E_INVALIDARG, "帧缓冲长度不足"));
        }
        for y in 0..hu {
            let yrow = ptr.add(src_row(y, hu, stride) * row_bytes);
            let uvrow = ptr.add(uv_off + src_row(y / 2, hu / 2, stride) * row_bytes);
            for x in 0..wu {
                let yv = *yrow.add(x);
                let uv = uvrow.add((x / 2) * 2);
                let (r, g, b) = yuv_to_rgb(yv, *uv, *uv.add(1));
                put(&mut rgba, x, y, r, g, b);
            }
        }
    } else if *subtype == MFVideoFormat_YUY2 {
        if len < row_bytes * hu {
            return Err(windows::core::Error::new(E_INVALIDARG, "帧缓冲长度不足"));
        }
        for y in 0..hu {
            let row = ptr.add(src_row(y, hu, stride) * row_bytes);
            for x in 0..wu {
                let p = row.add((x / 2) * 4);
                let yv = if x % 2 == 0 { *p } else { *p.add(2) };
                let (r, g, b) = yuv_to_rgb(yv, *p.add(1), *p.add(3));
                put(&mut rgba, x, y, r, g, b);
            }
        }
    } else if *subtype == MFVideoFormat_IYUV || *subtype == MFVideoFormat_YV12 {
        let uv_stride = row_bytes / 2;
        let plane = row_bytes * hu;
        let uv_plane = uv_stride * (hu / 2);
        if len < plane + uv_plane * 2 {
            return Err(windows::core::Error::new(E_INVALIDARG, "帧缓冲长度不足"));
        }
        let (u_off, v_off) = if *subtype == MFVideoFormat_IYUV {
            (plane, plane + uv_plane)
        } else {
            (plane + uv_plane, plane)
        };
        for y in 0..hu {
            let yrow = ptr.add(src_row(y, hu, stride) * row_bytes);
            let urow = ptr.add(u_off + src_row(y / 2, hu / 2, stride) * uv_stride);
            let vrow = ptr.add(v_off + src_row(y / 2, hu / 2, stride) * uv_stride);
            for x in 0..wu {
                let (r, g, b) = yuv_to_rgb(*yrow.add(x), *urow.add(x / 2), *vrow.add(x / 2));
                put(&mut rgba, x, y, r, g, b);
            }
        }
    } else {
        return Err(windows::core::Error::new(
            E_INVALIDARG,
            format!("不支持的视频格式: {subtype:?}"),
        ));
    }
    Ok(rgba)
}

/// 以 100ns 时间格式跳转并清空解码缓冲
#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn seek_to(reader: &IMFSourceReader, stream: u32, secs: f32) -> windows::core::Result<()> {
    let mut pv = PROPVARIANT::default();
    // 显式解引用 ManuallyDrop，避免隐式 DerefMut _union 字段警告
    let inner = &mut *pv.Anonymous.Anonymous;
    inner.vt = VT_I8;
    inner.Anonymous.hVal = (secs.max(0.) * 1e7) as i64;
    reader.SetCurrentPosition(&GUID::zeroed(), &pv)?;
    reader.Flush(stream)
}
