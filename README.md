[English](README.en.md) | [中文](README.md)

# gpui-widgets

基于 [GPUI](https://gpui.rs)（Zed 的 GPU 加速 UI 框架）的通用控件库，Rust 编写，包含 **98 个已文档化控件 / 101 个条目，另有 8 个公开高级/宿主模块**（含参照 Ant Design 组件总览补齐的导航 / 展示 / 录入 / 反馈类，及后续属性对齐、数据可视化、AI 对话与性能监视扩充），可直接被其他项目依赖使用。

依赖 crates.io 公开发布的 `gpui-pre = "=0.3.14"`（Zed GPUI 快照的第三方重发，crate 内库名仍为 `gpui`），**无需本地 zed 源码**。

> **命名澄清**：本 crate 是 crates.io 上独立发布的通用控件库，与 Zed 源码仓库内的
> `crates/gpui_widgets`（Zed 编辑器内部工作区组件）**不是同一个东西**：仅名字相近，
> 无代码同源或归属关系，API 亦不互通。

## 控件一览（按用途分类）

参照 Ant Design 组件总览的用途划分，同一控件只归属一类；`misc` / `theme` 为辅助模块单列。

> 逐个控件的完整用法（定位 / 创建签名 / 状态模型 / 属性 / 回调事件类型 / 示例 / 注意事项）见 **[控件参考手册 `docs/controls-reference.md`](docs/controls-reference.md)**。

| 类别 | 包含控件 |
|---|---|
| 通用 | `Button` `Link` `Title/Paragraph/Text` `Kbd` |
| 布局 | `Row/Col` `Header/Sider/Content/Footer` `Space` `Splitter` `Scrollbar` |
| 导航 | `Anchor` `Breadcrumb` `DropdownMenu` `NavMenu` `Pagination` `Steps` `Tabs` `ContextMenu` `FloatButton` `CommandPalette` |
| 数据录入 | `TextInput` `TextArea` `PasswordInput` `SearchInput` `NumberInput` `Stepper` `TagInput` `Autocomplete` `Mentions` `Select` `MultiSelect` `Cascader` `TreeSelect` `Checkbox` `Radio` `Switch` `Slider` `Rating` `DatePicker` `DateRangePicker` `TimePicker` `ColorPicker` `Transfer` `Upload` `UploadList` `FilePicker` `Form` `FormItem` |
| 数据展示 | `Table` `Tree` `List` `Card` `Descriptions` `Statistic` `Countdown` `Calendar` `Timeline` `Tag` `Badge` `CountBadge` `Avatar` `ImageView` `VideoPlayer` `Carousel` `BarChart` `LineChart` `Sparkline` `PieChart` `GaugeChart` `ScatterChart` `Heatmap` `RadarChart` `FunnelChart` `QRCode` `CodeBlock` `Empty` `Segmented` `Accordion` `Tooltip` `Popover` `Tour` |
| 反馈 | `Alert` `Modal` `Drawer` `ToastStack` `NotificationStack` `Popconfirm` `ProgressBar` `RingProgress` `Result` `Skeleton` `Spinner` `Watermark` |
| AI 对话 | `Welcome` `Prompts` `Bubble` `Sender` `Conversations` `ThoughtChain` |
| 辅助与主题 | `misc` `theme` `FpsMeter` |

### 通用

| 控件 | 说明 |
|---|---|
| `Button` | 按钮：`primary()` / `secondary()` / `ghost()` / `danger()` 变体，`small()` / `large()` 尺寸，`loading()` 加载态（阻点击），`block()` 占满整行，`icon()` 前置图标字符，`round()` 胶囊 / `circle()` 圆形，`disabled()` |
| `Link` | 链接：悬停下划线，`href()` 点击打开浏览器 / `on_click()` 自定义 / `disabled()` |
| `Title` / `Paragraph` / `Text` | 排版三件套：1~5 级标题 / 段落 / 行内文本（`strong()` / `italic()` / `underline()` / `delete()` / `code()` / `mark()` / `secondary()`） |
| `Kbd` | 键帽：`Kbd::new("Esc")` 单键 / `Kbd::combo("Ctrl+K")` 组合键自动拆分 |

### 布局

| 控件 | 说明 |
|---|---|
| `Row` / `Col` | 24 栅格：`Col::new(span)` + `offset()`，`Row::gutter()` 列间距 |
| `Header` / `Sider` / `Content` / `Footer` | 页面布局四件套：顶栏 / 侧栏（`Sider::new(width)`）/ 内容区 / 底栏 |
| `Space` | 间距容器：`vertical()` / `wrap()` / `size()` 控制子元素间距 |
| `Splitter` | 分割面板：左右 / `vertical()` 上下两栏，拖动分隔条调比例，`ratio()` 受控 + `on_change(&f32)`，需唯一 id |
| `Scrollbar` | 主题化滚动条：绑定跨帧持久的 `ScrollHandle`（滚动容器接 `.track_scroll()`），`horizontal()` / `vertical()` 两向，手写滚动可用 `vertical_with_metrics()` 适配；自绘轨道 + 滑块，支持拖拽 / 轨道点击 / 滚轮，内容未溢出自动隐藏；TextArea、Terminal、CodeBlock、Table 及主要下拉/浮层已内置 |

### 导航

| 控件 | 说明 |
|---|---|
| `Anchor` | 锚点导航：`AnchorItem` 嵌套目录，左侧指示轨 + 当前高亮，`active()` 受控 + `on_click(&SharedString)`；滚动定位由调用方处理 |
| `Breadcrumb` | 面包屑导航：末项为当前页不可点，`on_click(&usize)` 回调中间层级 |
| `DropdownMenu` | 动作菜单：任意触发元素 + `MenuItem` 列表（支持分隔线 / `danger()` / `disabled()`），点动作后收起 |
| `NavMenu` | 导航菜单：`MenuEntry` 嵌套子菜单，图标 / 禁用，选中与展开集合受控，`on_select()` / `on_toggle()` |
| `Pagination` | 分页器：页码从 1 起，页数多时自动省略号折叠，边界按钮置灰，`simple()` 简洁模式，`small()` 小尺寸 |
| `Steps` | 步骤条：完成 ✓ / 当前数字 / 待办灰，`current()` 受控，`vertical()` 纵向，`small()` 小尺寸，`progress_dot()` 点状，`descriptions()` 步骤描述 |
| `Tabs` | 页签：下划线指示器，`on_select(&usize)`；`disabled_tabs()` 禁用页签，`closable()` + `on_close(&usize)` 可关闭，`card()` 卡片式，`centered()` 居中 |
| `ContextMenu` | 右键菜单包装器：包裹任意内容，右键在光标处弹出 `MenuItem` 菜单（视口内夹取），需唯一 id |
| `FloatButton` | 悬浮按钮：absolute 圆形按钮固定容器右下角，`bottom()` / `right()` / `tooltip()`，父容器须 `relative()` |
| `CommandPalette` | 命令面板：Ctrl+K 唤起的搜索式命令选择器（遮罩 + 居中面板），`PaletteItem` 可带快捷键提示，本身是 `Entity` |

### 数据录入

| 控件 | 说明 |
|---|---|
| `TextInput` | 单行输入框：光标 / 选区 / 剪贴板 / 中文输入法（IME），本身是 `Entity`；`bare()` 去边框供包装，`masked()` 掩码显示；`disabled()` / `readonly()` / `error()` 状态，`allow_clear()` ✕ 清空，`max_length()` 字数上限，`prefix()` / `suffix()` 前后缀 |
| `TextArea` | 多行输入：硬换行 / 跨行选区 / 剪贴板 / IME / 光标自动滚动，内置纵向滚动条；`height()` / `scroll_handle()` / `scroll_to_line()` / `set_scroll_offset()` / `scroll_metrics()`，本身是 `Entity`，需额外 `text_area::bind_keys` |
| `PasswordInput` | 密码输入：掩码显示 + 明文切换按钮，本身是 `Entity` |
| `SearchInput` | 搜索输入：自绘放大镜图标 + ✕ 清空，`on_search()` 回车触发，本身是 `Entity` |
| `NumberInput` | 数值输入：▲▼ 按钮 / 上下方向键步进，`min()` / `max()` / `step()`，非法输入不回调，本身是 `Entity` |
| `Stepper` | 数值步进器：− / + 按钮，`min()` / `max()` / `step()` / `disabled()`，边界自动置灰 |
| `TagInput` | 标签输入：回车添加（自动去重）/ 空草稿退格删末尾 / ✕ 删单个，`initial_tags()`，本身是 `Entity` |
| `Autocomplete` | 自动完成：输入过滤候选（大小写不敏感），↑↓ 选择 / 回车确认，本身是 `Entity` |
| `Mentions` | @ 提及输入：输入 @ 弹出候选浮层，选中替换为「@名称 」，本身是 `Entity`，`on_select(&SharedString)` |
| `Select` | 下拉选择器：菜单默认与触发框同宽、贴框下方展开（下方空间不足时自动翻转到上方），点击外部收起；展开状态受控（`open()` + `on_open_change()`）；`value(Option<usize>)` 可空选中，`allow_clear()` ✕ 清空，`searchable()` 菜单内搜索过滤 |
| `MultiSelect` | 多选下拉：触发框显示已选芯片（✕ 可单项移除），菜单带勾选标记、选中不收起，`on_change(&Vec<usize>)`，展开状态同样受控 |
| `Cascader` | 级联选择：`CascaderNode` 多级菜单横排展开，路径值受控，点叶子收起，`on_change(&Vec<SharedString>)` |
| `TreeSelect` | 树选择：下拉浮层内嵌 `Tree`，复用 `TreeNode`，选中 id 与展开集合受控 |
| `Checkbox` | 复选框，可带标签，受控组件，`indeterminate()` 半选态 |
| `Radio` | 单选按钮，配合父视图状态组成单选组 |
| `Switch` | 开关，可带标签，`small()` 小尺寸，`loading()` 加载态，`text(on, off)` 轨道内文字 |
| `Slider` | 滑杆：点击定位 + 拖拽，`range()` / `step()`，`vertical()` 垂直方向，`marks()` 刻度圆点 + 标签，需唯一 id |
| `Rating` | 星级评分：`max()` / `readonly()`，点击当前星数可清零 |
| `DatePicker` | 日期选择器：日历浮层（月导航 / 今日快捷 / 选中高亮），值与显示月份均受控，`disabled()` 禁用，`allow_clear()` + `on_clear()` 清空 |
| `DateRangePicker` | 日期范围选择器：先点起点再点终点（早于起点则重选），值 `Option<(Date, Date)>` 与展开 / 月份受控，`allow_clear()` + `on_clear()`，范围内日期高亮 |
| `TimePicker` | 时间选择器：时 / 分 / 秒三列滚动菜单，值 `Option<(u32, u32, u32)>` 受控 |
| `ColorPicker` | 颜色选择器：24 色预设色板，触发框显示色块 + HEX，选中即回调并收起 |
| `Transfer` | 穿梭框：左右双面板勾选 + →← 移动，`target_keys()` / `selected()` 受控，`titles()` 自定义标题 |
| `Upload` | 上传触发区：虚线框 + 图标 + `text()` / `hint()` 文案，`disabled()`，`on_click` 2 参回调（由调用方打开文件对话框，不内置拖放） |
| `UploadList` | 上传文件列表：`UploadFile` 名称 / 状态（Done / Uploading / Error）/ 进度条，`on_remove(&usize)` |
| `FilePicker` | 文件选择按钮：系统原生对话框，`multiple()` / `directories()` / `prompt()`，异步 `on_pick` 回传路径 |
| `Form` | 表单布局容器：默认纵向排列 FormItem，`inline()` 横向换行，`gap()` 项间距；配套 `Rule`（`required` / `min_len` / `max_len` / `custom`）+ `validate()` 受控校验 helper |
| `FormItem` | 表单项包装器：标签 + `required()` 红星 + `error()` / `error_opt()` 红框错误提示 + `help()` 说明文字，`label_width()` 横向布局（标签左置） |

### 数据展示

| 控件 | 说明 |
|---|---|
| `Table` | 数据表格：`column_widths()` 列宽（默认均分），`striped()` 斑马纹，行选中受控 + `on_row_click()`；`sortable()` + `sort()` + `on_sort()` 表头排序，`selectable()` + `selected_rows()` + `on_selected_rows()` 多选，空数据自动显示空态；`pagination(page, page_size)` + `on_page_change()` 内置分页（回调仍用全局下标，全选仅当前页），`row_detail()` + `expanded_rows()` + `on_expand()` 展开行，`fixed_left()` 固定首列 + 横向滚动（与展开行互斥），`bordered()` 全边框，`show_header(bool)` 表头显隐 |
| `Tree` | 树形控件：`TreeNode` 嵌套（`Clone`），展开集合 / 选中 id 受控，`on_toggle()` / `on_select()`；`checkable()` + `checked()` + `on_check()` 勾选父子联动，`draggable()` + `on_drop_node(&TreeDrop)` 拖拽移动（重组由调用方用 `take_node()` / `find_node_mut()` / `push_child()` 完成） |
| `List` | 列表：`ListItem` 标题 / 描述 / 头像 / `extra()` 右侧元素，`header()` / `footer()` |
| `Card` | 卡片：标题 + `extra()` 右上操作 + `cover()` 封面 + `action()` 底部操作区，`bordered()`，`hoverable()` 悬停浮起阴影 |
| `Descriptions` | 描述列表：`item(label, value)` 键值对，`column()` 每行列数，`bordered()` 表格样式，`vertical()` 垂直布局（标签上值下） |
| `Statistic` | 统计数值：标题 + 大数字 + `prefix()` / `suffix()` + `trend()` 涨跌箭头 |
| `Countdown` | 倒计时：天/时/分/秒单位块，`Countdown::new(unix秒, cx)` / `after(Duration, cx)` 创建，每秒按系统时钟重算，`on_finish` 归零回调（1 参 `Fn(&mut App)`），本身是 `Entity` |
| `Calendar` | 日历：月视图大格 + 年视图 12 迷你月历，月份 / 选中日期 / 模式均受控，`on_month_change()` / `on_select()` / `on_mode_change()` |
| `Timeline` | 时间线：`TimelineItem` 标题 / 描述 / 时间 / 节点色，`alternate()` 交替左右，`pending()` 幽灵节点 |
| `Tag` | 标签：比徽章更紧凑，`on_close()` 显示 ✕ 移除按钮，`color(TagColor)` 8 种颜色预设，`bordered()` 边框开关 |
| `Badge` | 徽章：`info()` / `success()` / `warning()` / `danger()`，`color()` 自定义色 |
| `CountBadge` | 数字角标：包裹子元素右上角计数，`max()` 溢出显示 99+，`dot()` 红点模式，`show_zero()`，`color()` 自定义底色，`offset(x, y)` 位置偏移 |
| `Avatar` | 头像：圆形背景 + 名字首字符，背景色由名字哈希确定，`size()`，`square()` 圆角方形 |
| `ImageView` | 图片：封装 gpui `img`，`from_file()` 本地文件 / `from_render()` 内存帧，`fit_cover()` / `fit_contain()` / `fit_fill()` / `rounded()` / `grayscale()`，内置 loading 与加载失败占位 |
| `VideoPlayer` | 视频播放器（**仅 Windows**）：Media Foundation 后台解码，内置播放/暂停按钮 + 进度拖拽条，`play()` / `pause()` / `seek()`，本身是 `Entity` |
| `Carousel` | 轮播图：`ImageSource` 列表，‹ › 循环切换 + 底部圆点直达 + 计数，`current()` 受控，需唯一 id |
| `BarChart` | 柱状图：`BarDatum` 标签 + 数值 + 单柱颜色，`height()` / `max()` |
| `LineChart` | 折线图：网格 / `area()` 面积 / `dots()` 圆点 / `labels()` X 轴标签，自绘路径 |
| `Sparkline` | 迷你趋势图：无坐标轴小尺寸折线，`w()` / `h()` / `area()`，适合嵌入表格/卡片 |
| `PieChart` | 饼图 / 环形图：`PieDatum` 标签 + 数值 + 单扇区颜色，`donut()` / `inner_ratio()` 环图，`size()` / `legend()` / `labels()` 百分比标签，主题分类色板自绘扇区 |
| `GaugeChart` | 仪表盘：270° 圆弧 + 指针 + 中心数值，`max()` / `size()` / `thickness()` / `title()` / `label()`，`zones()` 阈值色区（绿/黄/红），`color()` 单色 |
| `ScatterChart` | 散点图：`ScatterSeries` 多系列 + `ScatterPoint` 坐标，网格 / 坐标轴 / 图例，`height()` / `x_max()` / `y_max()` / `dot_radius()` / `legend()` |
| `Heatmap` | 热力图：二维矩阵色块 + 行列标签 + 色阶图例，`row_labels()` / `col_labels()` / `cell_size()` / `gap()` / `colors(low, high)` 色阶两端 / `legend()` |
| `RadarChart` | 雷达图：`RadarSeries` 多系列半透明填充 + 多轴网格环，`max()` / `size()` / `levels()` 网格层数 / `legend()` |
| `FunnelChart` | 漏斗图：`FunnelStage` 阶段居中梯形堆叠 + 阶段标签 + 转化率，`height()` / `gap()` |
| `QRCode` | 二维码：纯 Rust 编码（byte 模式 / ECC L / version 1~6），`size()` 像素尺寸，自绘渲染无外部依赖 |
| `CodeBlock` | 代码块：等宽字体逐行渲染 + 语言标签 + 一键复制按钮 |
| `Empty` | 空状态：图标 + 标题 + `description()` + `action()` 操作区 |
| `Segmented` | 分段选择器：圆角容器内互斥选项，适合 2~5 个短选项，`small()` / `large()` 尺寸，`block()` 占满整行 |
| `Accordion` | 折叠面板组：多面板可同时展开，`AccordionItem` 标题 + 任意内容，展开集合受控 |
| `Tooltip` | 工具提示包装器：悬停任意控件显示气泡（GPUI 原生延迟与定位） |
| `Popover` | 气泡卡片：任意触发元素 + 任意可交互内容，比 Tooltip 更重，展开受控 |
| `Tour` | 分步引导：遮罩 + 居中卡片 + 步骤圆点，`current()` 受控，跳过 / 上一步 / 下一步完成 |

### 反馈

| 控件 | 说明 |
|---|---|
| `Alert` | 提示条：`info()` / `success()` / `warning()` / `danger()`，`description()` / `on_close()`，`banner()` 通栏模式（无边框圆角） |
| `Modal` | 模态对话框：遮罩 + 居中卡片，`width()` 卡片宽度，`closable()` / `mask_closable()` 控制 ✕ 与遮罩关闭，`footer()` 操作区，`id()` 实例唯一 id（默认按标题派生）；须在滚动容器之外渲染 |
| `Drawer` | 抽屉：遮罩 + 侧滑面板，`placement(DrawerPlacement)` 左 / 右（默认右），`width()` / `footer()`，`closable()` / `mask_closable()`，`id()` 实例唯一 id（默认按标题派生）；须在滚动容器之外渲染 |
| `ToastStack` | 轻提示堆栈：右上角浮层，`ToastItem` 四种类型，条目受控、✕ 移除 |
| `NotificationStack` | 通知堆栈：右上角浮层，`NotificationItem` 标题 / 描述 / 类型 / `action()` 按钮，条目受控 |
| `Popconfirm` | 气泡确认框：包裹触发元素，`description()` / `ok_text()` / `cancel_text()`，`on_confirm()` / `on_cancel()` |
| `ProgressBar` | 进度条（0.0 ~ 1.0），`steps(n)` 分段进度 |
| `RingProgress` | 环形进度：自绘圆弧轨道 + 进度弧，`size()` / `thickness()` / `color()` / `label()` 中心文字 |
| `Result` | 结果页：`ResultStatus` 四种状态图标 + 标题 + `description()` + `action()` |
| `Skeleton` | 骨架屏：呼吸动画占位，`circle()` / `lines()` / `w_percent()` |
| `Spinner` | 加载指示器：旋转淡出圆点环，`size()` / `color()`，`tip()` 加载提示文字 |
| `Watermark` | 水印包装器：文字隔行错位平铺铺满内容区，`font_size()` / `color()` / `gap()`，不拦截子控件交互 |

### AI 对话

| 控件 | 说明 |
|---|---|
| `Welcome` | 欢迎卡片：`icon()` 图标 + 标题 + `description()` 描述，AI 开场引导，受控无回调 |
| `Prompts` | 提示词卡片组：`PromptItem` 图标 + 文本，`on_click(&usize)` 点击回调 |
| `Bubble` | 消息气泡：`avatar()` 头像 + `right()` 用户侧 + `loading()` 加载态 + `variant(Filled/Outlined/Shadow)`，长文本自动换行限宽 |
| `Sender` | 发送框：本身是 `Entity`，`on_send(&SharedString, &mut Window, &mut App)`；Enter 发送 / Shift+Enter 换行 / 按钮发送 |
| `Conversations` | 会话列表：`ConversationItem` key + 标题 + 图标，`active()` 受控 + `on_select(&SharedString)` |
| `ThoughtChain` | 思维链：`ThoughtNode` 标题 / 描述 / `status()`，`expanded()` 受控 + `on_toggle(&usize)` 展开收起 |

### 辅助与主题

| 控件 | 说明 |
|---|---|
| `misc` | 布局辅助：`card()` / `Divider` / `section_title()` / `caption()` / `field_row()` |
| `theme` | 统一主题：21 个语义色（`bg()` / `surface()` / `accent()` / `text()` 等），全局可配置：`Theme::light()` / `Theme::dark()` 内置明暗两套，`set_theme()` 整体切换、`set_accent()` 换品牌主色（悬停/按下/柔和底自动派生）、`update()` 微调单色 |
| `FpsMeter` | 帧率/性能监视器：本身是 `Entity`，独立驱动连续渲染仅重绘自身，实测 FPS + 帧耗时 + 曲线；`max_fps()` 标称上限（默认 120）、`running()` / `start()` / `stop()`；实际帧率跟随显示器 vsync |

## 高级与宿主模块

以下 8 个模块已公开并完整收录于[控件参考手册](docs/controls-reference.md#十高级与宿主模块)，但不并入上面的 98 个已文档化控件 / 101 个条目，也不把总数写成 106。

| 模块 | 状态模型 | 用途 |
|---|---|---|
| `HotkeyInput` | `Entity` | 点击录入组合键，同时提供显示串与 GPUI 绑定串 |
| `Markdown` | `RenderOnce` | 零依赖 GFM 常用子集渲染、链接与代码块动作 |
| `TransferManager` | `RenderOnce` | 上传 / 下载队列、总进度、单项进度与状态操作 |
| `Terminal` | `Entity` | VT/ANSI 终端网格、回滚、搜索、选区与 PTY 输入桥接 |
| `CodeEditor` | 预设工厂 | 返回配置好的 `Entity<TextArea>`，提供逐行语法着色 |
| `CommandBar` | `Entity` | 命令输入、历史回溯、片段补全与管理 |
| `DropZone` | `RenderOnce` | 接收操作系统外部文件拖放，并按扩展名分流 |
| `TitleBar` | `RenderOnce` | 自绘标题栏、窗口按钮与 Windows 原生命中区域 |

## 在其他项目中使用

完整集成文档见 [`docs/integration-guide.md`](docs/integration-guide.md)（依赖配置 / 初始化 / 回调签名 / 受控约定 / 完整示例 / 常见坑 FAQ）。

### 1. 添加依赖

```toml
[dependencies]
gpui-widgets = "0.3"   # crates.io 发布版
# gpui-pre 的库名仍是 gpui；与官方 gpui 0.2.2 不能混用，下游需整体切换
gpui = { package = "gpui-pre", version = "=0.3.14" }
# 宿主入口（gpui 0.3.14 起平台层拆到独立 crate）
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.14" }
```

本地联合开发改用 path 依赖的写法见 [`docs/integration-guide.md`](docs/integration-guide.md)
的「本地开发模式」一节（仅开发文档保留 path 示例）。

### 2. 最小示例

```rust
use gpui::{App, Bounds, Context, Entity, Window, WindowBounds, WindowOptions,
           div, prelude::*, px, size};
use gpui_widgets::{Button, TextInput, theme};

struct MyApp {
    input: Entity<TextInput>,
}

impl Render for MyApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(theme::bg())
            .text_color(theme::text())
            .p_6()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.input.clone())
            .child(Button::new("确定").primary().on_click(cx.listener(|this, _, _, cx| {
                println!("输入内容：{}", this.input.read(cx).text());
            })))
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        // 必须：注册 TextInput 的编辑快捷键（整个应用调用一次）
        gpui_widgets::text_input::bind_keys(cx);

        let bounds = Bounds::centered(None, size(px(480.), px(320.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| MyApp {
                    input: cx.new(|cx| TextInput::new(cx).placeholder("请输入…")),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
```

完整可运行示例见 [`examples/minimal.rs`](examples/minimal.rs)，全部控件展示见 [`examples/gallery.rs`](examples/gallery.rs)（左侧 9 类分类导航：通用 / 布局 / 导航 / 数据录入 / 数据展示 / 反馈 / AI 对话 / 辅助与主题 / 综合演示，点击切换右侧仅渲染该类）：

```bat
cargo run --example minimal
cargo run --example integration   # 集成指南配套示例
cargo run --example gallery
cargo run --example gen_assets   # 重新生成演示视频 examples/assets/sample.avi（仓库已附带）
```

## 使用约定

- **无状态控件**（Button / Checkbox / Switch / Radio / Slider / ProgressBar / Badge /
  Tabs / Segmented / Select / Alert / Spinner / Avatar / Tag / Stepper / Accordion /
  Breadcrumb / Pagination / Rating / Table / Tree / DropdownMenu / Popover /
  DatePicker / ColorPicker / ToastStack / NavMenu / Card / Descriptions / List /
  CountBadge / Calendar / QRCode / Cascader / TimePicker / TreeSelect / Transfer /
  FormItem / UploadList / Popconfirm / NotificationStack / Tour / Anchor / Upload）：
  值与回调由调用方传入，状态保存在你自己的视图 `Entity` 中，回调里改状态后 `cx.notify()`。
  回调签名统一为 `Fn(&E, &mut Window, &mut App)`，可直接用 `cx.listener(...)` 构造。
- **Select / MultiSelect 的展开状态受控**：父视图持有 `open: bool`，
  用 `.open(self.open)` + `.on_open_change(...)` 回写；单选用 `.on_select(&usize)`，
  多选用 `.on_change(&Vec<usize>)`。菜单宽度默认等于触发框宽度，可用 `.menu_width(px(...))` 覆盖；
  菜单贴触发框下方 4px 展开，窗口下方放不下时自动翻转到框上方。
  单选额外支持 `.value(Option<usize>)` 空选中、`.allow_clear(true)` ✕ 清空（`on_clear` 为 2 参回调）、
  `.searchable(true)` 菜单顶部搜索框过滤选项。
- **Modal / Drawer / ToastStack / NotificationStack / Tour 的渲染位置**：放在根视图最后一个子元素、滚动容器之外；
  **叠层弹窗/抽屉**：✕ 与遮罩的 element id 默认按标题派生，标题不同即互不干扰；
  标题相同时必须给每层传彼此不同的 `.id(...)`，否则共享交互态会让内层 ✕ 点了没反应、外层反而进 active 态
  （id 须跨帧稳定，勿用每帧自增计数器）；
  Modal / Drawer 的 `on_close` 签名为 `Fn(&mut Window, &mut App)`（无事件参数），不适用 `cx.listener`，
  可用 `cx.entity()` 句柄在闭包内 `entity.update(cx, …)`；
  DropdownMenu 的 `MenuItem::on_click`、FloatButton 的 `on_click`、Popconfirm 的 `on_confirm` / `on_cancel`、
  Tour 的 `on_close`、Select / DatePicker 的 `on_clear`、Upload 的 `on_click` 同样是该签名。
- **Tooltip 是包装器**：`Tooltip::new("提示文字", 任意控件)`，悬停延迟与定位由 GPUI 原生处理。
- **TextInput 是有状态视图**：用 `cx.new(|cx| TextInput::new(cx))` 创建，
  render 中 `.child(input.clone())`；读取用 `input.read(cx).text()`，
  写入用 `input.update(cx, |i, cx| i.set_text("…", window, cx))`。
  输入增强控件（TextArea / PasswordInput / SearchInput / NumberInput / TagInput /
  Autocomplete / CommandPalette）同样是 `Entity`，创建与读写方式一致
  （NumberInput 用 `value()` / `set_value()`，TagInput 用 `tags()`）。
- **Slider 需要唯一 id**：`Slider::new("volume", value)`，同一窗口内不要重复。
- **Scrollbar 绑定持久 ScrollHandle**：`Scrollbar::horizontal/vertical(id, handle)`，handle 须放在视图字段或 `window.use_keyed_state` 跨帧持久，并给滚动容器接 `.track_scroll(&handle)`（二者共享 offset，`set_offset` 才能真正驱动滚动，滚轮也会实时联动滑块）；同一窗口 id 不要重复。
- **浮层类控件共享定位机制**：Select / MultiSelect / DropdownMenu / Popover / DatePicker / DateRangePicker / ColorPicker /
  Cascader / TimePicker / TreeSelect / Popconfirm
  的浮层都贴触发框下方 4px 展开，窗口下方放不下时自动翻转到上方，点击外部收起（展开均受控，
  用 `.open(bool)` + `.on_open_change(&bool)` 回写）。
- **DatePicker 的显示月份受控**：`.month(Option<(年, 月)>)` + `.on_month_change(...)` 回写，
  不传时默认显示选中值所在月（无值则当前月）；不接 `on_month_change` 时月导航不生效。
  DateRangePicker 同约定（月份默认取起点所在月）；待选起点按 id 跨帧持久化，收起 / 清空时自动丢弃。
- **Table 分页与固定列**：分页受控（页码从 1 起），排序 / 选中 / 展开等回调一律用**全局行下标**，
  表头全选只作用当前页；`fixed_left()` 要求统一行高（37px），未指定列宽的列默认 120px，
  与 `row_detail()` 同时设置时展开行不生效。
- **Form 校验是受控 helper**：`validate(&value, &[Rule])` 返回首条未通过规则的错误消息（`Option<SharedString>`），
  存入视图状态后交给 `FormItem::error_opt()` 显示；控件本身不持有校验状态，提交时机由调用方决定。
- **Tree 的节点 id**：同一棵树内唯一即可，展开集合与选中值都用 id 标识（`Vec<SharedString>` / `Option<SharedString>`）；
  TreeSelect 复用同一套 `TreeNode`，NavMenu 的 `MenuEntry` 用 key 标识。
  勾选联动已内置（勾选含全部子孙，祖先按子全选自动同步）；拖拽只报告 `TreeDrop { dragged, target }`
  （目标为自身 / 自身子孙时不触发），节点树重组由调用方用 `take_node` / `find_node_mut` / `push_child` 完成。
- **Calendar 全受控**：`.value(Option<Date>)` 选中日、`.month((年, 月))` 显示月、`.mode(CalendarMode)` 月 / 年视图，
  三个回调都要接，否则对应操作不生效；年视图点击迷你月历跳回月视图。
- **Transfer 双集合受控**：`target_keys()` 右栏集合、`selected()` 当前勾选集合，
  `on_change(&Vec<SharedString>)` 回写右栏、`on_select(&Vec<SharedString>)` 回写勾选。
- **QRCode 容量限制**：纯 Rust 编码器支持 version 1~6（ECC L），最多 **134 字节**（UTF-8 中文约 44 字），
  超长数据会被截断；适合短文本 / URL，不支持 logo 与纠错级别切换。
- **Mentions 是有状态视图**：`cx.new(|cx| Mentions::new(cx).options(vec![…]))` 创建，
  读写同 TextInput（`text()` / `set_text()`）；输入 @ 后无空白即弹候选，点击候选项选中。
- **FloatButton 需要 relative 父容器**：按钮 absolute 定位，`bottom()` / `right()` 默认 24px。
- **Anchor / Upload / Countdown 约定**：Anchor 受控（`active()` + `on_click(&SharedString)` 3 参），
  滚动定位由调用方处理；Upload 仅点击触发（`on_click` 为 2 参 `Fn(&mut Window, &mut App)`），
  文件对话框 / 上传逻辑由调用方实现，不承诺原生拖放；Countdown 是 `Entity`，
  `cx.new(|cx| Countdown::after(Duration::from_secs(90), cx))` 创建后自动每秒刷新，
  `on_finish` 为 1 参 `Fn(&mut App)`（定时器上下文无 Window），归零触发一次后停止计时，
  实体释放后计时循环自动退出；重新开始 = 重建实体。
- **ImageView 的来源语义**：`ImageView::new("字符串")` 走 GPUI 资产系统（Embedded），**不是磁盘路径**；
  本地文件一律用 `ImageView::from_file(path)`。尺寸/布局通过 `Styled` 转发（`.w() / .h() / .flex()` 等），
  加载中和失败占位可用 `.loading(|| …)` / `.fallback(|| …)` 自定义。
- **VideoPlayer 是有状态视图，仅 Windows**：`cx.new(|cx| VideoPlayer::new(cx, path))` 创建后自动播放，
  render 中 `.child(video.clone())`；控制用 `video.update(cx, |v, cx| v.pause(cx))`（另有 `play` / `toggle` / `seek(secs)`），
  读取用 `position()` / `duration()`（秒）/ `is_playing()`。解码在后台线程（MF SourceReader），
  帧经通道泵回 UI 线程；播完自动停止，再 `play()` 从头重播。非 Windows 平台该模块不编译。
- **FilePicker 的回调是例外**：对话框结果在异步任务中回传，`on_pick` 签名为
  `Fn(&Vec<PathBuf>, &mut App)`（**无 `&mut Window`**），不适用 `cx.listener`，
  用 `cx.entity()` 句柄在闭包内 `entity.update(cx, …)`；取消对话框不触发回调。
- **CommandPalette 的接入**：启动时 `command_palette::bind_keys(cx)` 注册 Ctrl+K；
  宿主视图挂 `.on_action(cx.listener(|this, _: &command_palette::Toggle, window, cx| …))`
  在动作中调用 `palette.update(cx, |p, cx| p.open(window, cx))`；
  面板实体渲染在滚动容器之外（内部 deferred 置顶），`on_select(&SharedString)` 回传选中命令。
- **Splitter / ContextMenu / Carousel 需要唯一 id**；Splitter 的 `ratio()` 受控，
  拖拽回调 `on_change(&f32)` 里回写比例；ContextMenu / Watermark 是包装器，
  `.child(任意内容)` 包裹后使用。
- **Sender / TextArea 提交模式**：TextArea 新增 `on_submit(Fn(&mut Window, &mut App))`；
  设置后 Enter 触发提交（不插换行）、Shift+Enter 换行；Sender 内部即用该模式实现回车发送。
  独立 TextArea 不设 `on_submit` 时行为不变（Enter 换行）。GPUI 的 action 在 bubble 阶段默认停止传播，
  容器 `on_key_down` 收不到已被 TextArea 绑定的 Enter，故回车提交必须走 action 层 `on_submit` 而非外层 `on_key_down`。
- **FpsMeter 性能监视**：`cx.new(|cx| FpsMeter::new(cx))` 创建后默认开启；连续刷新只 notify 本实体
  （`request_animation_frame` 仅重绘当前视图），不带动整页重绘；动画为时间基（delta-time），高刷新率下平滑；
  实际帧率上限由显示器 vsync 决定（120Hz 屏可达 ~120fps），`max_fps()` 仅为标称上限。
- **flex 横向溢出**：flex 子项默认 `min-width:auto` 不肯缩到 min-content 以下，长文本 / 宽控件会撑破父列；
  给可收缩列加 `.min_w(px(0.))`（外层滚动容器与内层 flex_1 列都要加），限宽气泡 / 卡片同理。
- 应用启动时调用一次 `gpui_widgets::text_input::bind_keys(cx)`，
  否则输入框的 Ctrl+A/C/V/X、方向键等不生效（`cmd` 在 Windows/Linux 自动映射为 Ctrl）；
  用 TextArea 还需 `text_area::bind_keys(cx)`（回车 / 上下方向键 / Shift 选择），
  用 CommandPalette 还需 `command_palette::bind_keys(cx)`。

## 平台与版本

- 已验证：Windows（DirectWrite + DirectX 后端）。macOS / Linux 由 GPUI 自身支持，未在本环境验证；
  仓库附 `.github/workflows/ci.yml` 的 **Linux（x11）编译门 + `cargo doc` 门**（推送到 git 远端后生效），
  本地验收门为 `bash harness/check.sh`（状态矩阵 + 截图基线 + 全量测试）。
- `VideoPlayer` 依赖 Windows Media Foundation，仅 Windows 编译（`#[cfg(windows)]`）；
  其他平台自动排除该控件，库其余部分不受影响。演示视频为 MJPEG AVI（`gen_assets` 生成，无需 ffmpeg）。
- **钉住 `gpui-pre 0.3.14`**（crates.io 公开版，Zed GPUI 快照重发；与官方 `gpui 0.2.2` 不能混用）。
  GPUI 处于 pre-1.0、破坏性变更频繁：
  **升级 gpui 必须同步验证本库兼容性**（全量 `harness/check.sh` + 截图基线逐组比对），
  不得只改版本号。每个发布版本对应同名 git tag（`v0.1.x`）便于回溯。
- **docs.rs 兼容**：上游传递依赖 `xattr 0.2.3`（gpui→gpui_http_client→zed-async-tar）引用
  `libc::ENOATTR`，而 libc ≥0.2.191 移除了该 linux 别名 ⇒ 文档构建 E0425。docs.rs 会忽略包内
  `Cargo.lock`、按最新版本全新解析，故本库在 `[target.'cfg(unix)'.dependencies]` 以
  `libc = "=0.2.189"` 精确钉版（cargo 全图 semver 统一到仍含别名的版本），仅 unix 生效、
  Windows 构建零影响。发布包仍随包携带 `Cargo.lock`，供 `cargo install --locked` 用户复现组合。
- Rust edition 2024，建议 Rust 1.89+。

## 许可

Apache-2.0