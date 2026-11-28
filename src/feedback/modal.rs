//! 模态对话框：半透明遮罩 + 居中卡片，点击遮罩或 ✕ 关闭。
//!
//! 根视图必须 `.relative()`（提供覆盖整窗的定位祖先），Modal 作为根视图最后一个
//! 子元素、滚动容器之外渲染：遮罩 `absolute inset_0` 按树序盖在所有先绘兄弟之上。

use std::rc::Rc;

use gpui::{
    AnyElement, App, CursorStyle, ElementId, IntoElement, MouseButton, ParentElement, Pixels, Role,
    SharedString, Window, div, prelude::*, px,
};

use crate::a11y::{self, Semantics};
use crate::{
    motion, theme,
    theme::metric::{ModalMetric, modal_style},
};

/// 通用模态对话框。
///
/// ```ignore
/// // render 中（根视图需 .relative()；Modal 放最后、滚动容器之外）：
/// if self.show_dialog {
///     let entity = cx.entity();
///     root = root.child(
///         Modal::new("确认操作")
///             .on_close(move |_, cx| {
///                 entity.update(cx, |this, cx| { this.show_dialog = false; cx.notify(); });
///             })
///             .child(div().child("确定要执行吗？"))
///             .footer(Button::new("确定").primary()),
///     );
/// }
/// ```
pub struct Modal {
    title: SharedString,
    id: Option<SharedString>,
    children: Vec<AnyElement>,
    footer: Option<AnyElement>,
    width: Pixels,
    closable: bool,
    mask_closable: bool,
    on_close: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    /// 关闭图标工厂：传入关闭按钮的 group 名，返回自定义图标元素（如 SVG）。
    /// 未设置时回退到内置 ✕ 字形。
    close_icon_factory: Option<Rc<dyn Fn(SharedString) -> AnyElement>>,
}

impl Modal {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            id: None,
            children: Vec::new(),
            footer: None,
            width: px(520.),
            closable: true,
            mask_closable: true,
            on_close: None,
            close_icon_factory: None,
        }
    }

    /// 实例唯一 id，用于派生关闭钮与遮罩的 element id。
    ///
    /// 不传则回退**标题**（`modal-close-{title}` / `modal-backdrop-{title}`），
    /// 因此标题不同的叠层弹窗天然互不干扰。gpui 的 `GlobalElementId` 是祖先 id 栈，
    /// 固定 id 会让两层弹窗的 ✕ 共用 `pending_mouse_down` / `clicked_state`：
    /// 外层 capture 先跑并清掉共享 pending，内层 ✕ 点了没反应、外层反而进 active 态。
    ///
    /// **标题相同的叠层弹窗必须用本旋钮传入彼此不同的 id。**
    /// id 必须**跨帧稳定**：Modal 是值组件、宿主每帧重建，自增计数器会让
    /// MouseDown 写入的状态在下一帧换 id 后丢失，点击彻底失效。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 卡片宽度，默认 520px（antd Modal 默认宽）
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
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

    /// 自定义右上角关闭图标（如原版 SVG）。工厂收到关闭按钮的 group 名，
    /// 返回的元素可用 `group_hover(group, …)` 声明悬停色。未设置时沿用内置 ✕ 字形。
    pub fn close_icon(mut self, factory: impl Fn(SharedString) -> AnyElement + 'static) -> Self {
        self.close_icon_factory = Some(Rc::new(factory));
        self
    }

    /// 底部操作区（右对齐）
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }
}

/// 派生关闭钮与遮罩的 element id（纯函数）：实例 id 优先，未传则回退标题。
fn element_ids(id: Option<&str>, title: &str) -> (String, String) {
    let key = id.unwrap_or(title);
    (
        format!("modal-close-{key}"),
        format!("modal-backdrop-{key}"),
    )
}

impl ParentElement for Modal {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl IntoElement for Modal {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        let m = theme::with(ModalMetric::resolve);
        let style = theme::with(modal_style);
        let (shadow, title_weight) =
            theme::with(|t| (t.shadow_secondary.0.clone(), t.font_weight_strong));
        let on_close = self.on_close.clone();
        let close_icon_factory = self.close_icon_factory.clone();
        let (close_id, backdrop_id) = element_ids(self.id.as_deref(), &self.title);
        let dialog_id = format!(
            "modal-dialog-{}",
            self.id.as_deref().unwrap_or(self.title.as_ref())
        );
        let dialog_semantics = Semantics::new(Role::Dialog, self.title.clone())
            .id(dialog_id.clone())
            .focusable(true);

        let mut card = div()
            .id(ElementId::Name(dialog_id.clone().into()))
            .relative()
            .flex()
            .flex_col()
            .w(self.width)
            .px(px(m.content_padding_inline))
            .py(px(m.content_padding_block))
            .rounded(px(m.radius))
            .bg(style.content_bg)
            .shadow(shadow)
            // 卡片内的点击不冒泡到遮罩，避免误关闭
            .on_mouse_down(MouseButton::Left, |_, _, cx: &mut App| {
                cx.stop_propagation()
            })
            .child(
                div().mb(px(m.header_margin_bottom)).child(
                    div()
                        .text_size(px(m.title_size))
                        .line_height(px(m.title_line_height))
                        .font_weight(gpui::FontWeight(title_weight))
                        .text_color(style.title)
                        .child(self.title),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .text_size(px(m.body_size))
                    .line_height(px(m.body_line_height))
                    .children(self.children),
            );

        if let Some(footer) = self.footer {
            card = card.child(
                div()
                    .mt(px(m.footer_margin_top))
                    .flex()
                    .flex_row()
                    .justify_end()
                    .child(footer),
            );
        }

        if self.closable {
            if let Some(on_close) = on_close.clone() {
                let mut close_btn = div()
                    .id(ElementId::Name(close_id.clone().into()))
                    .absolute()
                    .top(px(m.close_offset))
                    .right(px(m.close_offset))
                    .w(px(m.close_size))
                    .h(px(m.close_size))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(m.close_radius))
                    .text_size(px(m.close_icon_size))
                    .line_height(px(m.close_size))
                    .text_color(style.close)
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |s| s.text_color(style.close_hover).bg(style.close_hover_bg))
                    .active(move |s| s.bg(style.close_active_bg));
                close_btn = match &close_icon_factory {
                    Some(f) => {
                        let grp: SharedString = "modal-close-grp".into();
                        close_btn.group(grp.clone()).child(f(grp))
                    }
                    None => close_btn.child("✕"),
                };
                close_btn = a11y::click_void(close_btn, move |window, cx| on_close(window, cx));
                let close_btn = Semantics::new(Role::Button, "关闭对话框")
                    .id(close_id)
                    .focusable(true)
                    .apply(close_btn);
                card = card.child(close_btn);
            }
        }
        let card = dialog_semantics.apply(card);
        let card = motion::enter(
            format!("modal-card-enter-{dialog_id}"),
            card,
            |card, progress| card.opacity(progress).mt(px((1.0 - progress) * 8.0)),
        );

        let mut backdrop = div()
            .id(ElementId::Name(backdrop_id.clone().into()))
            .absolute()
            .inset_0()
            // 阻断 hover/点击穿透到遮罩下层内容
            .occlude()
            .bg(style.mask)
            .flex()
            .items_center()
            .justify_center()
            .child(card);

        if self.mask_closable {
            if let Some(on_close) = on_close {
                backdrop = backdrop
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| on_close(window, cx));
            }
        }

        motion::enter(backdrop_id, backdrop, |backdrop, progress| {
            backdrop.opacity(progress)
        })
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::element_ids;

    #[test]
    fn modal_id_default_derives_from_title() {
        let (close, backdrop) = element_ids(None, "编辑连接");
        assert_eq!(close, "modal-close-编辑连接");
        assert_eq!(backdrop, "modal-backdrop-编辑连接");
    }

    #[test]
    fn modal_id_explicit_overrides_title() {
        let (close, backdrop) = element_ids(Some("conn-edit-7"), "编辑连接");
        assert_eq!(close, "modal-close-conn-edit-7");
        assert_eq!(backdrop, "modal-backdrop-conn-edit-7");
    }

    #[test]
    fn modal_id_stable_across_repeated_calls() {
        // 值组件每帧重建：同输入必须同输出，否则 MouseDown 写的 pending 到 MouseUp 就丢了
        for _ in 0..3 {
            assert_eq!(element_ids(None, "确认"), element_ids(None, "确认"));
            assert_eq!(
                element_ids(Some("k"), "确认"),
                element_ids(Some("k"), "确认")
            );
        }
    }

    #[test]
    fn modal_id_distinct_titles_do_not_collide() {
        // 叠层弹窗（连接管理器 + 编辑连接）：这是本轮要修的场景
        let outer = element_ids(None, "连接管理器");
        let inner = element_ids(None, "编辑连接");
        assert_ne!(outer.0, inner.0);
        assert_ne!(outer.1, inner.1);
    }

    #[test]
    fn modal_id_close_and_backdrop_share_key() {
        let (close, backdrop) = element_ids(Some("k"), "标题");
        assert_eq!(close.strip_prefix("modal-close-"), Some("k"));
        assert_eq!(backdrop.strip_prefix("modal-backdrop-"), Some("k"));
        assert_ne!(close, backdrop);
    }

    #[test]
    fn modal_id_same_title_still_collides() {
        // 已知限制：同标题叠层必须由宿主传 .id()，派生函数无法凭空区分
        assert_eq!(element_ids(None, "确认"), element_ids(None, "确认"));
        assert_ne!(
            element_ids(Some("a"), "确认"),
            element_ids(Some("b"), "确认")
        );
    }

    #[test]
    fn modal_id_empty_title_keeps_prefix() {
        let (close, backdrop) = element_ids(None, "");
        assert_eq!(close, "modal-close-");
        assert_eq!(backdrop, "modal-backdrop-");
    }
}
