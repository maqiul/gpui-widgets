[English](README.en.md) | [中文](README.md)

# gpui-widgets

A general-purpose widget library written in Rust and built on [GPUI](https://gpui.rs), Zed's GPU-accelerated UI framework. It includes **98 documented controls / 101 entries, plus 8 public advanced/host modules**, covering navigation, display, data-entry, and feedback components aligned with the Ant Design component overview, as well as later additions for property alignment, data visualization, AI conversations, and performance monitoring. Other projects can depend on it directly.

It depends on the publicly released `gpui-pre = "=0.3.14"` on crates.io (a third-party repackaging of the Zed GPUI snapshot; the crate's library name is still `gpui`), so **no local Zed source code is required**.

> **Naming clarification**: This crate is an independently published general-purpose widget library on crates.io. It is **not the same thing** as `crates/gpui_widgets` in the Zed source repository (the Zed editor's internal workspace components). The names are similar, but there is no shared code origin or ownership relationship, and the APIs are not interoperable.

## Widget Overview (Grouped by Purpose)

Following the purpose-based grouping in the Ant Design component overview, each widget belongs to only one category; `misc` / `theme` are listed separately as auxiliary modules.

> For complete usage of each widget (positioning / creation signatures / state models / properties / callback event types / examples / caveats), see the **[Widget Reference `docs/controls-reference.en.md`](docs/controls-reference.en.md)**.

| Category | Included Widgets |
|---|---|
| General | `Button` `Link` `Title/Paragraph/Text` `Kbd` |
| Layout | `Row/Col` `Header/Sider/Content/Footer` `Space` `Splitter` `Scrollbar` |
| Navigation | `Anchor` `Breadcrumb` `DropdownMenu` `NavMenu` `Pagination` `Steps` `Tabs` `ContextMenu` `FloatButton` `CommandPalette` |
| Data Entry | `TextInput` `TextArea` `PasswordInput` `SearchInput` `NumberInput` `Stepper` `TagInput` `Autocomplete` `Mentions` `Select` `MultiSelect` `Cascader` `TreeSelect` `Checkbox` `Radio` `Switch` `Slider` `Rating` `DatePicker` `DateRangePicker` `TimePicker` `ColorPicker` `Transfer` `Upload` `UploadList` `FilePicker` `Form` `FormItem` |
| Data Display | `Table` `Tree` `List` `Card` `Descriptions` `Statistic` `Countdown` `Calendar` `Timeline` `Tag` `Badge` `CountBadge` `Avatar` `ImageView` `VideoPlayer` `Carousel` `BarChart` `LineChart` `Sparkline` `PieChart` `GaugeChart` `ScatterChart` `Heatmap` `RadarChart` `FunnelChart` `QRCode` `CodeBlock` `Empty` `Segmented` `Accordion` `Tooltip` `Popover` `Tour` |
| Feedback | `Alert` `Modal` `Drawer` `ToastStack` `NotificationStack` `Popconfirm` `ProgressBar` `RingProgress` `Result` `Skeleton` `Spinner` `Watermark` |
| AI Conversation | `Welcome` `Prompts` `Bubble` `Sender` `Conversations` `ThoughtChain` |
| Utilities and Theming | `misc` `theme` `FpsMeter` |

### General

| Widget | Description |
|---|---|
| `Button` | Button: `primary()` / `secondary()` / `ghost()` / `danger()` variants, `small()` / `large()` sizes, `loading()` state (blocks clicks), `block()` full-width layout, `icon()` leading icon character, `round()` pill shape / `circle()` circular shape, `disabled()` |
| `Link` | Link: underline on hover; `href()` opens the browser on click / `on_click()` custom behavior / `disabled()` |
| `Title` / `Paragraph` / `Text` | Typography trio: heading levels 1~5 / paragraph / inline text (`strong()` / `italic()` / `underline()` / `delete()` / `code()` / `mark()` / `secondary()`) |
| `Kbd` | Keycap: `Kbd::new("Esc")` for a single key / `Kbd::combo("Ctrl+K")` automatically splits key combinations |

### Layout

| Widget | Description |
|---|---|
| `Row` / `Col` | 24-column grid: `Col::new(span)` + `offset()`, `Row::gutter()` column spacing |
| `Header` / `Sider` / `Content` / `Footer` | Four-part page layout: top bar / sidebar (`Sider::new(width)`) / content area / bottom bar |
| `Space` | Spacing container: `vertical()` / `wrap()` / `size()` control spacing between children |
| `Splitter` | Split panel: two side-by-side panes, or top and bottom with `vertical()`; drag the divider to adjust the ratio, controlled `ratio()` + `on_change(&f32)`; requires a unique id |
| `Scrollbar` | Themed scrollbar: binds a `ScrollHandle` that persists across frames (attach `.track_scroll()` to the scrolling container), horizontal / vertical directions via `horizontal()` / `vertical()`, and `vertical_with_metrics()` adapts it for manual scrolling; custom-drawn track and thumb, with drag / track-click / wheel support; automatically hidden when content does not overflow; built into TextArea, Terminal, CodeBlock, Table, and major dropdowns/overlays |

### Navigation

| Widget | Description |
|---|---|
| `Anchor` | Anchor navigation: nested `AnchorItem` entries, left indicator track + current highlight, controlled `active()` + `on_click(&SharedString)`; scroll positioning is handled by the caller |
| `Breadcrumb` | Breadcrumb navigation: the last item represents the current page and is not clickable; `on_click(&usize)` reports intermediate levels |
| `DropdownMenu` | Action menu: any trigger element + a list of `MenuItem` entries (supports separators / `danger()` / `disabled()`); closes after an action is clicked |
| `NavMenu` | Navigation menu: `MenuEntry` supports nested submenus, icons, and disabled states; selected and expanded sets are controlled; `on_select()` / `on_toggle()` |
| `Pagination` | Pagination: page numbers start at 1; automatically collapses into ellipses when there are many pages; boundary buttons are disabled; `simple()` compact mode, `small()` small size |
| `Steps` | Steps: completed ✓ / current number / gray pending state, controlled `current()`, `vertical()` vertical layout, `small()` small size, `progress_dot()` dotted style, `descriptions()` step descriptions |
| `Tabs` | Tabs: underline indicator, `on_select(&usize)`; `disabled_tabs()` disables tabs, `closable()` + `on_close(&usize)` enables closing, `card()` card style, `centered()` centered layout |
| `ContextMenu` | Context-menu wrapper: wraps any content; right-click opens a `MenuItem` menu at the cursor position (clamped within the viewport); requires a unique id |
| `FloatButton` | Floating button: an absolute-positioned circular button anchored to the container's bottom-right corner, with `bottom()` / `right()` / `tooltip()`; the parent container must be `relative()` |
| `CommandPalette` | Command palette: a searchable command selector opened with Ctrl+K (overlay + centered panel); `PaletteItem` can include keyboard shortcut hints; is itself an `Entity` |

### Data Entry

| Widget | Description |
|---|---|
| `TextInput` | Single-line input: caret / selection / clipboard / Chinese input method (IME); is itself an `Entity`; `bare()` removes the border for wrapping, `masked()` masks the display; `disabled()` / `readonly()` / `error()` states, `allow_clear()` ✕ clearing, `max_length()` character limit, `prefix()` / `suffix()` affixes |
| `TextArea` | Multiline input: hard line breaks / cross-line selection / clipboard / IME / automatic caret scrolling, with a built-in vertical scrollbar; `height()` / `scroll_handle()` / `scroll_to_line()` / `set_scroll_offset()` / `scroll_metrics()`; is itself an `Entity` and additionally requires `text_area::bind_keys` |
| `PasswordInput` | Password input: masked display + plaintext toggle button; is itself an `Entity` |
| `SearchInput` | Search input: custom-drawn magnifying-glass icon + ✕ clearing, `on_search()` triggered by Enter; is itself an `Entity` |
| `NumberInput` | Number input: ▲▼ buttons / up and down arrow keys step the value, `min()` / `max()` / `step()`; invalid input does not trigger a callback; is itself an `Entity` |
| `Stepper` | Number stepper: − / + buttons, `min()` / `max()` / `step()` / `disabled()`; boundary states are automatically disabled |
| `TagInput` | Tag input: Enter adds a tag (automatically deduplicated) / Backspace on an empty draft removes the last tag / ✕ removes one tag, `initial_tags()`; is itself an `Entity` |
| `Autocomplete` | Autocomplete: filters candidates as the user types (case-insensitive), ↑↓ to select / Enter to confirm; is itself an `Entity` |
| `Mentions` | @ mention input: typing @ opens a candidate overlay; selecting a candidate replaces it with `"@name "`; is itself an `Entity`, with `on_select(&SharedString)` |
| `Select` | Dropdown selector: the menu defaults to the trigger's width and opens below it (automatically flips above when there is insufficient space below); clicks outside close it; expanded state is controlled (`open()` + `on_open_change()`); `value(Option<usize>)` supports a nullable selection, `allow_clear()` ✕ clearing, `searchable()` filtering within the menu |
| `MultiSelect` | Multi-select dropdown: the trigger displays selected chips (✕ removes one), the menu shows check marks and stays open after selection, `on_change(&Vec<usize>)`; expanded state is likewise controlled |
| `Cascader` | Cascading selector: `CascaderNode` expands multiple menu levels horizontally; path value is controlled; clicking a leaf closes it; `on_change(&Vec<SharedString>)` |
| `TreeSelect` | Tree selector: embeds `Tree` in a dropdown overlay and reuses `TreeNode`; selected id and expanded set are controlled |
| `Checkbox` | Checkbox with an optional label; controlled component; `indeterminate()` mixed state |
| `Radio` | Radio button; combine with parent-view state to form a radio group |
| `Switch` | Switch with an optional label, `small()` small size, `loading()` loading state, `text(on, off)` text inside the track |
| `Slider` | Slider: click-to-position + dragging, `range()` / `step()`, `vertical()` vertical orientation, `marks()` tick dots + labels; requires a unique id |
| `Rating` | Star rating: `max()` / `readonly()`; clicking the current star count clears the rating |
| `DatePicker` | Date selector: calendar overlay (month navigation / Today shortcut / selected highlight); value and displayed month are controlled; `disabled()` disables it, `allow_clear()` + `on_clear()` clears it |
| `DateRangePicker` | Date-range selector: click the start and then the end (selecting an end before the start restarts selection); value `Option<(Date, Date)>` and expanded / month state are controlled; `allow_clear()` + `on_clear()`; dates within the range are highlighted |
| `TimePicker` | Time selector: three scrolling columns for hours / minutes / seconds; value `Option<(u32, u32, u32)>` is controlled |
| `ColorPicker` | Color selector: 24 preset swatches; the trigger shows a color block + HEX; selecting a color immediately triggers a callback and closes the selector |
| `Transfer` | Transfer: check items in two side-by-side panels + →← movement; `target_keys()` / `selected()` are controlled; `titles()` customizes titles |
| `Upload` | Upload trigger area: dashed box + icon + `text()` / `hint()` copy, `disabled()`, 2-argument `on_click` callback (the caller opens the file dialog; drag-and-drop is not built in) |
| `UploadList` | Uploaded-file list: `UploadFile` name / status (Done / Uploading / Error) / progress bar, `on_remove(&usize)` |
| `FilePicker` | File-selection button: native system dialog, `multiple()` / `directories()` / `prompt()`; asynchronous `on_pick` returns paths |
| `Form` | Form layout container: FormItem entries are arranged vertically by default, `inline()` wraps them horizontally, `gap()` sets item spacing; paired `Rule` (`required` / `min_len` / `max_len` / `custom`) + controlled validation helper `validate()` |
| `FormItem` | Form-item wrapper: label + `required()` red star + red-box error message via `error()` / `error_opt()` + `help()` explanatory text, `label_width()` horizontal layout (label on the left) |

### Data Display

| Widget | Description |
|---|---|
| `Table` | Data table: `column_widths()` column widths (equal by default), `striped()` zebra stripes, controlled row selection + `on_row_click()`; header sorting via `sortable()` + `sort()` + `on_sort()`, multi-selection via `selectable()` + `selected_rows()` + `on_selected_rows()`, automatic empty state for no data; built-in pagination via `pagination(page, page_size)` + `on_page_change()` (callbacks still use global indices, and Select All affects only the current page); expandable rows via `row_detail()` + `expanded_rows()` + `on_expand()`, `fixed_left()` pins the first column + horizontal scrolling (mutually exclusive with expandable rows), `bordered()` full borders, `show_header(bool)` header visibility |
| `Tree` | Tree widget: nested `TreeNode` entries (`Clone`), controlled expanded set / selected id, `on_toggle()` / `on_select()`; `checkable()` + `checked()` + `on_check()` parent-child checkbox synchronization, `draggable()` + `on_drop_node(&TreeDrop)` drag-to-move (the caller restructures the tree with `take_node()` / `find_node_mut()` / `push_child()`) |
| `List` | List: `ListItem` title / description / avatar / `extra()` right-side element, `header()` / `footer()` |
| `Card` | Card: title + `extra()` top-right action + `cover()` cover + `action()` bottom action area, `bordered()`, `hoverable()` raised shadow on hover |
| `Descriptions` | Description list: `item(label, value)` key-value pairs, `column()` number of columns, `bordered()` table style, `vertical()` vertical layout (label above value) |
| `Statistic` | Statistic: title + large number + `prefix()` / `suffix()` + `trend()` increase/decrease arrow |
| `Countdown` | Countdown: day/hour/minute/second unit blocks; create with `Countdown::new(unix seconds, cx)` / `after(Duration, cx)`; recalculates from the system clock every second; `on_finish` callback when it reaches zero (1-argument `Fn(&mut App)`); is itself an `Entity` |
| `Calendar` | Calendar: large month-view cells + 12 mini month calendars in year view; month / selected date / mode are controlled, with `on_month_change()` / `on_select()` / `on_mode_change()` |
| `Timeline` | Timeline: `TimelineItem` title / description / time / node color, `alternate()` alternates left and right, `pending()` ghost node |
| `Tag` | Tag: more compact than a badge, `on_close()` shows a ✕ remove button, `color(TagColor)` provides 8 preset colors, `bordered()` toggles the border |
| `Badge` | Badge: `info()` / `success()` / `warning()` / `danger()`, `color()` custom color |
| `CountBadge` | Numeric badge: displays a count at the top-right of its child, `max()` displays 99+ on overflow, `dot()` red-dot mode, `show_zero()`, `color()` custom background, `offset(x, y)` position offset |
| `Avatar` | Avatar: circular background + first character of the name; background color is determined by a hash of the name, `size()`, `square()` rounded-square shape |
| `ImageView` | Image: wraps gpui `img`, `from_file()` local file / `from_render()` in-memory frame, `fit_cover()` / `fit_contain()` / `fit_fill()` / `rounded()` / `grayscale()`, with built-in loading and load-failure placeholders |
| `VideoPlayer` | Video player (**Windows only**): Media Foundation background decoding, built-in play/pause button + draggable progress bar, `play()` / `pause()` / `seek()`; is itself an `Entity` |
| `Carousel` | Carousel: `ImageSource` list, ‹ › cyclic navigation + bottom-dot direct access + counter, controlled `current()`; requires a unique id |
| `BarChart` | Bar chart: `BarDatum` label + value + per-bar color, `height()` / `max()` |
| `LineChart` | Line chart: grid / `area()` filled area / `dots()` point markers / `labels()` X-axis labels, custom-drawn path |
| `Sparkline` | Sparkline: small axis-free line chart, `w()` / `h()` / `area()`, suitable for embedding in tables/cards |
| `PieChart` | Pie / donut chart: `PieDatum` label + value + per-sector color, `donut()` / `inner_ratio()` donut styling, `size()` / `legend()` / `labels()` percentage labels, custom-drawn sectors using the theme's categorical palette |
| `GaugeChart` | Gauge: 270° arc + pointer + center value, `max()` / `size()` / `thickness()` / `title()` / `label()`, `zones()` threshold color bands (green/yellow/red), `color()` single color |
| `ScatterChart` | Scatter chart: multiple `ScatterSeries` + `ScatterPoint` coordinates, grid / axes / legend, `height()` / `x_max()` / `y_max()` / `dot_radius()` / `legend()` |
| `Heatmap` | Heatmap: two-dimensional matrix of colored cells + row/column labels + color-scale legend, `row_labels()` / `col_labels()` / `cell_size()` / `gap()` / `colors(low, high)` scale endpoints / `legend()` |
| `RadarChart` | Radar chart: multiple `RadarSeries` with translucent fills + multi-axis grid rings, `max()` / `size()` / `levels()` grid levels / `legend()` |
| `FunnelChart` | Funnel chart: centered trapezoids stacked by `FunnelStage` + stage labels + conversion rate, `height()` / `gap()` |
| `QRCode` | QR code: pure Rust encoding (byte mode / ECC L / version 1~6), `size()` pixel dimensions, custom rendering with no external dependencies |
| `CodeBlock` | Code block: line-by-line rendering in a monospace font + language label + one-click copy button |
| `Empty` | Empty state: icon + title + `description()` + `action()` action area |
| `Segmented` | Segmented selector: mutually exclusive options inside a rounded container, suitable for 2~5 short options, `small()` / `large()` sizes, `block()` full-width layout |
| `Accordion` | Accordion group: multiple panels can be expanded simultaneously, `AccordionItem` title + arbitrary content, controlled expanded set |
| `Tooltip` | Tooltip wrapper: hovering over any widget shows a bubble (native GPUI delay and positioning) |
| `Popover` | Popover card: any trigger element + arbitrary interactive content; heavier than Tooltip; expanded state is controlled |
| `Tour` | Step-by-step guide: overlay + centered card + step dots, controlled `current()`; skip / previous / next to finish |

### Feedback

| Widget | Description |
|---|---|
| `Alert` | Alert: `info()` / `success()` / `warning()` / `danger()`, `description()` / `on_close()`, `banner()` full-width mode (no border or rounded corners) |
| `Modal` | Modal dialog: overlay + centered card, `width()` card width, `closable()` / `mask_closable()` control ✕ and overlay closing, `footer()` action area, `id()` unique instance id (derived from the title by default); must be rendered outside the scrolling container |
| `Drawer` | Drawer: overlay + side-sliding panel, `placement(DrawerPlacement)` left / right (right by default), `width()` / `footer()`, `closable()` / `mask_closable()`, `id()` unique instance id (derived from the title by default); must be rendered outside the scrolling container |
| `ToastStack` | Toast stack: top-right overlay, four `ToastItem` types, controlled entries, ✕ removal |
| `NotificationStack` | Notification stack: top-right overlay, `NotificationItem` title / description / type / `action()` button, controlled entries |
| `Popconfirm` | Popconfirm: wraps a trigger element, `description()` / `ok_text()` / `cancel_text()`, `on_confirm()` / `on_cancel()` |
| `ProgressBar` | Progress bar (0.0 ~ 1.0), `steps(n)` segmented progress |
| `RingProgress` | Circular progress: custom-drawn arc track + progress arc, `size()` / `thickness()` / `color()` / `label()` center text |
| `Result` | Result page: four `ResultStatus` state icons + title + `description()` + `action()` |
| `Skeleton` | Skeleton screen: breathing-animation placeholders, `circle()` / `lines()` / `w_percent()` |
| `Spinner` | Loading indicator: rotating, fading ring of dots, `size()` / `color()`, `tip()` loading message |
| `Watermark` | Watermark wrapper: staggered text tiles fill the content area, `font_size()` / `color()` / `gap()`; does not intercept child-widget interactions |

### AI Conversation

| Widget | Description |
|---|---|
| `Welcome` | Welcome card: `icon()` icon + title + `description()` description, AI opening guidance; controlled with no callbacks |
| `Prompts` | Prompt-card group: `PromptItem` icon + text, `on_click(&usize)` click callback |
| `Bubble` | Message bubble: `avatar()` avatar + `right()` user side + `loading()` loading state + `variant(Filled/Outlined/Shadow)`, automatic wrapping and width limiting for long text |
| `Sender` | Message composer: is itself an `Entity`, `on_send(&SharedString, &mut Window, &mut App)`; Enter sends / Shift+Enter inserts a line break / button sends |
| `Conversations` | Conversation list: `ConversationItem` key + title + icon, controlled `active()` + `on_select(&SharedString)` |
| `ThoughtChain` | Chain of thought: `ThoughtNode` title / description / `status()`, controlled `expanded()` + `on_toggle(&usize)` expansion and collapse |

### Utilities and Theming

| Widget | Description |
|---|---|
| `misc` | Layout helpers: `card()` / `Divider` / `section_title()` / `caption()` / `field_row()` |
| `theme` | Unified theme: 21 semantic colors (`bg()` / `surface()` / `accent()` / `text()`, etc.), globally configurable: built-in light and dark themes via `Theme::light()` / `Theme::dark()`, `set_theme()` switches the entire theme, `set_accent()` changes the brand primary color (hover/pressed/soft backgrounds are derived automatically), `update()` fine-tunes a single color |
| `FpsMeter` | FPS/performance monitor: is itself an `Entity`; independently drives continuous rendering and repaints only itself; measured FPS + frame time + graph; `max_fps()` nominal cap (120 by default), `running()` / `start()` / `stop()`; actual frame rate follows the display's vsync |

## Advanced and Host Modules

The following 8 modules are public and fully covered in the [controls reference](docs/controls-reference.en.md#x-advanced-and-host-modules). They are not folded into the 98 documented controls / 101 entries, and the total is not presented as 106.

| Module | State Model | Purpose |
|---|---|---|
| `HotkeyInput` | `Entity` | Records a key combination and exposes both a display string and a GPUI binding |
| `Markdown` | `RenderOnce` | Dependency-free rendering of a common GFM subset, links, and code-block actions |
| `TransferManager` | `RenderOnce` | Upload / download queues with overall and per-task progress plus status actions |
| `Terminal` | `Entity` | VT/ANSI terminal grid, scrollback, search, selection, and PTY input bridging |
| `CodeEditor` | Preset factory | Returns a configured `Entity<TextArea>` with line-level syntax highlighting |
| `CommandBar` | `Entity` | Command input, history navigation, snippet completion, and snippet management |
| `DropZone` | `RenderOnce` | Accepts external OS file drops and routes paths by extension |
| `TitleBar` | `RenderOnce` | Custom title bar, window buttons, and native Windows hit-test areas |

## Using It in Other Projects

See the complete integration documentation at [`docs/integration-guide.en.md`](docs/integration-guide.en.md) (dependency configuration / initialization / callback signatures / controlled-state conventions / complete example / common pitfalls FAQ).

### 1. Add the Dependency

```toml
[dependencies]
gpui-widgets = "0.3"   # crates.io release
# gpui-pre's library name is still gpui; it cannot be mixed with the official gpui 0.2.2,
# so downstream projects must switch over entirely
gpui = { package = "gpui-pre", version = "=0.3.14" }
# Host entry point (GPUI 0.3.14 moved the platform layer into its own crate)
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.14" }
```

For local joint development, see the "Local Development Mode" section in [`docs/integration-guide.en.md`](docs/integration-guide.en.md) for path-dependency syntax (path examples are retained only in the development documentation).

### 2. Minimal Example

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
            .child(Button::new("OK").primary().on_click(cx.listener(|this, _, _, cx| {
                println!("Input: {}", this.input.read(cx).text());
            })))
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        // Required: register TextInput editing shortcuts (call once for the entire application)
        gpui_widgets::text_input::bind_keys(cx);

        let bounds = Bounds::centered(None, size(px(480.), px(320.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| MyApp {
                    input: cx.new(|cx| TextInput::new(cx).placeholder("Please enter…")),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
```

See [`examples/minimal.rs`](examples/minimal.rs) for a complete runnable example, and [`examples/gallery.rs`](examples/gallery.rs) for a showcase of all widgets (9-category navigation on the left: General / Layout / Navigation / Data Entry / Data Display / Feedback / AI Conversation / Utilities and Theming / Comprehensive Demos; clicking a category renders only that category on the right):

```bat
cargo run --example minimal
cargo run --example integration   # Example accompanying the integration guide
cargo run --example gallery
cargo run --example gen_assets   # Regenerate the demo video examples/assets/sample.avi (already included in the repository)
```

## Usage Conventions

- **Stateless widgets** (Button / Checkbox / Switch / Radio / Slider / ProgressBar / Badge /
  Tabs / Segmented / Select / Alert / Spinner / Avatar / Tag / Stepper / Accordion /
  Breadcrumb / Pagination / Rating / Table / Tree / DropdownMenu / Popover /
  DatePicker / ColorPicker / ToastStack / NavMenu / Card / Descriptions / List /
  CountBadge / Calendar / QRCode / Cascader / TimePicker / TreeSelect / Transfer /
  FormItem / UploadList / Popconfirm / NotificationStack / Tour / Anchor / Upload):
  The caller supplies values and callbacks, stores state in its own view `Entity`, and calls `cx.notify()` after changing state in a callback.
  Callback signatures consistently use `Fn(&E, &mut Window, &mut App)` and can be constructed directly with `cx.listener(...)`.
- **Controlled expanded state for Select / MultiSelect**: the parent view holds `open: bool`
  and writes it back with `.open(self.open)` + `.on_open_change(...)`; single selection uses `.on_select(&usize)`,
  while multiple selection uses `.on_change(&Vec<usize>)`. The menu width defaults to the trigger width and can be overridden with `.menu_width(px(...))`;
  the menu opens 4px below the trigger and automatically flips above it when it will not fit below the window.
  Single selection additionally supports a null selection via `.value(Option<usize>)`, ✕ clearing via `.allow_clear(true)` (`on_clear` is a 2-argument callback),
  and option filtering through a search box at the top of the menu via `.searchable(true)`.
- **Render location for Modal / Drawer / ToastStack / NotificationStack / Tour**: place them as the last child of the root view, outside the scrolling container;
  **stacked dialogs/drawers**: element ids for ✕ and the overlay are derived from the title by default, so different titles do not interfere with one another;
  when titles are identical, each layer must receive a distinct `.id(...)`, otherwise shared interaction state can make the inner ✕ unresponsive while the outer layer enters the active state
  (ids must remain stable across frames; do not use a counter incremented every frame);
  the `on_close` signature for Modal / Drawer is `Fn(&mut Window, &mut App)` (no event argument), so `cx.listener` does not apply;
  use a `cx.entity()` handle and `entity.update(cx, …)` inside the closure;
  `MenuItem::on_click` for DropdownMenu, `on_click` for FloatButton, `on_confirm` / `on_cancel` for Popconfirm,
  `on_close` for Tour, `on_clear` for Select / DatePicker, and `on_click` for Upload use the same signature.
- **Tooltip is a wrapper**: `Tooltip::new("Tooltip text", any widget)`; hover delay and positioning are handled natively by GPUI.
- **TextInput is a stateful view**: create it with `cx.new(|cx| TextInput::new(cx))`
  and use `.child(input.clone())` in render; read with `input.read(cx).text()`,
  and write with `input.update(cx, |i, cx| i.set_text("…", window, cx))`.
  Enhanced input widgets (TextArea / PasswordInput / SearchInput / NumberInput / TagInput /
  Autocomplete / CommandPalette) are likewise `Entity` values and use the same creation and read/write patterns
  (NumberInput uses `value()` / `set_value()`, and TagInput uses `tags()`).
- **Slider requires a unique id**: `Slider::new("volume", value)`; do not duplicate it within the same window.
- **Scrollbar binds a persistent ScrollHandle**: `Scrollbar::horizontal/vertical(id, handle)`; the handle must be stored in a view field or in `window.use_keyed_state` so it persists across frames, and the scrolling container must attach `.track_scroll(&handle)` (the two share an offset, so `set_offset` can actually drive scrolling, and the wheel updates the thumb in real time); do not duplicate ids within the same window.
- **Overlay widgets share a positioning mechanism**: overlays for Select / MultiSelect / DropdownMenu / Popover / DatePicker / DateRangePicker / ColorPicker /
  Cascader / TimePicker / TreeSelect / Popconfirm
  all open 4px below the trigger, automatically flip above it when they will not fit below the window, and close on an outside click (expanded state is always controlled;
  write it back with `.open(bool)` + `.on_open_change(&bool)`).
- **Controlled displayed month for DatePicker**: write back with `.month(Option<(year, month)>)` + `.on_month_change(...)`;
  when omitted, it defaults to the month of the selected value (or the current month if there is no value); month navigation does not work without `on_month_change`.
  DateRangePicker follows the same convention (the month defaults to the start date's month); the pending start date is persisted by id across frames and automatically discarded when closed or cleared.
- **Table pagination and fixed columns**: pagination is controlled (page numbers start at 1); callbacks for sorting / selection / expansion always use **global row indices**,
  and header Select All affects only the current page; `fixed_left()` requires a uniform row height (37px), columns without an explicit width default to 120px,
  and expandable rows do not work when combined with `row_detail()`.
- **Form validation is a controlled helper**: `validate(&value, &[Rule])` returns the error message for the first failing rule (`Option<SharedString>`),
  which is stored in view state and displayed with `FormItem::error_opt()`; widgets themselves do not hold validation state, and the caller decides when to submit.
- **Tree node ids**: ids only need to be unique within the same tree; both the expanded set and selected value use ids (`Vec<SharedString>` / `Option<SharedString>`);
  TreeSelect reuses the same `TreeNode`, while `MenuEntry` in NavMenu uses a key.
  Checkbox synchronization is built in (checking a node checks all descendants, and ancestors synchronize automatically when all children are selected); dragging only reports `TreeDrop { dragged, target }`
  (it is not triggered when the target is the node itself or one of its descendants), and the caller restructures the node tree with `take_node` / `find_node_mut` / `push_child`.
- **Calendar is fully controlled**: `.value(Option<Date>)` selects a date, `.month((year, month))` sets the displayed month, `.mode(CalendarMode)` selects month / year view,
  and all three callbacks must be handled or the corresponding actions will not work; clicking a mini calendar in year view returns to month view.
- **Controlled dual collections for Transfer**: `target_keys()` is the right-panel collection and `selected()` is the current checked collection;
  `on_change(&Vec<SharedString>)` writes back the right panel and `on_select(&Vec<SharedString>)` writes back the checked collection.
- **QRCode capacity limit**: the pure Rust encoder supports version 1~6 (ECC L), up to **134 bytes** (UTF-8 Chinese text: about 44 characters),
  and oversized data is truncated; it is suitable for short text / URLs and does not support logos or switching error-correction levels.
- **Mentions is a stateful view**: create it with `cx.new(|cx| Mentions::new(cx).options(vec![…]))`,
  and read/write it like TextInput (`text()` / `set_text()`); after typing @ with no whitespace, candidates appear, and clicking a candidate selects it.
- **FloatButton requires a relative parent container**: the button is absolutely positioned, and `bottom()` / `right()` default to 24px.
- **Conventions for Anchor / Upload / Countdown**: Anchor is controlled (`active()` + 3-argument `on_click(&SharedString)`),
  and the caller handles scroll positioning; Upload is triggered only by clicking (`on_click` is a 2-argument `Fn(&mut Window, &mut App)`),
  and the caller implements the file dialog / upload logic; native drag-and-drop is not promised; Countdown is an `Entity`,
  and automatically refreshes every second after creation with `cx.new(|cx| Countdown::after(Duration::from_secs(90), cx))`;
  `on_finish` is a 1-argument `Fn(&mut App)` (the timer context has no Window), fires once when the countdown reaches zero, and then stops the timer;
  the timer loop exits automatically when the entity is dropped; restarting means recreating the entity.
- **Source semantics for ImageView**: `ImageView::new("string")` uses the GPUI asset system (Embedded) and is **not a disk path**;
  always use `ImageView::from_file(path)` for local files. Size/layout is forwarded through `Styled` (`.w() / .h() / .flex()`, etc.),
  and loading and failure placeholders can be customized with `.loading(|| …)` / `.fallback(|| …)`.
- **VideoPlayer is a stateful view, Windows only**: create it with `cx.new(|cx| VideoPlayer::new(cx, path))` and it starts playing automatically;
  use `.child(video.clone())` in render; control it with `video.update(cx, |v, cx| v.pause(cx))` (also `play` / `toggle` / `seek(secs)`),
  and read with `position()` / `duration()` (seconds) / `is_playing()`. Decoding runs on a background thread (MF SourceReader),
  and frames are pumped back to the UI thread through a channel; playback stops automatically at the end, and calling `play()` again restarts from the beginning. This module does not compile on non-Windows platforms.
- **FilePicker's callback is an exception**: the dialog result is returned in an asynchronous task, and `on_pick` has the signature
  `Fn(&Vec<PathBuf>, &mut App)` (**no `&mut Window`**), so `cx.listener` does not apply;
  use a `cx.entity()` handle and `entity.update(cx, …)` inside the closure; cancelling the dialog does not trigger the callback.
- **CommandPalette integration**: register Ctrl+K at startup with `command_palette::bind_keys(cx)`;
  the host view attaches `.on_action(cx.listener(|this, _: &command_palette::Toggle, window, cx| …))`
  and calls `palette.update(cx, |p, cx| p.open(window, cx))` in the action;
  the panel entity is rendered outside the scrolling container (it uses internal deferred top layering), and `on_select(&SharedString)` returns the selected command.
- **Splitter / ContextMenu / Carousel require unique ids**; Splitter's `ratio()` is controlled,
  and the ratio is written back in the `on_change(&f32)` drag callback; ContextMenu / Watermark are wrappers
  used by wrapping content with `.child(any content)`.
- **Submission modes for Sender / TextArea**: TextArea adds `on_submit(Fn(&mut Window, &mut App))`;
  when set, Enter submits (without inserting a line break), while Shift+Enter inserts a line break; Sender uses this mode internally to send on Enter.
  A standalone TextArea without `on_submit` behaves as before (Enter inserts a line break). GPUI actions stop propagating by default during the bubble phase,
  so a container's `on_key_down` cannot receive Enter after TextArea has bound it; therefore, Enter-to-submit must use the action-layer `on_submit` rather than an outer `on_key_down`.
- **FpsMeter performance monitoring**: after creation with `cx.new(|cx| FpsMeter::new(cx))`, it is enabled by default; continuous refresh only notifies this entity
  (`request_animation_frame` repaints only the current view) and does not redraw the entire page; animation is time-based (delta-time) and remains smooth at high refresh rates;
  the actual frame-rate cap is determined by the display's vsync (a 120Hz display can reach ~120fps), and `max_fps()` is only a nominal cap.
- **Horizontal flex overflow**: flex children default to `min-width:auto` and will not shrink below min-content, so long text / wide widgets can break the parent column;
  add `.min_w(px(0.))` to shrinkable columns (add it to both the outer scrolling container and the inner flex_1 column); the same applies to width-constrained bubbles / cards.
- Call `gpui_widgets::text_input::bind_keys(cx)` once at application startup,
  otherwise Ctrl+A/C/V/X, arrow keys, and other input shortcuts will not work (`cmd` automatically maps to Ctrl on Windows/Linux);
  TextArea additionally requires `text_area::bind_keys(cx)` (Enter / up and down arrows / Shift selection),
  and CommandPalette additionally requires `command_palette::bind_keys(cx)`.

## Platforms and Versions

- Verified on Windows (DirectWrite + DirectX backend). macOS / Linux are supported by GPUI itself but have not been verified in this environment;
  the repository includes a **Linux (x11) compilation gate + `cargo doc` gate** in `.github/workflows/ci.yml` (effective after pushing to a git remote),
  while the local acceptance gate is `bash harness/check.sh` (state matrix + screenshot baseline + full test suite).
- `VideoPlayer` depends on Windows Media Foundation and compiles only on Windows (`#[cfg(windows)]`);
  it is automatically excluded on other platforms without affecting the rest of the library. The demo video is an MJPEG AVI (generated by `gen_assets`; no ffmpeg required).
- **Pin `gpui-pre 0.3.14`** (the public crates.io release, a repackaging of the Zed GPUI snapshot; it cannot be mixed with the official `gpui 0.2.2`).
  GPUI is pre-1.0 and has frequent breaking changes:
  **upgrading gpui must be accompanied by compatibility verification for this library** (full `harness/check.sh` + group-by-group comparison against the screenshot baseline),
  not just a version-number change. Each release corresponds to a same-named git tag (`v0.1.x`) for easier traceability.
- **docs.rs compatibility**: the upstream transitive dependency `xattr 0.2.3` (gpui→gpui_http_client→zed-async-tar) references
  `libc::ENOATTR`, and libc ≥0.2.191 removed this Linux alias ⇒ documentation build error E0425. docs.rs ignores the package's
  `Cargo.lock` and performs a fresh resolution using the latest versions, so this library pins
  `libc = "=0.2.189"` exactly under `[target.'cfg(unix)'.dependencies]` (cargo unifies semver across the full graph to a version that still contains the alias), applies only to unix,
  and has zero impact on Windows builds. The published package still includes `Cargo.lock` so `cargo install --locked` users can reproduce the dependency combination.
- Rust edition 2024; Rust 1.89+ recommended.

## License

Apache-2.0

