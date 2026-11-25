# Step 01: Amount control

**Goal.** Build one reusable control, `[−][amount][+]`, that sets how much of an item the player owns, and use it on the Inventory page's tiles. Step 02 puts it on the Requirements tiles and step 08 on the overlay.

**Depends on:** nothing.

## Decided
- The field shows the owned amount, formatted with `fmt_amount`.
  - Typing a number and pressing Enter, or leaving the field, sets inventory to that number. `commit_inventory` already does this.
  - Text that isn't a number puts the stored amount back.
  - A negative number saves as 0.
- − and + change the owned amount at once, with no Enter needed.
  - A plain click changes it by **1**; a click with **Shift** held changes it by **10**.
  - The result never goes below 0: − is disabled when the amount is 0, and Shift − from 4 gives 0.
  - Fractions are kept: 5.625 + 1 = 6.625. Round the result to 6 decimal places, as `Db::craft` does, so repeated clicks don't drift.

## Read first
- `src/ui/mod.rs`:
  - `ensure_fields`, which creates the per-item `inventory_fields`
  - `commit_inventory`, `reload` (it re-syncs the field text after every action), `finish`
  - the helpers at the bottom: `well()`, `button()`
- `src/ui/pages.rs`: `inventory_page`.
- `src/ui/requirements.rs`: `gather_tile`. Its small "Add" button is the size and style to use for − and +.
- `src/db.rs`: `set_inventory` (clamps to 0) and `craft` (the rounding).
- The `TESTING.md` of the vendored gpui-kit (path in `CLAUDE.md`), on clicking with modifiers held. Its section "Inspect accessibility targets and individual events" covers configurable clicks and `change_modifiers`.

## Changes
1. Add `Planner::change_inventory(&mut self, item_id, delta: f64, window, cx)` in `src/ui/mod.rs`, next to `commit_inventory`. It computes `max(owned + delta, 0)` with the rounding above, then calls `db.set_inventory` and `finish`.
2. Add a method `Planner::amount_control(&self, prefix: &'static str, item_id, cx) -> Div` beside the other shared pieces in `src/ui/mod.rs`. It renders a `well()` containing, in order:
   - **−**: a small button with id `(format!("{prefix}-minus"), key)`. It is disabled when the owned amount is ≤ 1e-9.
   - **The field:** the item's `inventory_fields` entry, as `Input::new(field).id(format!("{prefix}-{item name}")).appearance(false)`. It sits in a `flex_1().min_w_0()` wrapper so it shrinks inside a 132 px tile.
   - **+**: a small button with id `(format!("{prefix}-plus"), key)`.
   - The click handlers read Shift from the `ClickEvent`'s modifiers; check the exact accessor in the vendored gpui-fast source. They call `change_inventory` with ±1 or ±10.
   - Call `.test_support()` last on each button, as `CLAUDE.md` explains, so tests can find them by id.
3. In `inventory_page`, replace the bare `well().child(Input::new(field)…)` with `self.amount_control("inv", id, cx)`.
4. Update the Inventory page note: "Type an amount and press Enter, or use − and + (hold Shift for 10). The requirements recalculate from what you own."

## Tests (in `src/ui/tests.rs`)
The fixture owns 3 Stick and 5 Coal. Open the Inventory page by clicking its sidebar item (`"Inventory"`) and render a frame.
- Clicking `inv-plus` on Stick makes Stick 4.
- A Shift-click on `inv-plus` on Stick makes it 13.
- A Shift-click on `inv-minus` on Coal makes it 0, not −5, and then `inv-minus` on Coal does nothing.
- Typing `7` into `inv-Stick` and pressing Enter makes Stick 7.

Check each result with `fx.db.load_game(fx.game)` and `owned(..)`, as the existing tests do.

## Done when
- The README's done rule is met.
- On the Inventory page each tile shows `[−][amount][+]` and behaves as decided. Check this in the running app on the sample data (commands in `CLAUDE.md`) with a screenshot, since clicks can't be automated on the desktop.
