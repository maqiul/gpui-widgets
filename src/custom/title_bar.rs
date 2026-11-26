//! 自绘标题栏：图标 + 标题 + 右侧最小化 / 最大化 / 关闭按钮。
//!
//! Windows 上走「非客户区命中测试」（[`WindowControlArea`]）：拖拽、双击最大化 /
//! 还原、贴靠布局（Snap Layouts）、系统右键菜单全部由操作系统处理，无需自绘拖拽逻辑；
//! 其他平台回退为直接调用 [`Window`] 的窗口操作。
//!
//! 使用前提：窗口需要隐藏系统标题栏，即创建窗口时设置
//! `titlebar: Some(TitlebarOptions { appears_transparent: true, .. })`，
//! 可直接用 [`transparent_titlebar_options`] 生成这份配置。
//!
//! ```ignore
//! cx.open_window(
//!     gpui_widgets::transparent_titlebar_options("我的应用"),
//!     |_, cx| cx.new(|cx| MyView::new(cx)),
//! )?;
//! // 根视图里：
//! div()
//!     .flex()
//!     .flex_col()
//!     .size_full()
//!     .child(TitleBar::new("我的应用").icon("🪟"))
//!     .child(div().flex_1().child(content))
//! ```
//!
//! 仅想在页面里做静态展示时用 `TitleBar::new("…").live(false)`，此时不注册任何
//! 窗口命中区域与点击回调。

use std::rc::Rc;

use gpui::{
    AnyElement, App, CursorStyle, Div, ElementId, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, Stateful, StatefulInteractiveElement, StyleRefinement,
    Styled, TitlebarOptions, Window, WindowControlArea, WindowOptions, div, px,
};

use crate::a11y;
use crate::theme;

/// 标题栏默认高度
const BAR_H: f32 = 36.;
/// 窗口按钮默认宽度
const BTN_W: f32 = 46.;

/// 窗口按钮图标种类，供 [`TitleBar::window_icons`] 的工厂区分当前要画哪一枚。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowIconKind {
    Minimize,
    Maximize,
    Restore,
    Close,
}

/// 生成隐藏系统标题栏的窗口配置，配合 [`TitleBar`] 使用。
///
/// 注意 `window_decorations` 与 `is_movable` 保持默认（可拖动），
/// 否则 Windows 不会接管标题栏拖拽。
pub fn transparent_titlebar_options(title: impl Into<SharedString>) -> WindowOptions {
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            appears_transparent: true,
            traffic_light_position: None,
        }),
        ..Default::default()
    }
}

/// 自绘标题栏（无状态 `RenderOnce`）。
#[derive(IntoElement)]
pub struct TitleBar {
    id: ElementId,
    title: SharedString,
    icon: Option<SharedString>,
    icon_element: Option<AnyElement>,
    /// 窗口按钮图标工厂：传入按钮种类与其 group 名，返回自定义图标元素（如 SVG）。
    /// 未设置时回退到内置字形（─ ❐ □ ✕），保持默认渲染不变。
    window_icon_factory: Option<Rc<dyn Fn(WindowIconKind, SharedString) -> AnyElement>>,
    /// 关闭按钮悬停底色覆盖；未设置时用 `theme::danger()`。
    close_hover_bg: Option<gpui::Hsla>,
    height: Pixels,
    button_width: Pixels,
    show_minimize: bool,
    show_maximize: bool,
    show_close: bool,
    draggable: bool,
    /// 是否驱动真实窗口操作（`false` 为纯展示，不注册命中区域与回调）。
    live: bool,
    extra: Vec<AnyElement>,
    extra_trailing: Vec<AnyElement>,
    on_minimize: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_maximize: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    on_close: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl TitleBar {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            id: ElementId::Name("title-bar".into()),
            title: title.into(),
            icon: None,
            icon_element: None,
            window_icon_factory: None,
            close_hover_bg: None,
            height: px(BAR_H),
            button_width: px(BTN_W),
            show_minimize: true,
            show_maximize: true,
            show_close: true,
            draggable: true,
            live: true,
            extra: Vec::new(),
            extra_trailing: Vec::new(),
            on_minimize: None,
            on_maximize: None,
            on_close: None,
        }
    }

    /// 自定义 id（同页多实例 / 多窗口时区分命中区域）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 左侧图标（建议用单个 emoji 或字形）。
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// 左侧图标槽（自定义元素，如品牌位图）；优先于 `icon` 字形。
    pub fn icon_element(mut self, icon: impl IntoElement) -> Self {
        self.icon_element = Some(icon.into_any_element());
        self
    }

    /// 自定义右侧窗口按钮图标（最小化 / 最大化 / 还原 / 关闭）。
    ///
    /// 工厂收到按钮种类与该按钮的 group 名：返回的元素可用 `group_hover(group, …)`
    /// 声明悬停态（如关闭键灰字 → 白字）。最大化按钮会按窗口状态在 [`WindowIconKind::Maximize`]
    /// 与 [`WindowIconKind::Restore`] 之间切换。未设置时沿用内置字形，渲染逐像素不变。
    pub fn window_icons(
        mut self,
        factory: impl Fn(WindowIconKind, SharedString) -> AnyElement + 'static,
    ) -> Self {
        self.window_icon_factory = Some(Rc::new(factory));
        self
    }

    /// 关闭按钮悬停底色（默认 `theme::danger()`）。原版 Windows 标题栏关闭悬停是
    /// 系统砖红 `#c42b1c` 而非主题危险色，可用此旋钮对齐。
    pub fn close_hover_bg(mut self, color: gpui::Hsla) -> Self {
        self.close_hover_bg = Some(color);
        self
    }

    /// 标题栏高度（默认 36px）。
    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.height = height.into();
        self
    }

    /// 单个窗口按钮宽度（默认 46px）。
    pub fn button_width(mut self, width: impl Into<Pixels>) -> Self {
        self.button_width = width.into();
        self
    }

    pub fn show_minimize(mut self, show: bool) -> Self {
        self.show_minimize = show;
        self
    }

    pub fn show_maximize(mut self, show: bool) -> Self {
        self.show_maximize = show;
        self
    }

    pub fn show_close(mut self, show: bool) -> Self {
        self.show_close = show;
        self
    }

    /// 是否允许拖动 / 双击最大化（默认 true）。
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// 是否驱动真实窗口（默认 true）；`false` 时仅作静态展示。
    pub fn live(mut self, live: bool) -> Self {
        self.live = live;
        self
    }

    /// 标题与按钮之间的自定义内容（搜索框、菜单等）。
    pub fn extra(mut self, extra: impl IntoElement) -> Self {
        self.extra.push(extra.into_any_element());
        self
    }

    /// 右槽：拖拽区与窗口按钮之间的自定义内容（原版 WindowBar `.wb__extra` 位）。
    /// 在拖拽区外，不需要 occlude no-drag 洞；`.extra()` 仍留在拖拽区内标题旁。
    pub fn extra_trailing(mut self, extra: impl IntoElement) -> Self {
        self.extra_trailing.push(extra.into_any_element());
        self
    }

    /// 覆盖最小化行为；设置后该按钮不再交给系统处理。
    pub fn on_minimize(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_minimize = Some(Rc::new(listener));
        self
    }

    /// 覆盖最大化 / 还原行为；设置后该按钮不再交给系统处理。
    pub fn on_maximize(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_maximize = Some(Rc::new(listener));
        self
    }

    /// 覆盖关闭行为（默认 `window.remove_window()`）。
    pub fn on_close(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let maximized = window.is_maximized();
        let icon_factory = self.window_icon_factory.clone();
        let min_id = ElementId::Name(format!("{:?}-min", self.id).into());
        let max_id = ElementId::Name(format!("{:?}-max", self.id).into());
        let close_id = ElementId::Name(format!("{:?}-close", self.id).into());

        // ── 拖拽区：图标 + 标题 + 自定义内容 ──────────────────────────
        let mut drag = div()
            .id(ElementId::Name(format!("{:?}-drag", self.id).into()))
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .flex_1()
            .min_w(px(0.))
            .h_full()
            .px_3();
        if self.live && self.draggable {
            // Windows：交给系统处理移动 / 双击最大化 / 右键菜单 / 贴靠布局
            drag = drag.window_control_area(WindowControlArea::Drag);
        }
        if let Some(icon) = self.icon_element {
            drag = drag.child(div().flex_shrink_0().child(icon));
        } else if let Some(icon) = self.icon.clone() {
            drag = drag.child(div().flex_shrink_0().text_sm().child(icon));
        }
        drag = drag.child(
            div()
                .flex_shrink_0()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme::text())
                .child(self.title.clone()),
        );
        for el in self.extra {
            // Windows 命中测试中 Drag 区会吞掉其内所有点击（HTCAPTION），
            // extra 里的交互内容用 occlude 截断命中链，形成 no-drag 洞。
            drag = drag.child(div().occlude().child(el));
        }

        // ── 右侧窗口按钮 ──────────────────────────────────────────────
        let mut controls = div()
            .flex()
            .flex_row()
            .items_center()
            .flex_shrink_0()
            .h_full();

        if self.show_minimize {
            let min_group: SharedString = format!("{:?}-grp", min_id).into();
            let mut btn = a11y::Semantics::new(
                gpui::Role::Button,
                if self.live {
                    "最小化窗口"
                } else {
                    "最小化窗口（不可用）"
                },
            )
            .id(format!("{min_id:?}"))
            .disabled(!self.live)
            .focusable(self.live)
            .apply(control_button(min_id, self.button_width, |s| {
                s.bg(theme::control_item_bg_hover())
            }));
            btn = match &icon_factory {
                Some(f) => btn
                    .group(min_group.clone())
                    .child(f(WindowIconKind::Minimize, min_group)),
                None => btn.child(glyph("─")),
            };
            if self.live {
                if let Some(on_minimize) = self.on_minimize.clone() {
                    btn = a11y::click_void(btn, move |window, cx| on_minimize(window, cx));
                } else {
                    #[cfg(target_os = "windows")]
                    {
                        btn = btn
                            .window_control_area(WindowControlArea::Min)
                            .on_a11y_action(gpui::AccessibleAction::Click, |_, window, _| {
                                window.minimize_window();
                            })
                            .on_key_down(|event, window, cx| {
                                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                    cx.stop_propagation();
                                    window.minimize_window();
                                }
                            });
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        btn = a11y::click_void(btn, |window, _| window.minimize_window());
                    }
                }
            }
            controls = controls.child(btn);
        }

        if self.show_maximize {
            let label = if maximized { "❐" } else { "□" };
            let action_label = if maximized {
                "还原窗口"
            } else {
                "最大化窗口"
            };
            let max_group: SharedString = format!("{:?}-grp", max_id).into();
            let mut btn = a11y::Semantics::new(
                gpui::Role::Button,
                if self.live {
                    action_label
                } else {
                    "窗口最大化或还原（不可用）"
                },
            )
            .id(format!("{max_id:?}"))
            .disabled(!self.live)
            .focusable(self.live)
            .apply(control_button(max_id, self.button_width, |s| {
                s.bg(theme::control_item_bg_hover())
            }));
            btn = match &icon_factory {
                Some(f) => {
                    let kind = if maximized {
                        WindowIconKind::Restore
                    } else {
                        WindowIconKind::Maximize
                    };
                    btn.group(max_group.clone()).child(f(kind, max_group))
                }
                None => btn.child(glyph(label)),
            };
            if self.live {
                if let Some(on_maximize) = self.on_maximize.clone() {
                    btn = a11y::click_void(btn, move |window, cx| on_maximize(window, cx));
                } else {
                    #[cfg(target_os = "windows")]
                    {
                        btn = btn
                            .window_control_area(WindowControlArea::Max)
                            .on_a11y_action(gpui::AccessibleAction::Click, |_, window, _| {
                                window.zoom_window();
                            })
                            .on_key_down(|event, window, cx| {
                                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                    cx.stop_propagation();
                                    window.zoom_window();
                                }
                            });
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        btn = a11y::click_void(btn, |window, _| window.zoom_window());
                    }
                }
            }
            controls = controls.child(btn);
        }

        if self.show_close {
            // 悬停：按钮底色变红 + 白色字形。
            // gpui 的文字颜色在布局期定型，hover 上的 text_color 不会生效；
            // 故叠一层白色字形用 group_hover 切 opacity（opacity 在绘制期生效）。
            let close_group: SharedString = format!("{:?}-closegrp", self.id).into();
            let close_bg = self.close_hover_bg.unwrap_or_else(theme::danger);
            let mut btn = a11y::Semantics::new(
                gpui::Role::Button,
                if self.live {
                    "关闭窗口"
                } else {
                    "关闭窗口（不可用）"
                },
            )
            .id(format!("{close_id:?}"))
            .disabled(!self.live)
            .focusable(self.live)
            .apply(
                control_button(close_id, self.button_width, move |s| s.bg(close_bg))
                    .group(close_group.clone()),
            );
            btn = match &icon_factory {
                Some(f) => btn.child(f(WindowIconKind::Close, close_group)),
                None => btn.child(close_glyph(&self.id, close_group)),
            };
            if self.live {
                if let Some(on_close) = self.on_close.clone() {
                    btn = a11y::click_void(btn, move |window, cx| on_close(window, cx));
                } else {
                    #[cfg(target_os = "windows")]
                    {
                        btn = btn
                            .window_control_area(WindowControlArea::Close)
                            .on_a11y_action(gpui::AccessibleAction::Click, |_, window, _| {
                                window.remove_window();
                            })
                            .on_key_down(|event, window, cx| {
                                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                    cx.stop_propagation();
                                    window.remove_window();
                                }
                            });
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        btn = a11y::click_void(btn, |window, _| window.remove_window());
                    }
                }
            }
            controls = controls.child(btn);
        }

        let mut root = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(self.height)
            .flex_shrink_0()
            .bg(theme::bg_container())
            .border_b_1()
            .border_color(theme::border())
            .text_color(theme::text_secondary())
            .child(drag);
        for el in self.extra_trailing {
            root = root.child(
                div()
                    .flex_shrink_0()
                    .h_full()
                    .flex()
                    .items_center()
                    .child(el),
            );
        }
        root.child(controls)
    }
}

/// 窗口按钮外观：等高等宽、居中，单个悬停样式由调用方提供
/// （gpui 的 `hover` 只能设置一次，重复设置会在 debug 构建触发断言）。
fn control_button(
    id: ElementId,
    width: Pixels,
    hover: impl FnOnce(StyleRefinement) -> StyleRefinement,
) -> Stateful<Div> {
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w(width)
        .h_full()
        .cursor(CursorStyle::PointingHand)
        .hover(hover)
}

fn glyph(s: &'static str) -> Div {
    div().text_xs().child(s)
}

/// 关闭按钮的 ✕ 字形：灰字 + 叠一层白字，悬停（group hover）时白字浮现。
fn close_glyph(key: &ElementId, group: SharedString) -> Stateful<Div> {
    let glyph_size = theme::with(crate::theme::metric::title_bar_metric).control_glyph_size;
    let layer = |white: bool| -> Stateful<Div> {
        let mut layer = div()
            .id(ElementId::Name(
                format!("{key:?}-x{}", if white { "white" } else { "gray" }).into(),
            ))
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .text_xs()
            .child("✕");
        if white {
            layer = layer
                .text_color(theme::text_light_solid())
                .opacity(0.)
                .group_hover(group.clone(), |s| s.opacity(1.));
        }
        layer
    };
    div()
        .id(ElementId::Name(format!("{key:?}-xglyph").into()))
        .relative()
        .size(px(glyph_size))
        .child(layer(false))
        .child(layer(true))
}
