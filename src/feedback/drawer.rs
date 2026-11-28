//! 抽屉：从右侧（或左侧）滑出的覆盖面板，半透明遮罩，点击遮罩或 ✕ 关闭。
//!
//! 与 Modal 相同：应在根视图中、滚动容器之外渲染（作为最后一个子元素），
//! `deferred` 保证其绘制在所有内容之上。

use std::rc::Rc;

use gpui::{
    AnyElement, App, CursorStyle, Deferred, ElementId, IntoElement, MouseButton, ParentElement,
    Pixels, Role, SharedString, Window, deferred, div, prelude::*, px,
};

use crate::theme::metric::{DrawerMetric, drawer_style};
use crate::{
    a11y::{self, Semantics},
    controls::scrollbar::VerticalScroll,
    motion, theme,
};

/// 抽屉弹出方向
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawerPlacement {
    #[default]
    Right,
    Left,
}

/// 抽屉（默认右侧，可 `.placement(DrawerPlacement::Left)`）。
///
/// ```ignore
/// // render 中（根视图最后、滚动容器之外）：
/// if self.drawer_open {
///     let entity = cx.entity();
///     root = root.child(
///         Drawer::new("筛选条件")
///             .on_close(move |_, cx| {
///                 entity.update(cx, |this, cx| { this.drawer_open = false; cx.notify(); });
///             })
///             .child(div().child("抽屉内容"))
///             .footer(Button::new("应用").primary()),
///     );
/// }
/// ```
pub struct Drawer {
    title: SharedString,
    id: Option<SharedString>,
    children: Vec<AnyElement>,
    footer: Option<AnyElement>,
    width: Pixels,
    placement: DrawerPlacement,
    closable: bool,
    mask_closable: bool,
    on_close: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl Drawer {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            id: None,
            children: Vec::new(),
            footer: None,
            width: px(378.),
            placement: DrawerPlacement::default(),
            closable: true,
            mask_closable: true,
            on_close: None,
        }
    }

    /// 实例唯一 id，用于派生关闭钮、滚动正文与遮罩的 element id。
    ///
    /// 不传则回退**标题**（`drawer-close-{title}` / `drawer-body-{title}` /
    /// `drawer-backdrop-{title}`），因此标题不同的叠层抽屉天然互不干扰。
    /// gpui 的 `GlobalElementId` 是祖先 id 栈，固定 id 会让两层抽屉的 ✕ 共用
    /// `pending_mouse_down` / `clicked_state`（外层 capture 先清掉共享 pending ⇒ 内层 ✕ 失效），
    /// 正文滚动位置也会互相串。
    ///
    /// **标题相同的叠层抽屉必须用本旋钮传入彼此不同的 id。**
    /// id 必须**跨帧稳定**：Drawer 是值组件、宿主每帧重建，自增计数器会让
    /// MouseDown 写入的状态在下一帧换 id 后丢失，点击彻底失效。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 面板宽度，默认 378px（antd Drawer 默认宽）
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    /// 弹出方向（默认右侧）
    pub fn placement(mut self, placement: DrawerPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// 是否显示右上角 ✕（默认 true）
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// 点击遮罩是否关闭（默认 true）
    pub fn mask_closable(mut self, mask_closable: bool) -> Self {
        self.mask_closable = mask_closable;
        self
    }

    /// 关闭回调（点击遮罩或 ✕ 时触发）
    pub fn on_close(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(listener));
        self
    }

    /// 底部操作区（右对齐）
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }
}

/// 派生关闭钮、滚动正文与遮罩的 element id（纯函数）：实例 id 优先，未传则回退标题。
fn element_ids(id: Option<&str>, title: &str) -> (String, String, String) {
    let key = id.unwrap_or(title);
    (
        format!("drawer-close-{key}"),
        format!("drawer-body-{key}"),
        format!("drawer-backdrop-{key}"),
    )
}

impl ParentElement for Drawer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl IntoElement for Drawer {
    type Element = Deferred;

    fn into_element(self) -> Self::Element {
        let m = theme::with(DrawerMetric::resolve);
        let style = theme::with(drawer_style);
        let placement = self.placement;
        let width = self.width;
        let (shadow, title_weight) =
            theme::with(|t| (t.shadow_secondary.0.clone(), t.font_weight_strong));
        let on_close = self.on_close.clone();
        let (close_id, body_id, backdrop_id) = element_ids(self.id.as_deref(), &self.title);
        let dialog_id = format!(
            "drawer-dialog-{}",
            self.id.as_deref().unwrap_or(self.title.as_ref())
        );
        let dialog_semantics = Semantics::new(Role::Dialog, self.title.clone())
            .id(dialog_id.clone())
            .focusable(true);
        let mut header = div()
            .flex()
            .flex_row()
            .items_center()
            .px(px(m.header_padding_inline))
            .py(px(m.header_padding_block))
            .border_b(px(m.border_width))
            .border_color(style.border);

        if self.closable {
            if let Some(on_close) = on_close.clone() {
                let close = div()
                    .id(ElementId::Name(close_id.clone().into()))
                    .mr(px(m.close_gap))
                    .w(px(m.close_size))
                    .h(px(m.close_size))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(m.close_radius))
                    .text_size(px(m.header_font_size))
                    .line_height(px(m.header_font_size))
                    .text_color(style.close)
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |s| s.text_color(style.close_hover).bg(style.close_hover_bg))
                    .active(move |s| s.bg(style.close_active_bg))
                    .child("✕");
                let close = a11y::click_void(close, move |window, cx| on_close(window, cx));
                let close = Semantics::new(Role::Button, "关闭抽屉")
                    .id(close_id)
                    .focusable(true)
                    .apply(close);
                header = header.child(close);
            }
        }

        header = header.child(
            div()
                .flex_1()
                .text_size(px(m.header_font_size))
                .line_height(px(m.header_line_height))
                .font_weight(gpui::FontWeight(title_weight))
                .text_color(style.text)
                .child(self.title),
        );

        let mut panel = div()
            .id(ElementId::Name(dialog_id.clone().into()))
            .flex()
            .flex_col()
            .w(self.width)
            .h_full()
            .bg(style.bg)
            .shadow(shadow)
            // 面板内的点击不冒泡到遮罩，避免误关闭
            .on_mouse_down(MouseButton::Left, |_, _, cx: &mut App| {
                cx.stop_propagation()
            })
            .child(header)
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.))
                    .flex()
                    .flex_row()
                    .child(VerticalScroll::new(
                        ElementId::Name(body_id.into()),
                        div()
                            .flex()
                            .flex_col()
                            .p(px(m.body_padding))
                            .children(self.children)
                            .into_any_element(),
                    )),
            );
        if let Some(footer) = self.footer {
            panel = panel.child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .px(px(m.footer_padding_inline))
                    .py(px(m.footer_padding_block))
                    .border_t(px(m.border_width))
                    .border_color(style.border)
                    .child(footer),
            );
        }
        let panel = dialog_semantics.apply(panel);
        let panel = motion::enter(
            format!("drawer-panel-enter-{dialog_id}"),
            panel,
            move |panel, progress| match placement {
                DrawerPlacement::Right => panel
                    .opacity(progress)
                    .mr(px(-(1.0 - progress) * f32::from(width))),
                DrawerPlacement::Left => panel
                    .opacity(progress)
                    .ml(px(-(1.0 - progress) * f32::from(width))),
            },
        );

        let mut backdrop = div()
            .id(ElementId::Name(backdrop_id.clone().into()))
            .absolute()
            .inset_0()
            // 阻断 hover/点击穿透到遮罩下层内容
            .occlude()
            .overflow_hidden()
            .bg(style.mask)
            .flex()
            .flex_row();
        backdrop = match self.placement {
            DrawerPlacement::Right => backdrop.justify_end(),
            DrawerPlacement::Left => backdrop.justify_start(),
        }
        .child(panel);
        if self.mask_closable {
            if let Some(on_close) = on_close {
                backdrop = backdrop
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| on_close(window, cx));
            }
        }

        deferred(motion::enter(
            backdrop_id,
            backdrop,
            |backdrop, progress| backdrop.opacity(progress),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::element_ids;

    #[test]
    fn drawer_id_default_derives_from_title() {
        let (close, body, backdrop) = element_ids(None, "传输列表");
        assert_eq!(close, "drawer-close-传输列表");
        assert_eq!(body, "drawer-body-传输列表");
        assert_eq!(backdrop, "drawer-backdrop-传输列表");
    }

    #[test]
    fn drawer_id_explicit_overrides_title() {
        let (close, body, backdrop) = element_ids(Some("sftp-3"), "传输列表");
        assert_eq!(close, "drawer-close-sftp-3");
        assert_eq!(body, "drawer-body-sftp-3");
        assert_eq!(backdrop, "drawer-backdrop-sftp-3");
    }

    #[test]
    fn drawer_id_stable_across_repeated_calls() {
        for _ in 0..3 {
            assert_eq!(element_ids(None, "详情"), element_ids(None, "详情"));
            assert_eq!(
                element_ids(Some("k"), "详情"),
                element_ids(Some("k"), "详情")
            );
        }
    }

    #[test]
    fn drawer_id_distinct_titles_do_not_collide() {
        let outer = element_ids(None, "连接管理器");
        let inner = element_ids(None, "编辑连接");
        assert_ne!(outer.0, inner.0);
        assert_ne!(outer.1, inner.1);
        assert_ne!(outer.2, inner.2);
    }

    #[test]
    fn drawer_id_three_ids_share_key_and_differ() {
        let (close, body, backdrop) = element_ids(Some("k"), "标题");
        assert_eq!(close.strip_prefix("drawer-close-"), Some("k"));
        assert_eq!(body.strip_prefix("drawer-body-"), Some("k"));
        assert_eq!(backdrop.strip_prefix("drawer-backdrop-"), Some("k"));
        assert_ne!(close, body);
        assert_ne!(body, backdrop);
        assert_ne!(close, backdrop);
    }

    #[test]
    fn drawer_id_body_unique_per_instance() {
        // 滚动正文 id 也须唯一，否则叠层抽屉共享滚动位置
        assert_ne!(element_ids(None, "甲").1, element_ids(None, "乙").1);
    }

    #[test]
    fn drawer_id_same_title_still_collides() {
        assert_eq!(element_ids(None, "详情"), element_ids(None, "详情"));
        assert_ne!(
            element_ids(Some("a"), "详情"),
            element_ids(Some("b"), "详情")
        );
    }
}
