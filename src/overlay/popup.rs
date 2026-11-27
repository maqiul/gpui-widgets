//! 浮层定位基建：触发框边界探测 + 贴框下方弹出（溢出自动翻转）。
//!
//! 供 Select / MultiSelect / DropdownMenu / Popover / DatePicker / ColorPicker 复用。
//! 用法：[`anchored`] 包装触发框（可选追加浮层），浮层内容经 [`popup_layer`]
//! 以零高度占位层锚定在触发框之后，`defer_draw` 置顶绘制。

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AnyElement, App, Bounds, DefiniteLength, Display, Div, Edges, Element, ElementId,
    FlexDirection, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Length, Pixels,
    Position, Style, Window, div, point, prelude::*, px,
};

/// 触发框布局边界的共享单元（TriggerProbe 写入，PopupLayer 读取）
pub(crate) type BoundsCell = Rc<Cell<Option<Bounds<Pixels>>>>;
pub(crate) type PopupBoundsSink = Rc<RefCell<Vec<Bounds<Pixels>>>>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PopupPlacement {
    #[default]
    Bottom,
    RightTop,
}

/// 包装触发框与浮层：触发框边界经 [`TriggerProbe`] 记录，
/// 浮层（若有）以零高度占位层锚定在触发框之后
pub(crate) fn anchored(trigger: AnyElement, popup: Option<Div>) -> Div {
    anchored_with_placement(trigger, popup, PopupPlacement::Bottom)
}

pub(crate) fn anchored_with_placement(
    trigger: AnyElement,
    popup: Option<Div>,
    placement: PopupPlacement,
) -> Div {
    anchored_with_placement_and_bounds(trigger, popup, placement, None)
}

pub(crate) fn anchored_with_placement_and_bounds(
    trigger: AnyElement,
    popup: Option<Div>,
    placement: PopupPlacement,
    popup_bounds: Option<PopupBoundsSink>,
) -> Div {
    let bounds: BoundsCell = Rc::new(Cell::new(None));
    let mut root = div().flex().flex_col().w_full().child(TriggerProbe {
        child: trigger,
        bounds: bounds.clone(),
    });
    if let Some(popup) = popup {
        root = root.child(popup_layer(popup, bounds, placement, popup_bounds));
    }
    root
}

/// 零高度占位容器：给 PopupLayer 提供与触发框同宽的确定宽度，
/// 且流中位置在触发框之后（锚点即框体底边）
pub(crate) fn popup_layer(
    content: Div,
    trigger_bounds: BoundsCell,
    placement: PopupPlacement,
    popup_bounds: Option<PopupBoundsSink>,
) -> Div {
    div().h(px(0.)).w_full().child(PopupLayer {
        content: Some(content.into_any_element()),
        trigger_bounds,
        placement,
        popup_bounds,
    })
}

/// 浮层定位层：absolute + left/right 0 从父容器继承触发框宽度，top 4px 贴框体下方；
/// 下方空间不足时翻转到触发框上方，上下都放不下才夹在窗口内；
/// defer_draw 保证浮层绘制在滚动内容之上
struct PopupLayer {
    content: Option<AnyElement>,
    trigger_bounds: BoundsCell,
    placement: PopupPlacement,
    popup_bounds: Option<PopupBoundsSink>,
}

impl Element for PopupLayer {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let content_id = self.content.as_mut().unwrap().request_layout(window, cx);
        // antd 浮层默认 offset [0, 4]，与 `paddingXXS` 同值
        let gap = px(crate::theme::with(|t| t.padding_xxs));
        let inset = match self.placement {
            PopupPlacement::Bottom => Edges {
                left: Length::Definite(px(0.).into()),
                right: Length::Definite(px(0.).into()),
                top: Length::Definite(gap.into()),
                bottom: Length::Auto,
            },
            PopupPlacement::RightTop => Edges {
                left: Length::Definite(DefiniteLength::Fraction(1.0).into()),
                right: Length::Auto,
                top: Length::Definite(px(0.).into()),
                bottom: Length::Auto,
            },
        };
        let style = Style {
            position: Position::Absolute,
            inset,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..Style::default()
        };
        let layout_id = window.request_layout(style, [content_id], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let Some(content) = self.content.take() else {
            return;
        };
        let viewport = window.viewport_size();
        let gap = px(crate::theme::with(|t| t.padding_xxs));
        let mut shift = point(px(0.), px(0.));
        match self.placement {
            PopupPlacement::Bottom => {
                if bounds.bottom() > viewport.height {
                    let above_top = self
                        .trigger_bounds
                        .get()
                        .map(|tb| tb.origin.y - gap - bounds.size.height);
                    match above_top {
                        // 上方放得下：翻转到触发框上方，留与下方相同的间隙
                        Some(top) if top >= px(0.) => shift.y = top - bounds.origin.y,
                        // 上下都放不下：夹在窗口内
                        _ => {
                            shift.y = viewport.height - bounds.bottom();
                            if bounds.origin.y + shift.y < px(0.) {
                                shift.y = -bounds.origin.y;
                            }
                        }
                    }
                }
            }
            PopupPlacement::RightTop => {
                if bounds.right() > viewport.width {
                    let left = self.trigger_bounds.get().map(|tb| {
                        let left = tb.left() - gap - bounds.size.width;
                        left - bounds.origin.x
                    });
                    shift.x = left.unwrap_or_else(|| viewport.width - bounds.right());
                }
                if bounds.origin.x + shift.x < px(0.) {
                    shift.x = -bounds.origin.x;
                }
                if bounds.bottom() > viewport.height {
                    shift.y = viewport.height - bounds.bottom();
                    if bounds.origin.y + shift.y < px(0.) {
                        shift.y = -bounds.origin.y;
                    }
                }
            }
        }
        let absolute_offset = window.element_offset() + shift;
        if let Some(popup_bounds) = &self.popup_bounds {
            let final_bounds = Bounds {
                origin: absolute_offset + bounds.origin,
                size: bounds.size,
            };
            popup_bounds.borrow_mut().push(final_bounds);
        }
        window.defer_draw(content, absolute_offset, 1, None);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
    }
}

/// 记录触发框布局边界，供 PopupLayer 溢出翻转时计算上方空间
struct TriggerProbe {
    child: AnyElement,
    bounds: BoundsCell,
}

impl Element for TriggerProbe {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let child_id = self.child.request_layout(window, cx);
        let style = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..Style::default()
        };
        let layout_id = window.request_layout(style, [child_id], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.bounds.set(Some(bounds));
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}
