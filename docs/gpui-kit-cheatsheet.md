# gpui-kit 0.9.1 cheat-sheet (feature `gpui-fast`)

Built from the vendored source, not memory. `R` = `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f`.
Source tags: **[kit]** `R/gpui-kit-0.9.1`, **[comp]** `R/gpui-component-0.9.1/src`, **[base]** `R/gpui-base-0.9.1/src`,
**[fast]** `R/gpui-fast-0.1.15/src`, **[macros]** `R/gpui-fast-macros-0.1.15/src`, **[assets]** `R/gpui-kit-assets-0.9.1`.
Anything not verified in source is marked **UNCONFIRMED**. Snippets are adapted from the crates' own tests; none were compiled here.

## 0. Imports and name clashes (read first)

```rust
use gpui_kit::*;                              // GPUI core: div, px, rgb, Render, Window, Context, App, Entity, ...
use gpui_kit::prelude::FluentBuilder as _;    // .when / .when_some / .map  -- NOT in the root glob
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, Selectable as _, StyledExt as _, WindowExt as _,
    Icon, IconName, Theme, ThemeMode, TitleBar, h_flex, v_flex,
    button::{Button, ButtonVariants as _},
};
```

- `FluentBuilder` is exported only from `gpui_kit::prelude` ([fast] `prelude.rs`, `gpui.rs:167` re-exports only `FutureExt, Timeout` from `util`). Kit tests use `use gpui_kit::{..., prelude::*}`.
- Never glob-import a component module next to `use gpui_kit::*`. Clashes found in source:
  - `gpui_kit::InputEvent` is a GPUI core **trait**; `gpui_kit::component::input::InputEvent` is the input **enum** (kit tests write `use gpui_kit::InputEvent as _`).
  - `component::input` also exports `Point`, `Selection`, `Copy`, `Cut`, `Paste`, `Search`, `Delete`, `Enter`, `Position` ([comp] `input/mod.rs`).
  - `gpui_kit::Size<T>` (geometry struct) vs `gpui_kit::component::Size` (`XSmall|Small|Medium|Large|Size(Pixels)`, [comp] `sizing.rs`).
  - `gpui_kit::Result` is `anyhow::Result` ([fast] `gpui.rs:120`) and shadows std's `Result` under the glob.
- With `gpui-fast`, `gpui_kit::test` (the attribute macro) is re-exported **unconditionally** ([kit] `src/lib.rs`, the `#[cfg(feature = "gpui-fast")] pub use gpui_macros::{..., test, ...}` block), so `use gpui_kit::*` (and a test module's `use super::*`) shadows Rust's `#[test]`. Import explicitly in test modules.
- App code may write `gpui_kit::component::...` only; the kit's own tests sometimes say `gpui_component::` because they are inside the workspace.

## 1. Window setup

[kit] `src/lib.rs`:

```rust
pub fn open_window<V: Render>(
    options: WindowOptions, cx: &mut App,
    build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
) -> Result<(AnyWindowHandle, Entity<V>)>
pub fn init(cx: &mut App)            // = gpui_component::init (theme, root plugin, input, dialogs, ...)
pub use platform::application;       // application().with_assets(assets::Assets).run(|cx| { ... })
```

- `open_window` calls `cx.open_window` and wraps your view: `cx.new(|cx| base::Root::new(view, window, cx))`. The builder returns your content entity, never a `Root`.
- `Root` **is required** for dialogs, sheets, notifications, tooltips and focused-input tracking: `WindowExt` goes through `WindowState::entity`, which does `window.root::<gpui_base::Root>()` and panics with `"component window state is missing; call gpui_component::init before gpui_kit::open_window"` ([comp] `root.rs`). The plugin factories are captured when the Root is created, so `init(cx)` must run **before** `open_window`.
- Root also sets per frame: `window.set_rem_size(cx.theme().font_size)`, and styles the surface with theme `font_family`, `background`, `foreground` ([comp] `root.rs`, `RootPlugin for WindowState`). You do not need to paint a window background yourself.
- Do not use raw `cx.open_window(...)` unless you build the `Root` yourself.

`WindowOptions` ([fast] `platform.rs:2620`), fields that matter (all others via `..Default::default()`):

| Field | Type | Default |
| --- | --- | --- |
| `window_bounds` | `Option<WindowBounds>` (`Windowed(Bounds<Pixels>)`, `Maximized(..)`, `Fullscreen(..)`) | `None` |
| `titlebar` | `Option<TitlebarOptions { title: Option<SharedString>, appears_transparent: bool, traffic_light_position: Option<Point<Pixels>> }>` | `Some(default)` |
| `window_min_size` | `Option<Size<Pixels>>` | `None` |
| `is_resizable`, `is_minimizable`, `is_movable`, `focus`, `show` | `bool` | `true` |
| `app_id` | `Option<String>` (desktop grouping on Linux) | `None` |
| `window_decorations` | `Option<WindowDecorations>` (`Server` / `Client`, X11+Wayland) | `None` |
| `kind` | `WindowKind` | `Normal` |
| `icon` | `Option<Arc<image::RgbaImage>>` (X11 only) | `None` |

Helpers: `WindowBounds::centered(size: Size<Pixels>, cx: &App) -> WindowBounds` (`platform.rs:2805`), `Bounds::centered(display_id: Option<DisplayId>, size, cx)`, `Bounds::maximized(None, cx)`, `size(px(w), px(h))`.

```rust
fn main() {
    application().with_assets(assets::Assets).run(|cx| {
        init(cx);
        Theme::change(ThemeMode::Dark, None, cx);                    // section 2
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1280.), px(800.)), cx)),
            window_min_size: Some(size(px(960.), px(600.))),
            titlebar: Some(TitlebarOptions { title: Some("Game Planner".into()), ..Default::default() }),
            app_id: Some("art.bangmarcel.gameplanner".into()),
            ..Default::default()
        };
        open_window(options, cx, |_window, cx| cx.new(|_| HelloWorld)).expect("failed to open window");
    });
}
```

- Runtime title: `window.set_window_title(title: &str)` ([fast] `window.rs:3305`). Other `Window` methods: `viewport_size()`, `bounds()`, `refresh()`, `remove_window()`, `minimize_window()`, `zoom_window()`, `toggle_fullscreen()`, `activate_window()`.
- `TitlebarOptions` derives `Default` ([fast] `platform.rs:2842`), so the `..Default::default()` spread above is valid.
- Quit: `QuitMode::Default` = quit on last window closed on non-macOS ([fast] `app.rs:527`); override with `application().with_quit_mode(QuitMode::Explicit)`. `cx.quit()`; `cx.on_window_closed(|cx: &mut App, id: WindowId| ..) -> Subscription`.

## 2. Theme

Import: `gpui_kit::component::{Theme, ThemeMode, ThemeColor, ActiveTheme}` ([comp] `theme/mod.rs`, `theme/theme_color.rs`).

```rust
impl Theme {
    pub fn global(cx: &App) -> &Theme
    pub fn global_mut(cx: &mut App) -> &mut Theme     // raw: does NOT sync tokens/Base or refresh windows
    pub fn update<R>(cx: &mut App, edit: impl FnOnce(&mut Theme) -> R) -> R   // preferred write path
    pub fn change(mode: impl Into<ThemeMode>, _window: Option<&mut Window>, cx: &mut App)
    pub fn sync_system_appearance(window: Option<&mut Window>, cx: &mut App)
    pub fn sync_base(cx: &mut App)                    // call after global_mut edits, then cx.refresh_windows()
    pub fn is_dark(&self) -> bool
    pub fn input_background(&self) -> Hsla
}
pub enum ThemeMode { Light, Dark }                    // #[default] Light
pub trait ActiveTheme { fn theme(&self) -> &Theme; } // impl for App; Context<T> derefs to App
```

- `init(cx)` forces **Light** (`Theme::change(ThemeMode::Light, None, cx)` in `theme::init`). Nothing found in the component crate that follows the OS appearance automatically.
- Force dark, then override (two separate calls: changing `mode` reloads the mode's registered theme and overwrites colours edited in the same closure):

```rust
Theme::change(ThemeMode::Dark, None, cx);
Theme::update(cx, |theme| {
    theme.colors.background = rgb(0x16181d).into();   // Rgba -> Hsla
    theme.primary = rgb(0x4f8cff).into();             // Theme: DerefMut<Target = ThemeColor>
    theme.radius = px(6.);
    theme.radius_lg = px(10.);
    theme.font_family = "Inter".into();
    theme.font_size = px(14.);
});
```
(Adapted from [comp] `theme/mod.rs` `update_tests` and [kit] `tests/rendering.rs`, `tests/integration/text_selection.rs:409`.)

- `Theme` and `ThemeColor` are `#[non_exhaustive]`: assign fields, no struct literals. `ThemeColor::dark()` / `ThemeColor::light()` return `Arc<ThemeColor>`; `theme.colors = *ThemeColor::dark();` replaces the palette.
- `Theme` non-colour fields: `colors: ThemeColor`, `tokens: ThemeTokens` (same names, `ThemeToken { color: Hsla, background: Background }`, may carry gradients), `mode`, `font_family: SharedString` (default `.SystemUIFont`), `font_size: Pixels` (16; becomes the window rem size), `mono_font_family`, `mono_font_size` (13), `radius`, `radius_lg`, `window_radius`, `shadow: bool`, `focus_ring: bool`, `transparent: Hsla`, `scrollbar_mode`, `notification`, `list`, `sheet`, `motion`, `cursor_blink`, `light_theme`/`dark_theme: Rc<ThemeConfig>`, `highlight_theme`.
- Reading in render: `cx.theme().border`, `cx.theme().muted_foreground`, `cx.theme().radius` (needs `ActiveTheme` in scope; works with `&App` and `&Context<T>`). Gradient-capable fills: `cx.theme().tokens.background`.
- Because of rem sizing, `text_sm`, `gap_2`, `p_4` etc. scale with `theme.font_size` (rem = font_size). Use `px(..)` for fixed pixels.

Full `ThemeColor` field list (all `Hsla`, [comp] `theme/theme_color.rs`):

```
accent accent_foreground accordion background border
button button_active button_foreground button_hover
button_danger button_danger_active button_danger_foreground button_danger_hover
button_info button_info_active button_info_foreground button_info_hover
button_primary button_primary_active button_primary_foreground button_primary_hover
button_secondary button_secondary_active button_secondary_foreground button_secondary_hover
button_success button_success_active button_success_foreground button_success_hover
button_warning button_warning_active button_warning_foreground button_warning_hover
group_box group_box_foreground caret
chart_1 chart_2 chart_3 chart_4 chart_5 chart_bullish chart_bearish chart_grid
danger danger_active danger_foreground danger_hover
description_list_label description_list_label_foreground drag_border drop_target foreground
info info_active info_foreground info_hover input
link link_active link_hover
list list_active list_active_border list_even list_head list_hover
muted muted_foreground popover popover_foreground
primary primary_active primary_foreground primary_hover
progress_bar ring scrollbar scrollbar_thumb scrollbar_thumb_hover
secondary secondary_active secondary_foreground secondary_hover selection
sidebar sidebar_accent sidebar_accent_foreground sidebar_border sidebar_foreground
sidebar_primary sidebar_primary_foreground
skeleton slider_bar slider_thumb
success success_foreground success_hover success_active
switch switch_thumb
tab tab_active tab_active_foreground tab_bar tab_bar_segmented tab_foreground
table table_active table_active_border table_even table_head table_head_foreground
table_foot table_foot_foreground table_hover table_row_border
title_bar title_bar_border status_bar status_bar_border
warning warning_active warning_hover warning_foreground
overlay window_border
red red_light green green_light blue blue_light yellow yellow_light
magenta magenta_light cyan cyan_light
```
`input` is the input/select **border** colour; `button_*` variants fall back to the plain semantic colour per the doc comments.

## 3. Element basics

All from `use gpui_kit::*` ([fast] `styled.rs`, `elements/div.rs`, [macros] `styles.rs`).

**Units / colours** ([fast] `geometry.rs`, `color.rs`): `px(f32) -> Pixels`, `rems(f32)`, `relative(f32) -> DefiniteLength` (fraction), `auto()`, `size(w, h)`, `point(x, y)`; `rgb(0xRRGGBB) -> Rgba`, `rgba(0xRRGGBBAA) -> Rgba`, `hsla(h, s, l, a) -> Hsla` (all 0..1), `black()`, `white()`, `transparent_black()`; `Hsla::opacity(f32)`, `Hsla::alpha(f32)`. `Rgba: Into<Hsla>`, both `Into<Fill>` and `Into<Background>`.

**Layout** (`Styled` trait):
- Display: `flex()`, `grid()`, `block()`, `hidden()`; visibility `visible()`, `invisible()`.
- Flex: `flex_row()`, `flex_col()`, `flex_wrap()`, `flex_nowrap()`, `flex_1()`, `flex_auto()`, `flex_none()`, `flex_grow(grow: f32)`, `flex_grow_0()`, `flex_grow_1()`, `flex_shrink(f32)`, `flex_shrink_0()`, `flex_shrink_1()`, `flex_basis(impl Into<Length>)`.
- Align: `items_start|end|center|baseline|stretch()`, `justify_start|end|center|between|around|evenly()`, `self_start|end|center|stretch()`, `content_*()`.
- **CSS grid is available**: `grid()`, `grid_cols(cols: u16)`, `grid_rows(rows: u16)`, `grid_cols_min_content(u16)`, `grid_cols_max_content(u16)`, `col_span(u16)`, `col_span_full()`, `row_span(u16)`, `col_start(i16)`, `col_end(i16)`, `row_start(i16)`, `row_end(i16)`. No auto-fill/minmax template helper was found — responsive card grids need `flex().flex_wrap()` with fixed-width children, or a column count computed from `window.viewport_size()`.
- Box (generated): prefixes `w h size min_w min_h min_size max_w max_h max_size gap gap_x gap_y`, `p pt pb pl pr px py`, `m mt mb ml mr mx my`, `inset top bottom left right`. Each has a custom setter `prefix(length)` (e.g. `.w(px(240.))`, `.gap(px(12.))`, `.p(px(16.))`, `.size(px(32.))`) and suffixed presets `_0 _0p5 _1 _1p5 _2 _2p5 _3 _3p5 _4 _5 _6 _7 _8 _9 _10 _11 _12 _16 _20 _24 _32 _40 _48 _56 _64 _72 _80 _96 _112 _128 _px _full _auto _1_2 _1_3 _2_3 _1_4 _3_4 ...` (`_4` = 1rem). `size_full()`, `w_full()`, `h_full()`.
- Position: `relative()`, `absolute()` + `top/left/right/bottom/inset`; `aspect_ratio(f32)`, `aspect_square()`.

**Paint**:
- `bg(fill: impl Into<Fill>)` — `.bg(rgb(0x1e2128))`, `.bg(cx.theme().muted)`, `.bg(linear_gradient(..))`.
- Borders: `border_1()` (also `border_0 _2 _3 _4 ...`, sides `border_t_1 border_b_1 border_l_1 border_r_1 border_x_1 border_y_1`), `border_color(impl Into<Hsla>)`, `border_dashed()`. Custom width: `.border(px(1.5))` and `border_t/b/l/r/x/y(impl Into<AbsoluteLength>)` ([macros] `styles.rs:340`).
- Radius: `rounded(px(8.))`, presets `rounded_none xs sm md lg xl 2xl 3xl full` and per-side `rounded_t/b/l/r/tl/tr/bl/br`.
- Shadows: `shadow_2xs xs sm md lg xl 2xl()`, `shadow_none()`, `shadow(Vec<BoxShadow>)`.
- `opacity(f32)`, `overflow_hidden()`, `overflow_x_hidden()`, `overflow_y_hidden()`.
- Cursor: `cursor_pointer()`, `cursor_default()`, `cursor_text()`, `cursor_not_allowed()`, `cursor_grab()`, ...
- Gradient ([fast] `color.rs:865`), **two stops only**: `linear_gradient(angle: f32 /*deg, 0 = to top, clockwise*/, from: impl Into<LinearColorStop>, to: impl Into<LinearColorStop>) -> Background`; `linear_color_stop(color: impl Into<Hsla>, percentage: f32 /*0..1*/)`. No `From<Hsla> for LinearColorStop`; always use `linear_color_stop`.

```rust
div().bg(linear_gradient(135., linear_color_stop(rgb(0x2a2f3a), 0.), linear_color_stop(rgb(0x16181d), 1.)))
```

**Text**: `text_color(impl Into<Hsla>)`, `text_size(impl Into<AbsoluteLength>)` (`.text_size(px(13.))`), presets `text_xs sm base lg xl 2xl 3xl`, `font_weight(FontWeight::SEMIBOLD)` (consts `THIN EXTRA_LIGHT LIGHT NORMAL MEDIUM SEMIBOLD BOLD EXTRA_BOLD BLACK`; there is **no** `font_bold()` helper), `font_family(impl Into<SharedString>)`, `italic()`, `underline()`, `line_through()`, `line_height(impl Into<DefiniteLength>)`, `text_left/center/right()`, `truncate()`, `text_ellipsis()`, `line_clamp(usize)`, `whitespace_nowrap()`. Strings (`&'static str`, `String`, `SharedString`) are children directly.

**Children / conditionals**: `ParentElement::child(impl IntoElement)`, `children(impl IntoIterator<Item = impl IntoElement>)`; `Entity<V: Render>` is a child via `.child(entity.clone())`. `FluentBuilder` ([fast] `util.rs`): `when(cond, |el| ..)`, `when_else(cond, then, else)`, `when_some(opt, |el, v| ..)`, `when_none(&opt, |el| ..)`, `map(|el| ..)`.

**Interactivity** — `InteractiveElement` (no id needed): `id(impl Into<ElementId>) -> Stateful<Self>`, `hover(|style| style.bg(..))`, `group("name")` + `group_hover("name", |s| ..)`, `on_mouse_down(MouseButton::Left, |ev, window, cx| ..)`, `on_mouse_move`, `on_scroll_wheel`, `on_key_down`, `on_action`, `key_context`, `track_focus(&FocusHandle)`, `occlude()`.
`StatefulInteractiveElement` (**requires `.id(..)` first**, [fast] `div.rs:1354`): `on_click`, `active(|s| ..)`, `on_hover`, `tooltip`, `overflow_scroll()`, `overflow_x_scroll()`, `overflow_y_scroll()`, `track_scroll(&ScrollHandle)`, `on_drag`, `focusable()`.

```rust
fn on_click(self, listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self
fn tooltip(self, build_tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self
```

```rust
div()
    .id(("item-card", ix))                       // ElementId: &'static str, String, SharedString, usize, u64,
    .cursor_pointer()                            //   (&'static str, usize), (SharedString, usize), Uuid, ...
    .p_3().rounded(px(8.)).border_1().border_color(cx.theme().border)
    .hover(|s| s.bg(cx.theme().accent))
    .when(selected, |el| el.border_color(cx.theme().primary))
    .tooltip(|window, cx| gpui_kit::component::tooltip::Tooltip::new("Open item").build(window, cx))
    .on_click(cx.listener(|this, _ev: &ClickEvent, _window, cx| { this.selected = true; cx.notify(); }))
    .child("Iron Plate")
```
(Pattern from [kit] `tests/integration/window.rs`. `cx.theme()` inside the `hover` closure borrows `cx`; copy the colour to a local first if the borrow checker objects.) `From<Uuid> for ElementId` exists ([fast] `window.rs:8090`), uuid 1.x.

**Scrolling**
- Raw GPUI: `div().id("list").overflow_y_scroll()` — needs `.id()`, no visible scrollbar. The element needs a bounded height (`h(..)`, `max_h(..)`, or `flex_1().min_h_0()` in a column; the kit's tests use `flex_1().overflow_hidden()` parents).
- With a scrollbar ([comp] `scroll/scrollable.rs`): `use gpui_kit::component::scroll::ScrollableElement as _;` then `.overflow_y_scrollbar()` / `.overflow_x_scrollbar()` / `.overflow_scrollbar()` on a `Div` or `Stateful<_>`; returns `Scrollable<Self>` (still `Styled + ParentElement + InteractiveElement`). **No `.id()` needed** — the scroll state is keyed by call site (`#[track_caller]`); when one call site creates several scrollables (loop/helper fn) give each `.id(..)` via `Scrollable::id`.

```rust
v_flex().size_full().overflow_y_scrollbar().children(rows)      // [comp] scrollable.rs tests
```
- Manual: `ScrollHandle::new()`, `.track_scroll(&handle)`, then `.vertical_scrollbar(&handle)`; handle has `offset()`, `set_offset(point)`, `scroll_to_item(ix)`, `scroll_to_bottom()`.
- Long uniform lists: `uniform_list(id, item_count, |range, window, cx| -> Vec<impl IntoElement>)` ([fast] `elements/uniform_list.rs`); component `v_virtual_list` exists (`gpui_kit::component::v_virtual_list`), signature not read — **UNCONFIRMED**.

## 4. State and re-render

```rust
struct Counter { n: u32, _subs: Vec<Subscription> }
impl Render for Counter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement { div() }
}
```

Conventions: `window` before `cx` everywhere (`|this, event, window, cx|`, `fn new(window: &mut Window, cx: &mut Context<Self>)`); app-only callbacks take just `cx`.

[fast] `app/context.rs`, `app/entity_map.rs`, `app.rs`, `app/async_context.rs`:

```rust
// AppContext (App, Context<T>, AsyncApp, ...):
fn new<T: 'static>(&mut self, build: impl FnOnce(&mut Context<T>) -> T) -> Entity<T>

// Entity<T>
fn read<'a>(&self, cx: &'a App) -> &'a T
fn read_with<R, C: AppContext>(&self, cx: &C, f: impl FnOnce(&T, &App) -> R) -> R
fn update<R, C: AppContext>(&self, cx: &mut C, f: impl FnOnce(&mut T, &mut Context<T>) -> R) -> R
fn update_in<R, C: VisualContext>(&self, cx: &mut C, f: impl FnOnce(&mut T, &mut Window, &mut Context<T>) -> R) -> C::Result<R>
fn downgrade(&self) -> WeakEntity<T>;  fn entity_id(&self) -> EntityId
// WeakEntity<T>: update(..) -> Result<R>, update_in(..) -> Result<R>, read_with(..) -> Result<R>, upgrade() -> Option<Entity<T>>

// Context<T>
fn notify(&mut self)
fn entity(&self) -> Entity<T>;  fn weak_entity(&self) -> WeakEntity<T>
fn listener<E: ?Sized>(&self, f: impl Fn(&mut T, &E, &mut Window, &mut Context<T>) + 'static)
    -> impl Fn(&E, &mut Window, &mut App) + 'static
fn emit<Evt>(&mut self, event: Evt) where T: EventEmitter<Evt>
fn subscribe<T2, Evt>(&mut self, entity: &Entity<T2>,
    on_event: impl FnMut(&mut T, Entity<T2>, &Evt, &mut Context<T>) + 'static) -> Subscription
fn subscribe_in<Emitter, Evt>(&mut self, emitter: &Entity<Emitter>, window: &Window,
    on_event: impl FnMut(&mut T, &Entity<Emitter>, &Evt, &mut Window, &mut Context<T>) + 'static) -> Subscription
fn observe<W>(&mut self, entity: &Entity<W>, on_notify: impl FnMut(&mut T, Entity<W>, &mut Context<T>) + 'static) -> Subscription
fn observe_in<V2>(&mut self, observed: &Entity<V2>, window: &mut Window,
    on_notify: impl FnMut(&mut T, Entity<V2>, &mut Window, &mut Context<T>) + 'static) -> Subscription
fn observe_global<G: 'static>(&mut self, f: impl FnMut(&mut T, &mut Context<T>) + 'static) -> Subscription
fn defer_in(&mut self, window: &Window, f: impl FnOnce(&mut T, &mut Window, &mut Context<T>) + 'static)
fn spawn<AsyncFn, R>(&self, f: AsyncFn) -> Task<R>
    where AsyncFn: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> R + 'static
fn spawn_in<AsyncFn, R>(&self, window: &Window, f: AsyncFn) -> Task<R>
    where AsyncFn: AsyncFnOnce(WeakEntity<T>, &mut AsyncWindowContext) -> R + 'static

// App
fn spawn<AsyncFn, R>(&self, f: AsyncFn) -> Task<R> where AsyncFn: AsyncFnOnce(&mut AsyncApp) -> R + 'static
fn defer(&mut self, f: impl FnOnce(&mut App) + 'static);  fn refresh_windows(&mut self)
// Window
fn spawn<AsyncFn, R>(&self, cx: &App, f: AsyncFn) -> Task<R> where AsyncFn: AsyncFnOnce(&mut AsyncWindowContext) -> R + 'static
fn defer(&self, cx: &mut App, f: impl FnOnce(&mut Window, &mut App) + 'static)
// AsyncApp::update(&self, f: impl FnOnce(&mut App) -> R) -> R            (no Result)
// AsyncWindowContext::update(&mut self, f: impl FnOnce(&mut Window, &mut App) -> R) -> Result<R>
pub trait EventEmitter<E: Any>: 'static {}
```

- Spawn uses **async closures**: `cx.spawn(async move |this, cx| { .. })`. A dropped `Task` is cancelled: `.detach()` it or store it. `Task<Result<_>>` has `.detach_and_log_err(cx)` (`TaskExt`).
- Keep every `Subscription` (field `_subs: Vec<Subscription>`) or call `.detach()`; dropping unsubscribes.
- Background work: `cx.background_spawn(async move { .. })` (`Send` future) or `cx.background_executor().timer(duration).await`.

```rust
// [comp] root.rs:472 pattern
cx.spawn_in(window, async move |this, cx| {
    cx.background_executor().timer(Duration::from_millis(200)).await;
    let _ = this.update_in(cx, |this, window, cx| { this.n += 1; cx.notify(); });
}).detach();

// events
enum ItemEvent { Saved(SharedString) }
impl EventEmitter<ItemEvent> for ItemForm {}
// emitter: cx.emit(ItemEvent::Saved(name));
// listener, in a constructor that has `window`:
let sub = cx.subscribe_in(&form, window, |this, _form, ev: &ItemEvent, _window, cx| { /* .. */ cx.notify(); });
```

- Element-local state without a struct field: `window.use_keyed_state(key: impl Into<ElementId>, cx, |window, cx| State) -> Entity<S>`, `window.use_state(cx, init)` ([fast] `window.rs:4644`).

## 5. Images

[fast] `elements/img.rs`.

```rust
pub fn img(source: impl Into<ImageSource>) -> Img
pub enum ImageSource { Resource(Resource), Render(Arc<RenderImage>), Image(Arc<Image>), Custom(..) }
// From impls: PathBuf, &Path, Arc<Path>  => Resource::Path (read with std::fs::read)
//             &str, String, SharedString => URL if it parses as one, else Resource::Embedded (AssetSource lookup!)
//             SharedUri, Arc<RenderImage>, Arc<Image>, Fn(&mut Window, &mut App) -> Option<Result<Arc<RenderImage>, ImageCacheError>>
pub trait StyledImage {
    fn object_fit(self, object_fit: ObjectFit) -> Self      // Fill | Contain (default) | Cover | ScaleDown | None
    fn grayscale(self, grayscale: bool) -> Self
    fn with_fallback(self, fallback: impl Fn() -> AnyElement + 'static) -> Self   // shown when loading fails
    fn with_loading(self, loading: impl Fn() -> AnyElement + 'static) -> Self
}
```

```rust
img(path.clone())                                   // path: PathBuf  (NOT a &str/String path)
    .size(px(64.)).rounded(px(6.))
    .object_fit(ObjectFit::Cover)
    .with_fallback(|| div().size(px(64.)).bg(rgb(0x2a2f3a)).into_any_element())
```

- Passing a filesystem path as `&str`/`String` silently becomes an embedded-asset lookup and fails; convert to `PathBuf`.
- `Img` is `Styled + InteractiveElement`; `.id(..)` gives `Stateful<Img>` (still `StyledImage`).
- Formats: decoded via the `image` crate (`image::guess_format`), GIF and WebP handled explicitly, otherwise the bytes are tried as SVG. `Img::extensions()` lists supported extensions.
- Loaded images are cached by source. Invalidate after overwriting a file: `ImageSource::from(path).remove_asset(cx)` (`img.rs:573`).
- **Nearest-neighbour / pixelated rendering: not available.** No sampling option exists on `Img`/`ImageStyle`; the wgpu renderer hardcodes `FilterMode::Linear` (`R/gpui-fast-wgpu-0.1.15/src/wgpu_renderer.rs:1274`). Pre-scale pixel art yourself.
- Whether `rounded(..)` clips the image itself (vs. needing an `overflow_hidden()` parent) is **UNCONFIRMED**.

## 6. Native file-open dialog

[fast] `app.rs:2038`, `platform.rs:2953`:

```rust
pub fn prompt_for_paths(&self, options: PathPromptOptions)
    -> futures::channel::oneshot::Receiver<Result<Option<Vec<PathBuf>>>>     // method on App
pub struct PathPromptOptions { pub files: bool, pub directories: bool, pub multiple: bool, pub prompt: Option<SharedString> }
pub fn prompt_for_new_path(&self, directory: &Path, suggested_name: Option<&str>) -> oneshot::Receiver<Result<Option<PathBuf>>>
```

Awaiting the receiver yields `Result<Result<Option<Vec<PathBuf>>>, Canceled>`; `Ok(Ok(None))` = user cancelled. No `PathPromptOptions: Default`, and **no file-type filter field**. On Linux it goes through the XDG desktop portal (ashpd); `prompt` becomes the accept-button label; an error is returned if no portal is running (`R/gpui-fast-linux-0.1.15/src/linux/platform.rs:819`).

```rust
Button::new("pick-thumb").label("Choose image").on_click(cx.listener(|_this, _, _window, cx| {
    let paths = cx.prompt_for_paths(PathPromptOptions {
        files: true, directories: false, multiple: false, prompt: Some("Select".into()),
    });
    cx.spawn(async move |this, cx| {
        if let Ok(Ok(Some(paths))) = paths.await {
            if let Some(path) = paths.into_iter().next() {
                let _ = this.update(cx, |this, cx| { this.thumbnail = Some(path); cx.notify(); });
            }
        }
    }).detach();
}))
```
(No in-crate caller of `prompt_for_paths` exists to copy; the snippet composes confirmed signatures — treat as **UNCONFIRMED** until compiled.) Use `cx.spawn_in(window, ..)` + `this.update_in(cx, |this, window, cx| ..)` if the continuation needs the window.

## 7. gpui-component widgets

Shared traits (`gpui_kit::component::{Sizable, Disableable, Selectable}`, [comp] `sizing.rs`, `styled.rs`): `.xsmall() .small() .large() .with_size(impl Into<Size>)`; `.disabled(bool)`; `.selected(bool)`. Helpers: `h_flex()` = `div().flex().flex_row().items_center()`; `v_flex()` = `div().flex().flex_col()` ([base] `styled.rs`). `StyledExt`: `.h_flex()`, `.v_flex()`, `.paddings(..)`, `.debug_red()`.

### Button — `gpui_kit::component::button::{Button, ButtonVariants, ButtonVariant, ButtonRounded, ButtonCustomVariant}` ([comp] `button/button.rs`)

```rust
Button::new(id: impl Into<ElementId>)
  .label(impl Into<SharedString>)  .icon(impl Into<ButtonIcon>)   // IconName, Icon, Spinner, ProgressCircle
  // ButtonVariants: .primary() .secondary() .danger() .warning() .success() .info() .ghost() .link() .text() .custom(ButtonCustomVariant)
  .outline()                       // modifier on any variant, not a variant
  .xsmall() / .small() / .large()  .compact()  .rounded(ButtonRounded::Medium | px(..))
  .disabled(bool)  .loading(bool)  .selected(bool)  .tooltip(impl Into<SharedString>)
  .on_click(impl Fn(&ClickEvent, &mut Window, &mut App) + 'static)
  .on_hover(impl Fn(&bool, &mut Window, &mut App) + 'static)
```
`Button` is `Styled + ParentElement + InteractiveElement` (so `.w_full()`, `.child(..)` work). Icon-only: `Button::new("del").icon(IconName::Delete).ghost().small()`.
`ButtonCustomVariant::new(cx).color(Hsla).foreground(Hsla).hover(Hsla).active(Hsla).shadow(bool)`.

```rust
Button::new("save").primary().label("Save").on_click(cx.listener(|this, _, _window, cx| { this.save(cx); cx.notify(); }))
```

### Input — `gpui_kit::component::input::{Input, InputState, InputEvent}` ([base] `input/base/state.rs`, [comp] `input/input.rs`)

`InputState = InputBaseState<InputMode>` (single line; `TextareaState` + `Textarea` for multi-line). State is an entity created once (constructor, not `render`):

```rust
InputState::new(window: &mut Window, cx: &mut Context<Self>) -> Self
// builders: .placeholder(impl Into<SharedString>) .default_value(impl Into<SharedString>) .masked(bool)
//           .pattern(regex::Regex) .validate(impl Fn(&str, &mut App) -> bool + 'static) .clean_on_escape()
//           .mask_pattern(..) .step(impl Into<NumberStep>) .min(f64) .max(f64)
fn value(&self) -> SharedString
fn set_value(&mut self, value: impl Into<InputContent>, window: &mut Window, cx: &mut Context<Self>)   // emits NO Change event
fn set_placeholder(&mut self, placeholder: impl Into<SharedString>, _: &mut Window, cx: &mut Context<Self>)
fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>)
fn focus(&self, window: &mut Window, cx: &mut Context<Self>)
fn clean(&mut self, window: &mut Window, cx: &mut Context<Self>)        // clear
fn select_all(&mut self, _: &mut Window, cx: &mut Context<Self>)
pub enum InputEvent { Change, PressEnter { secondary: bool, shift: bool }, Focus, Blur }
// also EventEmitter<InputEditsEvent>, EventEmitter<NumberInputEvent>
```
Element: `Input::new(&Entity<InputState>)` + `.id(..)`, `.prefix(el)`, `.suffix(el)`, `.cleanable(bool)`, `.mask_toggle()`, `.disabled(bool)`, `.readonly(bool)`, `.appearance(bool)`, `.bordered(bool)`, `.aria_label(..)`, sizes via `Sizable`, plus `Styled` (`.w(px(240.))`).
`InputContent: From<&str | &String | String | ...>` ([base] `input/base/inline_tokens.rs:148`). `set_value` is documented to emit no `Change`; whether `clean` emits `Change` is **UNCONFIRMED**.

```rust
struct ItemForm { name: Entity<InputState>, _subs: Vec<Subscription> }
impl ItemForm {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("Item name"));
        let _subs = vec![cx.subscribe_in(&name, window, |this, input, ev: &InputEvent, window, cx| match ev {
            InputEvent::Change => { let _text = input.read(cx).value(); cx.notify(); }
            InputEvent::PressEnter { .. } => this.submit(window, cx),
            _ => {}
        })];
        Self { name, _subs }
    }
}
// render: Input::new(&self.name).id("item-name").w(px(240.))
// read:   self.name.read(cx).value()
// set:    self.name.update(cx, |s, cx| s.set_value("Iron", window, cx));
// clear:  self.name.update(cx, |s, cx| s.set_value("", window, cx));
```
(Patterns: [comp] `setting/fields/number.rs:91`, `input/search.rs:103`, [kit] `tests/ui.rs`.)

### NumberInput — `gpui_kit::component::input::{NumberInput, NumberInputEvent, NumberStep, StepAction}`

Same `InputState`, configured for stepping; text is still read with `.value()` and parsed (`text.parse::<f64>()`):

```rust
let qty = cx.new(|cx| InputState::new(window, cx).default_value("1").step(1.0).min(0.0).max(9999.0));
// render: NumberInput::new(&qty).placeholder("0").small()      // .prefix/.suffix/.appearance/.disabled
```
With `step`/`min`/`max` set the control updates its own value and emits `InputEvent::Change`; without them it emits `NumberInputEvent::Step(StepAction::Increment | Decrement)` and you apply the step ([base] `number_input.rs`, [base] `input/base/state.rs:12671` doc). Out-of-range typed text is clamped on blur ([comp] `setting/fields/number.rs`). Decimal types need the kit `decimal` feature (not enabled) — not inspected, **UNCONFIRMED**.

### Select — `gpui_kit::component::select::{Select, SelectState, SelectEvent, SelectItem, SearchableVec}` + `gpui_kit::component::IndexPath` ([comp] `select.rs`, `searchable_list/`)

```rust
SelectState::<D>::new(delegate: D, selected_index: Option<IndexPath>, window: &mut Window, cx: &mut Context<Self>) -> Self
  .searchable(bool)
fn selected_value(&self) -> Option<&<D::Item as SelectItem>::Value>
fn selected_index(&self, cx: &App) -> Option<IndexPath>
fn set_selected_index(&mut self, ix: Option<IndexPath>, window, cx)
fn set_selected_value(&mut self, value: &Value, window, cx)
fn set_items(&mut self, items: D, window, cx)
pub enum SelectEvent<D> { Confirm(Option<Value>) }       // also emits gpui_kit::DismissEvent
pub trait SelectItem: Clone {                             // = SearchableListItem
    type Value: Clone + PartialEq;
    fn title(&self) -> SharedString;
    fn value(&self) -> &Self::Value;
    // optional: display_title, render, matches(&self, query: &str) -> bool, disabled
}
```
Delegates: `Vec<T: SelectItem>` and `SearchableVec<T>` (`SearchableVec::new(vec![..])`); `String`, `SharedString`, `&'static str` already implement `SelectItem` with `Value = Self`. `IndexPath::new(row)`, `IndexPath::default().row(n).section(s)`.
Element: `Select::new(&state)` + `.id(..)`, `.placeholder(..)`, `.title_prefix(..)`, `.cleanable(bool)`, `.icon(..)`, `.menu_width(..)`, `.menu_max_h(..)`, `.disabled(bool)`, `.empty(..)`, sizes, `Styled`.

```rust
#[derive(Clone)] struct ItemOption { id: Uuid, name: SharedString }
impl SelectItem for ItemOption {
    type Value = Uuid;
    fn title(&self) -> SharedString { self.name.clone() }
    fn value(&self) -> &Uuid { &self.id }
}
let items: Entity<SelectState<SearchableVec<ItemOption>>> =
    cx.new(|cx| SelectState::new(SearchableVec::new(options), None, window, cx).searchable(true));
let sub = cx.subscribe_in(&items, window,
    |this, _, ev: &SelectEvent<SearchableVec<ItemOption>>, _window, cx| {
        let SelectEvent::Confirm(value) = ev;      // Option<Uuid>
        this.item_id = *value; cx.notify();
    });
// render: Select::new(&self.items).id("item").placeholder("Pick an item").w(px(240.))
// read:   self.items.read(cx).selected_value().cloned()
```
(Built from [kit] `tests/integration/controls.rs`; the custom-item variant is composed — **UNCONFIRMED** until compiled. Confirmation is applied through `defer_in`, after the click handler returns.)

### Dialog — `gpui_kit::component::WindowExt` + `gpui_kit::component::dialog::{Dialog, DialogFooter, DialogAction, DialogClose, AlertDialog, DialogButtonProps}` ([comp] `window_ext.rs`, `dialog/`)

```rust
fn open_dialog<F>(&mut self, cx: &mut App, build: F) where F: Fn(Dialog, &mut Window, &mut App) -> Dialog + 'static
fn open_alert_dialog<F>(&mut self, cx: &mut App, build: F) where F: Fn(AlertDialog, &mut Window, &mut App) -> AlertDialog + 'static
fn close_dialog(&mut self, cx: &mut App);  fn close_all_dialogs(&mut self, cx: &mut App);  fn has_active_dialog(&mut self, cx: &mut App) -> bool
fn open_sheet / open_sheet_at(placement, ..) / close_sheet
```
- `build` is a **render closure** (`Fn`, may rerun). Create `InputState`s etc. before opening and capture clones; never `cx.new` inside it.
- `Dialog` builders: `.title(el)`, `.child(el)` (`ParentElement`), `.footer(el)`, `.on_ok(Fn(&ClickEvent, &mut Window, &mut App) -> bool)` (return `true` to close), `.on_cancel(.. -> bool)`, `.on_close(Fn(&ClickEvent, &mut Window, &mut App))`, `.w(px)`/`.width(px)`, `.max_w(px)`, `.margin_top(px)`, `.centered()`, `.close_button(bool)`, `.overlay(bool)`, `.overlay_closable(bool)`, `.keyboard(bool)`, `.button_props(DialogButtonProps::default().ok_text(..).cancel_text(..).ok_variant(ButtonVariant::Danger).show_cancel(true))`.
- A plain `Dialog` renders **no buttons by itself**: compose a `DialogFooter`. `DialogAction::new().child(Button)` dispatches Confirm (runs `on_ok`); `DialogClose::new().trigger(|button| button.label("Cancel"))` cancels.

```rust
// [kit] tests/integration/overlays.rs
let draft = self.draft.clone();                     // Entity<InputState>, created earlier
window.open_dialog(cx, move |dialog, _, _| {
    let draft = draft.clone();
    dialog
        .title("New item")
        .child(Input::new(&draft).id("name"))
        .footer(DialogFooter::new()
            .child(DialogClose::new().trigger(|button| button.label("Cancel")))
            .child(DialogAction::new().child(Button::new("ok").primary().label("Save"))))
        .on_ok(move |_, window, cx| {
            if draft.read(cx).value().is_empty() { return false; }      // keep open
            window.push_notification(Notification::new().message("Saved"), cx);
            true                                                         // close
        })
});
```
To mutate your view from `on_ok`, capture `cx.entity().downgrade()` before opening and call `weak.update(cx, |this, cx| ..)` (pattern from [kit] `tests/integration/components.rs` popover test). Close from anywhere: `window.close_dialog(cx)`.

**Alert / confirm**: `AlertDialog` builders `.title(el)`, `.description(el)`, `.icon(el)`, `.confirm()` (adds Cancel), `.show_cancel(bool)`, `.ok_text(..)`, `.cancel_text(..)`, `.ok_variant(ButtonVariant)`, `.cancel_variant(..)`, `.on_ok(.. -> bool)`, `.on_cancel(.. -> bool)`, `.on_close(..)`, `.width(px)`, `.footer(el)`. It builds its own OK/Cancel footer.

```rust
window.open_alert_dialog(cx, move |alert, _, _| {
    let weak = weak.clone();
    alert.title("Delete recipe?").description("This cannot be undone.")
        .confirm().ok_text("Delete").ok_variant(ButtonVariant::Danger)
        .on_ok(move |_, _window, cx| { let _ = weak.update(cx, |this, cx| { this.delete(cx); cx.notify(); }); true })
});
```
The `alert.warning()` call in the `open_alert_dialog` doc comment does not exist on `AlertDialog` in 0.9.1 (no such method in `dialog/alert_dialog.rs`). Native OS prompt alternative ([fast] `window.rs:7024`): `window.prompt<T: Clone + Into<PromptButton>>(level: PromptLevel, message: &str, detail: Option<&str>, answers: &[T], cx: &mut App) -> oneshot::Receiver<usize>` (index of the chosen answer); `&str: Into<PromptButton>` is **UNCONFIRMED**.

### Notification — `gpui_kit::component::notification::{Notification, NotificationType}` ([comp] `notification.rs`)

`window.push_notification(note: impl Into<Notification>, cx)`; `Into` from `&str`, `String`, `SharedString`, `(NotificationType, T)`. Constructors `Notification::new()`, `::info(msg)`, `::success(msg)`, `::warning(msg)`, `::error(msg)`; builders `.message(..)`, `.title(..)`, `.icon(..)`, `.with_type(NotificationType::{Info,Success,Warning,Error})`, `.autohide(bool)`, `.id::<T>()` / `.id1::<T>(key)` (dedupe), `.on_click(..)`, `.action(..)`. Also `window.clear_notifications(cx)`, `window.remove_notification::<T>(cx)`.

```rust
window.push_notification(Notification::success("Plan saved").title("Saved"), cx);
window.push_notification((NotificationType::Error, "Could not read image"), cx);
```

### Small widgets

| Widget | Import (`gpui_kit::component::`) | API (source) |
| --- | --- | --- |
| Tag | `tag::Tag` | `Tag::new()`, `::primary() ::secondary() ::danger() ::success() ::warning() ::info()`, `::custom(color, foreground, border: Hsla)`, `::color(impl Into<ColorName>)`; `.outline() .rounded(..) .rounded_full() .small()`, `.child("text")` ([comp] `tag.rs`) |
| Badge | `badge::Badge` | `Badge::new().count(usize).max(usize).dot().icon(..).color(Hsla).child(target_el)` — wraps the element it decorates ([comp] `badge.rs`) |
| Progress | `progress::{Progress, ProgressCircle}` | `Progress::new(id).value(f32 /*0..=100*/).color(Hsla).loading(bool)` + `Sizable`, `Styled`; `ProgressCircle::new(id)` same builders |
| Switch | `switch::Switch` | `Switch::new(id).checked(bool).label(..).on_change(Fn(&bool, &mut Window, &mut App)).tooltip(..).disabled(bool).small()` — controlled: write the value and `cx.notify()` |
| Checkbox | `checkbox::Checkbox` | `Checkbox::new(id).label(..).checked(bool).on_change(Fn(&bool, &mut Window, &mut App)).disabled(bool)` (`on_click` is an alias) |
| Tooltip | `tooltip::Tooltip` | `Tooltip::new(text: impl Into<Text>).build(window, cx) -> AnyView`; `Tooltip::element(|window, cx| el)`; use inside `.tooltip(..)` of an element with `.id()`. Button/Switch/Checkbox have `.tooltip("text")` |
| Label | `label::Label` | `Label::new(text).secondary(text).highlights(query)` |
| Separator | `separator::Separator` | `Separator::horizontal()`, `::vertical()`, `.label(..)`, `.dashed()`, `.color(..)` |
| Spinner / Skeleton / Empty / GroupBox | `spinner`, `skeleton`, `empty`, `group_box` | modules exist; constructors not read — **UNCONFIRMED** |

```rust
Checkbox::new("craftable").label("Craftable").checked(self.craftable)
    .on_change(cx.listener(|this, checked: &bool, _, cx| { this.craftable = *checked; cx.notify(); }))
```
([kit] `tests/integration/controls.rs`.)

### Icon — `gpui_kit::component::{Icon, IconName}` ([comp] `icon.rs`, [assets])

`Icon::new(impl Into<Icon>)` (any `IconNamed`: component `IconName` or `gpui_kit::assets::IconName`), `.path("icons/x.svg")`, `.small()/.large()/.with_size(px(..))`, `.text_color(..)`, `.rotate(radians)`. `IconName` is itself `IntoElement` (`.child(IconName::Plus)`). Icon colour follows the text colour.

Component `IconName` variants (exactly the SVGs embedded by `assets::Assets`; generated from [assets] `default-icons.txt` by `build.rs`):

```
ALargeSmall ArrowDown ArrowLeft ArrowRight ArrowUp Asterisk Ban Battery BatteryCharging BatteryFull
BatteryLow BatteryMedium BatteryWarning Bell BookOpen Bot Building2 Calendar CaseSensitive ChartPie
Check ChevronDown ChevronLeft ChevronRight ChevronUp ChevronsUpDown CircleAlert CircleCheck CircleUser
CircleX Close Copy Cpu Dash Delete Ellipsis EllipsisVertical ExternalLink Eye EyeOff File FileText
Folder FolderClosed FolderOpen Frame GalleryVerticalEnd Github Globe HardDrive Heart HeartOff Inbox
Info Inspector LayoutDashboard Loader LoaderCircle Map Maximize MemoryStick Menu Mic Minimize Minus
Moon Network Palette PanelBottom PanelBottomOpen PanelLeft PanelLeftClose PanelLeftOpen PanelRight
PanelRightClose PanelRightOpen Pause Play Plus Redo Redo2 RefreshCw Replace ResizeCorner RotateCw
Search Settings Settings2 SortAscending SortDescending Square SquareTerminal Star StarFill StarOff Sun
ThumbsDown ThumbsUp TriangleAlert Undo Undo2 User WindowClose WindowMaximize WindowMinimize WindowRestore
```
No `Trash`, `Pencil`, `Save`, `Image`, `Package`, `X` here (use `Delete`, `Close`). `gpui_kit::assets::IconName` has all 1,830 Lucide names (e.g. `Trash`, `Pencil`, `Save`, `Image`, `ImagePlus`, `Package`, `Hammer`, `Box`, `Boxes`, `Layers`, `Target`, `Upload`, `Download`, `Library`, `ListChecks`, `Gamepad2`, `Swords`, `Wrench`, `X`), **but their SVGs are not embedded by `assets::Assets`** — an unembedded icon fails to load. Options ([assets] `README.md`, `src/lib.rs`, `examples/extra_assets.rs`):
1. `application().with_assets(gpui_kit::assets::AllAssets)` — every icon, about +1 MiB.
2. Select extras and compose a source:

```rust
gpui_kit::assets::icon_assets!(ExtraIcons, [Trash, Pencil, Package]);
struct AppAssets;
impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if let Some(bytes) = ExtraIcons.load(path)? { return Ok(Some(bytes)); }
        gpui_kit::assets::Assets.load(path)
    }
    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = gpui_kit::assets::Assets.list(path)?;
        paths.extend(ExtraIcons.list(path)?); paths.sort(); paths.dedup(); Ok(paths)
    }
}
// application().with_assets(AppAssets);   Icon::new(gpui_kit::assets::IconName::Trash)
```
The macro path `gpui_kit::assets::icon_assets!` is given by the assets README; expansion through the facade was not compiled here — **UNCONFIRMED**.

### TitleBar — `gpui_kit::component::TitleBar` ([comp] `title_bar.rs`)

`TitleBar::new().child(el)` (`ParentElement`), `TITLE_BAR_HEIGHT = px(34.)`, `.on_close_window(Fn(&ClickEvent, &mut Window, &mut App))` (Linux only). Open the window with `WindowOptions { window_min_size: .., ..TitleBar::window_options() }` (sets `titlebar: appears_transparent: true`, `app_owns_titlebar_drag: true`). On Linux a custom title bar also needs `window_decorations: Some(WindowDecorations::Client)` per the `TitlebarOptions` doc ("Refer to `WindowOptions::window_decorations` on Linux"); `window_options()` does not set it. Exact Linux behaviour **UNCONFIRMED**. For a plain app the system title bar + `TitlebarOptions.title` is simpler.

### Sidebar (brief) — `gpui_kit::component::sidebar::{Sidebar, SidebarHeader, SidebarFooter, SidebarGroup, SidebarMenu, SidebarMenuItem, SidebarToggleButton}` ([comp] `sidebar/`)

`Sidebar::<E: SidebarItem>::new(id)` `.side(Side)` `.collapsible(..)` `.collapsed(bool)` `.header(el)` `.footer(el)` `.child(E)` `.children(..)`; `SidebarGroup::new(label).child(E)`; `SidebarMenu::new().child(impl Into<SidebarMenuItem>)`; `SidebarMenuItem::new(label).icon(..).active(bool).on_click(Fn(&ClickEvent, &mut Window, &mut App)).children(..).suffix(..)`.

```rust
Sidebar::new("nav")
    .header(SidebarHeader::new().child("Game Planner"))
    .child(SidebarGroup::new("Library").child(SidebarMenu::new()
        .child(SidebarMenuItem::new("Items").icon(IconName::Inbox).active(self.page == Page::Items)
            .on_click(cx.listener(|this, _, _, cx| { this.page = Page::Items; cx.notify(); })))))
```
No in-crate usage example exists; signatures are confirmed, the composed snippet (generic inference, `SidebarHeader: ParentElement`) is **UNCONFIRMED**.

## 8. gpui-fast gotchas

- **Public API is the same as upstream `gpui-pre 0.3.12`.** A function-name diff of `styled.rs`, `elements/{div,img,text,svg,uniform_list}.rs`, `app.rs`, `app/context.rs`, `app/async_context.rs`, `color.rs`, `geometry.rs`, `style.rs`, `interactive.rs` between `R/gpui-pre-0.3.12` and `R/gpui-fast-0.1.15` shows no additions or renames; the only names missing from those files (`Div::is_hovered`, `Window::{hit_test, snap_bounds, snapped_content_mask, should_use_subpixel_rendering}`, several `ListState` fns in `list.rs`) are internals moved under `src/fast/`. Whether each is still callable was not checked — **UNCONFIRMED**; none are needed for app code.
- **Retained rendering** ([fast] `fast/retained.rs`, `fast/dependencies.rs`): every view is a retained subtree that is replayed from the previous frame unless something it *read while rendering* changed (entities, globals) or it was notified. Consequences: always `cx.notify()` after mutating view state; data held outside entities/globals (`Rc<RefCell<..>>`, statics, files) never triggers a re-render on its own; do not rely on `render` being called every frame for side effects.
- In tests, a bare `window.draw(cx)` may skip `render`; the component crate uses `window.refresh(); window.draw(cx).clear(cx);` ([comp] `lib.rs` `test_redraw`), and the kit helper `window.render_frame(cx)` does the refresh for you.
- The `test` attribute macro leaks through `use gpui_kit::*` even without `test-support` (section 0).
- Derive/attribute macros re-exported for the Fast engine: `Action, AppContext, IntoElement, Render, VisualContext, register_action, test, ...` ([kit] `src/lib.rs`). `#[derive(IntoElement)]` + `impl RenderOnce` works through the facade because the kit aliases itself as `gpui` (`pub use crate as gpui`, doc-hidden); if a macro emits an unresolved `gpui::` path in app code, the documented escape hatch is `extern crate gpui_kit as gpui;`. Whether `#[derive(IntoElement)]` needs that alias in this app is **UNCONFIRMED**. Use `gpui_kit::actions!` for actions (the kit's own macro, emits `$crate::Action`).
- Scroll containers are painted into cached tile layers on Linux/macOS/Windows ([fast] `fast/layers/mod.rs`); no API change.
- Image sampling is linear only (section 5).

## 9. Testing

`Cargo.toml` (per `CLAUDE.md` and [kit] `README.md`/`TESTING.md`: enable `test-support` under dev-dependencies; `gpui-fast` must match the main dependency):

```toml
[dev-dependencies]
gpui-kit = { version = "0.9.1", features = ["gpui-fast", "test-support"] }
```

Rules ([kit] `TESTING.md`, `tests/integration/test_macro.rs`): use `#[gpui_kit::test]` (sync or `async fn`, arg `cx: &mut TestAppContext`); import explicitly, no `use gpui_kit::*`; mark queryable elements with `.id("x").test_support()` (place before `.track_focus`); re-query after each interaction; `label()`/`value()` are accessibility properties, not rendered text.

```rust
// tests/ui.rs  (adapted from [kit] tests/ui.rs + tests/common/mod.rs)
use gpui_kit::test::{TestSupportExt, TestWindowExt};
use gpui_kit::{
    App, AppContext, Bounds, Context, Entity, Pixels, Point, Size, TestAppContext, Window, WindowBounds,
    WindowHandle, WindowOptions, base::Root,
    component::{button::Button, input::{Input, InputState}},
    div, prelude::*, px, size,
};

fn open<V: Render>(cx: &mut TestAppContext, size: Size<Pixels>,
    build: impl FnOnce(&mut Window, &mut App) -> Entity<V>) -> (WindowHandle<Root>, Entity<V>) {
    cx.update(|cx| {
        let bounds = Bounds { origin: Point::default(), size };
        let (window, content) = gpui_kit::open_window(
            WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() }, cx, build,
        ).expect("open test window");
        (window.downcast::<Root>().expect("Base Root"), content)
    })
}

struct Profile { name: Entity<InputState>, saved: Option<gpui_kit::SharedString> }
impl Render for Profile {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().flex().flex_col().gap_4()
            .child(Input::new(&self.name).id("name").w(px(240.)))
            .child(Button::new("save").label("Save").on_click(cx.listener(|this, _, _, cx| {
                this.saved = Some(this.name.read(cx).value()); cx.notify();
            })))
    }
}

#[gpui_kit::test]
fn saves_a_profile(cx: &mut TestAppContext) {
    cx.update(gpui_kit::init);
    let (handle, profile) = open(cx, size(px(640.), px(480.)), |window, cx| {
        cx.new(|cx| Profile { name: cx.new(|cx| InputState::new(window, cx)), saved: None })
    });
    cx.update_window(handle.into(), |_, window, cx| {
        window.render_frame(cx);
        window.click("name", cx);
        window.input("Ada", cx);
        assert_eq!(window.find("name").value(), Some("Ada"));
        window.press("backspace", cx);
        window.click("save", cx);
        assert_eq!(profile.read(cx).saved.as_deref(), Some("Ad"));
    }).unwrap();
}
```

`TestWindowExt` ([kit] `src/test.rs`): `find(id)`, `try_find(id)`, `find_all(id)`, `within(id)`, `render_frame(cx)`, `click`, `click_at`, `right_click`, `double_click`, `hover`, `scroll(id, ScrollDelta, cx)`, `drag_to`, `press(key, cx)`, `input(text, cx)`. `ElementSnapshot`: `bounds()`, `visible()`, `label()`, `value()`, `focused()`, `checked()`, `disabled()`, `selected()`, `expanded()`, `role()`. Async waits: `use gpui_kit::test::TestAppContextExt; cx.wait_for(handle.into(), Duration::from_millis(100), |window, cx| ..).await` (needed e.g. after confirming a `Select` or opening a dialog). Deterministic overlay tests: `cx.set_reduce_motion(true)` after `init`.
Run: `cargo test` / `cargo test <name>`.
