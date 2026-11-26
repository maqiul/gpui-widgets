//! 上传文件列表：文件名 + 状态 + 进度 + 移除（对应 antd Upload 的列表部分）。
//!
//! 文件选择可用 FilePicker，本控件负责展示上传队列。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, ElementId, IntoElement, Length, ParentElement, Role, SharedString, Window,
    div, prelude::*, px, relative,
};

use crate::{a11y, theme};

/// 上传状态
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum UploadStatus {
    #[default]
    Uploading,
    Done,
    Error,
}

/// 单个上传文件
#[derive(Clone)]
pub struct UploadFile {
    name: SharedString,
    status: UploadStatus,
    progress: f32,
}

impl UploadFile {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            status: UploadStatus::default(),
            progress: 0.,
        }
    }

    pub fn status(mut self, status: UploadStatus) -> Self {
        self.status = status;
        self
    }

    /// 上传进度 0.0 ~ 1.0（仅 Uploading 状态显示）
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0., 1.);
        self
    }
}

/// 上传文件列表。
///
/// ```ignore
/// UploadList::new("uploads", self.files.clone())
///     .on_remove(cx.listener(|this, ix, _, cx| { this.files.remove(*ix); cx.notify(); }))
/// ```
#[derive(IntoElement)]
pub struct UploadList {
    id: ElementId,
    files: Vec<UploadFile>,
    on_remove: Option<Rc<dyn Fn(&usize, &mut Window, &mut App)>>,
}

impl UploadList {
    pub fn new(id: impl Into<ElementId>, files: Vec<UploadFile>) -> Self {
        Self {
            id: id.into(),
            files,
            on_remove: None,
        }
    }

    /// 点击某行 ✕ 时回调其下标
    pub fn on_remove(mut self, listener: impl Fn(&usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for UploadList {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let m = theme::with(crate::theme::metric::upload_list_metric);
        let id = format!("{:?}", self.id);
        let semantics =
            a11y::Semantics::new(Role::List, "上传文件列表").id(format!("{:?}-files", self.id));
        let mut list = semantics.apply(div().id(self.id).flex().flex_col().gap_1().w_full());

        for (ix, file) in self.files.iter().enumerate() {
            let style = theme::with(|t| crate::theme::metric::upload_status_style(t, file.status));
            let (icon, color, status_text) = (style.icon, style.color, style.label);
            let name_color = style.name;

            let mut body = div().flex().flex_col().gap_1().flex_1().child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .text_color(name_color)
                            .child(file.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .child(status_text),
                    ),
            );

            if file.status == UploadStatus::Uploading {
                let mut bar = div().h_full().rounded_full().bg(theme::primary());
                bar.style().size.width = Some(Length::Definite(relative(file.progress)));
                body = body.child(
                    a11y::Semantics::new(
                        Role::ProgressIndicator,
                        SharedString::from(format!("{} 上传进度", file.name)),
                    )
                    .id(format!("{id}-progress-{ix}"))
                    .numeric(file.progress as f64)
                    .numeric_range(0., 1.)
                    .apply(
                        div()
                            .id(ElementId::Name(format!("{id}-progress-{ix}").into()))
                            .w_full()
                            .h(px(m.bar_height))
                            .rounded_full()
                            .bg(theme::fill_secondary())
                            .child(bar),
                    ),
                );
            }

            let mut row = div()
                .id(ElementId::Name(format!("{id}-file-{ix}").into()))
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_2()
                .py(px(m.row_padding_block))
                .rounded_md()
                .hover(|style| style.bg(theme::control_item_bg_hover()))
                .child(div().text_sm().text_color(color).child(icon))
                .child(body);
            let row_semantics = a11y::Semantics::new(
                Role::ListItem,
                SharedString::from(format!("{}，{status_text}", file.name)),
            )
            .id(format!("{id}-file-{ix}"))
            .position(ix + 1, self.files.len());
            row = row_semantics.apply(row);

            if let Some(on_remove) = self.on_remove.clone() {
                let remove_semantics = a11y::Semantics::new(
                    Role::Button,
                    SharedString::from(format!("移除 {}", file.name)),
                )
                .id(format!("{id}-rm-{ix}-a11y"))
                .focusable(true);
                row = row.child(a11y::click_void(
                    remove_semantics.apply(
                        div()
                            .id(ElementId::Name(format!("{id}-rm-{ix}").into()))
                            .cursor(CursorStyle::PointingHand)
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .hover(|style| style.text_color(theme::danger()))
                            .child("✕"),
                    ),
                    move |window, cx| on_remove(&ix, window, cx),
                ));
            }
            list = list.child(row);
        }

        list
    }
}
