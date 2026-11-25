# Step 10 (optional): Global shortcut for the overlay

**Goal.** A system-wide shortcut shows or hides the overlay while the game has focus.

**Optional.** Do this step only when the owner asks for it. Phase 2 is complete without it.

**Depends on:** step 07.

## Decided
- **Linux,** on Wayland and on X11 desktops with xdg-desktop-portal: use the portal's **GlobalShortcuts** interface through `ashpd`. `ashpd` 0.13.x is already in `Cargo.lock` through gpui-fast-linux, but without the `global_shortcuts` feature.
  - Add `ashpd` as a direct dependency at the version already in `Cargo.lock`, with the features gpui-fast-linux uses plus `global_shortcuts`.
  - Bind one shortcut, id `toggle-overlay`, description "Show or hide the Game Planner overlay", preferred trigger Ctrl+Alt+G.
  - The player confirms or changes the key in the desktop's own dialog or settings (KDE: System Settings → Shortcuts).
  - Portals identify a non-sandboxed app by its desktop file app id, which here is `art.bangmarcel.gameplanner` (`APP_ID` in `src/main.rs`). Check `ashpd`'s docs for registering a host app.
- **Windows and macOS:** the `global-hotkey` crate. Register Ctrl+Alt+G (on macOS, Cmd+Option+G) and poll its event receiver from a GPUI timer on the main thread. Both platforms need the registering thread to run the native event loop, which GPUI's main thread does. These platforms are untested; say so in the commit message.
- **When the shortcut fires,** it calls `Planner::toggle_overlay` on the main thread.
- **If registration fails,** there is no shortcut: log the error once, and the sidebar button keeps working.
- **The Settings Overlay section** shows the shortcut. On Linux it adds "Change it in your desktop's shortcut settings."

## Read first
- The current documentation of `ashpd` (`desktop::global_shortcuts`) and `global-hotkey`. Look these up rather than relying on memory.
- `src/ui/overlay.rs`, `Planner::toggle_overlay`.
- `Cargo.lock`, for the `ashpd` version and features already in use.

## Changes
1. A `src/ui/overlay/shortcut.rs` module, declared with `mod shortcut;` inside `src/ui/overlay.rs`, with a single `start(planner, cx)` entry point and `cfg` branches per platform.
2. Call `shortcut::start` once, after the main window opens.
3. **Removal checklist** in step 07: add `src/ui/overlay/shortcut.rs`, the `start` call, and the `ashpd` and `global-hotkey` dependencies in `Cargo.toml`.

## Tests
The real shortcut can't be exercised in tests. Unit-test whatever is pure, such as turning the trigger description into the portal's or the crate's format.

## Manual check (KDE Wayland)
On first launch, KDE asks to confirm the shortcut. With a game focused, Ctrl+Alt+G shows and hides the overlay. Take screenshots.

## Done when
- The README's done rule is met.
