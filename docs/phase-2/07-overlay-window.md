# Step 07: Overlay window (read-only)

**Goal.** A sidebar button opens and closes the **Overlay**: a small second window that stays above the game and shows the open game's progress and requirements. This step only shows them; step 08 adds actions.

**Depends on:** steps 01–05. The overlay shows projects.

## Decided
- **Kind of window.** Choose it once, when the overlay opens:
  - **Linux on Wayland** (`WAYLAND_DISPLAY` is set; GPUI picks its backend the same way): `WindowKind::LayerShell(LayerShellOptions { … })` with:
    - namespace `"game-planner-overlay"`
    - layer `Overlay`
    - anchor the chosen corner (for example `TOP | RIGHT`)
    - margin the chosen margin on the two anchored edges
    - keyboard interactivity `OnDemand`
    - no exclusive zone

    If opening fails because the compositor has no layer-shell (GNOME, `LayerShellNotSupportedError`), open it again as a normal small window titled "Game Planner overlay".
  - **X11, Windows and macOS:** `WindowKind::PopUp`, with no title bar, not resizable, placed at the chosen corner of the primary display with the margin.
  - **Untested platforms.** Only Linux on KDE Wayland can be tested here. Say so in the commit message, and add a comment at the PopUp branch saying it is untested.
- **Size.** 340 px wide and 560 px high; the content scrolls inside.
- **Settings for now.** Corner and margin come from an `OverlaySettings { corner, margin, opacity }` struct defined in this step: `Corner` has TopLeft, TopRight, BottomLeft and BottomRight, and the default is TopRight, 24 px, 100%. Step 09 saves and edits it; until then the default is used.
- **What it shows,** for the game open in the main window, all read-only:
  - **Header:** the game title, truncated, and a close button ✕.
  - **Progress:** each project as name plus "3 / 5 reached" (a Reached pill when all are), then the loose plans as item name plus "assigned / target" (a Reached pill when reached). The rows are compact, about 30 px.
  - **Gather:** one line per gather row: 24 px icon, name, "N missing" or "covered", and a "recipe loop" mark when flagged.
  - **Craft:** one line per craft row, inputs first: icon, name, "N crafts", and a Ready mark.
  - **Empty states:** "Open a game in Game Planner." with no game open; "No plans yet." with no plans.
- **No restructuring.** The overlay reads the main window's `Planner` state (`data`, `req`, `progress`, `games`) and reuses `Planner::pic` for icons. `Overlay` lives in a submodule of `ui`, so it can reach `Planner`'s private fields. It re-renders by observing the planner (`cx.observe`), whose `reload` already ends in `cx.notify()`.
- **Opening and closing.**
  - A sidebar row **Overlay** (id `overlay-toggle`, just above Settings) opens and closes it. It shows an "On" pill while the overlay is open, and it works with or without an open game.
  - Closing the overlay by any route (its ✕, the sidebar row, or the window manager on the fallback window) leaves `Planner::overlay` empty.
  - Closing the main window quits the app, overlay included.
- **The theme** is the same as the main window's. The window is opaque in this step; step 09 deals with opacity.

## Read first
- `src/main.rs`: window options and `QuitMode`.
- `src/ui/mod.rs`: `Planner` fields, `sidebar`, `nav_item`, `pic`, `reload`.
- `src/ui/requirements.rs`: how gather and craft rows and the plan strip read `req` and `progress`.
- The vendored gpui-fast 0.1.15 sources (path pattern in `CLAUDE.md`):
  - `src/platform.rs`: `WindowOptions`, `WindowKind`
  - `src/platform/layer_shell.rs`: `LayerShellOptions`, `Anchor`, `Layer`, `KeyboardInteractivity`, `LayerShellNotSupportedError`
  - In gpui-fast-linux 0.1.15, `src/linux/wayland/window.rs` shows how the layer surface takes its size from the window bounds.

  `WindowKind::LayerShell` exists only under `cfg(all(target_os = "linux", feature = "wayland"))`, and gpui-kit enables `wayland`. Find the path it is re-exported under in gpui-kit; it is probably `gpui_kit::layer_shell`.
- `docs/gpui-kit-cheatsheet.md` §1: `open_window`, `Root`, `cx.on_window_closed`.

## Changes
1. **New module `src/ui/overlay.rs`** with:
   - `OverlaySettings` and `Corner` (with `serde` derives, ready for step 09)
   - `pub(super) struct Overlay { planner: WeakEntity<Planner>, … }`, which implements `Render`
   - a pure function `window_options(settings, wayland: bool, display: Bounds<Pixels>) -> WindowOptions` that maps corner and margin to the layer-shell anchor and margin, or to PopUp bounds
   - `pub(super) fn open(planner, settings, cx) -> Option<AnyWindowHandle>`, with the GNOME fallback
2. **`Planner`:**
   - an `overlay: Option<AnyWindowHandle>` field
   - `toggle_overlay(window, cx)`
   - the sidebar row
   - a `cx.on_window_closed` subscription that clears `overlay` when the overlay closes, and calls `cx.quit()` when the main window closes
3. **`CLAUDE.md`:** under Architecture, add a line for `src/ui/overlay.rs`. Say that it is a second window reading `Planner` and that it can be removed with the checklist below.

## Tests
Put them at the end of `src/ui/tests.rs` under a `// ---- overlay ----` heading, so removing the overlay is one cut. The test platform has no layer-shell; open the overlay as `WindowKind::Normal` in tests, through a `cfg(test)` branch or a parameter.
- Clicking `overlay-toggle` opens a second window showing the fixture's Torch plan as "0 / 40". Clicking it again closes the window.
- After a craft in the main window (`panel-craft-all`), the overlay's numbers change without any action in the overlay.
- `window_options` (a unit test):
  - TopRight with margin 24 on Wayland gives anchor `TOP | RIGHT` and margin `(24, 24, 0, 0)`.
  - BottomLeft off Wayland gives PopUp bounds at the display's bottom-left corner, inset by 24.

## Manual check (KDE Wayland)
Run a game in a borderless window or fullscreen-windowed, then open the overlay from the sidebar. It stays above the game in the top-right corner. Take a screenshot.

## How to remove the overlay
Keep this list current: steps 08–10 add to it.
1. Delete `src/ui/overlay.rs`, and `src/ui/overlay/` if it exists.
2. In `src/ui/mod.rs`, remove:
   - `mod overlay;`
   - the `Planner::overlay` field and `toggle_overlay`
   - the `on_window_closed` overlay handling (keep quitting when the main window closes, or drop it, since the main window is then the only window)
   - the sidebar row `overlay-toggle`
3. Delete the `// ---- overlay ----` section of `src/ui/tests.rs`.
4. Remove **Overlay** from `CONTEXT.md` and the overlay line from `CLAUDE.md`.
5. Run `cargo test` and `cargo clippy --all-targets`.
