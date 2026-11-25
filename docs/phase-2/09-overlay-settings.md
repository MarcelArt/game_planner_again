# Step 09: Overlay settings

**Goal.** The player chooses the overlay's corner, margin and opacity in Settings, and the choice is remembered.

**Depends on:** step 07 (`OverlaySettings`). Step 08 is not required.

## Decided
- **Settings page section "Overlay":**
  - **Corner:** four choices, Top left, Top right (default), Bottom left and Bottom right, shown as a row of pills or buttons like the other Settings choices.
  - **Margin:** a number field in pixels from 0 to 200, default 24. Out-of-range values are clamped.
  - **Opacity:** 60–100%, default 100%. Use the component library's slider if it has one, otherwise a number field with "%".
- **Saved** in a small JSON file `overlay` in the data folder, written the same way `appearance` is (`Db::appearance` / `save_appearance`). A missing or unreadable file means the defaults.
- **Applying changes.**
  - A corner or margin change reopens the overlay if it is open, because layer-shell anchor and margin can't change after the window is created.
  - Opacity applies while the overlay is open.
- **Opacity: the open question, decided ahead of time.**
  - Open the overlay with `window_background: WindowBackgroundAppearance::Transparent` and have the overlay paint its own background with alpha = opacity.
  - The catch: gpui-component's `Root` paints `cx.theme().tokens.background` across every window's surface (`WindowState::style` in the vendored `gpui-component-0.9.1/src/root.rs`), and the theme is shared by all windows.
  - Look for a supported way to stop that for the overlay window only, such as a per-window override, a plugin hook, or a documented option. **Do not patch or fork vendored crates.**
  - If there is no such way, ship the overlay opaque: leave out the opacity control and the `opacity` field, and say so in the commit message and in the README's Overlay decisions ("opacity dropped: gpui-kit paints an opaque window background").

## Read first
- `src/ui/settings.rs` and `settings_page`, for the layout and the existing pill choices.
- `src/db.rs`: `appearance`, `save_appearance`.
- `src/ui/overlay.rs`: `OverlaySettings`, `open`.
- The vendored `gpui-component-0.9.1/src/root.rs`, around `impl gpui_base::RootPlugin for WindowState`.

## Changes
1. Add `Db::overlay_settings() -> Result<OverlaySettings>` and `Db::save_overlay_settings(&OverlaySettings)` next to the appearance methods.
2. `Planner` loads the settings at start, keeps them, and passes them to `overlay::open`.
3. Add the Settings section. Ids: `("overlay-corner", n)`, `overlay-margin`, `overlay-opacity`.
4. Opacity, as decided.
5. **Removal checklist** in step 07: add the Settings section, the two `Db` methods and the settings field on `Planner`. The `overlay` file left in data folders is harmless.
6. **`CLAUDE.md`:** in the paragraph about the data folder, mention the `overlay` file.

## Tests
- `Db`: settings round-trip through the file. A missing file gives the defaults, and a corrupt file gives an error, as `appearance` does today.
- UI (in the `// ---- overlay ----` section): choosing Bottom left in Settings saves it. With the overlay open, the overlay is reopened; check that the new window handle differs from the old one.

## Manual check (KDE Wayland)
Move the overlay to each corner, change the margin, and, if opacity shipped, set 60% over a running game. Take screenshots.

## Done when
- The README's done rule is met.
