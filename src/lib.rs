//! # gpui-widgets
//!
//! 基于 GPUI（Zed 的 GPU 加速 UI 框架）的通用控件库。
//!
//! 控件分两类：
//! - **无状态控件**（Button / Checkbox / Switch / Radio / Slider / ProgressBar / Badge）：
//!   直接接收当前值与回调，在 `render` 中作为元素使用，状态由父视图持有。
//! - **有状态控件**（TextInput）：本身是一个 `Entity`，内部维护光标、选区与 IME 状态，
//!   通过 `cx.new(|cx| TextInput::new(cx))` 创建后作为子视图渲染。
//!
//! 使用前请在应用启动时调用 [`text_input::bind_keys`] 注册文本编辑快捷键；
//! 若使用 [`TextArea`]，还需调用 [`text_area::bind_keys`]。

//! 源码按族放在子目录中，族的模块全部再导出到 crate 根，
//! 因此 `gpui_widgets::button`、`gpui_widgets::tree_select` 等原有路径不变。

pub(crate) mod a11y;

/// 基础控件：按钮 / 开关 / 选择控件 / 滑块 / 滚动条。
pub mod controls {
    pub mod button;
    pub mod checkbox;
    pub mod float_button;
    pub mod radio;
    pub mod scrollbar;
    pub mod slider;
    pub mod stepper;
    pub mod switch;
    pub mod toolbar;
}
pub use controls::*;

/// 文本与键盘输入。
pub mod input {
    pub mod autocomplete;
    pub mod hotkey_input;
    pub mod mentions;
    pub mod number_input;
    pub mod otp_input;
    pub mod password_input;
    pub mod search_input;
    pub mod tag_input;
    pub mod text_area;
    pub mod text_input;
}
pub use input::*;

/// 选择器与树：下拉 / 级联 / 树 / 穿梭框。
pub mod selection {
    pub mod cascader;
    pub mod select;
    pub mod transfer;
    pub mod transfer_manager;
    pub mod tree;
    pub mod tree_select;
}
pub use selection::*;

/// 日期 / 时间 / 颜色选择。
pub mod pickers {
    pub mod calendar;
    pub mod color_picker;
    pub mod date_picker;
    pub mod range_picker;
    pub mod time_picker;
}
pub use pickers::*;

/// 布局与容器：栅格 / 分隔 / 表单 / 卡片 / 列表。
pub mod containers {
    pub mod accordion;
    pub mod affix;
    pub mod anchor;
    pub mod card;
    pub mod flex;
    pub mod form;
    pub mod form_item;
    pub mod grid;
    pub mod layout;
    pub mod list;
    pub mod space;
    pub mod splitter;
    pub mod status_bar;
}
pub use containers::*;

/// 导航：菜单 / 标签页 / 分页 / 步骤条。
pub mod navigation {
    pub mod breadcrumb;
    pub mod menu;
    pub mod menu_bar;
    pub mod nav_menu;
    pub mod pagination;
    pub mod segmented;
    pub mod steps;
    pub mod tabs;
}
pub use navigation::*;

/// 反馈：提示 / 通知 / 进度 / 骨架屏 / 弹窗。
pub mod feedback {
    pub mod alert;
    pub mod drawer;
    pub mod empty;
    pub mod modal;
    pub(crate) mod notice_timer;
    pub mod notification;
    pub mod notification_center;
    pub mod popconfirm;
    pub mod progress;
    pub mod result;
    pub mod ring_progress;
    pub mod skeleton;
    pub mod spinner;
    pub mod toast;
    pub mod tour;
}
pub use feedback::*;

/// 浮层：气泡框 / 下拉菜单 / 气泡卡片。
pub mod overlay {
    pub mod context_menu;
    pub mod popover;
    pub(crate) mod popup;
    pub mod tooltip;
}
pub(crate) use overlay::popup;
pub use overlay::*;

/// 展示：头像 / 徽标 / 表格 / 统计 / 时间线 / 上传等。
pub mod display {
    pub mod avatar;
    pub mod badge;
    pub mod carousel;
    pub mod countdown;
    pub mod descriptions;
    pub mod diff;
    pub mod image;
    pub mod kbd;
    pub mod link;
    pub mod markdown;
    pub mod markdown_editor;
    pub mod misc;
    pub mod qrcode;
    pub mod rating;
    pub mod statistic;
    pub mod svg_path;
    pub mod table;
    pub mod tag;
    pub mod timeline;
    pub mod tree_table;
    pub mod typography;
    pub mod upload;
    pub mod upload_list;
    pub mod watermark;
}
pub use display::*;

/// 库内自研（antd 无对应）控件：对话气泡 / 代码块 / 命令面板 / 终端 / 标题栏等。
pub mod custom {
    pub mod bubble;
    pub mod code_block;
    pub mod code_editor;
    pub mod command_bar;
    pub mod command_palette;
    pub mod conversations;
    pub mod drop_zone;
    pub mod file_picker;
    pub mod fps;
    pub mod prompts;
    pub mod sender;
    pub mod terminal;
    pub mod thought_chain;
    pub mod title_bar;
    #[cfg(windows)]
    pub mod video;
    pub mod welcome;
}
pub use custom::*;

/// 图表。
pub mod charts {
    pub mod chart;
    pub(crate) mod chart_util;
    pub mod funnel;
    pub mod gauge;
    pub mod heatmap;
    pub mod pie_chart;
    pub mod radar;
    pub mod scatter;
}
pub(crate) use charts::chart_util;
pub use charts::*;

pub mod theme;

pub use accordion::{Accordion, AccordionItem};
pub use affix::Affix;
pub use alert::{Alert, AlertVariant};
pub use anchor::{Anchor, AnchorItem};
pub use autocomplete::Autocomplete;
pub use avatar::Avatar;
pub use badge::{Badge, BadgeSize, BadgeStatus, BadgeVariant, CountBadge};
pub use breadcrumb::Breadcrumb;
pub use bubble::{Bubble, BubblePlacement, BubbleVariant};
pub use button::{Button, ButtonShape, ButtonSize, ButtonVariant};
pub use calendar::{Calendar, CalendarMode};
pub use card::Card;
pub use carousel::Carousel;
pub use cascader::{Cascader, CascaderNode};
pub use chart::{BarChart, BarDatum, LineChart, Sparkline};
pub use checkbox::Checkbox;
pub use code_block::{CodeAction, CodeBlock};
pub use code_editor::{CodeEditor, CodeLanguage};
pub use color_picker::ColorPicker;
pub use command_bar::{
    CommandBar, CommandSnippet, SnippetEvent, SnippetIcon, group_snippets, panel_groups,
    snippet_can_save, sorted_snippets,
};
pub use command_palette::{CommandPalette, PaletteItem};
pub use context_menu::ContextMenu;
pub use conversations::{ConversationItem, Conversations};
pub use countdown::Countdown;
pub use date_picker::{Date, DatePicker};
pub use descriptions::Descriptions;
pub use diff::{DiffKind, DiffLine, DiffMode, DiffStats, DiffView, diff_lines, diff_stats};
pub use display::image::ImageView;
pub use drawer::{Drawer, DrawerPlacement};
pub use drop_zone::DropZone;
pub use empty::Empty;
pub use file_picker::FilePicker;
pub use flex::Flex;
pub use float_button::FloatButton;
pub use form::{Form, Rule, validate};
pub use form_item::FormItem;
pub use fps::FpsMeter;
pub use funnel::{FunnelChart, FunnelStage};
pub use gauge::GaugeChart;
pub use grid::{Col, Row};
pub use heatmap::Heatmap;
pub use hotkey_input::HotkeyInput;
pub use kbd::Kbd;
pub use layout::{Content, Footer, Header, Layout, Sider};
pub use link::Link;
pub use list::{List, ListItem};
pub use markdown::Markdown;
pub use markdown_editor::{MarkdownEditor, MarkdownEditorMode};
pub use mentions::Mentions;
pub use menu::{DropdownMenu, MenuItem};
pub use menu_bar::{MenuBar, MenuBarItem};
pub use misc::{Divider, card, section_title};
pub use modal::Modal;
pub use nav_menu::{MenuEntry, NavMenu};
pub use notification::{
    NotificationConfig, NotificationDismissed, NotificationHost, NotificationItem,
    NotificationStack, NotificationVariant, default_notification_duration,
};
pub use notification_center::NotificationCenter;
pub use number_input::NumberInput;
pub use otp_input::OtpInput;
pub use pagination::Pagination;
pub use password_input::PasswordInput;
pub use pie_chart::{PieChart, PieDatum};
pub use popconfirm::Popconfirm;
pub use popover::Popover;
pub use progress::ProgressBar;
pub use prompts::{PromptItem, Prompts};
pub use qrcode::QRCode;
pub use radar::{RadarChart, RadarSeries};
pub use radio::Radio;
pub use range_picker::DateRangePicker;
pub use rating::Rating;
pub use result::{Result, ResultStatus};
pub use ring_progress::RingProgress;
pub use scatter::{ScatterChart, ScatterPoint, ScatterSeries};
pub use scrollbar::Scrollbar;
pub use search_input::SearchInput;
pub use segmented::Segmented;
pub use select::{MultiSelect, Select};
pub use sender::Sender;
pub use skeleton::Skeleton;
pub use slider::{Slider, SliderColors};
pub use space::Space;
pub use spinner::Spinner;
pub use splitter::Splitter;
pub use statistic::{Statistic, Trend};
pub use status_bar::StatusBar;
pub use stepper::Stepper;
pub use steps::{StepStatus, Steps, StepsStatus, StepsVariant};
pub use switch::Switch;
pub use table::{Table, table_filtered_indices, table_fixed_column_ranges, table_resize_column};
pub use tabs::Tabs;
pub use tag::{Tag, TagColor};
pub use tag_input::TagInput;
pub use terminal::{CursorShape, Terminal, TerminalScrollMetrics};
pub use text_area::{TextArea, TextAreaScrollMetrics};
pub use text_input::TextInput;
pub use theme::Theme;
pub use thought_chain::{ThoughtChain, ThoughtNode, ThoughtStatus};
pub use time_picker::TimePicker;
pub use timeline::{Timeline, TimelineColor, TimelineItem};
pub use title_bar::{TitleBar, WindowIconKind, transparent_titlebar_options};
pub use toast::{
    ToastConfig, ToastDismissed, ToastHost, ToastItem, ToastStack, ToastVariant,
    default_toast_duration,
};
pub use toolbar::{Toolbar, ToolbarItem};
pub use tooltip::Tooltip;
pub use tour::{Tour, TourStep};
pub use transfer::{Transfer, TransferItem};
pub use transfer_manager::{TransferDirection, TransferManager, TransferStatus, TransferTask};
pub use tree::{Tree, TreeDrop, TreeNode, find_node_mut, take_node};
pub use tree_select::TreeSelect;
pub use tree_table::{
    TreeTable, TreeTableNode, tree_table_cascade_check, tree_table_check_state,
    tree_table_navigation_target, tree_table_toggle_expanded, tree_table_visible_ids,
};
pub use typography::{
    CopyConfig, EditableConfig, EllipsisConfig, Paragraph, Text, Title, truncate_text,
};
pub use upload::Upload;
pub use upload_list::{UploadFile, UploadList, UploadStatus};
#[cfg(windows)]
pub use video::VideoPlayer;
pub use watermark::Watermark;
pub use welcome::Welcome;
