[English](integration-guide.en.md) | [中文](integration-guide.md)

# gpui-widgets Integration Guide for External Projects

> Companion documentation: [README.en.md](../README.en.md) provides a complete overview of all 98 widgets; this guide focuses on how to add and correctly use these widgets in your own Rust project.
> The complete examples in this guide match [`examples/integration.rs`](../examples/integration.rs), have been verified to compile, and can be copied directly.

## 1. Add Dependencies

The `widgets/` directory is an **independent crate** (`gpui-widgets`). It depends on the publicly released `gpui-pre = "=0.3.14"` from crates.io (a third-party repackaging of the Zed GPUI snapshot; the library name is still `gpui`) and **does not require the zed source code**.

### Option A: crates.io Release (Recommended for Integration)

```toml
[dependencies]
# gpui-pre's library name is still gpui; it cannot be mixed with the official gpui 0.2.2,
# so downstream projects must switch over entirely
gpui = { package = "gpui-pre", version = "=0.3.14" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.14" }
gpui-widgets = "0.2"
```

### Local Development Mode (Development Only)

Use the following configuration only when developing this library alongside another project and needing changes to take effect immediately:

```toml
# Adjust this relative path to point to the library directory
gpui-widgets = { path = "../widgets" }
# Or a git repository (team collaboration / multiple machines): after pushing widgets/ to a git repository
# gpui-widgets = { git = "https://your-repo/gpui-widgets" }
```

Alternatively, copy the entire `widgets/` directory into your project (for example, `vendor/gpui-widgets`), then use `path = "vendor/gpui-widgets"`. The library has no external asset dependencies (demo videos are used only by examples), so the copied directory is ready to compile.

### Version and Compilation Recommendations

- Rust **1.89+**, edition 2024 (both this library and its examples use 2024).
- `gpui-pre` is pinned to **0.3.14** (pre-1.0; upgrades require compatibility to be verified again); it cannot be mixed with the official `gpui 0.2.2`.
- Verified platform: Windows (DirectWrite + DirectX). macOS / Linux are supported by GPUI itself but have not been verified in this environment.
- `VideoPlayer` compiles only on Windows (`#[cfg(windows)]`). It is automatically excluded on other platforms without affecting the remaining widgets.
- Optimization is recommended for the dev profile; otherwise, GPUI debug builds may run slowly (the same configuration used by consumer-demo):

```toml
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 2
```

## 2. Startup Initialization

```rust
fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        // Required: bind text input editing shortcuts (Ctrl+A/C/V/X, arrow keys, etc.) once for the entire application
        gpui_widgets::text_input::bind_keys(cx);
        // As needed:
        // gpui_widgets::text_area::bind_keys(cx);        // When using TextArea
        // gpui_widgets::command_palette::bind_keys(cx);  // When using CommandPalette (Ctrl+K)

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

A typical symptom of forgetting `bind_keys`: text can be entered, but Ctrl+C/V, Select All, and arrow-key cursor movement all stop working.

## 3. Usage Patterns for the Two Types of Widgets

### 3.1 Stateless Widgets (the Vast Majority)

Widgets such as Button / Checkbox / Switch / Select / Table / Calendar / Transfer **do not store state internally**:
You pass in values during render, update fields on your own view `Entity` in callbacks, and call `cx.notify()` to trigger a redraw.

```rust
struct MyApp { city: usize, city_open: bool, /* … */ }

// In render:
Select::new("city", ["Beijing", "Shanghai", "Guangzhou", "Shenzhen"], self.city)
    .open(self.city_open)
    .on_open_change(cx.listener(|this, open, _, cx| {
        this.city_open = *open;
        cx.notify();
    }))
    .on_select(cx.listener(|this, ix, _, cx| {
        this.city = *ix;
        this.city_open = false;   // Collapse after a single selection is made
        cx.notify();
    }))
```

### 3.2 Stateful Widgets (Entities)

Input / media / timer widgets hold internal state such as the cursor / selection / decoder / remaining seconds. They are `Entity`s and **must be created once with `cx.new`**; mount them with `.clone()` during render:

| Widget | Creation | Read / Write |
|---|---|---|
| `TextInput` | `cx.new(\|cx\| TextInput::new(cx).placeholder("…"))` | `read(cx).text()` / `update(cx, \|i, cx\| i.set_text("…", window, cx))` |
| `TextArea` | `cx.new(\|cx\| TextArea::new(cx).height(px(120.)))` | `text()` / `set_text()` / `set_caret()` / `scroll_handle()` / `scroll_to_line()` / `set_scroll_offset()` / `scroll_metrics()` |
| `PasswordInput` / `SearchInput` / `Autocomplete` / `Mentions` | Same as above, each with its own `::new(cx)` | Same as TextInput |
| `NumberInput` | `NumberInput::new(cx)` | `value()` / `set_value()` |
| `TagInput` | `TagInput::new(cx).initial_tags(…)` | `tags()` |
| `CommandPalette` | `CommandPalette::new(cx, items)` | `open(window, cx)` / `close(cx)` |
| `VideoPlayer` (Windows only) | `VideoPlayer::new(cx, path)` | `play/pause/seek`, `position()/duration()` |
| `Countdown` | `Countdown::after(Duration, cx)` or `::new(unix seconds, cx)` | `remaining()/finished()`; `on_finish` zero-crossing callback (1 parameter) |
| `Sender` | `Sender::new(cx).on_send(…)` | Internal TextArea; Enter sends / Shift+Enter inserts a newline |
| `FpsMeter` | `FpsMeter::new(cx).max_fps(120)` | `fps()/frame_ms()`; `start(cx)/stop(cx)`; redraws itself only |

```rust
// Create (in main or during parent view initialization):
name: cx.new(|cx| TextInput::new(cx).placeholder("Please enter your name")),
// In render:
div().child(self.name.clone())
// Read in an event handler:
let text = self.name.read(cx).text();
```

Note: The `cx` passed to `TextInput::new(cx)` is a `Context<TextInput>` and can only be called inside a `cx.new(|cx| …)` closure. It **cannot** be called directly with `new` in the parent view's render / partition methods.

## 4. Callback Signature Quick Reference

| Signature | Construction Method | Widgets Involved |
|---|---|---|
| `Fn(&E, &mut Window, &mut App)` | `cx.listener(\|this, &e, window, cx\| …)` | The vast majority: Button.on_click, Switch/Checkbox.on_change, Select.on_select, Table.on_row_click, etc. |
| `Fn(&mut Window, &mut App)` (**no event parameter**) | `let entity = cx.entity(); move \|_, cx\| { let _ = entity.update(cx, \|this, cx\| …); }` | Modal.on_close, Drawer.on_close, MenuItem::on_click, FloatButton.on_click, Popconfirm.on_confirm/on_cancel, Tour.on_close, Select.on_clear, DatePicker.on_clear, DateRangePicker.on_clear, Upload.on_click |
| `Fn(&mut App)` (**no Window and no event parameter**) | Same entity-handle pattern | Countdown.on_finish (timer context) |
| `Fn(&Vec<PathBuf>, &mut App)` (**no Window**) | Same entity-handle pattern | FilePicker.on_pick (asynchronous dialog) |

Two frequent compilation errors:

- **E0593**: using `cx.listener` for a 2-parameter callback → switch to the entity-handle pattern.
- **E0282**: event parameter inference fails for a `cx.listener` closure in a chained call → add an explicit type annotation, for example
  `cx.listener(|this, key: &gpui::SharedString, _, cx| …)`.

## 5. Controlled State and Overlay Conventions

- **Controlled expansion**: Select / MultiSelect / DropdownMenu / Popover / DatePicker / DateRangePicker / ColorPicker /
  Cascader / TimePicker / TreeSelect / Popconfirm all use `.open(bool)` + `.on_open_change(&bool)` to write the value back;
  without `on_open_change`, clicking the trigger will not expand the widget.
- **Controlled values**: all three Calendar callbacks, `.value()/.month()/.mode()`, must be connected; Transfer's
  `target_keys()` (right pane) and `selected()` (checked items) are separate collections written back through `on_change` / `on_select`, respectively;
  the displayed month of DatePicker / DateRangePicker uses `.month()` + `.on_month_change()`.
- **Common property extensions** (aligned with antd): Button `large()/loading()/block()/icon()`; TextInput
  `disabled()/readonly()/error()/allow_clear()/max_length()/prefix()/suffix()`; Select
  `value(Option<usize>)/allow_clear()/searchable()`; Checkbox `indeterminate()`; Switch `small()/loading()`;
  Table `column_widths()/sortable()+sort()+on_sort()/selectable()+selected_rows()+on_selected_rows()` (empty data automatically shows an empty state);
  Tabs `disabled_tabs()/closable()+on_close()/card()`; Tag `color(TagColor)/bordered()`; Badge/CountBadge `color()`
  (CountBadge also has `offset(x, y)`); Modal `width()/closable()/mask_closable()`; Drawer
  `placement(DrawerPlacement::Left)/closable()/mask_closable()`; DatePicker `disabled()/allow_clear()+on_clear()`;
  Slider `vertical()/marks([(value, label)])`; Spinner `color()`. Sorting, multiple selection, and similar features all use controlled mode; write state back in callbacks and call `cx.notify()`.
- **Advanced feature properties**: Table `pagination(page, page_size)+on_page_change()`, `row_detail()+expanded_rows()+on_expand()`,
  `fixed_left()`; Tree `checkable()+checked()+on_check()`, `draggable()+on_drop_node(&TreeDrop)`;
  FormItem `label_width()` (horizontal labels) / `error_opt()`; newer widgets `Form` (`inline()/gap()`),
  `Rule` + `validate()`, `DateRangePicker`. See "Advanced Feature Controlled Conventions" below for details.
- **Additional property-alignment extensions**: Button `round()/circle()`; Steps `vertical()/small()/progress_dot()/descriptions()`;
  Pagination `simple()/small()`; Tabs `centered()`; Segmented `small()/large()/block()`;
  Switch `text(on, off)`; Timeline `alternate()/pending()`; Descriptions `vertical()`;
  Table `bordered()/show_header(bool)`; ProgressBar `steps(n)`; Alert `banner()`; Spinner `tip()`;
  Avatar `square()`; Card `hoverable()`. These are display-only properties and do not change the controlled-state conventions.
- **Overlay rendering location**: place Modal / Drawer / ToastStack / NotificationStack / Tour as the **last child of the root view, outside scroll containers** (they are internally deferred to the top); otherwise, they may be clipped by scrolling or fail to cover the content.
- **Unique IDs**: widgets whose constructors accept an ID (Slider / Splitter / ContextMenu / Carousel / NavMenu /
  Cascader / TimePicker / TreeSelect / Transfer / Popconfirm, etc.) must not reuse the same ID within one window.
- **Globally configurable theme**: widgets consistently obtain colors from `gpui_widgets::theme` (21 semantic colors), with a light theme by default;
  `set_theme()` / `set_accent()` / `update()` can change the theme globally. See "Theme Configuration" below for details.
  Layout helpers are available in `misc::{card, section_title, caption, field_row, Divider}`.

### Theme Configuration (Light/Dark Switching / Brand Accent / Custom Palettes)

All widgets obtain colors through semantic color functions in the `theme` module, so changing the theme requires no changes to widget code:

```rust
use gpui::{rgb, prelude::*};
use gpui_widgets::theme::{self, Theme};

// 1) Switch between built-in themes globally
theme::set_theme(Theme::dark());          // Or Theme::light()
cx.refresh_windows();                     // Redraw all windows; a single view can also call cx.notify()

// 2) Change only the brand accent: hover / pressed shades are automatically derived
//    with decreasing brightness, and the soft background (accent_soft) uses a light
//    or dark tint depending on the current theme
theme::set_accent(rgb(0x8b5cf6));

// 3) Fine-tune a single semantic color
theme::update(|t| t.border = rgb(0xd0d3d9).into());

// 4) Customize the entire palette: all Theme fields are public, and a built-in theme can also be used as a base
let mut my = Theme::light();
my.bg = rgb(0xfaf7f2).into();
my.surface = rgb(0xffffff).into();
theme::set_theme(my.with_accent(rgb(0xc2410c)));

// Read the current palette
let now: Theme = theme::current();
```

Note: The theme is stored in global static state and shared by the entire application; `set_theme` and related functions only change the palette and do not trigger a redraw.
After switching themes, call `cx.refresh_windows()` yourself (or `cx.notify()` in the current view's callback).

### Advanced Feature Controlled Conventions (Table / Form / Tree / DateRangePicker)

**Table pagination / expandable rows / fixed left columns**:

```rust
Table::new("members", ["Name", "City"], rows)
    .pagination(self.page, 4)                 // Page numbers start at 1; controlled
    .on_page_change(cx.listener(|this, p, _, cx| { this.page = *p; cx.notify(); }))
    .row_detail(|ix| detail_view(ix).into_any_element()) // Generate details by global index
    .expanded_rows(self.expanded.clone())
    .on_expand(cx.listener(|this, ix, _, cx| { /* Toggle the expanded state of ix */ }))
```

- After pagination, sorting / selection / expansion / row-click callbacks still use **global row indices**; Select All in the header applies only to the current page.
- `fixed_left()` fixes the first data column (including the checkbox column), while the remaining columns scroll horizontally; it requires a uniform row height of 37px,
  and columns without an explicit width default to 120px; it is **mutually exclusive with `row_detail()`** (when both are set, expandable rows have no effect);
  a theme-styled horizontal scrollbar is built in below the right pane (thumb dragging / track-click jumping / mouse wheel; see the Scrollbar section).

**Form validation (controlled helper; widgets do not hold validation state)**:

```rust
use gpui_widgets::{Form, FormItem, Rule, validate};

// Compute errors on submission and store them in view fields
self.name_error = validate(&name, &[
    Rule::required("Please enter a username"),
    Rule::min_len(2, "At least 2 characters"),
    Rule::custom("Must not contain spaces", |v| !v.contains(' ')),
]);
// Display in render
Form::new().child(
    FormItem::new("Username").required().label_width(px(80.))
        .error_opt(self.name_error.clone())
        .child(self.name_input.clone()),
)
```

`validate` returns the message from the first rule that fails (`Option<SharedString>`), or `None` if all rules pass;
`Form::new().inline()` arranges items horizontally with wrapping and is suitable for filter bars.

**Tree checkbox cascading / drag-and-drop restructuring**:

```rust
use gpui_widgets::{Tree, TreeDrop, find_node_mut, take_node};

Tree::new("files", self.nodes.clone())        // TreeNode implements Clone; you own the node data
    .checkable(true)
    .checked(self.checked.clone())
    .on_check(cx.listener(|this, ids, _, cx| { this.checked = ids.clone(); cx.notify(); }))
    .draggable(true)
    .on_drop_node(cx.listener(|this, drop: &TreeDrop, _, cx| {
        // Checkbox cascading is built in; dragging only reports the event, so implement restructuring yourself:
        if let Some(node) = take_node(&mut this.nodes, &drop.dragged) {
            if let Some(target) = find_node_mut(&mut this.nodes, &drop.target) {
                target.push_child(node);      // Move under the target node
            }
        }
        cx.notify();
    }))
```

Checking a node includes all its descendants, and ancestors are synchronized automatically according to "checked if all children are checked"; dropping onto the node itself / its own descendants does not trigger `on_drop_node`.

**DateRangePicker**: the value is `Option<(Date, Date)>`. Click the start date and then the end date (if the end date is earlier than the start date, the start date is selected again).
It collapses automatically after submission; the pending start date is persisted across frames by widget ID and automatically discarded on collapse / clear;
when `.allow_clear(true)` is set and a value exists, an ✕ is displayed, and `on_clear` is a 2-parameter callback (if unset, it falls back to `on_change(None)`).

### New Widgets: Anchor / Upload / Countdown

**Anchor anchor navigation** (controlled; GPUI has no browser-style anchor scrolling, so the caller handles positioning):

```rust
Anchor::new(vec![
    AnchorItem::new("basic", "Basic Usage"),
    AnchorItem::new("api", "API").child(AnchorItem::new("props", "Properties")),
])
.active(self.anchor_key.clone())
.on_click(cx.listener(|this, key: &gpui::SharedString, _, cx| {
    this.anchor_key = key.clone();
    cx.notify();
}))
```

**Upload trigger area** (click-triggered UI only; the caller implements the file dialog / upload logic, and native drag-and-drop is not guaranteed):

```rust
let entity = cx.entity();
Upload::new()
    .text("Click to Upload")
    .hint("Supports png / jpg / pdf")
    .on_click(move |_, cx| {
        let _ = entity.update(cx, |this, cx| {
            // Open a file dialog here (using a library such as rfd) and perform the upload
        });
    })
```

Multiple Upload widgets in the same window must each have a unique `.id("…")` (the default is "upload-area"); otherwise, hover highlighting may interfere between them
(GPUI determines hover by ElementId).

**Countdown countdown widget** (Entity; recalculates from the system clock every second to prevent drift, and stops timing after triggering `on_finish` once when it reaches zero):

```rust
// Create (during parent view initialization / on reset):
countdown: {
    let entity = cx.entity();
    cx.new(|cx| {
        Countdown::after(std::time::Duration::from_secs(90), cx).on_finish(move |cx| {
            let _ = entity.update(cx, |this, cx| { /* Zero-crossing action, such as showing a Toast */ });
        })
    })
},
// In render:
div().child(self.countdown.clone())
```

- The `on_finish` signature is `Fn(&mut App)` (the timer context has no Window), so `cx.listener` does not apply; use an entity handle to write back state.
- To restart, recreate the entity; after the old entity is released, the timing loop exits automatically with no leak.
- Read progress with `countdown.read(cx).remaining()` / `finished()`.

### Scrollbar

GPUI does not draw scrollbars by default. `Scrollbar` supplies visible, interactive scroll feedback based on values measured from rc-virtual-list: track width 8, pill-shaped rounded thumb, minimum thumb length 20; thumb `rgba(0,0,0,0.5)` (a mismatch between antd's `--rc-` / `--vc-` variable names causes the `colorSplit` override to never take effect, so this fallback value is what actually renders); the track is normally transparent and uses `colorFillQuaternary` only on hover. Thumb dragging (which can continue outside the track after pressing), track-click jumping, and mouse-wheel scrolling are supported; it hides automatically when the content does not overflow.

**Key convention**: the bound `ScrollHandle` must persist across frames (store it in a view field or `window.use_keyed_state`), and the scroll container must have `.track_scroll(&handle)`—both share the same offset, so `set_offset` can actually drive scrolling, and mouse-wheel scrolling is immediately reflected in the thumb position.

```rust
// View field: scroll_handle: ScrollHandle (initialize with the ScrollHandle::new() constructor)
// In render:
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
    // Overlay the vertical scrollbar at the right edge of the scroll area
    .child(
        div()
            .absolute()
            .top_0()
            .bottom_0()
            .right_0()
            .child(Scrollbar::vertical("area-vsb", self.scroll_handle.clone())),
    )
```

- Use `Scrollbar::horizontal(id, handle)` for horizontal scrolling and `Scrollbar::vertical(id, handle)` for vertical scrolling; the track width is determined by a theme metric.
- For manually managed scroll offsets, `Scrollbar::vertical_with_metrics(id, handle, visible, max)` can be used for adaptation; the offset is still driven by the same `ScrollHandle`; `Terminal` already uses this entry point.
- `TextArea` has a built-in vertical scrollbar and provides `scroll_handle()` / `scroll_to_line()` / `set_scroll_offset()` / `scroll_metrics()`; mouse-wheel scrolling no longer gets pulled back to the caret line by paint, and automatic reveal occurs only when the caret actually moves.
- `Table::fixed_left()`, `CodeBlock`, `CommandPalette`, `CommandBar`, `Drawer`, `Select`, `MultiSelect`, `Transfer`, `Cascader`, `Autocomplete`, `Mentions`, `Calendar` year lists, `TimePicker`, and `Terminal` already have scrollbars built into their respective scroll areas.
- Do not reuse IDs within the same window (GPUI determines hover by ElementId).
- **Change thumb color / opacity**: globally, use `theme::set_scrollbar_thumb(Some(hsla))` (a single override for all custom-drawn scrollbars in the library, including those built into `Terminal` / `TextArea` / `Table`, etc.); pass `None` to restore antd's fallback black at 50%; for a single scrollbar, use `Scrollbar::vertical(..).thumb_color(hsla)`. Priority: element-level > global > theme default.
  ```rust
  // antd's black thumb is nearly invisible in dark themes, so hosts typically fix it this way:
  theme::set_scrollbar_thumb(Some(rgba(0xffffff4d).into())); // White 30%
  // Or, to make the thumb lighter only in light themes:
  theme::set_scrollbar_thumb(Some(rgba(0x00000033).into())); // Black 20%
  ```
  This override does not participate in token derivation: `set_theme` / `set_appearance` do not clear it, so after switching between light and dark themes, reset it for the new appearance. Color resolution is the pure function `resolve_layered_color(element, global, default)`; see the matrix in `tests/scrollbar_thumb.rs`.
- **Change track background color**: the same two layers apply—global `theme::set_scrollbar_track(Some(hsla))` and per-scrollbar `Scrollbar::vertical(..).track_color(hsla)`. When not overridden, the `colorFillQuaternary` token is used. The track itself is drawn only on hover; an override does not change the "no track while idle" behavior, only the hover color.

### Slider Color Configuration

All `Slider` colors are derived from theme tokens by default (changing the accent with `theme::set_accent` updates the entire slider). Use `SliderColors` to customize colors per instance:

```rust
use gpui::rgba;
use gpui_widgets::{Slider, SliderColors};

Slider::new("vol", 70.).range(0., 100.).colors(
    SliderColors::new()
        .rail(rgba(0x1f2329ff).into())   // Base groove
        .track(rgba(0x8b5cf6ff).into())  // Selected segment
        .handle(rgba(0x8b5cf6ff).into()) // Thumb core
        .handle_ring(rgba(0xffffffff).into()), // Thumb ring
)
```

- Only these four items are exposed; the pressed outer ring `handle_outline` and tick marks always follow the tokens, avoiding manual color assignments that would break antd's alpha semantics.
- Unset fields retain their token-derived values and are orthogonal to hover / pressed / disabled states: applying the same overrides to different states changes only those four items. Merging is the pure function `SliderColors::apply(base)`; see the matrix in `tests/slider_matrix.rs::slider_colors_override_matrix`.

### Advanced Charts (Data Visualization)

In addition to `BarChart` / `LineChart` / `Sparkline` / `RingProgress`, the library provides 6 advanced charts, all custom-drawn using GPUI's native `PathBuilder` + `window.paint_path`, with no third-party dependencies; they use a theme-derived **8-color categorical palette** by default (`theme::chart_color(i)`, which adapts to light and dark themes), and individual items can be overridden with `.color()`. All are **controlled components** (no callbacks; data is supplied by the parent view).

| Widget | Creation | Key Properties |
|---|---|---|
| `PieChart` pie/donut chart | `PieChart::new(Vec<PieDatum>)`, `PieDatum::new(label, value)` | `donut(true)` / `inner_ratio(0.6)` donut chart, `size(px)`, `legend(bool)`, `labels(bool)` percentages |
| `GaugeChart` gauge | `GaugeChart::new(value)` | `max(f32)`, `size(px)`, `thickness(px)`, `title(text)`, `label(text)`, `zones(vec![(ratio limit, color)])` threshold zones, `color()` |
| `ScatterChart` scatter plot | `ScatterChart::new(Vec<ScatterSeries>)`, `ScatterSeries::new(name, Vec<ScatterPoint>)`, `ScatterPoint::new(x, y)` | `height(px)`, `x_max(f32)` / `y_max(f32)`, `dot_radius(px)`, `legend(bool)` |
| `Heatmap` heat map | `Heatmap::new(Vec<Vec<f32>>)` (row-major matrix) | `row_labels(vec)` / `col_labels(vec)`, `cell_size(px)`, `gap(px)`, `colors(low, high)` ends of the color scale, `legend(bool)` |
| `RadarChart` radar chart | `RadarChart::new(axes: Vec<SharedString>, Vec<RadarSeries>)`, `RadarSeries::new(name, Vec<f32>)` | `max(f32)`, `size(px)`, `levels(usize)` number of grid levels, `legend(bool)` |
| `FunnelChart` funnel chart | `FunnelChart::new(Vec<FunnelStage>)`, `FunnelStage::new(label, value)` | `height(px)`, `gap(px)`; automatically labels the conversion rate relative to the previous stage |

```rust
use gpui_widgets::{PieChart, PieDatum, GaugeChart, theme};
use gpui::px;

// Donut chart (automatic legend on the right + sector percentages)
PieChart::new(vec![
    PieDatum::new("Direct Access", 335.),
    PieDatum::new("Search Engines", 548.),
    PieDatum::new("Email Marketing", 234.),
])
.donut(true)
.size(px(160.))

// Gauge: threshold zones change from green/yellow/red as the value changes
GaugeChart::new(72.)
    .max(100.)
    .title("CPU Usage")
    .zones(vec![
        (0.6, theme::success()),
        (0.85, theme::warning()),
        (1.0, theme::danger()),
    ])
```

- Charts are purely custom-drawn controlled elements with no internal interaction state and require no caller-maintained state; simply pass in the current data, and changes are redrawn with the parent view.
- The categorical palette is defined in `theme.rs` (`chart_palette()` / `chart_color(index)`), following the convention that "new widgets do not hard-code rgb"; pass `.color()` / `.colors()` explicitly when fixed brand colors are needed.
- The "Advanced Charts" section of the gallery contains complete demos of all 6 charts, including real-time coupling between the gauge and volume slider.

## 6. Complete Integration Example (Compilation Verified)

Create a new project, configure `Cargo.toml` as described in Section 1, and paste the following code into `src/main.rs` to run it
(equivalent to `cargo run --example integration`):

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
                Select::new("city", ["Beijing", "Shanghai", "Guangzhou", "Shenzhen"], self.city)
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
                Checkbox::new("I have read and agree to the terms", self.agreed).on_change(cx.listener(
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
                        Button::new("Open Dialog")
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.modal_open = true;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("Submit").primary().on_click(cx.listener(
                            |this, _, _, cx| {
                                let name = this.name.read(cx).text().to_string();
                                this.push_toast(
                                    ToastItem::new(format!("Submitted: {name}")).success(),
                                    cx,
                                );
                            },
                        )),
                    ),
            );

        // Place overlays at the end of the root view, outside scroll containers
        if self.modal_open {
            let entity = cx.entity();
            root = root.child(
                Modal::new("Confirmation")
                    .on_close(move |_, cx| {
                        let _ = entity.update(cx, |this, cx| {
                            this.modal_open = false;
                            cx.notify();
                        });
                    })
                    .child("Are you sure you want to submit the current form?")
                    .footer(
                        div()
                            .flex()
                            .gap_2()
                            .justify_end()
                            .child(Button::new("Cancel").secondary().on_click({
                                let entity = cx.entity();
                                move |_, _, cx| {
                                    let _ = entity.update(cx, |this, cx| {
                                        this.modal_open = false;
                                        cx.notify();
                                    });
                                }
                            }))
                            .child(
                                Button::new("Confirm")
                                    .primary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.modal_open = false;
                                        this.push_toast(
                                            ToastItem::new("Submitted successfully").success(),
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
        // Required: register text input editing shortcuts (call once for the entire application)
        gpui_widgets::text_input::bind_keys(cx);

        let bounds = Bounds::centered(None, size(px(520.), px(420.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| MyApp {
                    name: cx.new(|cx| TextInput::new(cx).placeholder("Please enter your name")),
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

## 7. Common Pitfalls FAQ

| Symptom | Cause and Solution |
|---|---|
| None of the text input shortcuts work | `text_input::bind_keys(cx)` was not called at startup |
| State changes in callbacks do not refresh the UI | `cx.notify()` was forgotten |
| E0593 closure parameter mismatch | `cx.listener` was incorrectly used for a 2-parameter callback; switch to an entity handle (Section 4) |
| E0282 type inference failure | Explicitly annotate the `cx.listener` event parameter type |
| Popups / notifications are clipped or fail to cover content | The overlay was placed inside a scroll container; move it to the end of the root view |
| A dropdown does not expand when clicked | `on_open_change` was not connected to write back `open` |
| A custom TextInput wrapper panics (double lease) | Use `set_text_silent` for programmatic writes in wrapper widgets and manually invoke the outer callback for user-action paths; built-in wrapper widgets already follow this pattern |
| QR code content is incomplete | QRCode has a limit of 134 bytes (approximately 44 Chinese characters); longer content is truncated |
| A list does not scroll | `overflow_y_scroll()` works only on a `Stateful<Div>` with an `.id()` |
| VideoPlayer compilation errors on non-Windows platforms | Gate references with `#[cfg(windows)]`; the library itself already excludes it automatically |
| Horizontal overflow / clipping on the right | Flex items default to min-width:auto and do not shrink below min-content; add `.min_w(px(0.))` to shrinkable columns (both the outer scroll container and inner flex_1 columns) |
| Enter in a wrapped input widget does not trigger the outer handler | GPUI actions stop propagation by default during bubbling, so a container's on_key_down cannot receive an already-bound Enter; use TextArea `on_submit` instead (action-layer submission mode) |
| The entire page stutters at high frame rates | Put high-frequency refreshes in a separate Entity (such as FpsMeter); `request_animation_frame` then notifies only the current view, avoiding a full-page redraw |

## 8. Run the Reference Examples

```bat
cd d:\gpui\widgets
cargo run --example integration   # Complete example from this guide
cargo run --example minimal       # Minimal-dependency example
cargo run --example gallery       # Showcase of all 98 widgets (9-category navigation on the left)
```

For an external standalone project, see `d:\gpui\consumer-demo` (depends only on a `gpui-widgets` path reference and has passed regression verification).
