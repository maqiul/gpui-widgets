//! 折叠面板（对应 antd Collapse）：多面板可同时展开，展开集合由父视图持有（受控）。
//! 令牌来自 antdv-next `collapse/style/index.js`；箭头默认置右（expandIconPosition=end）。

use std::rc::Rc;

use gpui::{
    AnyElement, App, CursorStyle, Div, ElementId, Hsla, IntoElement, Role, SharedString, Stateful,
    Window, canvas, div, prelude::*, px,
};

use crate::a11y;
use crate::svg_path::{paint_svg, svg_segments};
use crate::text_input::InputSize;
use crate::theme::{
    self,
    metric::{
        collapse_border, collapse_content_bg, collapse_header_bg, collapse_header_fg,
        collapse_metric,
    },
};

const ICON_RIGHT: &str = "M765.7 486.8L314.9 134.7A7.97 7.97 0 00302 141v77.3c0 4.9 2.3 9.6 6.1 12.6l360 281.1-360 281.1c-3.9 3-6.1 7.7-6.1 12.6V883c0 6.7 7.7 10.4 12.9 6.3l450.8-352.1a31.96 31.96 0 000-50.4z";
const ICON_DOWN: &str = "M884 256h-75c-5.1 0-9.9 2.5-12.9 6.6L512 654.2 227.9 262.6c-3-4.1-7.8-6.6-12.9-6.6h-75c-6.5 0-10.3 7.4-6.5 12.7l352.6 486.1c12.8 17.6 39 17.6 51.7 0l352.6-486.1c3.9-5.3.1-12.7-6.4-12.7z";

fn arrow_icon(d: &'static str, size: f32, color: Hsla) -> impl IntoElement {
    let segs = svg_segments(d);
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, _| paint_svg(bounds, (0., 0., 1024., 1024.), &segs, color, window),
    )
    .w(px(size))
    .h(px(size))
    .flex_shrink_0()
}

/// 单个折叠面板（标题 + 内容）
pub struct AccordionItem {
    title: SharedString,
    content: AnyElement,
    disabled: bool,
}

impl AccordionItem {
    pub fn new(title: impl Into<SharedString>, content: impl IntoElement) -> Self {
        Self {
            title: title.into(),
            content: content.into_any_element(),
            disabled: false,
        }
    }

    /// 禁用该面板：头文字 colorTextDisabled，点击不切换
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// 折叠面板组。
///
/// ```ignore
/// Accordion::new("faq", vec![
///     AccordionItem::new("什么是 GPUI？", div().child("Zed 的 GPU 加速 UI 框架")),
/// ])
/// .open(self.acc_open.clone())
/// .on_change(cx.listener(|this, open, _, cx| {
///     this.acc_open = open.clone();
///     cx.notify();
/// }))
/// ```
pub struct Accordion {
    id: ElementId,
    items: Vec<AccordionItem>,
    open: Vec<usize>,
    size: InputSize,
    on_change: Option<Rc<dyn Fn(&Vec<usize>, &mut Window, &mut App)>>,
}

impl Accordion {
    pub fn new(id: impl Into<ElementId>, items: Vec<AccordionItem>) -> Self {
        Self {
            id: id.into(),
            items,
            open: Vec::new(),
            size: InputSize::default(),
            on_change: None,
        }
    }

    /// 当前展开的面板下标集合（受控）
    pub fn open(mut self, open: Vec<usize>) -> Self {
        self.open = open;
        self
    }

    /// 尺寸档：small / middle / large（header 与 body 内边距、字号随档）
    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn on_change(
        mut self,
        listener: impl Fn(&Vec<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(listener));
        self
    }
}

impl IntoElement for Accordion {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let m = theme::with(|t| collapse_metric(t, self.size));
        let (header_bg, content_bg, border) = theme::with(|t| {
            (
                collapse_header_bg(t),
                collapse_content_bg(t),
                collapse_border(t),
            )
        });
        let root_id = format!("{:?}-accordion", self.id);
        let mut root = a11y::Semantics::new(Role::Region, "折叠面板")
            .id(root_id.clone())
            .apply(div().id(ElementId::Name(root_id.into())))
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(m.radius))
            .border_1()
            .border_color(border)
            .bg(header_bg)
            .overflow_hidden();

        for (ix, item) in self.items.into_iter().enumerate() {
            let is_open = self.open.contains(&ix);
            let disabled = item.disabled;
            let fg = theme::with(|t| collapse_header_fg(t, disabled));
            let on_change = self.on_change.clone();
            let mut next = self.open.clone();
            if is_open {
                next.retain(|i| *i != ix);
            } else {
                next.push(ix);
            }

            let head_id = format!("{:?}-head-{ix}", self.id);
            let mut header = a11y::Semantics::new(Role::Button, item.title.clone())
                .id(head_id.clone())
                .expanded(is_open)
                .disabled(disabled)
                .focusable(!disabled && self.on_change.is_some())
                .apply(div().id(ElementId::Name(head_id.into())))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .pl(px(m.header_pad_left))
                .pr(px(m.header_pad_h))
                .py(px(m.header_pad_v))
                .text_size(px(m.font))
                .line_height(px(m.font_height))
                .text_color(fg)
                .child(div().flex_1().child(item.title.clone()))
                .child(div().ml(px(m.icon_gap)).child(arrow_icon(
                    if is_open { ICON_DOWN } else { ICON_RIGHT },
                    m.icon_font,
                    fg,
                )));
            if disabled {
                header = header.cursor(CursorStyle::OperationNotAllowed);
            } else {
                header = header.cursor(CursorStyle::PointingHand);
                if let Some(on_change) = on_change {
                    header = a11y::click_void(header, move |window, cx| {
                        on_change(&next, window, cx);
                    });
                }
            }

            let mut panel = div().flex().flex_col().child(header);
            if ix > 0 {
                panel = panel.border_t_1().border_color(border);
            }
            if is_open {
                panel = panel.child(
                    a11y::Semantics::new(Role::Region, item.title.clone())
                        .id(format!("{:?}-panel-{ix}", self.id))
                        .apply(
                            div().id(ElementId::Name(format!("{:?}-panel-{ix}", self.id).into())),
                        )
                        .border_t_1()
                        .border_color(border)
                        .bg(content_bg)
                        .px(px(m.body_pad_h))
                        .py(px(m.body_pad_v))
                        .text_size(px(m.font))
                        .line_height(px(m.font_height))
                        .text_color(theme::text())
                        .child(item.content),
                );
            }
            root = root.child(panel);
        }

        root
    }
}
