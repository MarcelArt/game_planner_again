# Step 08: Actions in the overlay

**Goal.** From the overlay, the player changes owned amounts, crafts, finds any item, collapses the overlay to a bar, and on Wayland locks it so clicks reach the game.

**Depends on:** step 07.

## Decided
- **Requirement lines.** Every gather and craft line gets the amount control `[−][amount][+]`: same behaviour as step 01, Shift for 10, never below 0. Ready craft lines also get **Craft N**, which performs the crafts inventory covers, like the tile's **Craft all N**.
- **Search box** under the header, placeholder "Find an item". While it holds text, the requirement lines are replaced by every item whose name contains it (ignoring case), each with the amount control. Clearing it brings the lines back.
- **Collapse.** A header button shrinks the overlay to a slim bar about 36 px high showing the game title, "N ready" and an expand button.
  - Check whether gpui-fast can resize a layer-shell surface at run time.
  - If it can't, reopen the overlay at the new size; it is cheap to reopen.
- **Lock** (layer-shell overlay only; the button is hidden otherwise). A header button locks the overlay:
  - **While locked:** `window.set_input_region(Some(&[lock_button_bounds]))`, so every click except on the lock button passes through to the game, and the header shows "Locked".
  - **Unlocking:** clicking the lock button again calls `set_input_region(None)`.
  - **Placement:** give the lock button a fixed position and size in the header, so its bounds are known without measuring.
- **Keyboard.** The overlay takes the keyboard only when the player clicks into a field; that is layer-shell `OnDemand`. Nothing tries to hand focus back to the game.
- **Actions go through the main window.** The overlay calls the `Planner` action inside the main window's context, for example `main.update(cx, |_, window, cx| planner.update(cx, |p, cx| p.change_inventory(id, d, window, cx)))`. That way errors appear as notifications in the main window, and `finish`/`reload` run as usual.
  - Updating another window from an overlay event handler should be allowed.
  - If GPUI refuses, defer the call with `cx.defer`.
- **The overlay's own fields.** The overlay keeps its own `InputState` per shown item. The main window may be rendering the same item's `inventory_fields` at the same moment, and an `InputState` may be rendered by only one `Input` at a time.
  - When the planner changes, the overlay re-syncs its field texts, except a field that has focus.
  - Enter or leaving the field commits the value.

## Read first
- `src/ui/overlay.rs` from step 07.
- `src/ui/mod.rs`: `amount_control`, `change_inventory`, `commit_inventory`, `craft`.
- `Window::set_input_region` in the vendored gpui-fast `src/window.rs`; it is Wayland-only, and other platforms do nothing.

## Changes
1. **Split the amount control.** Extract the rendering part of `amount_control` into a free function that takes the field, the owned amount and the − and + click handlers. `Planner::amount_control` and the overlay both use it.
2. **Add `Planner::set_owned(item_id, amount, window, cx)`.** `commit_inventory` calls it, and so does the overlay.
3. **Overlay:**
   - per-item fields, created lazily and synced on planner notify
   - the search field
   - collapsed and locked state
   - the header buttons, with ids `overlay-collapse`, `overlay-lock` and `overlay-close`
   - control ids with prefix `ov` (`ov-plus`, `ov-minus`, `ov-<item name>`) and `("ov-craft", key)`
4. **Removal checklist** in step 07: add the `Planner::set_owned` note. It can stay, since `commit_inventory` uses it.

## Tests (in the `// ---- overlay ----` section of `src/ui/tests.rs`)
- `ov-plus` on the Coal line makes Coal 6. Render the main window afterwards: its Inventory field for Coal shows 6.
- Typing `2` into `ov-Stick` and pressing Enter makes Stick 2.
- `ov-craft` on Torch performs the crafts inventory covers (with the fixture: 3, leaving Stick 0 and Coal 2).
- Searching "sti" in the overlay shows a Stick control. Clearing the search shows the requirement lines again.
- Collapse hides the lines and expand shows them.
- In tests, which run without layer-shell, `overlay-lock` is not rendered.

## Manual check (KDE Wayland)
With a game running behind the overlay:
- Change an amount and craft from the overlay.
- Lock it, and confirm that clicks pass through to the game except on the lock button.
- Unlock it.
- Collapse and expand it.
- Take screenshots of the expanded, collapsed and locked states.

## Done when
- The README's done rule is met.
