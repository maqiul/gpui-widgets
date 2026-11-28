use std::{
    io,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context as TaskContext, Poll},
    time::Duration,
};

use futures::{AsyncReadExt, future::BoxFuture};
use gpui::http_client::{AsyncBody, HttpClient, Request, Response, Url, http::HeaderValue};
use gpui::{
    App, AppContext, Bounds, Context, Entity, IntoElement, ParentElement, Styled, Window,
    WindowBounds, WindowOptions, div, px, size,
};
use gpui_platform::application;
use gpui_widgets::{ImageView, RemoteImageCache, RemoteImageConfig, Title, remote_image_cache};

const IMAGE: &[u8] = include_bytes!("assets/sample.svg");
static RETRY_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static TIMEOUT_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static USER_AGENT: HeaderValue = HeaderValue::from_static("remote-image-demo");

struct DemoHttpClient;

struct PendingReader;

impl futures::AsyncRead for PendingReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut TaskContext<'_>,
        _buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Pending
    }
}

impl HttpClient for DemoHttpClient {
    fn user_agent(&self) -> Option<&HeaderValue> {
        Some(&USER_AGENT)
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    fn send(
        &self,
        request: Request<AsyncBody>,
    ) -> BoxFuture<'static, gpui::Result<Response<AsyncBody>>> {
        Box::pin(async move {
            let path = request.uri().path();
            let response = match path {
                "/ok.svg" => Response::builder()
                    .status(200)
                    .header("Content-Type", "image/svg+xml")
                    .body(AsyncBody::from(IMAGE)),
                "/retry.svg" => {
                    let attempt = RETRY_ATTEMPTS.fetch_add(1, Ordering::SeqCst);
                    eprintln!("retry.svg attempt {}", attempt + 1);
                    if attempt == 0 {
                        Response::builder()
                            .status(503)
                            .body(AsyncBody::from("try again"))
                    } else {
                        Response::builder()
                            .status(200)
                            .header("Content-Type", "image/svg+xml")
                            .body(AsyncBody::from(IMAGE))
                    }
                }
                "/missing.svg" => Response::builder()
                    .status(404)
                    .body(AsyncBody::from("missing")),
                "/oversized.svg" => Response::builder().status(200).body(AsyncBody::from_reader(
                    futures::io::repeat(0).take(3 * 1024 * 1024),
                )),
                "/slow.svg" => {
                    let attempt = TIMEOUT_ATTEMPTS.fetch_add(1, Ordering::SeqCst);
                    eprintln!("slow.svg attempt {}", attempt + 1);
                    Response::builder()
                        .status(200)
                        .body(AsyncBody::from_reader(PendingReader))
                }
                _ => Response::builder()
                    .status(404)
                    .body(AsyncBody::from("not found")),
            };
            Ok(response.unwrap())
        })
    }
}

struct Demo {
    cache: Entity<RemoteImageCache>,
}

impl Demo {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            cache: cx.new(|cx| {
                RemoteImageCache::with_config(
                    RemoteImageConfig::default()
                        .timeout(Duration::from_millis(200))
                        .retries(2)
                        .retry_delay(Duration::from_millis(50))
                        .max_retry_delay(Duration::from_millis(100))
                        .max_response_bytes(2 * 1024 * 1024),
                    cx,
                )
            }),
        }
    }
}

impl gpui::Render for Demo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let image = |path: &'static str, label: &'static str, note: &'static str| {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .w(px(360.))
                .p_4()
                .rounded_lg()
                .bg(gpui_widgets::theme::surface())
                .child(Title::new(3, label))
                .child(
                    ImageView::new(format!("https://demo.test{path}"))
                        .id(label)
                        .alt(label)
                        .w(px(328.))
                        .h(px(120.))
                        .fit_contain(),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(gpui_widgets::theme::text_muted())
                        .child(note),
                )
                .into_any_element()
        };

        let cards = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap_4()
            .child(image("/ok.svg", "成功", "200 · SVG 解码并缓存"))
            .child(image("/retry.svg", "重试", "首次 503，第二次 200"))
            .child(image("/missing.svg", "404", "不重试，显示失败占位"))
            .child(image(
                "/oversized.svg",
                "响应超限",
                "无 Content-Length，流式读取时拒绝",
            ))
            .child(image("/slow.svg", "超时重试", "响应体挂起，每次超时后重试"));

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .size_full()
            .bg(gpui_widgets::theme::bg())
            .child(Title::new(1, "RemoteImageCache · 超时重试与有界缓存"))
            .child(div().text_sm().text_color(gpui_widgets::theme::text_muted()).child(
                "确定性 HttpClient：成功 / 首次 503 后重试成功 / 404 回退 / 流式响应超限回退 / 响应体超时重试",
            ))
            .child(remote_image_cache(self.cache.clone()).child(cards))
            .child(div().text_xs().text_color(gpui_widgets::theme::text_muted()).child(format!(
                "cache entries={} decoded_bytes={} retry_attempts={} timeout_attempts={}",
                self.cache.read(cx).len(),
                self.cache.read(cx).cached_bytes(),
                RETRY_ATTEMPTS.load(Ordering::Relaxed),
                TIMEOUT_ATTEMPTS.load(Ordering::Relaxed),
            )))
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.set_http_client(Arc::new(DemoHttpClient));
        let bounds = Bounds::centered(None, size(px(1440.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Remote Image Cache Demo".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Demo::new),
        )
        .unwrap();
    });
}
