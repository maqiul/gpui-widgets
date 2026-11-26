//! 文件选择按钮：调用系统原生文件对话框，异步回传所选路径。

use std::{path::PathBuf, rc::Rc};

use gpui::{App, IntoElement, PathPromptOptions, RenderOnce, SharedString, Window};

use crate::Button;

/// 文件选择按钮。
///
/// 注意：回调在异步任务中执行，签名为 `Fn(&Vec<PathBuf>, &mut App)`
/// （无 `&mut Window`，是本库回调约定的例外）。
///
/// ```ignore
/// FilePicker::new("选择文件")
///     .multiple(true)
///     .on_pick(|paths, cx| { /* 处理所选路径 */ })
/// ```
#[derive(IntoElement)]
pub struct FilePicker {
    id: SharedString,
    label: SharedString,
    multiple: bool,
    directories: bool,
    prompt: Option<SharedString>,
    on_pick: Option<Rc<dyn Fn(&Vec<PathBuf>, &mut App)>>,
}

impl FilePicker {
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        Self {
            id: label.clone(),
            label,
            multiple: false,
            directories: false,
            prompt: None,
            on_pick: None,
        }
    }

    /// 覆盖稳定按钮 ID（同页多个同名选择按钮时使用）。
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    /// 允许多选
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// 选择目录而非文件
    pub fn directories(mut self, directories: bool) -> Self {
        self.directories = directories;
        self
    }

    /// 对话框提示语
    pub fn prompt(mut self, prompt: impl Into<SharedString>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// 选择完成回调（取消对话框不触发）
    pub fn on_pick(mut self, listener: impl Fn(&Vec<PathBuf>, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(listener));
        self
    }
}

impl RenderOnce for FilePicker {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let options = PathPromptOptions {
            files: !self.directories,
            directories: self.directories,
            multiple: self.multiple,
            prompt: self.prompt.clone(),
        };
        let on_pick = self.on_pick.clone();

        Button::new(self.label.clone())
            .id(self.id)
            .secondary()
            .on_click(move |_, _, cx| {
                let Some(on_pick) = on_pick.clone() else {
                    return;
                };
                let receiver = cx.prompt_for_paths(options.clone());
                let app = cx.to_async();
                cx.foreground_executor()
                    .spawn(async move {
                        if let Ok(Ok(Some(paths))) = receiver.await {
                            let _ = app.update(|cx| on_pick(&paths, cx));
                        }
                    })
                    .detach();
            })
    }
}
