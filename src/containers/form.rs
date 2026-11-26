//! 表单容器与校验规则（对应 antd Form）。
//!
//! 校验为受控模式：用 `validate(value, &[Rule])` 计算错误消息，
//! 存入视图状态后交给 `FormItem::error(...)` 显示。

use std::rc::Rc;

use gpui::{
    AnyElement, Div, ElementId, IntoElement, ParentElement, Pixels, Role, SharedString, Stateful,
    div, prelude::*, px,
};

use crate::a11y;

/// 校验规则：判定函数 + 失败时的错误消息
#[derive(Clone)]
pub struct Rule {
    message: SharedString,
    check: Rc<dyn Fn(&str) -> bool>,
}

impl Rule {
    /// 自定义规则：`check` 返回 true 表示通过
    pub fn custom(
        message: impl Into<SharedString>,
        check: impl Fn(&str) -> bool + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            check: Rc::new(check),
        }
    }

    /// 必填（去除首尾空白后非空）
    pub fn required(message: impl Into<SharedString>) -> Self {
        Self::custom(message, |v| !v.trim().is_empty())
    }

    /// 最少字符数（按 Unicode 字符计）
    pub fn min_len(len: usize, message: impl Into<SharedString>) -> Self {
        Self::custom(message, move |v| v.chars().count() >= len)
    }

    /// 最多字符数（按 Unicode 字符计）
    pub fn max_len(len: usize, message: impl Into<SharedString>) -> Self {
        Self::custom(message, move |v| v.chars().count() <= len)
    }

    /// 是否通过
    pub fn passes(&self, value: &str) -> bool {
        (self.check)(value)
    }

    /// 失败时的错误消息
    pub fn message(&self) -> SharedString {
        self.message.clone()
    }
}

/// 依次执行规则，返回第一条未通过规则的错误消息；全部通过返回 None。
///
/// ```ignore
/// let rules = [Rule::required("请输入用户名"), Rule::min_len(2, "至少 2 个字符")];
/// self.name_error = validate(&name, &rules);
/// // render 中：FormItem::new("用户名").error_opt(self.name_error.clone())…
/// ```
pub fn validate(value: &str, rules: &[Rule]) -> Option<SharedString> {
    rules.iter().find(|r| !r.passes(value)).map(|r| r.message())
}

/// 表单布局容器：默认纵向排列 FormItem，`inline()` 改为横向换行。
///
/// ```ignore
/// Form::new()
///     .child(FormItem::new("用户名").required().child(self.name.clone()))
///     .child(FormItem::new("邮箱").child(self.email.clone()))
/// ```
pub struct Form {
    id: Option<SharedString>,
    origin: SharedString,
    children: Vec<AnyElement>,
    inline: bool,
    gap: Option<Pixels>,
}

impl Form {
    #[track_caller]
    pub fn new() -> Self {
        let location = core::panic::Location::caller();
        Self {
            id: None,
            origin: format!(
                "form-{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
            .into(),
            children: Vec::new(),
            inline: false,
            gap: None,
        }
    }

    /// 实例 ID；同页多个表单应使用不同值。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 横向排列（自动换行），适合筛选条 / 行内表单
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    /// 表单项间距，默认 marginLG（antd itemMarginBottom）
    pub fn gap(mut self, gap: Pixels) -> Self {
        self.gap = Some(gap);
        self
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Form {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl IntoElement for Form {
    type Element = Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let gap = self.gap.unwrap_or_else(|| {
            px(crate::theme::with(|t| {
                crate::theme::metric::form_metric(t, crate::text_input::InputSize::default())
                    .item_gap
            }))
        });
        let form_id = self.id.as_ref().unwrap_or(&self.origin).clone();
        let mut form = a11y::Semantics::new(Role::Form, "表单")
            .id(form_id.clone())
            .apply(div().id(ElementId::Name(form_id.into())))
            .flex()
            .w_full()
            .gap(gap);
        form = if self.inline {
            form.flex_row().flex_wrap().items_start()
        } else {
            form.flex_col()
        };
        form.children(self.children)
    }
}
