//! 头像控件：圆形 / 方形背景 + 名字首字符。
//!
//! 对齐 antdv-next Avatar：默认底 `colorTextPlaceholder`、白色文字、字号 14。

use std::{cell::Cell, rc::Rc, sync::Arc};

use gpui::{
    App, Asset, ElementId, Hsla, ImageCacheError, ImageSource, ImgResourceLoader, IntoElement,
    ObjectFit, Pixels, RenderImage, RenderOnce, Role, SharedString, StyledImage, Window, div, img,
    prelude::*, px,
};

use crate::a11y::Semantics;
use crate::theme::{self, Tokens, metric};

/// antd Avatar 尺寸：small 24 / default 32 / large 40。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarSize {
    Small,
    Default,
    Large,
}

impl AvatarSize {
    fn index(self) -> usize {
        match self {
            AvatarSize::Small => 0,
            AvatarSize::Default => 1,
            AvatarSize::Large => 2,
        }
    }
}

/// 头像外观。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvatarStyle {
    pub bg: Hsla,
    pub text: Hsla,
    pub size: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
}

pub fn avatar_state(tokens: &Tokens, size: AvatarSize, square: bool) -> AvatarStyle {
    let m = metric::AvatarMetric::resolve(tokens);
    let ix = size.index();
    AvatarStyle {
        bg: tokens.color_text_placeholder,
        text: tokens.color_text_light_solid,
        size: m.size[ix],
        radius: if square {
            m.radius[ix]
        } else {
            m.size[ix] / 2.0
        },
        font_size: m.font_size,
        line_height: m.line_height,
    }
}

/// 自定义头像边长时，文字与行高按预设尺寸等比缩放。
pub fn avatar_scaled_text_metrics(style: AvatarStyle, size: f32) -> (f32, f32) {
    let scale = if style.size > 0. {
        size.max(0.) / style.size
    } else {
        1.
    };
    (style.font_size * scale, style.line_height * scale)
}

fn avatar_source_key(source: &ImageSource) -> String {
    match source {
        ImageSource::Resource(resource) => format!("resource:{resource:?}"),
        ImageSource::Render(image) => format!("render:{:p}", Arc::as_ptr(image)),
        ImageSource::Image(image) => format!("image:{:p}", Arc::as_ptr(image)),
        ImageSource::Custom(loader) => format!("custom:{:p}", Arc::as_ptr(loader)),
    }
}

struct AvatarImageDecoder;

impl Asset for AvatarImageDecoder {
    type Source = Arc<gpui::Image>;
    type Output = Result<Arc<RenderImage>, ImageCacheError>;

    fn load(
        source: Self::Source,
        cx: &mut App,
    ) -> impl std::future::Future<Output = Self::Output> + Send + 'static {
        let renderer = cx.svg_renderer();
        async move { source.to_image_data(renderer).map_err(Into::into) }
    }
}

fn load_avatar_image(
    source: &ImageSource,
    window: &mut Window,
    cx: &mut App,
) -> Option<Result<Arc<RenderImage>, ImageCacheError>> {
    match source {
        ImageSource::Resource(resource) => window.use_asset::<ImgResourceLoader>(resource, cx),
        ImageSource::Render(image) => Some(Ok(image.clone())),
        ImageSource::Image(image) => window.use_asset::<AvatarImageDecoder>(image, cx),
        ImageSource::Custom(loader) => loader(window, cx),
    }
}

/// 通用头像。
///
/// ```ignore
/// Avatar::new("张三")                          // 默认 32px 圆形
/// Avatar::new("Alice").preset(AvatarSize::Large)
/// Avatar::new("Bob").size(px(48.))
/// ```
#[derive(IntoElement)]
pub struct Avatar {
    id: Option<SharedString>,
    origin: SharedString,
    name: SharedString,
    preset: AvatarSize,
    size: Option<Pixels>,
    square: bool,
    image: Option<ImageSource>,
    image_error: bool,
    on_image_error: Option<Rc<dyn Fn(&ImageCacheError, &mut Window, &mut App)>>,
}

impl Avatar {
    #[track_caller]
    pub fn new(name: impl Into<SharedString>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "avatar-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            name: name.into(),
            preset: AvatarSize::Default,
            size: None,
            square: false,
            image: None,
            image_error: false,
            on_image_error: None,
        }
    }

    /// 实例 ID；同名头像并列显示时应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 自定义边长（antd 的 `style={{ width, height }}`）
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
        self
    }

    pub fn preset(mut self, preset: AvatarSize) -> Self {
        self.preset = preset;
        self
    }

    /// 方形头像（圆角矩形，默认圆形）
    pub fn square(mut self) -> Self {
        self.square = true;
        self
    }

    /// 图片内容；加载中或失败时回退到名字首字符。
    pub fn image(mut self, source: impl Into<ImageSource>) -> Self {
        self.image = Some(source.into());
        self
    }

    /// 外部控制图片失败态；为 `true` 时强制显示首字符回退。
    pub fn image_error(mut self, error: bool) -> Self {
        self.image_error = error;
        self
    }

    /// 图片加载失败通知；适合驱动受控的 `image_error` 状态。
    pub fn on_image_error(
        mut self,
        listener: impl Fn(&ImageCacheError, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_image_error = Some(Rc::new(listener));
        self
    }

    fn initial(&self) -> String {
        self.name
            .chars()
            .next()
            .map(|c| c.to_string())
            .unwrap_or_default()
    }
}

impl RenderOnce for Avatar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = theme::with(|t| avatar_state(t, self.preset, self.square));
        let size = self.size.unwrap_or(px(style.size));
        let radius = if self.square {
            px(style.radius)
        } else {
            size / 2.0
        };
        let (font_size, line_height) = avatar_scaled_text_metrics(style, f32::from(size));
        let owner = self.id.as_deref().unwrap_or(&self.origin);
        let semantics =
            Semantics::new(Role::Image, self.name.clone()).id(format!("{}-{}", owner, self.name));
        let element_id = semantics.element_id();
        let mut root = div()
            .id(element_id)
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .size(size)
            .rounded(radius)
            .overflow_hidden()
            .bg(style.bg)
            .text_color(style.text)
            .text_size(px(font_size))
            .line_height(px(line_height));

        let initial = self.initial();
        if let Some(source) = self.image.filter(|_| !self.image_error) {
            let notified = window
                .use_keyed_state(
                    ElementId::Name(format!("{owner}-image-{}", avatar_source_key(&source)).into()),
                    cx,
                    |_, _| Rc::new(Cell::new(false)),
                )
                .read(cx)
                .clone();
            let on_error = self.on_image_error.clone();
            let loading_initial = initial.clone();
            let fallback_initial = initial;
            let image = img(move |window: &mut Window, cx: &mut App| {
                let result = load_avatar_image(&source, window, cx);
                if let Some(Err(error)) = &result
                    && !notified.replace(true)
                    && let Some(on_error) = on_error.clone()
                {
                    let error = error.clone();
                    let on_error = on_error.clone();
                    window.defer(cx, move |window, cx| on_error(&error, window, cx));
                }
                result
            })
            .object_fit(ObjectFit::Cover)
            .size_full()
            .rounded(radius)
            .with_loading(move || {
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(loading_initial.clone())
                    .into_any_element()
            })
            .with_fallback(move || {
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(fallback_initial.clone())
                    .into_any_element()
            });
            root = root.child(image);
        } else {
            root = root.child(initial);
        }

        semantics.apply(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_uses_first_unicode_scalar() {
        assert_eq!(Avatar::new("张三").initial(), "张");
        assert_eq!(Avatar::new("Alice").initial(), "A");
        assert!(Avatar::new("").initial().is_empty());
    }

    #[test]
    fn custom_size_scales_text_metrics() {
        let style = AvatarStyle {
            bg: gpui::black(),
            text: gpui::white(),
            size: 32.,
            radius: 16.,
            font_size: 14.,
            line_height: 22.,
        };
        assert_eq!(avatar_scaled_text_metrics(style, 32.), (14., 22.));
        assert_eq!(avatar_scaled_text_metrics(style, 48.), (21., 33.));
        assert_eq!(avatar_scaled_text_metrics(style, 16.), (7., 11.));
    }

    #[test]
    fn resource_source_key_is_stable() {
        let a = avatar_source_key(&ImageSource::from("avatar.png"));
        let b = avatar_source_key(&ImageSource::from("avatar.png"));
        let c = avatar_source_key(&ImageSource::from("other.png"));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
