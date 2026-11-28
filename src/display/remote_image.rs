//! 远程图片的有界缓存与可靠加载。
//!
//! 通过 [`remote_image_cache`] 包住 [`super::image::ImageView`] 子树即可启用；
//! 本地路径和内嵌资源仍走 GPUI 原生加载路径。

use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    io::{self, Cursor},
    mem,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{AsyncReadExt, future::select};
use gpui::{
    App, BackgroundExecutor, Context, Entity, EntityId, ImageCache, ImageCacheElement,
    ImageCacheError, ImgResourceLoader, RenderImage, Resource, Result, SharedUri, Task, WeakEntity,
    Window, hash,
    http_client::{self, AsyncBody, HttpRequestExt, RedirectPolicy},
};
use image::{
    AnimationDecoder, DynamicImage, Frame, ImageDecoder, ImageFormat, ImageReader, Rgba,
    codecs::{gif::GifDecoder, webp::WebPDecoder},
};

/// 远程图片加载策略。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteImageConfig {
    timeout: Duration,
    retries: u32,
    retry_delay: Duration,
    max_retry_delay: Duration,
    max_response_bytes: usize,
    max_cache_entries: usize,
    max_cache_bytes: usize,
    failure_ttl: Duration,
}

impl Default for RemoteImageConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(20),
            retries: 2,
            retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(2),
            max_response_bytes: 8 * 1024 * 1024,
            max_cache_entries: 128,
            max_cache_bytes: 64 * 1024 * 1024,
            failure_ttl: Duration::from_secs(30),
        }
    }
}

impl RemoteImageConfig {
    /// 设置单次请求（含响应体）的超时时间。
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置失败后的额外重试次数。
    #[must_use]
    pub fn retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    /// 设置第一次重试前的等待时间，后续按指数退避。
    #[must_use]
    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// 设置指数退避的最大等待时间。
    #[must_use]
    pub fn max_retry_delay(mut self, delay: Duration) -> Self {
        self.max_retry_delay = delay;
        self
    }

    /// 设置单个 HTTP 响应体的字节上限。
    #[must_use]
    pub fn max_response_bytes(mut self, bytes: usize) -> Self {
        self.max_response_bytes = bytes;
        self
    }

    /// 设置 LRU 条目上限，包含加载中和失败条目。
    #[must_use]
    pub fn max_cache_entries(mut self, entries: usize) -> Self {
        self.max_cache_entries = entries;
        self
    }

    /// 设置单个解码图片及缓存合计的字节上限。
    #[must_use]
    pub fn max_cache_bytes(mut self, bytes: usize) -> Self {
        self.max_cache_bytes = bytes;
        self
    }

    /// 设置失败结果的复用时间，过期后允许重新请求。
    #[must_use]
    pub fn failure_ttl(mut self, ttl: Duration) -> Self {
        self.failure_ttl = ttl;
        self
    }

    fn validate(self) {
        assert!(!self.timeout.is_zero(), "image timeout must be positive");
        assert!(
            self.max_response_bytes > 0,
            "image response limit must be positive"
        );
        assert!(
            self.max_cache_entries > 0,
            "image cache entry limit must be positive"
        );
        assert!(
            self.max_cache_bytes > 0,
            "image cache byte limit must be positive"
        );
    }

    fn retry_wait(&self, retry_index: u32) -> Duration {
        self.retry_delay
            .saturating_mul(2u32.saturating_pow(retry_index))
            .min(self.max_retry_delay)
    }
}

type ImageResult = Result<Arc<RenderImage>, ImageCacheError>;

enum CacheLookup {
    Loading,
    Ready(ImageResult),
    Expired,
}

fn lookup_entry(state: Option<&EntryState>, failure_ttl: Duration) -> CacheLookup {
    let Some(state) = state else {
        return CacheLookup::Loading;
    };
    if state.result.is_err() && state.completed_at.elapsed() >= failure_ttl {
        CacheLookup::Expired
    } else {
        CacheLookup::Ready(state.result.clone())
    }
}

struct EntryState {
    result: ImageResult,
    completed_at: Instant,
    decoded_bytes: usize,
}

struct CacheEntry {
    state: Rc<RefCell<Option<EntryState>>>,
    task: Task<()>,
    generation: u64,
    counted_bytes: usize,
    subscribers: Vec<EntityId>,
}

/// 远程图片 LRU 缓存，同时负责超时、重试与在途请求去重。
pub struct RemoteImageCache {
    config: RemoteImageConfig,
    entity: WeakEntity<Self>,
    entries: HashMap<u64, CacheEntry>,
    lru: VecDeque<u64>,
    cached_bytes: usize,
    next_generation: u64,
}

impl RemoteImageCache {
    /// 使用默认策略创建缓存。
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self::with_config(RemoteImageConfig::default(), cx)
    }

    /// 使用自定义策略创建缓存。
    pub fn with_config(config: RemoteImageConfig, cx: &mut Context<Self>) -> Self {
        config.validate();
        let cache = Self {
            config,
            entity: cx.weak_entity(),
            entries: HashMap::new(),
            lru: VecDeque::new(),
            cached_bytes: 0,
            next_generation: 0,
        };
        let entity = cx.entity();
        cx.observe_release(&entity, |cache, _, cx| cache.release(cx))
            .detach();
        cache
    }

    /// 返回缓存中的条目数，包含加载中和失败条目。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 缓存是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 返回当前计入上限的解码后字节数。
    pub fn cached_bytes(&self) -> usize {
        self.cached_bytes
    }

    /// 清空缓存并取消未完成的加载。
    pub fn clear(&mut self, window: &mut Window, cx: &mut App) {
        let keys: Vec<_> = self.entries.keys().copied().collect();
        for key in keys {
            self.remove_entry(key, Some(window), cx);
        }
    }

    /// 移除指定远程资源并取消其未完成的加载。
    pub fn remove(&mut self, resource: &Resource, window: &mut Window, cx: &mut App) {
        if matches!(resource, Resource::Uri(_)) {
            self.remove_entry(hash(resource), Some(window), cx);
        }
    }

    fn touch(&mut self, key: u64) {
        if let Some(index) = self.lru.iter().position(|candidate| *candidate == key) {
            self.lru.remove(index);
        }
        self.lru.push_front(key);
    }

    fn remove_entry(&mut self, key: u64, window: Option<&mut Window>, cx: &mut App) {
        if let Some(index) = self.lru.iter().position(|candidate| *candidate == key) {
            self.lru.remove(index);
        }
        if let Some(entry) = self.entries.remove(&key) {
            let CacheEntry {
                state,
                task,
                counted_bytes,
                subscribers,
                generation: _,
            } = entry;
            drop(task);
            self.cached_bytes = self.cached_bytes.saturating_sub(counted_bytes);
            if let Some(state) = state.borrow().as_ref()
                && let Ok(image) = &state.result
            {
                cx.drop_image(image.clone(), window);
            }
            for view in subscribers {
                cx.notify(view);
            }
        }
    }

    fn evict(&mut self, mut window: Option<&mut Window>, cx: &mut App) {
        while self.entries.len() > self.config.max_cache_entries
            || self.cached_bytes > self.config.max_cache_bytes
        {
            let Some(key) = self.lru.pop_back() else {
                break;
            };
            self.remove_entry(key, window.as_deref_mut(), cx);
        }
    }

    fn release(&mut self, cx: &mut App) {
        for entry in self.entries.values() {
            if let Some(state) = entry.state.borrow().as_ref()
                && let Ok(image) = &state.result
            {
                cx.drop_image(image.clone(), None);
            }
        }
        self.entries.clear();
        self.lru.clear();
        self.cached_bytes = 0;
    }

    fn start_load(&mut self, key: u64, uri: SharedUri, window: &mut Window, cx: &mut App) {
        let state = Rc::new(RefCell::new(None));
        let generation = self.next_generation;
        self.next_generation = self.next_generation.wrapping_add(1);

        let client = cx.http_client();
        let executor = cx.background_executor().clone();
        let svg_renderer = cx.svg_renderer();
        let config = self.config;
        let task = executor
            .clone()
            .spawn(async move { load_remote(uri, config, client, executor, svg_renderer).await });
        let completion = cx.spawn({
            let entity = self.entity.clone();
            let state = state.clone();
            async move |cx| {
                let result = task.await;
                let decoded_bytes = result
                    .as_ref()
                    .ok()
                    .map_or(0, |image| image_bytes(image.as_ref()));
                *state.borrow_mut() = Some(EntryState {
                    result,
                    completed_at: Instant::now(),
                    decoded_bytes,
                });
                if let Some(entity) = entity.upgrade() {
                    entity.update(cx, |cache, cx| {
                        cache.finish_load(key, generation, cx);
                    });
                }
            }
        });

        self.entries.insert(
            key,
            CacheEntry {
                state,
                task: completion,
                generation,
                counted_bytes: 0,
                subscribers: Vec::new(),
            },
        );
        self.touch(key);
        self.evict(Some(window), cx);
    }

    fn finish_load(&mut self, key: u64, generation: u64, cx: &mut App) {
        if !self
            .entries
            .get(&key)
            .is_some_and(|entry| entry.generation == generation)
        {
            return;
        }
        let entry = self.entries.get_mut(&key).expect("checked above");
        {
            let mut state = entry.state.borrow_mut();
            if let Some(state) = state.as_mut()
                && state.result.is_ok()
                && state.decoded_bytes > self.config.max_cache_bytes
            {
                state.result = Err(ImageCacheError::Asset(
                    format!(
                        "decoded image exceeds {} bytes",
                        self.config.max_cache_bytes
                    )
                    .into(),
                ));
            }
        }
        let (decoded_bytes, is_success) = entry
            .state
            .borrow()
            .as_ref()
            .map(|state| (state.decoded_bytes, state.result.is_ok()))
            .unwrap_or((0, false));
        let subscribers = mem::take(&mut entry.subscribers);
        let counted_bytes = if is_success { decoded_bytes } else { 0 };
        entry.counted_bytes = counted_bytes;
        self.cached_bytes += counted_bytes;
        self.touch(key);
        self.evict(None, cx);
        for view in subscribers {
            cx.notify(view);
        }
    }

    fn subscribe(&mut self, key: u64, window: &Window) {
        let view = window.current_view();
        if let Some(entry) = self.entries.get_mut(&key)
            && !entry.subscribers.contains(&view)
        {
            entry.subscribers.push(view);
        }
    }

    fn load_uri(
        &mut self,
        key: u64,
        uri: SharedUri,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<ImageResult> {
        self.evict(Some(window), cx);
        if let Some(entry) = self.entries.get(&key) {
            let outcome = {
                let state = entry.state.borrow();
                lookup_entry(state.as_ref(), self.config.failure_ttl)
            };
            match outcome {
                CacheLookup::Ready(result) => {
                    self.touch(key);
                    self.subscribe(key, window);
                    return Some(result);
                }
                CacheLookup::Expired => self.remove_entry(key, Some(window), cx),
                CacheLookup::Loading => {
                    self.subscribe(key, window);
                    return None;
                }
            }
        }
        self.start_load(key, uri, window, cx);
        self.subscribe(key, window);
        None
    }
}

impl ImageCache for RemoteImageCache {
    fn load(
        &mut self,
        resource: &Resource,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<ImageResult> {
        match resource {
            Resource::Uri(uri) => self.load_uri(hash(resource), uri.clone(), window, cx),
            _ => window.use_asset::<ImgResourceLoader>(resource, cx),
        }
    }
}

/// 为子树启用远程图片缓存。
pub fn remote_image_cache(cache: Entity<RemoteImageCache>) -> ImageCacheElement {
    gpui::image_cache(cache)
}

fn image_bytes(image: &RenderImage) -> usize {
    (0..image.frame_count())
        .filter_map(|frame| image.as_bytes(frame))
        .map(<[u8]>::len)
        .sum()
}

fn should_retry(error: &ImageCacheError) -> bool {
    match error {
        ImageCacheError::Other(_) | ImageCacheError::Io(_) => true,
        ImageCacheError::BadStatus { status, .. } => {
            matches!(status.as_u16(), 408 | 429 | 500..=599)
        }
        _ => false,
    }
}

async fn load_remote(
    uri: SharedUri,
    config: RemoteImageConfig,
    client: Arc<dyn http_client::HttpClient>,
    executor: BackgroundExecutor,
    svg_renderer: gpui::SvgRenderer,
) -> ImageResult {
    let mut last_error = None;
    for attempt in 0..=config.retries {
        let result = fetch_attempt(
            uri.clone(),
            config,
            client.clone(),
            executor.clone(),
            svg_renderer.clone(),
        )
        .await;
        match result {
            Ok(image) => return Ok(image),
            Err(error) => {
                let retry = attempt < config.retries && should_retry(&error);
                last_error = Some(error);
                if !retry {
                    break;
                }
                executor.timer(config.retry_wait(attempt)).await;
            }
        }
    }
    Err(last_error.expect("remote image load performs at least one attempt"))
}

async fn read_limited_body(
    body: &mut (impl AsyncReadExt + Unpin),
    max_bytes: usize,
    output: &mut Vec<u8>,
) -> std::result::Result<(), ImageCacheError> {
    let mut buffer = [0u8; 16 * 1024];
    loop {
        let read = body.read(&mut buffer).await?;
        if read == 0 {
            return Ok(());
        }
        if output.len().saturating_add(read) > max_bytes {
            return Err(ImageCacheError::Asset(
                format!("remote image response exceeds {max_bytes} bytes").into(),
            ));
        }
        output.extend_from_slice(&buffer[..read]);
    }
}

async fn fetch_attempt(
    uri: SharedUri,
    config: RemoteImageConfig,
    client: Arc<dyn http_client::HttpClient>,
    executor: BackgroundExecutor,
    svg_renderer: gpui::SvgRenderer,
) -> ImageResult {
    let request = http_client::Builder::new()
        .uri(uri.as_ref())
        .follow_redirects(RedirectPolicy::FollowAll)
        .timeout(config.timeout)
        .body(AsyncBody::default())
        .map_err(|error| ImageCacheError::Other(Arc::new(error.into())))?;
    let operation = async {
        let mut response = client.send(request).await?;
        let status = response.status();
        if let Some(length) = response
            .headers()
            .get(http_client::http::header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok())
            && length > config.max_response_bytes
        {
            return Err(ImageCacheError::Asset(
                format!(
                    "remote image response exceeds {} bytes",
                    config.max_response_bytes
                )
                .into(),
            ));
        }

        let mut body = Vec::new();
        read_limited_body(response.body_mut(), config.max_response_bytes, &mut body).await?;
        Ok::<_, ImageCacheError>((uri, status, body))
    };
    let timeout = executor.timer(config.timeout);
    futures::pin_mut!(operation, timeout);
    let (uri, status, body) = match select(operation, timeout).await {
        futures::future::Either::Left((Ok(response), _)) => response,
        futures::future::Either::Left((Err(error), _)) => return Err(error),
        futures::future::Either::Right(_) => {
            return Err(ImageCacheError::Io(Arc::new(io::Error::new(
                io::ErrorKind::TimedOut,
                "remote image request timed out",
            ))));
        }
    };

    if !status.is_success() {
        let mut body = String::from_utf8_lossy(&body).into_owned();
        let first_line = body.lines().next().unwrap_or("").trim_end();
        body.truncate(first_line.len());
        return Err(ImageCacheError::BadStatus { uri, status, body });
    }

    if let Ok(format) = image::guess_format(&body) {
        let data = match format {
            image::ImageFormat::Gif => decode_gif_image(&body, config.max_cache_bytes)?,
            image::ImageFormat::WebP => decode_webp_image(&body, config.max_cache_bytes)?,
            _ => decode_static_image(&body, format, config.max_cache_bytes)?,
        };
        Ok(Arc::new(RenderImage::new(data)))
    } else {
        // GPUI's ParsedSvg hides dimensions, so preflight before rasterization.
        ensure_svg_render_size(&body, config.max_cache_bytes)?;
        let image = svg_renderer
            .render_single_frame(&body, 1.0)
            .map_err(ImageCacheError::from)?;
        if image_bytes(&image) > config.max_cache_bytes {
            return Err(ImageCacheError::Asset(
                format!("decoded image exceeds {} bytes", config.max_cache_bytes).into(),
            ));
        }
        Ok(image)
    }
}

fn decode_static_image(
    bytes: &[u8],
    format: ImageFormat,
    max_bytes: usize,
) -> std::result::Result<Vec<Frame>, ImageCacheError> {
    let decoder = ImageReader::with_format(Cursor::new(bytes), format).into_decoder()?;
    decode_static_image_from_decoder(decoder, max_bytes)
}

fn decode_static_image_from_decoder(
    mut decoder: impl ImageDecoder,
    max_bytes: usize,
) -> std::result::Result<Vec<Frame>, ImageCacheError> {
    let (width, height) = decoder.dimensions();
    ensure_decoded_size(width, height, max_bytes)?;
    let orientation = decoder.orientation()?;
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    let mut data = image.into_rgba8();
    for pixel in data.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Ok(vec![Frame::new(data)])
}

fn ensure_decoded_size(
    width: u32,
    height: u32,
    max_bytes: usize,
) -> std::result::Result<usize, ImageCacheError> {
    let bytes = u64::from(width) * u64::from(height) * 4;
    if bytes > max_bytes as u64 {
        return Err(ImageCacheError::Asset(
            format!("decoded image requires {bytes} bytes, limit is {max_bytes}").into(),
        ));
    }
    Ok(bytes as usize)
}

fn ensure_svg_render_size(
    bytes: &[u8],
    max_bytes: usize,
) -> std::result::Result<(), ImageCacheError> {
    let tree =
        usvg::Tree::from_data(bytes, &usvg::Options::default()).map_err(ImageCacheError::from)?;
    let size = tree.size();
    let mut width = size.width() * 2.0;
    let mut height = size.height() * 2.0;
    let scale = (8192.0 / width).min(8192.0 / height).min(1.0);
    width *= scale;
    height *= scale;
    ensure_decoded_size(width as u32, height as u32, max_bytes)?;
    Ok(())
}

fn decode_animated_frames<'a>(
    decoder: impl AnimationDecoder<'a> + ImageDecoder,
    format_name: &str,
    max_bytes: usize,
) -> std::result::Result<Vec<Frame>, ImageCacheError> {
    let (width, height) = decoder.dimensions();
    let frame_bytes = ensure_decoded_size(width, height, max_bytes)?;
    let mut frames = Vec::new();
    let mut decoded_bytes = 0usize;
    let mut iterator = decoder.into_frames();
    loop {
        let Some(frame) = iterator.next() else {
            if frames.is_empty() {
                return Err(ImageCacheError::Asset(
                    format!("{format_name} could not be decoded: no frames found").into(),
                ));
            }
            return Ok(frames);
        };
        if decoded_bytes + frame_bytes > max_bytes {
            return Err(ImageCacheError::Asset(
                format!("decoded image exceeds {max_bytes} bytes").into(),
            ));
        }
        let mut frame = frame?;
        decoded_bytes += frame_bytes;
        for pixel in frame.buffer_mut().chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        frames.push(frame);
    }
}

fn decode_gif_image(
    bytes: &[u8],
    max_bytes: usize,
) -> std::result::Result<Vec<Frame>, ImageCacheError> {
    let decoder = GifDecoder::new(Cursor::new(bytes))?;
    decode_animated_frames(decoder, "GIF", max_bytes)
}

fn decode_webp_image(
    bytes: &[u8],
    max_bytes: usize,
) -> std::result::Result<Vec<Frame>, ImageCacheError> {
    let mut decoder = WebPDecoder::new(Cursor::new(bytes))?;
    if decoder.has_animation() {
        let _ = decoder.set_background_color(Rgba([0, 0, 0, 0]));
        decode_animated_frames(decoder, "WebP", max_bytes)
    } else {
        decode_static_image_from_decoder(decoder, max_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use http_client::StatusCode;
    use image::{
        ImageBuffer, ImageEncoder, RgbaImage,
        codecs::{gif::GifEncoder, png::PngEncoder},
    };

    #[test]
    fn default_config_matches_documented_limits() {
        let config = RemoteImageConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(20));
        assert_eq!(config.retries, 2);
        assert_eq!(config.retry_delay, Duration::from_millis(100));
        assert_eq!(config.max_retry_delay, Duration::from_secs(2));
        assert_eq!(config.max_response_bytes, 8 * 1024 * 1024);
        assert_eq!(config.max_cache_entries, 128);
        assert_eq!(config.max_cache_bytes, 64 * 1024 * 1024);
        assert_eq!(config.failure_ttl, Duration::from_secs(30));
    }

    #[test]
    fn retry_backoff_is_exponential_and_capped() {
        let config = RemoteImageConfig {
            retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_millis(500),
            ..RemoteImageConfig::default()
        };
        assert_eq!(config.retry_wait(0), Duration::from_millis(100));
        assert_eq!(config.retry_wait(1), Duration::from_millis(200));
        assert_eq!(config.retry_wait(2), Duration::from_millis(400));
        assert_eq!(config.retry_wait(3), Duration::from_millis(500));
        assert_eq!(config.retry_wait(u32::MAX), Duration::from_millis(500));
    }

    #[test]
    fn retry_classification_covers_transient_failures_only() {
        let uri = SharedUri::from("https://example.com/image.png");
        for status in [408, 429, 500, 502, 503, 504] {
            assert!(should_retry(&ImageCacheError::BadStatus {
                uri: uri.clone(),
                status: StatusCode::from_u16(status).unwrap(),
                body: String::new(),
            }));
        }
        for status in [400, 401, 403, 404, 413] {
            assert!(!should_retry(&ImageCacheError::BadStatus {
                uri: uri.clone(),
                status: StatusCode::from_u16(status).unwrap(),
                body: String::new(),
            }));
        }
        assert!(should_retry(&ImageCacheError::Io(Arc::new(
            io::Error::new(io::ErrorKind::TimedOut, "timeout")
        ))));
        assert!(!should_retry(&ImageCacheError::Asset("too large".into())));
    }

    #[test]
    fn cache_lookup_keeps_failures_until_ttl_expires() {
        let ttl = Duration::from_secs(30);
        let state = EntryState {
            result: Err(ImageCacheError::Asset("failure".into())),
            completed_at: Instant::now(),
            decoded_bytes: 0,
        };
        assert!(matches!(
            lookup_entry(Some(&state), ttl),
            CacheLookup::Ready(Err(_))
        ));

        let expired = EntryState {
            completed_at: Instant::now() - ttl - Duration::from_secs(1),
            ..state
        };
        assert!(matches!(
            lookup_entry(Some(&expired), ttl),
            CacheLookup::Expired
        ));
        assert!(matches!(lookup_entry(None, ttl), CacheLookup::Loading));
    }

    #[test]
    fn response_body_limit_accepts_exact_size_and_rejects_overflow() {
        let mut output = Vec::new();
        block_on(read_limited_body(
            &mut futures::io::Cursor::new(vec![1, 2, 3, 4]),
            4,
            &mut output,
        ))
        .unwrap();
        assert_eq!(output, vec![1, 2, 3, 4]);

        let mut output = Vec::new();
        let error = block_on(read_limited_body(
            &mut futures::io::Cursor::new(vec![1, 2, 3, 4, 5]),
            4,
            &mut output,
        ))
        .unwrap_err();
        assert!(matches!(error, ImageCacheError::Asset(_)));
    }

    fn single_frame_gif() -> Vec<u8> {
        let source = RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 255]));
        let mut bytes = Vec::new();
        GifEncoder::new(&mut bytes)
            .encode(source.as_raw(), 1, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
        bytes
    }

    #[test]
    fn animation_accepts_exact_decoded_budget() {
        let bytes = single_frame_gif();
        let frames = decode_gif_image(&bytes, 4).unwrap();
        assert_eq!(frames.len(), 1);
        assert!(matches!(
            decode_gif_image(&bytes, 3),
            Err(ImageCacheError::Asset(_))
        ));
    }

    #[test]
    fn oversized_gif_canvas_is_rejected_before_frame_allocation() {
        let mut bytes = single_frame_gif();
        bytes[6..8].copy_from_slice(&100u16.to_le_bytes());
        bytes[8..10].copy_from_slice(&100u16.to_le_bytes());
        let error = decode_gif_image(&bytes, 1024).err().unwrap();
        assert!(matches!(error, ImageCacheError::Asset(_)), "{error:?}");
    }

    #[test]
    fn oversized_svg_is_rejected_before_rasterization() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="1000"/>"#;
        assert!(matches!(
            ensure_svg_render_size(svg, 1024),
            Err(ImageCacheError::Asset(_))
        ));
        assert!(ensure_svg_render_size(svg, 20 * 1024 * 1024).is_ok());
        let wide = br#"<svg xmlns="http://www.w3.org/2000/svg" width="5000" height="100"/>"#;
        assert!(ensure_svg_render_size(wide, 5_341_184).is_ok());
        assert!(matches!(
            ensure_svg_render_size(wide, 5_341_183),
            Err(ImageCacheError::Asset(_))
        ));
    }

    #[test]
    fn static_decode_matches_gpui_bgra_layout() {
        let source = RgbaImage::from_pixel(2, 1, Rgba([10, 20, 30, 40]));
        let mut bytes = Vec::new();
        PngEncoder::new(&mut bytes)
            .write_image(source.as_raw(), 2, 1, image::ExtendedColorType::Rgba8)
            .unwrap();

        let frames = decode_static_image(&bytes, ImageFormat::Png, 1024).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(
            frames[0].buffer(),
            &ImageBuffer::from_pixel(2, 1, Rgba([30, 20, 10, 40]))
        );
        assert!(matches!(
            decode_static_image(&bytes, ImageFormat::Png, 7),
            Err(ImageCacheError::Asset(_))
        ));
    }
}
