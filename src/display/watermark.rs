//! 水印：在内容上方平铺半透明文字，不拦截鼠标事件。

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, Hsla, InspectorElementId,
    IntoElement, LayoutId, ParentElement, Pixels, ShapedLine, SharedString, Style, TextRun, Window,
    div, point, prelude::*, px, relative,
};

/// 水印包装器：包裹任意内容，在其上方平铺水印文字。
///
/// ```ignore
/// Watermark::new("内部资料 · 张三")
///     .child(div().child("被保护的正文内容"))
/// ```
pub struct Watermark {
    text: SharedString,
    font_size: Pixels,
    color: Hsla,
    gap_x: Pixels,
    gap_y: Pixels,
    children: Vec<AnyElement>,
}

impl Watermark {
    pub fn new(text: impl Into<SharedString>) -> Self {
        let m = crate::theme::with(crate::theme::metric::watermark_metric);
        Self {
            text: text.into(),
            font_size: px(m.font_size),
            color: m.color,
            gap_x: px(m.gap_x),
            gap_y: px(m.gap_y),
            children: Vec::new(),
        }
    }

    pub fn font_size(mut self, font_size: Pixels) -> Self {
        self.font_size = font_size;
        self
    }

    /// 水印颜色（建议带低透明度）
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = color.into();
        self
    }

    /// 平铺间距（水平 / 垂直）
    pub fn gap(mut self, gap_x: Pixels, gap_y: Pixels) -> Self {
        self.gap_x = gap_x;
        self.gap_y = gap_y;
        self
    }
}

impl ParentElement for Watermark {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl IntoElement for Watermark {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        div()
            .relative()
            .children(self.children)
            .child(WatermarkLayer {
                text: self.text,
                font_size: self.font_size,
                color: self.color,
                gap_x: self.gap_x,
                gap_y: self.gap_y,
            })
    }
}

/// 覆盖在内容之上的平铺文字层（绝对定位，不参与交互）
struct WatermarkLayer {
    text: SharedString,
    font_size: Pixels,
    color: Hsla,
    gap_x: Pixels,
    gap_y: Pixels,
}

impl Element for WatermarkLayer {
    type RequestLayoutState = ();
    type PrepaintState = Option<ShapedLine>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    /// 水印只是重复的装饰性文字，不进入无障碍树，避免干扰正文朗读。
    fn a11y_role(&self) -> Option<gpui::Role> {
        None
    }

    fn write_a11y_info(&self, _node: &mut gpui::accesskit::Node) {}

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.position = gpui::Position::Absolute;
        style.inset.top = relative(0.).into();
        style.inset.left = relative(0.).into();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (_window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        if self.text.is_empty() {
            return None;
        }
        let run = TextRun {
            len: self.text.len(),
            font: window.text_style().font(),
            color: self.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        Some(
            window
                .text_system()
                .shape_line(self.text.clone(), self.font_size, &[run], None),
        )
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let Some(line) = prepaint.take() else {
            return;
        };
        let line_height = self.font_size * 1.4;
        let step_x = line.width + self.gap_x;
        let step_y = line_height + self.gap_y;
        if step_x <= px(0.) || step_y <= px(0.) {
            return;
        }

        let mut y = bounds.top();
        let mut row = 0usize;
        while y < bounds.bottom() {
            // 隔行错位半个间距，观感更自然
            let offset = if row % 2 == 1 { step_x / 2. } else { px(0.) };
            let mut x = bounds.left() - offset;
            while x < bounds.right() {
                if x + line.width > bounds.left() {
                    let _ = line.paint(
                        point(x, y),
                        line_height,
                        gpui::TextAlign::default(),
                        None,
                        window,
                        cx,
                    );
                }
                x += step_x;
            }
            y += step_y;
            row += 1;
        }
    }
}
