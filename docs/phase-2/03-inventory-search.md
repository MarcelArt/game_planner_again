# Step 03: Search and "Needed only" on the Inventory page

**Goal.** Make an item on the Inventory page quick to find.

**Depends on:** step 01.

## Decided
- **Search field.** It goes at the top of the Inventory page and keeps only tiles whose item name contains the query, ignoring case and surrounding spaces. The Items page filters the same way (`items_page`).
  - It has its own text: searching on Items does not filter Inventory, and the other way round.
  - Its placeholder is "Search items".
- **Needed only toggle.** It goes next to the search field and is off by default.
  - When on, it shows only items that appear in the requirements: those with a gather row or a craft row, covered ones included. That is the set whose tiles already say "need N".
  - It lives in `Planner` and is not saved between launches.
- **Both together.** When both are used, an item must match both.
- **Empty results.** If nothing matches, show:
  - "No item matches the search." when the query is not empty;
  - otherwise "Nothing is needed yet. Add a plan first." when Needed only is on.
- **Hidden tiles.** Tiles that don't match are not rendered at all.

## Read first
- `src/ui/pages.rs`: `inventory_page`; `items_page` for the search filter and its empty text.
- `src/ui/mod.rs`: how `search` is created in `Planner::new`, `field_box` in `pages.rs`, and `Checkbox` in `export_dialog`, the pattern for the toggle.

## Changes
1. Add `inventory_search: Entity<InputState>` (placeholder "Search items") and `needed_only: bool` to `Planner`, and create the field in `Planner::new` the way `search` is created. Typing must re-render the page; follow whatever the Items search does to achieve that.
2. In `inventory_page`, put the page title above a row holding the search field (`field_box(Input::new(&self.inventory_search).id("inventory-search"))`, 260 px wide) and a `Checkbox` with id `"needed-only"` and label "Needed only".
3. Filter the tiles by both conditions, and show the empty text when none are left.

## Tests (in `src/ui/tests.rs`)
- Typing `sti` into `inventory-search` leaves the Stick tile rendered and removes Coal. Check by looking up the `Input` ids `inv-Stick` and `inv-Coal`, using whatever lookup gpui-kit's `TESTING.md` offers for "is this element rendered".
- With Needed only on, an item that no plan needs is not rendered. Add an unused item such as "Feather" to the fixture inside this test. Stick, Coal and Torch still are.

## Done when
- The README's done rule is met.
