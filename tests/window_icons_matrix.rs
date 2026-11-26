//! TitleBar / Modal 图标工厂旋钮的加性冒烟与状态矩阵。
//!
//! 两个旋钮都只占渲染槽：未设置时回退内置字形（─ ❐ □ ✕ / ✕），逐像素不变；
//! 设置后由工厂产出自定义元素（如 SVG）。这里验证枚举取值、加性建容器、
//! 以及 Modal 工厂确实拿到稳定的 group 名（供 `group_hover` 声明悬停色）。

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{InteractiveElement, IntoElement, ParentElement, SharedString, div};
use gpui_widgets::{Modal, TitleBar, WindowIconKind};

#[test]
fn window_icon_kind_matrix() {
    let all = [
        WindowIconKind::Minimize,
        WindowIconKind::Maximize,
        WindowIconKind::Restore,
        WindowIconKind::Close,
    ];
    // 四枚互不相同，且 Copy/Clone 后仍相等。
    for (i, a) in all.iter().enumerate() {
        assert_eq!(*a, *a);
        #[allow(clippy::clone_on_copy)]
        let copied = a.clone();
        assert_eq!(copied, *a);
        for (j, b) in all.iter().enumerate() {
            assert_eq!(i == j, a == b, "{a:?} vs {b:?}");
        }
    }
}

#[test]
fn titlebar_window_icons_knob_is_additive() {
    // 默认（字形）与工厂（自定义元素）两种形态都能独立建容器。
    let _ = TitleBar::new("应用").live(false).into_element();
    let _ = TitleBar::new("应用")
        .live(false)
        .window_icons(|kind, group| {
            let label = match kind {
                WindowIconKind::Minimize => "min",
                WindowIconKind::Maximize => "max",
                WindowIconKind::Restore => "restore",
                WindowIconKind::Close => "close",
            };
            div().id(group).child(label).into_any_element()
        })
        .into_element();
}

#[test]
fn titlebar_close_hover_bg_knob_is_additive() {
    // 默认（theme::danger）与覆盖（自定义砖红）两态都能独立建容器。
    let _ = TitleBar::new("应用").live(false).into_element();
    let _ = TitleBar::new("应用")
        .live(false)
        .close_hover_bg(gpui::rgb(0xc42b1c).into())
        .into_element();
}

#[test]
fn modal_close_icon_factory_receives_stable_group() {
    let seen: Rc<RefCell<Vec<SharedString>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    let _ = Modal::new("确认")
        .on_close(|_, _| {})
        .close_icon(move |group| {
            sink.borrow_mut().push(group.clone());
            div().id(group).child("x").into_any_element()
        })
        .into_element();
    let calls = seen.borrow();
    assert_eq!(calls.len(), 1, "关闭按钮只画一次，工厂应被调用一次");
    assert_eq!(
        calls[0], "modal-close-grp",
        "group 名须稳定，供 group_hover 绑定"
    );
}

#[test]
fn modal_default_close_still_builds() {
    // 未设置 close_icon：回退 ✕ 字形，照常建容器。
    let _ = Modal::new("确认").on_close(|_, _| {}).into_element();
    // closable(false) 时不画关闭按钮，工厂不该被调用。
    let seen: Rc<RefCell<Vec<SharedString>>> = Rc::new(RefCell::new(Vec::new()));
    let sink = seen.clone();
    let _ = Modal::new("确认")
        .closable(false)
        .on_close(|_, _| {})
        .close_icon(move |group| {
            sink.borrow_mut().push(group.clone());
            div().child("x").into_any_element()
        })
        .into_element();
    assert!(seen.borrow().is_empty(), "closable=false 不应调用图标工厂");
}
