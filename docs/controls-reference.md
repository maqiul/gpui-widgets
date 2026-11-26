[English](controls-reference.en.md) | [中文](controls-reference.md)

# gpui-widgets 控件参考手册

> 本手册逐个说明库内 **98 个已文档化控件 / 101 个条目**，并补充 **8 个公开高级/宿主模块**；每项均给出**定位、创建签名、状态模型、属性、回调（含精确类型）、最小示例、注意事项**。
>
> 配套文档分工：
> - [`README.md`](../README.md)：控件分类一览表 + 使用约定（速查）。
> - [`integration-guide.md`](integration-guide.md)：如何在自己的 Rust 项目里引入、初始化、跑通完整示例。
> - **本手册**：单个控件的完整用法参考（按需查阅）。
>
> 所有签名均取自 `widgets/src/*.rs` 源码核实，回调事件类型逐一确认，可直接照抄。

---

## 目录

- [一、核心概念（用任何控件前必读）](#一核心概念用任何控件前必读)
- [二、通用](#二通用)：Button · Link · Title/Paragraph/Text · Kbd
- [三、布局](#三布局)：Row/Col · Header/Sider/Content/Footer · Space · Splitter · Scrollbar
- [四、导航](#四导航)：Anchor · Breadcrumb · DropdownMenu · NavMenu · Pagination · Steps · Tabs · ContextMenu · FloatButton · CommandPalette
- [五、数据录入](#五数据录入)：TextInput … FormItem（28 个）
- [六、数据展示](#六数据展示)：Table … Tour（33 个）
- [七、反馈](#七反馈)：Alert … Watermark（12 个）
- [八、辅助与主题](#八辅助与主题)：misc · theme · FpsMeter
- [九、AI 对话](#九ai-对话)：Welcome · Prompts · Bubble · Sender · Conversations · ThoughtChain
- [十、高级与宿主模块](#十高级与宿主模块)：HotkeyInput · Markdown · TransferManager · Terminal · CodeEditor · CommandBar · DropZone · TitleBar（8 个模块）
- [附录 A：回调签名速查表](#附录-a回调签名速查表)
- [附录 B：Entity 控件清单](#附录-bentity-控件清单)
- [附录 C：需唯一 id 的控件清单](#附录-c需唯一-id-的控件清单)
- [附录 D：常见坑 FAQ](#附录-d常见坑-faq)

---

## 一、核心概念（用任何控件前必读）

理解下面 6 点，几乎所有控件都能正确使用。

### 1.1 三种状态模型

库里的控件分三种「状态归属」，决定了你怎么创建、怎么读写：

| 模型 | 含义 | 代表控件 | 用法要点 |
|---|---|---|---|
| **受控组件** | 控件内部**不存状态**，值由你在 render 时传入，回调里改你自己视图 `Entity` 的字段再 `cx.notify()` | Button / Checkbox / Select / Table / Calendar / Transfer …（绝大多数） | 每帧重建，值来自 `self.xxx` |
| **Entity（有状态视图）** | 控件内部持有光标 / 选区 / 解码器 / 剩余秒数等，是 `Entity<T>` | TextInput 系 / VideoPlayer / Countdown / CommandPalette | `cx.new(\|cx\| T::new(cx))` 创建**一次**，render 中 `.clone()` 挂载 |
| **包装器** | 包裹任意子内容，附加一层行为 / 视觉 | Tooltip / Popover / ContextMenu / Watermark / CountBadge / FormItem | `Wrapper::new(…, child)` 或 `.child(…)` |

**受控组件范式**（最重要，覆盖绝大多数控件）：

```rust
struct MyApp { city: usize, city_open: bool }

impl Render for MyApp {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Select::new("city", ["北京", "上海", "广州"], self.city)
            .open(self.city_open)
            .on_open_change(cx.listener(|this, open, _, cx| {
                this.city_open = *open;      // 回写展开状态
                cx.notify();
            }))
            .on_select(cx.listener(|this, ix, _, cx| {
                this.city = *ix;             // 回写选中值
                this.city_open = false;      // 单选选中后收起
                cx.notify();
            }))
    }
}
```

**Entity 范式**：

```rust
// 创建（main 或父视图初始化时，cx 是 Context<TextInput>）：
name: cx.new(|cx| TextInput::new(cx).placeholder("请输入姓名")),
// render 中：
div().child(self.name.clone())
// 事件中读取 / 写入：
let text = self.name.read(cx).text();
self.name.update(cx, |i, cx| i.set_text("新值", window, cx));
```

> ⚠️ `TextInput::new(cx)` 的 `cx` 是 `Context<TextInput>`，**只能**在 `cx.new(|cx| …)` 闭包里调用，不能在父视图 render / 分区方法里直接 new。

### 1.2 回调签名三族

回调按参数个数分三族，**构造方式不同**，用错会编译失败：

| 族 | 签名 | 构造方式 | 代表 |
|---|---|---|---|
| **3 参** | `Fn(&E, &mut Window, &mut App)` | `cx.listener(\|this, event, window, cx\| …)` | Button.on_click、Select.on_select、Table.on_row_click …（绝大多数） |
| **2 参** | `Fn(&mut Window, &mut App)`（无事件参） | `cx.entity()` 句柄 + `entity.update(cx, …)` | Modal/Drawer.on_close、MenuItem.on_click、FloatButton.on_click、Popconfirm、Tour.on_close、Select/DatePicker.on_clear、Upload.on_click、Link.on_click |
| **1 参** | `Fn(&mut App)`（定时器上下文无 Window） | `cx.entity()` 句柄 | Countdown.on_finish |
| **特例** | `Fn(&Vec<PathBuf>, &mut App)`（异步任务无 Window） | `cx.entity()` 句柄 | FilePicker.on_pick |

```rust
// 3 参：直接 cx.listener
Button::new("确定").on_click(cx.listener(|this, _event: &ClickEvent, _, cx| {
    this.do_something();
    cx.notify();
}))

// 2 参：用实体句柄（on_close 无事件参数，不适用 cx.listener）
let entity = cx.entity();
Modal::new("标题").on_close(move |_window, cx| {
    let _ = entity.update(cx, |this, cx| { this.modal_open = false; cx.notify(); });
})

// 1 参：Countdown 归零（定时器线程上下文）
let entity = cx.entity();
Countdown::after(Duration::from_secs(90), cx).on_finish(move |cx| {
    let _ = entity.update(cx, |this, cx| { /* 弹 Toast */ });
})
```

> 完整每个控件的回调事件类型见[附录 A](#附录-a回调签名速查表)。

### 1.3 id 唯一性与 hover

GPUI 的 hover / 拖拽 / keyed-state 按 `ElementId` 判定。**同一窗口内多个实例的 id 必须唯一**，否则悬停高亮、拖拽状态会串扰。需要 id 的控件见[附录 C](#附录-c需唯一-id-的控件清单)。

- 字符串 id 用 `impl Into<ElementId>`（如 `"volume"`、`format!("row-{i}")`）。
- 列表里批量生成控件时，用下标 / 业务 key 拼唯一 id。

### 1.4 浮层定位机制（共享约定）

Select / MultiSelect / DropdownMenu / Popover / DatePicker / DateRangePicker / ColorPicker / Cascader / TimePicker / TreeSelect / Popconfirm 的浮层共享同一套定位：

- 贴触发框**下方 4px** 展开；窗口下方放不下时**自动翻转到上方**。
- **点击外部收起**。
- 展开状态**受控**：`.open(bool)` + `.on_open_change(&bool)` 回写。
- 菜单宽度默认等于触发框宽度，可用 `.menu_width(px(…))` 覆盖（Select / MultiSelect）。

### 1.5 主题系统

所有控件颜色取自 `theme` 模块的 **21 个语义色**（`bg()` / `surface()` / `accent()` / `text()` / `track()` / `text_disabled()` …），全局可配置：

```rust
theme::set_theme(theme::Theme::dark());     // 整体切明暗
theme::set_accent(gpui::rgb(0x3B82F6));      // 换品牌主色（悬停/按下/柔和底自动派生）
theme::update(|t| { t.success = …; });       // 微调单个语义色
```

详见[八、辅助与主题](#八辅助与主题)。

### 1.6 启动初始化（bind_keys）

应用启动时按需调用一次（整个应用一次，不是每个视图）：

```rust
gpui_platform::application().run(|cx: &mut App| {
    gpui_widgets::text_input::bind_keys(cx);        // 必须：输入框 Ctrl+A/C/V/X、方向键
    // gpui_widgets::text_area::bind_keys(cx);       // 用 TextArea 时（回车/上下/Shift 选择）
    // gpui_widgets::command_palette::bind_keys(cx); // 用 CommandPalette 时（Ctrl+K）
    // … open_window …
});
```

> 忘记 `text_input::bind_keys` 的典型症状：输入框能打字，但 Ctrl+C/V、全选、方向键移动光标全部无效。快捷键用 `secondary` 书写，由 `bind_keys` 按宿主 OS 解析为 Cmd（macOS）或 Ctrl（其余平台）。

---

## 二、通用

### Button · 按钮

触发一次动作。**受控组件**。

- **创建**：`Button::new(label: impl Into<SharedString>)`，如 `Button::new("确定")`。
- **属性**：
  - 变体：`primary()` / `secondary()` / `ghost()` / `danger()`。
  - 尺寸：`small()` / `large()`。
  - `loading()`：加载态，**期间阻断点击**。
  - `block()`：占满整行；`icon(char)`：前置图标字符；`round()` 胶囊 / `circle()` 圆形；`disabled()`。
- **回调**：`on_click(&ClickEvent)`（3 参）。
- **注意**：`loading()` 与 `disabled()` 都会让点击不触发回调。

```rust
Button::new("提交").primary().loading(self.submitting)
    .on_click(cx.listener(|this, _, _, cx| { this.submit(cx); }))
```

### Link · 链接

悬停下划线的文本链接。**受控组件**。

- **创建**：`Link::new(label)`。
- **属性**：`href(url)`：点击打开系统浏览器；`disabled()`。
- **回调**：`on_click(&mut Window, &mut App)`（**2 参**，无事件参）——自定义点击行为，与 `href` 二选一。
- **注意**：`on_click` 是 2 参族，用 `cx.entity()` 句柄构造，不能用 `cx.listener`。

```rust
Link::new("打开官网").href("https://gpui.rs")
// 或自定义：
let entity = cx.entity();
Link::new("详情").on_click(move |_w, cx| { let _ = entity.update(cx, |this, cx| { … }); })
```

### Title / Paragraph / Text · 排版三件套

纯展示文本。**受控组件**（无回调）。

- **创建**：
  - `Title::new(level: u8, text)`：1~5 级标题。
  - `Paragraph::new(text)`：段落。
  - `Text::new(text)`：行内文本。
- **属性**（Text 系）：`strong()` / `italic()` / `underline()` / `delete()` / `code()` / `mark()` / `secondary()`。
- **注意**：尺寸 / 颜色等可通过 `Styled` 转发（`.text_color()` / `.text_size()` 等）。

```rust
div().flex().flex_col().gap_2()
    .child(Title::new(2, "章节标题"))
    .child(Paragraph::new("这是一段说明文字。"))
    .child(Text::new("行内强调").strong())
```

### Kbd · 键帽

键盘按键可视化。**受控组件**（无回调）。

- **创建**：`Kbd::new(key)` 单键；`Kbd::combo(combo)` 组合键（自动按 `+` 拆分成多个键帽）。
- **注意**：`Kbd::combo("Ctrl+K")` 会渲染成 `Ctrl` `K` 两个键帽。

```rust
Kbd::new("Esc")
Kbd::combo("Ctrl+K")
```

---

## 三、布局

### Row / Col · 24 栅格

响应式栅格布局。**受控组件**（无回调）。

- **创建**：`Row::new()`；`Col::new(span: u8)`（span 为 1~24 栅格份数）。
- **属性**：`Col::offset(n)` 左偏移栅格数；`Row::gutter(px)` 列间距。
- **用法**：`Row` 内放若干 `Col`，`Col` 内放任意内容。

```rust
Row::new().gutter(px(16.))
    .child(Col::new(12).child(left_panel))
    .child(Col::new(12).child(right_panel))
```

### Header / Sider / Content / Footer · 页面布局四件套

经典后台页面骨架。**受控组件**（无回调）。

- **创建**：`Header::new()` / `Sider::new(width: f32)` / `Content::new()` / `Footer::new()`。
- **用法**：外层 `div().flex().flex_col()` 放 Header + 中部 `div().flex().flex_1()`（Sider + Content）+ Footer。

```rust
div().size_full().flex().flex_col()
    .child(Header::new().child(top_bar))
    .child(div().flex().flex_1()
        .child(Sider::new(220.).child(nav))
        .child(Content::new().child(main)))
    .child(Footer::new().child(footer))
```

### Space · 间距容器

统一控制子元素间距。**受控组件**（无回调）。

- **创建**：`Space::new()`。
- **属性**：`vertical()` 纵向排列；`wrap()` 允许换行；`size(px)` 间距大小。

```rust
Space::new().size(px(8.)).wrap()
    .child(Button::new("A"))
    .child(Button::new("B"))
```

### Splitter · 分割面板

可拖拽调整比例的两栏。**受控组件**，**需唯一 id**。

- **创建**：`Splitter::new(id: impl Into<ElementId>, left, right)`。
- **属性**：`vertical()` 改为上下两栏；`ratio(f32)` 受控比例（0~1）。
- **回调**：`on_change(&f32)`（3 参）——拖拽分隔条时回传新比例。
- **注意**：`ratio` 受控，必须在 `on_change` 里回写 `self.ratio` 再 `cx.notify()`，否则拖拽不生效。

```rust
Splitter::new("main-split", left_panel, right_panel)
    .ratio(self.ratio)
    .on_change(cx.listener(|this, r, _, cx| { this.ratio = *r; cx.notify(); }))
```

### Scrollbar · 主题化滚动条

GPUI 默认不画滚动条，本控件以库主题色补上可见、可交互的滚动反馈。**包装器 / 绑定持久 ScrollHandle**，**需唯一 id**。

- **创建**：`Scrollbar::horizontal(id, handle)` / `Scrollbar::vertical(id, handle)`，`handle: ScrollHandle`。
- **视觉**：自绘轨道（`theme::track()`）+ 滑块（`theme::text_disabled()`），悬停 / 拖拽高亮主色（`theme::accent()`）；内容未溢出（`max <= 0`）自动隐藏。
- **交互**：滑块拖拽（按住后允许移出轨道）、轨道点击跳转、滚轮驱动。
- **关键约定**：
  1. 绑定的 `ScrollHandle` **必须跨帧持久**（放视图字段或 `window.use_keyed_state`）。
  2. 滚动容器必须接 `.track_scroll(&handle)`——二者共享同一 offset，`set_offset` 才能真正驱动滚动，滚轮也会实时联动滑块。
  3. 父容器须 `.relative()`，滚动条用 `.absolute()` 贴边。
- **注意**：Table `fixed_left()` 已内置横向滚动条，无需手动加。

```rust
// 视图字段：scroll_handle: ScrollHandle（构造器 ScrollHandle::new() 初始化）
// render 中：
div().relative().size_full()
    .child(
        div().id("content").size_full().overflow_y_scroll()
            .track_scroll(&self.scroll_handle)
            .child(long_content),
    )
    .child(
        div().absolute().top_0().bottom_0().right_0()
            .child(Scrollbar::vertical("vsb", self.scroll_handle.clone())),
    )
```

---

## 四、导航

### Anchor · 锚点导航

页面内目录跳转。**受控组件**。

- **创建**：`AnchorItem::new(key, title)`（可嵌套子目录）；`Anchor::new(items)`。
- **属性**：左侧指示轨 + 当前项高亮；`active(key)` 受控当前项。
- **回调**：`on_click(&SharedString)`（3 参）——回传被点项的 key。
- **注意**：**滚动定位由调用方处理**（控件只报告点击，不自动滚动到目标）。

```rust
Anchor::new(vec![
    AnchorItem::new("intro", "简介"),
    AnchorItem::new("usage", "用法"),
]).active(self.active_anchor.clone())
  .on_click(cx.listener(|this, key, _, cx| {
      this.active_anchor = key.clone();
      // 自行滚动到对应区域
      cx.notify();
  }))
```

### Breadcrumb · 面包屑

层级路径导航。**受控组件**。

- **创建**：`Breadcrumb::new(items)`（items 为可迭代的文本集合）。
- **回调**：`on_click(&usize)`（3 参）——回传被点层级下标。
- **注意**：**末项为当前页，不可点**；回调只针对中间层级。

```rust
Breadcrumb::new(["首页", "产品", "详情"])
    .on_click(cx.listener(|this, ix, _, cx| { this.navigate(*ix); }))
```

### DropdownMenu · 动作菜单

任意触发元素 + 下拉动作列表。**受控组件**，**需唯一 id**。

- **创建**：`MenuItem::new(label)`；`DropdownMenu::new(id, trigger, items)`。
- **MenuItem 属性**：分隔线、`danger()` 危险项、`disabled()` 禁用项。
- **MenuItem 回调**：`on_click(&mut Window, &mut App)`（**2 参**）。
- **行为**：点动作后自动收起；浮层定位见 [1.4](#14-浮层定位机制共享约定)。

```rust
let entity = cx.entity();
DropdownMenu::new("more", Button::new("更多"), vec![
    MenuItem::new("编辑").on_click(move |_w, cx| { let _ = entity.update(cx, |this, cx| { … }); }),
    MenuItem::new("删除").danger(),
])
```

### NavMenu · 导航菜单

侧栏多级导航。**受控组件**，**需唯一 id**。

- **创建**：`MenuEntry::new(key, label)`（可嵌套子菜单）；`NavMenu::new(id, entries)`。
- **属性**：图标 / 禁用；选中项与展开集合均受控。
- **回调**：`on_select(&SharedString)`（3 参，回传选中 key）；`on_toggle(&SharedString)`（3 参，回传展开/收起的 key）。
- **注意**：`MenuEntry` 用 **key** 标识（不是下标）。

```rust
NavMenu::new("side-nav", entries)
    .selected(self.nav_selected.clone())
    .expanded(self.nav_expanded.clone())
    .on_select(cx.listener(|this, key, _, cx| { this.nav_selected = key.clone(); cx.notify(); }))
    .on_toggle(cx.listener(|this, key, _, cx| { /* 增删 this.nav_expanded */ cx.notify(); }))
```

### Pagination · 分页器

页码导航。**受控组件**，**需唯一 id**。

- **创建**：`Pagination::new(id, current, total)`（**页码从 1 起**，total 为总页数）。
- **属性**：页数多时自动省略号折叠；边界按钮置灰；`simple()` 简洁模式；`small()` 小尺寸。
- **回调**：`on_change(&usize)`（3 参）——回传新页码。

```rust
Pagination::new("pager", self.page, self.total_pages)
    .on_change(cx.listener(|this, p, _, cx| { this.page = *p; cx.notify(); }))
```

### Steps · 步骤条

流程进度展示。**受控组件**（无回调）。

- **创建**：`Steps::new(titles: Vec<SharedString>)`。
- **属性**：完成 ✓ / 当前数字 / 待办灰；`current(usize)` 受控当前步；`vertical()` 纵向；`small()`；`progress_dot()` 点状；`descriptions(vec)` 步骤描述。

```rust
Steps::new(vec!["填写".into(), "审核".into(), "完成".into()])
    .current(self.step)
    .descriptions(vec!["填写信息".into(), "等待审核".into(), "流程结束".into()])
```

### Tabs · 页签

标签页切换。**受控组件**。

- **创建**：`Tabs::new(items, active)`（items 为页签文本集合，active 为当前下标）。
- **属性**：下划线指示器；`disabled_tabs(vec)` 禁用页签；`closable()` 可关闭；`card()` 卡片式；`centered()` 居中。
- **回调**：`on_select(&usize)`（3 参，选中下标）；`on_close(&usize)`（3 参，关闭下标，需 `closable()`）。
- **注意**：内容区由调用方根据 active 自行渲染（Tabs 只负责页签条）。

```rust
Tabs::new(["概览", "明细", "设置"], self.tab)
    .closable()
    .on_select(cx.listener(|this, ix, _, cx| { this.tab = *ix; cx.notify(); }))
    .on_close(cx.listener(|this, ix, _, cx| { /* 移除第 ix 个页签 */ cx.notify(); }))
```

### ContextMenu · 右键菜单

包裹任意内容，右键弹出菜单。**包装器**，**需唯一 id**。

- **创建**：`ContextMenu::new(id: impl Into<ElementId>, items)`，再 `.child(任意内容)`。
- **行为**：右键在光标处弹出 `MenuItem` 菜单（视口内夹取，不超出窗口）。
- **MenuItem 回调**：`on_click`（2 参）。

```rust
ContextMenu::new("ctx", vec![MenuItem::new("复制"), MenuItem::new("粘贴")])
    .child(my_content)
```

### FloatButton · 悬浮按钮

固定在容器右下角的圆形按钮。**受控组件**，**需唯一 id**。

- **创建**：`FloatButton::new(id, label)`。
- **属性**：`bottom(px)` / `right(px)`（默认 24px）；`tooltip(text)`。
- **回调**：`on_click(&mut Window, &mut App)`（**2 参**）。
- **注意**：按钮 absolute 定位，**父容器须 `.relative()`**。

```rust
div().relative().size_full()
    .child(content)
    .child(FloatButton::new("fab", "↑").tooltip("回到顶部")
        .on_click(move |_w, cx| { … }))
```

### CommandPalette · 命令面板

Ctrl+K 唤起的搜索式命令选择器。**Entity**，遮罩 + 居中面板。

- **创建**：`PaletteItem::new(label)`（可带快捷键提示）；`cx.new(|cx| CommandPalette::new(cx, items))`。
- **方法**：`open(window, cx)` / `close(cx)`。
- **回调**：`on_select(&SharedString)`（3 参）——回传选中命令 label。
- **接入步骤**：
  1. 启动时 `command_palette::bind_keys(cx)` 注册 Ctrl+K。
  2. 宿主视图挂 `.on_action(cx.listener(|this, _: &command_palette::Toggle, window, cx| { this.palette.update(cx, |p, cx| p.open(window, cx)); }))`。
  3. 面板实体渲染在**滚动容器之外**（内部 deferred 置顶）。

```rust
// 字段：palette: Entity<CommandPalette>
palette: cx.new(|cx| CommandPalette::new(cx, vec![
    PaletteItem::new("新建文件"),
    PaletteItem::new("打开设置"),
]).on_select(…)),
// render 末尾（滚动容器外）：
.child(self.palette.clone())
```

---

## 五、数据录入

### 输入系 Entity 通用说明

TextInput / TextArea / PasswordInput / SearchInput / NumberInput / TagInput / Autocomplete / Mentions 都是 **Entity（有状态视图）**，内部持有光标 / 选区 / 草稿等。统一范式：

```rust
// 创建一次（cx 是 Context<该控件>）：
field: cx.new(|cx| TextInput::new(cx).placeholder("…")),
// render 挂载：
div().child(self.field.clone())
// 读取：
let s = self.field.read(cx).text();
// 写入（set_text 需要 window）：
self.field.update(cx, |i, cx| i.set_text("…", window, cx));
```

启动时须 `text_input::bind_keys(cx)`（TextArea 另需 `text_area::bind_keys(cx)`）。

### TextInput · 单行输入框

最基础的文本输入。**Entity**。

- **创建**：`cx.new(|cx| TextInput::new(cx))`。
- **属性**：`placeholder(text)`；`bare()` 去边框（供包装）；`masked()` 掩码显示；`disabled()` / `readonly()` / `error()` 状态；`allow_clear()` ✕ 清空；`max_length(n)` 字数上限；`prefix(el)` / `suffix(el)` 前后缀。
- **读写**：`text()` / `set_text(s, window, cx)`。
- **能力**：光标 / 选区 / 剪贴板 / 中文输入法（IME）。
- **注意**：`set_text` 需要 `&mut Window`，在 `entity.update` 闭包里用外层传入的 window。

### TextArea · 多行输入

多行文本编辑。**Entity**。

- **创建**：`cx.new(|cx| TextArea::new(cx))`。
- **属性**：`height(px)` 高度；其余同 TextInput。
- **能力**：硬换行 / 跨行选区 / 剪贴板 / IME / 光标自动滚动。
- **注意**：**额外**需要 `text_area::bind_keys(cx)`（回车 / 上下方向键 / Shift 选择），否则换行与纵向移动无效。

### PasswordInput · 密码输入

掩码 + 明文切换。**Entity**。

- **创建**：`cx.new(|cx| PasswordInput::new(cx))`。
- **能力**：掩码显示 + **自绘眼睛图标**切换明文 / 掩码。
- **读写**：同 TextInput（`text()` 拿到的是真实明文）。

### SearchInput · 搜索输入

带放大镜与清空的搜索框。**Entity**。

- **创建**：`cx.new(|cx| SearchInput::new(cx))`。
- **能力**：自绘放大镜图标 + ✕ 清空。
- **回调**：`on_change(&str)`（3 参，每次输入）；`on_search(&str)`（3 参，**回车触发**）。
- **注意**：搜索动作接 `on_search`，不要只用 `on_change`。

```rust
search: cx.new(|cx| SearchInput::new(cx).on_search(cx.listener(|this, q, _, cx| {
    this.query = q.to_string();
    this.run_search(cx);
}))),
```

### NumberInput · 数值输入

带步进的数值框。**Entity**。

- **创建**：`cx.new(|cx| NumberInput::new(cx))`。
- **属性**：`min(f64)` / `max(f64)` / `step(f64)`。
- **能力**：▲▼ 按钮 / 上下方向键步进；非法输入不回调。
- **回调**：`on_change(Option<f64>, &mut Window, &mut App)`（3 参，**首参按值传递**，不是引用；`None` 表示空 / 非法）。
- **读写**：`value()` / `set_value()`。

```rust
num: cx.new(|cx| NumberInput::new(cx).min(0.).max(100.).step(1.)
    .on_change(cx.listener(|this, v: Option<f64>, _, cx| {
        this.amount = v.unwrap_or(0.);
        cx.notify();
    }))),
```

### Stepper · 数值步进器

− / + 按钮步进。**受控组件**，**需唯一 id**。

- **创建**：`Stepper::new(id, value: f32)`。
- **属性**：`min(f32)` / `max(f32)` / `step(f32)` / `disabled()`；边界自动置灰对应按钮。
- **回调**：`on_change(&f32)`（3 参）。
- **区别**：Stepper 是受控组件（值在父视图），NumberInput 是 Entity（值在控件内）。

```rust
Stepper::new("qty", self.qty).min(0.).max(99.).step(1.)
    .on_change(cx.listener(|this, v, _, cx| { this.qty = *v; cx.notify(); }))
```

### TagInput · 标签输入

回车添加标签。**Entity**。

- **创建**：`cx.new(|cx| TagInput::new(cx))`。
- **属性**：`initial_tags(vec)` 初始标签。
- **能力**：回车添加（自动去重）/ 空草稿退格删末尾 / ✕ 删单个。
- **回调**：`on_change(&Vec<SharedString>)`（3 参）。
- **读写**：`tags()`。

### Autocomplete · 自动完成

输入过滤候选。**Entity**。

- **创建**：`cx.new(|cx| Autocomplete::new(cx, items))`。
- **能力**：输入过滤候选（大小写不敏感），↑↓ 选择 / 回车确认。
- **回调**：`on_select(&SharedString)`（3 参，回传选中候选）。

### Mentions · @ 提及输入

输入 @ 弹候选。**Entity**。

- **创建**：`cx.new(|cx| Mentions::new(cx).options(vec![…]))`。
- **能力**：输入 `@` 后无空白即弹候选浮层，选中替换为「@名称 」。
- **回调**：`on_change(&str)`（3 参）；`on_select(&SharedString)`（3 参，回传选中名称）。
- **读写**：同 TextInput（`text()` / `set_text()`）。

### Select · 下拉选择器

单选下拉。**受控组件**，**需唯一 id**。

- **创建**：`Select::new(id, items, selected: usize)`。
- **属性**：
  - `open(bool)` + `on_open_change(&bool)`：展开状态受控。
  - `value(Option<usize>)`：可空选中（`None` 显示占位）。
  - `allow_clear()`：✕ 清空。
  - `searchable()`：菜单顶部搜索框过滤选项。
  - `menu_width(px)`：覆盖默认菜单宽度（默认等于触发框）。
- **回调**：`on_select(&usize)`（3 参）；`on_open_change(&bool)`（3 参）；`on_clear(&mut Window, &mut App)`（**2 参**）。
- **行为**：菜单贴框下方展开，下方不足自动翻转上方，点击外部收起，单选选中后收起。

```rust
Select::new("city", ["北京", "上海", "广州"], self.city)
    .open(self.city_open)
    .on_open_change(cx.listener(|this, o, _, cx| { this.city_open = *o; cx.notify(); }))
    .on_select(cx.listener(|this, ix, _, cx| { this.city = *ix; this.city_open = false; cx.notify(); }))
```

### MultiSelect · 多选下拉

多选下拉。**受控组件**，**需唯一 id**。

- **创建**：`MultiSelect::new(id, items, selected: Vec<usize>)`。
- **属性**：触发框显示已选芯片（✕ 可单项移除）；菜单带勾选标记；`open()` / `menu_width()` 同 Select。
- **回调**：`on_change(&Vec<usize>)`（3 参，回传全部选中下标）；`on_open_change(&bool)`（3 参）。
- **行为**：选中**不收起**（便于多选）。

```rust
MultiSelect::new("tags", items, self.selected.clone())
    .open(self.ms_open)
    .on_open_change(cx.listener(|this, o, _, cx| { this.ms_open = *o; cx.notify(); }))
    .on_change(cx.listener(|this, sel, _, cx| { this.selected = sel.clone(); cx.notify(); }))
```

### Cascader · 级联选择

多级菜单横排展开。**受控组件**，**需唯一 id**。

- **创建**：`CascaderNode::new(value, label)`（可 `.children(vec)` 嵌套）；`Cascader::new(id, options)`。
- **属性**：路径值受控（`Vec<SharedString>`）；`open()` + `on_open_change()`。
- **回调**：`on_change(&Vec<SharedString>)`（3 参，回传完整路径值）；`on_open_change(&bool)`（3 参）。
- **行为**：点叶子收起。

```rust
let options = vec![
    CascaderNode::new("zj", "浙江").children(vec![
        CascaderNode::new("hz", "杭州"),
        CascaderNode::new("nb", "宁波"),
    ]),
];
Cascader::new("region", options).value(self.region.clone())
    .on_change(cx.listener(|this, path, _, cx| { this.region = path.clone(); cx.notify(); }))
```

### TreeSelect · 树选择

下拉浮层内嵌 Tree。**受控组件**，**需唯一 id**。

- **创建**：`TreeSelect::new(id, nodes)`（复用 `TreeNode`）。
- **属性**：选中 id 与展开集合受控；`open()` + `on_open_change()`。
- **回调**：`on_change(&SharedString)`（3 参，回传选中节点 id）；`on_toggle(&SharedString)`（3 参，展开/收起）；`on_open_change(&bool)`（3 参）。

### Checkbox · 复选框

勾选框。**受控组件**。

- **创建**：`Checkbox::new(label, checked: bool)`。
- **属性**：`indeterminate()` 半选态。
- **回调**：`on_change(&bool)`（3 参）。

```rust
Checkbox::new("同意协议", self.agreed)
    .on_change(cx.listener(|this, v, _, cx| { this.agreed = *v; cx.notify(); }))
```

### Radio · 单选按钮

单选组的一项。**受控组件**。

- **创建**：`Radio::new(label, selected: bool)`。
- **回调**：`on_select(&ClickEvent)`（3 参）。
- **用法**：配合父视图状态组成单选组——每个 Radio 的 `selected` 由 `self.choice == i` 决定，`on_select` 里设 `self.choice = i`。

```rust
div().flex().gap_3()
    .child(Radio::new("男", self.gender == 0).on_select(cx.listener(|this, _, _, cx| { this.gender = 0; cx.notify(); })))
    .child(Radio::new("女", self.gender == 1).on_select(cx.listener(|this, _, _, cx| { this.gender = 1; cx.notify(); })))
```

### Switch · 开关

布尔切换。**受控组件**。

- **创建**：`Switch::new(on: bool)`（可带标签）。
- **属性**：`small()`；`loading()` 加载态；`text(on, off)` 轨道内文字。
- **回调**：`on_change(&bool)`（3 参）。

```rust
Switch::new(self.enabled).text("开", "关")
    .on_change(cx.listener(|this, v, _, cx| { this.enabled = *v; cx.notify(); }))
```

### Slider · 滑杆

点击定位 + 拖拽。**受控组件**，**需唯一 id**。

- **创建**：`Slider::new(id, value: f32)`。
- **属性**：`range(min, max)`；`step(f32)`；`vertical()` 垂直方向；`marks(vec)` 刻度圆点 + 标签。
- **回调**：`on_change(&f32)`（3 参）。
- **注意**：同窗口多实例 id 须唯一，否则拖拽 / hover 串扰。

```rust
Slider::new("volume", self.volume).range(0., 100.).step(1.)
    .on_change(cx.listener(|this, v, _, cx| { this.volume = *v; cx.notify(); }))
```

### Rating · 星级评分

星数选择。**受控组件**，**需唯一 id**。

- **创建**：`Rating::new(id, value: usize)`。
- **属性**：`max(n)` 星数上限；`readonly()` 只读。
- **回调**：`on_change(&usize)`（3 参）。
- **行为**：点击当前星数可清零。

### DatePicker · 日期选择器

日历浮层选日期。**受控组件**，**需唯一 id**。

- **创建**：`DatePicker::new(id, value: Option<Date>)`。
- **属性**：`month(Option<(i32, u32)>)` 显示月份受控；`disabled()`；`allow_clear()`；`open()` + `on_open_change()`。
- **回调**：
  - `on_change(&Option<Date>)`（3 参）。
  - `on_month_change(&(i32, u32))`（3 参）。
  - `on_open_change(&bool)`（3 参）。
  - `on_clear(&mut Window, &mut App)`（**2 参**）。
- **注意**：不传 `month` 时默认显示选中值所在月（无值则当前月）；**不接 `on_month_change` 时月导航不生效**。

```rust
DatePicker::new("date", self.date)
    .month(self.dp_month)
    .open(self.dp_open)
    .on_open_change(cx.listener(|this, o, _, cx| { this.dp_open = *o; cx.notify(); }))
    .on_month_change(cx.listener(|this, m, _, cx| { this.dp_month = Some(*m); cx.notify(); }))
    .on_change(cx.listener(|this, d, _, cx| { this.date = *d; this.dp_open = false; cx.notify(); }))
```

### DateRangePicker · 日期范围选择器

选起止日期。**受控组件**，**需唯一 id**。

- **创建**：`DateRangePicker::new(id, value: Option<(Date, Date)>)`。
- **属性**：值 / 展开 / 月份均受控；`allow_clear()`；范围内日期高亮。
- **回调**：`on_change(&Option<(Date, Date)>)`（3 参）；`on_month_change(&(i32, u32))`；`on_open_change(&bool)`；`on_clear`（2 参）。
- **行为**：先点起点再点终点（终点早于起点则重选起点）；待选起点按 id 跨帧持久化，收起 / 清空时自动丢弃；月份默认取起点所在月。

### TimePicker · 时间选择器

时 / 分 / 秒三列滚动菜单。**受控组件**，**需唯一 id**。

- **创建**：`TimePicker::new(id)`。
- **属性**：值 `Option<(u32, u32, u32)>` 受控；`open()` + `on_open_change()`。
- **回调**：`on_change(&(u32, u32, u32))`（3 参）；`on_open_change(&bool)`（3 参）。

### ColorPicker · 颜色选择器

24 色预设色板。**受控组件**，**需唯一 id**。

- **创建**：`ColorPicker::new(id, value: Rgba)`。
- **能力**：触发框显示色块 + HEX；选中即回调并收起。
- **回调**：`on_change(&Rgba)`（3 参）；`on_open_change(&bool)`（3 参）。

```rust
ColorPicker::new("brand", self.brand_color)
    .on_change(cx.listener(|this, c, _, cx| { this.brand_color = *c; cx.notify(); }))
```

### Transfer · 穿梭框

左右双面板勾选移动。**受控组件**，**需唯一 id**。

- **创建**：`TransferItem::new(key, label)`；`Transfer::new(id, items)`。
- **属性**：`target_keys(vec)` 右栏集合受控；`selected(vec)` 当前勾选集合受控；`titles(left, right)` 自定义标题。
- **回调**：`on_change(&Vec<SharedString>)`（3 参，回写右栏）；`on_select(&Vec<SharedString>)`（3 参，回写勾选）。
- **注意**：**双集合都要受控回写**，否则移动 / 勾选不生效。

```rust
Transfer::new("perm", items)
    .target_keys(self.target.clone())
    .selected(self.checked.clone())
    .on_change(cx.listener(|this, keys, _, cx| { this.target = keys.clone(); cx.notify(); }))
    .on_select(cx.listener(|this, keys, _, cx| { this.checked = keys.clone(); cx.notify(); }))
```

### Upload · 上传触发区

虚线框点击触发上传。**受控组件**。

- **创建**：`Upload::new()`（默认 id `"upload-area"`）。
- **属性**：`text(t)` / `hint(t)` 文案；`disabled()`。
- **回调**：`on_click(&mut Window, &mut App)`（**2 参**）。
- **注意**：**仅点击触发**，文件对话框 / 上传逻辑由调用方实现，不内置原生拖放。

### UploadList · 上传文件列表

展示上传中 / 完成 / 失败的文件。**受控组件**，**需唯一 id**。

- **创建**：`UploadFile::new(name)`（可设状态 Done / Uploading / Error 与进度）；`UploadList::new(id, files)`。
- **回调**：`on_remove(&usize)`（3 参，回传被移除下标）。

### FilePicker · 文件选择按钮

系统原生文件对话框。**受控组件**。

- **创建**：`FilePicker::new(label)`。
- **属性**：`multiple()` 多选；`directories()` 选目录；`prompt(text)` 对话框标题。
- **回调**：`on_pick(&Vec<PathBuf>, &mut App)`（**特例**：异步任务回传，**无 `&mut Window`**）。
- **注意**：用 `cx.entity()` 句柄构造；取消对话框不触发回调。

```rust
let entity = cx.entity();
FilePicker::new("选择文件").multiple().on_pick(move |paths, cx| {
    let paths = paths.clone();
    let _ = entity.update(cx, |this, cx| { this.files = paths; cx.notify(); });
})
```

### Form · 表单布局容器

排列 FormItem + 受控校验 helper。**受控组件**。

- **创建**：`Form::new()`。
- **属性**：默认纵向排列；`inline()` 横向换行；`gap(px)` 项间距。
- **校验**：配套 `Rule` + `Form::validate()`：
  - `Rule::required(message)`：非空（trim 后）。
  - `Rule::min_len(n, message)` / `Rule::max_len(n, message)`：字符数下限 / 上限。
  - `Rule::custom(message, |v| bool)`：自定义断言。
  - `Form::validate(value: &str, rules: &[Rule]) -> Option<SharedString>`：返回首条未通过规则的错误消息。
- **注意**：控件本身**不持有校验状态**，提交时机由调用方决定；把 `validate` 结果存入视图状态再交给 `FormItem::error_opt()` 显示。

```rust
let err = Form::validate(&name, &[
    Rule::required("请输入姓名"),
    Rule::min_len(2, "至少 2 个字符"),
]);
self.name_error = err;   // Option<SharedString>
```

### FormItem · 表单项包装器

标签 + 错误提示 + 说明。**包装器**。

- **创建**：`FormItem::new(label)`，再 `.child(输入控件)`。
- **属性**：`required()` 红星；`error(text)` / `error_opt(Option<SharedString>)` 红框错误提示；`help(text)` 说明文字；`label_width(px)` 横向布局（标签左置）。

```rust
FormItem::new("姓名").required()
    .error_opt(self.name_error.clone())
    .help("请填写真实姓名")
    .child(self.name_input.clone())
```

---

## 六、数据展示

### Table · 数据表格

功能最全的展示控件。**受控组件**，**需唯一 id**。

- **创建**：`Table::new(id, headers, rows)`，`rows: Vec<Vec<SharedString>>`。
- **属性**：
  - `column_widths(vec)`：列宽（默认均分）。
  - `striped()`：斑马纹；`bordered()`：全边框；`show_header(bool)`：表头显隐。
  - 行选中受控 + `on_row_click()`。
  - `sortable()` + `sort(col, asc)` + `on_sort()`：表头排序。
  - `selectable()` + `selected_rows(vec)` + `on_selected_rows()`：多选。
  - `pagination(page, page_size)` + `on_page_change()`：内置分页。
  - `row_detail(fn)` + `expanded_rows(vec)` + `on_expand()`：展开行。
  - `fixed_left()`：固定首列 + 横向滚动（**内置横向 Scrollbar**）。
  - 空数据自动显示空态。
- **回调**（均 3 参，**一律用全局行下标**）：
  - `on_row_click(&usize)`
  - `on_sort(&(usize, bool))`：列下标 + 是否升序。
  - `on_selected_rows(&Vec<usize>)`
  - `on_page_change(&usize)`：新页码（从 1 起）。
  - `on_expand(&usize)`
- **注意**：
  - 分页受控（页码从 1 起）；表头全选**只作用当前页**。
  - `fixed_left()` 要求统一行高（37px），未指定列宽的列默认 120px。
  - `fixed_left()` 与 `row_detail()` **互斥**——同时设置时展开行不生效。

```rust
Table::new("users", vec!["姓名".into(), "电话".into(), "状态".into()], rows)
    .column_widths(vec![px(120.), px(160.), px(100.)])
    .striped().selectable().sortable()
    .selected_rows(self.selected.clone())
    .pagination(self.page, 10)
    .fixed_left()
    .on_selected_rows(cx.listener(|this, sel, _, cx| { this.selected = sel.clone(); cx.notify(); }))
    .on_page_change(cx.listener(|this, p, _, cx| { this.page = *p; cx.notify(); }))
    .on_sort(cx.listener(|this, (col, asc), _, cx| { this.sort_by(*col, *asc); cx.notify(); }))
```

### Tree · 树形控件

嵌套节点 + 勾选 + 拖拽。**受控组件**，**需唯一 id**。

- **创建**：`TreeNode::new(id: impl Into<SharedString>, label)`（可 `.children(vec)` 嵌套，`Clone`）；`Tree::new(id, nodes)`。
- **属性**：
  - 展开集合（`Vec<SharedString>`）/ 选中 id（`Option<SharedString>`）受控。
  - `checkable()` + `checked(vec)` + `on_check()`：勾选父子联动。
  - `draggable()` + `on_drop_node()`：拖拽移动。
- **回调**（均 3 参）：
  - `on_toggle(&SharedString)`：展开 / 收起节点 id。
  - `on_select(&SharedString)`：选中节点 id。
  - `on_check(&Vec<SharedString>)`：勾选集合。
  - `on_drop_node(&TreeDrop)`：`TreeDrop { dragged, target }`。
- **注意**：
  - 节点 id 同一棵树内唯一即可。
  - 勾选联动**已内置**（勾选含全部子孙，祖先按子全选自动同步）。
  - 拖拽**只报告** `TreeDrop`（目标为自身 / 自身子孙时不触发）；节点树重组由调用方用 `take_node()` / `find_node_mut()` / `push_child()` 完成。

```rust
Tree::new("org", nodes)
    .expanded(self.expanded.clone())
    .selected(self.selected.clone())
    .checkable().checked(self.checked.clone())
    .on_toggle(cx.listener(|this, id, _, cx| { /* 增删 this.expanded */ cx.notify(); }))
    .on_select(cx.listener(|this, id, _, cx| { this.selected = Some(id.clone()); cx.notify(); }))
    .on_check(cx.listener(|this, ids, _, cx| { this.checked = ids.clone(); cx.notify(); }))
```

### List · 列表

垂直列表。**受控组件**（无回调）。

- **创建**：`ListItem::new(title)`（可设描述 / 头像 / `extra()` 右侧元素）；`List::new(items)`。
- **属性**：`header(el)` / `footer(el)`。

```rust
List::new(vec![
    ListItem::new("条目一").description("说明文字"),
    ListItem::new("条目二").extra(Button::new("操作")),
]).header(Text::new("列表标题").strong())
```

### Card · 卡片

带标题的内容容器。**受控组件**（无回调）。

- **创建**：`Card::new(title)`。
- **属性**：`extra(el)` 右上操作；`cover(el)` 封面；`action(el)` 底部操作区；`bordered()`；`hoverable()` 悬停浮起阴影。

```rust
Card::new("卡片标题").hoverable()
    .extra(Link::new("更多").href("…"))
    .child(body_content)
    .action(Button::new("确定").primary())
```

### Descriptions · 描述列表

键值对展示。**受控组件**（无回调）。

- **创建**：`Descriptions::new(title)`。
- **属性**：`item(label, value)` 键值对；`column(n)` 每行列数；`bordered()` 表格样式；`vertical()` 垂直布局（标签上值下）。

```rust
Descriptions::new("用户信息").bordered().column(2)
    .item("姓名", "张三")
    .item("电话", "138-0000-0000")
```

### Statistic · 统计数值

大数字展示。**受控组件**（无回调）。

- **创建**：`Statistic::new(title, value)`。
- **属性**：`prefix(el)` / `suffix(el)`；`trend(up)` 涨跌箭头。

### Countdown · 倒计时

天/时/分/秒单位块。**Entity**。

- **创建**：`cx.new(|cx| Countdown::new(unix_secs: u64, cx))` 或 `Countdown::after(Duration, cx)`。
- **能力**：每秒按系统时钟重算（防漂移）；归零触发一次 `on_finish` 后停止计时。
- **回调**：`on_finish(&mut App)`（**1 参**，定时器上下文无 Window）。
- **读写**：`remaining()` / `finished()`。
- **注意**：重新开始 = 重建实体；旧实体释放后计时循环自动退出，无泄漏。

```rust
let entity = cx.entity();
countdown: cx.new(|cx| {
    Countdown::after(Duration::from_secs(90), cx).on_finish(move |cx| {
        let _ = entity.update(cx, |this, cx| { /* 弹 Toast */ });
    })
}),
// render：
div().child(self.countdown.clone())
```

### Calendar · 日历

月视图 / 年视图。**受控组件**，**需唯一 id**，**全受控**。

- **创建**：`Calendar::new(id, month: (i32, u32))`。
- **属性**：`value(Option<Date>)` 选中日；`month((年, 月))` 显示月；`mode(CalendarMode)` 月 / 年视图。
- **回调**（均 3 参）：`on_month_change(&(i32, u32))`；`on_select(&Date)`；`on_mode_change(&CalendarMode)`。
- **注意**：**三个回调都要接**，否则对应操作不生效；年视图点击迷你月历跳回月视图。

```rust
Calendar::new("cal", self.cal_month)
    .value(self.cal_date).mode(self.cal_mode)
    .on_month_change(cx.listener(|this, m, _, cx| { this.cal_month = *m; cx.notify(); }))
    .on_select(cx.listener(|this, d, _, cx| { this.cal_date = Some(*d); cx.notify(); }))
    .on_mode_change(cx.listener(|this, mode, _, cx| { this.cal_mode = *mode; cx.notify(); }))
```

### Timeline · 时间线

时间节点列表。**受控组件**（无回调）。

- **创建**：`TimelineItem::new(title)`（可设描述 / 时间 / 节点色）；`Timeline::new(items)`。
- **属性**：`alternate()` 交替左右；`pending()` 幽灵节点。

### Tag · 标签

紧凑标记。**受控组件**。

- **创建**：`Tag::new(label)`。
- **属性**：`color(TagColor)` 8 种颜色预设；`bordered()` 边框开关。
- **回调**：`on_close(&ClickEvent)`（3 参）——设置后显示 ✕ 移除按钮。

```rust
Tag::new("标签").color(TagColor::Blue)
    .on_close(cx.listener(|this, _, _, cx| { /* 移除该标签 */ cx.notify(); }))
```

### Badge · 徽章

状态标记。**受控组件**（无回调）。

- **创建**：`Badge::new(label)`。
- **属性**：`info()` / `success()` / `warning()` / `danger()`；`color(c)` 自定义色。

### CountBadge · 数字角标

包裹子元素右上角计数。**包装器**。

- **创建**：`CountBadge::new(count: u32)`，再 `.child(子元素)`。
- **属性**：`max(n)` 溢出显示 99+；`dot()` 红点模式；`show_zero()`；`color(c)` 自定义底色；`offset(x, y)` 位置偏移。

```rust
CountBadge::new(5).max(99).child(Avatar::new("张三"))
```

### Avatar · 头像

名字首字符头像。**受控组件**（无回调）。

- **创建**：`Avatar::new(name)`。
- **能力**：圆形背景 + 名字首字符，背景色由名字哈希确定。
- **属性**：`size(px)`；`square()` 圆角方形。

### ImageView · 图片

封装 gpui `img`。**受控组件**（无回调）。

- **创建**：
  - `ImageView::new(source)`：走 GPUI 资产系统（Embedded），**不是磁盘路径**。
  - `ImageView::from_file(path)`：本地文件。
  - `ImageView::from_render(frame)`：内存帧。
- **属性**：`fit_cover()` / `fit_contain()` / `fit_fill()`；`rounded()`；`grayscale()`；尺寸 / 布局通过 `Styled` 转发（`.w()` / `.h()` / `.flex()`）。
- **能力**：内置 loading 与加载失败占位，可用 `.loading(|| …)` / `.fallback(|| …)` 自定义。
- **注意**：本地文件**一律用 `from_file`**，`new("字符串")` 不是磁盘路径。

```rust
ImageView::from_file("d:/assets/photo.png").w(px(200.)).h(px(150.)).fit_cover().rounded()
```

### VideoPlayer · 视频播放器

**仅 Windows**。**Entity**。

- **创建**：`cx.new(|cx| VideoPlayer::new(cx, path))`，创建后自动播放。
- **能力**：Media Foundation 后台解码；内置播放/暂停按钮 + 进度拖拽条。
- **方法**：`play(cx)` / `pause(cx)` / `toggle(cx)` / `seek(secs)`。
- **读写**：`position()` / `duration()`（秒）/ `is_playing()`。
- **注意**：解码在后台线程，帧经通道泵回 UI 线程；播完自动停止，再 `play()` 从头重播；非 Windows 平台该模块不编译。

```rust
video: cx.new(|cx| VideoPlayer::new(cx, "examples/assets/sample.avi")),
// render：
div().child(self.video.clone())
// 控制：
self.video.update(cx, |v, cx| v.pause(cx));
```

### Carousel · 轮播图

图片循环切换。**受控组件**，**需唯一 id**。

- **创建**：`Carousel::new(id: impl Into<ElementId>, sources)`（`ImageSource` 列表）。
- **能力**：‹ › 循环切换 + 底部圆点直达 + 计数。
- **属性**：`current(usize)` 受控。
- **回调**：`on_change(&usize)`（3 参）。

### BarChart · 柱状图

自绘柱状图。**受控组件**（无回调）。

- **创建**：`BarDatum::new(label, value: f32)`（可设单柱颜色）；`BarChart::new(data)`。
- **属性**：`height(px)`；`max(f32)` 数值上限。

### LineChart · 折线图

自绘路径折线。**受控组件**（无回调）。

- **创建**：`LineChart::new(values: Vec<f32>)`。
- **属性**：网格；`area()` 面积；`dots()` 圆点；`labels(vec)` X 轴标签。

### Sparkline · 迷你趋势图

无坐标轴小尺寸折线。**受控组件**（无回调）。

- **创建**：`Sparkline::new(values: Vec<f32>)`。
- **属性**：`w(px)` / `h(px)` / `area()`。
- **用途**：适合嵌入表格 / 卡片。

### PieChart · 饼图 / 环形图

GPU 路径自绘扇区 + 百分比标签 + 图例。**受控组件**（无回调）。

- **创建**：`PieDatum::new(label, value: f32)`（可 `.color()` 单扇区色）；`PieChart::new(data: Vec<PieDatum>)`。
- **属性**：`size(px)` 直径；`donut(true)` 环图（内半径 0.6）/ `inner_ratio(0~0.85)`；`legend(bool)` 右侧图例；`labels(bool)` 扇区百分比。
- **取色**：默认主题分类色板 `theme::chart_color(i)`。
- **注意**：环图中心显示总计；value 负值按 0 处理。

```rust
PieChart::new(vec![
    PieDatum::new("直接访问", 335.),
    PieDatum::new("搜索引擎", 548.),
]).donut(true).size(px(160.))
```

### GaugeChart · 仪表盘

270° 圆弧（缺口朝下）+ 指针 + 中心数值。**受控组件**（无回调）。

- **创建**：`GaugeChart::new(value: f32)`。
- **属性**：`max(f32)` 量程上限（默认 100）；`size(px)`；`thickness(px)` 弧粗；`title(text)` 数值下说明；`label(text)` 中心文字（默认当前值）；`color()` 单色；`zones(vec![(比例上限, Hsla)])` 阈值色区（升序，命中第一个上限 >= 当前比例）。
- **取色**：无色区且无 `color` 时用 `theme::accent()`。

```rust
GaugeChart::new(72.).max(100.).title("CPU 使用率")
    .zones(vec![(0.6, theme::success()), (0.85, theme::warning()), (1.0, theme::danger())])
```

### ScatterChart · 散点图

多系列散点 + 网格 + 坐标轴 + 图例。**受控组件**（无回调）。

- **创建**：`ScatterPoint::new(x: f32, y: f32)`；`ScatterSeries::new(name, points)`（可 `.color()`）；`ScatterChart::new(series: Vec<ScatterSeries>)`。
- **属性**：`height(px)`；`x_max(f32)` / `y_max(f32)`（默认取数据最大值留 5% 余量）；`dot_radius(px)`；`legend(bool)`。
- **注意**：假设坐标非负、原点 0。

### Heatmap · 热力图

矩阵色块 + 行列标签 + 色阶图例，颜色按数值在两端色间插值。**受控组件**（无回调）。

- **创建**：`Heatmap::new(values: Vec<Vec<f32>>)`（行优先二维矩阵）。
- **属性**：`row_labels(vec)` 左侧 / `col_labels(vec)` 顶部；`cell_size(px)` 格边长；`gap(px)`；`colors(low, high)` 色阶两端（默认 `accent_soft` → `accent`）；`legend(bool)` 底部色阶。
- **实现**：div 渲染圆角格（非路径），`lerp_color` 插值。

### RadarChart · 雷达图

多轴网格环 + 轴线 + 多系列半透明填充。**受控组件**（无回调）。

- **创建**：`RadarSeries::new(name, values: Vec<f32>)`（可 `.color()`）；`RadarChart::new(axes: Vec<SharedString>, series: Vec<RadarSeries>)`。
- **属性**：`max(f32)` 数值上限；`size(px)`；`levels(usize)` 网格层数；`legend(bool)`。
- **注意**：每个系列的 `values` 与 `axes` 一一对应。

### FunnelChart · 漏斗图

居中梯形堆叠 + 阶段标签 + 转化率。**受控组件**（无回调）。

- **创建**：`FunnelStage::new(label, value: f32)`（可 `.color()`）；`FunnelChart::new(stages: Vec<FunnelStage>)`。
- **属性**：`height(px)`；`gap(px)` 阶段间距。
- **能力**：梯形宽度按 `value/max` 比例，自动标注相对上一阶段转化率（右侧）。

```rust
FunnelChart::new(vec![
    FunnelStage::new("访问", 1000.),
    FunnelStage::new("注册", 620.),
    FunnelStage::new("付费", 280.),
]).height(px(220.))
```

### QRCode · 二维码

纯 Rust 编码自绘渲染。**受控组件**（无回调）。

- **创建**：`QRCode::new(data: impl AsRef<str>)`。
- **属性**：`size(px)` 像素尺寸。
- **能力**：byte 模式 / ECC L / version 1~6，无外部依赖。
- **注意**：最多 **134 字节**（UTF-8 中文约 44 字），超长数据会被截断；适合短文本 / URL，不支持 logo 与纠错级别切换。

### CodeBlock · 代码块

等宽字体代码展示。**受控组件**（无回调）。

- **创建**：`CodeBlock::new(code)`。
- **能力**：逐行渲染 + 语言标签 + 一键复制按钮。

### Empty · 空状态

无数据占位。**受控组件**（无回调）。

- **创建**：`Empty::new(title)`。
- **属性**：`description(text)`；`action(el)` 操作区。

### Segmented · 分段选择器

圆角容器内互斥选项。**受控组件**。

- **创建**：`Segmented::new(items, active)`。
- **属性**：`small()` / `large()`；`block()` 占满整行。
- **回调**：`on_select(&usize)`（3 参）。
- **用途**：适合 2~5 个短选项。

```rust
Segmented::new(["日", "周", "月"], self.period)
    .on_select(cx.listener(|this, ix, _, cx| { this.period = *ix; cx.notify(); }))
```

### Accordion · 折叠面板组

多面板可同时展开。**受控组件**，**需唯一 id**。

- **创建**：`AccordionItem::new(title, content)`；`Accordion::new(id, items)`。
- **属性**：展开集合（`Vec<usize>`）受控。
- **回调**：`on_change(&Vec<usize>)`（3 参，回传展开下标集合）。

```rust
Accordion::new("faq", vec![
    AccordionItem::new("问题一", answer_one),
    AccordionItem::new("问题二", answer_two),
]).expanded(self.expanded.clone())
  .on_change(cx.listener(|this, ids, _, cx| { this.expanded = ids.clone(); cx.notify(); }))
```

### Tooltip · 工具提示

悬停气泡。**包装器**（无回调）。

- **创建**：`Tooltip::new(text, child)`。
- **能力**：悬停任意控件显示气泡，延迟与定位由 GPUI 原生处理。

```rust
Tooltip::new("这是提示", Button::new("悬停我"))
```

### Popover · 气泡卡片

可交互内容浮层。**包装器 / 受控**，**需唯一 id**。

- **创建**：`Popover::new(id, trigger, content)`。
- **属性**：展开受控（`open()` + `on_open_change()`）。
- **回调**：`on_open_change(&bool)`（3 参）。
- **区别**：比 Tooltip 更重，内容可交互。

### Tour · 分步引导

遮罩 + 居中卡片 + 步骤圆点。**受控组件**。

- **创建**：`TourStep::new(title, desc)`；`Tour::new(steps)`。
- **属性**：`current(usize)` 受控；跳过 / 上一步 / 下一步完成。
- **回调**：`on_change(&usize)`（3 参，步骤切换）；`on_close(&mut Window, &mut App)`（**2 参**，关闭引导）。
- **注意**：渲染在滚动容器之外。

---

## 七、反馈

### Alert · 提示条

页内静态提示。**受控组件**。

- **创建**：`Alert::new(title)`。
- **属性**：`info()` / `success()` / `warning()` / `danger()`；`description(text)`；`banner()` 通栏模式（无边框圆角）。
- **回调**：`on_close(&mut Window, &mut App)`（**2 参**）——设置后显示 ✕。

```rust
Alert::new("操作成功").success().description("数据已保存。")
```

### Modal · 模态对话框

遮罩 + 居中卡片。**受控组件**。

- **创建**：`Modal::new(title)`。
- **属性**：`width(px)` 卡片宽度；`closable()` ✕ 关闭；`mask_closable()` 点遮罩关闭；`footer(el)` 操作区；`open(bool)` 显隐受控。
- **回调**：`on_close(&mut Window, &mut App)`（**2 参**）。
- **注意**：**须在滚动容器之外渲染**（放根视图最后一个子元素）。

```rust
let entity = cx.entity();
// render 末尾（滚动容器外）：
if self.modal_open {
    root = root.child(
        Modal::new("确认").width(px(420.))
            .footer(Button::new("取消").on_click(/* … */))
            .on_close(move |_w, cx| { let _ = entity.update(cx, |this, cx| { this.modal_open = false; cx.notify(); }); })
            .child(body),
    );
}
```

### Drawer · 抽屉

遮罩 + 侧滑面板。**受控组件**。

- **创建**：`Drawer::new(title)`。
- **属性**：`placement(DrawerPlacement)` 左 / 右（默认右）；`width(px)`；`footer(el)`；`closable()` / `mask_closable()`；`open(bool)`。
- **回调**：`on_close(&mut Window, &mut App)`（**2 参**）。
- **注意**：**须在滚动容器之外渲染**。

### ToastStack · 轻提示堆栈

右上角浮层轻提示。**受控组件**。

- **创建**：`ToastItem::new(message)`（四种类型 info/success/warning/error）；`ToastStack::new(items)`。
- **属性**：条目受控（`Vec<ToastItem>`）。
- **回调**：`on_close(&usize)`（3 参，回传被移除条目下标）。
- **注意**：渲染在滚动容器之外；自动消失 / 手动移除由调用方管理条目集合。

### NotificationStack · 通知堆栈

右上角浮层通知。**受控组件**。

- **创建**：`NotificationItem::new(title)`（可设描述 / 类型 / `action()` 按钮）；`NotificationStack::new(items)`。
- **属性**：条目受控。
- **回调**：`on_close(&usize)`（3 参）。
- **注意**：渲染在滚动容器之外。

### Popconfirm · 气泡确认框

包裹触发元素的轻量确认。**包装器 / 受控**，**需唯一 id**。

- **创建**：`Popconfirm::new(id, title)`，再 `.child(触发元素)`。
- **属性**：`description(text)`；`ok_text(t)` / `cancel_text(t)`；`open()` + `on_open_change()`。
- **回调**：`on_confirm(&mut Window, &mut App)`（**2 参**）；`on_cancel(&mut Window, &mut App)`（**2 参**）；`on_open_change(&bool)`（3 参）。

```rust
let entity = cx.entity();
Popconfirm::new("del-confirm", "确定删除？").description("此操作不可恢复")
    .on_confirm(move |_w, cx| { let _ = entity.update(cx, |this, cx| { this.delete(); cx.notify(); }); })
    .child(Button::new("删除").danger())
```

### ProgressBar · 进度条

线性进度。**受控组件**（无回调）。

- **创建**：`ProgressBar::new(value: f32)`（**0.0 ~ 1.0**）。
- **属性**：`steps(n)` 分段进度。

### RingProgress · 环形进度

自绘圆弧进度。**受控组件**（无回调）。

- **创建**：`RingProgress::new(value: f32)`（0.0 ~ 1.0）。
- **属性**：`size(px)`；`thickness(px)`；`color(c)`；`label(text)` 中心文字。

### Result · 结果页

操作结果整页反馈。**受控组件**（无回调）。

- **创建**：`Result::new(status: ResultStatus, title)`（四种状态图标 success/error/info/warning）。
- **属性**：`description(text)`；`action(el)` 操作区。

### Skeleton · 骨架屏

加载占位呼吸动画。**受控组件**（无回调）。

- **创建**：`Skeleton::new()`。
- **属性**：`circle()` 圆形占位；`lines(n)` 行数；`w_percent(f32)` 宽度百分比。

### Spinner · 加载指示器

旋转淡出圆点环。**受控组件**（无回调）。

- **创建**：`Spinner::new()`。
- **属性**：`size(px)`；`color(c)`；`tip(text)` 加载提示文字。

### Watermark · 水印

文字平铺铺满内容区。**包装器**（无回调）。

- **创建**：`Watermark::new(text)`，再 `.child(内容)`。
- **属性**：`font_size(px)`；`color(c)`；`gap(px)`。
- **能力**：隔行错位平铺；**不拦截子控件交互**。

```rust
Watermark::new("内部资料").font_size(px(14.)).gap(px(80.))
    .child(confidential_content)
```

---

## 八、辅助与主题

### misc · 布局辅助

一组轻量布局 helper（非控件，是函数 / 简单结构）。

- `card()`：快速卡片容器。
- `Divider`：分隔线。
- `section_title()`：分区标题。
- `caption()`：次要说明文字。
- `field_row()`：标签 + 内容的横向行。

```rust
div().flex().flex_col().gap_3()
    .child(section_title("基本信息"))
    .child(field_row("姓名", Text::new("张三")))
    .child(Divider)
    .child(caption("以上信息仅供展示"))
```

### theme · 统一主题

全局语义色系统，所有控件颜色来源。

- **21 个语义色**：`bg()` / `surface()` / `accent()` / `text()` / `text_secondary()` / `text_disabled()` / `border()` / `track()` / `success()` / `warning()` / `danger()` / `info()` 等（每个返回 gpui 颜色）。
- **内置两套**：`Theme::light()` / `Theme::dark()`。
- **配置 API**：
  - `theme::set_theme(Theme)`：整体切换明暗。
  - `theme::set_accent(color)`：换品牌主色（悬停 / 按下 / 柔和底**自动派生**）。
  - `theme::update(|t| { … })`：微调单个语义色。
- **注意**：主题全局生效，切换后所有控件下次重绘即用新色；自定义控件也应取 `theme::xxx()` 而非硬编码颜色，才能跟随主题。

```rust
// 启动时设暗色 + 品牌蓝：
theme::set_theme(theme::Theme::dark());
theme::set_accent(gpui::rgb(0x3B82F6));
// 微调成功色：
theme::update(|t| { t.success = gpui::rgb(0x22C55E); });
// render 中取色：
div().bg(theme::bg()).text_color(theme::text())
```

### FpsMeter · 帧率/性能监视器

实测渲染 FPS 与帧耗时的高性能监视器，本身是 `Entity`。

- **创建**：`cx.new(|cx| FpsMeter::new(cx))`，默认开启、标称上限 120fps。
- **属性**：`running(bool)` 开关连续刷新；`max_fps(u32)` 标称上限（默认 120）。
- **方法**：`start(cx)` / `stop(cx)` / `is_running()` / `fps()` / `frame_ms()`。
- **性能模型**：开启后经 `Window::request_animation_frame` 驱动连续渲染，该调用只 notify 当前视图，故高频刷新仅重绘本控件小子树，不带动整页重绘；帧耗时用 60 帧环形缓冲平滑。
- **帧率上限**：实际帧率跟随显示器 vsync（Windows 读 DWM 合成周期），120Hz 屏可达 ~120fps；`max_fps` 仅为标称/配色阈值，不做硬限帧。动画均为时间基（delta-time），高刷新率下平滑。
- **注意**：控件不在渲染树中（如分类切走）时自动停止驱动，无需手动 stop。

```rust
let meter = cx.new(|cx| FpsMeter::new(cx).max_fps(120));
// render：
div().child(meter.clone())
// 控制：
meter.update(cx, |m, cx| m.stop(cx));
```

---

## 九、AI 对话

对标 antd X 的对话套件：会话切换、提示词、消息气泡、发送框、思维链。

### Welcome · 欢迎卡片

- 受控无回调。`Welcome::new(title).icon(emoji).description(text)`。
- 布局：左图标（56px 固定不缩）+ 右标题/描述列（flex_1 + min_w(0)，描述长文本自动换行不溢出）。

### Prompts · 提示词卡片组

- `Prompts::new(vec![PromptItem::new(text).icon(emoji)])`。
- 回调 `on_click(&usize)`（3 参族），回传点击下标。

### Bubble · 消息气泡

- 受控无回调。`Bubble::new(content).avatar(name)`；`right()` 用户侧；`loading(true)` 显示 Spinner；`variant(BubbleVariant::Filled/Outlined/Shadow)`。
- 限宽 max_w(420) + min_w(0) + flex_shrink，长文本换行不撑破父列。

### Sender · 发送框（Entity）

- `cx.new(|cx| Sender::new(cx).on_send(|text, window, cx| …))`；`on_send` 事件 `&SharedString`（3 参族）。
- Enter 发送 / Shift+Enter 换行 / 按钮发送；发送后自动清空。内部用 TextArea `on_submit` 提交模式（action 层），非外层 on_key_down。

### Conversations · 会话列表

- `Conversations::new(vec![ConversationItem::new(key, title).icon(emoji)])`。
- `active(SharedString)` 受控 + `on_select(&SharedString)`（3 参族）。id 为 conversation-{ix}，勿与其它列表撞 id。

### ThoughtChain · 思维链

- `ThoughtChain::new(vec![ThoughtNode::new(title).description(text).status(ThoughtStatus::Success/Pending/…)])`。
- `expanded(Vec<usize>)` 受控 + `on_toggle(&usize)`（3 参族）展开/收起。

---

## 十、高级与宿主模块

这 8 个模块均已通过 `src/lib.rs` 的 `pub mod` / `pub use` 公开；单独收录，**不计入上面的 98 个已文档化控件 / 101 个条目，也不合并写成 106**。

### HotkeyInput · 快捷键录入

- **定位**：点击聚焦后直接按下组合键的录入器；同时产出给人看的显示串和给 GPUI 用的规范绑定串。
- **状态模型**：有状态 `Entity<HotkeyInput>`；内部保存 `combo`、`binding`、修饰键预览、录入态、焦点与外观配置。
- **创建**：`HotkeyInput::new(cx: &mut Context<Self>) -> Self`，再用 `cx.new(|cx| …)` 创建实体。
- **属性**：`id(impl Into<ElementId>)`；`placeholder(impl Into<SharedString>)`；`disabled(bool)`；`variant(InputVariant)`；`status(InputStatus)`；`size(InputSize)`；`require_modifier(bool)`；`allow_clear(bool)`（默认开启）。
- **方法**：`value() -> String`；`binding() -> &str`；`keys() -> &[SharedString]`；`is_empty()`；`is_recording()`；`set_binding(&str, &mut Context<Self>)`；`clear(&mut Context<Self>)`；`start_recording(&mut Window, &mut Context<Self>)`；`stop_recording(&mut Context<Self>)`。
- **回调**：`on_change(Fn(&str, &mut Window, &mut App))`，首参是显示串（如 `"Ctrl+Shift+K"`）；经 `window.defer` 触发。程序化 `set_binding` / `clear` **不触发**该回调。
- **最小示例**：

```rust
use gpui_widgets::HotkeyInput;

let hotkey = cx.new(|cx| {
    let mut input = HotkeyInput::new(cx)
        .id("save-shortcut")
        .placeholder("Click to record a shortcut")
        .on_change(|value, _, _| println!("display: {value}"));
    input.set_binding("ctrl-shift-k", cx);
    input
});
// render: div().child(hotkey.clone())
let binding = hotkey.read(cx).binding().to_owned(); // "ctrl-shift-k"
```

- **注意事项**：`value()` 与 `binding()` 格式不同，注册快捷键应使用后者；`Esc` 取消并保留原值，Backspace / Delete 清空并回调；失焦结束录入。`require_modifier(true)` 要求 Ctrl / Alt / 平台键之一，裸键与仅 Shift 不计。无需 `bind_keys`，同窗多实例应给稳定唯一 id。

### Markdown · Markdown 渲染

- **定位**：把对话 / 文档常用的 GFM 子集直接渲染为 GPUI 元素，支持链接点击和代码块动作。
- **状态模型**：无状态 `RenderOnce`；源文本与配置由宿主每帧重建传入，不持有受控值。
- **创建**：`Markdown::new(source: impl Into<SharedString>) -> Self`。
- **属性**：`compact()` 使用紧凑聊天气泡度量；`text_size(f32)` 覆盖正文并同比缩放标题 / 代码；`code_action(CodeAction)` 可重复调用，为每个围栏代码块追加动作；`code_copy_labels(copy, copied)` 覆盖复制按钮两级文案。
- **回调**：`on_link(Fn(&str, &mut Window, &mut App))` 回传 URL；`CodeAction::new(label, Fn(&str, &mut Window, &mut App))` 的回调回传代码块文本。
- **最小示例**：

```rust
use gpui_widgets::{CodeAction, Markdown};

Markdown::new("# Result\n\n**Done** — [docs](https://example.com)\n\n```rust\nfn main() {}\n```")
    .on_link(|url, _, _| println!("open: {url}"))
    .code_action(CodeAction::new("Insert", |code, _, _| println!("{code}")))
```

- **支持范围**：1~6 级标题、段落、有序 / 无序嵌套列表、任务列表、引用、分隔线、表格列对齐、反引号 / 波浪线围栏代码块，以及粗体 / 斜体 / 删除线 / 行内代码 / 链接 / 裸 URL 自动链接。
- **注意事项**：手写零依赖解析器只覆盖上述子集，不支持图片渲染（`![alt](url)` 只产生可点击的 alt 文本）、原始 HTML、脚注等完整 GFM 语法，其它未支持内容按文本降级；任务列表勾选框仅展示，不提供勾选回调。`compact()` / `text_size()` / `code_copy_labels()` 适合宿主语言与窄面板适配。

### TransferManager · 传输任务管理器

- **定位**：文件上传 / 下载队列的总览进度、单项进度、大小 / 速度 / 剩余时间与状态操作面板。
- **状态模型**：无状态 `RenderOnce`；在 render 时接收 `Vec<TransferTask>` 快照（通常传 `self.tasks.clone()`）用于展示，任务队列与状态机由宿主视图持有。
- **创建**：`TransferManager::new(id: impl Into<ElementId>, tasks: Vec<TransferTask>) -> Self`；`TransferTask::new(id, name, TransferDirection)`，另有 `upload(id, name)` / `download(id, name)` 便捷构造。
- **状态类型**：`TransferDirection::{Upload, Download}`；`TransferStatus::{Queued, Transferring, Paused, Done, Failed, Canceled}`，默认 `Queued`；`TransferStatus::is_active()` 仅对 `Queued | Transferring` 为真。
- **属性 / 任务方法**：面板 `title(…)` / `show_summary(bool)`；任务 builder `status(...)` / `progress(f32)` / `size(u64)` / `speed(f64)` / `error(...)`；getter `id()` / `name()` / `task_status()` / `progress_value()`；驱动方法 `set_status(...)` / `set_progress(...)`。
- **回调**：`on_pause` / `on_resume` / `on_cancel` / `on_retry` / `on_remove` 均为 `Fn(&SharedString, &mut Window, &mut App)`，回传任务 id；`on_clear_done(Fn(&mut Window, &mut App))` 无事件参。
- **最小示例**：

```rust
use gpui_widgets::{TransferManager, TransferStatus, TransferTask};

let tasks = vec![TransferTask::upload("file-1", "main.rs")
    .status(TransferStatus::Transferring)
    .progress(0.4)
    .size(1024)
    .speed(256.0)];

TransferManager::new("transfer-queue", tasks)
    .title("Transfers")
    .on_pause(|id, _, _| println!("pause: {id}"))
```

- **注意事项**：`progress` 自动夹到 `0.0..=1.0`，但组件不会自行改状态或推进任务；回调里更新宿主集合后需重新渲染。总进度按文件大小加权，无大小按 1 计，`Canceled` 不计入，`Done` 恒显示 100%。操作按钮仅在对应状态且注册了回调时出现；「清除已完成」也仅在存在 `Done` 且注册回调时出现。同窗多面板须使用唯一 id。

### Terminal · 终端视图

- **定位**：纯 GPUI 的 VT/ANSI 终端视图，用于 SSH / 本地 shell 等宿主；不创建进程，也不替代 PTY / SSH 通道。
- **状态模型**：有状态 `Entity<Terminal>`；内部包含解析状态机、字符网格、回滚 `history`、固定 `screen`、光标 /画笔、IME 预编辑、选区、搜索与滚动偏移。
- **创建**：`Terminal::new(cx: &mut Context<Self>) -> Self`，再用 `cx.new(|cx| …)`；默认 80×24、14px 字号、18px 行高、1000 行回滚、Bar 光标、关闭闪烁、带边框。
- **公开类型**：`CursorShape::{Block, Bar, Underline}`；`TerminalScrollMetrics { history_lines, visible_rows, viewport_height: Pixels, max_scroll: Pixels }`。
- **属性**：`id(...)`；`cols(usize)` / `rows(usize)`；`font_size(Pixels)` / `line_height(f32)`；`font_family(impl Into<SharedString>)` / `font_fallbacks(Vec<SharedString>)`；`bordered(bool)`；`max_history(usize)`；`cursor_style(CursorShape)` / `cursor_blink(bool)`；`fit_min(cols, rows)`。对应运行期 setter 均以 `cx: &mut Context<Self>` 结尾并 `notify`。
- **回调（均为普通 `Fn`，不是 GPUI 事件包装）**：
  - `on_input(Fn(&[u8], &mut Window, &mut App))`：键盘转成的字节序列。
  - `on_title(Fn(&str, &mut Window, &mut App))`：OSC 0/2 标题变化。
  - `on_fit(Fn(usize, usize, &mut Window, &mut App))`：pane-fit 网格变化后的 `(cols, rows)`；本地先 resize，再回调，**不经 defer**。
  - `on_search_open(Fn(&mut Window, &mut App))`：Ctrl+Shift+F 请求宿主打开搜索 UI。
  - `on_search_results(Fn(usize, Option<usize>, &mut Window, &mut App))`：匹配总数与 0 基当前下标。
  - `on_cwd(Fn(&str, &mut Window, &mut App))`：OSC 7 工作目录。
  - `on_selection_change(Fn(Option<String>, &mut Window, &mut App))`：选区文本，`None` 表示清除。
  - 除 `on_fit` 外，上述回调均经 `window.defer` 触发。
- **方法**：
  - 输出：`write(&[u8], cx)` / `write_str(&str, cx)` / `clear(cx)` / `reset(cx)` / `resize(cols, rows, cx)` / `paste_text(&str, window, cx)`。
  - 读取：`title()` / `cwd()` / `cursor_visible()` / `cursor_row()` / `screen_top()` / `selection_text()`。
  - 搜索：`set_search(&str, cx)` / `search_next(cx)` / `search_prev(cx)` / `search_matches()` / `search_index()`。
  - 选区 / 复制：`select_all(cx)` / `set_selection(anchor, head, cx)` / `copy_selection(&mut App)` / `copy_all(&mut App)`。
  - 缓冲 / 滚动：`buffer_text()` / `buffer_tail(max_lines)` / `scroll_handle()` / `scroll_offset()` / `scroll_metrics()` / `set_scroll_offset(lines, cx)` / `scroll_to_bottom(cx)`。
- **最小示例**：

```rust
use gpui_widgets::Terminal;

let term = cx.new(|cx| Terminal::new(cx)
    .id("shell")
    .cols(100)
    .rows(28)
    .on_input(|bytes, _, _| { /* write bytes to the PTY / SSH channel */ }));
// render: div().child(term.clone())
term.update(cx, |term, cx| term.write_str("hello\r\n", cx));
```

- **注意事项**：键盘使用原始 `on_key_down`，无需 `text_input::bind_keys`；宿主必须把 `on_input` 写入通道，并把通道输出送入 `write`。`write` 会清选区并回到底部；`resize` 会按软 / 硬换行重排并同步光标、选区与搜索。搜索只提供匹配逻辑，搜索面板由宿主实现。`on_fit` 要求父容器为定高 flex 列，终端以 `flex_1` 占剩余空间；`fit_min` 可拦截拖动期间的瞬态小尺寸。粘贴会把 LF / CRLF 归一为 CR，并按远端 mode 2004 加 bracketed-paste 包裹。同窗多终端使用稳定唯一 id。

### CodeEditor · 轻量代码编辑器预设

- **定位**：在 `TextArea` 代码模式之上提供常用语言的轻量逐行着色；不是完整 IDE 编辑器。
- **状态模型**：`CodeEditor` 是无状态单位类型工厂，**本身不是 Entity**；返回的 `TextArea` 才包装为 `Entity<TextArea>`。
- **创建**：`CodeEditor::new(cx: &mut Context<TextArea>, language: CodeLanguage) -> TextArea`；默认启用行号、当前行高亮、等宽字体、4 空格缩进、13px 字号，并安装语言高亮器。
- **语言**：`CodeLanguage::{Plain, Rust, JavaScript, Json, Toml, Python, Shell}`；`label() -> &'static str`；`all() -> &'static [CodeLanguage]`；`from_name(&str) -> Option<CodeLanguage>`（大小写不敏感，支持 `rs` / `js` / `ts` / `jsonc` / `py` / `sh` 等别名）；`highlighter() -> Option<Highlighter>`，`Plain` 返回 `None`。
- **属性 / 方法**：`CodeEditor::set_language(&mut TextArea, CodeLanguage, &mut Context<TextArea>)` 运行期切语言；返回对象继续使用 `TextArea` 的 `height(Pixels)`、`placeholder(...)`、`on_change(...)`、`text()` / `set_text(...)`、`caret()`、滚动与选区 API。
- **回调**：不新增回调；沿用 `TextArea::on_change(Fn(&str, &mut Window, &mut App))` 与 `on_submit(Fn(&mut Window, &mut App))`。
- **最小示例**：

```rust
use gpui_widgets::{CodeEditor, CodeLanguage};

gpui_widgets::text_area::bind_keys(cx); // once at application startup
let editor = cx.new(|cx| CodeEditor::new(cx, CodeLanguage::Rust)
    .height(px(240.))
    .on_change(|text, _, _| println!("{text}")));
// render: div().child(editor.clone())
editor.update(cx, |area, cx| {
    CodeEditor::set_language(area, CodeLanguage::JavaScript, cx);
});
```

- **注意事项**：必须调用 `text_area::bind_keys` 才能使用完整编辑快捷键。着色仅为单行词法近似（关键字 / 类型 / 字面量 / 字符串 / 数字 / 行注释 / 函数名），不做跨行状态推导，也不提供 LSP、折叠、补全或语法诊断。

### CommandBar · 命令输入条

- **定位**：单行命令输入 + 历史回溯 + 片段补全 / 管理，可用四个动作按钮适配终端下沿命令条。
- **状态模型**：有状态 `Entity<CommandBar>`；内部包装 bare `TextInput`，保存旧到新的 `history`、回溯下标 / 草稿、简单片段、`CommandSnippet` 列表、面板与内联表单；焦点委托给输入框。
- **创建**：`CommandBar::new(cx: &mut Context<Self>) -> Self`，再用 `cx.new(|cx| …)`。
- **属性**：
  - 基础：`id(...)` / `prompt(...)` / `placeholder(...)` / `max_history(usize)`（最小 1，默认 100）/ `history_items(IntoIterator<Item = impl Into<String>>)`。
  - 片段：`snippets(IntoIterator<Item = impl Into<SharedString>>)` / `snippet_items(IntoIterator<Item = CommandSnippet>)` / `manage_snippets(bool)` / `dismiss_outside(bool)` / `empty_text(history, snippets)` / `snippet_icon_builder(Fn(SnippetIcon, &str) -> AnyElement)`。
  - 外观：`framed(bool, &mut App)`（默认 `true`）/ `actions([impl Into<SharedString>; 4])`，顺序为片段 / 历史 / 清空 / 执行。
- **片段类型 / helper**：`CommandSnippet { id: usize, name, command, group, send_enter: bool, sort: usize }`；`new(name, command, group)` 默认 `send_enter = true`、`sort = 0`，可用 builder 覆盖。`SnippetIcon::{Edit, Trash}`；`SnippetEvent::{Saved(CommandSnippet), DeleteRequested(CommandSnippet), Warn(SharedString)}`。helper：`sorted_snippets` / `group_snippets` / `panel_groups` / `snippet_can_save`。
- **回调**：
  - `on_submit(Fn(&str, &mut Window, &mut App))`：首参为去首尾空白后的命令，经 defer。
  - `on_new_snippet(Fn(&mut Window, &mut App))`：非管理模式的新建按钮。
  - `on_snippet_event(Fn(SnippetEvent, &mut Window, &mut App))`：事件**按值**传入。
  - `on_history_clear(Fn(&mut App))`：无 Window。
  - 以上均有 `set_on_*` 运行期版本；图标工厂也有 `set_snippet_icon_builder`。
- **方法**：`text(&App)` / `set_text(..., cx)` / `set_placeholder(..., cx)` / `set_actions(..., cx)` / `set_empty_text(..., cx)`；`history()` / `clear_history(cx)` / `set_history(Vec<String>, cx)`；`snippet_count()` / `snippet_list()` / `push_snippet(...)` / `remove_snippet(id)` / `set_snippet_items(Vec<CommandSnippet>, cx)`；`show_snippets(cx)` / `open_snippet_new(cx)` / `open_snippet_edit(id, cx)` / `submit(window, cx)`。
- **最小示例**：

```rust
use gpui_widgets::{CommandBar, CommandSnippet};

let bar = cx.new(|cx| CommandBar::new(cx)
    .id("command-input")
    .prompt(">")
    .snippets(["git status", "cargo build"])
    .snippet_items([CommandSnippet::new("Build", "cargo build", "Rust")
        .send_enter(false)])
    .on_submit(|command, _, _| println!("run: {command}")));
// render: div().child(bar.clone())
```

- **注意事项**：启动需 `text_input::bind_keys`。Enter 去重写入历史、清空输入并回调；↑↓ 回溯并在回到底部时恢复草稿；Tab 补全第一个前缀匹配；Esc 依次收面板、退出回溯、（有动作按钮时）清输入。管理模式会内置新建 / 编辑 / 删除请求，宿主负责持久化、警告提示与删除确认；`send_enter(true)` 的片段点击直接调用 `on_submit`，**不自动入历史或清空当前输入**。`push_snippet` / `remove_snippet` / `open_snippet_*` 不接收 `cx`，调用后由宿主确保 `notify`。

### DropZone · 外部文件拖放区

- **定位**：接收操作系统拖入的文件 / 文件夹，按扩展名分成接受与拒绝两组。
- **状态模型**：无状态 `RenderOnce`；过滤规则与回调随每次 render 传入，不保存文件或上传进度。
- **创建**：`DropZone::new(id: impl Into<ElementId>) -> Self`。
- **属性**：`icon(...)` / `title(...)` / `hint(...)`；`accept<I, S>(I) where I: IntoIterator<Item = S>, S: Into<SharedString>`；`height(f32)`（默认 132）；`disabled(bool)`。扩展名不带点，空列表表示全部接受，匹配不区分大小写。
- **辅助函数**：`is_accepted(&Path, &[SharedString]) -> bool`。
- **回调**：`on_drop(Fn(&[PathBuf], &mut Window, &mut App))` 与 `on_reject(Fn(&[PathBuf], &mut Window, &mut App))`，分别接收通过 / 拒绝的路径切片。
- **最小示例**：

```rust
use gpui_widgets::DropZone;

DropZone::new("source-drop")
    .title("Drop Rust or TOML files")
    .accept(["rs", "toml"])
    .on_drop(|paths, _, _| { /* enqueue accepted paths */ })
    .on_reject(|paths, _, _| { /* show rejected paths */ })
```

- **注意事项**：仅处理 GPUI 从平台 `FileDropEvent` 转换出的 `ExternalPaths`，不自行读取文件。一次拖放会按扩展名 `partition`，混合批次可同时触发两个回调。`can_drop` 要求至少一个路径被接受，因此**全部被拒绝的批次不会触发 `on_reject`**。禁用态不注册拖放处理；同窗多个区域必须使用唯一 id。

### TitleBar · 自绘标题栏

- **定位**：图标 + 标题 + 最小化 / 最大化（还原）/ 关闭按钮的自绘宿主标题栏。
- **状态模型**：无状态 `RenderOnce`；只读取当前 `Window` 状态并注册窗口交互，不持有标题或按钮状态。
- **创建**：`TitleBar::new(title: impl Into<SharedString>) -> Self`；窗口配置 helper：`transparent_titlebar_options(title) -> WindowOptions`。
- **属性**：`id(...)`；`icon(...)` / `icon_element(impl IntoElement)`（元素优先）；`window_icons(Fn(WindowIconKind, SharedString) -> AnyElement)`；`close_hover_bg(Hsla)`；`height(impl Into<Pixels>)`（默认 36）；`button_width(impl Into<Pixels>)`（默认 46）；`show_minimize(bool)` / `show_maximize(bool)` / `show_close(bool)`；`draggable(bool)` / `live(bool)`（默认均 `true`）；`extra(impl IntoElement)` / `extra_trailing(impl IntoElement)` 可重复追加。
- **窗口图标类型**：`WindowIconKind::{Minimize, Maximize, Restore, Close}`；最大化按钮按 `window.is_maximized()` 在 Maximize / Restore 间切换。图标工厂第二参是按钮 group 名，可用于 `group_hover`。
- **回调**：`on_minimize` / `on_maximize` / `on_close` 均为 `Fn(&mut Window, &mut App)`；设置后覆盖该按钮默认行为，不再交给系统。
- **最小示例**：

```rust
use gpui_widgets::{TitleBar, transparent_titlebar_options};

cx.open_window(
    transparent_titlebar_options("My App"),
    |_, cx| cx.new(|cx| MyView::new(cx)),
)?;
// In MyView::render:
TitleBar::new("My App")
    .id("main-titlebar")
    .icon("A")
    .extra_trailing(div().child("Status"))
```

- **注意事项**：必须隐藏系统标题栏，创建窗口时使用 `transparent_titlebar_options`（`appears_transparent: true`）。Windows 下拖拽 / 双击 / 贴靠 / 系统右键菜单走 `WindowControlArea`；其他平台回退到 `Window` 操作。`window_decorations` 与 `is_movable` 应保持默认，否则 Windows 不接管拖拽。`extra` 位于拖拽区但会挖 no-drag 洞；`extra_trailing` 位于拖拽区外。`live(false)` 仅静态展示，不注册命中区域或按钮回调；同窗多标题栏 / 多窗口使用唯一 id。

---

## 附录 A：回调签名速查表

> 族：**3** = `Fn(&E, &mut Window, &mut App)`（用 `cx.listener`）；**2** = `Fn(&mut Window, &mut App)`（用 `cx.entity()` 句柄）；**1** = `Fn(&mut App)`；**特** = 异步特例。

| 控件 | 回调 | 事件类型 E | 族 |
|---|---|---|---|
| Button | `on_click` | `&ClickEvent` | 3 |
| Link | `on_click` | —（无事件参） | 2 |
| Anchor | `on_click` | `&SharedString` | 3 |
| Breadcrumb | `on_click` | `&usize` | 3 |
| MenuItem（DropdownMenu/ContextMenu） | `on_click` | — | 2 |
| NavMenu | `on_select` / `on_toggle` | `&SharedString` | 3 |
| Pagination | `on_change` | `&usize` | 3 |
| Tabs | `on_select` / `on_close` | `&usize` | 3 |
| FloatButton | `on_click` | — | 2 |
| CommandPalette | `on_select` | `&SharedString` | 3 |
| Splitter | `on_change` | `&f32` | 3 |
| SearchInput | `on_change` / `on_search` | `&str` | 3 |
| NumberInput | `on_change` | `Option<f64>`（**按值**） | 3 |
| Stepper | `on_change` | `&f32` | 3 |
| TagInput | `on_change` | `&Vec<SharedString>` | 3 |
| Autocomplete | `on_select` | `&SharedString` | 3 |
| Mentions | `on_change` / `on_select` | `&str` / `&SharedString` | 3 |
| Select | `on_select` / `on_open_change` | `&usize` / `&bool` | 3 |
| Select | `on_clear` | — | 2 |
| MultiSelect | `on_change` / `on_open_change` | `&Vec<usize>` / `&bool` | 3 |
| Cascader | `on_change` / `on_open_change` | `&Vec<SharedString>` / `&bool` | 3 |
| TreeSelect | `on_change` / `on_toggle` / `on_open_change` | `&SharedString` / `&SharedString` / `&bool` | 3 |
| Checkbox | `on_change` | `&bool` | 3 |
| Radio | `on_select` | `&ClickEvent` | 3 |
| Switch | `on_change` | `&bool` | 3 |
| Slider | `on_change` | `&f32` | 3 |
| Rating | `on_change` | `&usize` | 3 |
| DatePicker | `on_change` / `on_month_change` / `on_open_change` | `&Option<Date>` / `&(i32,u32)` / `&bool` | 3 |
| DatePicker | `on_clear` | — | 2 |
| DateRangePicker | `on_change` / `on_month_change` / `on_open_change` | `&Option<(Date,Date)>` / `&(i32,u32)` / `&bool` | 3 |
| DateRangePicker | `on_clear` | — | 2 |
| TimePicker | `on_change` / `on_open_change` | `&(u32,u32,u32)` / `&bool` | 3 |
| ColorPicker | `on_change` / `on_open_change` | `&Rgba` / `&bool` | 3 |
| Transfer | `on_change` / `on_select` | `&Vec<SharedString>` | 3 |
| Upload | `on_click` | — | 2 |
| UploadList | `on_remove` | `&usize` | 3 |
| FilePicker | `on_pick` | `&Vec<PathBuf>`（无 Window） | 特 |
| Table | `on_row_click` / `on_page_change` / `on_expand` | `&usize` | 3 |
| Table | `on_sort` | `&(usize, bool)` | 3 |
| Table | `on_selected_rows` | `&Vec<usize>` | 3 |
| Tree | `on_toggle` / `on_select` | `&SharedString` | 3 |
| Tree | `on_check` | `&Vec<SharedString>` | 3 |
| Tree | `on_drop_node` | `&TreeDrop` | 3 |
| Countdown | `on_finish` | —（无 Window） | 1 |
| Calendar | `on_month_change` / `on_select` / `on_mode_change` | `&(i32,u32)` / `&Date` / `&CalendarMode` | 3 |
| Tag | `on_close` | `&ClickEvent` | 3 |
| Carousel | `on_change` | `&usize` | 3 |
| Segmented | `on_select` | `&usize` | 3 |
| Accordion | `on_change` | `&Vec<usize>` | 3 |
| Popover | `on_open_change` | `&bool` | 3 |
| Tour | `on_change` | `&usize` | 3 |
| Tour | `on_close` | — | 2 |
| Alert | `on_close` | — | 2 |
| Modal / Drawer | `on_close` | — | 2 |
| ToastStack / NotificationStack | `on_close` | `&usize` | 3 |
| Popconfirm | `on_open_change` | `&bool` | 3 |
| Popconfirm | `on_confirm` / `on_cancel` | — | 2 |
| HotkeyInput | `on_change` | `&str` | 3 |
| Markdown | `on_link` | `&str` | 3 |
| Markdown | `code_action(CodeAction)` 的处理函数 | `&str` | 3 |
| TransferManager | `on_pause` / `on_resume` / `on_cancel` / `on_retry` / `on_remove` | `&SharedString` | 3 |
| TransferManager | `on_clear_done` | — | 2 |
| Terminal | `on_input` | `&[u8]` | 3 |
| Terminal | `on_title` / `on_cwd` | `&str` | 3 |
| Terminal | `on_fit` | `(usize, usize)`（按值） | 3 |
| Terminal | `on_search_open` | — | 2 |
| Terminal | `on_search_results` | `(usize, Option<usize>)`（按值） | 3 |
| Terminal | `on_selection_change` | `Option<String>`（按值） | 3 |
| CodeEditor（返回 TextArea） | `on_change` | `&str` | 3 |
| CodeEditor（返回 TextArea） | `on_submit` | — | 2 |
| CommandBar | `on_submit` | `&str` | 3 |
| CommandBar | `on_snippet_event` | `SnippetEvent`（按值） | 3 |
| CommandBar | `on_new_snippet` | — | 2 |
| CommandBar | `on_history_clear` | —（无 Window） | 1 |
| CommandBar | `snippet_icon_builder` | `(SnippetIcon, &str) -> AnyElement` | 工厂 |
| DropZone | `on_drop` / `on_reject` | `&[PathBuf]` | 3 |
| TitleBar | `on_minimize` / `on_maximize` / `on_close` | — | 2 |

## 附录 B：Entity 控件清单

这些控件**用 `cx.new(|cx| T::new(cx))` 创建一次**，render 中 `.clone()` 挂载，通过 `entity.read(cx)` / `entity.update(cx, …)` 读写：

| 控件 | 创建 | 主要读写方法 |
|---|---|---|
| TextInput | `TextInput::new(cx)` | `text()` / `set_text(s, window, cx)` |
| TextArea | `TextArea::new(cx)` | 同 TextInput |
| PasswordInput | `PasswordInput::new(cx)` | 同 TextInput |
| SearchInput | `SearchInput::new(cx)` | 同 TextInput |
| Autocomplete | `Autocomplete::new(cx, items)` | 同 TextInput |
| Mentions | `Mentions::new(cx)` | 同 TextInput |
| NumberInput | `NumberInput::new(cx)` | `value()` / `set_value()` |
| TagInput | `TagInput::new(cx)` | `tags()` |
| CommandPalette | `CommandPalette::new(cx, items)` | `open(window, cx)` / `close(cx)` |
| VideoPlayer（仅 Windows） | `VideoPlayer::new(cx, path)` | `play/pause/toggle/seek`、`position()/duration()/is_playing()` |
| Countdown | `Countdown::new(unix, cx)` / `after(Duration, cx)` | `remaining()` / `finished()` |
| Sender | `Sender::new(cx).on_send(…)` | 内部 TextArea；`on_send` 回传文本 |
| FpsMeter | `FpsMeter::new(cx)` | `fps()` / `frame_ms()` / `start/stop` |
| HotkeyInput | `HotkeyInput::new(cx)` | `value()` / `binding()` / `set_binding` / `clear` / 录入控制 |
| Terminal | `Terminal::new(cx)` | `write` / `resize` / 搜索 / 选区 / 滚动 / 缓冲读取 |
| CommandBar | `CommandBar::new(cx)` | `text` / `set_text` / 历史 / 片段 / `submit` |
| CodeEditor（工厂，返回 TextArea） | `cx.new(\|cx\| CodeEditor::new(cx, language))` | 实体类型为 `Entity<TextArea>`；`CodeEditor::set_language` 与 TextArea API |

> `Markdown` / `TransferManager` / `DropZone` / `TitleBar` 是 `RenderOnce`；`CodeEditor` 本身不是 Entity，只有它返回的 `TextArea` 需要实体化。

## 附录 C：需唯一 id 的控件清单

同一窗口内多实例 id 必须唯一（hover / 拖拽 / keyed-state 按 `ElementId` 判定）：

`Slider` `Splitter` `Scrollbar` `Select` `MultiSelect` `Cascader` `TreeSelect` `DatePicker` `DateRangePicker` `TimePicker` `ColorPicker` `Transfer` `UploadList` `Table` `Tree` `Calendar` `Carousel` `Accordion` `Popover` `Popconfirm` `ContextMenu` `DropdownMenu` `NavMenu` `Pagination` `FloatButton` `Rating` `Stepper` `HotkeyInput` `TransferManager` `Terminal` `CommandBar` `DropZone` `TitleBar`

> `Upload::new()` 默认 id 为 `"upload-area"`，多个 Upload 实例须用 `.id(…)` 区分，否则 hover 串扰。

## 附录 D：常见坑 FAQ

**Q：输入框能打字，但 Ctrl+C/V、全选、方向键无效？**
A：忘记启动时 `text_input::bind_keys(cx)`。TextArea 另需 `text_area::bind_keys(cx)`，CommandPalette 需 `command_palette::bind_keys(cx)`。

**Q：`cx.listener` 闭包报 E0282（类型推断失败）？**
A：给事件参数显式标注类型，如 `cx.listener(|this, _e: &ClickEvent, _, cx| …)`。

**Q：`on_close` / `on_confirm` 用 `cx.listener` 编译失败？**
A：它们是 **2 参族**（无事件参），改用 `cx.entity()` 句柄 + `entity.update(cx, …)`。Countdown.on_finish 是 1 参，FilePicker.on_pick 无 Window，同理。

**Q：多个 Slider / Select 悬停高亮互相串扰？**
A：id 重复了。同窗口每个实例用唯一 id（列表里用下标 / 业务 key 拼）。

**Q：Modal / Drawer 被滚动容器裁切 / 位置错乱？**
A：必须渲染在**滚动容器之外**（根视图最后一个子元素）。ToastStack / NotificationStack / Tour / CommandPalette 同理。

**Q：Scrollbar 滑块不动 / 滚轮不联动？**
A：`ScrollHandle` 没跨帧持久，或滚动容器没接 `.track_scroll(&handle)`。二者必须共享同一 handle（放视图字段或 `window.use_keyed_state`）。

**Q：`TextInput::new(cx)` 在 render 里编译失败？**
A：它的 `cx` 是 `Context<TextInput>`，只能在 `cx.new(|cx| …)` 闭包里调用，不能在父视图 render / 分区方法里直接 new。

**Q：NumberInput 的 `on_change` 闭包参数类型怎么写？**
A：首参是 `Option<f64>` **按值**（不是引用）：`cx.listener(|this, v: Option<f64>, _, cx| …)`。

**Q：自绘 Element 里 `if … { px(0.) } else { 0. }` 报 E0308？**
A：if/else 分支类型须一致。`Pixels / Pixels = f32`，所以两分支都应是 `f32`（`else { 0. }`），不要混用 `Pixels` 与 `f32`。

**Q：QRCode 中文内容显示不全？**
A：容量上限 134 字节（UTF-8 中文约 44 字），超长被截断。适合短文本 / URL。

**Q：Table 设了 `fixed_left()` 后展开行不生效？**
A：`fixed_left()` 与 `row_detail()` 互斥，同时设置时展开行被禁用。

**Q：Calendar 点月导航 / 切视图没反应？**
A：Calendar 全受控，`on_month_change` / `on_select` / `on_mode_change` 三个回调都要接并回写状态。

**Q：Transfer 移动 / 勾选不生效？**
A：`target_keys`（右栏）与 `selected`（勾选）双集合都要受控回写（`on_change` / `on_select`）。

**Q：`ImageView::new("d:/a.png")` 加载不出图片？**
A：`new` 走 GPUI 资产系统（Embedded），不是磁盘路径。本地文件用 `ImageView::from_file(path)`。

**Q：VideoPlayer 在非 Windows 编译报错？**
A：该控件 `#[cfg(windows)]` 仅 Windows 编译，其他平台自动排除，不影响其余控件。

**Q：Select 月导航 / DatePicker 翻月不生效？**
A：DatePicker / DateRangePicker 的显示月份受控，必须接 `on_month_change` 回写 `month`，否则月导航无效。

---

> 手册完。控件分类速查见 [`README.md`](../README.md)，项目集成与完整可运行示例见 [`integration-guide.md`](integration-guide.md)。