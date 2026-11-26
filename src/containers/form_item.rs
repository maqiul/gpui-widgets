//! 表单项（对应 antd Form.Item）：标签 + 必填星号 + 校验提示 / extra 说明。
//! 令牌来自 antdv-next `form/style/index.js`：标签 colorTextHeading / controlHeight 高，
//! 星号 colorError，提示 fontSize / colorTextDescription（错误 colorError）。

use gpui::{AnyElement, ElementId, IntoElement, Pixels, Role, SharedString, div, prelude::*, px};

use crate::a11y;
use crate::text_input::InputSize;
use crate::theme::{
    self,
    metric::{form_label_color, form_metric, form_msg_color, form_required_color},
};

/// 表单项包装器。
///
/// ```ignore
/// FormItem::new("用户名")
///     .required()
///     .error("用户名不能为空")
///     .child(self.username.clone())
/// ```
#[derive(IntoElement)]
pub struct FormItem {
    id: Option<SharedString>,
    origin: SharedString,
    label: SharedString,
    required: bool,
    error: Option<SharedString>,
    help: Option<SharedString>,
    label_width: Option<Pixels>,
    label_left: bool,
    colon: bool,
    size: InputSize,
    child: Option<AnyElement>,
}

impl FormItem {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "form-item-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            label: label.into(),
            required: false,
            error: None,
            help: None,
            label_width: None,
            label_left: false,
            colon: true,
            size: InputSize::default(),
            child: None,
        }
    }

    /// 实例 ID；同页重复标签或复用同一构造位置时应显式指定。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 显示红色必填星号
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// 校验错误提示（colorError，优先于 help 显示）
    pub fn error(mut self, error: impl Into<SharedString>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// 可选错误提示（配合 form::validate 的返回值，None 时不显示）
    pub fn error_opt(mut self, error: Option<SharedString>) -> Self {
        self.error = error;
        self
    }

    /// 标签固定宽度：设置后改为横向布局（标签在左、控件在右），
    /// 同一 Form 内各项传相同宽度即可对齐；默认右对齐（antd labelAlign）
    pub fn label_width(mut self, width: Pixels) -> Self {
        self.label_width = Some(width);
        self
    }

    /// 横向布局时标签左对齐（antd labelAlign="left"）
    pub fn label_left(mut self) -> Self {
        self.label_left = true;
        self
    }

    /// 横向布局标签后的冒号（默认显示，antd colon）
    pub fn colon(mut self, colon: bool) -> Self {
        self.colon = colon;
        self
    }

    /// 尺寸档：标签 / 控件最小高随 controlHeight 三档
    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    /// 辅助说明文字（antd extra：colorTextDescription，最小高 controlHeightSM）
    pub fn help(mut self, help: impl Into<SharedString>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.child = Some(child.into_any_element());
        self
    }
}

impl RenderOnce for FormItem {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let m = theme::with(|t| form_metric(t, self.size));
        let (label_color, required_color) =
            theme::with(|t| (form_label_color(t), form_required_color(t)));
        let line = theme::with(|t| t.font_size * t.line_height);
        let owner = self.id.as_deref().unwrap_or(&self.origin);
        let item_id = format!("{owner}-{}", self.label);
        let description = self.error.clone().or_else(|| self.help.clone());
        let mut semantics = a11y::Semantics::new(Role::Group, self.label.clone())
            .id(item_id.clone())
            .invalid(self.error.is_some());
        if let Some(description) = description {
            semantics = semantics.description(description);
        }

        let mut label_row = div()
            .flex()
            .flex_row()
            .items_center()
            .h(px(m.label_height))
            .text_size(px(m.label_font))
            .line_height(px(line))
            .text_color(label_color);
        if self.required {
            label_row = label_row.child(
                div()
                    .mr(px(m.required_gap))
                    .line_height(px(m.label_font))
                    .text_color(required_color)
                    .child("*"),
            );
        }
        label_row = label_row.child(div().child(self.label.clone()));

        // 错误 / 帮助提示（错误优先）；extra 最小高 controlHeightSM
        let msg = if let Some(error) = &self.error {
            Some(
                div()
                    .text_size(px(m.msg_font))
                    .line_height(px(line))
                    .text_color(theme::with(|t| form_msg_color(t, true)))
                    .child(error.clone()),
            )
        } else {
            self.help.as_ref().map(|help| {
                div()
                    .min_h(px(m.extra_min_height))
                    .text_size(px(m.msg_font))
                    .line_height(px(line))
                    .text_color(theme::with(|t| form_msg_color(t, false)))
                    .child(help.clone())
            })
        };

        if let Some(width) = self.label_width {
            // 横向布局：固定宽标签（默认右对齐 + 冒号）+ 右侧控件 / 提示
            let mut label_cell = div()
                .flex()
                .flex_row()
                .items_center()
                .w(width)
                .flex_shrink_0()
                .child(label_row);
            if self.label_left {
                label_cell = label_cell.justify_start();
            } else {
                label_cell = label_cell.justify_end();
            }
            if self.colon {
                label_cell = label_cell.child(
                    div()
                        .ml(px(m.colon_start))
                        .mr(px(m.colon_end))
                        .text_size(px(m.label_font))
                        .line_height(px(line))
                        .text_color(label_color)
                        .child(":"),
                );
            }
            let mut right = div().flex_1().flex().flex_col();
            if let Some(child) = self.child {
                right = right.child(
                    div()
                        .flex()
                        .items_center()
                        .min_h(px(m.label_height))
                        .child(child),
                );
            }
            if let Some(msg) = msg {
                right = right.child(msg);
            }
            semantics.apply(
                div()
                    .id(ElementId::Name(item_id.into()))
                    .flex()
                    .flex_row()
                    .items_start()
                    .w_full()
                    .child(label_cell)
                    .child(right),
            )
        } else {
            // 纵向布局：标签在上（下内边距 paddingXS，冒号隐藏）；gpui 的 h() 是
            // 盒高（padding 含在内），要撑出 antd 的 行高 + padding 才能把控件下移 8px
            let mut item = div().flex().flex_col().w_full().child(
                label_row
                    .h(px(m.vertical_label_height + m.label_pad_bottom))
                    .pb(px(m.label_pad_bottom)),
            );
            if let Some(child) = self.child {
                item = item.child(
                    div()
                        .flex()
                        .items_center()
                        .min_h(px(m.label_height))
                        .child(child),
                );
            }
            if let Some(msg) = msg {
                item = item.child(msg);
            }
            semantics.apply(item.id(ElementId::Name(item_id.into())))
        }
    }
}
