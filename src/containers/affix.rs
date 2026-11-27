//! 吸顶容器：观察 ScrollHandle，在滚动越过阈值后把子元素固定到滚动视口顶部或底部。

use gpui::{
    AnyElement, App, Bounds, ElementId, IntoElement, Pixels, ScrollHandle, Window, anchored,
    deferred, div, point, prelude::*, px,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct AffixState {
    measured: bool,
    height: f32,
    width: f32,
    doc_left: f32,
    doc_top: f32,
    stuck: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum AffixEdge {
    #[default]
    Top,
    Bottom,
}

fn affix_is_stuck(
    doc_top: f32,
    height: f32,
    scroll_y: f32,
    viewport_top: f32,
    viewport_bottom: f32,
    edge: AffixEdge,
    offset: f32,
) -> bool {
    match edge {
        AffixEdge::Top => doc_top + scroll_y < viewport_top + offset,
        AffixEdge::Bottom => {
            let normal_top = doc_top + scroll_y;
            let normal_bottom = normal_top + height;
            let threshold = viewport_bottom - offset;
            normal_top < threshold && normal_bottom > threshold
        }
    }
}

/// 吸顶/吸底容器。调用方须在滚动容器上使用同一个 `ScrollHandle`：
/// `overflow_y_scroll().track_scroll(&scroll)`。
#[derive(IntoElement)]
pub struct Affix {
    id: ElementId,
    child: AnyElement,
    scroll: ScrollHandle,
    edge: AffixEdge,
    offset: Pixels,
}

impl Affix {
    pub fn new(id: impl Into<ElementId>, child: impl IntoElement, scroll: ScrollHandle) -> Self {
        Self {
            id: id.into(),
            child: child.into_any_element(),
            scroll,
            edge: AffixEdge::Top,
            offset: px(0.),
        }
    }

    pub fn offset_top(mut self, offset: impl Into<Pixels>) -> Self {
        self.edge = AffixEdge::Top;
        self.offset = offset.into();
        self
    }

    pub fn offset_bottom(mut self, offset: impl Into<Pixels>) -> Self {
        self.edge = AffixEdge::Bottom;
        self.offset = offset.into();
        self
    }
}

impl RenderOnce for Affix {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(
            ElementId::Name(format!("{:?}-affix-state", self.id).into()),
            cx,
            |_, _| AffixState::default(),
        );
        let edge = self.edge;
        let offset = self.offset;

        let observe_state = state.clone();
        let observe_scroll = self.scroll.clone();
        let observe = move |bounds: Vec<Bounds<Pixels>>, _window: &mut Window, cx: &mut App| {
            let (Some(placeholder), Some(child)) = (bounds.first(), bounds.get(1)) else {
                return;
            };
            let current = *observe_state.read(cx);
            let scroll_offset = observe_scroll.offset();
            let viewport = observe_scroll.bounds();
            let height: f32 = child.size.height.into();
            let width: f32 = child.size.width.into();
            let (doc_left, doc_top) = if current.stuck {
                (current.doc_left, current.doc_top)
            } else {
                (
                    f32::from(placeholder.left()) - f32::from(scroll_offset.x),
                    f32::from(placeholder.top()) - f32::from(scroll_offset.y),
                )
            };
            let stuck = affix_is_stuck(
                doc_top,
                height,
                scroll_offset.y.into(),
                viewport.top().into(),
                viewport.bottom().into(),
                edge,
                offset.into(),
            );
            let next = AffixState {
                measured: true,
                height,
                width,
                doc_left,
                doc_top,
                stuck,
            };
            if next != current {
                observe_state.write(cx, next);
                cx.refresh_windows();
            }
        };

        let current = *state.read(cx);
        let viewport = self.scroll.bounds();
        let scroll_offset = self.scroll.offset();
        let mut root = div()
            .relative()
            .w_full()
            .flex_shrink_0()
            .on_children_prepainted(observe);
        root = root.child(
            div()
                .w_full()
                .h(px(if current.measured { current.height } else { 0. })),
        );

        let positioned = if !current.measured {
            anchored()
        } else {
            let y = if current.stuck {
                match edge {
                    AffixEdge::Top => viewport.top() + offset,
                    AffixEdge::Bottom => viewport.bottom() - offset - px(current.height),
                }
            } else {
                px(current.doc_top) + scroll_offset.y
            };
            anchored().position(point(px(current.doc_left) + scroll_offset.x, y))
        };
        let mut child_wrap = div();
        if current.measured {
            child_wrap = child_wrap.w(px(current.width));
        }
        root = root.child(deferred(positioned.child(child_wrap.child(self.child))));
        root
    }
}

#[cfg(test)]
mod tests {
    use super::{AffixEdge, affix_is_stuck};

    #[test]
    fn top_affix_switches_at_offset_boundary() {
        assert!(!affix_is_stuck(
            100.,
            30.,
            -20.,
            0.,
            200.,
            AffixEdge::Top,
            8.
        ));
        assert!(affix_is_stuck(
            100.,
            30.,
            -100.,
            0.,
            200.,
            AffixEdge::Top,
            8.
        ));
    }

    #[test]
    fn bottom_affix_switches_at_offset_boundary() {
        assert!(!affix_is_stuck(
            100.,
            30.,
            0.,
            0.,
            200.,
            AffixEdge::Bottom,
            8.
        ));
        assert!(affix_is_stuck(
            100.,
            30.,
            80.,
            0.,
            200.,
            AffixEdge::Bottom,
            8.
        ));
        assert!(!affix_is_stuck(
            100.,
            30.,
            150.,
            0.,
            200.,
            AffixEdge::Bottom,
            8.
        ));
    }
}
