[English](integration-guide.en.md) | [中文](integration-guide.md)

# gpui-widgets 外部项目集成指南

> 配套文档：[README.md](../README.md) 提供 103 个控件的完整一览表；本文专注「如何在自己的 Rust 项目里引入并正确使用这些控件」。
> 文中完整示例与 [`examples/integration.rs`](../examples/integration.rs) 一致，已通过编译验证，可直接复制使用。

## 1. 引入依赖

`widgets/` 目录是一个**独立 crate**（`gpui-widgets`），依赖 crates.io 公开发布的 `gpui-pre = "=0.3.14"`（Zed GPUI 快照的第三方重发，库名仍为 `gpui`），**不需要 zed 源码**。

### 方式 A：crates.io 发布版（集成推荐）

```toml
[dependencies]
# gpui-pre 的库名仍是 gpui；与官方 gpui 0.2.2 不能混用，下游需整体切换
gpui = { package = "gpui-pre", version = "=0.3.14" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.14" }
gpui-widgets = "0.3"
```

### 本地开发模式（仅开发期）

与本库联合开发、需要即时改即时生效时，才用以下写法：

```toml
# 本地路径：指向本库目录
gpui-widgets = { path = "d:/gpui/widgets" }
# 或 git 仓库（团队协作 / 多机）：把 widgets/ 推送到 git 仓库后
# gpui-widgets = { git = "https://your-repo/gpui-widgets" }
```

也可以把 `widgets/` 整个目录拷进你的项目（如 `vendor/gpui-widgets`），再 `path = "vendor/gpui-widgets"`。库内无外部资产依赖（演示视频仅 examples 用），拷贝即可编译。

### 版本与编译建议

- Rust **1.89+**，edition 2024（本库与示例均用 2024）。
- `gpui-pre` 钉住 **0.3.14**（pre-1.0，升级需同步验证兼容性）；与官方 `gpui 0.2.2` 不能混用。
- 已验证平台：Windows（DirectWrite + DirectX）。macOS / Linux 由 GPUI 自身支持，未在本环境验证。
- `VideoPlayer` 仅 Windows 编译（`#[cfg(windows)]`），其他平台自动排除，不影响其余控件。
- dev  profile 建议加优化，否则 GPUI  debug 构建运行卡顿（consumer-demo 同款配置）：

```toml
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 2
```

## 2. 启动初始化

```rust
fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        // 必须：输入框编辑快捷键（Ctrl+A/C/V/X、方向键等），整个应用调用一次
        gpui_widgets::text_input::bind_keys(cx);
        // 按需：
        // gpui_widgets::text_area::bind_keys(cx);        // 用 TextArea 时
        // gpui_widgets::command_palette::bind_keys(cx);  // 用 CommandPalette 时（Ctrl+K）

        let bounds = Bounds::centered(None, size(px(520.), px(420.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| MyApp { /* … */ }),
        )
        .unwrap();
        cx.activate(true);
    });
}
```

忘记 `bind_keys` 的典型症状：输入框能打字但 Ctrl+C/V、全选、方向键移动光标全部无效。

## 3. 两类控件的使用范式

### 3.1 无状态控件（绝大多数）

Button / Checkbox / Switch / Select / Table / Calendar / Transfer 等**不在内部存状态**：
值由你在 render 时传入，回调里改你自己视图 `Entity` 的字段，然后 `cx.notify()` 触发重绘。

```rust
struct MyApp { city: usize, city_open: bool, /* … */ }

// render 中：
Select::new("city", ["北京", "上海", "广州", "深圳"], self.city)
    .open(self.city_open)
    .on_open_change(cx.listener(|this, open, _, cx| {
        this.city_open = *open;
        cx.notify();
    }))
    .on_select(cx.listener(|this, ix, _, cx| {
        this.city = *ix;
        this.city_open = false;   // 单选选中后收起
        cx.notify();
    }))
```

### 3.2 有状态控件（Entity）

输入系 / 媒体系 / 计时器控件内部持有光标 / 选区 / 解码器 / 剩余秒数等状态，是 `Entity`，**用 `cx.new` 创建一次**，render 中 `.clone()` 挂载：

| 控件 | 创建 | 读 / 写 |
|---|---|---|
| `TextInput` | `cx.new(\|cx\| TextInput::new(cx).placeholder("…"))` | `read(cx).text()` / `update(cx, \|i, cx\| i.set_text("…", window, cx))` |
| `TextArea` | `cx.new(\|cx\| TextArea::new(cx).height(px(120.)))` | `text()` / `set_text()` / `set_caret()` / `scroll_handle()` / `scroll_to_line()` / `set_scroll_offset()` / `scroll_metrics()` |
| `PasswordInput` / `SearchInput` / `Autocomplete` / `Mentions` | 同上，各自 `::new(cx)` | 同 TextInput |
| `OtpInput` | `cx.new(\|cx\| OtpInput::new(cx, 6))` | `value()` / `set_value(s, cx)` / `clear(cx)` / `is_complete()` |
| `NumberInput` | `NumberInput::new(cx)` | `value()` / `set_value()` |
| `TagInput` | `TagInput::new(cx).initial_tags(…)` | `tags()` |
| `CommandPalette` | `CommandPalette::new(cx, items)` | `open(window, cx)` / `close(cx)` |
| `VideoPlayer`（仅 Windows） | `VideoPlayer::new(cx, path)` | `play/pause/seek`、`position()/duration()` |
| `Countdown` | `Countdown::after(Duration, cx)` 或 `::new(unix秒, cx)` | `remaining()/finished()`；`on_finish` 归零回调（1 参） |
| `Sender` | `Sender::new(cx).on_send(…)` | 内部 TextArea；Enter 发送 / Shift+Enter 换行 |
| `FpsMeter` | `FpsMeter::new(cx).max_fps(120)` | `fps()/frame_ms()`；`start(cx)/stop(cx)`；仅重绘自身 |

```rust
// 创建（main 或父视图初始化时）：
name: cx.new(|cx| TextInput::new(cx).placeholder("请输入姓名")),
// render 中：
div().child(self.name.clone())
// 事件中读取：
let text = self.name.read(cx).text();
```

注意：`TextInput::new(cx)` 的 `cx` 是 `Context<TextInput>`，只能在 `cx.new(|cx| …)` 闭包里调用，**不能**在父视图的 render / 分区方法里直接 new。

## 4. 回调签名速查

| 签名 | 构造方式 | 涉及控件 |
|---|---|---|
| `Fn(&E, &mut Window, &mut App)` | `cx.listener(\|this, &e, window, cx\| …)` | 绝大多数：Button.on_click、Switch/Checkbox.on_change、Select.on_select、Table.on_row_click 等 |
| `Fn(&mut Window, &mut App)`（**无事件参**） | `let entity = cx.entity(); move \|_, cx\| { let _ = entity.update(cx, \|this, cx\| …); }` | Modal.on_close、Drawer.on_close、MenuItem::on_click、FloatButton.on_click、Popconfirm.on_confirm/on_cancel、Tour.on_close、Select.on_clear、DatePicker.on_clear、DateRangePicker.on_clear、Upload.on_click |
| `Fn(&mut App)`（**无 Window 无事件参**） | 同上实体句柄 | Countdown.on_finish（定时器上下文） |
| `Fn(&Vec<PathBuf>, &mut App)`（**无 Window**） | 同上实体句柄 | FilePicker.on_pick（异步对话框） |

两个高频编译错误：

- **E0593**：对 2 参回调用 `cx.listener` → 改实体句柄模式。
- **E0282**：链式调用中 `cx.listener` 闭包的事件参数推断失败 → 显式标注，如
  `cx.listener(|this, key: &gpui::SharedString, _, cx| …)`。

## 5. 受控状态与浮层约定

- **展开受控**：Select / MultiSelect / DropdownMenu / Popover / DatePicker / DateRangePicker / ColorPicker /
  Cascader / TimePicker / TreeSelect / Popconfirm 都用 `.open(bool)` + `.on_open_change(&bool)` 回写；
  不接 `on_open_change` 时点击触发框不会展开。
- **值受控**：Calendar 的 `.value()/.month()/.mode()` 三个回调都要接；Transfer 的
  `target_keys()`（右栏）与 `selected()`（勾选）双集合分别用 `on_change` / `on_select` 回写；
  DatePicker / DateRangePicker 的显示月用 `.month()` + `.on_month_change()`。
- **常用属性扩展**（对齐 antd）：Button `large()/loading()/block()/icon()`；TextInput
  `disabled()/readonly()/error()/allow_clear()/max_length()/prefix()/suffix()`；Select
  `value(Option<usize>)/allow_clear()/searchable()`；Checkbox `indeterminate()`；Switch `small()/loading()`；
  Table `column_widths()/sortable()+sort()+on_sort()/selectable()+selected_rows()+on_selected_rows()`（空数据自动空态）；
  Tabs `disabled_tabs()/closable()+on_close()/card()`；Tag `color(TagColor)/bordered()`；Badge
  `status(BadgeStatus)/text()/title()/aria_label()`，CountBadge 另有 `size(BadgeSize)/title()/aria_label()/offset(x, y)`；Modal `width()/closable()/mask_closable()`；Drawer
  `placement(DrawerPlacement::Left)/closable()/mask_closable()`；DatePicker `disabled()/allow_clear()+on_clear()`；
  Slider `vertical()/marks([(值, 标签)])`；Spinner `color()`。排序/多选等均为受控模式，回调里回写状态后 `cx.notify()`。
- **深化功能属性**：Table `pagination(page, page_size)+on_page_change()`、`row_detail()+expanded_rows()+on_expand()`、
  `fixed_left()/fixed_right_columns(n)`、`resizable_columns(true)+on_column_resize()`、
  `filterable()+filter()+on_filter_change()`、`editable_columns()+editing()+on_edit_start()/on_cell_change()/on_edit_end()`；
  Tree `checkable()+checked()+on_check()`、`draggable()+on_drop_node(&TreeDrop)`；
  TreeTable `expanded()/selected()/checked()` 三集合受控，`virtualized(height)` 固定行高虚拟滚动；
  FormItem `label_width()`（横向标签）/ `error_opt()`；新控件 `Form`（`inline()/gap()`）、
  `Rule` + `validate()`、`DateRangePicker`。详见下方「深化功能受控约定」。
- **属性对齐扩充**：Button `round()/circle()`；Steps `vertical()/small()/progress_dot()/descriptions()`；
  Pagination `simple()/small()`；Tabs `centered()`；Segmented `small()/large()/block()`；
  Switch `text(on, off)`；Timeline `alternate()/pending()`；Descriptions `vertical()`；
  Table `bordered()/show_header(bool)`；ProgressBar `steps(n)`；Alert `banner()`；Spinner `tip()`；
  Avatar `image()/image_error()/on_image_error()` 为受控图片失败态，`size()` 同步缩放文字；Card `hoverable()`。
  其中 `square()` 等纯展示属性不改变受控约定。
- **浮层渲染位置**：Modal / Drawer / ToastStack / NotificationStack / Tour 放在**根视图最后一个子元素、滚动容器之外**（内部 deferred 置顶），否则会被滚动裁剪或盖不住内容。
- **唯一 id**：构造函数带 id 的控件（Slider / Splitter / ContextMenu / Carousel / NavMenu /
  Cascader / TimePicker / TreeSelect / Transfer / Popconfirm 等）同一窗口内 id 不要重复。
- **主题全局可配置**：控件统一从 `gpui_widgets::theme` 取色（21 个语义色），默认浅色；
  `set_theme()` / `set_accent()` / `update()` 可全局换肤，详见下方「主题配置」。
  布局辅助用 `misc::{card, section_title, caption, field_row, Divider}`。

### 主题配置（明暗切换 / 品牌主色 / 自定义色板）

所有控件通过 `theme` 模块的语义色函数取色，因此换肤无需改动任何控件代码：

```rust
use gpui::{rgb, prelude::*};
use gpui_widgets::theme::{self, Theme};

// 1) 整体切换内置主题
theme::set_theme(Theme::dark());          // 或 Theme::light()
cx.refresh_windows();                     // 多窗口全量重绘；单视图也可 cx.notify()

// 2) 只换品牌主色：悬停 / 按下按亮度递减自动派生，
//    柔和底（accent_soft）按当前主题明暗取浅染底或深染底
theme::set_accent(rgb(0x8b5cf6));

// 3) 微调单个语义色
theme::update(|t| t.border = rgb(0xd0d3d9).into());

// 4) 自定义整套色板：Theme 字段全公开，也可基于内置主题改
let mut my = Theme::light();
my.bg = rgb(0xfaf7f2).into();
my.surface = rgb(0xffffff).into();
theme::set_theme(my.with_accent(rgb(0xc2410c)));

// 读取当前色板
let now: Theme = theme::current();
```

注意：主题存在全局静态中，整个应用共享一套；`set_theme` 等只改色板不触发重绘，
切换后需自行 `cx.refresh_windows()`（或在当前视图回调里 `cx.notify()`）。

### 深化功能受控约定（Table / Form / Tree / DateRangePicker）

**Table 分页 / 展开行 / 固定左右列 / DataGrid 能力**：

```rust
Table::new("members", ["姓名", "城市"], rows)
    .pagination(self.page, 4)                 // 页码从 1 起，受控
    .on_page_change(cx.listener(|this, p, _, cx| { this.page = *p; cx.notify(); }))
    .row_detail(|ix| detail_view(ix).into_any_element()) // 按全局下标生成详情
    .expanded_rows(self.expanded.clone())
    .on_expand(cx.listener(|this, ix, _, cx| { /* 切换 ix 的展开态 */ }))
```

- 分页后排序 / 选中 / 展开 / 行点击回调仍用**全局行下标**；表头全选只作用当前页。
- `fixed_left()` 固定首数据列（含勾选列），`fixed_right_columns(n)` 固定末尾 n 列，其余列横向滚动；
  要求统一行高 37px，未指定列宽的列默认 120px；**与 `row_detail()` 互斥**（同时设置时展开行不生效）；
  右栏下方内置主题化横向滚动条（滑块拖拽 / 轨道点击跳转 / 滚轮，见 Scrollbar 小节）。
- DataGrid P2：`filterable()` 在分页前做大小写不敏感筛选；`resizable_columns(true)` 需用
  `on_column_resize()` 回写完整列宽；`editable_columns()` + `editing()` 控制编辑单元格，双击或聚焦后按
  Enter/Space 进入，Enter 或失焦提交。完整示例见 `examples/data_grid.rs`。

**Form 校验（受控 helper，控件不持有校验状态）**：

```rust
use gpui_widgets::{Form, FormItem, Rule, validate};

// 提交时计算错误，存入视图字段
self.name_error = validate(&name, &[
    Rule::required("请输入用户名"),
    Rule::min_len(2, "至少 2 个字符"),
    Rule::custom("不能含空格", |v| !v.contains(' ')),
]);
// render 中显示
Form::new().child(
    FormItem::new("用户名").required().label_width(px(80.))
        .error_opt(self.name_error.clone())
        .child(self.name_input.clone()),
)
```

`validate` 返回首条未通过规则的消息（`Option<SharedString>`），全部通过返回 `None`；
`Form::new().inline()` 横向换行排列，适合筛选条。

**Tree 勾选联动 / 拖拽重组**：

```rust
use gpui_widgets::{Tree, TreeDrop, find_node_mut, take_node};

Tree::new("files", self.nodes.clone())        // TreeNode 实现 Clone，节点数据由你持有
    .checkable(true)
    .checked(self.checked.clone())
    .on_check(cx.listener(|this, ids, _, cx| { this.checked = ids.clone(); cx.notify(); }))
    .draggable(true)
    .on_drop_node(cx.listener(|this, drop: &TreeDrop, _, cx| {
        // 联动勾选已内置；拖拽只报告事件，重组自己写：
        if let Some(node) = take_node(&mut this.nodes, &drop.dragged) {
            if let Some(target) = find_node_mut(&mut this.nodes, &drop.target) {
                target.push_child(node);      // 移到目标节点下
            }
        }
        cx.notify();
    }))
```

勾选含全部子孙，祖先按「子全选则勾」自动同步；拖到自身 / 自身子孙上不触发 `on_drop_node`。

**DateRangePicker**：值 `Option<(Date, Date)>`，先点起点再点终点（早于起点则重选起点），
提交后自动收起；待选起点按控件 id 跨帧持久化，收起 / 清空时自动丢弃；
`.allow_clear(true)` 后有值时显示 ✕，`on_clear` 为 2 参回调（未设置时回退 `on_change(None)`）。

### 新增控件：Anchor / Upload / Countdown

**Anchor 锚点导航**（受控；GPUI 无浏览器式锚点滚动，定位由调用方处理）：

```rust
Anchor::new(vec![
    AnchorItem::new("basic", "基本用法"),
    AnchorItem::new("api", "API").child(AnchorItem::new("props", "属性")),
])
.active(self.anchor_key.clone())
.on_click(cx.listener(|this, key: &gpui::SharedString, _, cx| {
    this.anchor_key = key.clone();
    cx.notify();
}))
```

**Upload 上传触发区**（仅点击触发 UI；文件对话框 / 上传逻辑由调用方实现，不承诺原生拖放）：

```rust
let entity = cx.entity();
Upload::new()
    .text("点击上传")
    .hint("支持 png / jpg / pdf")
    .on_click(move |_, cx| {
        let _ = entity.update(cx, |this, cx| {
            // 在此打开文件对话框（可配合 rfd 等库）并执行上传
        });
    })
```

同窗口多个 Upload 须各自 `.id("…")` 唯一（默认 "upload-area"），否则 hover 高亮会互相串扰
（GPUI 按 ElementId 判定 hover）。

**Countdown 倒计时**（Entity；每秒按系统时钟重算防漂移，归零触发一次 `on_finish` 后停止计时）：

```rust
// 创建（父视图初始化 / 重置时）：
countdown: {
    let entity = cx.entity();
    cx.new(|cx| {
        Countdown::after(std::time::Duration::from_secs(90), cx).on_finish(move |cx| {
            let _ = entity.update(cx, |this, cx| { /* 归零动作，如弹 Toast */ });
        })
    })
},
// render 中：
div().child(self.countdown.clone())
```

- `on_finish` 签名为 `Fn(&mut App)`（定时器上下文无 Window），不适用 `cx.listener`，用实体句柄回写。
- 重新开始 = 重建实体；旧实体释放后计时循环自动退出，无泄漏。
- 读取进度用 `countdown.read(cx).remaining()` / `finished()`。

### 滚动条 Scrollbar

GPUI 默认不绘制滚动条。`Scrollbar` 按 rc-virtual-list 的实测值补上可见、可交互的滚动反馈：槽宽 8、滑块药丸圆角、长度下限 20；滑块 `rgba(0,0,0,0.5)`（antd 的 `--rc-` / `--vc-` 变量名失配使 `colorSplit` 覆写永不生效，实际渲染就是这个回退值）；轨道常态透明、悬停才 `colorFillQuaternary`。支持滑块拖拽（按住后允许移出轨道）、轨道点击跳转、滚轮驱动；内容未溢出时自动隐藏。

**关键约定**：绑定的 `ScrollHandle` 必须跨帧持久（放视图字段或 `window.use_keyed_state`），并给滚动容器接 `.track_scroll(&handle)`——二者共享同一 offset，`set_offset` 才能真正驱动滚动，滚轮滚动也会实时反映到滑块位置。

```rust
// 视图字段：scroll_handle: ScrollHandle（构造器 ScrollHandle::new() 初始化）
// render 中：
div()
    .relative()
    .size_full()
    .child(
        div()
            .id("area")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll_handle)
            .child(content),
    )
    // 纵向滚动条 overlay 在滚动区右缘
    .child(
        div()
            .absolute()
            .top_0()
            .bottom_0()
            .right_0()
            .child(Scrollbar::vertical("area-vsb", self.scroll_handle.clone())),
    )
```

- 横向用 `Scrollbar::horizontal(id, handle)`，纵向用 `Scrollbar::vertical(id, handle)`；槽宽由主题度量决定。
- 手写滚动偏移可用 `Scrollbar::vertical_with_metrics(id, handle, visible, max)` 适配，offset 仍由同一个 `ScrollHandle` 驱动；`Terminal` 已采用此入口。
- `TextArea` 已内置纵向滚动条，并提供 `scroll_handle()` / `scroll_to_line()` / `set_scroll_offset()` / `scroll_metrics()`；滚轮不再被 paint 拉回光标行，只有光标实际变化时才自动 reveal。
- `Table::fixed_left()`、`CodeBlock`、`CommandPalette`、`CommandBar`、`Drawer`、`Select`、`MultiSelect`、`Transfer`、`Cascader`、`Autocomplete`、`Mentions`、`Calendar` 年份列表、`TimePicker` 与 `Terminal` 已在对应滚动区内置滚动条。
- 同一窗口 id 不要重复（GPUI 按 ElementId 判定 hover）。
- **调滑块颜色 / 透明度**：全局用 `theme::set_scrollbar_thumb(Some(hsla))`（一次覆盖库内所有自绘滚动条，含 `Terminal` / `TextArea` / `Table` 等内置的），传 `None` 恢复 antd 回退黑 50%；单条用 `Scrollbar::vertical(..).thumb_color(hsla)`。优先级：元素级 > 全局 > 主题默认。
  ```rust
  // 深色主题下 antd 的黑滑块几乎看不见，宿主通常这样修：
  theme::set_scrollbar_thumb(Some(rgba(0xffffff4d).into())); // 白 30%
  // 只想让浅色下的滑块更淡：
  theme::set_scrollbar_thumb(Some(rgba(0x00000033).into())); // 黑 20%
  ```
  该覆盖不参与令牌派生：`set_theme` / `set_appearance` 不会清掉它，切明暗后需按新形态重设一次。取色是纯函数 `resolve_layered_color(element, global, default)`，矩阵见 `tests/scrollbar_thumb.rs`。
- **调轨道底色**：同款两层——全局 `theme::set_scrollbar_track(Some(hsla))`、单条 `Scrollbar::vertical(..).track_color(hsla)`，未覆写时取令牌 `colorFillQuaternary`。轨道本身只在悬停时绘制，覆写不改「常态不画轨道」这条语义，只换悬停色。

### 滑杆 Slider 配色

`Slider` 的取色默认全部由主题令牌派生（换主色用 `theme::set_accent` 即可整条跟着变）。需要逐实例换色时用 `SliderColors`：

```rust
use gpui::rgba;
use gpui_widgets::{Slider, SliderColors};

Slider::new("vol", 70.).range(0., 100.).colors(
    SliderColors::new()
        .rail(rgba(0x1f2329ff).into())   // 底槽
        .track(rgba(0x8b5cf6ff).into())  // 已选段
        .handle(rgba(0x8b5cf6ff).into()) // 滑块内芯
        .handle_ring(rgba(0xffffffff).into()), // 滑块环
)
```

- 只开放这四项；按下外圈 `handle_outline` 与刻度点始终跟令牌，免得把 antd 的 alpha 语义拆成手工配色。
- 未设置的字段保持令牌派生值，与悬停 / 按下 / 禁用态正交：同一份覆写套到不同状态，只换那四项。合并是纯函数 `SliderColors::apply(base)`，矩阵见 `tests/slider_matrix.rs::slider_colors_override_matrix`。

### 高级图表（数据可视化）

除 `BarChart` / `LineChart` / `Sparkline` / `RingProgress` 外，库提供 6 个高级图表，全部基于 GPUI 原生 `PathBuilder` + `window.paint_path` 自绘，无第三方依赖；默认取主题 **8 色分类色板**（`theme::chart_color(i)`，明暗主题自适应），也可逐项 `.color()` 覆盖。均为**受控组件**（无回调，数据由父视图传入）。

| 控件 | 创建 | 关键属性 |
|---|---|---|
| `PieChart` 饼/环图 | `PieChart::new(Vec<PieDatum>)`，`PieDatum::new(label, value)` | `donut(true)` / `inner_ratio(0.6)` 环图、`size(px)`、`legend(bool)`、`labels(bool)` 百分比 |
| `GaugeChart` 仪表盘 | `GaugeChart::new(value)` | `max(f32)`、`size(px)`、`thickness(px)`、`title(text)`、`label(text)`、`zones(vec![(比例上限, 色)])` 阈值色区、`color()` |
| `ScatterChart` 散点 | `ScatterChart::new(Vec<ScatterSeries>)`，`ScatterSeries::new(name, Vec<ScatterPoint>)`，`ScatterPoint::new(x, y)` | `height(px)`、`x_max(f32)` / `y_max(f32)`、`dot_radius(px)`、`legend(bool)` |
| `Heatmap` 热力 | `Heatmap::new(Vec<Vec<f32>>)`（行优先矩阵） | `row_labels(vec)` / `col_labels(vec)`、`cell_size(px)`、`gap(px)`、`colors(low, high)` 色阶两端、`legend(bool)` |
| `RadarChart` 雷达 | `RadarChart::new(axes: Vec<SharedString>, Vec<RadarSeries>)`，`RadarSeries::new(name, Vec<f32>)` | `max(f32)`、`size(px)`、`levels(usize)` 网格层数、`legend(bool)` |
| `FunnelChart` 漏斗 | `FunnelChart::new(Vec<FunnelStage>)`，`FunnelStage::new(label, value)` | `height(px)`、`gap(px)`；自动标注相对上一阶段转化率 |

```rust
use gpui_widgets::{PieChart, PieDatum, GaugeChart, theme};
use gpui::px;

// 环形饼图（右侧自动图例 + 扇区百分比）
PieChart::new(vec![
    PieDatum::new("直接访问", 335.),
    PieDatum::new("搜索引擎", 548.),
    PieDatum::new("邮件营销", 234.),
])
.donut(true)
.size(px(160.))

// 仪表盘：阈值色区随数值变绿/黄/红
GaugeChart::new(72.)
    .max(100.)
    .title("CPU 使用率")
    .zones(vec![
        (0.6, theme::success()),
        (0.85, theme::warning()),
        (1.0, theme::danger()),
    ])
```

- 图表为纯自绘受控元素，无内部交互状态，无需调用方维护；直接把当前数据传入即可，数据变化随父视图重绘。
- 分类色板定义在 `theme.rs`（`chart_palette()` / `chart_color(index)`），遵循「新控件不硬编码 rgb」约定；需要固定品牌色时用 `.color()` / `.colors()` 显式传入。
- gallery 「高级图表」分区有 6 图完整演示，仪表盘与音量滑杆实时联动可参考。

## 6. 完整集成示例（编译验证过）

新建项目，`Cargo.toml` 按第 1 节配置，`src/main.rs` 粘贴以下代码即可运行
（等价于 `cargo run --example integration`）：

```rust
use gpui::{
    App, Bounds, Context, Entity, FocusHandle, Focusable, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, size,
};
use gpui_widgets::{Button, Checkbox, Modal, Select, TextInput, ToastItem, ToastStack, theme};

struct MyApp {
    name: Entity<TextInput>,
    city: usize,
    city_open: bool,
    agreed: bool,
    modal_open: bool,
    toasts: Vec<ToastItem>,
    focus_handle: FocusHandle,
}

impl Focusable for MyApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl MyApp {
    fn push_toast(&mut self, item: ToastItem, cx: &mut Context<Self>) {
        self.toasts.push(item);
        cx.notify();
    }
}

impl Render for MyApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div()
            .size_full()
            .bg(theme::bg())
            .text_color(theme::text())
            .track_focus(&self.focus_handle(cx))
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.name.clone())
            .child(
                Select::new("city", ["北京", "上海", "广州", "深圳"], self.city)
                    .open(self.city_open)
                    .on_open_change(cx.listener(|this, open, _, cx| {
                        this.city_open = *open;
                        cx.notify();
                    }))
                    .on_select(cx.listener(|this, ix, _, cx| {
                        this.city = *ix;
                        this.city_open = false;
                        cx.notify();
                    })),
            )
            .child(
                Checkbox::new("我已阅读并同意条款", self.agreed).on_change(cx.listener(
                    |this, on, _, cx| {
                        this.agreed = *on;
                        cx.notify();
                    },
                )),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("打开对话框")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.modal_open = true;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("提交").primary().on_click(cx.listener(
                            |this, _, _, cx| {
                                let name = this.name.read(cx).text().to_string();
                                this.push_toast(
                                    ToastItem::new(format!("已提交：{name}")).success(),
                                    cx,
                                );
                            },
                        )),
                    ),
            );

        // 浮层放在根视图末尾、滚动容器之外
        if self.modal_open {
            let entity = cx.entity();
            root = root.child(
                Modal::new("确认")
                    .on_close(move |_, cx| {
                        let _ = entity.update(cx, |this, cx| {
                            this.modal_open = false;
                            cx.notify();
                        });
                    })
                    .child("确认提交当前表单吗？")
                    .footer(
                        div()
                            .flex()
                            .gap_2()
                            .justify_end()
                            .child(Button::new("取消").secondary().on_click({
                                let entity = cx.entity();
                                move |_, _, cx| {
                                    let _ = entity.update(cx, |this, cx| {
                                        this.modal_open = false;
                                        cx.notify();
                                    });
                                }
                            }))
                            .child(
                                Button::new("确定")
                                    .primary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.modal_open = false;
                                        this.push_toast(
                                            ToastItem::new("提交成功").success(),
                                            cx,
                                        );
                                    })),
                            ),
                    ),
            );
        }
        if !self.toasts.is_empty() {
            root = root.child(
                ToastStack::new(self.toasts.clone()).on_close(cx.listener(
                    |this, ix, _, cx| {
                        this.toasts.remove(*ix);
                        cx.notify();
                    },
                )),
            );
        }
        root
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        // 必须：注册输入框编辑快捷键（整个应用调用一次）
        gpui_widgets::text_input::bind_keys(cx);

        let bounds = Bounds::centered(None, size(px(520.), px(420.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| MyApp {
                    name: cx.new(|cx| TextInput::new(cx).placeholder("请输入姓名")),
                    city: 0,
                    city_open: false,
                    agreed: false,
                    modal_open: false,
                    toasts: Vec::new(),
                    focus_handle: cx.focus_handle(),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
```

## 7. 常见坑 FAQ

| 症状 | 原因与解法 |
|---|---|
| 输入框快捷键全无效 | 启动时没调 `text_input::bind_keys(cx)` |
| 回调里改状态界面不刷新 | 忘了 `cx.notify()` |
| E0593 闭包参数不匹配 | 2 参回调误用 `cx.listener`，改实体句柄（第 4 节） |
| E0282 类型推断失败 | `cx.listener` 事件参显式标注类型 |
| 弹窗 / 通知被裁剪、盖不住 | 浮层放进了滚动容器内，移到根视图末尾 |
| 下拉点了不展开 | 没接 `on_open_change` 回写 `open` |
| 自己包装 TextInput 时 panic（double lease） | 包装控件程序化写入用 `set_text_silent`，用户动作路径手动补外层回调；库内包装控件已遵循此模式 |
| 二维码内容不全 | QRCode 上限 134 字节（中文约 44 字），超长截断 |
| 列表滚不动 | `overflow_y_scroll()` 只在带 `.id()` 的 `Stateful<Div>` 上可用 |
| 非 Windows 编译报 VideoPlayer 错 | 用 `#[cfg(windows)]` 门控引用，库本身已自动排除 |
| 横向溢出 / 右侧被裁切 | flex 子项默认 min-width:auto 不缩到 min-content 以下；给可收缩列加 `.min_w(px(0.))`（外层滚动容器与内层 flex_1 列都要加） |
| 包装输入框回车不触发外层 | GPUI action 在 bubble 阶段默认停传播，容器 on_key_down 收不到已绑定 Enter；改用 TextArea `on_submit`（action 层提交模式） |
| 高帧率下整页卡顿 | 高频刷新放进独立 Entity（如 FpsMeter），`request_animation_frame` 只 notify 当前视图，避免整页重绘 |

## 8. 运行参考示例

```bat
cd d:\gpui\widgets
cargo run --example integration   # 本文完整示例
cargo run --example minimal       # 最小依赖示例
cargo run --example gallery       # 98 控件全量展示（左侧 9 类分类导航）
cargo run --example otp_input     # OTP 逐格输入与键盘 / 粘贴
cargo run --example notification_center # 通知中心、虚拟列表与已读 / 删除交互
cargo run --example tree_table      # 树形表格、级联勾选、键盘导航与虚拟滚动
cargo run --example data_grid      # Table 筛选、列宽拖拽、固定左右列与单元格编辑
```

外部独立项目参考 `d:\gpui\consumer-demo`（仅依赖 `gpui-widgets` path 引用，已经回归验证）。