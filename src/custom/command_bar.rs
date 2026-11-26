//! 命令输入条：单行命令输入 + `↑`/`↓` 历史回溯 + 片段（snippet）补全 / 点击插入 + `Enter` 提交。
//!
//! 内部包装 [`TextInput`]（bare 模式）复用其光标 / 选区 / IME 能力；
//! `↑`/`↓`/`Enter`/`Tab`/`Esc` 未被 TextInput 绑定，故冒泡到本控件根节点统一处理。
//!
//! 交互：
//! - `Enter` 提交当前命令（去重入历史）并触发 [`CommandBar::on_submit`]，随后清空输入；
//! - `↑` 向更旧历史回溯（首次按下保存当前草稿），`↓` 向更新回溯 / 回到草稿；
//! - `Tab` 以当前输入为前缀补全第一个匹配片段；点击下方片段芯片直接插入；
//! - `Esc` 先收浮层，其次退出历史回溯态，最后清空输入。
//!
//! 两种外观：默认 `framed`（整条自带输入框外框 + 下方片段芯片行）；
//! [`CommandBar::framed`] 传 `false` 时由宿主画底色与上边线，输入框自己描边，
//! 配 [`CommandBar::actions`] 换成右侧四个动作按钮 + 上浮面板（终端下沿命令条形态）。
//!
//! ```ignore
//! let bar = cx.new(|cx| CommandBar::new(cx)
//!     .prompt("❯")
//!     .placeholder("输入命令，Enter 提交")
//!     .snippets(["git status", "cargo build", "ls -la"])
//!     .on_submit(|cmd, _w, _cx| println!("run: {cmd}")));
//! // render 中：div().child(bar.clone())
//! ```

use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use gpui::{
    AnyElement, App, AppContext, Context, CursorStyle, ElementId, Entity, FocusHandle, Focusable,
    FontWeight, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, Orientation,
    ParentElement, Render, Role, SharedString, Styled, Toggled, Window, div, px,
};

use crate::controls::scrollbar::VerticalScroll;
use crate::text_input::InputSize;
use crate::theme::metric::{
    CommandBarAction, command_bar_action_style, command_bar_metric, command_bar_style,
    mono_font_family,
};
use crate::{TextArea, TextInput, a11y, theme};

static CMD_SEQ: AtomicUsize = AtomicUsize::new(0);
static SNIP_SEQ: AtomicUsize = AtomicUsize::new(1);

/// 右侧动作按钮浮层的展开态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    None,
    Snippets,
    History,
}

/// 片段面板里的一条命令片段。
///
/// `name` 是左列显示名，`command` 是右列等宽命令（也是点击后填进输入条的内容），
/// `group` 为空时归入「未分组」。`send_enter` 决定点击是直接执行还是只填输入条
/// （原版 `SnippetPanel` 把这条交回父组件判断）；`sort` 是排序号，编辑时沿用原值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSnippet {
    pub id: usize,
    pub name: SharedString,
    pub command: SharedString,
    pub group: SharedString,
    pub send_enter: bool,
    pub sort: usize,
}

impl CommandSnippet {
    pub fn new(
        name: impl Into<SharedString>,
        command: impl Into<SharedString>,
        group: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: SNIP_SEQ.fetch_add(1, Ordering::Relaxed),
            name: name.into(),
            command: command.into(),
            group: group.into(),
            send_enter: true,
            sort: 0,
        }
    }

    /// 点击后是否直接执行（原版表单里的 sendEnter 勾选）。
    pub fn send_enter(mut self, send_enter: bool) -> Self {
        self.send_enter = send_enter;
        self
    }

    /// 排序号：面板按它稳定排序后再分组（原版 `sortedSnippets`）。
    pub fn sort(mut self, sort: usize) -> Self {
        self.sort = sort;
        self
    }
}

/// 按 `sort` 升序、同号按名称字典序（原版 `sortedSnippets`：
/// `sortIndex` 差值优先，其次 `name.localeCompare`；gpui 侧用 Unicode 字典序近似）。
pub fn sorted_snippets(items: &[CommandSnippet]) -> Vec<CommandSnippet> {
    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| {
        a.sort
            .cmp(&b.sort)
            .then_with(|| a.name.as_ref().cmp(b.name.as_ref()))
    });
    sorted
}

/// 面板分组：先按排序号排，再按首次出现顺序聚合，**未分组永远垫底**
/// （原版 `groups` 计算属性的 `.sort((a, b) => (a.group ? 0 : 1) - …)`）。
///
/// 与 [`group_snippets`] 的区别仅在这两步；后者保持纯首现顺序供补全等场景用。
pub fn panel_groups(
    items: &[CommandSnippet],
    ungrouped: &str,
) -> Vec<(SharedString, Vec<CommandSnippet>)> {
    let mut groups = group_snippets(&sorted_snippets(items), "");
    if let Some(pos) = groups.iter().position(|(g, _)| g.is_empty()) {
        let (_, items) = groups.remove(pos);
        groups.push((SharedString::from(ungrouped.to_string()), items));
    }
    groups
}

/// 表单可保存条件：名称与命令去空白后都非空（原版 `canSave`）。
pub fn snippet_can_save(name: &str, command: &str) -> bool {
    !name.trim().is_empty() && !command.trim().is_empty()
}

/// 行悬停图标槽的两种图标（宿主用 [`CommandBar::snippet_icon_builder`] 供元素）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnippetIcon {
    Edit,
    Trash,
}

/// 片段管理事件：库内改列表，宿主负责落盘 / 提示 / 删除确认。
#[derive(Debug, Clone)]
pub enum SnippetEvent {
    /// 表单保存成功（新建或编辑），列表已更新。
    Saved(CommandSnippet),
    /// 点了删除图标：宿主弹确认，确认后调 [`CommandBar::remove_snippet`]。
    DeleteRequested(CommandSnippet),
    /// 名称/命令没填全（原版 `message.warning(term.snip.required)`）。
    Warn(SharedString),
}

/// 内联新建/编辑表单的状态（原版 `editing` + 四个 form ref）。
struct SnipForm {
    /// `None` = 新建；`Some(id)` = 编辑已有。
    id: Option<usize>,
    sort: usize,
    send_enter: bool,
    name: Entity<TextInput>,
    group: Entity<TextInput>,
    command: Entity<TextArea>,
}

/// 按首次出现顺序把片段分组（原版 `SnippetPanel` 的 `groups` 计算属性）。
///
/// 空 `group` 统一落到 `ungrouped` 标签下，且只在该组真的存在空分组条目时才出现。
pub fn group_snippets(
    items: &[CommandSnippet],
    ungrouped: &str,
) -> Vec<(SharedString, Vec<CommandSnippet>)> {
    let mut order: Vec<SharedString> = Vec::new();
    let mut groups: Vec<(SharedString, Vec<CommandSnippet>)> = Vec::new();
    for item in items {
        let label = if item.group.is_empty() {
            SharedString::from(ungrouped.to_string())
        } else {
            item.group.clone()
        };
        if let Some(ix) = order.iter().position(|g| *g == label) {
            groups[ix].1.push(item.clone());
        } else {
            order.push(label.clone());
            groups.push((label, vec![item.clone()]));
        }
    }
    groups
}

/// 命令输入条（有状态 `Entity`）。
pub struct CommandBar {
    input: Entity<TextInput>,
    id: ElementId,
    /// 实例序号，用于生成唯一子元素 id。
    uid: usize,
    prompt: SharedString,
    placeholder: SharedString,
    /// 历史命令（旧 → 新）。
    history: Vec<String>,
    /// 历史回溯当前位置；None 表示处于实时输入态。
    history_idx: Option<usize>,
    /// 首次按 ↑ 时保存的草稿，↓ 到底时恢复。
    draft: String,
    /// 可插入 / 补全的命令片段。
    snippets: Vec<SharedString>,
    /// 带名称与分组的片段（非空时片段浮层换成面板形态）。
    snippet_items: Vec<CommandSnippet>,
    /// 空态文案：`(历史, 片段)`。
    empty_text: [SharedString; 2],
    on_new_snippet: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
    max_history: usize,
    on_submit: Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>,
    /// `false` = 由宿主画底色与上边线，输入框自己描边。
    framed: bool,
    /// 右侧四个动作按钮的文案（片段 / 历史 / 清空 / 执行）；`None` = 显示快捷键提示。
    actions: Option<[SharedString; 4]>,
    panel: Panel,
    /// 片段管理模式：内联新建/编辑表单 + 行悬停图标 + sendEnter 语义 + 排序分组。
    manage: bool,
    /// 浮层点外部收起（原版 antd dropdown 的 body 点击收起）。
    dismiss_outside: bool,
    icon_builder: Option<Rc<dyn Fn(SnippetIcon, &str) -> AnyElement>>,
    on_snippet_event: Option<Rc<dyn Fn(SnippetEvent, &mut Window, &mut App)>>,
    on_history_clear: Option<Rc<dyn Fn(&mut App)>>,
    form: Option<SnipForm>,
}

impl CommandBar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            input: cx.new(|cx| TextInput::new(cx).bare(true)),
            id: ElementId::Name("command-bar".into()),
            uid: CMD_SEQ.fetch_add(1, Ordering::Relaxed),
            prompt: "❯".into(),
            placeholder: "输入命令，Enter 提交".into(),
            history: Vec::new(),
            history_idx: None,
            draft: String::new(),
            snippets: Vec::new(),
            snippet_items: Vec::new(),
            empty_text: ["暂无记录".into(), "暂无片段".into()],
            on_new_snippet: None,
            max_history: 100,
            on_submit: None,
            framed: true,
            actions: None,
            panel: Panel::None,
            manage: false,
            dismiss_outside: false,
            icon_builder: None,
            on_snippet_event: None,
            on_history_clear: None,
            form: None,
        }
    }

    /// 自定义元素 id（同页多实例时区分）。
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// 行首提示符（如 "❯" / "$" / ">"）。
    pub fn prompt(mut self, prompt: impl Into<SharedString>) -> Self {
        self.prompt = prompt.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// 命令片段列表（点击插入 / Tab 前缀补全）。
    pub fn snippets(mut self, snippets: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.snippets = snippets.into_iter().map(Into::into).collect();
        self
    }

    /// 带名称与分组的片段：片段浮层随之换成面板形态（标题 + 新建 + 分组 + 命令列）。
    ///
    /// 命令内容同时并入 [`CommandBar::snippets`]，Tab 前缀补全与芯片行为保持一致。
    pub fn snippet_items(mut self, items: impl IntoIterator<Item = CommandSnippet>) -> Self {
        self.snippet_items = items.into_iter().collect();
        self.snippets
            .extend(self.snippet_items.iter().map(|s| s.command.clone()));
        self
    }

    /// 空态文案：`(历史浮层, 片段面板)`，默认「暂无记录 / 暂无片段」。
    pub fn empty_text(
        mut self,
        history: impl Into<SharedString>,
        snippets: impl Into<SharedString>,
    ) -> Self {
        self.empty_text = [history.into(), snippets.into()];
        self
    }

    /// 片段面板头部的「新建」按钮回调；不设置则不画按钮（宿主没有片段存储时）。
    pub fn on_new_snippet(mut self, listener: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_new_snippet = Some(Rc::new(listener));
        self
    }

    /// 同上，供实体建好之后再挂（回调里需要拿到自身句柄时用）。
    pub fn set_on_new_snippet(&mut self, listener: impl Fn(&mut Window, &mut App) + 'static) {
        self.on_new_snippet = Some(Rc::new(listener));
    }

    /// 追加一条片段（宿主「新建」保存后回填），同时并入补全用的命令列表。
    pub fn push_snippet(&mut self, snippet: CommandSnippet) {
        self.snippets.push(snippet.command.clone());
        self.snippet_items.push(snippet);
    }

    /// 已登记的片段条数。
    pub fn snippet_count(&self) -> usize {
        self.snippet_items.len()
    }

    /// 已登记的片段（宿主删除确认 / 演示态取 id 用）。
    pub fn snippet_list(&self) -> &[CommandSnippet] {
        &self.snippet_items
    }

    /// 片段管理模式（原版 SnippetPanel 全量行为）：头部「新建」开内联表单、
    /// 行悬停显编辑/删除图标、点击按 `send_enter` 决定执行或填输入条、
    /// 列表按 `sort` 排序且未分组垫底。默认 `false` 保持旧的面板形态。
    pub fn manage_snippets(mut self, manage: bool) -> Self {
        self.manage = manage;
        self
    }

    /// 浮层点外部收起（历史与片段浮层都生效）。默认 `false`。
    pub fn dismiss_outside(mut self, dismiss: bool) -> Self {
        self.dismiss_outside = dismiss;
        self
    }

    /// 行悬停图标槽的元素工厂：`(图标种类, 槽 group 名)`。
    ///
    /// 槽的显隐由库用行 group 的 opacity 级联控制；图标自身悬停换色靠宿主在
    /// 元素上挂 `group_hover(槽 group)`（gpui 的 svg 不继承父文字色）。
    /// 不设置则管理模式也不画图标槽。
    pub fn snippet_icon_builder(
        mut self,
        builder: impl Fn(SnippetIcon, &str) -> AnyElement + 'static,
    ) -> Self {
        self.icon_builder = Some(Rc::new(builder));
        self
    }

    /// 同上，供实体建好之后再挂。
    pub fn set_snippet_icon_builder(
        &mut self,
        builder: impl Fn(SnippetIcon, &str) -> AnyElement + 'static,
    ) {
        self.icon_builder = Some(Rc::new(builder));
    }

    /// 片段管理事件回调（保存成功 / 请求删除确认 / 必填警告）。
    pub fn on_snippet_event(
        mut self,
        listener: impl Fn(SnippetEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_snippet_event = Some(Rc::new(listener));
        self
    }

    /// 同上，供实体建好之后再挂。
    pub fn set_on_snippet_event(
        &mut self,
        listener: impl Fn(SnippetEvent, &mut Window, &mut App) + 'static,
    ) {
        self.on_snippet_event = Some(Rc::new(listener));
    }

    /// 删除一条片段（宿主删除确认后回填），同时从补全命令列表移除一次出现。
    pub fn remove_snippet(&mut self, id: usize) {
        let Some(ix) = self.snippet_items.iter().position(|s| s.id == id) else {
            return;
        };
        let removed = self.snippet_items.remove(ix);
        if let Some(pos) = self.snippets.iter().position(|c| *c == removed.command) {
            self.snippets.remove(pos);
        }
        if self.form.as_ref().is_some_and(|f| f.id == Some(id)) {
            self.form = None;
        }
    }

    /// 打开片段面板（宿主演示态 / 验收截图用）。
    pub fn show_snippets(&mut self, cx: &mut Context<Self>) {
        self.panel = Panel::Snippets;
        cx.notify();
    }

    /// 打开「新建」表单（管理模式头部按钮 / 宿主演示态）。
    pub fn open_snippet_new(&mut self, cx: &mut Context<Self>) {
        let m = theme::with(command_bar_metric);
        let name = cx.new(|cx| {
            TextInput::new(cx)
                .size(InputSize::Small)
                .placeholder("名称（列表里显示这一行）")
        });
        let group = cx.new(|cx| {
            TextInput::new(cx)
                .size(InputSize::Small)
                .placeholder("分组，可留空")
        });
        let command = cx.new(|cx| {
            TextArea::new(cx)
                .monospace()
                .height(px(m.snip_form_command_height))
                .placeholder("命令内容")
        });
        self.form = Some(SnipForm {
            id: None,
            sort: 0,
            send_enter: true,
            name,
            group,
            command,
        });
        cx.notify();
    }

    /// 打开某条片段的编辑表单：排序号沿用原值（原版一改不挪位）。
    pub fn open_snippet_edit(&mut self, id: usize, cx: &mut Context<Self>) {
        let Some(item) = self.snippet_items.iter().find(|s| s.id == id).cloned() else {
            return;
        };
        self.open_snippet_new(cx);
        let Some(form) = self.form.as_mut() else {
            return;
        };
        form.id = Some(id);
        form.sort = item.sort;
        form.send_enter = item.send_enter;
        form.name
            .update(cx, |ti, cx| ti.set_text_silent(item.name.clone(), cx));
        form.group
            .update(cx, |ti, cx| ti.set_text_silent(item.group.clone(), cx));
        form.command
            .update(cx, |ta, cx| ta.set_text_silent(item.command.clone(), cx));
        cx.notify();
    }

    /// 表单保存：必填校验 → 改列表 → 抛 [`SnippetEvent::Saved`]。
    fn save_snippet_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(form) = self.form.as_ref() else {
            return;
        };
        let name = form.name.read(cx).text().trim().to_string();
        let group = form.group.read(cx).text().trim().to_string();
        let command = form.command.read(cx).text().to_string();
        let id = form.id;
        let sort = form.sort;
        let send_enter = form.send_enter;
        if !snippet_can_save(&name, &command) {
            if let Some(cb) = self.on_snippet_event.clone() {
                cb(
                    SnippetEvent::Warn("名称和命令内容都要填".into()),
                    window,
                    cx,
                );
            }
            return;
        }
        let saved = match id {
            Some(id) => {
                let Some(item) = self.snippet_items.iter_mut().find(|s| s.id == id) else {
                    return;
                };
                let old = item.command.clone();
                item.name = name.into();
                item.group = group.into();
                item.command = command.into();
                item.send_enter = send_enter;
                if let Some(pos) = self.snippets.iter().position(|c| *c == old) {
                    self.snippets[pos] = item.command.clone();
                }
                item.clone()
            }
            None => {
                let item = CommandSnippet {
                    id: SNIP_SEQ.fetch_add(1, Ordering::Relaxed),
                    name: name.into(),
                    command: command.into(),
                    group: group.into(),
                    send_enter,
                    sort,
                };
                self.push_snippet(item.clone());
                item
            }
        };
        self.form = None;
        if let Some(cb) = self.on_snippet_event.clone() {
            cb(SnippetEvent::Saved(saved), window, cx);
        }
        cx.notify();
    }

    /// 收浮层：表单随面板卸载一起丢（原版 SnippetPanel 卸载即重置 editing）。
    fn close_panel(&mut self) {
        self.panel = Panel::None;
        self.form = None;
    }

    /// 历史上限（默认 100）。
    pub fn max_history(mut self, max: usize) -> Self {
        self.max_history = max.max(1);
        self
    }

    /// 是否由控件自己画整条外框（默认 `true`）。
    ///
    /// 传 `false` 时条身只剩固定行高，底色与上边线都交给宿主，描边收进输入框——
    /// 嵌在终端 / 编辑器面板下沿的命令条形态。原版这一形态的输入框是 12px 等宽，
    /// 所以这里同步改输入框排版，`true` 则回到令牌字号 + 正文字体。
    pub fn framed(mut self, framed: bool, cx: &mut App) -> Self {
        self.framed = framed;
        let size = theme::with(|t| command_bar_metric(t).input_font_size);
        self.input.update(cx, |input, _| {
            input.set_typography(!framed, (!framed).then_some(size));
        });
        self
    }

    /// 用右侧四个动作按钮替掉快捷键提示：`[片段, 历史, 清空, 执行]`。
    ///
    /// 片段 / 历史切换上浮面板（历史条目倒序，末行清空历史），清空只清输入，
    /// 执行等价于 `Enter`（无输入时禁用）。
    pub fn actions(mut self, labels: [impl Into<SharedString>; 4]) -> Self {
        self.actions = Some(labels.map(Into::into));
        self
    }

    /// 提交回调（参数为去除首尾空白的命令）。经 defer 触发，回调内可安全 update 任意实体。
    pub fn on_submit(mut self, listener: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(Rc::new(listener));
        self
    }

    /// 当前输入文本。
    pub fn text(&self, cx: &App) -> String {
        self.input.read(cx).text().to_string()
    }

    /// 程序化设置输入文本（不触发提交）。
    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.input.update(cx, |ti, cx| ti.set_text_silent(text, cx));
    }

    /// 运行时换占位文本（界面语言切换时用，不触发提交）。
    pub fn set_placeholder(
        &mut self,
        placeholder: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        let ph: SharedString = placeholder.into();
        self.placeholder = ph.clone();
        self.input.update(cx, |ti, _| ti.set_placeholder(ph));
        cx.notify();
    }

    /// 运行时换右侧四个动作按钮文案（界面语言切换时用）。
    pub fn set_actions(&mut self, labels: [impl Into<SharedString>; 4], cx: &mut Context<Self>) {
        self.actions = Some(labels.map(Into::into));
        cx.notify();
    }

    /// 运行时换空态文案 `(历史, 片段)`（界面语言切换时用）。
    pub fn set_empty_text(
        &mut self,
        history: impl Into<SharedString>,
        snippets: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.empty_text = [history.into(), snippets.into()];
        cx.notify();
    }

    /// 历史命令（旧 → 新）。
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// 清空历史。
    pub fn clear_history(&mut self, cx: &mut Context<Self>) {
        self.history.clear();
        self.history_idx = None;
        self.draft.clear();
        if let Some(f) = self.on_history_clear.clone() {
            f(cx);
        }
        cx.notify();
    }

    /// 构建期种子历史（旧 → 新）：宿主从持久化加载后回填。
    pub fn history_items(mut self, items: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.history = items.into_iter().map(Into::into).collect();
        self
    }

    /// 运行时整体替换历史（宿主全局历史同步）：回溯位置重置，
    /// 但保留输入草稿——原版各会话的输入条草稿互不影响。
    pub fn set_history(&mut self, items: Vec<String>, cx: &mut Context<Self>) {
        self.history = items;
        self.history_idx = None;
        cx.notify();
    }

    /// 运行时整体替换片段（宿主全局片段同步），补全命令列表随之重建；
    /// 正在编辑的片段若已不在列表里，内联表单一并收起。
    pub fn set_snippet_items(&mut self, items: Vec<CommandSnippet>, cx: &mut Context<Self>) {
        self.snippet_items = items;
        self.snippets = self
            .snippet_items
            .iter()
            .map(|s| s.command.clone())
            .collect();
        if self.form.as_ref().is_some_and(|f| {
            f.id.is_some_and(|id| !self.snippet_items.iter().any(|s| s.id == id))
        }) {
            self.form = None;
        }
        cx.notify();
    }

    /// 清空历史后的回调：宿主持久化要跟着清盘上历史。
    pub fn on_history_clear(mut self, listener: impl Fn(&mut App) + 'static) -> Self {
        self.on_history_clear = Some(Rc::new(listener));
        self
    }

    /// 同上，供实体建好之后再挂。
    pub fn set_on_history_clear(&mut self, listener: impl Fn(&mut App) + 'static) {
        self.on_history_clear = Some(Rc::new(listener));
    }

    /// 主动提交当前输入（等价于按 Enter）。
    pub fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.text(cx).trim().to_string();
        if text.is_empty() {
            return;
        }
        self.history.retain(|h| h != &text);
        self.history.push(text.clone());
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }
        self.history_idx = None;
        self.draft.clear();
        self.panel = Panel::None;
        self.input.update(cx, |ti, cx| ti.set_text_silent("", cx));
        if let Some(f) = self.on_submit.clone() {
            window.defer(cx, move |window, cx| f(&text, window, cx));
        }
        cx.notify();
    }

    fn set_input(&mut self, text: &str, cx: &mut Context<Self>) {
        let owned = text.to_string();
        self.input
            .update(cx, move |ti, cx| ti.set_text_silent(owned, cx));
    }

    /// ↑ 向更旧历史回溯。
    fn history_prev(&mut self, cx: &mut Context<Self>) {
        if self.history.is_empty() {
            return;
        }
        let idx = match self.history_idx {
            None => {
                self.draft = self.text(cx);
                self.history.len() - 1
            }
            Some(i) => i.saturating_sub(1),
        };
        self.history_idx = Some(idx);
        let t = self.history[idx].clone();
        self.set_input(&t, cx);
        cx.notify();
    }

    /// ↓ 向更新历史回溯 / 回到草稿。
    fn history_next(&mut self, cx: &mut Context<Self>) {
        let Some(i) = self.history_idx else {
            return;
        };
        if i + 1 < self.history.len() {
            self.history_idx = Some(i + 1);
            let t = self.history[i + 1].clone();
            self.set_input(&t, cx);
        } else {
            self.history_idx = None;
            let d = self.draft.clone();
            self.set_input(&d, cx);
        }
        cx.notify();
    }

    /// Tab：以当前输入为前缀补全第一个匹配片段。
    fn complete_snippet(&mut self, cx: &mut Context<Self>) {
        if self.snippets.is_empty() {
            return;
        }
        let cur = self.text(cx);
        let target = if cur.is_empty() {
            self.snippets.first().cloned()
        } else {
            self.snippets
                .iter()
                .find(|s| s.starts_with(cur.as_str()) && s.as_ref() != cur)
                .cloned()
        };
        if let Some(t) = target {
            let t = t.to_string();
            self.set_input(&t, cx);
            cx.notify();
        }
    }

    /// 点击片段芯片：把片段写入输入框并聚焦。
    fn insert_snippet(&mut self, snippet: String, window: &mut Window, cx: &mut Context<Self>) {
        self.set_input(&snippet, cx);
        // 延迟到本次鼠标事件处理完再聚焦：避免被窗口点击非聚焦元素的默认失焦逻辑覆盖。
        let focus_handle = self.input.focus_handle(cx);
        window.defer(cx, move |window, cx| focus_handle.focus(window, cx));
        cx.notify();
    }

    /// 把输入焦点交给命令输入框（点条身任意空白处）。
    fn focus_input(
        &mut self,
        _: &gpui::MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let focus_handle = self.input.focus_handle(cx);
        window.defer(cx, move |window, cx| focus_handle.focus(window, cx));
    }

    /// 片段 / 历史按钮：同一个按钮再点一次收起。
    fn toggle_panel(&mut self, kind: Panel, cx: &mut Context<Self>) {
        self.panel = if self.panel == kind {
            Panel::None
        } else {
            kind
        };
        if self.panel != Panel::Snippets {
            self.form = None;
        }
        cx.notify();
    }

    /// 「清空」按钮：只清输入，历史留着。
    fn clear_text(&mut self, cx: &mut Context<Self>) {
        self.history_idx = None;
        self.draft.clear();
        self.set_input("", cx);
        cx.notify();
    }

    fn on_key_down(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let k = &ev.keystroke;
        let mods = k.modifiers;
        // 仅处理无主修饰键的组合，避免劫持 Ctrl+↑ 等系统快捷键。
        let plain = !mods.control && !mods.alt && !mods.platform;
        let input_focused = self.input.focus_handle(cx).is_focused(window);
        match k.key.as_str() {
            "up" if plain && input_focused => {
                self.history_prev(cx);
                cx.stop_propagation();
            }
            "down" if plain && input_focused => {
                self.history_next(cx);
                cx.stop_propagation();
            }
            "enter" if plain => {
                // 焦点在内联表单的名称/分组框时，Enter 是提交表单（原版 form submit），
                // 不是提交命令条；命令 TextArea 的 Enter 自己换行并止冒泡。
                let in_form = self.form.as_ref().is_some_and(|f| {
                    f.name.focus_handle(cx).is_focused(window)
                        || f.group.focus_handle(cx).is_focused(window)
                });
                if in_form {
                    self.save_snippet_form(window, cx);
                } else if input_focused {
                    self.submit(window, cx);
                } else {
                    // 动作按钮自己处理 Enter / Space，避免键盘激活时额外提交命令。
                    return;
                }
                cx.stop_propagation();
            }
            "tab" if plain && !mods.shift && input_focused => {
                self.complete_snippet(cx);
                cx.stop_propagation();
            }
            "escape" if plain => {
                if self.panel != Panel::None {
                    // 浮层挡的就是终端内容，先收浮层。
                    self.close_panel();
                    cx.notify();
                } else if self.history_idx.is_some() {
                    self.history_idx = None;
                    let d = self.draft.clone();
                    self.set_input(&d, cx);
                    cx.notify();
                } else if self.actions.is_some() {
                    self.clear_text(cx);
                }
            }
            _ => {}
        }
    }
}

impl Render for CommandBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let m = theme::with(command_bar_metric);
        let focused = self.input.focus_handle(cx).is_focused(window);
        let framed = self.framed;
        let st = theme::with(|t| command_bar_style(t, framed, focused));
        let shadow = theme::with(|t| t.shadow_secondary.0.clone());
        let placeholder = self.placeholder.clone();
        self.input
            .update(cx, |ti, _| ti.set_placeholder(placeholder));

        let uid = self.uid;
        let entity_id = format!("command-bar-{:?}", cx.entity_id());
        let panel = self.panel;
        let snippets = self.snippets.clone();
        let can_send = !self.text(cx).trim().is_empty();
        // 下拉里最新的排最前，和键盘 ↑ 的方向一致。
        let history: Vec<SharedString> = self
            .history
            .iter()
            .rev()
            .map(|h| SharedString::from(h.clone()))
            .collect();
        let history_hint = match self.history_idx {
            Some(i) => format!("历史 {}/{}", i + 1, self.history.len()),
            None => "⏎ 提交 · ↑↓ 历史".to_string(),
        };

        let root_id = format!("cmdbar-{uid}-root");
        let mut root = a11y::Semantics::new(Role::Group, "命令输入条")
            .id(root_id)
            .apply(
                div()
                    .id(self.id.clone())
                    .key_context("CommandBar")
                    .on_key_down(cx.listener(Self::on_key_down))
                    .relative()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .w_full()
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::focus_input)),
            );

        let mut bar = div().flex().flex_row().items_center().w_full();
        bar = match st.bar_bg {
            Some(color) => bar.bg(color),
            None => bar,
        };
        bar = if framed {
            bar.px(px(m.bar_padding_inline))
                .py(px(m.bar_padding_block))
                .gap(px(m.bar_gap))
                .rounded(px(m.bar_radius))
        } else {
            bar.h(px(m.bar_height))
                .px(px(m.bare_padding_inline))
                .gap(px(m.bare_gap))
        };
        bar = match st.bar_border {
            Some(color) => bar.border_1().border_color(color),
            None => bar,
        };

        let mut prompt = div()
            .flex_shrink_0()
            .text_color(st.prompt_fg)
            .font_weight(FontWeight::SEMIBOLD)
            .child(self.prompt.clone());
        if !framed {
            prompt = prompt.text_size(px(m.prompt_font_size));
        }
        bar = bar.child(prompt);

        let input_id = format!("cmdbar-{uid}-input");
        let input_semantics = a11y::Semantics::new(Role::TextInput, "命令输入")
            .id(input_id.clone())
            .value(self.input.read(cx).a11y_value())
            .placeholder(self.placeholder.clone())
            .disabled(false)
            .read_only(false)
            .focusable(true);
        let input_entity = self.input.clone();
        bar = bar.child(if framed {
            self.input.update(cx, |input, cx| {
                let host = div()
                    .id(ElementId::Name(input_id.into()))
                    .flex_1()
                    .min_w(px(0.))
                    .track_focus(&input.focus_handle(cx));
                input
                    .with_a11y_actions(input_semantics.apply(host), cx)
                    .child(input_entity)
            })
        } else {
            let mut box_ = div()
                .flex_1()
                .min_w(px(0.))
                .h(px(m.input_height))
                .flex()
                .items_center()
                .px(px(m.input_padding_inline))
                .rounded(px(m.bare_radius))
                .bg(st.input_bg);
            box_ = match st.input_border {
                Some(color) => box_.border_1().border_color(color),
                None => box_,
            };
            if focused {
                box_ = box_.shadow(theme::focus_ring());
            }
            self.input.update(cx, |input, cx| {
                let host = box_
                    .id(ElementId::Name(input_id.into()))
                    .track_focus(&input.focus_handle(cx));
                input
                    .with_a11y_actions(input_semantics.apply(host), cx)
                    .child(input_entity)
            })
        });

        if let Some(labels) = self.actions.clone() {
            for (ix, label) in labels.into_iter().enumerate() {
                let go = ix == 3;
                let active =
                    (ix == 0 && panel == Panel::Snippets) || (ix == 1 && panel == Panel::History);
                let disabled = match ix {
                    0 => snippets.is_empty(),
                    1 => history.is_empty(),
                    _ => !can_send,
                };
                let kind = if disabled {
                    if go {
                        CommandBarAction::GoDisabled
                    } else {
                        CommandBarAction::Disabled
                    }
                } else if active {
                    CommandBarAction::Active
                } else if go {
                    CommandBarAction::Go
                } else {
                    CommandBarAction::Default
                };
                let a = theme::with(|t| command_bar_action_style(t, kind));
                let mut btn = div()
                    .id(ElementId::Name(format!("cmdbar-{uid}-act-{ix}").into()))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .h(px(m.action_height))
                    .px(px(m.action_padding_inline))
                    .rounded(px(m.bare_radius))
                    .border_1()
                    .border_color(a.border)
                    .text_size(px(m.action_font_size))
                    .text_color(a.fg)
                    .child(label);
                btn = match a.bg {
                    Some(color) => btn.bg(color),
                    None => btn,
                };
                let mut semantics =
                    a11y::Semantics::new(Role::Button, self.actions.as_ref().unwrap()[ix].clone())
                        .id(format!("cmdbar-{uid}-act-{ix}"))
                        .disabled(disabled)
                        .focusable(!disabled);
                if ix < 2 {
                    semantics = semantics.expanded(active);
                }
                btn = semantics.apply(btn);
                bar = bar.child(if disabled {
                    btn.cursor(CursorStyle::Arrow)
                } else {
                    let btn = btn.cursor(CursorStyle::PointingHand).hover(|s| {
                        let s = s.border_color(a.hover_border).text_color(a.hover_fg);
                        match a.hover_bg {
                            Some(color) => s.bg(color),
                            None => s,
                        }
                    });
                    a11y::click(
                        btn,
                        cx.listener(move |this, _: &gpui::ClickEvent, window, cx| match ix {
                            0 => this.toggle_panel(Panel::Snippets, cx),
                            1 => this.toggle_panel(Panel::History, cx),
                            2 => this.clear_text(cx),
                            _ => this.submit(window, cx),
                        }),
                    )
                });
            }
        } else {
            bar = bar.child(
                div()
                    .flex_shrink_0()
                    .text_xs()
                    .text_color(st.hint_fg)
                    .child(history_hint),
            );
        }
        root = root.child(bar);

        // 浮层：贴在条上方、右对齐。历史是命令列表；片段在给了 `snippet_items`
        // 时换成原版 SnippetPanel 的面板形态（定宽 320 + 标题 + 分组 + 等宽命令列）。
        if panel != Panel::None {
            let scroll_key = if panel == Panel::History {
                "history"
            } else {
                "snippets"
            };
            let scroll_id = format!("{entity_id}-{scroll_key}-scroll");
            let shell = div()
                .id(ElementId::Name(format!("cmdbar-{uid}-pop").into()))
                .absolute()
                .right(px(m.bare_padding_inline))
                .bottom(px(m.bar_height + m.pop_gap))
                .flex()
                .flex_col()
                .overflow_hidden()
                .occlude()
                .rounded(px(m.pop_radius))
                .border_1()
                .border_color(theme::border())
                .bg(theme::bg_container())
                .text_color(theme::text())
                .shadow(shadow);
            let shell = if self.dismiss_outside {
                shell.on_mouse_down_out(cx.listener(|this, _: &gpui::MouseDownEvent, _, cx| {
                    this.close_panel();
                    cx.notify();
                }))
            } else {
                shell
            };

            let panel_label = if panel == Panel::History {
                "历史命令"
            } else {
                "命令片段"
            };
            let shell = a11y::Semantics::new(Role::Group, panel_label)
                .id(format!("cmdbar-{uid}-pop"))
                .apply(shell);

            let pop = if panel == Panel::Snippets && (self.manage || !self.snippet_items.is_empty())
            {
                let mut pop = shell
                    .w(px(m.snip_width))
                    .max_h(px(m.snip_max_height))
                    .p(px(m.snip_padding))
                    .flex_col();
                let a = theme::with(|t| command_bar_action_style(t, CommandBarAction::Default));
                let mut head = div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .pt(px(m.snip_head_padding_top))
                    .px(px(m.snip_head_padding_inline))
                    .pb(px(m.snip_head_padding_bottom))
                    .child(
                        div()
                            .text_size(px(m.snip_title_font_size))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(st.snip_title_fg)
                            .child("命令片段"),
                    );
                if self.manage {
                    // 管理态：「新建」开内联表单；表单开着时藏按钮（原版 v-if=!formOpen）。
                    if self.form.is_none() {
                        let new_button = div()
                            .id(ElementId::Name(format!("cmdbar-{uid}-snip-new").into()))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(m.snip_new_gap))
                            .h(px(m.snip_new_height))
                            .px(px(m.snip_new_padding_inline))
                            .rounded(px(m.bare_radius))
                            .border_1()
                            .border_color(a.border)
                            .text_size(px(m.action_font_size))
                            .text_color(a.fg)
                            .cursor(CursorStyle::PointingHand)
                            .hover(|s| {
                                let s = s.border_color(a.hover_border).text_color(a.hover_fg);
                                match a.hover_bg {
                                    Some(color) => s.bg(color),
                                    None => s,
                                }
                            })
                            .child("+")
                            .child("新建");
                        let new_button = a11y::Semantics::new(Role::Button, "新建命令片段")
                            .id(format!("cmdbar-{uid}-snip-new"))
                            .focusable(true)
                            .apply(new_button);
                        head = head.child(a11y::click(
                            new_button,
                            cx.listener(|this, _: &gpui::ClickEvent, _, cx| {
                                this.open_snippet_new(cx)
                            }),
                        ));
                    }
                } else if let Some(cb) = self.on_new_snippet.clone() {
                    let new_button = div()
                        .id(ElementId::Name(format!("cmdbar-{uid}-snip-new").into()))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(m.snip_new_gap))
                        .h(px(m.snip_new_height))
                        .px(px(m.snip_new_padding_inline))
                        .rounded(px(m.bare_radius))
                        .border_1()
                        .border_color(a.border)
                        .text_size(px(m.action_font_size))
                        .text_color(a.fg)
                        .cursor(CursorStyle::PointingHand)
                        .hover(|s| {
                            let s = s.border_color(a.hover_border).text_color(a.hover_fg);
                            match a.hover_bg {
                                Some(color) => s.bg(color),
                                None => s,
                            }
                        })
                        .child("+")
                        .child("新建");
                    let new_button = a11y::Semantics::new(Role::Button, "新建命令片段")
                        .id(format!("cmdbar-{uid}-snip-new"))
                        .focusable(true)
                        .apply(new_button);
                    head = head.child(a11y::click(new_button, {
                        // 不能用 cx.listener：派发期间实体的可变借用还没释放，
                        // 宿主回调里再 `handle.update` 同一个实体就是再入借用。
                        let cb = cb.clone();
                        move |_, window, cx| cb(window, cx)
                    }));
                }
                pop = pop.child(head);

                if let Some(form) = self.form.as_ref() {
                    let mut form_box = div()
                        .flex()
                        .flex_col()
                        .gap(px(m.snip_form_gap))
                        .p(px(m.snip_form_padding))
                        .mb(px(m.snip_form_margin_bottom))
                        .border_1()
                        .border_color(theme::border())
                        .rounded(px(m.snip_form_radius))
                        .bg(st.snip_form_bg)
                        .child(form.name.clone())
                        .child(form.group.clone())
                        .child(form.command.clone());
                    let checked = form.send_enter;
                    let check = div()
                        .id(ElementId::Name(format!("cmdbar-{uid}-snip-check").into()))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(m.snip_check_gap))
                        .cursor(CursorStyle::PointingHand)
                        .child({
                            let mut check_box = div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .flex_shrink_0()
                                .w(px(m.snip_check_box))
                                .h(px(m.snip_check_box))
                                .rounded(px(m.snip_check_radius))
                                .border_1()
                                .border_color(if checked {
                                    theme::accent()
                                } else {
                                    theme::border()
                                })
                                .bg(if checked {
                                    theme::accent()
                                } else {
                                    theme::surface()
                                });
                            if checked {
                                check_box = check_box.child(
                                    div()
                                        .text_size(px(m.snip_check_glyph))
                                        .text_color(st.snip_check_box_fg)
                                        .child("✓"),
                                );
                            }
                            check_box
                        })
                        .child(
                            div()
                                .text_size(px(m.snip_check_font_size))
                                .text_color(st.snip_check_fg)
                                .child("点击后直接执行（不勾只填进输入条）"),
                        );
                    let check =
                        a11y::Semantics::new(Role::CheckBox, "点击后直接执行（不勾只填进输入条）")
                            .id(format!("cmdbar-{uid}-snip-check"))
                            .toggled(if checked {
                                Toggled::True
                            } else {
                                Toggled::False
                            })
                            .focusable(true)
                            .apply(check);
                    form_box = form_box.child(a11y::click(
                        check,
                        cx.listener(|this, _: &gpui::ClickEvent, _, cx| {
                            if let Some(f) = this.form.as_mut() {
                                f.send_enter = !f.send_enter;
                            }
                            cx.notify();
                        }),
                    ));
                    form_box = form_box.child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(m.snip_foot_gap))
                            .child({
                                let cancel = div()
                                    .id(ElementId::Name(format!("cmdbar-{uid}-snip-cancel").into()))
                                    .flex()
                                    .items_center()
                                    .h(px(m.snip_btn_height))
                                    .px(px(m.snip_btn_padding_inline))
                                    .border_1()
                                    .border_color(theme::border())
                                    .rounded(px(m.bare_radius))
                                    .text_size(px(m.snip_btn_font_size))
                                    .text_color(st.snip_btn_fg)
                                    .cursor(CursorStyle::PointingHand)
                                    .hover(|s| {
                                        s.text_color(st.snip_btn_hover_fg)
                                            .border_color(st.snip_btn_hover_border)
                                    })
                                    .child("取消");
                                let cancel = a11y::Semantics::new(Role::Button, "取消编辑")
                                    .id(format!("cmdbar-{uid}-snip-cancel"))
                                    .focusable(true)
                                    .apply(cancel);
                                a11y::click(
                                    cancel,
                                    cx.listener(|this, _: &gpui::ClickEvent, _, cx| {
                                        this.form = None;
                                        cx.notify();
                                    }),
                                )
                            })
                            .child({
                                let save = div()
                                    .id(ElementId::Name(format!("cmdbar-{uid}-snip-save").into()))
                                    .flex()
                                    .items_center()
                                    .h(px(m.snip_btn_height))
                                    .px(px(m.snip_btn_padding_inline))
                                    .border_1()
                                    .border_color(st.snip_save_fg)
                                    .rounded(px(m.bare_radius))
                                    .text_size(px(m.snip_btn_font_size))
                                    .text_color(st.snip_save_fg)
                                    .cursor(CursorStyle::PointingHand)
                                    .hover(|s| {
                                        s.text_color(st.snip_btn_hover_fg)
                                            .border_color(st.snip_btn_hover_border)
                                    })
                                    .child("保存");
                                let save = a11y::Semantics::new(Role::Button, "保存命令片段")
                                    .id(format!("cmdbar-{uid}-snip-save"))
                                    .focusable(true)
                                    .apply(save);
                                a11y::click(
                                    save,
                                    cx.listener(|this, _: &gpui::ClickEvent, window, cx| {
                                        this.save_snippet_form(window, cx);
                                    }),
                                )
                            }),
                    );
                    pop = pop.child(form_box);
                }

                let list_id = format!("{entity_id}-snippets-list");
                let mut scroll_content = div()
                    .id(ElementId::Name(list_id.clone().into()))
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.));
                if self.manage && self.snippet_items.is_empty() && self.form.is_none() {
                    scroll_content = scroll_content.child(
                        a11y::Semantics::new(Role::Status, self.empty_text[1].clone())
                            .id(format!("{entity_id}-snippets-empty"))
                            .apply(
                                div()
                                    .id(ElementId::Name(
                                        format!("{entity_id}-snippets-empty").into(),
                                    ))
                                    .w_full()
                                    .text_center()
                                    .py(px(m.snip_empty_padding_block))
                                    .px(px(m.snip_empty_padding_inline))
                                    .text_size(px(m.snip_name_font_size))
                                    .line_height(px(m.snip_name_font_size * 1.6))
                                    .text_color(st.pop_empty_fg)
                                    .child(self.empty_text[1].clone()),
                            ),
                    );
                }

                let snippet_count = self.snippet_items.len();
                let mut option_index = 0usize;
                let groups = if self.manage {
                    panel_groups(&self.snippet_items, "未分组")
                } else {
                    group_snippets(&self.snippet_items, "未分组")
                };
                for (gi, (gname, items)) in groups.into_iter().enumerate() {
                    let mut group = div().flex().flex_col();
                    if gi > 0 {
                        group = group.mt(px(m.snip_group_gap));
                    }
                    group = group.child(
                        div()
                            .pt(px(m.snip_group_name_padding_top))
                            .px(px(m.snip_group_name_padding_inline))
                            .pb(px(m.snip_group_name_padding_bottom))
                            .text_size(px(m.snip_group_name_font_size))
                            .text_color(st.snip_group_fg)
                            .child(gname),
                    );
                    for (ix, item) in items.into_iter().enumerate() {
                        let command = item.command.to_string();
                        let row_id = ElementId::Name(format!("cmdbar-{uid}-snip-{gi}-{ix}").into());
                        let name_el = div()
                            .flex_shrink_0()
                            .text_size(px(m.snip_name_font_size))
                            .text_color(st.snip_name_fg)
                            .child(item.name.clone());
                        let cmd_el = div()
                            .flex_1()
                            .min_w(px(0.))
                            .text_size(px(m.snip_cmd_font_size))
                            .font_family(mono_font_family())
                            .text_color(st.snip_cmd_fg)
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(item.command.clone());
                        let row =
                            if self.manage {
                                // 原版 .snip__item：整行 center 对齐 + 悬停底色；
                                // 名称/命令在内层 baseline 行里，图标槽贴右、悬停才显形。
                                let row_group = format!("cmdbar-{uid}-sniprow-{gi}-{ix}");
                                let send_enter = item.send_enter;
                                let mut row =
                                    div()
                                        .id(row_id)
                                        .group(row_group.clone())
                                        .w_full()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .px(px(m.snip_row_padding_inline))
                                        .py(px(m.snip_row_padding_block))
                                        .rounded(px(m.bare_radius))
                                        .hover(|s| s.bg(st.pop_row_hover_bg))
                                        .child({
                                            let use_id = format!("{row_group}-use");
                                            let use_item = div()
                                                .id(ElementId::Name(use_id.clone().into()))
                                                .flex_1()
                                                .min_w(px(0.))
                                                .flex()
                                                .flex_row()
                                                .items_baseline()
                                                .gap(px(m.snip_row_gap))
                                                .cursor(CursorStyle::PointingHand)
                                                .child(name_el)
                                                .child(cmd_el);
                                            let use_item = a11y::Semantics::new(
                                                Role::ListBoxOption,
                                                item.name.clone(),
                                            )
                                            .id(use_id)
                                            .value(item.command.clone())
                                            .selected(false)
                                            .position(option_index + 1, snippet_count)
                                            .apply(use_item);
                                            a11y::click(
                                        use_item,
                                        cx.listener(move |this, _: &gpui::ClickEvent, window, cx| {
                                            if send_enter {
                                                if let Some(cb) = this.on_submit.clone() {
                                                    cb(&command, window, cx);
                                                }
                                            } else {
                                                this.insert_snippet(command.clone(), window, cx);
                                            }
                                            this.close_panel();
                                            cx.notify();
                                        }),
                                    )
                                        });
                                if let Some(builder) = self.icon_builder.clone() {
                                    for (kind, danger) in
                                        [(SnippetIcon::Edit, false), (SnippetIcon::Trash, true)]
                                    {
                                        let slot_group = format!(
                                            "{row_group}-{}",
                                            if danger { "trash" } else { "edit" }
                                        );
                                        let icon = builder(kind, &slot_group);
                                        let id = item.id;
                                        let slot_id = format!("{slot_group}-button");
                                        let button = div()
                                            .id(ElementId::Name(slot_id.clone().into()))
                                            .group(slot_group)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .flex_shrink_0()
                                            .w(px(m.snip_icon_size))
                                            .h(px(m.snip_icon_size))
                                            .rounded(px(m.snip_icon_radius))
                                            .cursor(CursorStyle::PointingHand)
                                            // 显隐走 opacity：div 的 opacity 会经
                                            // with_element_opacity 级联到子 svg 蒙版。
                                            .opacity(0.)
                                            .group_hover(row_group.clone(), |s| s.opacity(1.))
                                            .child(icon);
                                        let button = a11y::Semantics::new(
                                            Role::Button,
                                            format!(
                                                "{}命令片段：{}",
                                                if danger { "删除" } else { "编辑" },
                                                item.name
                                            ),
                                        )
                                        .id(slot_id)
                                        .focusable(true)
                                        .apply(button);
                                        row = row.child(a11y::click(
                                            button,
                                            cx.listener(
                                                move |this, _: &gpui::ClickEvent, window, cx| {
                                                    if danger {
                                                        let target = this
                                                            .snippet_items
                                                            .iter()
                                                            .find(|s| s.id == id)
                                                            .cloned();
                                                        if let (Some(cb), Some(target)) =
                                                            (this.on_snippet_event.clone(), target)
                                                        {
                                                            cb(
                                                                SnippetEvent::DeleteRequested(
                                                                    target,
                                                                ),
                                                                window,
                                                                cx,
                                                            );
                                                        }
                                                    } else {
                                                        this.open_snippet_edit(id, cx);
                                                    }
                                                },
                                            ),
                                        ));
                                    }
                                }
                                row
                            } else {
                                let row = div()
                                    .id(row_id)
                                    .w_full()
                                    .flex()
                                    .flex_row()
                                    .items_baseline()
                                    .gap(px(m.snip_row_gap))
                                    .px(px(m.snip_row_padding_inline))
                                    .py(px(m.snip_row_padding_block))
                                    .rounded(px(m.bare_radius))
                                    .cursor(CursorStyle::PointingHand)
                                    // 悬停只换底色，名称与命令的文字色都不动。
                                    .hover(|s| s.bg(st.pop_row_hover_bg))
                                    .child(name_el)
                                    .child(cmd_el);
                                let row =
                                    a11y::Semantics::new(Role::ListBoxOption, item.name.clone())
                                        .id(format!("cmdbar-{uid}-snip-{gi}-{ix}"))
                                        .value(item.command.clone())
                                        .selected(false)
                                        .position(option_index + 1, snippet_count)
                                        .apply(row);
                                a11y::click(
                                    row,
                                    cx.listener(move |this, _: &gpui::ClickEvent, window, cx| {
                                        this.insert_snippet(command.clone(), window, cx);
                                        this.close_panel();
                                    }),
                                )
                            };
                        option_index += 1;
                        group = group.child(row);
                    }
                    scroll_content = scroll_content.child(group);
                }
                let scroll_content = a11y::Semantics::new(Role::ListBox, "命令片段")
                    .id(format!("{entity_id}-snippets-list"))
                    .orientation(Orientation::Vertical)
                    .apply(scroll_content);
                pop.child(VerticalScroll::new(
                    ElementId::Name(scroll_id.clone().into()),
                    scroll_content.into_any_element(),
                ))
            } else {
                let rows: Vec<SharedString> = if panel == Panel::Snippets {
                    snippets.clone()
                } else {
                    history.clone()
                };
                let empty = self.empty_text[usize::from(panel == Panel::Snippets)].clone();
                let pop = shell
                    .min_w(px(m.pop_min_width))
                    .max_h(px(m.pop_max_height))
                    .p(px(m.pop_padding))
                    .flex_col();
                let list_id = format!("{entity_id}-{scroll_key}-list");
                let mut scroll_content = div()
                    .id(ElementId::Name(list_id.clone().into()))
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.));
                if rows.is_empty() {
                    // 历史空态是 8px 等距，片段空态上下 12px（原版两处 padding 不同）。
                    let (block, inline) = if panel == Panel::Snippets {
                        (m.snip_empty_padding_block, m.snip_empty_padding_inline)
                    } else {
                        (m.pop_empty_padding, m.pop_empty_padding)
                    };
                    scroll_content = scroll_content.child(
                        a11y::Semantics::new(Role::Status, empty.clone())
                            .id(format!("{entity_id}-{scroll_key}-empty"))
                            .apply(
                                div()
                                    .id(ElementId::Name(
                                        format!("{entity_id}-{scroll_key}-empty").into(),
                                    ))
                                    .w_full()
                                    .text_center()
                                    .py(px(block))
                                    .px(px(inline))
                                    .text_size(px(m.pop_row_font_size))
                                    .text_color(st.pop_empty_fg)
                                    .child(empty),
                            ),
                    );
                }
                let row_count = rows.len();
                for (ix, row) in rows.into_iter().enumerate() {
                    let text = row.to_string();
                    let row_id = format!("cmdbar-{uid}-{scroll_key}-{ix}");
                    let selected = panel == Panel::History
                        && self.history_idx == Some(self.history.len().saturating_sub(ix + 1));
                    let row = div()
                        .id(ElementId::Name(row_id.clone().into()))
                        .w_full()
                        .px(px(m.pop_row_padding_inline))
                        .py(px(m.pop_row_padding_block))
                        .rounded(px(m.bare_radius))
                        .text_size(px(m.pop_row_font_size))
                        .font_family(mono_font_family())
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .cursor(CursorStyle::PointingHand)
                        .hover(|s| s.bg(st.pop_row_hover_bg).text_color(st.pop_row_hover_fg))
                        .child(row);
                    let row = a11y::Semantics::new(Role::ListBoxOption, text.clone())
                        .id(row_id)
                        .value(text.clone())
                        .selected(selected)
                        .position(ix + 1, row_count)
                        .apply(row);
                    scroll_content = scroll_content.child(a11y::click(
                        row,
                        cx.listener(move |this, _: &gpui::ClickEvent, window, cx| {
                            this.insert_snippet(text.clone(), window, cx);
                            this.panel = Panel::None;
                        }),
                    ));
                }
                if panel == Panel::History && !history.is_empty() {
                    let clear = div()
                        .id(ElementId::Name(format!("cmdbar-{uid}-pop-clear").into()))
                        .w_full()
                        .mt_1()
                        .border_t_1()
                        .border_color(theme::border())
                        .px(px(m.pop_row_padding_inline))
                        .py(px(m.pop_row_padding_block))
                        .text_size(px(m.pop_row_font_size))
                        .text_color(st.pop_empty_fg)
                        .cursor(CursorStyle::PointingHand)
                        .hover(|s| s.text_color(st.pop_danger_hover_fg))
                        .child("清空历史");
                    let clear = a11y::Semantics::new(Role::Button, "清空历史")
                        .id(format!("cmdbar-{uid}-pop-clear"))
                        .focusable(true)
                        .apply(clear);
                    scroll_content = scroll_content.child(a11y::click(
                        clear,
                        cx.listener(|this, _: &gpui::ClickEvent, _, cx| this.clear_history(cx)),
                    ));
                }
                let scroll_content = a11y::Semantics::new(Role::ListBox, panel_label)
                    .id(format!("{entity_id}-{scroll_key}-list"))
                    .orientation(Orientation::Vertical)
                    .apply(scroll_content);
                pop.child(VerticalScroll::new(
                    ElementId::Name(scroll_id.into()),
                    scroll_content.into_any_element(),
                ))
            };
            root = root.child(pop);
        }

        if self.actions.is_none() && !snippets.is_empty() {
            let mut chips = div().flex().flex_row().flex_wrap().gap(px(m.chip_gap));
            for (i, snippet) in snippets.into_iter().enumerate() {
                let label = snippet.to_string();
                let insert = snippet;
                let chip = div()
                    .id(ElementId::Name(format!("cmdbar-{uid}-chip-{i}").into()))
                    .cursor(CursorStyle::PointingHand)
                    .px_2()
                    .py(px(m.chip_padding_block))
                    .rounded(px(m.chip_radius))
                    .border_1()
                    .border_color(theme::border())
                    .bg(theme::primary_bg())
                    .text_xs()
                    .text_color(theme::primary())
                    .hover(|style| style.border_color(theme::primary()))
                    .child(label.clone());
                let chip = a11y::Semantics::new(Role::Button, label)
                    .id(format!("cmdbar-{uid}-chip-{i}"))
                    .focusable(true)
                    .apply(chip);
                chips = chips.child(a11y::click(
                    chip,
                    cx.listener(move |this, _: &gpui::ClickEvent, window, cx| {
                        this.insert_snippet(insert.to_string(), window, cx);
                    }),
                ));
            }
            root = root.child(chips);
        }

        root
    }
}

impl Focusable for CommandBar {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.focus_handle(cx)
    }
}
