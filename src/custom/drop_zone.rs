//! 外部文件拖放区：接收从操作系统拖入的文件 / 文件夹，
//!
//! - 拖入悬停时整块高亮（拖拽落点高亮）
//! - 支持按扩展名过滤，未命中的文件走 `on_reject`
//! - 禁用态不响应拖放
//!
//! gpui 会把平台的 `FileDropEvent` 翻译成携带 [`ExternalPaths`] 的内部拖拽，
//! 因此这里用普通的 `drag_over` / `can_drop` / `on_drop` 即可完成落点高亮与接收。
//!
//! ```ignore
//! DropZone::new("drop")
//!     .title("拖拽文件到此处")
//!     .hint("支持批量上传")
//!     .accept(["rs", "toml"])
//!     .on_drop(|paths, _window, cx| { /* 处理文件 */ })
//! ```

use std::{path::Path, path::PathBuf, rc::Rc};

use gpui::{
    App, ElementId, ExternalPaths, FontWeight, IntoElement, RenderOnce, Role, SharedString, Window,
    div, prelude::*, px,
};

use crate::{a11y, theme};

/// 文件拖放区
#[derive(IntoElement)]
pub struct DropZone {
    id: ElementId,
    icon: SharedString,
    title: SharedString,
    hint: SharedString,
    accept: Vec<SharedString>,
    height: f32,
    disabled: bool,
    on_drop: Option<Rc<dyn Fn(&[PathBuf], &mut Window, &mut App)>>,
    on_reject: Option<Rc<dyn Fn(&[PathBuf], &mut Window, &mut App)>>,
}

impl DropZone {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            icon: "📥".into(),
            title: "拖拽文件到此处上传".into(),
            hint: "松开鼠标即可添加".into(),
            accept: Vec::new(),
            height: 132.,
            disabled: false,
            on_drop: None,
            on_reject: None,
        }
    }

    /// 顶部图标（默认 📥）
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = icon.into();
        self
    }

    /// 主标题
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    /// 副标题 / 操作提示
    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = hint.into();
        self
    }

    /// 允许的扩展名（不带点；空表示不限制）
    pub fn accept<I, S>(mut self, accept: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.accept = accept.into_iter().map(Into::into).collect();
        self
    }

    /// 区域高度（默认 132）
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// 禁用拖放（灰色的静态区域）
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 接收通过过滤的文件
    pub fn on_drop(
        mut self,
        listener: impl Fn(&[PathBuf], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_drop = Some(Rc::new(listener));
        self
    }

    /// 被扩展名规则拒绝的文件
    pub fn on_reject(
        mut self,
        listener: impl Fn(&[PathBuf], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_reject = Some(Rc::new(listener));
        self
    }
}

/// 判断路径扩展名是否命中白名单（空名单视为全部接受）
pub fn is_accepted(path: &Path, accept: &[SharedString]) -> bool {
    if accept.is_empty() {
        return true;
    }
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    accept
        .iter()
        .any(|item| item.trim_start_matches('.').eq_ignore_ascii_case(ext))
}

impl RenderOnce for DropZone {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let m = theme::with(crate::theme::metric::drop_zone_metric);
        let disabled = self.disabled;
        let mut body = div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_1()
            .px_4()
            .py_3()
            .child(
                div()
                    .text_size(px(m.icon_size))
                    .text_color(if disabled {
                        theme::text_disabled()
                    } else {
                        theme::primary()
                    })
                    .child(self.icon.clone()),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if disabled {
                        theme::text_disabled()
                    } else {
                        theme::text()
                    })
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme::text_secondary())
                    .child(self.hint.clone()),
            );

        if !self.accept.is_empty() {
            let list = self
                .accept
                .iter()
                .map(|ext| format!(".{ext}"))
                .collect::<Vec<_>>()
                .join(" ");
            body = body.child(
                div()
                    .mt_1()
                    .text_xs()
                    .text_color(theme::text_disabled())
                    .child(format!("支持格式：{list}")),
            );
        }

        let formats = if self.accept.is_empty() {
            "支持所有文件格式".to_string()
        } else {
            format!(
                "支持格式：{}",
                self.accept
                    .iter()
                    .map(|ext| format!(".{ext}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        let base = div()
            .id(self.id.clone())
            .flex()
            .items_center()
            .justify_center()
            .w_full()
            .h(px(self.height))
            .rounded_lg()
            .border_1()
            .border_dashed()
            .border_color(theme::border())
            .bg(if disabled {
                theme::bg_layout()
            } else {
                theme::bg_container()
            })
            .child(body);
        let base = a11y::Semantics::new(Role::Group, self.title.clone())
            .id(format!("drop-zone-{:?}", self.id))
            .description(format!("{}；{formats}", self.hint))
            .disabled(disabled)
            .apply(base);

        if disabled {
            return base;
        }

        let accept = self.accept.clone();
        let accept_for_can = accept.clone();
        let on_drop = self.on_drop.clone();
        let on_reject = self.on_reject.clone();

        base.hover(|style| style.border_color(theme::primary_hover()))
            .drag_over::<ExternalPaths>(|style, _paths, _window, _cx| {
                // 拖入悬停：整块高亮（落点反馈）
                style.bg(theme::primary_bg()).border_color(theme::primary())
            })
            .can_drop(move |value, _window, _cx| {
                value
                    .downcast_ref::<ExternalPaths>()
                    .map(|paths| {
                        accept_for_can.is_empty()
                            || paths
                                .paths()
                                .iter()
                                .any(|path| is_accepted(path, &accept_for_can))
                    })
                    .unwrap_or(false)
            })
            .on_drop::<ExternalPaths>(move |paths, window, cx| {
                let (accepted, rejected): (Vec<PathBuf>, Vec<PathBuf>) = paths
                    .paths()
                    .iter()
                    .cloned()
                    .partition(|path| is_accepted(path, &accept));
                if !accepted.is_empty()
                    && let Some(listener) = on_drop.as_ref()
                {
                    listener(&accepted, window, cx);
                }
                if !rejected.is_empty()
                    && let Some(listener) = on_reject.as_ref()
                {
                    listener(&rejected, window, cx);
                }
            })
    }
}
