//! 上传触发区：虚线框点击区域，点击后由调用方打开文件选择 / 执行上传逻辑。
//!
//! 仅提供触发 UI（对应 antd Upload.Dragger 的点击形态），不内置文件对话框；
//! GPUI 的外部文件拖放事件未在本库验证，故不承诺拖放能力。

use std::rc::Rc;

use gpui::{
    App, CursorStyle, Div, ElementId, IntoElement, Role, SharedString, Stateful, Window, div,
    prelude::*, px,
};

use crate::a11y::{self, Semantics};
use crate::theme::{
    self,
    metric::{UploadMetric, upload_style},
};

/// 上传触发区。
///
/// ```ignore
/// Upload::new()
///     .text("点击上传")
///     .hint("支持单个或批量文件")
///     .on_click(|window, cx| {
///         // 由调用方打开文件对话框（可配合 rfd 等库）
///     })
/// ```
pub struct Upload {
    id: ElementId,
    text: SharedString,
    hint: Option<SharedString>,
    disabled: bool,
    on_click: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl Upload {
    pub fn new() -> Self {
        Self {
            id: "upload-area".into(),
            text: "点击上传".into(),
            hint: None,
            disabled: false,
            on_click: None,
        }
    }

    /// 元素 id（默认 "upload-area"；同窗口多个 Upload 时须各自唯一，否则 hover 高亮会串扰）
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 主文案（默认"点击上传"）
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = text.into();
        self
    }

    /// 次级提示文案（灰字，显示在主文案下方）
    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 点击回调（禁用时不触发）
    pub fn on_click(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(listener));
        self
    }
}

impl Default for Upload {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoElement for Upload {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let style = theme::with(upload_style);
        let m = theme::with(UploadMetric::resolve);
        let (icon_color, text_color, hint_color) = if self.disabled {
            (style.disabled, style.disabled, style.disabled)
        } else {
            (style.icon, style.text, style.hint)
        };

        let hint_gap = if self.hint.is_some() {
            m.text_margin_bottom.max(m.hint_margin_top)
        } else {
            m.text_margin_bottom
        };

        let semantics = Semantics::new(Role::Button, self.text.clone())
            .id(format!("{:?}-a11y", self.id))
            .disabled(self.disabled)
            .focusable(!self.disabled && self.on_click.is_some());
        let mut area = semantics.apply(
            div()
                .id(self.id)
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .p(px(m.padding))
                .rounded(px(m.radius))
                .border_1()
                .border_dashed()
                .border_color(style.border)
                .bg(style.bg)
                .child(
                    div()
                        .mt(px(m.icon_margin_top))
                        .mb(px(m.icon_margin_bottom))
                        .text_size(px(m.icon_size))
                        .line_height(px(m.icon_size))
                        .text_color(icon_color)
                        .child("↑"),
                )
                .child(
                    div()
                        .mb(px(hint_gap))
                        .text_size(px(m.text_size))
                        .line_height(px(m.text_line_height))
                        .text_color(text_color)
                        .child(self.text),
                ),
        );

        if let Some(hint) = self.hint {
            area = area.child(
                div()
                    .mb(px(m.hint_margin_bottom))
                    .text_size(px(m.hint_size))
                    .line_height(px(m.hint_line_height))
                    .text_color(hint_color)
                    .child(hint),
            );
        }

        if !self.disabled {
            area = area
                .cursor(CursorStyle::PointingHand)
                .hover(move |s| s.border_color(style.border_hover));
            if let Some(on_click) = self.on_click {
                area = a11y::click_void(area, move |window, cx| on_click(window, cx));
            }
        }

        area
    }
}
