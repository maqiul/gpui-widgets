//! 图片控件：封装 GPUI 原生 `img` 元素，补充语义化适配模式、圆角与
//! 内置的加载中（Spinner）/ 加载失败占位。
//!
//! 支持本地文件路径、http(s) URI、SVG 以及内存图像（`RenderImage`，
//! 例如视频帧、动态纹理）。格式覆盖 png/jpg/gif/webp/avif/svg 等。
//!
//! ```
//! # use gpui::{div, px, IntoElement, Styled};
//! # use gpui_widgets::ImageView;
//! let photo = ImageView::from_file("assets/photo.png")
//!     .fit_cover()
//!     .rounded(px(8.))
//!     .w(px(220.))
//!     .h(px(140.));
//! ```
//!
//! 注意：`new("...")` 传字符串时走 GPUI 资产系统（Embedded）或 http(s) URI；
//! 本地文件请用 [`ImageView::from_file`]。

use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, Element, GlobalElementId, Hitbox, ImageSource, Img, ImgLayoutState,
    InspectorElementId, IntoElement, LayoutId, ObjectFit, ParentElement, Pixels, RenderImage, Role,
    SharedString, StyleRefinement, Styled, StyledImage, Window, div, img, px,
};

use crate::{Spinner, a11y::Semantics, theme};

/// 图片控件。尺寸/布局通过 `Styled` _fluent 接口设置（`.w()` / `.h()` / `.flex_1()` 等）。
pub struct ImageView {
    id: Option<SharedString>,
    origin: SharedString,
    img: Img,
    alt: Option<SharedString>,
}

impl ImageView {
    /// 从路径 / URI / `ImageSource` 创建。默认 `Contain` 适配，
    /// 加载中显示 Spinner，失败显示占位块。
    #[track_caller]
    pub fn new(source: impl Into<ImageSource>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "image-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            img: img(source)
                .with_loading(|| Spinner::new().size(px(20.)).into_any_element())
                .with_fallback(|| {
                    div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(theme::hover_bg())
                        .rounded(px(6.))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("图片加载失败"),
                        )
                        .into_any_element()
                }),
            alt: None,
        }
    }

    /// 实例 ID；同页多张无替代文本的图片应显式指定。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 图片替代文本；装饰图片可传空字符串。
    pub fn alt(mut self, alt: impl Into<SharedString>) -> Self {
        self.alt = Some(alt.into());
        self
    }

    /// 本地文件路径（png/jpg/svg/gif/webp 等）
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Self {
        Self::new(path.as_ref().to_path_buf())
    }

    /// 内存图像（视频帧、程序生成的纹理等）
    pub fn from_render(image: Arc<RenderImage>) -> Self {
        Self::new(ImageSource::Render(image))
    }

    /// 设置适配模式
    pub fn fit(mut self, fit: ObjectFit) -> Self {
        self.img = self.img.object_fit(fit);
        self
    }

    /// 铺满裁剪（cover）
    pub fn fit_cover(self) -> Self {
        self.fit(ObjectFit::Cover)
    }

    /// 完整包含（contain，默认）
    pub fn fit_contain(self) -> Self {
        self.fit(ObjectFit::Contain)
    }

    /// 拉伸铺满（fill）
    pub fn fit_fill(self) -> Self {
        self.fit(ObjectFit::Fill)
    }

    /// 圆角
    pub fn rounded(mut self, radius: Pixels) -> Self {
        self.img = self.img.rounded(radius);
        self
    }

    /// 灰度显示
    pub fn grayscale(mut self, grayscale: bool) -> Self {
        self.img = self.img.grayscale(grayscale);
        self
    }

    /// 自定义加载失败占位
    pub fn fallback(mut self, fallback: impl Fn() -> AnyElement + 'static) -> Self {
        self.img = self.img.with_fallback(fallback);
        self
    }

    /// 自定义加载中占位
    pub fn loading(mut self, loading: impl Fn() -> AnyElement + 'static) -> Self {
        self.img = self.img.with_loading(loading);
        self
    }
}

impl Styled for ImageView {
    fn style(&mut self) -> &mut StyleRefinement {
        self.img.style()
    }
}

impl IntoElement for ImageView {
    type Element = AccessibleImage;

    fn into_element(self) -> Self::Element {
        let label = self
            .alt
            .clone()
            .unwrap_or_else(|| SharedString::from("图片"));
        let decorative = label.is_empty();
        let owner = self.id.as_deref().unwrap_or(&self.origin);
        let semantics = Semantics::new(Role::Image, label).id(format!(
            "{owner}-{}",
            self.alt.as_deref().unwrap_or("default")
        ));
        AccessibleImage {
            img: self.img,
            semantics,
            decorative,
        }
    }
}

/// 为 GPUI 原生图片补齐替代文本；布局与绘制完全委托给 `Img`。
pub struct AccessibleImage {
    img: Img,
    semantics: Semantics,
    decorative: bool,
}

impl Styled for AccessibleImage {
    fn style(&mut self) -> &mut StyleRefinement {
        self.img.style()
    }
}

impl Element for AccessibleImage {
    type RequestLayoutState = ImgLayoutState;
    type PrepaintState = Option<Hitbox>;

    fn id(&self) -> Option<gpui::ElementId> {
        Some(self.semantics.element_id())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        self.img.source_location()
    }

    fn a11y_role(&self) -> Option<Role> {
        (!self.decorative).then_some(Role::Image)
    }

    fn write_a11y_info(&self, node: &mut gpui::accesskit::Node) {
        self.semantics.write(node);
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.img.request_layout(id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.img
            .prepaint(id, inspector_id, bounds, request_layout, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.img.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            prepaint,
            window,
            cx,
        )
    }
}
