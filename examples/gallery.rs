//! GPUI 通用控件库展示窗口：集中演示所有控件的用法与交互。

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    App, Bounds, Context, Div, Entity, FocusHandle, Focusable, KeyBinding, Rgba, ScrollHandle,
    TitlebarOptions, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
#[cfg(windows)]
use gpui_widgets::VideoPlayer;
use gpui_widgets::button::{ButtonColor, ButtonVariant};
use gpui_widgets::code_editor::{
    DEMO_JS, DEMO_JSON, DEMO_PLAIN, DEMO_PYTHON, DEMO_RUST, DEMO_SHELL, DEMO_TOML,
};
use gpui_widgets::{
    Accordion,
    AccordionItem,
    Alert,
    Anchor,
    AnchorItem,
    Autocomplete,
    Avatar,
    Badge,
    BarChart,
    BarDatum,
    Breadcrumb,
    // AI 对话组件
    Bubble,
    BubbleVariant,
    Button,
    Calendar,
    CalendarMode,
    Card,
    Carousel,
    Cascader,
    CascaderNode,
    Checkbox,
    CodeBlock,
    CodeEditor,
    CodeLanguage,
    Col,
    ColorPicker,
    CommandBar,
    CommandPalette,
    Content,
    ContextMenu,
    ConversationItem,
    Conversations,
    CountBadge,
    Countdown,
    Date,
    DatePicker,
    DateRangePicker,
    Descriptions,
    Divider,
    Drawer,
    DrawerPlacement,
    DropZone,
    DropdownMenu,
    Empty,
    FilePicker,
    FloatButton,
    Footer,
    Form,
    FormItem,
    FpsMeter,
    FunnelChart,
    FunnelStage,
    GaugeChart,
    Header,
    Heatmap,
    HotkeyInput,
    ImageView,
    Kbd,
    LineChart,
    Link,
    List,
    ListItem,
    Markdown,
    Mentions,
    MenuEntry,
    MenuItem,
    Modal,
    MultiSelect,
    NavMenu,
    NotificationItem,
    NotificationStack,
    NumberInput,
    Pagination,
    PaletteItem,
    Paragraph,
    PasswordInput,
    PieChart,
    PieDatum,
    Popconfirm,
    Popover,
    ProgressBar,
    PromptItem,
    Prompts,
    QRCode,
    RadarChart,
    RadarSeries,
    Radio,
    Rating,
    Result,
    ResultStatus,
    RingProgress,
    Row,
    Rule,
    ScatterChart,
    ScatterPoint,
    ScatterSeries,
    Scrollbar,
    SearchInput,
    Segmented,
    Select,
    Sender,
    Sider,
    Skeleton,
    Slider,
    Space,
    Sparkline,
    Spinner,
    Splitter,
    Statistic,
    Stepper,
    Steps,
    Switch,
    Table,
    Tabs,
    Tag,
    TagColor,
    TagInput,
    Terminal,
    Text,
    TextArea,
    TextInput,
    Theme,
    ThoughtChain,
    ThoughtNode,
    ThoughtStatus,
    TimePicker,
    Timeline,
    TimelineItem,
    Title,
    TitleBar,
    ToastItem,
    ToastStack,
    Tooltip,
    Tour,
    TourStep,
    Transfer,
    TransferItem,
    TransferManager,
    TransferStatus,
    TransferTask,
    Tree,
    TreeDrop,
    TreeNode,
    TreeSelect,
    Trend,
    Upload,
    UploadFile,
    UploadList,
    UploadStatus,
    Watermark,
    Welcome,
    find_node_mut,
    misc::{caption, card, field_row, section_title},
    take_node,
    theme,
    transparent_titlebar_options,
    validate,
};

gpui::actions!(gallery, [Quit]);

const PLANS: [&str; 3] = ["免费版", "专业版", "企业版"];
const TABS: [&str; 3] = ["概览", "详情", "设置"];
const CITIES: [&str; 4] = ["北京", "上海", "广州", "深圳"];
const SKILLS: [&str; 5] = ["Rust", "GPUI", "桌面端", "跨平台", "开源"];
const CRUMBS: [&str; 3] = ["首页", "产品中心", "控件库"];

const TERM_PROMPT: &str = "\x1b[1;32muser@gpui\x1b[0m:\x1b[1;34m~\x1b[0m$ ";
const TERM_BANNER: &str = "\x1b[1;36mGPUI Terminal\x1b[0m · VT/ANSI native terminal\r\n\x1b[90m16/256/truecolor · scrollback · selection · search · links\x1b[0m\r\nType \x1b[1;33mhelp\x1b[0m for commands. Try \x1b[1;32mls\x1b[0m, \x1b[1;35mcolor\x1b[0m, \x1b[1;31mclear\x1b[0m.\r\n\r\n";

/// 变体 × 色彩矩阵：与 antd 参照面板同一组格，
/// 静止 / hover / 按下 / 禁用全部由 `theme::Tokens` 派生，写这里只负责摆位置。
fn variant_grid() -> Div {
    const COLORS: [(&str, ButtonColor); 3] = [
        ("default", ButtonColor::Default),
        ("primary", ButtonColor::Primary),
        ("danger", ButtonColor::Danger),
    ];
    const VARIANTS: [(&str, ButtonVariant); 6] = [
        ("solid", ButtonVariant::Solid),
        ("outlined", ButtonVariant::Outlined),
        ("dashed", ButtonVariant::Dashed),
        ("filled", ButtonVariant::Filled),
        ("text", ButtonVariant::Text),
        ("link", ButtonVariant::Link),
    ];

    div()
        .flex()
        .flex_col()
        .gap_2()
        .children(COLORS.into_iter().map(|(color_name, color)| {
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap_2()
                .child(caption(color_name.to_string()))
                .children(VARIANTS.into_iter().map(|(variant_name, variant)| {
                    Button::new(variant_name)
                        .id(format!("{color_name}-{variant_name}"))
                        .color(color)
                        .variant(variant)
                }))
                .child(
                    Button::new("disabled")
                        .id(format!("{color_name}-disabled"))
                        .color(color)
                        .variant(ButtonVariant::Outlined)
                        .disabled(true),
                )
        }))
}

/// 演示用回显 shell：将命令映射为 ANSI 输出文本。
fn shell_output(cmd: &str) -> String {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return String::new();
    }
    let mut parts = cmd.splitn(2, ' ');
    let name = parts.next().unwrap_or("");
    let arg = parts.next().unwrap_or("");
    let mut out = String::new();
    match name {
        "help" => out.push_str("commands: help ls color pwd date echo <text> clear sel find <w> next prev scroll\r\n"),
        "ls" => out.push_str("\x1b[1;34mprojects\x1b[0m  \x1b[1;34mdocs\x1b[0m  \x1b[32mmain.rs\x1b[0m  \x1b[33mREADME.md\x1b[0m  \x1b[36mCargo.toml\x1b[0m\r\n"),
        "color" => {
            for i in 0..8 {
                out.push_str(&format!("\x1b[{}m██\x1b[0m", 30 + i));
            }
            for i in 0..8 {
                out.push_str(&format!("\x1b[{}m██\x1b[0m", 90 + i));
            }
            out.push_str("\r\n\x1b[38;5;208m256-orange\x1b[0m \x1b[38;2;255;105;180mtruecolor-pink\x1b[0m \x1b[1mbold\x1b[0m \x1b[4munderline\x1b[0m \x1b[7minverse\x1b[0m\r\n");
        }
        "pwd" => out.push_str("/home/user\r\n"),
        "date" => out.push_str("Sat Nov 14 12:00:00 2026\r\n"),
        "echo" => out.push_str(&format!("{arg}\r\n")),
        _ => out.push_str(&format!("\x1b[31mcommand not found: {name}\x1b[0m\r\n")),
    }
    out
}

fn default_tags() -> Vec<String> {
    ["Rust", "GPUI", "桌面端"].map(String::from).to_vec()
}

fn default_input_tags() -> Vec<gpui::SharedString> {
    vec!["Rust".into(), "GPUI".into()]
}

fn cascader_options() -> Vec<CascaderNode> {
    vec![
        CascaderNode::new("zj", "浙江")
            .child(CascaderNode::new("hz", "杭州").child(CascaderNode::new("xh", "西湖")))
            .child(CascaderNode::new("nb", "宁波").child(CascaderNode::new("bl", "北仑"))),
        CascaderNode::new("js", "江苏")
            .child(CascaderNode::new("nj", "南京").child(CascaderNode::new("xw", "玄武")))
            .child(CascaderNode::new("sz", "苏州").child(CascaderNode::new("gy", "工业园"))),
    ]
}

fn transfer_items() -> Vec<TransferItem> {
    [
        "需求评审",
        "交互设计",
        "视觉设计",
        "前端开发",
        "后端开发",
        "测试验收",
    ]
    .iter()
    .enumerate()
    .map(|(ix, label)| TransferItem::new(format!("t{ix}"), *label))
    .collect()
}

/// 传输任务管理器的演示数据：覆盖全部状态
fn demo_transfers() -> Vec<TransferTask> {
    vec![
        TransferTask::upload("tm-1", "项目源码打包.zip")
            .status(TransferStatus::Transferring)
            .progress(0.42)
            .size(18 * 1024 * 1024)
            .speed(2.4 * 1024. * 1024.),
        TransferTask::download("tm-2", "服务器日志-2026-11-14.log")
            .status(TransferStatus::Transferring)
            .progress(0.71)
            .size(6 * 1024 * 1024)
            .speed(1.1 * 1024. * 1024.),
        TransferTask::upload("tm-3", "设计稿.fig")
            .status(TransferStatus::Queued)
            .size(42 * 1024 * 1024),
        TransferTask::download("tm-4", "数据集样本.csv")
            .status(TransferStatus::Paused)
            .progress(0.18)
            .size(120 * 1024 * 1024),
        TransferTask::upload("tm-5", "旧版备份.tar.gz")
            .status(TransferStatus::Done)
            .progress(1.)
            .size(88 * 1024 * 1024),
        TransferTask::upload("tm-6", "演示视频.mp4")
            .status(TransferStatus::Failed)
            .progress(0.35)
            .size(512 * 1024 * 1024)
            .error("远端存储空间不足"),
    ]
}

/// 文件大小格式化（B / KB / MB / GB）
fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.;
    let value = bytes as f64;
    if value >= KB * KB * KB {
        format!("{:.1} GB", value / (KB * KB * KB))
    } else if value >= KB * KB {
        format!("{:.1} MB", value / (KB * KB))
    } else if value >= KB {
        format!("{:.1} KB", value / KB)
    } else {
        format!("{bytes} B")
    }
}

/// 代码编辑器演示：各语言的示例源码
fn demo_source(lang: CodeLanguage) -> &'static str {
    match lang {
        CodeLanguage::Plain => DEMO_PLAIN,
        CodeLanguage::Rust => DEMO_RUST,
        CodeLanguage::JavaScript => DEMO_JS,
        CodeLanguage::Json => DEMO_JSON,
        CodeLanguage::Toml => DEMO_TOML,
        CodeLanguage::Python => DEMO_PYTHON,
        CodeLanguage::Shell => DEMO_SHELL,
    }
}

fn ts_nodes() -> Vec<TreeNode> {
    vec![
        TreeNode::new("tech", "技术部")
            .child(TreeNode::new("fe", "前端组"))
            .child(TreeNode::new("be", "后端组")),
        TreeNode::new("product", "产品部")
            .child(TreeNode::new("pm", "产品组"))
            .child(TreeNode::new("design", "设计组")),
    ]
}

fn default_tree_nodes() -> Vec<TreeNode> {
    vec![
        TreeNode::new("src", "src")
            .child(TreeNode::new("lib", "lib.rs"))
            .child(TreeNode::new("select", "select.rs"))
            .child(TreeNode::new("popup", "popup.rs")),
        TreeNode::new("examples", "examples")
            .child(TreeNode::new("gallery", "gallery.rs"))
            .child(TreeNode::new("minimal", "minimal.rs")),
        TreeNode::new("readme", "README.md"),
    ]
}

fn tour_steps() -> Vec<TourStep> {
    vec![
        TourStep::new(
            "欢迎使用控件库",
            "这里集中演示了 84 个通用控件，滚动浏览各分区。",
        ),
        TourStep::new(
            "交互状态",
            "所有控件的状态都由父视图持有，点击「重置全部」可恢复初始状态。",
        ),
        TourStep::new("开始探索", "试试下方的级联选择、穿梭框和 @ 提及输入框吧！"),
    ]
}

fn default_uploads() -> Vec<UploadFile> {
    vec![
        UploadFile::new("设计稿.fig").status(UploadStatus::Done),
        UploadFile::new("接口文档.pdf")
            .status(UploadStatus::Uploading)
            .progress(0.6),
        UploadFile::new("演示视频.mp4").status(UploadStatus::Error),
    ]
}

fn grid_cell(label: &str) -> Div {
    div()
        .flex()
        .items_center()
        .justify_center()
        .h(px(32.))
        .rounded_md()
        .bg(theme::accent_soft())
        .text_color(theme::accent())
        .text_xs()
        .child(label.to_string())
}

/// 控件分类（左侧导航口径，沿用控件参考手册 + AI 对话 + 综合演示）。
/// 每项：(名称, 图标, 描述)。
const CATEGORIES: [(&str, &str, &str); 10] = [
    ("通用", "🧩", "Button 按钮各形态与点击交互"),
    (
        "布局",
        "📐",
        "Row·Col / Header·Sider·Content·Footer / Space / Splitter / Scrollbar",
    ),
    (
        "导航",
        "🧭",
        "Tabs / Steps / Breadcrumb / Pagination / 菜单 / Anchor / NavMenu / FloatButton",
    ),
    (
        "数据录入",
        "✍",
        "输入框 / 选择器 / 日期时间 / 滑杆 / 开关 / 表单 / 上传 / 穿梭框",
    ),
    (
        "数据展示",
        "📊",
        "表格 / 列表 / 树 / 头像 / 徽标 / 卡片 / 媒体 / 统计 / 图表",
    ),
    (
        "反馈",
        "🔔",
        "警告 / 弹窗 / 抽屉 / 通知 / 轻提示 / 进度 / 结果 / 骨架屏 / 加载",
    ),
    (
        "AI 对话",
        "🤖",
        "Welcome / Prompts / Bubble / Sender / Conversations / ThoughtChain",
    ),
    ("辅助与主题", "🎨", "主题配置 / 语义色板 / misc 辅助函数"),
    (
        "综合演示",
        "🎬",
        "属性对齐 / 新增控件 / 轮播 / 水印 等跨类演示",
    ),
    (
        "终端与开发",
        "💻",
        "Terminal / CodeEditor / CommandBar / TransferManager / TitleBar",
    ),
];

struct Gallery {
    click_count: usize,
    agreed: bool,
    newsletter: bool,
    notifications: bool,
    plan: usize,
    volume: f32,
    name_input: Entity<TextInput>,
    sample_name_input: Entity<TextInput>,
    tab_ix: usize,
    seg_ix: usize,
    city: usize,
    select_open: bool,
    skills: Vec<usize>,
    skills_open: bool,
    modal_open: bool,
    step_val: f32,
    tags: Vec<String>,
    acc_open: Vec<usize>,
    page: usize,
    score: usize,
    table_row: Option<usize>,
    table_sort: Option<(usize, bool)>,
    table_checked: Vec<usize>,
    table_page: usize,
    table_expanded: Vec<usize>,
    range_sel: Vec<usize>,
    close_tabs: Vec<gpui::SharedString>,
    close_tab_ix: usize,
    drag_tabs: Vec<gpui::SharedString>,
    drag_active: usize,
    transfers: Vec<TransferTask>,
    search_city: Option<usize>,
    search_city_open: bool,
    marked_val: f32,
    vert_val: f32,
    drawer_left: bool,
    clear_input: Entity<TextInput>,
    error_input: Entity<TextInput>,
    disabled_input: Entity<TextInput>,
    readonly_input: Entity<TextInput>,
    form_name: Entity<TextInput>,
    form_email: Entity<TextInput>,
    form_query: Entity<TextInput>,
    form_name_error: Option<gpui::SharedString>,
    form_email_error: Option<gpui::SharedString>,
    form_ok: bool,
    tree_expanded: Vec<String>,
    tree_selected: Option<String>,
    tree_nodes: Vec<TreeNode>,
    tree_checked: Vec<gpui::SharedString>,
    menu_open: bool,
    popover_open: bool,
    date: Option<Date>,
    date_open: bool,
    date_month: Option<(i32, u32)>,
    range: Option<(Date, Date)>,
    range_open: bool,
    range_month: Option<(i32, u32)>,
    color: Rgba,
    color_open: bool,
    dark_mode: bool,
    theme_accent: Rgba,
    theme_accent_open: bool,
    drawer_open: bool,
    toasts: Vec<ToastItem>,
    toast_count: usize,
    password: Entity<PasswordInput>,
    search: Entity<SearchInput>,
    number: Entity<NumberInput>,
    tag_input: Entity<TagInput>,
    autocomplete: Entity<Autocomplete>,
    text_area: Entity<TextArea>,
    palette: Entity<CommandPalette>,
    split_ratio: f32,
    vsplit_ratio: f32,
    step_ix: usize,
    carousel_ix: usize,
    last_search: String,
    number_val: Option<f64>,
    input_tags: Vec<gpui::SharedString>,
    auto_pick: String,
    picked: String,
    last_command: String,
    // antd 导航布局
    nav_selected: Option<gpui::SharedString>,
    nav_expanded: Vec<gpui::SharedString>,
    // antd 数据展示
    cal_month: (i32, u32),
    cal_value: Option<Date>,
    cal_mode: CalendarMode,
    badge_count: u32,
    // antd 数据录入
    cascader_path: Vec<gpui::SharedString>,
    cascader_open: bool,
    time_value: Option<(u32, u32, u32)>,
    time_open: bool,
    ts_value: Option<gpui::SharedString>,
    ts_expanded: Vec<gpui::SharedString>,
    ts_open: bool,
    transfer_right: Vec<gpui::SharedString>,
    transfer_checked: Vec<gpui::SharedString>,
    upload_files: Vec<UploadFile>,
    // antd 反馈交互
    popconfirm_open: bool,
    notifs: Vec<NotificationItem>,
    tour_open: bool,
    tour_step: usize,
    mentions: Entity<Mentions>,
    mention_pick: String,
    // 属性对齐演示
    align_tab: usize,
    align_seg: usize,
    align_switch: bool,
    align_page: usize,
    // 新增控件演示
    anchor_key: gpui::SharedString,
    countdown: Entity<Countdown>,
    #[cfg(windows)]
    video: Entity<VideoPlayer>,
    // AI 对话组件演示
    ai_sender: Entity<Sender>,
    ai_messages: Vec<(bool, String)>,
    ai_conversation: gpui::SharedString,
    ai_thought_expanded: Vec<usize>,
    // 性能监视器
    fps_meter: Entity<FpsMeter>,
    // 终端演示
    terminal: Entity<Terminal>,
    term_search: Entity<TextInput>,
    term_title: String,
    // 快捷键录入演示
    hotkey_save: Entity<HotkeyInput>,
    hotkey_palette: Entity<HotkeyInput>,
    hotkey_disabled: Entity<HotkeyInput>,
    hotkey_log: String,
    // 命令输入条演示
    cmd_bar: Entity<CommandBar>,
    cmd_runs: Vec<String>,
    // 代码编辑器演示
    code_editor: Entity<TextArea>,
    code_lang: CodeLanguage,
    // 外部文件拖放演示
    drop_files: Vec<(String, u64)>,
    drop_rejected: usize,
    // 左侧分类导航当前选中类
    active_category: usize,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
}

impl Focusable for Gallery {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Gallery {
    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.click_count = 0;
        self.agreed = false;
        self.newsletter = false;
        self.notifications = true;
        self.plan = 0;
        self.volume = 30.;
        self.tab_ix = 0;
        self.seg_ix = 0;
        self.city = 0;
        self.select_open = false;
        self.skills = vec![0, 1];
        self.skills_open = false;
        self.modal_open = false;
        self.step_val = 1.;
        self.tags = default_tags();
        self.acc_open = vec![0];
        self.page = 1;
        self.score = 3;
        self.table_row = None;
        self.table_sort = None;
        self.table_checked.clear();
        self.table_page = 1;
        self.table_expanded.clear();
        self.range_sel.clear();
        self.close_tabs = vec!["文件 1".into(), "文件 2".into(), "文件 3".into()];
        self.close_tab_ix = 0;
        self.drag_tabs = vec![
            "文件 1".into(),
            "文件 2".into(),
            "文件 3".into(),
            "文件 4".into(),
        ];
        self.drag_active = 0;
        self.transfers = demo_transfers();
        self.code_lang = CodeLanguage::Rust;
        self.code_editor.update(cx, |area, cx| {
            CodeEditor::set_language(area, CodeLanguage::Rust, cx);
            area.set_text_silent(DEMO_RUST, cx);
        });
        self.drop_files.clear();
        self.drop_rejected = 0;
        self.search_city = None;
        self.search_city_open = false;
        self.marked_val = 50.;
        self.vert_val = 30.;
        self.drawer_left = false;
        self.tree_expanded = vec!["src".to_string()];
        self.tree_selected = None;
        self.tree_nodes = default_tree_nodes();
        self.tree_checked.clear();
        self.menu_open = false;
        self.popover_open = false;
        self.date = None;
        self.date_open = false;
        self.date_month = None;
        self.range = None;
        self.range_open = false;
        self.range_month = None;
        self.color = rgb(0x3b82f6);
        self.color_open = false;
        self.dark_mode = false;
        self.theme_accent = rgb(0x3b82f6);
        self.theme_accent_open = false;
        theme::set_theme(Theme::light());
        self.drawer_open = false;
        self.toasts.clear();
        self.split_ratio = 0.5;
        self.vsplit_ratio = 0.5;
        self.step_ix = 0;
        self.carousel_ix = 0;
        self.last_search.clear();
        self.number_val = None;
        self.input_tags = default_input_tags();
        self.auto_pick.clear();
        self.picked.clear();
        self.last_command.clear();
        self.nav_selected = None;
        self.nav_expanded = vec!["settings".into()];
        self.cal_month = (2026, 11);
        self.cal_value = None;
        self.cal_mode = CalendarMode::Month;
        self.badge_count = 5;
        self.cascader_path.clear();
        self.cascader_open = false;
        self.time_value = None;
        self.time_open = false;
        self.ts_value = None;
        self.ts_expanded = vec!["tech".into()];
        self.ts_open = false;
        self.transfer_right.clear();
        self.transfer_checked.clear();
        self.upload_files = default_uploads();
        self.popconfirm_open = false;
        self.notifs.clear();
        self.tour_open = false;
        self.tour_step = 0;
        self.mention_pick.clear();
        self.mentions
            .update(cx, |mentions, cx| mentions.set_text("", window, cx));
        self.name_input
            .update(cx, |input, cx| input.set_text("", window, cx));
        self.form_name
            .update(cx, |input, cx| input.set_text("", window, cx));
        self.form_email
            .update(cx, |input, cx| input.set_text("", window, cx));
        self.form_query
            .update(cx, |input, cx| input.set_text("", window, cx));
        self.form_name_error = None;
        self.form_email_error = None;
        self.form_ok = false;
        self.clear_input
            .update(cx, |input, cx| input.set_text("", window, cx));
        self.password
            .update(cx, |input, cx| input.set_text("", window, cx));
        self.search
            .update(cx, |input, cx| input.set_text("", window, cx));
        self.number
            .update(cx, |input, cx| input.set_value(None, window, cx));
        self.text_area
            .update(cx, |input, cx| input.set_text("", window, cx));
        self.palette.update(cx, |palette, cx| palette.close(cx));
        // TagInput 无程序化清空接口，重建实体恢复初始标签
        let entity = cx.entity();
        self.tag_input = cx.new(|cx| {
            TagInput::new(cx)
                .placeholder("输入后回车添加")
                .initial_tags(default_input_tags())
                .on_change(move |tags, _, cx| {
                    let tags = tags.clone();
                    let _ = entity.update(cx, |this, cx| {
                        this.input_tags = tags;
                        cx.notify();
                    });
                })
        });
        self.align_tab = 0;
        self.align_seg = 0;
        self.align_switch = true;
        self.align_page = 1;
        self.anchor_key = "basic".into();
        self.countdown = Self::new_countdown(cx);
        cx.notify();
    }

    /// 90 秒倒计时：归零弹 Toast（on_finish 只有 &mut App，用实体句柄回写）
    fn new_countdown(cx: &mut Context<Self>) -> Entity<Countdown> {
        let entity = cx.entity();
        cx.new(|cx| {
            Countdown::after(Duration::from_secs(90), cx).on_finish(move |cx| {
                let _ = entity.update(cx, |this, cx| {
                    this.push_toast(ToastItem::new("倒计时结束").success(), cx);
                });
            })
        })
    }

    fn theme_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("主题 Theme"))
            .child(caption(
                "全局主题配置：明暗切换 + 品牌主色（悬停 / 按下 / 柔和底自动派生），所有控件同步生效",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_6()
                    .child(
                        Switch::new(self.dark_mode)
                            .label("深色模式")
                            .on_change(cx.listener(|this, on, _, cx| {
                                this.dark_mode = *on;
                                let base = if *on { Theme::dark() } else { Theme::light() };
                                theme::set_theme(base.with_accent(this.theme_accent));
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("恢复默认主题").secondary().on_click(cx.listener(
                            |this, _, _, cx| {
                                this.dark_mode = false;
                                this.theme_accent = rgb(0x3b82f6);
                                theme::set_theme(Theme::light());
                                cx.notify();
                            },
                        )),
                    ),
            )
            .child(
                field_row("品牌主色").child(
                    div().flex_1().child(
                        ColorPicker::new("theme-accent", self.theme_accent)
                            .open(self.theme_accent_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.theme_accent_open = *open;
                                cx.notify();
                            }))
                            .on_change(cx.listener(|this, color, _, cx| {
                                this.theme_accent = *color;
                                theme::set_accent(*color);
                                cx.notify();
                            })),
                    ),
                ),
            )
    }

    fn align_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("属性对齐 · 导航与选择"))
            .child(caption(
                "对照 antd 补齐：Button 形状 / Steps 变体 / Pagination 简洁 / Tabs 居中 / Segmented 尺寸 / Switch 轨道文字",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(Button::new("胶囊按钮").primary().round())
                    .child(Button::new("＋").secondary().circle())
                    .child(Button::new("✓").primary().circle().small()),
            )
            .child(Divider::new())
            .child(caption("Steps：small + descriptions"))
            .child(
                Steps::new(vec!["草稿".into(), "审核".into(), "发布".into()])
                    .id("align-steps-small")
                    .current(1)
                    .small()
                    .descriptions(vec!["保存草稿".into(), "人工复核".into(), "对外可见".into()]),
            )
            .child(Divider::new())
            .child(caption("Steps：progress_dot + vertical"))
            .child(
                Steps::new(vec!["需求".into(), "设计".into(), "开发".into(), "验收".into()])
                    .id("align-steps-dot")
                    .current(1)
                    .progress_dot()
                    .vertical(),
            )
            .child(Divider::new())
            .child(caption("Pagination：simple + small"))
            .child(
                Pagination::new("align-page", self.align_page, 8)
                    .simple()
                    .small()
                    .on_change(cx.listener(|this, page, _, cx| {
                        this.align_page = *page;
                        cx.notify();
                    })),
            )
            .child(Divider::new())
            .child(caption("Tabs：centered"))
            .child(
                Tabs::new(TABS, self.align_tab)
                    .id("align-tabs")
                    .centered()
                    .on_select(cx.listener(|this, ix, _, cx| {
                        this.align_tab = *ix;
                        cx.notify();
                    })),
            )
            .child(Divider::new())
            .child(caption("Segmented：small / large / block（三行联动同一状态）"))
            .child(
                Segmented::new(["日", "周", "月"], self.align_seg)
                    .id("align-segmented-small")
                    .small()
                    .on_select(cx.listener(|this, ix, _, cx| {
                        this.align_seg = *ix;
                        cx.notify();
                    })),
            )
            .child(
                Segmented::new(["日", "周", "月"], self.align_seg)
                    .id("align-segmented-large")
                    .large()
                    .on_select(cx.listener(|this, ix, _, cx| {
                        this.align_seg = *ix;
                        cx.notify();
                    })),
            )
            .child(
                Segmented::new(["日", "周", "月"], self.align_seg)
                    .id("align-segmented-block")
                    .block()
                    .on_select(cx.listener(|this, ix, _, cx| {
                        this.align_seg = *ix;
                        cx.notify();
                    })),
            )
            .child(Divider::new())
            .child(
                Switch::new(self.align_switch)
                    .label("轨道内文字")
                    .text("开", "关")
                    .on_change(cx.listener(|this, on, _, cx| {
                        this.align_switch = *on;
                        cx.notify();
                    })),
            )
    }

    fn align_display_section(&self) -> Div {
        card()
            .child(section_title("属性对齐 · 数据展示与反馈"))
            .child(caption(
                "Timeline 交替/幽灵节点 · Descriptions 垂直 · Table 边框/无表头 · Progress 分段 · Alert 通栏 · Spinner 提示 · Avatar 方形 · Card 悬停",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_6()
                    .child(
                        div().flex_1().child(
                            Timeline::new(vec![
                                TimelineItem::new("需求评审").time("周一"),
                                TimelineItem::new("视觉设计").time("周三"),
                                TimelineItem::new("开发联调")
                                    .time("周五")
                                    .color(theme::success()),
                            ])
                            .alternate()
                            .pending("测试中…"),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            Descriptions::new("垂直布局")
                                .column(2)
                                .vertical()
                                .item("姓名", "张三")
                                .item("电话", "13800000000")
                                .item("城市", "杭州")
                                .item("状态", "活跃"),
                        ),
                    ),
            )
            .child(
                Descriptions::untitled()
                    .column(3)
                    .vertical()
                    .bordered()
                    .item("订单号", "202611130001")
                    .item("金额", "¥ 199.00")
                    .item("支付方式", "余额"),
            )
            .child(Divider::new())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_6()
                    .child(
                        div().flex_1().child(
                            Table::new(
                                "align-table",
                                ["姓名", "城市", "技能"],
                                vec![
                                    vec!["张三".into(), "北京".into(), "Rust".into()],
                                    vec!["李四".into(), "上海".into(), "GPUI".into()],
                                    vec!["王五".into(), "深圳".into(), "设计".into()],
                                ],
                            )
                            .bordered()
                            .striped(true),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            Table::new(
                                "align-table-nohead",
                                ["姓名", "城市", "技能"],
                                vec![
                                    vec!["赵六".into(), "广州".into(), "产品".into()],
                                    vec!["钱七".into(), "杭州".into(), "运营".into()],
                                ],
                            )
                            .bordered()
                            .show_header(false),
                        ),
                    ),
            )
            .child(Divider::new())
            .child(caption("ProgressBar：steps(10) 分段（与上方音量滑杆联动）"))
            .child(ProgressBar::new(self.volume / 100.).steps(10))
            .child(Alert::new("通栏警告：剩余空间不足 10%").warning().banner())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_6()
                    .child(Spinner::new().tip("加载中…"))
                    .child(Avatar::new("张").square())
                    .child(Avatar::new("李").square().size(px(40.))),
            )
            .child(
                Card::new("悬停卡片")
                    .hoverable()
                    .child("鼠标移入试试：hoverable 让卡片浮起（阴影加深）"),
            )
    }

    fn new_controls_section(&self, cx: &mut Context<Self>) -> Div {
        let entity = cx.entity();
        card()
            .child(section_title("新增控件 Anchor / Upload / Countdown"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_6()
                    .child(
                        div().w(px(200.)).child(
                            Anchor::new(vec![
                                AnchorItem::new("basic", "基本用法"),
                                AnchorItem::new("props", "属性说明")
                                    .child(AnchorItem::new("props-a", "受控 active"))
                                    .child(AnchorItem::new("props-b", "点击回调")),
                                AnchorItem::new("faq", "常见问题"),
                            ])
                            .active(self.anchor_key.clone())
                            .on_click(cx.listener(
                                |this, key: &gpui::SharedString, _, cx| {
                                    this.anchor_key = key.clone();
                                    cx.notify();
                                },
                            )),
                        ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(caption(format!(
                                "当前锚点：{}（滚动定位由调用方处理）",
                                self.anchor_key
                            )))
                            .child(
                                Upload::new()
                                    .text("点击上传")
                                    .hint("支持 png / jpg / pdf，单文件不超过 10MB")
                                    .on_click(move |_, cx| {
                                        let _ = entity.update(cx, |this, cx| {
                                            this.push_toast(
                                                ToastItem::new(
                                                    "上传触发：请在回调中打开文件对话框",
                                                )
                                                .success(),
                                                cx,
                                            );
                                        });
                                    }),
                            )
                            .child(
                                Upload::new()
                                    .id("upload-disabled")
                                    .text("禁用状态")
                                    .disabled(true),
                            ),
                    ),
            )
            .child(Divider::new())
            .child(caption(
                "Countdown：每秒按系统时钟重算，归零弹出 Toast 并停止计时",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_4()
                    .child(self.countdown.clone())
                    .child(
                        Button::new("重新开始")
                            .secondary()
                            .small()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.countdown = Self::new_countdown(cx);
                                cx.notify();
                            })),
                    ),
            )
    }

    fn buttons_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("按钮 Button"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_3()
                    .child(Button::new("主要按钮").primary().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.click_count += 1;
                            cx.notify();
                        },
                    )))
                    .child(Button::new("次要按钮").secondary())
                    .child(Button::new("幽灵按钮").ghost())
                    .child(Button::new("危险按钮").danger())
                    .child(Button::new("小按钮").primary().small())
                    .child(Button::new("大按钮").primary().large())
                    .child(Button::new("加载中").primary().loading(true))
                    .child(Button::new("图标按钮").secondary().icon("＋"))
                    .child(Button::new("禁用按钮").secondary().disabled(true))
                    .child(
                        Button::new("重置全部")
                            .secondary()
                            .on_click(cx.listener(|this, _, window, cx| this.reset(window, cx))),
                    ),
            )
            .child(Button::new("块级按钮（占满整行）").secondary().block())
            .child(caption(format!("主要按钮已点击 {} 次", self.click_count)))
            .child(variant_grid())
            .child(caption(
                "变体 × 色彩矩阵：hover / 按下 / 禁用的每一格都由 theme::Tokens 派生",
            ))
    }

    fn selection_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("选择控件 Checkbox / Switch / Radio"))
            .child(
                Checkbox::new("我已阅读并同意服务条款", self.agreed).on_change(cx.listener(
                    |this, checked, _, cx| {
                        this.agreed = *checked;
                        cx.notify();
                    },
                )),
            )
            .child(Checkbox::new("禁用且选中", true).disabled(true))
            .child(Checkbox::new("半选状态（父级全选）", false).indeterminate(true))
            .child(Divider::new())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_6()
                    .child(Switch::new(self.notifications).label("启用通知").on_change(
                        cx.listener(|this, on, _, cx| {
                            this.notifications = *on;
                            cx.notify();
                        }),
                    ))
                    .child(
                        Switch::new(self.newsletter)
                            .label("订阅周报")
                            .on_change(cx.listener(|this, on, _, cx| {
                                this.newsletter = *on;
                                cx.notify();
                            })),
                    )
                    .child(Switch::new(true).label("小尺寸").small())
                    .child(Switch::new(true).label("加载中").loading(true)),
            )
            .child(Divider::new())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_6()
                    .children(PLANS.iter().enumerate().map(|(ix, label)| {
                        Radio::new(*label, self.plan == ix).on_select(cx.listener(
                            move |this, _, _, cx| {
                                this.plan = ix;
                                cx.notify();
                            },
                        ))
                    })),
            )
    }

    fn text_section(&self, cx: &mut Context<Self>) -> Div {
        let name = self.name_input.read(cx).text().to_string();
        card()
            .child(section_title("文本输入 TextInput"))
            .child(caption(
                "支持光标 / 选区 / 剪贴板 / 中文输入法，试试 Ctrl+A、Ctrl+C、Ctrl+V",
            ))
            .child(field_row("姓名").child(div().flex_1().child(self.name_input.clone())))
            .child(field_row("前后缀").child(div().flex_1().child(self.clear_input.clone())))
            .child(caption(
                "上行：前缀 https:// + 后缀 .com，最多 10 字符，有值时点 ✕ 清空",
            ))
            .child(field_row("错误态").child(div().flex_1().child(self.error_input.clone())))
            .child(field_row("禁用").child(div().flex_1().child(self.disabled_input.clone())))
            .child(field_row("只读").child(div().flex_1().child(self.readonly_input.clone())))
            .child(caption(if name.is_empty() {
                "等待输入…".to_string()
            } else {
                format!("你好，{name}！")
            }))
    }

    fn slider_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("滑杆 Slider / 进度条 ProgressBar"))
            .child(
                field_row("音量")
                    .child(
                        div().flex_1().child(
                            Slider::new("volume", self.volume)
                                .label("音量")
                                .range(0., 100.)
                                .step(1.)
                                .on_change(cx.listener(|this, value, _, cx| {
                                    this.volume = *value;
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(
                        div()
                            .w(px(40.))
                            .text_sm()
                            .text_color(theme::text_muted())
                            .child(format!("{:.0}", self.volume)),
                    ),
            )
            .child(ProgressBar::new(self.volume / 100.))
            .child(caption("拖动滑杆，进度条实时联动"))
            .child(Divider::new())
            .child(
                field_row("刻度").child(
                    div().flex_1().child(
                        Slider::new("marked", self.marked_val)
                            .label("刻度")
                            .range(0., 100.)
                            .step(25.)
                            .marks([
                                (0., "0"),
                                (25., "25"),
                                (50., "50"),
                                (75., "75"),
                                (100., "100"),
                            ])
                            .on_change(cx.listener(|this, value, _, cx| {
                                this.marked_val = *value;
                                cx.notify();
                            })),
                    ),
                ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_4()
                    .child(
                        div().h(px(120.)).child(
                            Slider::new("vertical", self.vert_val)
                                .label("垂直滑杆")
                                .range(0., 100.)
                                .vertical()
                                .on_change(cx.listener(|this, value, _, cx| {
                                    this.vert_val = *value;
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(caption(format!("垂直滑杆：{:.0}", self.vert_val))),
            )
    }

    fn badge_section(&self) -> Div {
        card()
            .child(section_title("徽章 Badge"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_2()
                    .child(Badge::new("默认"))
                    .child(Badge::new("信息").info())
                    .child(Badge::new("运行中").success())
                    .child(Badge::new("警告").warning())
                    .child(Badge::new("错误").danger())
                    .child(Badge::new("自定义色").color(rgb(0x722ed1).into())),
            )
            .child(Divider::new())
            .child(caption(format!(
                "当前状态：条款 {} · 通知 {} · 周报 {} · 方案 {} · 音量 {:.0}",
                if self.agreed {
                    "已同意"
                } else {
                    "未同意"
                },
                if self.notifications { "开" } else { "关" },
                if self.newsletter { "开" } else { "关" },
                PLANS[self.plan],
                self.volume,
            )))
    }

    fn tabs_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("页签 Tabs / 分段选择 Segmented"))
            .child(
                Tabs::new(TABS, self.tab_ix)
                    .disabled_tabs(vec![2])
                    .on_select(cx.listener(|this, ix, _, cx| {
                        this.tab_ix = *ix;
                        cx.notify();
                    })),
            )
            .child(caption(format!(
                "当前页签：{}（「设置」已禁用）",
                TABS[self.tab_ix]
            )))
            .child(Divider::new())
            .child(caption("卡片式 + 可关闭页签："))
            .child(
                Tabs::new(self.close_tabs.clone(), self.close_tab_ix)
                    .card()
                    .closable(true)
                    .on_select(cx.listener(|this, ix, _, cx| {
                        this.close_tab_ix = *ix;
                        cx.notify();
                    }))
                    .on_close(cx.listener(|this, ix, _, cx| {
                        this.close_tabs.remove(*ix);
                        if this.close_tab_ix >= this.close_tabs.len() {
                            this.close_tab_ix = this.close_tabs.len().saturating_sub(1);
                        }
                        cx.notify();
                    })),
            )
            .child(Divider::new())
            .child(caption(
                "拖拽排序：按住页签拖动，松手到目标页签上即重新排序（悬停目标会高亮）",
            ))
            .child(
                Tabs::new(self.drag_tabs.clone(), self.drag_active)
                    .id("tabs-drag-demo")
                    .draggable(true)
                    .on_select(cx.listener(|this, ix, _, cx| {
                        this.drag_active = *ix;
                        cx.notify();
                    }))
                    .on_reorder(cx.listener(|this, ix: &(usize, usize), _, cx| {
                        let (from, to) = *ix;
                        if from >= this.drag_tabs.len() || to > this.drag_tabs.len() {
                            return;
                        }
                        let item = this.drag_tabs.remove(from);
                        let to = to.min(this.drag_tabs.len());
                        this.drag_tabs.insert(to, item);
                        // 选中页签下标跟随移动
                        let active = this.drag_active;
                        this.drag_active = if active == from {
                            to
                        } else if from < active && active <= to {
                            active - 1
                        } else if to <= active && active < from {
                            active + 1
                        } else {
                            active
                        };
                        cx.notify();
                    })),
            )
            .child(caption(format!(
                "当前顺序：{}（选中：{}）",
                self.drag_tabs
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(" · "),
                self.drag_tabs
                    .get(self.drag_active)
                    .map(|t| t.to_string())
                    .unwrap_or_default()
            )))
            .child(Divider::new())
            .child(
                Segmented::new(["日", "周", "月"], self.seg_ix).on_select(cx.listener(
                    |this, ix, _, cx| {
                        this.seg_ix = *ix;
                        cx.notify();
                    },
                )),
            )
    }

    fn select_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("下拉选择 Select / 工具提示 Tooltip"))
            .child(
                field_row("城市").child(
                    div().flex_1().child(
                        Select::new("city", CITIES, self.city)
                            .open(self.select_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.select_open = *open;
                                cx.notify();
                            }))
                            .on_select(cx.listener(|this, ix, _, cx| {
                                this.city = *ix;
                                cx.notify();
                            })),
                    ),
                ),
            )
            .child(caption(format!("已选择：{}", CITIES[self.city])))
            .child(
                field_row("可搜索").child(
                    div().flex_1().child(
                        Select::new("city-search", CITIES, 0)
                            .value(self.search_city)
                            .searchable(true)
                            .allow_clear(true)
                            .open(self.search_city_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.search_city_open = *open;
                                cx.notify();
                            }))
                            .on_select(cx.listener(|this, ix, _, cx| {
                                this.search_city = Some(*ix);
                                cx.notify();
                            }))
                            .on_clear({
                                let entity = cx.entity();
                                move |_, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.search_city = None;
                                        cx.notify();
                                    });
                                }
                            }),
                    ),
                ),
            )
            .child(caption(match self.search_city {
                Some(ix) => format!("可搜索下拉已选：{}（点 ✕ 清空）", CITIES[ix]),
                None => "可搜索下拉：展开后输入关键字过滤，未选择".to_string(),
            }))
            .child(
                field_row("技能（多选）").child(
                    div().flex_1().child(
                        MultiSelect::new("skills", SKILLS, self.skills.clone())
                            .open(self.skills_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.skills_open = *open;
                                cx.notify();
                            }))
                            .on_change(cx.listener(|this, sel: &Vec<usize>, _, cx| {
                                this.skills = sel.clone();
                                cx.notify();
                            })),
                    ),
                ),
            )
            .child(caption(format!(
                "已选技能：{}",
                if self.skills.is_empty() {
                    "无".to_string()
                } else {
                    self.skills
                        .iter()
                        .map(|i| SKILLS[*i])
                        .collect::<Vec<_>>()
                        .join("、")
                }
            )))
            .child(Divider::new())
            .child(Tooltip::new(
                "这是一条工具提示，悬停片刻即可显示",
                Button::new("悬停查看提示").secondary(),
            ))
    }

    fn feedback_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title(
                "提示 Alert / 标签 Tag / 头像 Avatar / 加载 Spinner",
            ))
            .child(
                Alert::new("保存成功")
                    .success()
                    .description("所有更改已同步到云端。"),
            )
            .child(
                Alert::new("磁盘空间不足")
                    .warning()
                    .description("剩余空间不足 10%，请及时清理。"),
            )
            .child(Divider::new())
            .child(div().flex().flex_row().flex_wrap().gap_2().children(
                self.tags.iter().enumerate().map(|(ix, label)| {
                    Tag::new(label.clone()).on_close(cx.listener(move |this, _, _, cx| {
                        this.tags.remove(ix);
                        cx.notify();
                    }))
                }),
            ))
            .child(caption("点击标签上的 ✕ 可移除，「重置全部」可恢复"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_2()
                    .child(Tag::new("默认"))
                    .child(Tag::new("蓝").color(TagColor::Blue))
                    .child(Tag::new("绿").color(TagColor::Green))
                    .child(Tag::new("橙").color(TagColor::Orange))
                    .child(Tag::new("红").color(TagColor::Red))
                    .child(Tag::new("紫").color(TagColor::Purple))
                    .child(Tag::new("青").color(TagColor::Cyan))
                    .child(Tag::new("品红").color(TagColor::Magenta))
                    .child(Tag::new("无边框").color(TagColor::Blue).bordered(false)),
            )
            .child(Divider::new())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(Avatar::new("张三"))
                    .child(Avatar::new("李四").size(px(40.)))
                    .child(Avatar::new("王五").size(px(48.)))
                    .child(Spinner::new())
                    .child(Spinner::new().size(px(28.))),
            )
    }

    fn stepper_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("步进器 Stepper / 对话框 Modal"))
            .child(
                field_row("数量").child(
                    Stepper::new("qty", self.step_val)
                        .min(0.)
                        .max(10.)
                        .step(1.)
                        .on_change(cx.listener(|this, value, _, cx| {
                            this.step_val = *value;
                            cx.notify();
                        })),
                ),
            )
            .child(
                Button::new("打开对话框")
                    .primary()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.modal_open = true;
                        cx.notify();
                    })),
            )
    }

    fn push_toast(&mut self, item: ToastItem, cx: &mut Context<Self>) {
        self.toast_count += 1;
        self.toasts.push(item);
        // 最多同时展示 4 条
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }
        cx.notify();
    }

    fn nav_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("面包屑 Breadcrumb / 分页 Pagination"))
            .child(
                Breadcrumb::new(CRUMBS).on_click(cx.listener(|this, ix, _, cx| {
                    this.push_toast(ToastItem::new(format!("导航到：{}", CRUMBS[*ix])), cx);
                })),
            )
            .child(Divider::new())
            .child(
                Pagination::new("demo-page", self.page, 12).on_change(cx.listener(
                    |this, page, _, cx| {
                        this.page = *page;
                        cx.notify();
                    },
                )),
            )
            .child(caption(format!(
                "当前第 {} / 12 页，页数多时自动省略号折叠",
                self.page
            )))
    }

    fn accordion_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("折叠面板 Accordion / 时间线 Timeline"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_5()
                    .child(
                        div().flex_1().child(
                            Accordion::new(
                                "faq",
                                vec![
                                    AccordionItem::new(
                                        "什么是 GPUI？",
                                        div()
                                            .text_color(theme::text_muted())
                                            .child("Zed 编辑器的 GPU 加速 UI 框架，Rust 编写。"),
                                    ),
                                    AccordionItem::new(
                                        "如何使用本控件库？",
                                        div().text_color(theme::text_muted()).child(
                                            "在 Cargo.toml 中以路径依赖引入 gpui-widgets，详见 README。",
                                        ),
                                    ),
                                    AccordionItem::new(
                                        "支持哪些平台？",
                                        div()
                                            .text_color(theme::text_muted())
                                            .child("Windows 已验证；macOS / Linux 由 GPUI 自身支持。"),
                                    ),
                                ],
                            )
                            .open(self.acc_open.clone())
                            .on_change(cx.listener(|this, open: &Vec<usize>, _, cx| {
                                this.acc_open = open.clone();
                                cx.notify();
                            })),
                        ),
                    )
                    .child(
                        div().w(px(260.)).child(Timeline::new(vec![
                            TimelineItem::new("创建项目").time("09:00"),
                            TimelineItem::new("控件库成型")
                                .time("11:30")
                                .description("基础控件 + gallery 示例"),
                            TimelineItem::new("修复 Select 定位")
                                .time("14:00")
                                .description("同宽贴框、溢出翻转")
                                .color(theme::success()),
                            TimelineItem::new("扩充控件")
                                .time("16:00")
                                .description("新增 16 个常用控件"),
                        ])),
                    ),
            )
    }

    fn data_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("表格 Table / 树形 Tree"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_5()
                    .child(div().flex_1().child({
                        let mut rows: Vec<Vec<gpui::SharedString>> = vec![
                            vec!["张三".into(), "北京".into(), "Rust".into()],
                            vec!["李四".into(), "上海".into(), "GPUI".into()],
                            vec!["王五".into(), "广州".into(), "桌面端".into()],
                            vec!["赵六".into(), "深圳".into(), "跨平台".into()],
                            vec!["孙七".into(), "杭州".into(), "开源".into()],
                            vec!["周八".into(), "成都".into(), "Rust".into()],
                            vec!["吴九".into(), "武汉".into(), "GPUI".into()],
                            vec!["郑十".into(), "南京".into(), "桌面端".into()],
                            vec!["钱一".into(), "西安".into(), "跨平台".into()],
                            vec!["陈二".into(), "苏州".into(), "开源".into()],
                        ];
                        if let Some((col, desc)) = self.table_sort {
                            rows.sort_by(|a, b| {
                                let ord = a[col].to_string().cmp(&b[col].to_string());
                                if desc { ord.reverse() } else { ord }
                            });
                        }
                        let detail_rows = rows.clone();
                        Table::new("members", ["姓名", "城市", "技能"], rows)
                            .striped(true)
                            .column_widths(vec![px(80.), px(80.), px(110.)])
                            .sortable(vec![0, 1])
                            .sort(self.table_sort)
                            .on_sort(cx.listener(|this, sort: &(usize, bool), _, cx| {
                                let (col, _) = *sort;
                                this.table_sort = Some(match this.table_sort {
                                    Some((c, desc)) if c == col => (col, !desc),
                                    _ => (col, false),
                                });
                                cx.notify();
                            }))
                            .selectable(true)
                            .selected_rows(self.table_checked.clone())
                            .on_selected_rows(cx.listener(|this, rows: &Vec<usize>, _, cx| {
                                this.table_checked = rows.clone();
                                cx.notify();
                            }))
                            .selected(self.table_row)
                            .on_row_click(cx.listener(|this, ix, _, cx| {
                                this.table_row = Some(*ix);
                                cx.notify();
                            }))
                            .pagination(self.table_page, 4)
                            .on_page_change(cx.listener(|this, page, _, cx| {
                                this.table_page = *page;
                                cx.notify();
                            }))
                            .row_detail(move |ix| {
                                let row = &detail_rows[ix];
                                div()
                                    .text_sm()
                                    .text_color(theme::text_muted())
                                    .child(format!(
                                        "{} · {} · 技能：{}（展开行详情，全局下标 {ix}）",
                                        row[0], row[1], row[2]
                                    ))
                                    .into_any_element()
                            })
                            .expanded_rows(self.table_expanded.clone())
                            .on_expand(cx.listener(|this, ix, _, cx| {
                                if let Some(pos) =
                                    this.table_expanded.iter().position(|i| i == ix)
                                {
                                    this.table_expanded.remove(pos);
                                } else {
                                    this.table_expanded.push(*ix);
                                }
                                cx.notify();
                            }))
                    }))
                    .child(
                        div().w(px(240.)).child(
                            Tree::new("files", self.tree_nodes.clone())
                            .expanded(
                                self.tree_expanded.iter().map(|s| s.clone().into()).collect(),
                            )
                            .on_toggle(cx.listener(|this, id: &gpui::SharedString, _, cx| {
                                let id = id.to_string();
                                if let Some(pos) =
                                    this.tree_expanded.iter().position(|s| *s == id)
                                {
                                    this.tree_expanded.remove(pos);
                                } else {
                                    this.tree_expanded.push(id);
                                }
                                cx.notify();
                            }))
                            .selected(self.tree_selected.clone().map(Into::into))
                            .on_select(cx.listener(|this, id: &gpui::SharedString, _, cx| {
                                this.tree_selected = Some(id.to_string());
                                cx.notify();
                            }))
                            .checkable(true)
                            .checked(self.tree_checked.clone())
                            .on_check(cx.listener(
                                |this, checked: &Vec<gpui::SharedString>, _, cx| {
                                    this.tree_checked = checked.clone();
                                    cx.notify();
                                },
                            ))
                            .draggable(true)
                            .on_drop_node(cx.listener(|this, drop: &TreeDrop, _, cx| {
                                // 重组：把被拖节点移到目标节点下
                                if let Some(node) = take_node(&mut this.tree_nodes, &drop.dragged) {
                                    if let Some(target) =
                                        find_node_mut(&mut this.tree_nodes, &drop.target)
                                    {
                                        target.push_child(node);
                                    } else {
                                        this.tree_nodes.push(node);
                                    }
                                }
                                // 展开目标节点便于看到移动结果
                                let target = drop.target.to_string();
                                if !this.tree_expanded.contains(&target) {
                                    this.tree_expanded.push(target);
                                }
                                cx.notify();
                            })),
                        ),
                    ),
            )
            .child(caption(
                "表：点表头排序；勾选多选（仅当前页）；▸ 展开详情；底部分页。树：勾选父子联动，拖节点到另一节点上移动",
            ))
            .child(Divider::new())
            .child(caption("固定左列：首列固定，其余列横向滚动（统一行高，不支持展开行）"))
            .child(
                div().w(px(560.)).child(
                    Table::new(
                        "wide",
                        ["姓名", "部门", "城市", "职位", "入职年份", "邮箱", "电话", "状态"],
                        vec![
                            vec![
                                "张三".into(), "技术部".into(), "北京".into(), "工程师".into(),
                                "2021".into(), "zhang@corp.cn".into(), "138-0001".into(), "在职".into(),
                            ],
                            vec![
                                "李四".into(), "产品部".into(), "上海".into(), "产品经理".into(),
                                "2022".into(), "li@corp.cn".into(), "138-0002".into(), "在职".into(),
                            ],
                            vec![
                                "王五".into(), "设计部".into(), "广州".into(), "设计师".into(),
                                "2023".into(), "wang@corp.cn".into(), "138-0003".into(), "休假".into(),
                            ],
                        ],
                    )
                    .fixed_left()
                    .column_widths(vec![
                        px(70.), px(90.), px(80.), px(100.), px(90.), px(150.), px(110.), px(80.),
                    ])
                    .striped(true),
                ),
            )
            .child(Divider::new())
            .child(caption(
                "区间多选：普通点击选单行 · Shift 点击选连续区间 · Ctrl 点击切换该行（结果实时回显）",
            ))
            .child(div().flex().flex_col().gap_2().w(px(560.)).child(
                Table::new(
                    "range-select",
                    ["姓名", "部门", "状态"],
                    vec![
                        vec!["张三".into(), "技术部".into(), "在职".into()],
                        vec!["李四".into(), "产品部".into(), "在职".into()],
                        vec!["王五".into(), "设计部".into(), "休假".into()],
                        vec!["赵六".into(), "运维部".into(), "在职".into()],
                        vec!["孙七".into(), "测试部".into(), "出差".into()],
                        vec!["周八".into(), "市场部".into(), "在职".into()],
                    ],
                )
                .column_widths(vec![px(90.), px(120.), px(90.)])
                .striped(true)
                .range_select(true)
                .selected_rows(self.range_sel.clone())
                .on_selected_rows(cx.listener(|this, rows: &Vec<usize>, _, cx| {
                    this.range_sel = rows.clone();
                    cx.notify();
                })),
            ),)
            .child(div().text_sm().text_color(theme::text_muted()).child(format!(
                "已选 {} 行：{}",
                self.range_sel.len(),
                if self.range_sel.is_empty() {
                    "—".to_string()
                } else {
                    self.range_sel
                        .iter()
                        .map(|i| (i + 1).to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            )))
    }

    fn menu_section(&self, cx: &mut Context<Self>) -> Div {
        // MenuItem 回调签名为 (&mut Window, &mut App)，不适用 cx.listener，改用实体句柄
        let entity = cx.entity();
        let menu = DropdownMenu::new(
            "more-actions",
            Button::new("更多操作").secondary(),
            vec![
                MenuItem::new("编辑").on_click({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.push_toast(ToastItem::new("触发了编辑动作"), cx)
                        })
                    }
                }),
                MenuItem::new("复制").on_click({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.push_toast(ToastItem::new("触发了复制动作"), cx)
                        })
                    }
                }),
                MenuItem::separator(),
                MenuItem::new("删除").danger().on_click({
                    let entity = entity.clone();
                    move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.push_toast(ToastItem::new("已删除").danger(), cx)
                        })
                    }
                }),
            ],
        )
        .open(self.menu_open)
        .on_open_change(cx.listener(|this, open, _, cx| {
            this.menu_open = *open;
            cx.notify();
        }));

        card()
            .child(section_title(
                "动作菜单 DropdownMenu / 气泡 Popover / 评分 Rating",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(menu)
                    .child(
                        Popover::new(
                            "info-pop",
                            Button::new("查看详情").secondary(),
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(div().child("这是气泡内容，可放置任意控件。"))
                                .child(Button::new("知道了").primary().small().on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.popover_open = false;
                                        cx.notify();
                                    }),
                                )),
                        )
                        .width(px(240.))
                        .open(self.popover_open)
                        .on_open_change(cx.listener(
                            |this, open, _, cx| {
                                this.popover_open = *open;
                                cx.notify();
                            },
                        )),
                    ),
            )
            .child(Divider::new())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        Rating::new("score", self.score as f32).on_change(cx.listener(
                            |this, value, _, cx| {
                                this.score = *value as usize;
                                cx.notify();
                            },
                        )),
                    )
                    .child(caption(format!(
                        "当前评分：{} 星（点击当前星数可清零）",
                        self.score
                    ))),
            )
    }

    fn picker_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title(
                "日期 DatePicker / 范围 DateRangePicker / 颜色 ColorPicker",
            ))
            .child(
                field_row("日期").child(
                    div().flex_1().child(
                        DatePicker::new("due-date", self.date)
                            .allow_clear(true)
                            .open(self.date_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.date_open = *open;
                                cx.notify();
                            }))
                            .on_change(cx.listener(|this, date, _, cx| {
                                this.date = *date;
                                this.date_month = date.map(|(y, m, _)| (y, m));
                                cx.notify();
                            }))
                            .month(self.date_month)
                            .on_month_change(cx.listener(|this, m, _, cx| {
                                this.date_month = Some(*m);
                                cx.notify();
                            })),
                    ),
                ),
            )
            .child(caption(match self.date {
                Some((y, m, d)) => format!("已选择：{y:04}-{m:02}-{d:02}（点 ✕ 清空）"),
                None => "未选择日期".to_string(),
            }))
            .child(
                field_row("日期范围").child(
                    div().flex_1().child(
                        DateRangePicker::new("stay-range", self.range)
                            .allow_clear(true)
                            .open(self.range_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.range_open = *open;
                                cx.notify();
                            }))
                            .on_change(cx.listener(|this, range, _, cx| {
                                this.range = *range;
                                if let Some(((y, m, _), _)) = range {
                                    this.range_month = Some((*y, *m));
                                }
                                cx.notify();
                            }))
                            .month(self.range_month)
                            .on_month_change(cx.listener(|this, m, _, cx| {
                                this.range_month = Some(*m);
                                cx.notify();
                            })),
                    ),
                ),
            )
            .child(caption(match self.range {
                Some(((y1, m1, d1), (y2, m2, d2))) => format!(
                    "已选范围：{y1:04}-{m1:02}-{d1:02} ~ {y2:04}-{m2:02}-{d2:02}（点 ✕ 清空）"
                ),
                None => "未选范围：先点开始日期，再点结束日期".to_string(),
            }))
            .child(
                field_row("禁用").child(
                    div().flex_1().child(
                        DatePicker::new("due-disabled", Some((2026, 11, 12))).disabled(true),
                    ),
                ),
            )
            .child(
                field_row("主题色").child(
                    div().flex_1().child(
                        ColorPicker::new("brand-color", self.color)
                            .open(self.color_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.color_open = *open;
                                cx.notify();
                            }))
                            .on_change(cx.listener(|this, color, _, cx| {
                                this.color = *color;
                                cx.notify();
                            })),
                    ),
                ),
            )
    }

    fn overlay_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("抽屉 Drawer / 轻提示 Toast"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .child(Button::new("打开抽屉").secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.drawer_left = false;
                            this.drawer_open = true;
                            cx.notify();
                        },
                    )))
                    .child(Button::new("左侧抽屉").secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.drawer_left = true;
                            this.drawer_open = true;
                            cx.notify();
                        },
                    )))
                    .child(Button::new("发送轻提示").primary().on_click(cx.listener(
                        |this, _, _, cx| {
                            let n = this.toast_count + 1;
                            this.push_toast(
                                ToastItem::new(format!("通知 #{n}"))
                                    .success()
                                    .description("点 ✕ 移除，最多同时 4 条"),
                                cx,
                            );
                        },
                    ))),
            )
            .child(caption(
                "抽屉默认右侧滑出，可切换左侧；轻提示悬浮在窗口右上角",
            ))
    }

    fn placeholder_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("骨架屏 Skeleton / 空状态 Empty"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(Skeleton::circle(px(40.)))
                    .child(div().flex_1().child(Skeleton::lines(3))),
            )
            .child(Divider::new())
            .child(
                Empty::new("暂无数据")
                    .description("点击下方按钮创建第一条记录")
                    .action(Button::new("新建").primary().small().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.push_toast(ToastItem::new("创建成功").success(), cx);
                        },
                    ))),
            )
    }

    fn media_section(&self) -> Div {
        let assets = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/");
        let mut section = card()
            .child(section_title("图片 ImageView / 视频 VideoPlayer"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                ImageView::from_file(format!("{assets}sample.png"))
                                    .alt("示例 PNG 图片")
                                    .fit_cover()
                                    .rounded(px(8.))
                                    .w(px(220.))
                                    .h(px(140.)),
                            )
                            .child(caption("PNG · cover 裁剪")),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                ImageView::from_file(format!("{assets}sample.svg"))
                                    .alt("示例 SVG 图片")
                                    .fit_contain()
                                    .rounded(px(8.))
                                    .bg(theme::surface())
                                    .w(px(220.))
                                    .h(px(140.)),
                            )
                            .child(caption("SVG · contain")),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                ImageView::from_file(format!("{assets}missing.png"))
                                    .alt("")
                                    .id("missing-image")
                                    .w(px(220.))
                                    .h(px(140.)),
                            )
                            .child(caption("加载失败占位")),
                    ),
            );
        #[cfg(windows)]
        {
            section = section
                .child(div().w(px(420.)).child(self.video.clone()))
                .child(caption(
                    "AVI/MP4 等 · Media Foundation 后台解码 · 播放/暂停/拖动进度",
                ));
        }
        section
    }

    fn inputs_plus_section(&self, cx: &mut Context<Self>) -> Div {
        let password_len = self.password.read(cx).text(cx).chars().count();
        card()
            .child(section_title(
                "输入增强 PasswordInput / SearchInput / NumberInput",
            ))
            .child(field_row("密码").child(div().flex_1().child(self.password.clone())))
            .child(caption(format!(
                "已输入 {password_len} 个字符，点右侧按钮切换明文/掩码"
            )))
            .child(field_row("搜索").child(div().flex_1().child(self.search.clone())))
            .child(caption(if self.last_search.is_empty() {
                "输入关键词后回车触发搜索，✕ 一键清空".to_string()
            } else {
                format!("搜索：{}", self.last_search)
            }))
            .child(field_row("数值").child(div().w(px(200.)).child(self.number.clone())))
            .child(caption(match self.number_val {
                Some(v) => format!("当前值：{v}（范围 0~100，步长 5，上下方向键 / 悬停步进按钮）"),
                None => "等待输入有效数值…".to_string(),
            }))
    }

    fn tags_auto_section(&self, _cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("标签输入 TagInput / 自动完成 Autocomplete"))
            .child(field_row("标签").child(div().flex_1().child(self.tag_input.clone())))
            .child(caption(format!(
                "共 {} 个标签：{}（回车添加，空草稿时退格删末尾）",
                self.input_tags.len(),
                self.input_tags
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join("、")
            )))
            .child(field_row("框架").child(div().flex_1().child(self.autocomplete.clone())))
            .child(caption(if self.auto_pick.is_empty() {
                "输入过滤候选，↑↓ 选择，回车确认".to_string()
            } else {
                format!("已选择：{}", self.auto_pick)
            }))
    }

    fn textarea_section(&self, cx: &mut Context<Self>) -> Div {
        let len = self.text_area.read(cx).text().chars().count();
        card()
            .child(section_title("多行输入 TextArea"))
            .child(self.text_area.clone())
            .child(caption(format!(
                "已输入 {len} 个字符 · 支持选区 / 剪贴板 / 中文输入法 / 光标自动滚动"
            )))
    }

    /// 性能监视：FpsMeter 连续渲染实测 FPS（最高 120fps），仅重绘自身子树。
    fn perf_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("性能监视 FpsMeter（最高 120fps）"))
            .child(caption(
                "独立 Entity 驱动连续渲染，仅重绘本控件；时间基动画在高刷新率下平滑；帧率跟随显示器 vsync",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_4()
                    .child(div().flex_1().min_w(px(0.)).child(self.fps_meter.clone()))
                    .child(
                        Button::new("开始 / 停止").secondary().on_click(cx.listener(
                            |this, _, _, cx| {
                                this.fps_meter.update(cx, |m, cx| {
                                    if m.is_running() {
                                        m.stop(cx);
                                    } else {
                                        m.start(cx);
                                    }
                                });
                                cx.notify();
                            },
                        )),
                    )
                    .child(Spinner::new().tip("时间基动画")),
            )
    }

    /// 终端 Terminal：VT/ANSI 纯自绘，内置回显 shell 演示输入回调。
    fn terminal_section(&self, cx: &mut Context<Self>) -> Div {
        let title = if self.term_title.is_empty() {
            "—"
        } else {
            self.term_title.as_str()
        };
        card()
            .child(section_title("终端 Terminal（VT/ANSI 纯自绘）"))
            .child(caption(
                "字符网格 + 回滚缓冲 + 选区复制 + 搜索高亮 + 输入回调；键盘经 on_input 投递字节，演示内置回显 shell（help / ls / color / clear）",
            ))
            .child(
                div()
                    .text_sm()
                    .text_color(theme::text_muted())
                    .mb_2()
                    .child(format!("窗口标题（OSC 0/2）：{title}")),
            )
            .child(self.terminal.clone())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .mt_3()
                    .child(div().flex_1().min_w(px(0.)).child(self.term_search.clone()))
                    .child(Button::new("搜索").small().on_click(cx.listener(
                        |this, _, _, cx| {
                            let q = this.term_search.read(cx).text().to_string();
                            this.terminal.update(cx, |t, cx| t.set_search(&q, cx));
                        },
                    )))
                    .child(Button::new("上一个").small().secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.terminal.update(cx, |t, cx| t.search_prev(cx));
                        },
                    )))
                    .child(Button::new("下一个").small().secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.terminal.update(cx, |t, cx| t.search_next(cx));
                        },
                    )))
                    .child(Button::new("清屏").small().ghost().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.terminal.update(cx, |t, cx| {
                                t.clear(cx);
                                t.write_str(TERM_PROMPT, cx);
                            });
                        },
                    )))
                    .child(Button::new("回到底部").small().ghost().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.terminal.update(cx, |t, cx| t.scroll_to_bottom(cx));
                        },
                    ))),
            )
            .child(caption(
                "提示：鼠标拖拽选区，Ctrl+C 复制（无选区发送 ^C），Ctrl+V 粘贴，滚轮 / Shift+PgUp 翻阅回滚；点击终端区域获取焦点后即可输入",
            ))
    }

    /// 快捷键录入演示（终端与开发分类）。
    fn hotkey_section(&self, cx: &mut Context<Self>) -> Div {
        let save_binding = self.hotkey_save.read(cx).binding().to_string();
        let palette_binding = self.hotkey_palette.read(cx).binding().to_string();
        let log = if self.hotkey_log.is_empty() {
            "（尚未录入）".to_string()
        } else {
            self.hotkey_log.clone()
        };
        card()
            .child(section_title("快捷键录入 HotkeyInput"))
            .child(caption(
                "点击聚焦后按下组合键录入：修饰键实时预览，主键完成录入；Esc 取消、Backspace 清空、✕ 清除、失焦自动结束",
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .mt_2()
                    .child(
                        field_row("保存文件")
                            .child(div().flex_1().child(self.hotkey_save.clone())),
                    )
                    .child(
                        field_row("命令面板")
                            .child(div().flex_1().child(self.hotkey_palette.clone())),
                    )
                    .child(
                        field_row("禁用态")
                            .child(div().flex_1().child(self.hotkey_disabled.clone())),
                    ),
            )
            .child(
                div()
                    .mt_3()
                    .text_sm()
                    .text_color(theme::text_muted())
                    .child(format!("最近录入：{log}")),
            )
            .child(
                div()
                    .mt_1()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child(format!(
                        "binding() 可注册串 → 保存：{save_binding} ｜ 命令面板：{palette_binding}"
                    )),
            )
            .child(caption(
                "提示：命令面板要求至少一个主修饰键（Ctrl/Alt/Win），裸键与仅 Shift 组合将被忽略",
            ))
    }

    /// 命令输入条演示（终端与开发分类）。
    fn cmd_section(&self, cx: &mut Context<Self>) -> Div {
        let count = self.cmd_bar.read(cx).history().len();
        let mut log = div().flex().flex_col().gap_1().mt_3();
        if self.cmd_runs.is_empty() {
            log = log.child(
                div()
                    .text_sm()
                    .text_color(theme::text_muted())
                    .child("尚未执行命令"),
            );
        } else {
            for (i, c) in self.cmd_runs.iter().enumerate() {
                log = log.child(div().text_sm().text_color(theme::text()).child(format!(
                    "{}. {}",
                    i + 1,
                    c
                )));
            }
        }
        card()
            .child(section_title("命令输入条 CommandBar"))
            .child(caption(
                "单行命令输入：Enter 提交、↑↓ 历史回溯、Tab 前缀补全、点击片段芯片插入",
            ))
            .child(div().mt_2().child(self.cmd_bar.clone()))
            .child(
                div()
                    .mt_3()
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child(format!("历史条数：{count}")),
            )
            .child(log)
            .child(caption(
                "提示：提交后命令去重入历史；聚焦输入框后按 ↑ 可回溯刚才的命令",
            ))
    }

    /// 外部文件拖放演示（终端与开发分类）。
    fn drop_section(&self, cx: &mut Context<Self>) -> Div {
        let mut list = div().flex().flex_col().gap_1().mt_3();
        if self.drop_files.is_empty() {
            list =
                list.child(div().text_sm().text_color(theme::text_muted()).child(
                    "尚未接收文件（从资源管理器拖一个 .rs / .toml / .md 文件到上方区域试试）",
                ));
        } else {
            for (name, size) in self.drop_files.iter().rev().take(8) {
                list = list.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(theme::hover_bg())
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme::text())
                                .child(name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child(format_size(*size)),
                        ),
                );
            }
        }
        let mut summary = div()
            .mt_2()
            .text_xs()
            .text_color(theme::text_muted())
            .child(format!("已接收 {} 个文件", self.drop_files.len()));
        if self.drop_rejected > 0 {
            summary = summary.child(div().text_xs().text_color(theme::danger()).child(format!(
                "最近一次拖入有 {} 个文件因扩展名不符被拒绝",
                self.drop_rejected
            )));
        }
        card()
            .child(section_title("外部文件拖放 External Drop"))
            .child(caption(
                "从系统资源管理器拖入文件：拖拽悬停时整块高亮（落点反馈），松手后按扩展名过滤，扩展名命中才进入上传列表",
            ))
            .child(div().mt_2().gap_3().flex().flex_col()
                .child(
                    DropZone::new("drop-zone-demo")
                        .icon("📥")
                        .title("拖拽文件到此处")
                        .hint("支持批量，松开鼠标即加入列表")
                        .accept(["rs", "toml", "md"])
                        .height(140.)
                        .on_drop(cx.listener(|this, paths: &[std::path::PathBuf], _, cx| {
                            this.drop_rejected = 0;
                            for path in paths {
                                let name = path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| path.display().to_string());
                                let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                                this.drop_files.push((name, size));
                            }
                            cx.notify();
                        }))
                        .on_reject(cx.listener(|this, paths: &[std::path::PathBuf], _, cx| {
                            this.drop_rejected = paths.len();
                            cx.notify();
                        })),
                )
                .child(
                    DropZone::new("drop-zone-disabled")
                        .icon("🚫")
                        .title("禁用态拖放区")
                        .hint("disabled(true) 时不响应外部拖放")
                        .height(84.)
                        .disabled(true),
                ))
            .child(summary)
            .child(list)
            .child(caption(
                "提示：拖拽悬停高亮由 drag_over 完成；能否落点由 can_drop 谓词（扩展名）决定，被拒绝的文件不会进入列表",
            ))
    }

    /// 传输任务管理器演示（终端与开发分类）。
    fn transfer_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("传输任务 TransferManager"))
            .child(caption(
                "上传 / 下载队列总览：总进度、单项进度、速度与剩余时间、状态标签，以及暂停 / 继续 / 取消 / 重试 / 移除操作",
            ))
            .child(
                div()
                    .mt_3()
                    .child(
                        TransferManager::new("transfer-demo", self.transfers.clone())
                            .title("传输任务")
                            .on_pause(cx.listener(|this, id, _, cx| {
                                this.transfer_set_status(id, TransferStatus::Paused);
                                cx.notify();
                            }))
                            .on_resume(cx.listener(|this, id, _, cx| {
                                this.transfer_set_status(id, TransferStatus::Transferring);
                                cx.notify();
                            }))
                            .on_cancel(cx.listener(|this, id, _, cx| {
                                this.transfer_set_status(id, TransferStatus::Canceled);
                                cx.notify();
                            }))
                            .on_retry(cx.listener(|this, id, _, cx| {
                                for task in this.transfers.iter_mut().filter(|t| t.id() == id) {
                                    task.set_progress(0.);
                                    task.set_status(TransferStatus::Queued);
                                }
                                cx.notify();
                            }))
                            .on_remove(cx.listener(|this, id: &gpui::SharedString, _, cx| {
                                let id = id.clone();
                                this.transfers.retain(|t| t.id() != &id);
                                cx.notify();
                            }))
                            .on_clear_done({
                                let entity = cx.entity();
                                move |_, cx| {
                                    entity.update(cx, |this, cx| {
                                        this.transfers.retain(|t| {
                                            t.task_status() != TransferStatus::Done
                                        });
                                        cx.notify();
                                    });
                                }
                            }),
                    ),
            )
            .child(
                div()
                    .mt_3()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .child(Button::new("推进进度").primary().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.transfer_advance();
                            cx.notify();
                        },
                    )))
                    .child(Button::new("全部暂停").secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            for task in this
                                .transfers
                                .iter_mut()
                                .filter(|t| t.task_status() == TransferStatus::Transferring)
                            {
                                task.set_status(TransferStatus::Paused);
                            }
                            cx.notify();
                        },
                    )))
                    .child(Button::new("全部继续").secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            for task in this
                                .transfers
                                .iter_mut()
                                .filter(|t| t.task_status() == TransferStatus::Paused)
                            {
                                task.set_status(TransferStatus::Transferring);
                            }
                            cx.notify();
                        },
                    )))
                    .child(Button::new("+ 新任务").secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            let n = this.transfers.len() + 1;
                            this.transfers.push(
                                TransferTask::upload(
                                    format!("tm-new-{n}"),
                                    format!("新任务-{n}.bin"),
                                )
                                .size(20 * 1024 * 1024),
                            );
                            cx.notify();
                        },
                    )))
                    .child(Button::new("重置队列").secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.transfers = demo_transfers();
                            cx.notify();
                        },
                    ))),
            )
            .child(Divider::new())
            .child(caption(
                "状态覆盖：传输中 / 等待中 / 已暂停 / 已完成 / 失败；「推进进度」每次 +12%，达到 100% 自动转为已完成",
            ))
    }

    /// 修改指定任务的状态
    fn transfer_set_status(&mut self, id: &gpui::SharedString, status: TransferStatus) {
        for task in self.transfers.iter_mut().filter(|t| t.id() == id) {
            task.set_status(status);
        }
    }

    /// 推进演示：传输中任务 +12%，并自动启动一个排队任务
    fn transfer_advance(&mut self) {
        for task in self
            .transfers
            .iter_mut()
            .filter(|t| t.task_status() == TransferStatus::Transferring)
        {
            let next = (task.progress_value() + 0.12).min(1.);
            task.set_progress(next);
            if next >= 1. {
                task.set_status(TransferStatus::Done);
            }
        }
        let idle = !self
            .transfers
            .iter()
            .any(|t| t.task_status() == TransferStatus::Transferring);
        if idle {
            if let Some(task) = self
                .transfers
                .iter_mut()
                .find(|t| t.task_status() == TransferStatus::Queued)
            {
                task.set_status(TransferStatus::Transferring);
            }
        }
    }

    /// 代码编辑器演示（终端与开发分类）。
    fn code_section(&self, cx: &mut Context<Self>) -> Div {
        let text = self.code_editor.read(cx).text().to_string();
        let lines = text.lines().count().max(1);
        let chars = text.chars().count();
        let mut langs = div().flex().flex_row().items_center().gap_2().mb_3();
        for lang in CodeLanguage::all() {
            let mut btn = Button::new(lang.label()).small();
            if *lang != self.code_lang {
                btn = btn.secondary();
            }
            langs = langs.child(btn.on_click(cx.listener(move |this, _, _, cx| {
                this.set_code_language(*lang, false, cx);
            })));
        }
        card()
            .child(section_title("代码编辑器 CodeEditor"))
            .child(caption(
                "行号栏 + 当前行高亮 + 等宽字体 + 4 空格缩进 + 逐行语法着色；Enter 换行、Tab 缩进、Shift+Tab 反缩进，鼠标单击定位、拖拽 / Shift 点击选词",
            ))
            .child(langs)
            .child(self.code_editor.clone())
            .child(
                div()
                    .mt_3()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child(format!(
                                "语言：{} ｜ 行数：{lines} ｜ 字符：{chars}",
                                self.code_lang.label()
                            )),
                    )
                    .child(Button::new("载入示例").small().secondary().on_click(
                        cx.listener(|this, _, _, cx| {
                            let lang = this.code_lang;
                            this.set_code_language(lang, true, cx);
                        }),
                    )),
            )
            .child(caption(
                "提示：切换语言只更换着色规则，内容保持不变；选择「纯文本」关闭着色",
            ))
    }

    /// 切换代码编辑器的语法着色（可选同时载入该语言示例）
    fn set_code_language(&mut self, lang: CodeLanguage, load_demo: bool, cx: &mut Context<Self>) {
        self.code_lang = lang;
        let demo = demo_source(lang);
        self.code_editor.update(cx, |area, cx| {
            CodeEditor::set_language(area, lang, cx);
            if load_demo {
                area.set_text_silent(demo, cx);
            }
        });
        cx.notify();
    }

    /// 自绘标题栏演示（终端与开发分类）。
    fn titlebar_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("自绘标题栏 TitleBar"))
            .child(caption(
                "Windows 上拖拽、双击最大化 / 还原、贴靠布局、最小化 / 关闭全部交给系统非客户区处理；其他平台回退为直接窗口操作",
            ))
            .child(
                div()
                    .mt_3()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        // 静态预览：live(false) 不注册窗口命中区域，可安全嵌入页面
                        div()
                            .border_1()
                            .border_color(theme::border())
                            .rounded_md()
                            .overflow_hidden()
                            .child(
                                TitleBar::new("预览：未命名文档 - 编辑器")
                                    .id("titlebar-preview")
                                    .icon("🪟")
                                    .live(false)
                                    .extra(
                                        div()
                                            .ml_2()
                                            .text_xs()
                                            .text_color(theme::danger())
                                            .child("● 未保存"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(
                                Button::new("打开自绘标题栏窗口")
                                    .primary()
                                    .on_click(cx.listener(|_, _, window, cx| {
                                        // 延迟到当前事件处理完再建窗，避免重入
                                        window.defer(cx, |_w, cx| open_titlebar_demo_window(cx));
                                    })),
                            )
                            .child(caption("新窗口无系统标题栏，全部由控件绘制")),
                    ),
            )
            .child(caption(
                "提示：预览条上的按钮不可交互；在真实窗口里标题栏按钮由操作系统接管（包括 Win11 贴靠布局）",
            ))
    }

    /// 左侧分类导航菜单（唯一 id cat-nav-{i}，避免与 AI 分区 Conversations 撞 id）。
    fn category_nav(&self, cx: &mut Context<Self>) -> Div {
        let mut nav = div().flex().flex_col().gap_1().w_full();
        for (i, &(name, icon, _)) in CATEGORIES.iter().enumerate() {
            let is_active = self.active_category == i;
            let mouse_listener = cx.listener(move |this, _, _, cx| {
                this.active_category = i;
                this.scroll_handle.set_offset(gpui::Point::default());
                cx.notify();
            });
            let a11y_listener = cx.listener(move |this, _, _, cx| {
                this.active_category = i;
                this.scroll_handle.set_offset(gpui::Point::default());
                cx.notify();
            });
            let mut row = div()
                .id(gpui::ElementId::Name(format!("cat-nav-{i}").into()))
                .role(gpui::Role::Button)
                .aria_label(name)
                .accessibility_id(format!("cat-nav-{i}"))
                .focusable()
                .tab_stop(true)
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .py_2()
                .rounded_md()
                .cursor(gpui::CursorStyle::PointingHand)
                .on_click(move |event, window, cx| mouse_listener(event, window, cx))
                .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                    a11y_listener(&gpui::ClickEvent::default(), window, cx)
                });
            row = if is_active {
                row.bg(theme::accent_soft())
                    .text_color(theme::accent())
                    .font_weight(gpui::FontWeight::MEDIUM)
            } else {
                row.text_color(theme::text())
                    .hover(|style| style.bg(theme::hover_bg()))
            };
            row = row
                .child(div().text_sm().child(icon))
                .child(div().flex_1().text_sm().child(name));
            nav = nav.child(row);
        }
        nav
    }

    /// 按分类派发分区（每个分区方法保持不动，仅重组归属）。
    fn category_sections(&self, category: usize, cx: &mut Context<Self>) -> Div {
        let mut col = div().flex().flex_col().gap_5().w_full();
        match category {
            // 通用
            0 => {
                col = col.child(self.buttons_section(cx));
            }
            // 布局
            1 => {
                col = col.child(self.layout_section(cx));
            }
            // 导航
            2 => {
                col = col
                    .child(self.tabs_section(cx))
                    .child(self.stepper_section(cx))
                    .child(self.nav_section(cx))
                    .child(self.menu_section(cx))
                    .child(self.antd_nav_section(cx));
            }
            // 数据录入
            3 => {
                col = col
                    .child(self.text_section(cx))
                    .child(self.selection_section(cx))
                    .child(self.slider_section(cx))
                    .child(self.select_section(cx))
                    .child(self.picker_section(cx))
                    .child(self.inputs_plus_section(cx))
                    .child(self.tags_auto_section(cx))
                    .child(self.textarea_section(cx))
                    .child(self.antd_input_section(cx))
                    .child(self.form_section(cx));
            }
            // 数据展示
            4 => {
                col = col
                    .child(self.badge_section())
                    .child(self.accordion_section(cx))
                    .child(self.data_section(cx))
                    .child(self.media_section())
                    .child(self.viz_section())
                    .child(self.advanced_charts_section())
                    .child(self.antd_display_section(cx));
            }
            // 反馈
            5 => {
                col = col
                    .child(self.feedback_section(cx))
                    .child(self.overlay_section(cx))
                    .child(self.placeholder_section(cx))
                    .child(self.antd_feedback_section(cx));
            }
            // AI 对话
            6 => {
                col = col.child(self.ai_chat_section(cx));
            }
            // 辅助与主题
            7 => {
                col = col
                    .child(self.theme_section(cx))
                    .child(self.misc_plus_section(cx))
                    .child(self.perf_section(cx));
            }
            // 综合演示
            8 => {
                col = col
                    .child(self.showcase_section(cx))
                    .child(self.align_section(cx))
                    .child(self.align_display_section())
                    .child(self.new_controls_section(cx));
            }
            // 终端与开发
            _ => {
                col = col
                    .child(self.terminal_section(cx))
                    .child(self.hotkey_section(cx))
                    .child(self.cmd_section(cx))
                    .child(self.code_section(cx))
                    .child(self.drop_section(cx))
                    .child(self.transfer_section(cx))
                    .child(self.titlebar_section(cx));
            }
        }
        col
    }

    fn ai_chat_section(&self, cx: &mut Context<Self>) -> Div {
        let questions = ["Sender 怎么用？", "如何切换主题？", "图表有哪些？"];

        // 消息气泡列表
        let mut messages = div().flex().flex_col().gap_3().w_full();
        for (ix, (is_user, text)) in self.ai_messages.iter().enumerate() {
            let mut bubble = Bubble::new(text.clone())
                .id(format!("ai-message-{ix}"))
                .avatar(if *is_user { "我" } else { "助手" });
            if *is_user {
                bubble = bubble.right();
            }
            messages = messages.child(bubble);
        }
        messages = messages.child(
            Bubble::new("")
                .id("ai-message-pending")
                .avatar("助手")
                .loading(true),
        );

        card()
            .child(section_title(
                "AI 对话 Welcome / Prompts / Bubble / Sender / Conversations / ThoughtChain",
            ))
            .child(caption(
                "对标 antd X：会话切换、提示词、消息气泡、发送框（Enter 发送 / Shift+Enter 换行）、思维链",
            ))
            .child(Divider::new())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap_4()
                    .w_full()
                    // 左：会话列表
                    .child(
                        div().w(px(200.)).flex_shrink_0().child(
                            Conversations::new(vec![
                                ConversationItem::new("c1", "新的对话").icon("💬"),
                                ConversationItem::new("c2", "代码审查").icon("🔍"),
                                ConversationItem::new("c3", "组件用法").icon("📦"),
                            ])
                            .active(self.ai_conversation.clone())
                            .on_select(cx.listener(|this, key: &gpui::SharedString, _, cx| {
                                this.ai_conversation = key.clone();
                                cx.notify();
                            })),
                        ),
                    )
                    // 右：欢迎 + 提示词 + 消息 + 发送框
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                Welcome::new("GPUI 助手").icon("🤖").description(
                                    "我可以帮你解答组件用法，点击下方提示词快速开始",
                                ),
                            )
                            .child(
                                Prompts::new(vec![
                                    PromptItem::new(questions[0]).icon("⌨"),
                                    PromptItem::new(questions[1]).icon("🎨"),
                                    PromptItem::new(questions[2]).icon("📊"),
                                ])
                                .on_click(cx.listener(move |this, ix: &usize, _, cx| {
                                    let q = questions[*ix];
                                    this.ai_messages.push((true, q.to_string()));
                                    this.ai_messages
                                        .push((false, format!("收到：{q}（演示回复）")));
                                    cx.notify();
                                })),
                            )
                            .child(Divider::new())
                            .child(messages)
                            .child(self.ai_sender.clone()),
                    ),
            )
            .child(Divider::new())
            .child(caption("气泡变体 Filled / Outlined / Shadow"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_3()
                    .w_full()
                    .child(Bubble::new("填充 Filled").id("bubble-filled").avatar("助"))
                    .child(
                        Bubble::new("描边 Outlined")
                            .id("bubble-outlined")
                            .avatar("助")
                            .variant(BubbleVariant::Outlined),
                    )
                    .child(
                        Bubble::new("阴影 Shadow")
                            .id("bubble-shadow")
                            .avatar("助")
                            .variant(BubbleVariant::Shadow),
                    ),
            )
            .child(Divider::new())
            .child(caption("思维链 ThoughtChain（点击标题展开/收起）"))
            .child(
                ThoughtChain::new(vec![
                    ThoughtNode::new("理解需求")
                        .description("解析用户输入，识别意图为组件用法咨询")
                        .status(ThoughtStatus::Success),
                    ThoughtNode::new("检索文档")
                        .description("在控件参考手册中定位 Sender 章节")
                        .status(ThoughtStatus::Success),
                    ThoughtNode::new("生成回复")
                        .description("组织答案并输出…")
                        .status(ThoughtStatus::Pending),
                ])
                .expanded(self.ai_thought_expanded.clone())
                .on_toggle(cx.listener(|this, ix: &usize, _, cx| {
                    if let Some(pos) = this.ai_thought_expanded.iter().position(|&i| i == *ix) {
                        this.ai_thought_expanded.remove(pos);
                    } else {
                        this.ai_thought_expanded.push(*ix);
                    }
                    cx.notify();
                })),
            )
    }

    fn viz_section(&self) -> Div {
        card()
            .child(section_title(
                "可视化 RingProgress / Statistic / BarChart / LineChart",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap_6()
                    .child(RingProgress::new(self.volume / 100.).label("音量"))
                    .child(
                        RingProgress::new(0.68)
                            .size(px(96.))
                            .thickness(px(10.))
                            .color(theme::success())
                            .label("完成率"),
                    )
                    .child(Statistic::new("月活跃用户", "12,847").trend(Trend::Up, "+12.5%"))
                    .child(
                        Statistic::new("平均响应", "238")
                            .suffix("ms")
                            .trend(Trend::Down, "-8.1%"),
                    ),
            )
            .child(caption("左侧环形进度与上方音量滑杆实时联动"))
            .child(Divider::new())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_5()
                    .child(
                        div().flex_1().child(
                            BarChart::new(vec![
                                BarDatum::new("周一", 32.),
                                BarDatum::new("周二", 48.),
                                BarDatum::new("周三", 41.),
                                BarDatum::new("周四", 66.),
                                BarDatum::new("周五", 58.),
                                BarDatum::new("周六", 74.),
                                BarDatum::new("周日", 62.),
                            ])
                            .height(px(150.)),
                        ),
                    )
                    .child(
                        div().flex_1().child(
                            LineChart::new(vec![12., 18., 15., 26., 22., 34., 30.])
                                .labels(vec![
                                    "一".into(),
                                    "二".into(),
                                    "三".into(),
                                    "四".into(),
                                    "五".into(),
                                    "六".into(),
                                    "日".into(),
                                ])
                                .area(true)
                                .dots(true)
                                .height(px(150.)),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(caption("迷你趋势 Sparkline"))
                    .child(
                        Sparkline::new(vec![5., 8., 6., 11., 9., 14., 12., 16.])
                            .w(px(180.))
                            .h(px(36.)),
                    )
                    .child(
                        Sparkline::new(vec![16., 12., 14., 9., 11., 6., 8., 5.])
                            .w(px(180.))
                            .h(px(36.))
                            .color(theme::danger())
                            .area(false),
                    ),
            )
    }

    fn advanced_charts_section(&self) -> Div {
        card()
            .child(section_title(
                "高级图表 PieChart / GaugeChart / ScatterChart / RadarChart / Heatmap / FunnelChart",
            ))
            .child(caption(
                "全部基于 GPUI 原生路径自绘，默认取主题 8 色分类色板，明暗主题自适应",
            ))
            .child(Divider::new())
            // 第一行：环形饼图 + 仪表盘
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap_6()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(caption("流量来源 · 环形图"))
                            .child(
                                PieChart::new(vec![
                                    PieDatum::new("直接访问", 335.),
                                    PieDatum::new("邮件营销", 234.),
                                    PieDatum::new("联盟广告", 154.),
                                    PieDatum::new("视频广告", 135.),
                                    PieDatum::new("搜索引擎", 548.),
                                ])
                                .donut(true)
                                .size(px(160.)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(caption("CPU 使用率 · 仪表盘（与上方音量滑杆联动）"))
                            .child(
                                GaugeChart::new(self.volume)
                                    .max(100.)
                                    .size(px(180.))
                                    .title("CPU 使用率")
                                    .zones(vec![
                                        (0.6, theme::success()),
                                        (0.85, theme::warning()),
                                        (1.0, theme::danger()),
                                    ]),
                            ),
                    ),
            )
            .child(Divider::new())
            // 第二行：散点图 + 雷达图
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_start()
                    .gap_6()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(caption("两组样本分布 · 散点图"))
                            .child(
                                ScatterChart::new(vec![
                                    ScatterSeries::new(
                                        "甲组",
                                        vec![
                                            ScatterPoint::new(1., 2.),
                                            ScatterPoint::new(2., 3.5),
                                            ScatterPoint::new(1.5, 3.),
                                            ScatterPoint::new(3., 4.5),
                                            ScatterPoint::new(2.5, 4.),
                                            ScatterPoint::new(3.5, 5.5),
                                            ScatterPoint::new(4., 5.),
                                            ScatterPoint::new(2., 2.5),
                                            ScatterPoint::new(3., 3.),
                                            ScatterPoint::new(4.5, 6.),
                                        ],
                                    ),
                                    ScatterSeries::new(
                                        "乙组",
                                        vec![
                                            ScatterPoint::new(5., 7.),
                                            ScatterPoint::new(6., 6.5),
                                            ScatterPoint::new(5.5, 8.),
                                            ScatterPoint::new(7., 7.5),
                                            ScatterPoint::new(6.5, 9.),
                                            ScatterPoint::new(7.5, 8.5),
                                            ScatterPoint::new(8., 9.5),
                                            ScatterPoint::new(6., 7.),
                                            ScatterPoint::new(7., 6.),
                                            ScatterPoint::new(8.5, 10.),
                                        ],
                                    ),
                                ])
                                .x_max(10.)
                                .y_max(11.)
                                .height(px(200.)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(caption("能力评估 · 雷达图"))
                            .child(
                                RadarChart::new(
                                    vec![
                                        "销售".into(),
                                        "管理".into(),
                                        "技术".into(),
                                        "客服".into(),
                                        "研发".into(),
                                        "市场".into(),
                                    ],
                                    vec![
                                        RadarSeries::new("预算分配", vec![80., 70., 90., 60., 85., 75.]),
                                        RadarSeries::new("实际开销", vec![65., 80., 75., 70., 90., 60.]),
                                    ],
                                )
                                .max(100.)
                                .size(px(220.)),
                            ),
                    ),
            )
            .child(Divider::new())
            // 第三行：热力图 + 漏斗图
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_start()
                    .gap_6()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(caption("一周时段活跃度 · 热力图"))
                            .child(
                                Heatmap::new(vec![
                                    vec![12., 35., 68., 82., 54., 20.],
                                    vec![18., 42., 75., 90., 61., 25.],
                                    vec![15., 38., 70., 85., 58., 22.],
                                    vec![22., 48., 80., 95., 66., 30.],
                                    vec![28., 55., 88., 99., 72., 38.],
                                    vec![40., 62., 70., 78., 85., 55.],
                                    vec![35., 50., 58., 64., 76., 48.],
                                ])
                                .row_labels(vec![
                                    "周一".into(),
                                    "周二".into(),
                                    "周三".into(),
                                    "周四".into(),
                                    "周五".into(),
                                    "周六".into(),
                                    "周日".into(),
                                ])
                                .col_labels(vec![
                                    "早".into(),
                                    "上午".into(),
                                    "中午".into(),
                                    "下午".into(),
                                    "晚".into(),
                                    "深夜".into(),
                                ])
                                .cell_size(px(26.)),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(caption("用户转化 · 漏斗图（右侧为相对上一阶段转化率）"))
                            .child(
                                FunnelChart::new(vec![
                                    FunnelStage::new("访问", 1000.),
                                    FunnelStage::new("注册", 620.),
                                    FunnelStage::new("激活", 410.),
                                    FunnelStage::new("付费", 280.),
                                    FunnelStage::new("复购", 150.),
                                ])
                                .height(px(220.)),
                            ),
                    ),
            )
    }

    fn layout_section(&self, cx: &mut Context<Self>) -> Div {
        // MenuItem / FilePicker 回调无 Entity 上下文，改用实体句柄
        let entity = cx.entity();
        card()
            .child(section_title("分割面板 Splitter / 步骤条 Steps"))
            .child(
                Splitter::new(
                    "h-split",
                    div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(theme::accent_soft())
                        .child("左栏"),
                    div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child("右栏"),
                )
                .ratio(self.split_ratio)
                .size(px(120.))
                .on_change(cx.listener(|this, ratio: &f32, _, cx| {
                    this.split_ratio = *ratio;
                    cx.notify();
                })),
            )
            .child(caption(format!(
                "左右分割 · 左栏占比 {:.0}%，拖动中间分隔条调整",
                self.split_ratio * 100.
            )))
            .child(
                div().h(px(140.)).child(
                    Splitter::new(
                        "v-split",
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(theme::accent_soft())
                            .child("上栏"),
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child("下栏"),
                    )
                    .vertical()
                    .ratio(self.vsplit_ratio)
                    .size(px(280.))
                    .on_change(cx.listener(|this, ratio: &f32, _, cx| {
                        this.vsplit_ratio = *ratio;
                        cx.notify();
                    })),
                ),
            )
            .child(caption(format!(
                "上下分割 · 上栏占比 {:.0}%",
                self.vsplit_ratio * 100.
            )))
            .child(Divider::new())
            .child(
                Steps::new(vec![
                    "填写信息".into(),
                    "确认订单".into(),
                    "支付".into(),
                    "完成".into(),
                ])
                .current(self.step_ix),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .child(
                        Button::new("上一步")
                            .secondary()
                            .small()
                            .disabled(self.step_ix == 0)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.step_ix = this.step_ix.saturating_sub(1);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("下一步")
                            .primary()
                            .small()
                            .disabled(self.step_ix >= 3)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.step_ix = (this.step_ix + 1).min(3);
                                cx.notify();
                            })),
                    ),
            )
            .child(Divider::new())
            .child(section_title(
                "右键菜单 ContextMenu / 文件选择 FilePicker / 命令面板",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .child(
                        ContextMenu::new(
                            "ctx-demo",
                            vec![
                                MenuItem::new("复制").on_click({
                                    let entity = entity.clone();
                                    move |_, cx| {
                                        let _ = entity.update(cx, |this, cx| {
                                            this.push_toast(ToastItem::new("已复制"), cx)
                                        });
                                    }
                                }),
                                MenuItem::new("粘贴").on_click({
                                    let entity = entity.clone();
                                    move |_, cx| {
                                        let _ = entity.update(cx, |this, cx| {
                                            this.push_toast(ToastItem::new("已粘贴"), cx)
                                        });
                                    }
                                }),
                                MenuItem::separator(),
                                MenuItem::new("删除").danger().on_click({
                                    let entity = entity.clone();
                                    move |_, cx| {
                                        let _ = entity.update(cx, |this, cx| {
                                            this.push_toast(ToastItem::new("已删除").danger(), cx)
                                        });
                                    }
                                }),
                            ],
                        )
                        .child(Button::new("右键点我").secondary()),
                    )
                    .child(FilePicker::new("选择文件").multiple(true).on_pick({
                        let entity = entity.clone();
                        move |paths, cx| {
                            let _ = entity.update(cx, |this, cx| {
                                this.picked = paths
                                    .iter()
                                    .map(|p| p.display().to_string())
                                    .collect::<Vec<_>>()
                                    .join("；");
                                cx.notify();
                            });
                        }
                    }))
                    .child(FilePicker::new("选择目录").directories(true).on_pick({
                        let entity = entity.clone();
                        move |paths, cx| {
                            let _ = entity.update(cx, |this, cx| {
                                this.picked = paths
                                    .iter()
                                    .map(|p| p.display().to_string())
                                    .collect::<Vec<_>>()
                                    .join("；");
                                cx.notify();
                            });
                        }
                    }))
                    .child(Button::new("命令面板").primary().on_click(cx.listener(
                        |this, _, window, cx| {
                            this.palette
                                .update(cx, |palette, cx| palette.open(window, cx));
                        },
                    ))),
            )
            .child(caption(if self.picked.is_empty() {
                "FilePicker 调用系统原生对话框，支持多选 / 选目录".to_string()
            } else {
                format!("已选择：{}", self.picked)
            }))
            .child(caption(format!(
                "命令面板：Ctrl+K 唤起，最近执行「{}」",
                if self.last_command.is_empty() {
                    "无"
                } else {
                    &self.last_command
                }
            )))
    }

    fn misc_plus_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("Markdown 渲染"))
            .child(Markdown::new(
                "## 常用语法\n\n支持 **粗体**、*斜体*、~~删除线~~、`行内代码` 与 [链接](https://gpui.rs)，裸链接 https://gpui.rs 自动识别。\n\n- 无序列表\n  - 嵌套项\n- 第二项\n\n1. 有序列表\n2. 第二项\n\n- [x] 已完成任务\n- [ ] 待办任务\n\n> 引用块文本\n\n```rust\nlet answer = 42;\n```\n\n| 左对齐 | 居中 | 右对齐 |\n| :-- | :-: | --: |\n| answer | 42 | 3.14 |",
            ))
            .child(section_title("代码块 CodeBlock / 键帽 Kbd / 链接 Link"))
            .child(
                CodeBlock::new("fn main() {\n    println!(\"Hello, GPUI!\");\n}")
                    .language("rust"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(caption("快捷键"))
                    .child(Kbd::combo("Ctrl+K"))
                    .child(caption("命令面板"))
                    .child(Kbd::new("Esc"))
                    .child(caption("关闭弹层"))
                    .child(Kbd::combo("Ctrl+Shift+P")),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .child(Link::new("GPUI 官网").href("https://gpui.rs"))
                    .child(
                        Link::new("Zed 仓库").href("https://github.com/zed-industries/zed"),
                    )
                    .child(Link::new("禁用链接").disabled()),
            )
            .child(Divider::new())
            .child(section_title("结果页 Result"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_5()
                    .child(div().flex_1().child(
                        Result::new(ResultStatus::Success, "提交成功")
                            .description("订单已生成，可在列表中查看。")
                            .action(
                                Button::new("返回列表").primary().small().on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.push_toast(ToastItem::new("返回列表"), cx)
                                    }),
                                ),
                            ),
                    ))
                    .child(div().flex_1().child(
                        Result::new(ResultStatus::Error, "支付失败")
                            .description("银行卡余额不足，请更换支付方式。")
                            .action(
                                Button::new("重试").secondary().small().on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.push_toast(
                                            ToastItem::new("重新发起支付").warning(),
                                            cx,
                                        )
                                    }),
                                ),
                            ),
                    )),
            )
    }

    fn showcase_section(&self, cx: &mut Context<Self>) -> Div {
        let assets = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/");
        card()
            .child(section_title("轮播 Carousel / 水印 Watermark"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_5()
                    .child(
                        div().w(px(320.)).child(
                            Carousel::new(
                                "demo-carousel",
                                vec![
                                    PathBuf::from(format!("{assets}sample.png")).into(),
                                    PathBuf::from(format!("{assets}sample.svg")).into(),
                                    PathBuf::from(format!("{assets}sample.png")).into(),
                                ],
                            )
                            .current(self.carousel_ix)
                            .height(px(180.))
                            .on_change(cx.listener(
                                |this, ix, _, cx| {
                                    this.carousel_ix = *ix;
                                    cx.notify();
                                },
                            )),
                        ),
                    )
                    .child(
                        div().flex_1().min_w(px(260.)).child(
                            Watermark::new("GPUI Widgets").child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .p_4()
                                    .border_1()
                                    .border_color(theme::border())
                                    .rounded_md()
                                    .child(
                                        div()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child("机密文档"),
                                    )
                                    .child(
                                        div().text_sm().text_color(theme::text_muted()).child(
                                            "水印铺满内容区，隔行错位平铺，不拦截子控件交互。",
                                        ),
                                    )
                                    .child(
                                        Button::new("水印下的按钮").secondary().small().on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.push_toast(
                                                    ToastItem::new("水印不拦截点击").success(),
                                                    cx,
                                                )
                                            }),
                                        ),
                                    ),
                            ),
                        ),
                    ),
            )
            .child(caption(format!(
                "轮播第 {} / 3 张，‹ › 循环切换，底部圆点直达",
                self.carousel_ix + 1
            )))
    }
    fn antd_nav_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("导航菜单 NavMenu / 悬浮按钮 FloatButton"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .child(
                        div().w(px(220.)).child(
                            NavMenu::new(
                                "nav",
                                vec![
                                    MenuEntry::new("home", "首页").icon("⌂"),
                                    MenuEntry::new("docs", "文档").icon("📄"),
                                    MenuEntry::new("settings", "设置")
                                        .icon("⚙")
                                        .child(MenuEntry::new("profile", "个人资料"))
                                        .child(MenuEntry::new("security", "安全中心")),
                                ],
                            )
                            .selected(self.nav_selected.clone())
                            .on_select(cx.listener(|this, key: &gpui::SharedString, _, cx| {
                                this.nav_selected = Some(key.clone());
                                cx.notify();
                            }))
                            .expanded(self.nav_expanded.clone())
                            .on_toggle(cx.listener(
                                |this, key: &gpui::SharedString, _, cx| {
                                    if let Some(pos) =
                                        this.nav_expanded.iter().position(|k| k == key)
                                    {
                                        this.nav_expanded.remove(pos);
                                    } else {
                                        this.nav_expanded.push(key.clone());
                                    }
                                    cx.notify();
                                },
                            )),
                        ),
                    )
                    .child(
                        div()
                            .relative()
                            .flex_1()
                            .h(px(160.))
                            .rounded_md()
                            .border_1()
                            .border_color(theme::border())
                            .bg(theme::bg())
                            .child(
                                div()
                                    .p_3()
                                    .child(caption("悬浮按钮固定在容器右下角，悬停显示提示")),
                            )
                            .child({
                                let entity = cx.entity();
                                FloatButton::new("fb", "↑")
                                    .primary()
                                    .tooltip("回到顶部")
                                    .on_click(move |_, cx| {
                                        let _ = entity.update(cx, |this, cx| {
                                            this.push_toast(ToastItem::new("已回到顶部"), cx);
                                        });
                                    })
                            }),
                    ),
            )
            .child(caption(format!(
                "当前选中：{}",
                self.nav_selected
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "无".to_string())
            )))
            .child(Divider::new())
            .child(section_title("排版 Typography / 间距 Space"))
            .child(Title::new(3, "三级标题"))
            .child(Paragraph::new(
                "GPUI 是 Zed 编辑器开源的 GPU 加速 UI 框架，本控件库在其上提供了 80 个通用控件。",
            ))
            .child(
                Space::new()
                    .child(Text::new("加粗").strong())
                    .child(Text::new("代码").code())
                    .child(Text::new("高亮").mark())
                    .child(Text::new("删除线").delete())
                    .child(Text::new("次要").secondary()),
            )
            .child(Divider::new())
            .child(section_title("栅格 Grid / 布局 Layout"))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        Row::new()
                            .gutter(8.)
                            .child(Col::new(8).child(grid_cell("8")))
                            .child(Col::new(8).child(grid_cell("8")))
                            .child(Col::new(8).child(grid_cell("8"))),
                    )
                    .child(
                        Row::new()
                            .gutter(8.)
                            .child(Col::new(12).child(grid_cell("12")))
                            .child(Col::new(12).child(grid_cell("12"))),
                    ),
            )
            .child(
                div()
                    .h(px(180.))
                    .rounded_md()
                    .overflow_hidden()
                    .border_1()
                    .border_color(theme::border())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .h_full()
                            .child(
                                Header::new().child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .child("Header"),
                                ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_1()
                                    .child(Sider::new(120.).child(caption("Sider")))
                                    .child(Content::new().child(caption("Content"))),
                            )
                            .child(Footer::new().child("Footer")),
                    ),
            )
    }

    fn antd_display_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("卡片 Card / 描述列表 Descriptions"))
            .child(
                Card::new("卡片标题")
                    .extra(Link::new("更多"))
                    .child("卡片内容区，可放置任意元素。")
                    .action(Button::new("操作一").secondary().small())
                    .action(Button::new("操作二").secondary().small()),
            )
            .child(
                Descriptions::new("用户信息")
                    .column(2)
                    .item("姓名", "张三")
                    .item("电话", "13800000000")
                    .item("地址", "浙江杭州")
                    .item("状态", "活跃"),
            )
            .child(
                Descriptions::untitled()
                    .bordered()
                    .column(1)
                    .item("订单号", "202611120001")
                    .item("金额", "¥ 199.00"),
            )
            .child(Divider::new())
            .child(section_title("列表 List / 角标 CountBadge"))
            .child(
                List::new(vec![
                    ListItem::new("任务一")
                        .description("明天截止")
                        .avatar("①")
                        .extra(Badge::new("紧急").danger()),
                    ListItem::new("任务二").description("下周截止").avatar("②"),
                ])
                .header("任务列表"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_6()
                    .items_center()
                    .child(
                        CountBadge::new(self.badge_count).child(
                            Button::new("消息").secondary().on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.badge_count += 1;
                                    cx.notify();
                                },
                            )),
                        ),
                    )
                    .child(CountBadge::new(120).child(Avatar::new("张")))
                    .child(CountBadge::new(0).dot().child(Avatar::new("李")))
                    .child(
                        CountBadge::new(8)
                            .color(rgb(0x1677ff).into())
                            .offset(4., 4.)
                            .child(Avatar::new("王")),
                    )
                    .child(caption(
                        "点击消息递增计数，120 显示为 99+；蓝色角标为自定义色 + 偏移",
                    )),
            )
            .child(Divider::new())
            .child(section_title("日历 Calendar / 二维码 QRCode"))
            .child(
                Calendar::new("cal", self.cal_month)
                    .value(self.cal_value)
                    .mode(self.cal_mode)
                    .on_month_change(cx.listener(|this, month, _, cx| {
                        this.cal_month = *month;
                        cx.notify();
                    }))
                    .on_select(cx.listener(|this, date, _, cx| {
                        this.cal_value = Some(*date);
                        cx.notify();
                    }))
                    .on_mode_change(cx.listener(|this, mode, _, cx| {
                        this.cal_mode = *mode;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .items_center()
                    .child(QRCode::new("https://ant.design").size(px(140.)))
                    .child(caption(
                        "纯 Rust 编码（byte 模式 / ECC L / v1-6），可扫码访问 antd 官网",
                    )),
            )
    }
    fn form_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("表单 Form / 校验 Rule"))
            .child(caption(
                "受控校验：提交时 validate + Rule 计算错误，交给 FormItem.error_opt 显示",
            ))
            .child(
                Form::new()
                    .id("form-basic")
                    .child(
                        FormItem::new("用户名")
                            .required()
                            .label_width(px(80.))
                            .error_opt(self.form_name_error.clone())
                            .child(div().flex_1().child(self.form_name.clone())),
                    )
                    .child(
                        FormItem::new("邮箱")
                            .required()
                            .label_width(px(80.))
                            .error_opt(self.form_email_error.clone())
                            .child(div().flex_1().child(self.form_email.clone())),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        Button::new("提交")
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                let name = this.form_name.read(cx).text().to_string();
                                let email = this.form_email.read(cx).text().to_string();
                                this.form_name_error = validate(
                                    &name,
                                    &[
                                        Rule::required("请输入用户名"),
                                        Rule::min_len(2, "至少 2 个字符"),
                                        Rule::max_len(12, "最多 12 个字符"),
                                    ],
                                );
                                this.form_email_error = validate(
                                    &email,
                                    &[
                                        Rule::required("请输入邮箱"),
                                        Rule::custom("邮箱需包含 @", |v| v.contains('@')),
                                    ],
                                );
                                this.form_ok = this.form_name_error.is_none()
                                    && this.form_email_error.is_none();
                                cx.notify();
                            })),
                    )
                    .child(caption(if self.form_ok {
                        "校验通过，提交成功！"
                    } else {
                        "用户名 2~12 字符，邮箱需含 @"
                    })),
            )
            .child(Divider::new())
            .child(caption("inline 横向表单（自动换行）："))
            .child(
                Form::new()
                    .id("form-inline")
                    .inline()
                    .gap(px(12.))
                    .child(
                        FormItem::new("关键词")
                            .child(div().w(px(200.)).child(self.form_query.clone())),
                    )
                    .child(Button::new("查询").secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            let q = this.form_query.read(cx).text().to_string();
                            this.push_toast(ToastItem::new(format!("查询：{q}")), cx);
                        },
                    ))),
            )
    }

    fn antd_input_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title("表单项 FormItem / 上传列表 UploadList"))
            .child(
                FormItem::new("姓名")
                    .id("form-sample-name")
                    .required()
                    .help("请输入真实姓名")
                    .child(self.sample_name_input.clone()),
            )
            .child(
                FormItem::new("邮箱")
                    .id("form-sample-email")
                    .error("邮箱格式不正确")
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .border_1()
                            .border_color(theme::danger())
                            .bg(theme::surface())
                            .text_sm()
                            .text_color(theme::text_muted())
                            .child("name@example.com"),
                    ),
            )
            .child(
                UploadList::new("uploads", self.upload_files.clone()).on_remove(cx.listener(
                    |this, ix, _, cx| {
                        this.upload_files.remove(*ix);
                        cx.notify();
                    },
                )),
            )
            .child(Divider::new())
            .child(section_title("级联选择 Cascader / 时间选择 TimePicker"))
            .child(
                field_row("地区").child(
                    div().flex_1().child(
                        Cascader::new("region", cascader_options())
                            .value(self.cascader_path.clone())
                            .open(self.cascader_open)
                            .placeholder("请选择地区")
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.cascader_open = *open;
                                cx.notify();
                            }))
                            .on_change(cx.listener(
                                |this, path: &Vec<gpui::SharedString>, _, cx| {
                                    this.cascader_path = path.clone();
                                    cx.notify();
                                },
                            )),
                    ),
                ),
            )
            .child(
                field_row("时间").child(
                    div().flex_1().child(
                        TimePicker::new("time")
                            .value(self.time_value)
                            .open(self.time_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.time_open = *open;
                                cx.notify();
                            }))
                            .on_change(cx.listener(|this, time, _, cx| {
                                this.time_value = Some(*time);
                                cx.notify();
                            })),
                    ),
                ),
            )
            .child(caption(format!(
                "已选时间：{}",
                self.time_value
                    .map(|(h, m, s)| format!("{h:02}:{m:02}:{s:02}"))
                    .unwrap_or_else(|| "无".to_string())
            )))
            .child(Divider::new())
            .child(section_title("树选择 TreeSelect"))
            .child(
                field_row("部门").child(
                    div().flex_1().child(
                        TreeSelect::new("dept", ts_nodes())
                            .value(self.ts_value.clone())
                            .expanded(self.ts_expanded.clone())
                            .open(self.ts_open)
                            .placeholder("请选择部门")
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.ts_open = *open;
                                cx.notify();
                            }))
                            .on_change(cx.listener(|this, id: &gpui::SharedString, _, cx| {
                                this.ts_value = Some(id.clone());
                                cx.notify();
                            }))
                            .on_toggle(cx.listener(|this, id: &gpui::SharedString, _, cx| {
                                if let Some(pos) = this.ts_expanded.iter().position(|k| k == id) {
                                    this.ts_expanded.remove(pos);
                                } else {
                                    this.ts_expanded.push(id.clone());
                                }
                                cx.notify();
                            })),
                    ),
                ),
            )
            .child(Divider::new())
            .child(section_title("穿梭框 Transfer"))
            .child(
                Transfer::new("transfer", transfer_items())
                    .titles("待办环节", "已选环节")
                    .target_keys(self.transfer_right.clone())
                    .selected(self.transfer_checked.clone())
                    .on_change(cx.listener(|this, keys: &Vec<gpui::SharedString>, _, cx| {
                        this.transfer_right = keys.clone();
                        cx.notify();
                    }))
                    .on_select(cx.listener(|this, keys: &Vec<gpui::SharedString>, _, cx| {
                        this.transfer_checked = keys.clone();
                        cx.notify();
                    })),
            )
            .child(caption("勾选条目后点击中间按钮左右移动"))
    }

    fn antd_feedback_section(&self, cx: &mut Context<Self>) -> Div {
        card()
            .child(section_title(
                "气泡确认 Popconfirm / 通知 Notification / 引导 Tour",
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_3()
                    .child(
                        Popconfirm::new("del", "确定删除吗？")
                            .description("删除后不可恢复")
                            .open(self.popconfirm_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.popconfirm_open = *open;
                                cx.notify();
                            }))
                            .on_confirm({
                                let entity = cx.entity();
                                move |_, cx| {
                                    let _ = entity.update(cx, |this, cx| {
                                        this.push_toast(ToastItem::new("已删除").success(), cx);
                                    });
                                }
                            })
                            .child(Button::new("删除").danger()),
                    )
                    .child(Button::new("发送通知").secondary().on_click(cx.listener(
                        |this, _, _, cx| {
                            let entity = cx.entity();
                            this.notifs.push(
                                NotificationItem::new("新消息")
                                    .success()
                                    .description("你有一条新的待办事项。")
                                    .action("查看详情", move |_, cx| {
                                        let _ = entity.update(cx, |this, cx| {
                                            this.push_toast(ToastItem::new("查看详情被点击"), cx);
                                        });
                                    }),
                            );
                            cx.notify();
                        },
                    )))
                    .child(Button::new("开始引导").primary().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.tour_open = true;
                            this.tour_step = 0;
                            cx.notify();
                        },
                    ))),
            )
            .child(Divider::new())
            .child(section_title("提及输入 Mentions"))
            .child(caption("输入 @ 触发候选名单，选中后自动替换为「@名称 」"))
            .child(self.mentions.clone())
            .child(caption(format!(
                "最近提及：{}",
                if self.mention_pick.is_empty() {
                    "无"
                } else {
                    &self.mention_pick
                }
            )))
    }
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = div()
            .flex()
            .flex_col()
            .w_full()
            .gap_5()
            .p_6()
            // 页头
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("GPUI 通用控件库"),
                    )
                    .child(caption(
                        "基于 Zed GPUI 框架 · Rust 编写 · GPU 加速渲染 · Ctrl+K 命令面板",
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::accent())
                            .child(format!(
                                "{} {}",
                                CATEGORIES[self.active_category].1,
                                CATEGORIES[self.active_category].0
                            )),
                    )
                    .child(caption(CATEGORIES[self.active_category].2)),
            )
            .child(self.category_sections(self.active_category, cx));

        let mut root = div()
            .id("gallery")
            .relative()
            .size_full()
            .bg(theme::bg())
            .track_focus(&self.focus_handle(cx))
            .text_color(theme::text())
            .on_action(cx.listener(
                |this, _: &gpui_widgets::command_palette::Toggle, window, cx| {
                    this.palette
                        .update(cx, |palette, cx| palette.open(window, cx));
                },
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .size_full()
                    // 左侧分类导航
                    .child(
                        div()
                            .w(px(220.))
                            .flex_shrink_0()
                            .h_full()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_3()
                            .bg(theme::surface())
                            .border_r_1()
                            .border_color(theme::border())
                            .child(
                                div()
                                    .px_3()
                                    .pt_2()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_muted())
                                    .child("控件分类"),
                            )
                            .child(self.category_nav(cx)),
                    )
                    // 右侧内容滚动区
                    .child(
                        div()
                            .id("gallery-scroll")
                            .flex_1()
                            .min_w(px(0.))
                            .h_full()
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll_handle)
                            .child(content),
                    ),
            )
            // 主纵向滚动条：主题化自绘，overlay 在滚动区右缘
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right_0()
                    .child(Scrollbar::vertical(
                        "gallery-vsb",
                        self.scroll_handle.clone(),
                    )),
            );

        // Modal 渲染在滚动容器之外，deferred 保证其覆盖所有内容
        if self.modal_open {
            // on_close 签名为 (&mut Window, &mut App)，不适用 cx.listener，改用实体句柄
            let entity = cx.entity();
            root = root.child(
                Modal::new("确认操作")
                    .width(px(480.))
                    .on_close(move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.modal_open = false;
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .text_sm()
                            .child(format!("当前数量为 {:.0}，确定要提交吗？", self.step_val)),
                    )
                    .child(caption("点击遮罩或右上角 ✕ 可关闭"))
                    .footer(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .child(Button::new("取消").secondary().on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.modal_open = false;
                                    cx.notify();
                                },
                            )))
                            .child(Button::new("确定").primary().on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.modal_open = false;
                                    cx.notify();
                                },
                            ))),
                    ),
            );
        }

        // Drawer / Toast 同样渲染在滚动容器之外，deferred 置顶
        if self.drawer_open {
            let entity = cx.entity();
            root = root.child(
                Drawer::new("筛选条件")
                    .placement(if self.drawer_left {
                        DrawerPlacement::Left
                    } else {
                        DrawerPlacement::Right
                    })
                    .on_close(move |_, cx| {
                        entity.update(cx, |this, cx| {
                            this.drawer_open = false;
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .text_sm()
                            .child("抽屉内容区：可放置表单、筛选器等任意控件。"),
                    )
                    .child(
                        Checkbox::new("复用主视图的「同意条款」状态", self.agreed).on_change(
                            cx.listener(|this, checked, _, cx| {
                                this.agreed = *checked;
                                cx.notify();
                            }),
                        ),
                    )
                    .child(caption("点击遮罩或右上角 ✕ 可关闭"))
                    .footer(Button::new("应用").primary().on_click(cx.listener(
                        |this, _, _, cx| {
                            this.drawer_open = false;
                            this.push_toast(ToastItem::new("筛选已应用").success(), cx);
                        },
                    ))),
            );
        }

        if !self.toasts.is_empty() {
            root = root.child(ToastStack::new(self.toasts.clone()).on_close(cx.listener(
                |this, ix, _, cx| {
                    this.toasts.remove(*ix);
                    cx.notify();
                },
            )));
        }

        // 命令面板同样渲染在滚动容器之外，内部 deferred 置顶
        root = root.child(self.palette.clone());

        // 通知堆栈与漫游引导同样置顶绘制
        if !self.notifs.is_empty() {
            root = root.child(
                NotificationStack::new(self.notifs.clone()).on_close(cx.listener(
                    |this, ix, _, cx| {
                        this.notifs.remove(*ix);
                        cx.notify();
                    },
                )),
            );
        }
        if self.tour_open {
            root = root.child(
                Tour::new(tour_steps())
                    .current(self.tour_step)
                    .on_change(cx.listener(|this, step, _, cx| {
                        this.tour_step = *step;
                        cx.notify();
                    }))
                    .on_close({
                        let entity = cx.entity();
                        move |_, cx| {
                            let _ = entity.update(cx, |this, cx| {
                                this.tour_open = false;
                                cx.notify();
                            });
                        }
                    }),
            );
        }

        root
    }
}

/// 自绘标题栏演示窗口：整窗无系统标题栏，顶部完全由 [`TitleBar`] 绘制。
struct TitleBarDemoWindow {
    clicks: usize,
}

impl gpui::Render for TitleBarDemoWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme::bg())
            .child(
                TitleBar::new("自绘标题栏 · 演示窗口")
                    .id("titlebar-demo")
                    .icon("🪟")
                    .height(px(40.))
                    .extra(
                        div()
                            .ml_2()
                            .text_xs()
                            .text_color(theme::danger())
                            .child("● 未保存"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_6()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::text())
                            .child("TitleBar 自绘标题栏"),
                    )
                    .child(div().text_sm().text_color(theme::text_muted()).child(
                        "拖动顶部标题栏可移动窗口；双击标题栏最大化 / 还原；\
                             ─ □ ✕ 按钮由系统非客户区接管（含贴靠布局与右键系统菜单）。",
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(
                                Button::new(format!("点击计数：{}", self.clicks))
                                    .primary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clicks += 1;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("关闭本窗口")
                                    .secondary()
                                    .on_click(|_, window, _| window.remove_window()),
                            ),
                    ),
            )
    }
}

/// 打开自绘标题栏演示窗口（无系统标题栏）。
fn open_titlebar_demo_window(cx: &mut App) {
    let mut options = transparent_titlebar_options("自绘标题栏演示");
    options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
        None,
        size(px(560.), px(340.)),
        cx,
    )));
    options.window_min_size = Some(size(px(420.), px(260.)));
    if let Err(err) = cx.open_window(options, |_, cx| {
        cx.new(|_| TitleBarDemoWindow { clicks: 0 })
    }) {
        eprintln!("打开自绘标题栏窗口失败: {err:?}");
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        gpui_widgets::text_input::bind_keys(cx);
        gpui_widgets::text_area::bind_keys(cx);
        gpui_widgets::command_palette::bind_keys(cx);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| cx.quit());

        let bounds = Bounds::centered(None, size(px(920.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("GPUI Widgets Gallery".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| Gallery {
                    click_count: 0,
                    agreed: false,
                    newsletter: false,
                    notifications: true,
                    plan: 0,
                    volume: 30.,
                    name_input: cx.new(|cx| TextInput::new(cx).placeholder("请输入姓名")),
            sample_name_input: cx.new(|cx| TextInput::new(cx).placeholder("请输入姓名")),
                    tab_ix: 0,
                    seg_ix: 0,
                    city: 0,
                    select_open: false,
                    skills: vec![0, 1],
                    skills_open: false,
                    modal_open: false,
                    step_val: 1.,
                    tags: default_tags(),
                    acc_open: vec![0],
                    page: 1,
                    score: 3,
                    table_row: None,
                    table_sort: None,
                    table_checked: Vec::new(),
                    table_page: 1,
                    table_expanded: Vec::new(),
                    range_sel: Vec::new(),
                    close_tabs: vec!["文件 1".into(), "文件 2".into(), "文件 3".into()],
                    close_tab_ix: 0,
                    drag_tabs: vec![
                        "文件 1".into(),
                        "文件 2".into(),
                        "文件 3".into(),
                        "文件 4".into(),
                    ],
                    drag_active: 0,
                    transfers: demo_transfers(),
                    search_city: None,
                    search_city_open: false,
                    marked_val: 50.,
                    vert_val: 30.,
                    drawer_left: false,
                    clear_input: cx.new(|cx| {
                        TextInput::new(cx)
                            .placeholder("输入域名")
                            .prefix("https://")
                            .suffix(".com")
                            .allow_clear(true)
                            .max_length(10)
                    }),
                    error_input: cx.new(|cx| {
                        TextInput::new(cx).placeholder("错误态：边框变红").error(true)
                    }),
                    disabled_input: cx.new(|cx| {
                        TextInput::new(cx).placeholder("禁用：不可聚焦输入").disabled(true)
                    }),
                    readonly_input: cx.new(|cx| {
                        TextInput::new(cx).placeholder("只读：可选中不可编辑").readonly(true)
                    }),
                    form_name: cx.new(|cx| TextInput::new(cx).placeholder("2~12 个字符")),
                    form_email: cx.new(|cx| TextInput::new(cx).placeholder("name@example.com")),
                    form_query: cx.new(|cx| TextInput::new(cx).placeholder("输入关键词")),
                    form_name_error: None,
                    form_email_error: None,
                    form_ok: false,
                    tree_expanded: vec!["src".to_string()],
                    tree_selected: None,
                    tree_nodes: default_tree_nodes(),
                    tree_checked: Vec::new(),
                    menu_open: false,
                    popover_open: false,
                    date: None,
                    date_open: false,
                    date_month: None,
                    range: None,
                    range_open: false,
                    range_month: None,
                    color: rgb(0x3b82f6),
                    color_open: false,
                    dark_mode: false,
                    theme_accent: rgb(0x3b82f6),
                    theme_accent_open: false,
                    drawer_open: false,
                    toasts: Vec::new(),
                    toast_count: 0,
                    password: cx.new(|cx| PasswordInput::new(cx).placeholder("请输入密码")),
                    search: {
                        let entity: Entity<Gallery> = cx.entity();
                        cx.new(|cx| {
                            SearchInput::new(cx)
                                .placeholder("搜索控件…")
                                .on_search(move |text, _, cx| {
                                    let text = text.to_string();
                                    let _ = entity.update(cx, |this, cx| {
                                        this.last_search = text;
                                        cx.notify();
                                    });
                                })
                        })
                    },
                    number: {
                        let entity = cx.entity();
                        cx.new(|cx| {
                            NumberInput::new(cx)
                                .placeholder("0 ~ 100")
                                .width(200.)
                                .min(0.)
                                .max(100.)
                                .step(5.)
                                .on_change(move |value, _, cx| {
                                    let _ = entity.update(cx, |this, cx| {
                                        this.number_val = value;
                                        cx.notify();
                                    });
                                })
                        })
                    },
                    tag_input: {
                        let entity = cx.entity();
                        cx.new(|cx| {
                            TagInput::new(cx)
                                .placeholder("输入后回车添加")
                                .initial_tags(default_input_tags())
                                .on_change(move |tags, _, cx| {
                                    let tags = tags.clone();
                                    let _ = entity.update(cx, |this, cx| {
                                        this.input_tags = tags;
                                        cx.notify();
                                    });
                                })
                        })
                    },
                    autocomplete: {
                        let entity = cx.entity();
                        cx.new(|cx| {
                            Autocomplete::new(
                                cx,
                                vec![
                                    "GPUI".into(),
                                    "GTK".into(),
                                    "Qt".into(),
                                    "Electron".into(),
                                    "Tauri".into(),
                                    "Slint".into(),
                                    "Iced".into(),
                                    "Flutter".into(),
                                ],
                            )
                            .placeholder("输入框架名过滤…")
                            .on_select(move |label, _, cx| {
                                let label = label.to_string();
                                let _ = entity.update(cx, |this, cx| {
                                    this.auto_pick = label;
                                    cx.notify();
                                });
                            })
                        })
                    },
                    text_area: cx.new(|cx| {
                        TextArea::new(cx)
                            .placeholder("多行文本，支持换行 / 选区 / 中文输入法")
                            .height(px(140.))
                    }),
                    palette: {
                        let entity = cx.entity();
                        cx.new(|cx| {
                            CommandPalette::new(
                                cx,
                                vec![
                                    PaletteItem::new("新建文件").hint("Ctrl+N"),
                                    PaletteItem::new("打开设置"),
                                    PaletteItem::new("导出报表"),
                                    PaletteItem::new("清空通知"),
                                    PaletteItem::new("关于控件库"),
                                ],
                            )
                            .on_select(move |label, _, cx| {
                                let label = label.to_string();
                                let _ = entity.update(cx, |this, cx| {
                                    this.last_command = label.clone();
                                    this.push_toast(
                                        ToastItem::new(format!("执行命令：{label}")).success(),
                                        cx,
                                    );
                                });
                            })
                        })
                    },
                    split_ratio: 0.5,
                    vsplit_ratio: 0.5,
                    step_ix: 0,
                    carousel_ix: 0,
                    last_search: String::new(),
                    number_val: None,
                    input_tags: default_input_tags(),
                    auto_pick: String::new(),
                    picked: String::new(),
                    last_command: String::new(),
                    nav_selected: None,
                    nav_expanded: vec!["settings".into()],
                    cal_month: (2026, 11),
                    cal_value: None,
                    cal_mode: CalendarMode::Month,
                    badge_count: 5,
                    cascader_path: Vec::new(),
                    cascader_open: false,
                    time_value: None,
                    time_open: false,
                    ts_value: None,
                    ts_expanded: vec!["tech".into()],
                    ts_open: false,
                    transfer_right: Vec::new(),
                    transfer_checked: Vec::new(),
                    upload_files: default_uploads(),
                    popconfirm_open: false,
                    notifs: Vec::new(),
                    tour_open: false,
                    tour_step: 0,
                    mentions: {
                        let entity: Entity<Gallery> = cx.entity();
                        cx.new(|cx| {
                            Mentions::new(cx)
                                .options(vec![
                                    "张三".into(),
                                    "李四".into(),
                                    "王五".into(),
                                    "赵六".into(),
                                ])
                                .on_select(move |name, _, cx| {
                                    let name = name.to_string();
                                    let _ = entity.update(cx, |this, cx| {
                                        this.mention_pick = name;
                                        cx.notify();
                                    });
                                })
                        })
                    },
                    mention_pick: String::new(),
                    align_tab: 0,
                    align_seg: 0,
                    align_switch: true,
                    align_page: 1,
                    anchor_key: "basic".into(),
                    countdown: Gallery::new_countdown(cx),
                    #[cfg(windows)]
                    video: cx.new(|cx| {
                        VideoPlayer::new(
                            cx,
                            concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/sample.avi"),
                        )
                    }),
                    ai_sender: {
                        let entity = cx.entity();
                        cx.new(|cx| {
                            Sender::new(cx).on_send(move |text, _window, cx| {
                                let _ = entity.update(cx, |this, cx| {
                                    this.ai_messages.push((true, text.to_string()));
                                    this.ai_messages
                                        .push((false, format!("收到：{text}（演示回复）")));
                                    cx.notify();
                                });
                            })
                        })
                    },
                    ai_messages: vec![
                        (false, "你好！我是 GPUI 助手，可以帮你解答组件用法。".to_string()),
                        (true, "Sender 组件怎么用？".to_string()),
                        (
                            false,
                            "用 cx.new 创建 Sender，设置 on_send 回调即可。Enter 发送，Shift+Enter 换行。"
                                .to_string(),
                        ),
                    ],
                    ai_conversation: "c1".into(),
                    ai_thought_expanded: vec![0],
                    fps_meter: cx.new(|cx| FpsMeter::new(cx)),
                    terminal: {
                        let gallery = cx.entity();
                        cx.new(|cx| {
                            let term_entity = cx.entity();
                            let line = Rc::new(RefCell::new(String::new()));
                            let line_cb = line.clone();
                            let g = gallery.clone();
                            let mut t = Terminal::new(cx)
                                .cols(90)
                                .rows(22)
                                .on_title(move |title, _w, cx| {
                                    let _ = g.update(cx, |this, cx| {
                                        this.term_title = title.to_string();
                                        cx.notify();
                                    });
                                })
                                .on_input(move |bytes, _w, cx| {
                                    let mut line = line_cb.borrow_mut();
                                    for &b in bytes {
                                        match b {
                                            0x0d => {
                                                let cmd = line.clone();
                                                line.clear();
                                                term_entity.update(cx, |t, cx| {
                                                    t.write_str("\r\n", cx);
                                                    let c = cmd.trim();
                                                    if c == "clear" {
                                                        t.clear(cx);
                                                    } else if c == "sel" {
                                                        t.write_str(
                                                            "the quick brown fox jumps over the lazy dog\r\n",
                                                            cx,
                                                        );
                                                        t.write_str(TERM_PROMPT, cx);
                                                        let row = t.screen_top()
                                                            + t.cursor_row().saturating_sub(1);
                                                        t.set_selection((row, 4), (row, 19), cx);
                                                        return;
                                                    } else if let Some(w) = c.strip_prefix("find ") {
                                                        t.set_search(w, cx);
                                                    } else if c == "next" {
                                                        t.search_next(cx);
                                                    } else if c == "prev" {
                                                        t.search_prev(cx);
                                                    } else if c == "scroll" {
                                                        for i in 0..40 {
                                                            t.write_str(
                                                                &format!("scrollback line {i}\r\n"),
                                                                cx,
                                                            );
                                                        }
                                                    } else {
                                                        t.write_str(&shell_output(c), cx);
                                                    }
                                                    t.write_str(TERM_PROMPT, cx);
                                                });
                                            }
                                            0x7f => {
                                                if line.pop().is_some() {
                                                    term_entity.update(cx, |t, cx| {
                                                        t.write_str("\x08 \x08", cx);
                                                    });
                                                }
                                            }
                                            0x03 => {
                                                line.clear();
                                                term_entity.update(cx, |t, cx| {
                                                    t.write_str("^C\r\n", cx);
                                                    t.write_str(TERM_PROMPT, cx);
                                                });
                                            }
                                            _ if b >= 0x20 => {
                                                line.push(b as char);
                                                term_entity.update(cx, |t, cx| t.write(&[b], cx));
                                            }
                                            _ => {}
                                        }
                                    }
                                });
                            t.write_str(TERM_BANNER, cx);
                            t.write_str(TERM_PROMPT, cx);
                            t
                        })
                    },
                    term_search: cx.new(|cx| TextInput::new(cx).placeholder("搜索终端内容")),
                    term_title: String::new(),
                    hotkey_save: {
                        let g = cx.entity();
                        cx.new(|cx| {
                            let mut h = HotkeyInput::new(cx)
                                .id("hotkey-save")
                                .placeholder("点击录入保存快捷键")
                                .on_change(move |value, _w, cx| {
                                    let v = value.to_string();
                                    g.update(cx, |this, cx| {
                                        this.hotkey_log = v;
                                        cx.notify();
                                    });
                                });
                            h.set_binding("ctrl-s", cx);
                            h
                        })
                    },
                    hotkey_palette: {
                        let g = cx.entity();
                        cx.new(|cx| {
                            let mut h = HotkeyInput::new(cx)
                                .id("hotkey-palette")
                                .placeholder("需含 Ctrl/Alt/Win")
                                .require_modifier(true)
                                .on_change(move |value, _w, cx| {
                                    let v = value.to_string();
                                    g.update(cx, |this, cx| {
                                        this.hotkey_log = v;
                                        cx.notify();
                                    });
                                });
                            h.set_binding("ctrl-shift-p", cx);
                            h
                        })
                    },
                    hotkey_disabled: cx.new(|cx| {
                        let mut h = HotkeyInput::new(cx)
                            .id("hotkey-disabled")
                            .disabled(true);
                        h.set_binding("alt-f4", cx);
                        h
                    }),
                    hotkey_log: String::new(),
                    cmd_bar: {
                        let g = cx.entity();
                        cx.new(|cx| {
                            CommandBar::new(cx)
                                .id("cmd-bar")
                                .prompt("❯")
                                .placeholder("输入命令，Enter 提交；↑↓ 翻历史；Tab 补全片段")
                                .snippets([
                                    "git status",
                                    "cargo build --release",
                                    "cargo test",
                                    "ls -la",
                                    "help",
                                ])
                                .on_submit(move |cmd, _w, cx| {
                                    let c = cmd.to_string();
                                    g.update(cx, |this, cx| {
                                        this.cmd_runs.push(c);
                                        if this.cmd_runs.len() > 6 {
                                            this.cmd_runs.remove(0);
                                        }
                                        cx.notify();
                                    });
                                })
                        })
                    },
                    cmd_runs: Vec::new(),
                    code_editor: cx.new(|cx| {
                        CodeEditor::new(cx, CodeLanguage::Rust)
                            .height(px(300.))
                            .initial_text(DEMO_RUST)
                    }),
                    code_lang: CodeLanguage::Rust,
                    drop_files: Vec::new(),
                    drop_rejected: 0,
                    active_category: 0,
                    scroll_handle: ScrollHandle::new(),
                    focus_handle: cx.focus_handle(),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
