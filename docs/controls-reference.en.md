[English](controls-reference.en.md) | [中文](controls-reference.md)

# gpui-widgets Controls Reference

> This reference documents the library's **103 documented controls / 106 entries** and also covers **8 public advanced/host modules**. Every entry includes its **purpose, creation signature, state model, properties, callbacks (with exact types), minimal example, and caveats**.
>
> Companion documentation:
> - [`README.en.md`](../README.en.md): a control category overview and usage conventions (quick reference).
> - [`integration-guide.en.md`](integration-guide.en.md): how to add the library to your own Rust project, initialize it, and run the complete examples.
> - **This reference**: complete usage documentation for individual controls (consult as needed).
>
> All signatures have been verified against the `widgets/src/*.rs` source code, and every callback event type has been checked, so you can copy them directly.

---

## Table of Contents

- [I. Core Concepts (Read Before Using Any Control)](#i-core-concepts-read-before-using-any-control)
- [II. General](#ii-general): Button · Link · Title/Paragraph/Text · Kbd · Toolbar
- [III. Layout](#iii-layout): Row/Col · Header/Sider/Content/Footer · Space · Splitter · Scrollbar · Flex · Affix · StatusBar
- [IV. Navigation](#iv-navigation): Anchor · Breadcrumb · DropdownMenu · MenuBar · NavMenu · Pagination · Steps · Tabs · ContextMenu · FloatButton · CommandPalette
- [V. Data Entry](#v-data-entry): TextInput … FormItem (28 controls)
- [VI. Data Display](#vi-data-display): Table … Tour (33 controls)
- [VII. Feedback](#vii-feedback): Alert … Watermark (12 controls)
- [VIII. Utilities and Theming](#viii-utilities-and-theming): misc · theme · FpsMeter
- [IX. AI Conversations](#ix-ai-conversations): Welcome · Prompts · Bubble · Sender · Conversations · ThoughtChain
- [X. Advanced and Host Modules](#x-advanced-and-host-modules): HotkeyInput · Markdown · TransferManager · Terminal · CodeEditor · CommandBar · DropZone · TitleBar (8 modules)
- [Appendix A: Callback Signature Quick Reference](#appendix-a-callback-signature-quick-reference)
- [Appendix B: Entity Control List](#appendix-b-entity-control-list)
- [Appendix C: Controls Requiring Unique IDs](#appendix-c-controls-requiring-unique-ids)
- [Appendix D: Common Pitfalls FAQ](#appendix-d-common-pitfalls-faq)

---

## I. Core Concepts (Read Before Using Any Control)

Understanding these 6 points will enable you to use almost every control correctly.

### 1.1 Three State Models

Controls in the library fall into three categories based on **where state lives**, which determines how you create them and read or write their values:

| Model | Meaning | Representative Controls | Key Usage Point |
|---|---|---|---|
| **Controlled component** | The control **stores no state internally**. You pass in the value during render, modify a field on your own view `Entity` in the callback, and then call `cx.notify()` | Button / Checkbox / Select / Table / Calendar / Transfer … (the vast majority) | Rebuilt every frame; values come from `self.xxx` |
| **Entity (stateful view)** | The control internally holds state such as the cursor / selection / decoder / remaining seconds and is an `Entity<T>` | TextInput family / VideoPlayer / Countdown / CommandPalette | Create it **once** with `cx.new(|cx| T::new(cx))`; attach `.clone()` during render |
| **Wrapper** | Wraps arbitrary child content and adds a layer of behavior / visuals | Tooltip / Popover / ContextMenu / Watermark / CountBadge / FormItem | `Wrapper::new(…, child)` or `.child(…)` |

**Controlled component pattern** (the most important; covers the vast majority of controls):

```rust
struct MyApp { city: usize, city_open: bool }

impl Render for MyApp {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Select::new("city", ["Beijing", "Shanghai", "Guangzhou"], self.city)
            .open(self.city_open)
            .on_open_change(cx.listener(|this, open, _, cx| {
                this.city_open = *open;      // Write back the expanded state
                cx.notify();
            }))
            .on_select(cx.listener(|this, ix, _, cx| {
                this.city = *ix;             // Write back the selected value
                this.city_open = false;      // Collapse after a single selection
                cx.notify();
            }))
    }
}
```

**Entity pattern**:

```rust
// Creation (during main or parent view initialization; cx is Context<TextInput>):
name: cx.new(|cx| TextInput::new(cx).placeholder("Enter your name")),
// During render:
div().child(self.name.clone())
// Reading / writing in an event handler:
let text = self.name.read(cx).text();
self.name.update(cx, |i, cx| i.set_text("New value", window, cx));
```

> ⚠️ The `cx` passed to `TextInput::new(cx)` is a `Context<TextInput>` and can **only** be used inside a `cx.new(|cx| …)` closure. You cannot call `new` directly in the parent view's render / partition methods.

### 1.2 Three Callback Signature Families

Callbacks fall into three families based on their argument count, and **their construction methods differ**. Using the wrong construction method will fail to compile:

| Family | Signature | Construction Method | Examples |
|---|---|---|---|
| **3 arguments** | `Fn(&E, &mut Window, &mut App)` | `cx.listener(|this, event, window, cx| …)` | Button.on_click, Select.on_select, Table.on_row_click … (the vast majority) |
| **2 arguments** | `Fn(&mut Window, &mut App)` (no event argument) | `cx.entity()` handle + `entity.update(cx, …)` | Modal/Drawer.on_close, MenuItem.on_click, FloatButton.on_click, Popconfirm, Tour.on_close, Select/DatePicker.on_clear, Upload.on_click, Link.on_click |
| **1 argument** | `Fn(&mut App)` (timer context has no Window) | `cx.entity()` handle | Countdown.on_finish |
| **Special case** | `Fn(&Vec<PathBuf>, &mut App)` (asynchronous task has no Window) | `cx.entity()` handle | FilePicker.on_pick |

```rust
// 3 arguments: use cx.listener directly
Button::new("OK").on_click(cx.listener(|this, _event: &ClickEvent, _, cx| {
    this.do_something();
    cx.notify();
}))

// 2 arguments: use an entity handle (on_close has no event argument, so cx.listener does not apply)
let entity = cx.entity();
Modal::new("Title").on_close(move |_window, cx| {
    let _ = entity.update(cx, |this, cx| { this.modal_open = false; cx.notify(); });
})

// 1 argument: Countdown reaches zero (timer thread context)
let entity = cx.entity();
Countdown::after(Duration::from_secs(90), cx).on_finish(move |cx| {
    let _ = entity.update(cx, |this, cx| { /* Show a Toast */ });
})
```

> See [Appendix A](#appendix-a-callback-signature-quick-reference) for the exact callback event type of every control.

### 1.3 ID Uniqueness and Hover

GPUI determines hover / drag / keyed state by `ElementId`. **IDs of multiple instances in the same window must be unique**, otherwise hover highlights and drag state will interfere with one another. See [Appendix C](#appendix-c-controls-requiring-unique-ids) for controls that require IDs.

- String IDs use `impl Into<ElementId>` (for example, `"volume"` or `format!("row-{i}")`).
- When generating controls in a list, build unique IDs from the index / business key.

### 1.4 Popup Positioning Mechanism (Shared Convention)

The popups of Select / MultiSelect / DropdownMenu / MenuBar / Popover / DatePicker / DateRangePicker / ColorPicker / Cascader / TimePicker / TreeSelect / Popconfirm share the same positioning behavior:

- Open **4px below** the trigger; **automatically flip above** it when there is insufficient space below the window edge.
- **Click outside to dismiss**.
- Expanded state is **controlled**: write it back through `.open(bool)` + `.on_open_change(&bool)`.
- The menu width defaults to the trigger width and can be overridden with `.menu_width(px(…))` (Select / MultiSelect).

### 1.5 Theme System

All control colors come from the theme module's **21 semantic colors** (`bg()` / `surface()` / `accent()` / `text()` / `track()` / `text_disabled()` …), which can be configured globally:

```rust
theme::set_theme(theme::Theme::dark());     // Switch between light and dark globally
theme::set_accent(gpui::rgb(0x3B82F6));      // Change the brand accent (hover/pressed/soft backgrounds are derived automatically)
theme::update(|t| { t.success = …; });       // Fine-tune an individual semantic color
```

See [VIII. Utilities and Theming](#viii-utilities-and-theming) for details.

### 1.6 Startup Initialization (bind_keys)

Call these once as needed at application startup (once for the entire application, not once per view):

```rust
gpui_platform::application().run(|cx: &mut App| {
    gpui_widgets::text_input::bind_keys(cx);        // Required: input Ctrl+A/C/V/X and arrow keys
    // gpui_widgets::text_area::bind_keys(cx);       // When using TextArea (Enter/up/down/Shift selection)
    // gpui_widgets::command_palette::bind_keys(cx); // When using CommandPalette (Ctrl+K)
    // … open_window …
});
```

> Typical symptoms of forgetting `text_input::bind_keys`: the input accepts typing, but Ctrl+C/V, Select All, and arrow-key cursor movement do not work. Shortcuts are written with `secondary`, which `bind_keys` resolves to Cmd (macOS) or Ctrl (other platforms) for the host OS.

---

## II. General

### Button · Button

Triggers an action once. **Controlled component**.

- **Creation**: `Button::new(label: impl Into<SharedString>)`, for example, `Button::new("OK")`.
- **Properties**:
  - Variants: `primary()` / `secondary()` / `ghost()` / `danger()`.
  - Sizes: `small()` / `large()`.
  - `loading()`: loading state; **blocks clicks while active**.
  - `block()`: fill the entire row; `icon(char)`: leading icon character; `round()`: pill shape / `circle()`: circular shape; `disabled()`.
- **Callback**: `on_click(&ClickEvent)` (3 arguments).
- **Caveat**: both `loading()` and `disabled()` prevent the click callback from firing.

```rust
Button::new("Submit").primary().loading(self.submitting)
    .on_click(cx.listener(|this, _, _, cx| { this.submit(cx); }))
```

### Toolbar · Toolbar

A horizontal or vertical collection of action buttons with selected and disabled states, separators, and flexible spacers. **Controlled component**, **requires a unique ID**.

- **Creation**: `Toolbar::new(id, vec![ToolbarItem…])`; create an action with `ToolbarItem::new(id, label)`.
- **ToolbarItem properties**: `icon(element)`, `icon_only()`, `selected(bool)`, `disabled(bool)`, and `on_click(&mut Window, &mut App)`; structural entries are `ToolbarItem::separator()` / `ToolbarItem::spacer()`.
- **Toolbar properties**: `vertical(true)` switches to a vertical layout.
- **Callback**: `on_action(&SharedString, &mut Window, &mut App)` receives the action item ID; both the item-level `on_click` and the unified `on_action` fire.
- **Keyboard / accessibility**: uses `Role::Toolbar`; arrow keys move focus between actions, Home / End jump to the first / last action, selected items expose toggled state, and disabled items cannot receive focus or fire.
- **Caveat**: `icon_only()` still uses `label` as its accessible name and falls back to visible text when no icon is supplied.

```rust
Toolbar::new("editor-tools", vec![
    ToolbarItem::new("bold", "Bold").icon_only().selected(self.bold),
    ToolbarItem::separator(),
    ToolbarItem::spacer(),
    ToolbarItem::new("save", "Save"),
])
.vertical(false)
.on_action(cx.listener(|this, id, _, cx| { this.run_tool(id); cx.notify(); }))
```

### Link · Link

A text link underlined on hover. **Controlled component**.

- **Creation**: `Link::new(label)`.
- **Properties**: `href(url)`: open the system browser when clicked; `disabled()`.
- **Callback**: `on_click(&mut Window, &mut App)` (**2 arguments**, no event argument)—custom click behavior; use either this or `href`.
- **Caveat**: `on_click` belongs to the 2-argument family. Construct it with a `cx.entity()` handle; you cannot use `cx.listener`.

```rust
Link::new("Open website").href("https://gpui.rs")
// Or customize:
let entity = cx.entity();
Link::new("Details").on_click(move |_w, cx| { let _ = entity.update(cx, |this, cx| { … }); })
```

### Title / Paragraph / Text · Typography Trio

Supports display-only text, controlled editing, copying, and truncation with expansion. The parent view owns editing and expansion state.

- **Creation**:
  - `Title::new(level: u8, text)`: heading levels 1–5.
  - `Paragraph::new(text)`: paragraph.
  - `Text::new(text)`: inline text.
- **Shared properties**: `id()` for a stable identity; `title()` for a custom hover tooltip; `editable()` / `copyable()` / `ellipsis()` interaction configs.
- **Controlled editing**: `EditableConfig` provides `editing(bool)`, `on_editing_change(&bool)`, `on_change(&str)`, `on_start()`, `on_cancel()`, `on_end()`, and `max_length()`. Enter commits and fires `on_change`; Escape cancels without committing.
- **Copying**: `CopyConfig` provides `text()` for custom clipboard content, `tooltip()`, and `on_copy()`. The action icon briefly changes to ✓ after copying.
- **Truncation and expansion**: `EllipsisConfig` provides `max_chars()`, `expandable()`, `expanded(bool)`, `on_expand(&bool)`, `symbol()`, and `tooltip()`. Truncation respects Unicode grapheme clusters and shows the full text in a tooltip while collapsed; `truncate_text(text, max_chars, symbol)` is also available separately.
- **Text modifiers**: `strong()` / `italic()` / `underline()` / `delete()` / `code()` / `mark()` / `secondary()`; `Paragraph` supports every modifier.
- **Caveat**: provide a unique `id()` when a view contains multiple interactive instances. Size and color can still be forwarded through `Styled` (`.text_color()` / `.text_size()`, etc.).

```rust
Title::new(2, title)
    .id("article-title")
    .editable(
        EditableConfig::default()
            .editing(editing)
            .max_length(80)
            .on_editing_change(cx.listener(|this, editing, _, cx| {
                this.editing = *editing;
                cx.notify();
            }))
            .on_change(cx.listener(|this, value: &str, _, cx| {
                this.title = value.to_string();
                cx.notify();
            })),
    );

Paragraph::new("Click to copy this description")
    .copyable(CopyConfig::default().tooltip("Copy description"));

Text::new("This is a longer inline text")
    .ellipsis(
        EllipsisConfig::default()
            .max_chars(12)
            .expandable(true)
            .expanded(expanded)
            .on_expand(cx.listener(|this, expanded, _, cx| {
                this.expanded = *expanded;
                cx.notify();
            })),
    );
```

### Kbd · Keycap

Visualizes keyboard keys. **Controlled component** (no callbacks).

- **Creation**: `Kbd::new(key)` for a single key; `Kbd::combo(combo)` for a key combination (automatically splits it into multiple keycaps at `+`).
- **Caveat**: `Kbd::combo("Ctrl+K")` renders as two keycaps: `Ctrl` and `K`.

```rust
Kbd::new("Esc")
Kbd::combo("Ctrl+K")
```

---

## III. Layout

### Row / Col · 24-Column Grid

Responsive grid layout. **Controlled component** (no callbacks).

- **Creation**: `Row::new()`; `Col::new(span: u8)` (`span` is a share of the 1–24 column grid).
- **Properties**: `Col::offset(n)`: left offset in grid columns; `Row::gutter(px)`: column spacing.
- **Usage**: place several `Col`s inside a `Row`, and arbitrary content inside each `Col`.

```rust
Row::new().gutter(px(16.))
    .child(Col::new(12).child(left_panel))
    .child(Col::new(12).child(right_panel))
```

### Header / Sider / Content / Footer · Page Layout Quartet

The classic admin page skeleton. **Controlled component** (no callbacks).

- **Creation**: `Header::new()` / `Sider::new(width: f32)` / `Content::new()` / `Footer::new()`.
- **Usage**: an outer `div().flex().flex_col()` contains Header + a middle `div().flex().flex_1()` (Sider + Content) + Footer.

```rust
div().size_full().flex().flex_col()
    .child(Header::new().child(top_bar))
    .child(div().flex().flex_1()
        .child(Sider::new(220.).child(nav))
        .child(Content::new().child(main)))
    .child(Footer::new().child(footer))
```

### Space · Spacing Container

Controls child spacing consistently. **Controlled component** (no callbacks).

- **Creation**: `Space::new()`.
- **Properties**: `vertical()`: vertical arrangement; `wrap()`: allow wrapping; `size(px)`: spacing size.

```rust
Space::new().size(px(8.)).wrap()
    .child(Button::new("A"))
    .child(Button::new("B"))
```

### Splitter · Split Panel

Two panes with a draggable divider for adjusting their proportions. **Controlled component**, **requires a unique ID**.

- **Creation**: `Splitter::new(id: impl Into<ElementId>, left, right)`.
- **Properties**: `vertical()`: switch to stacked panes; `ratio(f32)`: controlled proportion (0–1).
- **Callback**: `on_change(&f32)` (3 arguments)—returns the new proportion when the divider is dragged.
- **Caveat**: `ratio` is controlled. Write `self.ratio` back in `on_change` and then call `cx.notify()`, or dragging will have no effect.

```rust
Splitter::new("main-split", left_panel, right_panel)
    .ratio(self.ratio)
    .on_change(cx.listener(|this, r, _, cx| { this.ratio = *r; cx.notify(); }))
```

### Scrollbar · Themed Scrollbar

GPUI does not draw scrollbars by default. This control provides visible, interactive scroll feedback using the library's theme colors. **Wrapper / bound persistent ScrollHandle**, **requires a unique ID**.

- **Creation**: `Scrollbar::horizontal(id, handle)` / `Scrollbar::vertical(id, handle)`, where `handle: ScrollHandle`.
- **Visuals**: custom-drawn track (`theme::track()`) + thumb (`theme::text_disabled()`); hover / drag highlights use the accent color (`theme::accent()`). Automatically hidden when the content does not overflow (`max <= 0`).
- **Interaction**: thumb dragging (the pointer may leave the track while held), track-click jumping, and scroll-wheel driven movement.
- **Key conventions**:
  1. The bound `ScrollHandle` **must persist across frames** (store it in a view field or `window.use_keyed_state`).
  2. The scroll container must attach `.track_scroll(&handle)`—both share the same offset, allowing `set_offset` to drive scrolling and keeping the thumb synchronized with the scroll wheel in real time.
  3. The parent container must use `.relative()`, while the scrollbar uses `.absolute()` to stay at the edge.
- **Caveat**: `Table` `fixed_left()` / `fixed_right_columns(n)` already includes a horizontal scrollbar; you do not need to add one manually.

```rust
// View field: scroll_handle: ScrollHandle (initialize with the ScrollHandle::new() constructor)
// During render:
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

### Flex · Flex Layout

A lightweight wrapper over GPUI flex styles. Defaults to horizontal, no-wrap, `stretch` alignment, and `justify-start`. **Controlled component** (no callbacks, no ID required).

- **Creation**: `Flex::new()`.
- **Direction / wrapping**: `row()` / `column()`; `wrap()` / `wrap_reverse()` / `nowrap()`.
- **Spacing**: `gap(px)`.
- **Cross-axis alignment**: `items_start()` / `items_center()` / `items_end()` / `items_baseline()` / `items_stretch()`.
- **Main-axis alignment**: `justify_start()` / `justify_center()` / `justify_end()` / `justify_between()` / `justify_around()` / `justify_evenly()`.
- **Own flexing**: `grow(f32)` / `shrink(f32)` / `basis(impl Into<Length>)`.
- **Children**: `child(...)` / `children(...)`.

```rust
Flex::new()
    .wrap()
    .gap(px(8.))
    .items_center()
    .justify_between()
    .child("Left")
    .children((1..=4).map(|i| format!("Item {i}")))
```

### Affix · Sticky Top / Bottom

Pins its child to the top or bottom of a scroll viewport after the scroll crosses a threshold. **Wrapper / bound persistent ScrollHandle**, **requires a unique ID**.

- **Creation**: `Affix::new(id, child, scroll: ScrollHandle)`.
- **Properties**: `offset_top(px)` selects top pinning; `offset_bottom(px)` selects bottom pinning. Defaults to top with zero offset.
- **Key conventions**: the scroll container must use the same `ScrollHandle` and attach `.overflow_y_scroll().track_scroll(&scroll)`; the handle must persist across frames.
- **Implementation**: a placeholder preserves document-flow height, while the real child stays mounted. `anchored()` positions it in window coordinates, and `deferred` drawing keeps it above the scrolling content, so switching between pinned and unpinned states does not unmount the child or lose its internal state.

```rust
let scroll = window
    .use_keyed_state("page-scroll", cx, |_, _| ScrollHandle::new())
    .read(cx)
    .clone();

div().id("page").size_full().overflow_y_scroll().track_scroll(&scroll)
    .child(Affix::new("page-top", toolbar, scroll.clone()).offset_top(px(0.)))
    .child(long_content)
```

### StatusBar · Status Bar

A compact information bar for the bottom of an application. Supports text items, arbitrary content, separators, and elastic spacing. **Controlled component** (no callbacks), **requires a unique ID**; its accessibility role is `Status`.

- **Creation**: `StatusBar::new(id)`.
- **Content**: `item(label)` adds text inheriting the bar's color and font size; `child(...)` / `children(...)` add arbitrary elements.
- **Separators / spacing**: `separator()` adds a vertical divider; `spacer()` adds elastic space that pushes following items to the right.
- **Visuals**: height, padding, font size, and colors are all derived from the active tokens by `theme::metric::status_bar_metric`.

```rust
StatusBar::new("editor-status")
    .item("Ready")
    .separator()
    .item("Ln 1, Col 1")
    .spacer()
    .item("UTF-8")
    .separator()
    .item("Rust")
```

---

## IV. Navigation

### Anchor · Anchor Navigation

Jumps to sections in an in-page table of contents. **Controlled component**.

- **Creation**: `AnchorItem::new(key, title)` (can contain nested entries); `Anchor::new(items)`.
- **Properties**: an indicator track on the left + current-item highlighting; `active(key)`: controlled current item.
- **Callback**: `on_click(&SharedString)` (3 arguments)—returns the key of the clicked item.
- **Caveat**: **the caller handles scroll positioning** (the control only reports clicks; it does not automatically scroll to the target).

```rust
Anchor::new(vec![
    AnchorItem::new("intro", "Introduction"),
    AnchorItem::new("usage", "Usage"),
]).active(self.active_anchor.clone())
  .on_click(cx.listener(|this, key, _, cx| {
      this.active_anchor = key.clone();
      // Scroll to the corresponding section yourself
      cx.notify();
  }))
```

### Breadcrumb · Breadcrumb

Hierarchical path navigation. **Controlled component**.

- **Creation**: `Breadcrumb::new(items)` (`items` is an iterable collection of text).
- **Callback**: `on_click(&usize)` (3 arguments)—returns the index of the clicked level.
- **Caveat**: **the last item is the current page and cannot be clicked**; callbacks only apply to intermediate levels.

```rust
Breadcrumb::new(["Home", "Products", "Details"])
    .on_click(cx.listener(|this, ix, _, cx| { this.navigate(*ix); }))
```

### DropdownMenu · Action Menu

An arbitrary trigger element + a dropdown action list. **Controlled component**, **requires a unique ID**.

- **Creation**: `MenuItem::new(label)`; `DropdownMenu::new(id, trigger, items)`.
- **MenuItem properties**: separators, `danger()` for destructive actions, `disabled()` for disabled items, and `submenu(label, items)` for recursive multilevel menus.
- **MenuItem callback**: `on_click(&mut Window, &mut App)` (**2 arguments**); submenu parent items do not register their own action callback.
- **Behavior**: hovering or clicking a submenu parent opens it to the right, recursive levels are supported, and it flips left near the right window edge. Actions dismiss the menu; see [1.4](#14-popup-positioning-mechanism-shared-convention) for popup positioning.

```rust
let entity = cx.entity();
DropdownMenu::new("more", Button::new("More"), vec![
    MenuItem::submenu("Export", vec![
        MenuItem::new("PNG").on_click({
            let entity = entity.clone();
            move |_w, cx| { let _ = entity.update(cx, |this, cx| { … }); }
        }),
        MenuItem::new("PDF"),
    ]),
    MenuItem::new("Delete").danger(),
])
```

### MenuBar · Application Menu Bar

A desktop-style top-level menu that reuses `MenuItem` submenus, danger items, and disabled items. **Controlled component**, **requires a unique ID**.

- **Creation**: `MenuBarItem::new(id, label)`; pass `.children(vec![MenuItem…])` for a submenu, or omit children for a direct action button.
- **MenuBarItem properties**: `icon(element)`, `disabled(bool)`, and `on_click(&mut Window, &mut App)`.
- **MenuBar properties**: `open(Option<usize>)` controls the expanded top-level item, `selected(Option<usize>)` controls the selected item, and `menu_width(px)` sets the dropdown width.
- **Callbacks**: `on_open_change(&Option<usize>, &mut Window, &mut App)`; `on_select(usize, &SharedString, &mut Window, &mut App)` receives the top-level index and action label.
- **Behavior**: clicking a top-level item opens it; while a menu is open, hovering another top-level item switches directly; clicking outside or activating an action dismisses it; submenus open to the right and flip left near the right window edge.
- **Keyboard / accessibility**: uses `Role::MenuBar` plus top-level `Role::Button` nodes with expanded / selected state; Left / Right wrap, Home / End jump, keyboard switching follows an open menu, and Esc dismisses it.
- **Caveat**: write `open` and `selected` back in callbacks; submenu parents have no action callback, and actions fire from leaf items.

```rust
MenuBar::new("app-menu", vec![
    MenuBarItem::new("file", "File").children(vec![
        MenuItem::new("New"),
        MenuItem::separator(),
        MenuItem::new("Exit").danger(),
    ]),
    MenuBarItem::new("help", "Help"),
])
.open(self.open_menu)
.selected(self.selected_menu)
.on_open_change(cx.listener(|this, open, _, cx| {
    this.open_menu = *open;
    cx.notify();
}))
```

### NavMenu · Navigation Menu

Multilevel sidebar navigation. **Controlled component**, **requires a unique ID**.

- **Creation**: `MenuEntry::new(key, label)` (can contain nested submenus); `NavMenu::new(id, entries)`.
- **Properties**: icon / disabled; both the selected item and expanded set are controlled.
- **Callbacks**: `on_select(&SharedString)` (3 arguments; returns the selected key); `on_toggle(&SharedString)` (3 arguments; returns the key that was expanded / collapsed).
- **Caveat**: `MenuEntry` is identified by its **key**, not an index.

```rust
NavMenu::new("side-nav", entries)
    .selected(self.nav_selected.clone())
    .expanded(self.nav_expanded.clone())
    .on_select(cx.listener(|this, key, _, cx| { this.nav_selected = key.clone(); cx.notify(); }))
    .on_toggle(cx.listener(|this, key, _, cx| { /* Add/remove from this.nav_expanded */ cx.notify(); }))
```

### Pagination · Pagination

Page-number navigation. **Controlled component**, **requires a unique ID**.

- **Creation**: `Pagination::new(id, current, total)` (**page numbers start at 1**; `total` is the total number of pages).
- **Properties**: automatically collapses with ellipses when there are many pages; boundary buttons are disabled; `simple()`: compact mode; `small()`: small size.
- **Callback**: `on_change(&usize)` (3 arguments)—returns the new page number.

```rust
Pagination::new("pager", self.page, self.total_pages)
    .on_change(cx.listener(|this, p, _, cx| { this.page = *p; cx.notify(); }))
```

### Steps · Stepper

Displays process progress. **Controlled component** (no callbacks).

- **Creation**: `Steps::new(titles: Vec<SharedString>)`.
- **Properties**: completed ✓ / current number / gray pending state; `current(usize)`: controlled current step; `vertical()`: vertical layout; `small()`; `progress_dot()`: dotted style; `descriptions(vec)`: step descriptions.

```rust
Steps::new(vec!["Fill in".into(), "Review".into(), "Complete".into()])
    .current(self.step)
    .descriptions(vec!["Enter information".into(), "Awaiting review".into(), "Process finished".into()])
```

### Tabs · Tabs

Switches between tab pages. **Controlled component**.

- **Creation**: `Tabs::new(items, active)` (`items` is a collection of tab labels; `active` is the current index).
- **Properties**: underline indicator; `disabled_tabs(vec)`: disabled tabs; `closable()`: allow closing; `card()`: card style; `centered()`: centered; `scrollable()`: shows ‹ › scroll arrows on overflow and automatically reveals the active tab.
- **Callbacks**: `on_select(&usize)` (3 arguments; selected index); `on_close(&usize)` (3 arguments; closed index; requires `closable()`); `on_add` (2 arguments; shows a "+" button).
- **Caveat**: the caller renders the content area based on `active` (Tabs only provides the tab bar). `centered()` is ignored in `scrollable()` mode; antd's default "»" ellipsis dropdown is not implemented.

```rust
Tabs::new(["Overview", "Details", "Settings"], self.tab)
    .closable()
    .on_select(cx.listener(|this, ix, _, cx| { this.tab = *ix; cx.notify(); }))
    .on_close(cx.listener(|this, ix, _, cx| { /* Remove tab ix */ cx.notify(); }))
```

### ContextMenu · Context Menu

Wraps arbitrary content and opens a menu on right-click. **Wrapper**, **requires a unique ID**.

- **Creation**: `ContextMenu::new(id: impl Into<ElementId>, items)`, then `.child(arbitrary content)`.
- **Behavior**: opens the `MenuItem` menu at the cursor on right-click, clamped within the viewport; `MenuItem::submenu(...)` is also supported.
- **MenuItem callback**: `on_click` (2 arguments).

```rust
ContextMenu::new("ctx", vec![MenuItem::new("Copy"), MenuItem::new("Paste")])
    .child(my_content)
```

### FloatButton · Floating Button

A circular button fixed in the bottom-right corner of a container. **Controlled component**, **requires a unique ID**.

- **Creation**: `FloatButton::new(id, label)`.
- **Properties**: `bottom(px)` / `right(px)` (default 24px); `tooltip(text)`.
- **Callback**: `on_click(&mut Window, &mut App)` (**2 arguments**).
- **Caveat**: the button uses absolute positioning, so the **parent container must use `.relative()`**.

```rust
div().relative().size_full()
    .child(content)
    .child(FloatButton::new("fab", "↑").tooltip("Back to top")
        .on_click(move |_w, cx| { … }))
```

### CommandPalette · Command Palette

A searchable command selector opened with Ctrl+K. **Entity**, with a backdrop + centered panel.

- **Creation**: `PaletteItem::new(label)` (can include a keyboard shortcut hint); `cx.new(|cx| CommandPalette::new(cx, items))`.
- **Methods**: `open(window, cx)` / `close(cx)`.
- **Callback**: `on_select(&SharedString)` (3 arguments)—returns the selected command label.
- **Integration steps**:
  1. Register Ctrl+K at startup with `command_palette::bind_keys(cx)`.
  2. Attach `.on_action(cx.listener(|this, _: &command_palette::Toggle, window, cx| { this.palette.update(cx, |p, cx| p.open(window, cx)); }))` to the host view.
  3. Render the panel entity **outside the scroll container** (it uses deferred rendering to stay on top).

```rust
// Field: palette: Entity<CommandPalette>
palette: cx.new(|cx| CommandPalette::new(cx, vec![
    PaletteItem::new("New file"),
    PaletteItem::new("Open settings"),
]).on_select(…)),
// At the end of render (outside the scroll container):
.child(self.palette.clone())
```

---

## V. Data Entry

### Common Notes for Input Entities

TextInput / TextArea / PasswordInput / SearchInput / NumberInput / TagInput / Autocomplete / Mentions are all **Entities (stateful views)** that internally hold state such as the cursor / selection / draft. They share the following pattern:

```rust
// Create once (cx is Context<this control>):
field: cx.new(|cx| TextInput::new(cx).placeholder("…")),
// Attach during render:
div().child(self.field.clone())
// Read:
let s = self.field.read(cx).text();
// Write (set_text requires window):
self.field.update(cx, |i, cx| i.set_text("…", window, cx));
```

`text_input::bind_keys(cx)` is required at startup (TextArea additionally requires `text_area::bind_keys(cx)`).

### TextInput · Single-Line Input

The most basic text input. **Entity**.

- **Creation**: `cx.new(|cx| TextInput::new(cx))`.
- **Properties**: `placeholder(text)`; `bare()` removes the border (for wrapping); `masked()` masks the display; `disabled()` / `readonly()` / `error()` states; `allow_clear()` ✕ clear button; `max_length(n)` character limit; `prefix(el)` / `suffix(el)` prefix and suffix.
- **Reading / writing**: `text()` / `set_text(s, window, cx)`.
- **Capabilities**: cursor / selection / clipboard / Chinese input method (IME).
- **Caveat**: `set_text` requires `&mut Window`; pass the outer `window` into the `entity.update` closure.

### TextArea · Multiline Input

Multiline text editing. **Entity**.

- **Creation**: `cx.new(|cx| TextArea::new(cx))`.
- **Properties**: `height(px)`: height; otherwise the same as TextInput.
- **Capabilities**: hard line breaks / cross-line selection / clipboard / IME / automatic cursor scrolling.
- **Caveat**: **additionally** requires `text_area::bind_keys(cx)` (Enter / up and down arrow keys / Shift selection), or line breaks and vertical movement will not work.

### PasswordInput · Password Input

Masking + plaintext toggle. **Entity**.

- **Creation**: `cx.new(|cx| PasswordInput::new(cx))`.
- **Capabilities**: masked display + a **custom-drawn eye icon** to toggle between plaintext and masking.
- **Reading / writing**: the same as TextInput (`text()` returns the actual plaintext).

### OtpInput · One-Time Password Input

A single-focus, per-cell verification-code / PIN input. **Entity**.

- **Creation**: `cx.new(|cx| OtpInput::new(cx, 6))`; length is clamped to 1–32.
- **Properties**: `label(text)`, `placeholder(text)`, `mask(bool)`, `separator(bool)`, `variant(InputVariant)`, `status(InputStatus)`, `size(InputSize)`, `disabled(bool)`.
- **Reading / writing**: `value()` / `set_value(s, cx)` / `clear(cx)`; `length()`, `active_index()`, `is_complete()`, `focus(window, cx)`.
- **Keyboard / clipboard**: Left / Right / Home / End move the active cell; Backspace / Delete clear it; Ctrl / Cmd+V pastes a whole string from the active cell. Only Unicode letters and numbers are accepted.
- **Callbacks**: `on_change(&str, &mut Window, &mut App)`; `on_complete(&str, &mut Window, &mut App)` fires only on the incomplete-to-complete transition. Programmatic `set_value` does not fire callbacks.
- **a11y**: `Role::TextInput` exposes value / placeholder / disabled / invalid and supports SetValue.

```rust
code: cx.new(|cx| OtpInput::new(cx, 6)
    .on_change(cx.listener(|this, value, _, cx| {
        this.code = value.to_string();
        cx.notify();
    }))
    .on_complete(cx.listener(|this, value, _, cx| {
        this.verify(value, cx);
    }))),
```

### SearchInput · Search Input

A search box with a magnifying glass and clear button. **Entity**.

- **Creation**: `cx.new(|cx| SearchInput::new(cx))`.
- **Capabilities**: custom-drawn magnifying glass icon + ✕ clear button.
- **Callbacks**: `on_change(&str)` (3 arguments; every input change); `on_search(&str)` (3 arguments; **triggered by Enter**).
- **Caveat**: connect the search action to `on_search`; do not rely only on `on_change`.

```rust
search: cx.new(|cx| SearchInput::new(cx).on_search(cx.listener(|this, q, _, cx| {
    this.query = q.to_string();
    this.run_search(cx);
}))),
```

### NumberInput · Number Input

A numeric field with step controls. **Entity**.

- **Creation**: `cx.new(|cx| NumberInput::new(cx))`.
- **Properties**: `min(f64)` / `max(f64)` / `step(f64)`.
- **Capabilities**: ▲▼ buttons / up and down arrow-key stepping; invalid input does not trigger callbacks.
- **Callback**: `on_change(Option<f64>, &mut Window, &mut App)` (3 arguments; **the first argument is passed by value**, not by reference; `None` means empty / invalid).
- **Reading / writing**: `value()` / `set_value()`.

```rust
num: cx.new(|cx| NumberInput::new(cx).min(0.).max(100.).step(1.)
    .on_change(cx.listener(|this, v: Option<f64>, _, cx| {
        this.amount = v.unwrap_or(0.);
        cx.notify();
    }))),
```

### Stepper · Numeric Stepper

Steps a value with − / + buttons. **Controlled component**, **requires a unique ID**.

- **Creation**: `Stepper::new(id, value: f32)`.
- **Properties**: `min(f32)` / `max(f32)` / `step(f32)` / `disabled()`; the corresponding button is automatically disabled at a boundary.
- **Callback**: `on_change(&f32)` (3 arguments).
- **Difference**: Stepper is a controlled component (the value lives in the parent view), whereas NumberInput is an Entity (the value lives inside the control).

```rust
Stepper::new("qty", self.qty).min(0.).max(99.).step(1.)
    .on_change(cx.listener(|this, v, _, cx| { this.qty = *v; cx.notify(); }))
```

### TagInput · Tag Input

Adds tags on Enter. **Entity**.

- **Creation**: `cx.new(|cx| TagInput::new(cx))`.
- **Properties**: `initial_tags(vec)`: initial tags.
- **Capabilities**: add on Enter (automatically deduplicated) / Backspace with an empty draft removes the last tag / ✕ removes an individual tag.
- **Callback**: `on_change(&Vec<SharedString>)` (3 arguments).
- **Reading / writing**: `tags()`.

### Autocomplete · Autocomplete

Filters candidates as the user types. **Entity**.

- **Creation**: `cx.new(|cx| Autocomplete::new(cx, items))`.
- **Capabilities**: filters candidates case-insensitively; ↑↓ to select / Enter to confirm.
- **Callback**: `on_select(&SharedString)` (3 arguments; returns the selected candidate).

### Mentions · @ Mention Input

Opens candidate suggestions when typing @. **Entity**.

- **Creation**: `cx.new(|cx| Mentions::new(cx).options(vec![…]))`.
- **Capabilities**: after `@` is entered, opens a candidate popup if no whitespace follows; selecting a candidate replaces the input with "@name ".
- **Callbacks**: `on_change(&str)` (3 arguments); `on_select(&SharedString)` (3 arguments; returns the selected name).
- **Reading / writing**: the same as TextInput (`text()` / `set_text()`).

### Select · Dropdown Selector

A single-select dropdown. **Controlled component**, **requires a unique ID**.

- **Creation**: `Select::new(id, items, selected: usize)`.
- **Properties**:
  - `open(bool)` + `on_open_change(&bool)`: controlled expanded state.
  - `value(Option<usize>)`: nullable selection (`None` displays the placeholder).
  - `allow_clear()`: ✕ clear button.
  - `searchable()`: a search box at the top of the menu to filter options.
  - `menu_width(px)`: override the default menu width (which equals the trigger width).
- **Callbacks**: `on_select(&usize)` (3 arguments); `on_open_change(&bool)` (3 arguments); `on_clear(&mut Window, &mut App)` (**2 arguments**).
- **Behavior**: the menu opens below the trigger, automatically flips above when space below is insufficient, dismisses on an outside click, and closes after a single selection.

```rust
Select::new("city", ["Beijing", "Shanghai", "Guangzhou"], self.city)
    .open(self.city_open)
    .on_open_change(cx.listener(|this, o, _, cx| { this.city_open = *o; cx.notify(); }))
    .on_select(cx.listener(|this, ix, _, cx| { this.city = *ix; this.city_open = false; cx.notify(); }))
```

### MultiSelect · Multi-Select Dropdown

A multi-select dropdown. **Controlled component**, **requires a unique ID**.

- **Creation**: `MultiSelect::new(id, items, selected: Vec<usize>)`.
- **Properties**: the trigger displays selected chips (✕ removes individual items); the menu shows check marks; `open()` / `menu_width()` are the same as Select.
- **Callbacks**: `on_change(&Vec<usize>)` (3 arguments; returns all selected indices); `on_open_change(&bool)` (3 arguments).
- **Behavior**: selecting an item **does not dismiss** the menu (making multi-selection easier).

```rust
MultiSelect::new("tags", items, self.selected.clone())
    .open(self.ms_open)
    .on_open_change(cx.listener(|this, o, _, cx| { this.ms_open = *o; cx.notify(); }))
    .on_change(cx.listener(|this, sel, _, cx| { this.selected = sel.clone(); cx.notify(); }))
```

### Cascader · Cascading Selector

Expands multilevel menus horizontally. **Controlled component**, **requires a unique ID**.

- **Creation**: `CascaderNode::new(value, label)` (can be nested with `.children(vec)`); `Cascader::new(id, options)`.
- **Properties**: the path value is controlled (`Vec<SharedString>`); `open()` + `on_open_change()`.
- **Callbacks**: `on_change(&Vec<SharedString>)` (3 arguments; returns the complete path values); `on_open_change(&bool)` (3 arguments).
- **Behavior**: clicking a leaf dismisses the menu.

```rust
let options = vec![
    CascaderNode::new("zj", "Zhejiang").children(vec![
        CascaderNode::new("hz", "Hangzhou"),
        CascaderNode::new("nb", "Ningbo"),
    ]),
];
Cascader::new("region", options).value(self.region.clone())
    .on_change(cx.listener(|this, path, _, cx| { this.region = path.clone(); cx.notify(); }))
```

### TreeSelect · Tree Selector

Embeds a Tree inside a dropdown popup. **Controlled component**, **requires a unique ID**.

- **Creation**: `TreeSelect::new(id, nodes)` (reuses `TreeNode`).
- **Properties**: the selected ID and expanded set are controlled; `open()` + `on_open_change()`.
- **Callbacks**: `on_change(&SharedString)` (3 arguments; returns the selected node ID); `on_toggle(&SharedString)` (3 arguments; expand / collapse); `on_open_change(&bool)` (3 arguments).

### Checkbox · Checkbox

A checkbox. **Controlled component**.

- **Creation**: `Checkbox::new(label, checked: bool)`.
- **Properties**: `indeterminate()`: indeterminate state.
- **Callback**: `on_change(&bool)` (3 arguments).

```rust
Checkbox::new("I agree to the terms", self.agreed)
    .on_change(cx.listener(|this, v, _, cx| { this.agreed = *v; cx.notify(); }))
```

### Radio · Radio Button

One item in a radio group. **Controlled component**.

- **Creation**: `Radio::new(label, selected: bool)`.
- **Callback**: `on_select(&ClickEvent)` (3 arguments).
- **Usage**: combine with parent view state to form a radio group—each Radio's `selected` is determined by `self.choice == i`, and `on_select` sets `self.choice = i`.

```rust
div().flex().gap_3()
    .child(Radio::new("Male", self.gender == 0).on_select(cx.listener(|this, _, _, cx| { this.gender = 0; cx.notify(); })))
    .child(Radio::new("Female", self.gender == 1).on_select(cx.listener(|this, _, _, cx| { this.gender = 1; cx.notify(); })))
```

### Switch · Switch

Boolean toggling. **Controlled component**.

- **Creation**: `Switch::new(on: bool)` (can include a label).
- **Properties**: `small()`; `loading()`: loading state; `text(on, off)`: text inside the track.
- **Callback**: `on_change(&bool)` (3 arguments).

```rust
Switch::new(self.enabled).text("On", "Off")
    .on_change(cx.listener(|this, v, _, cx| { this.enabled = *v; cx.notify(); }))
```

### Slider · Slider

Click-to-position + dragging. **Controlled component**, **requires a unique ID**.

- **Creation**: `Slider::new(id, value: f32)`.
- **Properties**: `range(min, max)`; `step(f32)`; `vertical()`: vertical orientation; `marks(vec)`: tick dots + labels.
- **Callback**: `on_change(&f32)` (3 arguments).
- **Caveat**: multiple instances in the same window require unique IDs, or drag / hover state will interfere.

```rust
Slider::new("volume", self.volume).range(0., 100.).step(1.)
    .on_change(cx.listener(|this, v, _, cx| { self.volume = *v; cx.notify(); }))
```

### Rating · Star Rating

Selects a star count. **Controlled component**, **requires a unique ID**.

- **Creation**: `Rating::new(id, value: usize)`.
- **Properties**: `max(n)`: maximum star count; `readonly()`: read-only.
- **Callback**: `on_change(&usize)` (3 arguments).
- **Behavior**: clicking the current star count clears the rating.

### DatePicker · Date Picker

Selects a date from a calendar popup. **Controlled component**, **requires a unique ID**.

- **Creation**: `DatePicker::new(id, value: Option<Date>)`.
- **Properties**: `month(Option<(i32, u32)>)`: controlled displayed month; `disabled()`; `allow_clear()`; `open()` + `on_open_change()`.
- **Callbacks**:
  - `on_change(&Option<Date>)` (3 arguments).
  - `on_month_change(&(i32, u32))` (3 arguments).
  - `on_open_change(&bool)` (3 arguments).
  - `on_clear(&mut Window, &mut App)` (**2 arguments**).
- **Caveat**: when `month` is omitted, the month containing the selected value is displayed by default (or the current month if there is no value); **month navigation does not work without `on_month_change`**.

```rust
DatePicker::new("date", self.date)
    .month(self.dp_month)
    .open(self.dp_open)
    .on_open_change(cx.listener(|this, o, _, cx| { this.dp_open = *o; cx.notify(); }))
    .on_month_change(cx.listener(|this, m, _, cx| { this.dp_month = Some(*m); cx.notify(); }))
    .on_change(cx.listener(|this, d, _, cx| { this.date = *d; self.dp_open = false; cx.notify(); }))
```

### DateRangePicker · Date Range Picker

Selects start and end dates. **Controlled component**, **requires a unique ID**.

- **Creation**: `DateRangePicker::new(id, value: Option<(Date, Date)>)`.
- **Properties**: value / expanded state / month are controlled; `allow_clear()`; dates within the range are highlighted.
- **Callbacks**: `on_change(&Option<(Date, Date)>)` (3 arguments); `on_month_change(&(i32, u32))`; `on_open_change(&bool)`; `on_clear` (2 arguments).
- **Behavior**: click the start date and then the end date (if the end is earlier than the start, start over by selecting a new start); the pending start is persisted across frames by ID and automatically discarded on dismissal / clearing; the displayed month defaults to the start date's month.

### TimePicker · Time Picker

Three scrolling menu columns for hours / minutes / seconds. **Controlled component**, **requires a unique ID**.

- **Creation**: `TimePicker::new(id)`.
- **Properties**: the value `Option<(u32, u32, u32)>` is controlled; `open()` + `on_open_change()`.
- **Callbacks**: `on_change(&(u32, u32, u32))` (3 arguments); `on_open_change(&bool)` (3 arguments).

### ColorPicker · Color Picker

A preset palette of 24 colors. **Controlled component**, **requires a unique ID**.

- **Creation**: `ColorPicker::new(id, value: Rgba)`.
- **Capabilities**: the trigger displays a color swatch + HEX; selection immediately triggers the callback and dismisses the popup.
- **Callbacks**: `on_change(&Rgba)` (3 arguments); `on_open_change(&bool)` (3 arguments).

```rust
ColorPicker::new("brand", self.brand_color)
    .on_change(cx.listener(|this, c, _, cx| { this.brand_color = *c; cx.notify(); }))
```

### Transfer · Transfer

Checks and moves items between left and right panels. **Controlled component**, **requires a unique ID**.

- **Creation**: `TransferItem::new(key, label)`; `Transfer::new(id, items)`.
- **Properties**: `target_keys(vec)`: controlled set in the right panel; `selected(vec)`: controlled set of currently checked items; `titles(left, right)`: custom titles.
- **Callbacks**: `on_change(&Vec<SharedString>)` (3 arguments; writes back the right panel); `on_select(&Vec<SharedString>)` (3 arguments; writes back the checked items).
- **Caveat**: **both sets must be written back as controlled state**, or moving / checking will not work.

```rust
Transfer::new("perm", items)
    .target_keys(self.target.clone())
    .selected(self.checked.clone())
    .on_change(cx.listener(|this, keys, _, cx| { this.target = keys.clone(); cx.notify(); }))
    .on_select(cx.listener(|this, keys, _, cx| { this.checked = keys.clone(); cx.notify(); }))
```

### Upload · Upload Trigger Area

A dashed box that triggers an upload when clicked. **Controlled component**.

- **Creation**: `Upload::new()` (default ID `"upload-area"`).
- **Properties**: `text(t)` / `hint(t)`: labels; `disabled()`.
- **Callback**: `on_click(&mut Window, &mut App)` (**2 arguments**).
- **Caveat**: **click-triggered only**; the caller implements the file dialog / upload logic. Native drag-and-drop is not built in.

### UploadList · Uploaded File List

Displays files that are uploading / complete / failed. **Controlled component**, **requires a unique ID**.

- **Creation**: `UploadFile::new(name)` (can set the Done / Uploading / Error status and progress); `UploadList::new(id, files)`.
- **Callback**: `on_remove(&usize)` (3 arguments; returns the index of the removed item).

### FilePicker · File Selection Button

Uses the system's native file dialog. **Controlled component**.

- **Creation**: `FilePicker::new(label)`.
- **Properties**: `multiple()`: multiple selection; `directories()`: select directories; `prompt(text)`: dialog title.
- **Callback**: `on_pick(&Vec<PathBuf>, &mut App)` (**special case**: called from an asynchronous task; **no `&mut Window`**).
- **Caveat**: construct it with a `cx.entity()` handle; canceling the dialog does not trigger the callback.

```rust
let entity = cx.entity();
FilePicker::new("Select file").multiple().on_pick(move |paths, cx| {
    let paths = paths.clone();
    let _ = entity.update(cx, |this, cx| { this.files = paths; cx.notify(); });
})
```

### Form · Form Layout Container

Arranges FormItems + controlled validation helpers. **Controlled component**.

- **Creation**: `Form::new()`.
- **Properties**: vertical layout by default; `inline()`: horizontal wrapping layout; `gap(px)`: item spacing.
- **Validation**: use the companion `Rule` + `Form::validate()`:
  - `Rule::required(message)`: non-empty (after trimming).
  - `Rule::min_len(n, message)` / `Rule::max_len(n, message)`: minimum / maximum character count.
  - `Rule::custom(message, |v| bool)`: custom assertion.
  - `Form::validate(value: &str, rules: &[Rule]) -> Option<SharedString>`: returns the error message from the first failed rule.
- **Caveat**: the control itself **does not hold validation state**, and the caller decides when to submit; store the `validate` result in view state and pass it to `FormItem::error_opt()` for display.

```rust
let err = Form::validate(&name, &[
    Rule::required("Enter your name"),
    Rule::min_len(2, "At least 2 characters"),
]);
self.name_error = err;   // Option<SharedString>
```

### FormItem · Form Item Wrapper

A label + error message + help text. **Wrapper**.

- **Creation**: `FormItem::new(label)`, then `.child(input control)`.
- **Properties**: `required()`: red asterisk; `error(text)` / `error_opt(Option<SharedString>)`: red-bordered error message; `help(text)`: help text; `label_width(px)`: horizontal layout (label on the left).

```rust
FormItem::new("Name").required()
    .error_opt(self.name_error.clone())
    .help("Enter your real name")
    .child(self.name_input.clone())
```

---

## VI. Data Display

### Table · Data Table

The most feature-rich display control. **Controlled component**, **requires a unique ID**.

- **Creation**: `Table::new(id, headers, rows)`, where `rows: Vec<Vec<SharedString>>`.
- **Properties**:
  - `column_widths(vec)`: column widths (equal distribution by default).
  - `striped()`: zebra striping; `bordered()`: full borders; `show_header(bool)`: header visibility.
  - Controlled row selection + `on_row_click()`.
  - `sortable()` + `sort(col, asc)` + `on_sort()`: header sorting.
  - `selectable()` + `selected_rows(vec)` + `on_selected_rows()`: multi-selection.
  - `pagination(page, page_size)` + `on_page_change()`: built-in pagination.
  - `row_detail(fn)` + `expanded_rows(vec)` + `on_expand()`: expandable rows.
  - `fixed_left()`: pin the first column + horizontal scrolling (**includes a horizontal Scrollbar**).
  - `fixed_right_columns(n)`: pin the last n columns; combine with `fixed_left()` for a left / scrolling / right three-pane layout.
  - `resizable_columns(true)` + `on_column_resize()`: drag header edges to resize; handles support Left/Right by 8px and enforce a 64px minimum.
  - `filterable()` + `filter(query)` + `on_filter_change()`: case-insensitive substring filtering, applied before pagination.
  - `editable_columns(vec)` + `editing(Option<(row, col)>)` + `on_edit_start()` / `on_cell_change()` / `on_edit_end()`: controlled cell editing.
  - `virtualized(px(viewport_height))` + `virtual_scroll_handle(handle)`: virtual scrolling (only visible rows are mounted, **uniform 37px row height**, includes a vertical Scrollbar).
  - Automatically displays an empty state when there is no data.
- **Row-related callbacks** (all 3 arguments; **all use global row indices**):
  - `on_row_click(&usize)`
  - `on_sort(&(usize, bool))`: column index + whether ascending.
  - `on_selected_rows(&Vec<usize>)`
  - `on_page_change(&usize)`: new page number (starting at 1).
  - `on_expand(&usize)`
- **DataGrid callbacks**:
  - `on_column_resize(&Vec<Pixels>)`: full column-width snapshot; write it back through `column_widths()`.
  - `on_filter_change(&str)`: filter text changed.
  - `on_edit_start(&(usize, usize))` / `on_edit_end(&(usize, usize))`: editing started / ended.
  - `on_cell_change(&((usize, usize), SharedString))`: committed cell value.
- **Caveats**:
  - Pagination is controlled (page numbers start at 1); header Select All **applies only to the current page**.
  - Filtering runs before pagination; sorting / selection / expansion / editing callbacks still use **global row indices**.
  - `fixed_left()` / `fixed_right_columns(n)` require a uniform row height (37px); columns without an explicit width default to 120px.
  - `fixed_left()` / `fixed_right_columns(n)` and `row_detail()` are **mutually exclusive**—if both are set, expandable rows do not work.
  - `resizable_columns(true)` requires `on_column_resize()` to write widths back. Double-click an editable cell, or focus it and press Enter/Space; press Enter or blur to commit.
  - `virtualized()` falls back to full rendering when combined with `row_detail()` / `fixed_left()`, and paginates the mounted window when `pagination()` is set.

```rust
Table::new("users", vec!["Name".into(), "Phone".into(), "Status".into()], rows)
    .column_widths(vec![px(120.), px(160.), px(100.)])
    .striped().selectable().sortable()
    .selected_rows(self.selected.clone())
    .pagination(self.page, 10)
    .fixed_left()
    .on_selected_rows(cx.listener(|this, sel, _, cx| { this.selected = sel.clone(); cx.notify(); }))
    .on_page_change(cx.listener(|this, p, _, cx| { self.page = *p; cx.notify(); }))
    .on_sort(cx.listener(|this, (col, asc), _, cx| { this.sort_by(*col, *asc); cx.notify(); }))
```

See [`examples/data_grid.rs`](../examples/data_grid.rs) for the complete DataGrid interaction example.

### Tree · Tree

Nested nodes + checkboxes + dragging. **Controlled component**, **requires a unique ID**.

- **Creation**: `TreeNode::new(id: impl Into<SharedString>, label)` (can be nested with `.children(vec)`; `Clone`); `Tree::new(id, nodes)`.
- **Properties**:
  - The expanded set (`Vec<SharedString>`) / selected ID (`Option<SharedString>`) is controlled.
  - `checkable()` + `checked(vec)` + `on_check()`: parent/child checkbox synchronization.
  - `draggable()` + `on_drop_node()`: drag to move.
- **Callbacks** (all 3 arguments):
  - `on_toggle(&SharedString)`: expanded / collapsed node ID.
  - `on_select(&SharedString)`: selected node ID.
  - `on_check(&Vec<SharedString>)`: checked set.
  - `on_drop_node(&TreeDrop)`: `TreeDrop { dragged, target }`.
- **Caveats**:
  - Node IDs only need to be unique within the same tree.
  - Checkbox synchronization is **built in** (checking a node includes all descendants; ancestors synchronize automatically when all children are checked).
  - Dragging only **reports** `TreeDrop` (it does not fire when the target is the node itself or one of its descendants); the caller restructures the node tree using `take_node()` / `find_node_mut()` / `push_child()`.

```rust
Tree::new("org", nodes)
    .expanded(self.expanded.clone())
    .selected(self.selected.clone())
    .checkable().checked(self.checked.clone())
    .on_toggle(cx.listener(|this, id, _, cx| { /* Add/remove from this.expanded */ cx.notify(); }))
    .on_select(cx.listener(|this, id, _, cx| { this.selected = Some(id.clone()); cx.notify(); }))
    .on_check(cx.listener(|this, ids, _, cx| { this.checked = ids.clone(); cx.notify(); }))
```

### TreeTable · Hierarchical Table

Table columns combined with hierarchical nodes. **Controlled component**, **requires a unique ID**. Node IDs must be unique within the same tree, and callbacks always use node IDs rather than visible row indices.

- **Creation**: `TreeTableNode::new(id, cells)` (`cells` contains the row's column values; nest with `.child(node)` / `.children(nodes)`; `Clone`); `TreeTable::new(id, headers, nodes)`.
- **Properties**:
  - Controlled `expanded(Vec<SharedString>)` and `selected(Option<SharedString>)`.
  - `checkable()` + `checked(Vec<SharedString>)`: parent-child checkbox synchronization; ancestors become checked when all children are checked and show a mixed state otherwise.
  - `column_widths(Vec<Pixels>)`, `striped()`, `bordered()`, and `show_header(bool)`.
  - `virtualized(height)`: fixed row height plus `uniform_list` virtualization; `virtual_scroll_handle(handle)` reuses an external scroll handle.
- **Callbacks** (all 3 arguments): `on_toggle(&SharedString)`, `on_select(&SharedString)`, and `on_check(&Vec<SharedString>)`.
- **Keyboard**: after focusing the table, ↑ / ↓ move, Home / End jump to the first / last row; ← collapses the current node or moves to its parent; → expands the current node or moves to its first child; Enter selects; Space toggles checking when checkboxes are enabled.
- **Pure helpers**: `tree_table_visible_ids()`, `tree_table_toggle_expanded()`, `tree_table_navigation_target()`, `tree_table_check_state()`, and `tree_table_cascade_check()` can be used directly for host state management or matrix tests.
- **Accessibility**: `Role::Table` for the container; `Role::ColumnHeader` for headers; table row/cell semantics; expand buttons expose `Role::Button` plus expanded state; checkboxes expose `Role::CheckBox` plus mixed state.
- **Performance**: visible rows are flattened depth-first; virtual mode instantiates only visible rows. Row height is fixed, so cell text is single-line ellipsized.
- **Caveat**: the first version does not include fixed left/right columns, column dragging, cell editing, or sorting; those belong to the P2 DataGrid expansion.

```rust
TreeTable::new("files-table", ["Name", "Size", "Status"], self.nodes.clone())
    .expanded(self.expanded.clone())
    .selected(self.selected.clone())
    .checkable().checked(self.checked.clone())
    .column_widths(vec![px(320.), px(120.), px(180.)])
    .virtualized(px(480.))
    .on_toggle(cx.listener(|this, id, _, cx| {
        this.expanded = tree_table_toggle_expanded(&this.expanded, id);
        cx.notify();
    }))
    .on_select(cx.listener(|this, id, _, cx| {
        this.selected = Some(id.clone());
        cx.notify();
    }))
    .on_check(cx.listener(|this, ids, _, cx| {
        this.checked = ids.clone();
        cx.notify();
    }))
```

### List · List

A vertical list. **Controlled component** (no callbacks).

- **Creation**: `ListItem::new(title)` (can set a description / avatar / `extra()` right-side element); `List::new(items)`.
- **Virtual scrolling**: `List::virtualized(px(viewport_height), px(row_height), count, |ix| ListItem)` + `virtual_scroll_handle(handle)` mounts only visible rows; each row is built on demand by the factory, and header / footer stay fixed outside the viewport.
- **Properties**: `header(el)` / `footer(el)`.

```rust
List::new(vec![
    ListItem::new("Item one").description("Description text"),
    ListItem::new("Item two").extra(Button::new("Action")),
]).header(Text::new("List title").strong())

List::virtualized(px(520.), px(56.), 10_000, |ix|
    ListItem::new(format!("Session {ix:05}")).description("host-000.internal"))
    .header(Text::new("Sessions").strong())
```

> Virtual scrolling requires a **uniform row height**: padding for multi-line rows (description, avatar, separators) must fit within the row height you specify.

### Card · Card

A content container with a title. **Controlled component** (no callbacks).

- **Creation**: `Card::new(title)`.
- **Properties**: `extra(el)`: top-right action; `cover(el)`: cover; `action(el)`: bottom action area; `bordered()`; `hoverable()`: raised shadow on hover.

```rust
Card::new("Card title").hoverable()
    .extra(Link::new("More").href("…"))
    .child(body_content)
    .action(Button::new("OK").primary())
```

### Descriptions · Descriptions

Displays key-value pairs. **Controlled component** (no callbacks).

- **Creation**: `Descriptions::new(title)`.
- **Properties**: `item(label, value)`: key-value pair; `column(n)`: number of columns per row; `bordered()`: table style; `vertical()`: vertical layout (label above value).

```rust
Descriptions::new("User information").bordered().column(2)
    .item("Name", "Zhang San")
    .item("Phone", "138-0000-0000")
```

### Statistic · Statistic

Displays a large number. **Controlled component** (no callbacks).

- **Creation**: `Statistic::new(title, value)`.
- **Properties**: `prefix(el)` / `suffix(el)`; `trend(up)`: increase / decrease arrow.

### Countdown · Countdown

Day/hour/minute/second unit blocks. **Entity**.

- **Creation**: `cx.new(|cx| Countdown::new(unix_secs: u64, cx))` or `Countdown::after(Duration, cx)`.
- **Capabilities**: recalculates every second from the system clock (preventing drift); triggers `on_finish` once when it reaches zero and then stops counting.
- **Callback**: `on_finish(&mut App)` (**1 argument**; timer context has no Window).
- **Reading / writing**: `remaining()` / `finished()`.
- **Caveat**: to restart, create a new entity; after the old entity is released, the timer loop exits automatically without leaking.

```rust
let entity = cx.entity();
countdown: cx.new(|cx| {
    Countdown::after(Duration::from_secs(90), cx).on_finish(move |cx| {
        let _ = entity.update(cx, |this, cx| { /* Show a Toast */ });
    })
}),
// During render:
div().child(self.countdown.clone())
```

### Calendar · Calendar

Month view / year view. **Controlled component**, **requires a unique ID**, **fully controlled**.

- **Creation**: `Calendar::new(id, month: (i32, u32))`.
- **Properties**: `value(Option<Date>)`: selected date; `month((year, month))`: displayed month; `mode(CalendarMode)`: month / year view.
- **Callbacks** (all 3 arguments): `on_month_change(&(i32, u32))`; `on_select(&Date)`; `on_mode_change(&CalendarMode)`.
- **Caveat**: **all three callbacks must be handled**, or the corresponding operations will not work; clicking a mini calendar in year view returns to month view.

```rust
Calendar::new("cal", self.cal_month)
    .value(self.cal_date).mode(self.cal_mode)
    .on_month_change(cx.listener(|this, m, _, cx| { this.cal_month = *m; cx.notify(); }))
    .on_select(cx.listener(|this, d, _, cx| { this.cal_date = Some(*d); cx.notify(); }))
    .on_mode_change(cx.listener(|this, mode, _, cx| { self.cal_mode = *mode; cx.notify(); }))
```

### Timeline · Timeline

A list of time nodes. **Controlled component** (no callbacks).

- **Creation**: `TimelineItem::new(title)` (can set a description / time / node color); `Timeline::new(items)`.
- **Properties**: `alternate()`: alternate left and right; `pending()`: ghost node.

### Tag · Tag

A compact marker. **Controlled component**.

- **Creation**: `Tag::new(label)`.
- **Properties**: `color(TagColor)`: 8 preset colors; `bordered()`: border toggle.
- **Callback**: `on_close(&ClickEvent)` (3 arguments)—setting it displays a ✕ remove button.

```rust
Tag::new("Tag").color(TagColor::Blue)
    .on_close(cx.listener(|this, _, _, cx| { /* Remove this tag */ cx.notify(); }))
```

### Badge · Badge

A status marker. **Controlled component** (no callbacks).

- **Creation**: `Badge::new(label)`.
- **Pill mode**: `info()` / `success()` / `warning()` / `danger()`; `color(c)` sets a custom color.
- **Status-dot mode**: `status(BadgeStatus::{Default, Processing, Success, Error, Warning})` renders a token-colored dot plus text; `text()` updates the label.
- **Tooltip and accessibility**: `title()` provides both a hover tooltip and an accessible description; `aria_label()` overrides the accessible name.
- **Unique id**: pass distinct `id()` values when multiple badges use the same text.

```rust
Badge::new("Running").success()
Badge::new("Processing")
    .id("job-processing")
    .status(BadgeStatus::Processing)
    .title("The task is processing")
    .aria_label("Task status: processing")
```

### CountBadge · Count Badge

Displays a count in the top-right corner of a child element. **Wrapper**.

- **Creation**: `CountBadge::new(count: u32)`, then `.child(child element)`.
- **Properties**: `max(n)` displays 99+ on overflow; `dot()` enables red-dot mode; `show_zero()`; `color(c)` sets a custom background; `offset(x, y)` adjusts position.
- **Size**: `size(BadgeSize::Small)` uses a compact 14px-high badge; the default remains 20px.
- **Tooltip and accessibility**: `title()` adds a hover tooltip; `aria_label()` overrides the default "N messages" name.

```rust
CountBadge::new(5).max(99).child(Avatar::new("Zhang San"))
CountBadge::new(8)
    .size(BadgeSize::Small)
    .title("Compact count")
    .aria_label("Eight unread messages")
    .child(Avatar::new("Li Si"))
```

### Avatar · Avatar

An avatar showing the first character of a name or an image. **Controlled component**; drive the image failure state through `image_error()`.

- **Creation**: `Avatar::new(name)`.
- **Capabilities**: circular background + the first character of the name, using the fixed `colorTextPlaceholder` background; `image(source)` is cropped with cover and falls back to the initial while loading or on failure.
- **Properties**: `size(px)` sets a custom edge length and scales font size/line height; `square()` rounded square; `image_error(bool)` forces the failure fallback.
- **Callback**: `on_image_error(&ImageCacheError, &mut Window, &mut App)`, suitable for driving controlled failure state.
- **Unique ID**: pass distinct `id()` values for same-name avatars in one window to avoid element and accessibility ID collisions.

```rust
Avatar::new("Zhang San")
    .id("user-42-avatar")
    .size(px(48.))
    .image(source)
    .image_error(self.avatar_failed)
    .on_image_error(cx.listener(|this, _error, _, cx| {
        this.avatar_failed = true;
        cx.notify();
    }))
```

### ImageView · Image

Wraps GPUI `img`. **Controlled component** (no callbacks).

- **Creation**:
  - `ImageView::new(source)`: uses the GPUI asset system (Embedded); **not a disk path**.
  - `ImageView::from_file(path)`: local file.
  - `ImageView::from_render(frame)`: in-memory frame.
- **Properties**: `fit_cover()` / `fit_contain()` / `fit_fill()`; `rounded()`; `grayscale()`; size / layout forwarded through `Styled` (`.w()` / `.h()` / `.flex()`).
- **Capabilities**: built-in loading and load-failure placeholders, customizable with `.loading(|| …)` / `.fallback(|| …)`.
- **Caveat**: **always use `from_file` for local files**; `new("string")` is not a disk path.

```rust
ImageView::from_file("d:/assets/photo.png").w(px(200.)).h(px(150.)).fit_cover().rounded()
```

### VideoPlayer · Video Player

**Windows only**. **Entity**.

- **Creation**: `cx.new(|cx| VideoPlayer::new(cx, path))`; starts playing automatically after creation.
- **Capabilities**: background decoding with Media Foundation; built-in play/pause button + progress slider.
- **Methods**: `play(cx)` / `pause(cx)` / `toggle(cx)` / `seek(secs)`.
- **Reading / writing**: `position()` / `duration()` (seconds) / `is_playing()`.
- **Caveat**: decoding runs on a background thread, and frames are pumped back to the UI thread through a channel; playback stops automatically at the end, and another `play()` restarts from the beginning; the module does not compile on non-Windows platforms.

```rust
video: cx.new(|cx| VideoPlayer::new(cx, "examples/assets/sample.avi")),
// During render:
div().child(self.video.clone())
// Controls:
self.video.update(cx, |v, cx| v.pause(cx));
```

### Carousel · Carousel

Cycles through images. **Controlled component**, **requires a unique ID**.

- **Creation**: `Carousel::new(id: impl Into<ElementId>, sources)` (a list of `ImageSource`s).
- **Capabilities**: ‹ › to cycle + bottom dots for direct navigation + a counter.
- **Properties**: `current(usize)`: controlled.
- **Callback**: `on_change(&usize)` (3 arguments).

### BarChart · Bar Chart

A custom-drawn bar chart. **Controlled component** (no callbacks).

- **Creation**: `BarDatum::new(label, value: f32)` (can set an individual bar color); `BarChart::new(data)`.
- **Properties**: `height(px)`; `max(f32)`: maximum value.

### LineChart · Line Chart

A custom-drawn path-based line chart. **Controlled component** (no callbacks).

- **Creation**: `LineChart::new(values: Vec<f32>)`.
- **Properties**: grid; `area()`: filled area; `dots()`: circular points; `labels(vec)`: X-axis labels.

### Sparkline · Sparkline

A small line chart without axes. **Controlled component** (no callbacks).

- **Creation**: `Sparkline::new(values: Vec<f32>)`.
- **Properties**: `w(px)` / `h(px)` / `area()`.
- **Use case**: embedding in tables / cards.

### PieChart · Pie Chart / Donut Chart

Custom GPU path rendering of sectors + percentage labels + a legend. **Controlled component** (no callbacks).

- **Creation**: `PieDatum::new(label, value: f32)` (can set an individual sector color with `.color()`); `PieChart::new(data: Vec<PieDatum>)`.
- **Properties**: `size(px)`: diameter; `donut(true)`: donut chart (inner radius 0.6) / `inner_ratio(0–0.85)`; `legend(bool)`: legend on the right; `labels(bool)`: sector percentages.
- **Colors**: the default theme categorical palette `theme::chart_color(i)`.
- **Caveat**: the donut center displays the total; negative `value`s are treated as 0.

```rust
PieChart::new(vec![
    PieDatum::new("Direct", 335.),
    PieDatum::new("Search engine", 548.),
]).donut(true).size(px(160.))
```

### GaugeChart · Gauge

A 270° arc (opening at the bottom) + pointer + central value. **Controlled component** (no callbacks).

- **Creation**: `GaugeChart::new(value: f32)`.
- **Properties**: `max(f32)`: upper range limit (default 100); `size(px)`; `thickness(px)`: arc thickness; `title(text)`: description below the value; `label(text)`: central text (the current value by default); `color()`: solid color; `zones(vec![(ratio limit, Hsla)])`: threshold color zones (ascending; the first zone whose limit >= the current ratio applies).
- **Colors**: uses `theme::accent()` when neither zones nor `color` are set.

```rust
GaugeChart::new(72.).max(100.).title("CPU usage")
    .zones(vec![(0.6, theme::success()), (0.85, theme::warning()), (1.0, theme::danger())])
```

### ScatterChart · Scatter Chart

Multiple scatter series + grid + axes + legend. **Controlled component** (no callbacks).

- **Creation**: `ScatterPoint::new(x: f32, y: f32)`; `ScatterSeries::new(name, points)` (can use `.color()`); `ScatterChart::new(series: Vec<ScatterSeries>)`.
- **Properties**: `height(px)`; `x_max(f32)` / `y_max(f32)` (defaults to the data maximum plus 5% headroom); `dot_radius(px)`; `legend(bool)`.
- **Caveat**: assumes non-negative coordinates with origin 0.

### Heatmap · Heatmap

A matrix of colored cells + row/column labels + a color-scale legend; colors are interpolated between the two endpoint colors based on values. **Controlled component** (no callbacks).

- **Creation**: `Heatmap::new(values: Vec<Vec<f32>>)` (a row-major two-dimensional matrix).
- **Properties**: `row_labels(vec)`: left / `col_labels(vec)`: top; `cell_size(px)`: cell side length; `gap(px)`; `colors(low, high)`: color-scale endpoints (default `accent_soft` → `accent`); `legend(bool)`: color scale at the bottom.
- **Implementation**: rounded cells rendered with divs (not paths), using `lerp_color` interpolation.

### RadarChart · Radar Chart

A multi-axis grid ring + axes + multiple series with translucent fills. **Controlled component** (no callbacks).

- **Creation**: `RadarSeries::new(name, values: Vec<f32>)` (can use `.color()`); `RadarChart::new(axes: Vec<SharedString>, series: Vec<RadarSeries>)`.
- **Properties**: `max(f32)`: maximum value; `size(px)`; `levels(usize)`: number of grid levels; `legend(bool)`.
- **Caveat**: each series' `values` corresponds one-to-one with `axes`.

### FunnelChart · Funnel Chart

Stacked centered trapezoids + stage labels + conversion rates. **Controlled component** (no callbacks).

- **Creation**: `FunnelStage::new(label, value: f32)` (can use `.color()`); `FunnelChart::new(stages: Vec<FunnelStage>)`.
- **Properties**: `height(px)`; `gap(px)`: spacing between stages.
- **Capabilities**: trapezoid widths are proportional to `value/max`; automatically labels the conversion rate relative to the previous stage (on the right).

```rust
FunnelChart::new(vec![
    FunnelStage::new("Visit", 1000.),
    FunnelStage::new("Sign up", 620.),
    FunnelStage::new("Pay", 280.),
]).height(px(220.))
```

### QRCode · QR Code

Pure-Rust encoding and custom rendering. **Controlled component** (no callbacks).

- **Creation**: `QRCode::new(data: impl AsRef<str>)`.
- **Properties**: `size(px)`: pixel size.
- **Capabilities**: byte mode / ECC L / versions 1–6, with no external dependencies.
- **Caveat**: holds at most **134 bytes** (approximately 44 UTF-8 Chinese characters); longer data is truncated. Suitable for short text / URLs; logos and error-correction level switching are not supported.

### CodeBlock · Code Block

Displays code in a monospace font. **Controlled component** (no callbacks).

- **Creation**: `CodeBlock::new(code)`.
- **Capabilities**: line-by-line rendering + language label + one-click copy button.

### Empty · Empty State

A placeholder for no data. **Controlled component** (no callbacks).

- **Creation**: `Empty::new(title)`.
- **Properties**: `description(text)`; `action(el)`: action area.

### Segmented · Segmented Control

Mutually exclusive options inside a rounded container. **Controlled component**.

- **Creation**: `Segmented::new(items, active)`.
- **Properties**: `small()` / `large()`; `block()`: fill the entire row.
- **Callback**: `on_select(&usize)` (3 arguments).
- **Use case**: 2–5 short options.

```rust
Segmented::new(["Day", "Week", "Month"], self.period)
    .on_select(cx.listener(|this, ix, _, cx| { self.period = *ix; cx.notify(); }))
```

### Accordion · Accordion

Allows multiple panels to be expanded simultaneously. **Controlled component**, **requires a unique ID**.

- **Creation**: `AccordionItem::new(title, content)`; `Accordion::new(id, items)`.
- **Properties**: the expanded set (`Vec<usize>`) is controlled.
- **Callback**: `on_change(&Vec<usize>)` (3 arguments; returns the set of expanded indices).

```rust
Accordion::new("faq", vec![
    AccordionItem::new("Question one", answer_one),
    AccordionItem::new("Question two", answer_two),
]).expanded(self.expanded.clone())
  .on_change(cx.listener(|this, ids, _, cx| { self.expanded = ids.clone(); cx.notify(); }))
```

### Tooltip · Tooltip

A hover bubble. **Wrapper** (no callbacks).

- **Creation**: `Tooltip::new(text, child)`.
- **Capabilities**: displays a bubble when hovering over any control; delay and positioning are handled natively by GPUI.

```rust
Tooltip::new("This is a tooltip", Button::new("Hover over me"))
```

### Popover · Popover Card

An interactive content popup. **Wrapper / controlled**, **requires a unique ID**.

- **Creation**: `Popover::new(id, trigger, content)`.
- **Properties**: expanded state is controlled (`open()` + `on_open_change()`).
- **Callback**: `on_open_change(&bool)` (3 arguments).
- **Difference**: heavier than Tooltip, with interactive content.

### Tour · Step-by-Step Guide

A backdrop + centered card + step dots. **Controlled component**.

- **Creation**: `TourStep::new(title, desc)`; `Tour::new(steps)`.
- **Properties**: `current(usize)`: controlled; skip / previous / next or finish.
- **Callbacks**: `on_change(&usize)` (3 arguments; step changed); `on_close(&mut Window, &mut App)` (**2 arguments**; close the guide).
- **Caveat**: render outside the scroll container.

---

## VII. Feedback

### Alert · Alert

A static in-page message. **Controlled component**.

- **Creation**: `Alert::new(title)`.
- **Properties**: `info()` / `success()` / `warning()` / `danger()`; `description(text)`; `banner()`: full-width mode (no border or rounded corners).
- **Callback**: `on_close(&mut Window, &mut App)` (**2 arguments**)—setting it displays ✕.

```rust
Alert::new("Operation successful").success().description("Your data has been saved.")
```

### Modal · Modal Dialog

A backdrop + centered card. **Controlled component**.

- **Creation**: `Modal::new(title)`.
- **Properties**: `width(px)`: card width; `closable()`: ✕ close button; `mask_closable()`: close on backdrop click; `footer(el)`: action area; `open(bool)`: controlled visibility.
- **Callback**: `on_close(&mut Window, &mut App)` (**2 arguments**).
- **Caveat**: **must be rendered outside the scroll container** (as the last child of the root view).

```rust
let entity = cx.entity();
// At the end of render (outside the scroll container):
if self.modal_open {
    root = root.child(
        Modal::new("Confirm").width(px(420.))
            .footer(Button::new("Cancel").on_click(/* … */))
            .on_close(move |_w, cx| { let _ = entity.update(cx, |this, cx| { this.modal_open = false; cx.notify(); }); })
            .child(body),
    );
}
```

### Drawer · Drawer

A backdrop + side-sliding panel. **Controlled component**.

- **Creation**: `Drawer::new(title)`.
- **Properties**: `placement(DrawerPlacement)`: left / right (right by default); `width(px)`; `footer(el)`; `closable()` / `mask_closable()`; `open(bool)`.
- **Callback**: `on_close(&mut Window, &mut App)` (**2 arguments**).
- **Caveat**: **must be rendered outside the scroll container**.

### ToastStack · Toast Stack

Lightweight toast messages in a top-center overlay. **Controlled component**; the caller owns the collection and lifecycle.

- **Creation**: `ToastItem::new(message)` (four types: info/success/warning/error); `ToastStack::new(items)`.
- **Stable identity**: `ToastItem::key(key)` produces element id `toast-{key}` and close-button id `toast-close-{key}`. Unkeyed controlled entries fall back to `index-{index}`, so list entries should always provide a business key.
- **Properties**: items are controlled (`Vec<ToastItem>`); `duration()` / `pause_on_hover()` are consumed only by `ToastHost`. `ToastStack` does not run a timer.
- **Callback**: prefer `on_dismiss(&ToastDismissed, &mut Window, &mut App)` (3 arguments). The event contains `key: Option<SharedString>`, the current `index`, and `by_timer`. Legacy `on_close(&usize)` remains available.
- **Caveat**: render outside the scroll container; the caller manages the collection for automatic dismissal / manual removal. Use `ToastHost` for antd-style automatic expiry.

### ToastHost · Self-managed Toasts

Lightweight toast messages in a top-center overlay. **Entity component** that owns entries, absolute deadlines, hover pause, max-count, and one ticker.

- **Creation**: `cx.new(|cx| ToastHost::new(cx).config(ToastConfig { max_count: 6, pause_on_hover: true }))`; mount `host.clone()` during render.
- **Default TTLs**: `default_toast_duration(variant)` uses 3s for Info / Success, 4.5s for Warning, and 6s for Danger, matching antd Message.
- **Entry API**: `show(item, cx)` uses the item or variant default; `show_with(item, duration, cx)` overrides it; `Duration::ZERO` means persistent. `ToastItem::pause_on_hover(bool)` overrides the global setting.
- **Stable keys**: an explicit `key` wins; otherwise the host assigns `toast-{n}`. Showing the same key again updates it in place and restarts its timer.
- **Count and order**: newest entries are inserted at the top; entries beyond `max_count` are dropped from the tail, preserving the remaining order.
- **Callback**: `on_dismiss(&ToastDismissed, &mut App)` (2 arguments, no Window). Manual and timer dismissal both provide the stable key, current index, and `by_timer`.
- **Control API**: `dismiss(&key, cx) -> bool`, `clear(cx)`, `len()`, and `is_empty()`.
- **Performance**: at most one 50ms ticker per host. It runs only while an unpaused timer exists and exits after all entries expire or are cleared; there is no async task per notice.
- **Caveat**: place it as the last child of the root view, outside the scroll container. `ToastHost` is an `Entity`; do not put it inside a controlled `ToastStack` entry list.

### NotificationStack · Notification Stack

Notifications in a top-right overlay. **Controlled component**.

- **Creation**: `NotificationItem::new(title)` (can set a description / type / `action()` button); `NotificationStack::new(items)`.
- **Stable identity**: `NotificationItem::key(key)` produces element id `notification-{key}`, close-button id `notif-close-{key}`, and action-button id `notification-action-{key}`. Unkeyed entries fall back to `index-{index}`.
- **Properties**: items are controlled; `duration()` / `pause_on_hover()` are consumed only by `NotificationHost`.
- **Callback**: prefer `on_dismiss(&NotificationDismissed, &mut Window, &mut App)` (3 arguments). The event contains `key`, the current `index`, and `by_timer`. Legacy `on_close(&usize)` remains available.
- **Caveat**: render outside the scroll container; the caller manages the collection for automatic dismissal / manual removal.

### NotificationHost · Self-managed Notifications

Notifications in a top-right overlay. **Entity component** that owns entries, expiry, hover pause, max-count, and one ticker.

- **Creation**: `cx.new(|cx| NotificationHost::new(cx).config(NotificationConfig { max_count: 4, pause_on_hover: true }))`; mount `host.clone()` during render.
- **Default TTL**: `default_notification_duration()` is 4.5s, matching antd Notification; `Duration::ZERO` means persistent.
- **Entry API**: `show(item, cx)` / `show_with(item, duration, cx)`; `NotificationItem::pause_on_hover(bool)` overrides the global setting.
- **Stable keys**: an explicit `key` wins; otherwise the host assigns `notification-{n}`. Showing the same key again updates it in place and restarts its timer.
- **Count and order**: newest entries are inserted at the top; entries beyond `max_count` are dropped from the tail.
- **Callback**: `on_dismiss(&NotificationDismissed, &mut App)` (2 arguments, no Window), distinguishing manual and timer dismissal.
- **Control API**: `dismiss(&key, cx) -> bool`, `clear(cx)`, `len()`, and `is_empty()`.
- **Performance and placement**: at most one 50ms ticker per host, exiting when no active timer remains; place it as the last child of the root view, outside the scroll container.

### NotificationCenter · Notification Center

A right-side notification-history overlay. **Controlled component**; the parent view owns the entries.

- **Creation**: `NotificationCenter::new(id, items)`; `id` must be a stable, unique `ElementId` within the window.
- **Properties**: `open(bool)`, `selected(Option<usize>)`, `title(text)`, `empty_text(text)`; an empty collection shows the empty state.
- **Entry capabilities**: reuses `NotificationItem`, adding `.read(bool)`, `.set_read(bool)`, and `.timestamp(text)`; read them with `is_read()`, `timestamp_text()`, `description_text()`, `title()`, and `variant()`.
- **Callbacks**: `on_select(&usize, &mut Window, &mut App)`, `on_read(&usize, &mut Window, &mut App)`, `on_remove(&usize, &mut Window, &mut App)`, `on_mark_all_read(&mut Window, &mut App)`, and `on_dismiss(&mut Window, &mut App)`. The last two have no event argument, so they cannot use `cx.listener` directly; capture a `cx.entity()` handle and update the parent inside the closure.
- **Interaction**: clicking an entry selects it and, when unread, triggers `on_read`; ✕ removes it; the header can mark all as read or close the panel; ↑ / ↓ wrap, Home / End jump to the first / last entry, and Esc closes it.
- **Performance**: fixed row height plus `uniform_list` virtualization; long histories instantiate only visible rows, and titles / descriptions are single-line ellipsized.
- **Accessibility**: the panel uses `Role::Dialog`, rows use `Role::ListItem` with selected state, and remove / header actions use `Role::Button`.
- **Caveat**: render as the last child of the root view, outside the scroll container; the caller redraws after changing the collection.

```rust
NotificationCenter::new("inbox-center", self.items.clone())
    .open(self.center_open)
    .selected(self.selected)
    .on_select(cx.listener(|this, index, _, cx| {
        this.selected = Some(*index);
        cx.notify();
    }))
    .on_read(cx.listener(|this, index, _, cx| {
        if let Some(item) = this.items.get_mut(*index) {
            item.set_read(true);
            cx.notify();
        }
    }))
    .on_remove(cx.listener(|this, index, _, cx| {
        this.items.remove(*index);
        cx.notify();
    }))
```

### Popconfirm · Popconfirm

A lightweight confirmation attached to a trigger. **Wrapper / controlled**, **requires a unique ID**.

- **Creation**: `Popconfirm::new(id, title)`, then `.child(trigger element)`.
- **Properties**: `description(text)`; `ok_text(t)` / `cancel_text(t)`; `open()` + `on_open_change()`.
- **Callbacks**: `on_confirm(&mut Window, &mut App)` (**2 arguments**); `on_cancel(&mut Window, &mut App)` (**2 arguments**); `on_open_change(&bool)` (3 arguments).

```rust
let entity = cx.entity();
Popconfirm::new("del-confirm", "Are you sure you want to delete?").description("This action cannot be undone")
    .on_confirm(move |_w, cx| { let _ = entity.update(cx, |this, cx| { this.delete(); cx.notify(); }); })
    .child(Button::new("Delete").danger())
```

### ProgressBar · Progress Bar

Linear progress. **Controlled component** (no callbacks).

- **Creation**: `ProgressBar::new(value: f32)` (**0.0–1.0**).
- **Properties**: `steps(n)`: segmented progress.

### RingProgress · Ring Progress

Custom-drawn arc progress. **Controlled component** (no callbacks).

- **Creation**: `RingProgress::new(value: f32)` (0.0–1.0).
- **Properties**: `size(px)`; `thickness(px)`; `color(c)`; `label(text)`: central text.

### Result · Result Page

Full-page feedback for an operation's outcome. **Controlled component** (no callbacks).

- **Creation**: `Result::new(status: ResultStatus, title)` (four status icons: success/error/info/warning).
- **Properties**: `description(text)`; `action(el)`: action area.

### Skeleton · Skeleton Screen

Animated breathing placeholders for loading. **Controlled component** (no callbacks).

- **Creation**: `Skeleton::new()`.
- **Properties**: `circle()`: circular placeholder; `lines(n)`: number of lines; `w_percent(f32)`: width percentage.

### Spinner · Loading Indicator

A rotating, fading ring of dots. **Controlled component** (no callbacks).

- **Creation**: `Spinner::new()`.
- **Properties**: `size(px)`; `color(c)`; `tip(text)`: loading message.

### Watermark · Watermark

Tiles text across the content area. **Wrapper** (no callbacks).

- **Creation**: `Watermark::new(text)`, then `.child(content)`.
- **Properties**: `font_size(px)`; `color(c)`; `gap(px)`.
- **Capabilities**: staggered tiling on alternating rows; **does not intercept child control interactions**.

```rust
Watermark::new("Internal use only").font_size(px(14.)).gap(px(80.))
    .child(confidential_content)
```

---

## VIII. Utilities and Theming

### misc · Layout Helpers

A collection of lightweight layout helpers (not controls; functions / simple structures).

- `card()`: a quick card container.
- `Divider`: a divider line.
- `section_title()`: a section title.
- `caption()`: secondary explanatory text.
- `field_row()`: a horizontal row containing a label + content.

```rust
div().flex().flex_col().gap_3()
    .child(section_title("Basic information"))
    .child(field_row("Name", Text::new("Zhang San")))
    .child(Divider)
    .child(caption("The information above is for display only"))
```

### theme · Unified Theme

A global semantic color system and the source of all control colors.

- **21 semantic colors**: `bg()` / `surface()` / `accent()` / `text()` / `text_secondary()` / `text_disabled()` / `border()` / `track()` / `success()` / `warning()` / `danger()` / `info()`, etc. (each returns a GPUI color).
- **Two built-in themes**: `Theme::light()` / `Theme::dark()`.
- **Configuration API**:
  - `theme::set_theme(Theme)`: switch between light and dark globally.
  - `theme::set_accent(color)`: change the brand accent (hover / pressed / soft backgrounds are **derived automatically**).
  - `theme::update(|t| { … })`: fine-tune an individual semantic color.
- **Caveat**: themes apply globally, and all controls use the new colors on their next redraw after a change. Custom controls should also use `theme::xxx()` instead of hard-coded colors so they follow the theme.

```rust
// Set the dark theme and brand blue at startup:
theme::set_theme(theme::Theme::dark());
theme::set_accent(gpui::rgb(0x3B82F6));
// Fine-tune the success color:
theme::update(|t| { t.success = gpui::rgb(0x22C55E); });
// Read colors during render:
div().bg(theme::bg()).text_color(theme::text())
```

### FpsMeter · Frame Rate / Performance Monitor

A high-performance monitor that measures actual rendering FPS and frame time; it is itself an `Entity`.

- **Creation**: `cx.new(|cx| FpsMeter::new(cx))`; enabled by default, with a nominal maximum of 120fps.
- **Properties**: `running(bool)`: toggle continuous refresh; `max_fps(u32)`: nominal maximum (default 120).
- **Methods**: `start(cx)` / `stop(cx)` / `is_running()` / `fps()` / `frame_ms()`.
- **Performance model**: when enabled, `Window::request_animation_frame` drives continuous rendering. That call only notifies the current view, so high-frequency refreshes redraw only this control's small subtree without redrawing the entire page; frame time is smoothed with a 60-frame ring buffer.
- **Frame-rate cap**: the actual frame rate follows the display's vsync (Windows reads the DWM composition interval); a 120Hz display can reach ~120fps. `max_fps` is only a nominal / color threshold and does not impose a hard frame-rate limit. All animations are time-based (delta-time) and remain smooth at high refresh rates.
- **Caveat**: automatically stops driving updates when the control is not in the render tree (for example, after switching to another category); no manual `stop` is needed.

```rust
let meter = cx.new(|cx| FpsMeter::new(cx).max_fps(120));
// During render:
div().child(meter.clone())
// Controls:
meter.update(cx, |m, cx| m.stop(cx));
```

---

## IX. AI Conversations

A conversation suite comparable to antd X: conversation switching, prompts, message bubbles, a composer, and thought chains.

### Welcome · Welcome Card

- Controlled, with no callbacks. `Welcome::new(title).icon(emoji).description(text)`.
- Layout: an icon on the left (fixed at 56px, non-shrinking) + a title/description column on the right (`flex_1` + `min_w(0)`); long descriptions wrap automatically without overflowing.

### Prompts · Prompt Card Group

- `Prompts::new(vec![PromptItem::new(text).icon(emoji)])`.
- Callback `on_click(&usize)` (3-argument family), returning the clicked index.

### Bubble · Message Bubble

- Controlled, with no callbacks. `Bubble::new(content).avatar(name)`; `right()` for the user side; `loading(true)` displays a Spinner; `variant(BubbleVariant::Filled/Outlined/Shadow)`.
- Limited to max_w(420) with min_w(0) + flex_shrink, so long text wraps without bursting the parent column.

### Sender · Composer (Entity)

- `cx.new(|cx| Sender::new(cx).on_send(|text, window, cx| …))`; the `on_send` event is `&SharedString` (3-argument family).
- Enter sends / Shift+Enter inserts a line break / the button sends; the input is cleared automatically after sending. Internally, TextArea's `on_submit` submission mode (action layer) is used rather than an outer `on_key_down`.

### Conversations · Conversation List

- `Conversations::new(vec![ConversationItem::new(key, title).icon(emoji)])`.
- `active(SharedString)` is controlled + `on_select(&SharedString)` (3-argument family). IDs use conversation-{ix}; do not collide with IDs from other lists.

### ThoughtChain · Thought Chain

- `ThoughtChain::new(vec![ThoughtNode::new(title).description(text).status(ThoughtStatus::Success/Pending/…)])`.
- `expanded(Vec<usize>)` is controlled + `on_toggle(&usize)` (3-argument family) expands / collapses.

---

## X. Advanced and Host Modules

All 8 modules below are public through `pub mod` / `pub use` in `src/lib.rs`. They are documented separately: **they are not folded into the 103 documented controls / 106 entries, and the total is not presented as 107**.

### HotkeyInput · Hotkey Recorder

- **Purpose**: Click to focus, then press a key combination. Produces both a human-readable display string and a canonical GPUI binding.
- **State model**: Stateful `Entity<HotkeyInput>`; stores the display parts, binding, modifier preview, recording state, focus, and appearance options.
- **Creation**: `HotkeyInput::new(cx: &mut Context<Self>) -> Self`, then create the entity with `cx.new(|cx| …)`.
- **Properties**: `id(impl Into<ElementId>)`; `placeholder(impl Into<SharedString>)`; `disabled(bool)`; `variant(InputVariant)`; `status(InputStatus)`; `size(InputSize)`; `require_modifier(bool)`; `allow_clear(bool)` (enabled by default).
- **Methods**: `value() -> String`; `binding() -> &str`; `keys() -> &[SharedString]`; `is_empty()`; `is_recording()`; `set_binding(&str, &mut Context<Self>)`; `clear(&mut Context<Self>)`; `start_recording(&mut Window, &mut Context<Self>)`; `stop_recording(&mut Context<Self>)`.
- **Callbacks**: `on_change(Fn(&str, &mut Window, &mut App))`; the first argument is the display string, such as `"Ctrl+Shift+K"`. It runs through `window.defer`. Programmatic `set_binding` / `clear` **do not trigger** this callback.
- **Minimal example**:

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

- **Caveats**: `value()` and `binding()` use different formats; register shortcuts with the latter. `Esc` cancels and preserves the previous value; Backspace / Delete clears the value and fires the callback. Losing focus ends recording. `require_modifier(true)` requires Ctrl / Alt / the platform key; a bare key or Shift alone does not qualify. No `bind_keys` call is needed. Use stable, unique ids for multiple instances in one window.

### DiffView · Line-level Code Diff

- **Purpose**: Compares two text snapshots and displays added, removed, and unchanged lines in unified or side-by-side form. Suitable for release notes, patch previews, and configuration review.
- **State model**: Stateless `RenderOnce`; the host passes old text, new text, and display mode on each render.
- **Creation**: `DiffView::new(old: impl Into<SharedString>, new: impl Into<SharedString>) -> Self`.
- **Properties**: `id(...)` sets a stable unique id; `title(...)` customizes the heading; `mode(DiffMode::{Split, Unified})` selects side-by-side or unified layout; `line_numbers(bool)` controls line numbers, enabled by default.
- **Pure helpers**: `diff_lines(&str, &str) -> Vec<DiffLine>` returns a shortest line-level edit script. Each `DiffLine` contains `DiffKind::{Equal, Added, Removed}`, one-based `old_line` / `new_line`, and text. `diff_stats(&[DiffLine]) -> DiffStats` counts additions and removals.
- **Algorithm**: Strips common prefixes and suffixes before running Myers' shortest edit path. Completely disjoint inputs produce a direct replacement, avoiding a needless quadratic trace. Comparison uses `str::lines()`; a trailing newline is not represented as a separate changed line.
- **Minimal example**:

```rust
use gpui_widgets::{DiffMode, DiffView, diff_lines, diff_stats};

let old = "let value = 1;\nprintln!(\"{value}\");";
let new = "let value = 2;\nprintln!(\"{value}\");";
let lines = diff_lines(old, new);
let stats = diff_stats(&lines); // added: 1, removed: 1

DiffView::new(old, new)
    .id("value-diff")
    .title("main.rs")
    .mode(DiffMode::Split)
```

- **Caveats**: Long lines are horizontally clipped inside their cells rather than wrapped. Colors follow the theme's success / danger tokens. The view renders every diff row; hosts should chunk or paginate very large files. Multiple instances in one window require unique ids.

### MarkdownEditor · Markdown Source Editor and Preview

- **Purpose**: Combines `TextArea` source editing with the existing `Markdown` renderer for live preview. It is not a WYSIWYG editor.
- **State model**: Controlled `RenderOnce`. The host owns the source string; id-scoped keyed state retains the `Entity<TextArea>`. User edits fire `on_change` and the host passes the updated value back. External text changes are synchronized silently without firing `on_change` in reverse.
- **Creation**: `MarkdownEditor::new(text: impl Into<SharedString>) -> Self`.
- **Properties**: `id(...)`; `title(...)`; `placeholder(...)`; `mode(MarkdownEditorMode::{Split, Editor, Preview})`; `height(Pixels)` sets the visible source and preview height.
- **Callbacks**: `on_change(Fn(&str, &mut Window, &mut App))` receives the complete source.
- **Prerequisite**: Call `text_area::bind_keys(cx)` at application startup for Enter, Tab / Shift+Tab, navigation, selection, clipboard, undo, and redo shortcuts.
- **Minimal example**:

```rust
use gpui_widgets::{MarkdownEditor, MarkdownEditorMode};

MarkdownEditor::new(self.markdown.clone())
    .id("article-editor")
    .mode(MarkdownEditorMode::Split)
    .on_change(cx.listener(|this, text: &str, _, cx| {
        this.markdown = text.to_string();
        cx.notify();
    }))
```

- **Caveats**: The initial caret starts at the beginning of the source. The source pane uses line numbers and a monospace font. Preview syntax exactly matches `Markdown`; images, raw HTML, footnotes, and other full-GFM features remain unsupported. Multiple editors in one window require unique ids.

### Markdown · Markdown Rendering

- **Purpose**: Renders a common GFM subset used in conversations / documents directly into GPUI elements, with clickable links and code-block actions.
- **State model**: Stateless `RenderOnce`; the host rebuilds it with the source text and options on each render. It stores no controlled value.
- **Creation**: `Markdown::new(source: impl Into<SharedString>) -> Self`.
- **Properties**: `compact()` selects compact chat-bubble metrics; `text_size(f32)` overrides the body size and scales headings / code proportionally; `code_action(CodeAction)` may be called repeatedly to append an action to every fenced code block; `code_copy_labels(copy, copied)` overrides the copy button's normal / copied labels.
- **Callbacks**: `on_link(Fn(&str, &mut Window, &mut App))` receives the URL. The handler in `CodeAction::new(label, Fn(&str, &mut Window, &mut App))` receives the code-block text.
- **Minimal example**:

```rust
use gpui_widgets::{CodeAction, Markdown};

Markdown::new("# Result\n\n**Done** — [docs](https://example.com)\n\n```rust\nfn main() {}\n```")
    .on_link(|url, _, _| println!("open: {url}"))
    .code_action(CodeAction::new("Insert", |code, _, _| println!("{code}")))
```

- **Supported syntax**: Heading levels 1~6, paragraphs, nested ordered / unordered lists, task lists, quotes, rules, table-column alignment, backtick / tilde fenced code blocks, bold / italic / strikethrough / inline code / links / bare-URL autolinks.
- **Caveats**: The handwritten, dependency-free parser covers only the subset above. It does not render images (`![alt](url)` produces clickable alt text), raw HTML, footnotes, or other full-GFM features; other unsupported content degrades to text. Task-list boxes are display-only and have no toggle callback. Use `compact()` / `text_size()` / `code_copy_labels()` to adapt narrow panels and host languages.

### TransferManager · Transfer Task Manager

- **Purpose**: File upload / download queue panel with overall progress, per-task progress, size / speed / ETA, and status actions.
- **State model**: Stateless `RenderOnce`; receives a `Vec<TransferTask>` snapshot for display during render (typically `self.tasks.clone()`). The host view owns the queue and state machine.
- **Creation**: `TransferManager::new(id: impl Into<ElementId>, tasks: Vec<TransferTask>) -> Self`; `TransferTask::new(id, name, TransferDirection)`, with `upload(id, name)` / `download(id, name)` shortcuts.
- **State types**: `TransferDirection::{Upload, Download}`; `TransferStatus::{Queued, Transferring, Paused, Done, Failed, Canceled}`, defaulting to `Queued`; `TransferStatus::is_active()` is true only for `Queued | Transferring`.
- **Properties / task methods**: Panel `title(…)` / `show_summary(bool)`; task builders `status(...)` / `progress(f32)` / `size(u64)` / `speed(f64)` / `error(...)`; getters `id()` / `name()` / `task_status()` / `progress_value()`; driver methods `set_status(...)` / `set_progress(...)`.
- **Callbacks**: `on_pause` / `on_resume` / `on_cancel` / `on_retry` / `on_remove` all use `Fn(&SharedString, &mut Window, &mut App)` and receive the task id; `on_clear_done(Fn(&mut Window, &mut App))` has no event argument.
- **Minimal example**:

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

- **Caveats**: `progress` is clamped to `0.0..=1.0`, but the component never changes status or advances tasks itself; update the host collection in callbacks and rerender. Overall progress is weighted by file size, treats missing sizes as 1, excludes `Canceled`, and always shows `Done` as 100%. Action buttons appear only for the matching status and only when their callback is registered; Clear Completed also requires at least one `Done` task and a registered callback. Use unique ids for multiple panels in one window.

### Terminal · Terminal View

- **Purpose**: A pure-GPUI VT/ANSI terminal view for hosts such as SSH / local shells. It neither spawns processes nor replaces a PTY / SSH channel.
- **State model**: Stateful `Entity<Terminal>`; contains the parser state machine, character grid, `history` scrollback, fixed `screen`, cursor / pen, IME composition, selection, search, and scroll offset.
- **Creation**: `Terminal::new(cx: &mut Context<Self>) -> Self`, then `cx.new(|cx| …)`. Defaults: 80×24, 14px font, 18px line height, 1000 scrollback lines, Bar cursor, blinking off, border on.
- **Public types**: `CursorShape::{Block, Bar, Underline}`; `TerminalScrollMetrics { history_lines, visible_rows, viewport_height: Pixels, max_scroll: Pixels }`.
- **Properties**: `id(...)`; `cols(usize)` / `rows(usize)`; `font_size(Pixels)` / `line_height(f32)`; `font_family(impl Into<SharedString>)` / `font_fallbacks(Vec<SharedString>)`; `bordered(bool)`; `max_history(usize)`; `cursor_style(CursorShape)` / `cursor_blink(bool)`; `fit_min(cols, rows)`. Corresponding runtime setters take `cx: &mut Context<Self>` and call `notify`.
- **Callbacks (plain `Fn` values, not GPUI event wrappers)**:
  - `on_input(Fn(&[u8], &mut Window, &mut App))`: bytes produced from keyboard input.
  - `on_title(Fn(&str, &mut Window, &mut App))`: OSC 0/2 title changes.
  - `on_fit(Fn(usize, usize, &mut Window, &mut App))`: `(cols, rows)` after a pane-fit grid change; local resize happens first and the callback is **not deferred**.
  - `on_search_open(Fn(&mut Window, &mut App))`: Ctrl+Shift+F asks the host to open search UI.
  - `on_search_results(Fn(usize, Option<usize>, &mut Window, &mut App))`: match count and zero-based current index.
  - `on_cwd(Fn(&str, &mut Window, &mut App))`: OSC 7 working directory.
  - `on_selection_change(Fn(Option<String>, &mut Window, &mut App))`: selected text, or `None` when cleared.
  - All except `on_fit` run through `window.defer`.
- **Methods**:
  - Output: `write(&[u8], cx)` / `write_str(&str, cx)` / `clear(cx)` / `reset(cx)` / `resize(cols, rows, cx)` / `paste_text(&str, window, cx)`.
  - Reads: `title()` / `cwd()` / `cursor_visible()` / `cursor_row()` / `screen_top()` / `selection_text()`.
  - Search: `set_search(&str, cx)` / `search_next(cx)` / `search_prev(cx)` / `search_matches()` / `search_index()`.
  - Selection / clipboard: `select_all(cx)` / `set_selection(anchor, head, cx)` / `copy_selection(&mut App)` / `copy_all(&mut App)`.
  - Buffer / scrolling: `buffer_text()` / `buffer_tail(max_lines)` / `scroll_handle()` / `scroll_offset()` / `scroll_metrics()` / `set_scroll_offset(lines, cx)` / `scroll_to_bottom(cx)`.
- **Minimal example**:

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

- **Caveats**: Keyboard handling uses raw `on_key_down`; `text_input::bind_keys` is not required. The host must write `on_input` bytes to its channel and feed channel output into `write`. `write` clears the selection and returns to the bottom. `resize` reflows soft / hard wraps and remaps the cursor, selection, and search. Search provides matching only; the host implements the search panel. `on_fit` requires a fixed-height flex column and makes the terminal fill the remaining height with `flex_1`; `fit_min` can reject transient small sizes while dragging. Paste normalizes LF / CRLF to CR and adds bracketed-paste wrappers when remote mode 2004 is active. Use stable, unique ids for multiple terminals in one window.

### CodeEditor · Lightweight Code Editor Preset

- **Purpose**: Adds lightweight line-level highlighting for common languages on top of `TextArea` code mode. It is not a full IDE editor.
- **State model**: `CodeEditor` is a stateless unit-type factory and **is not itself an Entity**. The returned `TextArea` is what gets wrapped in `Entity<TextArea>`.
- **Creation**: `CodeEditor::new(cx: &mut Context<TextArea>, language: CodeLanguage) -> TextArea`; enables line numbers, current-line highlighting, a monospace font, four-space indentation, a 13px font size, and the language highlighter by default.
- **Languages**: `CodeLanguage::{Plain, Rust, JavaScript, Json, Toml, Python, Shell}`; `label() -> &'static str`; `all() -> &'static [CodeLanguage]`; `from_name(&str) -> Option<CodeLanguage>` (case-insensitive, with aliases such as `rs` / `js` / `ts` / `jsonc` / `py` / `sh`); `highlighter() -> Option<Highlighter>`, returning `None` for `Plain`.
- **Properties / methods**: `CodeEditor::set_language(&mut TextArea, CodeLanguage, &mut Context<TextArea>)` changes the language at runtime. Continue using `TextArea` APIs on the returned value, including `height(Pixels)`, `placeholder(...)`, `on_change(...)`, `text()` / `set_text(...)`, `caret()`, scrolling, and selection.
- **Callbacks**: Adds none; inherits `TextArea::on_change(Fn(&str, &mut Window, &mut App))` and `on_submit(Fn(&mut Window, &mut App))`.
- **Minimal example**:

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

- **Caveats**: Call `text_area::bind_keys` for the complete editing shortcuts. Highlighting is only a per-line lexical approximation (keywords / types / literals / strings / numbers / line comments / function names); it does not infer multiline state or provide LSP, folding, completion, or diagnostics.

### CommandBar · Command Input Bar

- **Purpose**: Single-line command input with history navigation and snippet completion / management; four action buttons adapt it to a terminal-bottom command bar.
- **State model**: Stateful `Entity<CommandBar>`; wraps a bare `TextInput` and stores oldest-to-newest `history`, navigation index / draft, simple snippets, `CommandSnippet` entries, the open panel, and inline form. Focus is delegated to the input.
- **Creation**: `CommandBar::new(cx: &mut Context<Self>) -> Self`, then `cx.new(|cx| …)`.
- **Properties**:
  - Basic: `id(...)` / `prompt(...)` / `placeholder(...)` / `max_history(usize)` (minimum 1, default 100) / `history_items(IntoIterator<Item = impl Into<String>>)`.
  - Snippets: `snippets(IntoIterator<Item = impl Into<SharedString>>)` / `snippet_items(IntoIterator<Item = CommandSnippet>)` / `manage_snippets(bool)` / `dismiss_outside(bool)` / `empty_text(history, snippets)` / `snippet_icon_builder(Fn(SnippetIcon, &str) -> AnyElement)`.
  - Appearance: `framed(bool, &mut App)` (default `true`) / `actions([impl Into<SharedString>; 4])`, ordered Snippets / History / Clear / Execute.
- **Snippet types / helpers**: `CommandSnippet { id: usize, name, command, group, send_enter: bool, sort: usize }`; `new(name, command, group)` defaults to `send_enter = true`, `sort = 0`, with builders to override them. `SnippetIcon::{Edit, Trash}`; `SnippetEvent::{Saved(CommandSnippet), DeleteRequested(CommandSnippet), Warn(SharedString)}`. Helpers: `sorted_snippets` / `group_snippets` / `panel_groups` / `snippet_can_save`.
- **Callbacks**:
  - `on_submit(Fn(&str, &mut Window, &mut App))`: receives the trimmed command through defer.
  - `on_new_snippet(Fn(&mut Window, &mut App))`: the New button in non-managed mode.
  - `on_snippet_event(Fn(SnippetEvent, &mut Window, &mut App))`: the event is passed **by value**.
  - `on_history_clear(Fn(&mut App))`: no Window.
  - Each has a runtime `set_on_*` variant; the icon factory also has `set_snippet_icon_builder`.
- **Methods**: `text(&App)` / `set_text(..., cx)` / `set_placeholder(..., cx)` / `set_actions(..., cx)` / `set_empty_text(..., cx)`; `history()` / `clear_history(cx)` / `set_history(Vec<String>, cx)`; `snippet_count()` / `snippet_list()` / `push_snippet(...)` / `remove_snippet(id)` / `set_snippet_items(Vec<CommandSnippet>, cx)`; `show_snippets(cx)` / `open_snippet_new(cx)` / `open_snippet_edit(id, cx)` / `submit(window, cx)`.
- **Minimal example**:

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

- **Caveats**: Call `text_input::bind_keys` at startup. Enter deduplicates into history, clears the input, and fires the callback; ↑↓ navigate history and restore the draft at the newest end; Tab completes the first prefix match; Esc closes the panel, exits history navigation, then clears input when action buttons are present. Managed mode includes new / edit / delete-request UI; the host handles persistence, warnings, and delete confirmation. Clicking a snippet with `send_enter(true)` calls `on_submit` directly and **does not automatically add history or clear the current input**. `push_snippet` / `remove_snippet` / `open_snippet_*` take no `cx`; the host must ensure a subsequent `notify`.

### DropZone · External File Drop Area

- **Purpose**: Accepts files / folders dragged in from the OS and partitions them into accepted and rejected paths by extension.
- **State model**: Stateless `RenderOnce`; filter rules and callbacks are supplied on each render. It stores neither files nor upload progress.
- **Creation**: `DropZone::new(id: impl Into<ElementId>) -> Self`.
- **Properties**: `icon(...)` / `title(...)` / `hint(...)`; `accept<I, S>(I) where I: IntoIterator<Item = S>, S: Into<SharedString>`; `height(f32)` (default 132); `disabled(bool)`. Extensions are supplied without a dot, an empty list accepts everything, and matching is case-insensitive.
- **Helper**: `is_accepted(&Path, &[SharedString]) -> bool`.
- **Callbacks**: `on_drop(Fn(&[PathBuf], &mut Window, &mut App))` and `on_reject(Fn(&[PathBuf], &mut Window, &mut App))` receive accepted / rejected path slices respectively.
- **Minimal example**:

```rust
use gpui_widgets::DropZone;

DropZone::new("source-drop")
    .title("Drop Rust or TOML files")
    .accept(["rs", "toml"])
    .on_drop(|paths, _, _| { /* enqueue accepted paths */ })
    .on_reject(|paths, _, _| { /* show rejected paths */ })
```

- **Caveats**: It only handles `ExternalPaths` converted by GPUI from platform `FileDropEvent` values and does not read files itself. Each drop is partitioned by extension, so a mixed batch may invoke both callbacks. `can_drop` requires at least one accepted path, so **a fully rejected batch does not invoke `on_reject`**. Disabled mode registers no drag handling. Multiple areas in one window require unique ids.

### TitleBar · Custom Title Bar

- **Purpose**: Custom title bar with an icon, title, and Minimize / Maximize (Restore) / Close buttons.
- **State model**: Stateless `RenderOnce`; reads the current `Window` state and registers window interactions, but stores no title or button state.
- **Creation**: `TitleBar::new(title: impl Into<SharedString>) -> Self`; window-options helper: `transparent_titlebar_options(title) -> WindowOptions`.
- **Properties**: `id(...)`; `icon(...)` / `icon_element(impl IntoElement)` (the element takes precedence); `window_icons(Fn(WindowIconKind, SharedString) -> AnyElement)`; `close_hover_bg(Hsla)`; `height(impl Into<Pixels>)` (default 36); `button_width(impl Into<Pixels>)` (default 46); `show_minimize(bool)` / `show_maximize(bool)` / `show_close(bool)`; `draggable(bool)` / `live(bool)` (both default `true`); repeatable `extra(impl IntoElement)` / `extra_trailing(impl IntoElement)`.
- **Window icon type**: `WindowIconKind::{Minimize, Maximize, Restore, Close}`; the Maximize button switches between Maximize / Restore according to `window.is_maximized()`. The factory's second argument is the button group name and can be used with `group_hover`.
- **Callbacks**: `on_minimize` / `on_maximize` / `on_close` all use `Fn(&mut Window, &mut App)`. Registering one overrides that button's default behavior instead of delegating to the system.
- **Minimal example**:

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

- **Caveats**: Hide the system title bar by creating the window with `transparent_titlebar_options` (`appears_transparent: true`). On Windows, dragging / double-click / Snap Layouts / the system context menu use `WindowControlArea`; other platforms fall back to `Window` operations. Keep `window_decorations` and `is_movable` at their defaults or Windows will not take over dragging. `extra` is inside the drag area but punches a no-drag hole; `extra_trailing` is outside it. `live(false)` is display-only and registers no hit areas or button callbacks. Use unique ids for multiple title bars in one window.

---

## Appendix A: Callback Signature Quick Reference

> Families: **3** = `Fn(&E, &mut Window, &mut App)` (use `cx.listener`); **2** = `Fn(&mut Window, &mut App)` (use a `cx.entity()` handle); **1** = `Fn(&mut App)`; **Special** = asynchronous special case.

| Control | Callback | Event Type E | Family |
|---|---|---|---|
| Button | `on_click` | `&ClickEvent` | 3 |
| Link | `on_click` | — (no event argument) | 2 |
| Anchor | `on_click` | `&SharedString` | 3 |
| Breadcrumb | `on_click` | `&usize` | 3 |
| MenuItem (DropdownMenu/ContextMenu) | `on_click` | — | 2 |
| MenuBar | `on_open_change` | `&Option<usize>` | 3 |
| MenuBar | `on_select` | `usize` + `&SharedString` | Special |
| MenuBarItem | `on_click` | — | 2 |
| Toolbar | `on_action` | `&SharedString` | 3 |
| ToolbarItem | `on_click` | — | 2 |
| NavMenu | `on_select` / `on_toggle` | `&SharedString` | 3 |
| Pagination | `on_change` | `&usize` | 3 |
| Tabs | `on_select` / `on_close` | `&usize` | 3 |
| Tabs | `on_add` | — | 2 |
| FloatButton | `on_click` | — | 2 |
| CommandPalette | `on_select` | `&SharedString` | 3 |
| Splitter | `on_change` | `&f32` | 3 |
| SearchInput | `on_change` / `on_search` | `&str` | 3 |
| OtpInput | `on_change` / `on_complete` | `&str` | 3 |
| NumberInput | `on_change` | `Option<f64>` (**by value**) | 3 |
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
| FilePicker | `on_pick` | `&Vec<PathBuf>` (no Window) | Special |
| Table | `on_row_click` / `on_page_change` / `on_expand` | `&usize` | 3 |
| Table | `on_sort` | `&(usize, bool)` | 3 |
| Table | `on_selected_rows` | `&Vec<usize>` | 3 |
| Tree | `on_toggle` / `on_select` | `&SharedString` | 3 |
| Tree | `on_check` | `&Vec<SharedString>` | 3 |
| Tree | `on_drop_node` | `&TreeDrop` | 3 |
| TreeTable | `on_toggle` / `on_select` | `&SharedString` | 3 |
| TreeTable | `on_check` | `&Vec<SharedString>` | 3 |
| Countdown | `on_finish` | — (no Window) | 1 |
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
| ToastStack / NotificationStack | `on_dismiss` | `&ToastDismissed` / `&NotificationDismissed` | 3 |
| ToastStack / NotificationStack | `on_close` (legacy) | `&usize` | 3 |
| ToastHost / NotificationHost | `on_dismiss` | `&ToastDismissed` / `&NotificationDismissed` (no Window) | 2 |
| NotificationCenter | `on_select` / `on_read` / `on_remove` | `&usize` | 3 |
| NotificationCenter | `on_mark_all_read` / `on_dismiss` | — | 2 |
| Popconfirm | `on_open_change` | `&bool` | 3 |
| Popconfirm | `on_confirm` / `on_cancel` | — | 2 |
| HotkeyInput | `on_change` | `&str` | 3 |
| Markdown | `on_link` | `&str` | 3 |
| Markdown | `code_action(CodeAction)` handler | `&str` | 3 |
| TransferManager | `on_pause` / `on_resume` / `on_cancel` / `on_retry` / `on_remove` | `&SharedString` | 3 |
| TransferManager | `on_clear_done` | — | 2 |
| Terminal | `on_input` | `&[u8]` | 3 |
| Terminal | `on_title` / `on_cwd` | `&str` | 3 |
| Terminal | `on_fit` | `(usize, usize)` (**by value**) | 3 |
| Terminal | `on_search_open` | — | 2 |
| Terminal | `on_search_results` | `(usize, Option<usize>)` (**by value**) | 3 |
| Terminal | `on_selection_change` | `Option<String>` (**by value**) | 3 |
| CodeEditor (returns TextArea) | `on_change` | `&str` | 3 |
| CodeEditor (returns TextArea) | `on_submit` | — | 2 |
| CommandBar | `on_submit` | `&str` | 3 |
| CommandBar | `on_snippet_event` | `SnippetEvent` (**by value**) | 3 |
| CommandBar | `on_new_snippet` | — | 2 |
| CommandBar | `on_history_clear` | — (no Window) | 1 |
| CommandBar | `snippet_icon_builder` | `(SnippetIcon, &str) -> AnyElement` | Factory |
| DropZone | `on_drop` / `on_reject` | `&[PathBuf]` | 3 |
| TitleBar | `on_minimize` / `on_maximize` / `on_close` | — | 2 |

## Appendix B: Entity Control List

These controls are **created once with `cx.new(|cx| T::new(cx))`**, attached with `.clone()` during render, and read or written through `entity.read(cx)` / `entity.update(cx, …)`:

| Control | Creation | Primary Read / Write Methods |
|---|---|---|
| TextInput | `TextInput::new(cx)` | `text()` / `set_text(s, window, cx)` |
| TextArea | `TextArea::new(cx)` | Same as TextInput |
| PasswordInput | `PasswordInput::new(cx)` | Same as TextInput |
| OtpInput | `OtpInput::new(cx, length)` | `value()` / `set_value(s, cx)` / `clear(cx)` / `is_complete()` |
| SearchInput | `SearchInput::new(cx)` | Same as TextInput |
| Autocomplete | `Autocomplete::new(cx, items)` | Same as TextInput |
| Mentions | `Mentions::new(cx)` | Same as TextInput |
| NumberInput | `NumberInput::new(cx)` | `value()` / `set_value()` |
| TagInput | `TagInput::new(cx)` | `tags()` |
| CommandPalette | `CommandPalette::new(cx, items)` | `open(window, cx)` / `close(cx)` |
| VideoPlayer (Windows only) | `VideoPlayer::new(cx, path)` | `play/pause/toggle/seek`, `position()/duration()/is_playing()` |
| Countdown | `Countdown::new(unix, cx)` / `after(Duration, cx)` | `remaining()` / `finished()` |
| ToastHost | `ToastHost::new(cx)` | `show` / `show_with` / `dismiss` / `clear` / `len` / `is_empty` |
| NotificationHost | `NotificationHost::new(cx)` | `show` / `show_with` / `dismiss` / `clear` / `len` / `is_empty` |
| Sender | `Sender::new(cx).on_send(…)` | Internal TextArea; `on_send` returns the text |
| FpsMeter | `FpsMeter::new(cx)` | `fps()` / `frame_ms()` / `start/stop` |
| HotkeyInput | `HotkeyInput::new(cx)` | `value()` / `binding()` / `set_binding` / `clear` / recording controls |
| Terminal | `Terminal::new(cx)` | `write` / `resize` / search / selection / scrolling / buffer reads |
| CommandBar | `CommandBar::new(cx)` | `text` / `set_text` / history / snippets / `submit` |
| CodeEditor (factory returning TextArea) | `cx.new(\|cx\| CodeEditor::new(cx, language))` | Entity type is `Entity<TextArea>`; use `CodeEditor::set_language` and TextArea APIs |

> `Markdown` / `TransferManager` / `DropZone` / `TitleBar` are `RenderOnce` values. `CodeEditor` itself is not an Entity; only the `TextArea` it returns needs to be wrapped in one.

## Appendix C: Controls Requiring Unique IDs

Multiple instances in the same window must have unique IDs (hover / drag / keyed state is determined by `ElementId`):

`Slider` `Splitter` `Scrollbar` `Affix` `StatusBar` `Toolbar` `Select` `MultiSelect` `Cascader` `TreeSelect` `DatePicker` `DateRangePicker` `TimePicker` `ColorPicker` `Transfer` `UploadList` `Table` `TreeTable` `Tree` `Calendar` `Carousel` `Accordion` `Popover` `Popconfirm` `ContextMenu` `DropdownMenu` `MenuBar` `NavMenu` `Pagination` `FloatButton` `Rating` `Stepper` `OtpInput` `NotificationCenter` `HotkeyInput` `TransferManager` `Terminal` `CommandBar` `DropZone` `TitleBar`

> `Upload::new()` uses the default ID `"upload-area"`. Multiple Upload instances must be distinguished with `.id(…)`, or their hover state will interfere.

## Appendix D: Common Pitfalls FAQ

**Q: The input accepts typing, but Ctrl+C/V, Select All, and arrow keys do not work?**
A: You forgot to call `text_input::bind_keys(cx)` at startup. TextArea additionally requires `text_area::bind_keys(cx)`, and CommandPalette requires `command_palette::bind_keys(cx)`.

**Q: The `cx.listener` closure reports E0282 (type inference failed)?**
A: Explicitly annotate the event parameter, for example: `cx.listener(|this, _e: &ClickEvent, _, cx| …)`.

**Q: Using `cx.listener` with `on_close` / `on_confirm` fails to compile?**
A: They belong to the **2-argument family** (no event argument). Use a `cx.entity()` handle + `entity.update(cx, …)` instead. Countdown.on_finish takes 1 argument, and FilePicker.on_pick has no Window, so the same approach applies.

**Q: Do multiple Slider / Select hover highlights interfere with one another?**
A: Their IDs are duplicated. Use a unique ID for every instance in the same window (build it from the index / business key in lists).

**Q: Are Modal / Drawer clipped by the scroll container or positioned incorrectly?**
A: They must be rendered **outside the scroll container** (as the last child of the root view). The same applies to ToastStack / ToastHost / NotificationStack / NotificationHost / NotificationCenter / Tour / CommandPalette.

**Q: Does the Scrollbar thumb not move / not respond to the scroll wheel?**
A: The `ScrollHandle` is not persisted across frames, or the scroll container lacks `.track_scroll(&handle)`. Both must share the same handle (store it in a view field or `window.use_keyed_state`).

**Q: Does Affix fail to pin, or does its position jump?**
A: Make sure Affix and the scroll container use the same persistent `ScrollHandle`, and that the scroll container has `.id(...)`, `.overflow_y_scroll()`, and `.track_scroll(&handle)`. Keep Affix's placeholder in the original document flow; do not wrap it in another container that clips or changes positioning.

**Q: Does `TextInput::new(cx)` fail to compile in render?**
A: Its `cx` is a `Context<TextInput>`, so it can only be called inside a `cx.new(|cx| …)` closure; you cannot call `new` directly in the parent view's render / partition methods.

**Q: What type should the NumberInput `on_change` closure parameter use?**
A: The first argument is `Option<f64>` **by value** (not by reference): `cx.listener(|this, v: Option<f64>, _, cx| …)`.

**Q: Does `if … { px(0.) } else { 0. }` in a custom-drawn Element report E0308?**
A: The if/else branch types must match. `Pixels / Pixels = f32`, so both branches should use `f32` (`else { 0. }`); do not mix `Pixels` and `f32`.

**Q: Is Chinese content in QRCode cut off?**
A: The capacity limit is 134 bytes (approximately 44 UTF-8 Chinese characters), and longer content is truncated. It is suitable for short text / URLs.

**Q: Do expandable rows stop working after enabling `fixed_left()` on Table?**
A: `fixed_left()` / `fixed_right_columns(n)` and `row_detail()` are mutually exclusive; if both are set, expandable rows are disabled.

**Q: Does Calendar month navigation / view switching do nothing?**
A: Calendar is fully controlled. Handle and write back state for all three callbacks: `on_month_change` / `on_select` / `on_mode_change`.

**Q: Does moving / checking items in Transfer have no effect?**
A: Both `target_keys` (right panel) and `selected` (checked items) must be written back as controlled state (`on_change` / `on_select`).

**Q: Does `ImageView::new("d:/a.png")` fail to load the image?**
A: `new` uses the GPUI asset system (Embedded), not disk paths. Use `ImageView::from_file(path)` for local files.

**Q: Does VideoPlayer fail to compile on non-Windows platforms?**
A: The control is guarded by `#[cfg(windows)]` and only compiles on Windows; it is automatically excluded on other platforms without affecting the remaining controls.

**Q: Does Select month navigation / DatePicker month switching not work?**
A: The displayed month of DatePicker / DateRangePicker is controlled. You must handle `on_month_change` and write back `month`, or month navigation will not work.

---

> End of reference. See [`README.en.md`](../README.en.md) for a quick control category overview, and [`integration-guide.en.md`](integration-guide.en.md) for project integration and complete runnable examples.
