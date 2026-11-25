# Step 02: Gather and craft tiles set the owned amount

**Goal.** On the Requirements page, gather tiles and craft tiles get the amount control from step 01. The player types what they own, or nudges it with − and +, without going to the Inventory page.

**Depends on:** step 01.

## Decided
- **Gather tile.** The amount control shows the owned amount of the raw material, for example 60 of a needed 120. Typing 50 means the player now owns 50. The **Add** button, and adding the typed number to inventory, are removed. Covered tiles and recipe-loop tiles get the control too.
- **Craft tile.** The control shows the owned amount of the crafted item and sits above **Craft all N**, which stays. Changing the owned amount changes the tile's "Crafts" number. That is expected.
- **Right-hand panel.**
  - The craft panel keeps **Craft 1 / Craft all** and gets no field.
  - The gather panel's footer text changes from "Record what you gathered in the field on its tile. It is added to inventory." to: "Type the amount you own on its tile, or use − and +. The requirements recalculate from it."
- **Shared field.** The tiles use the same per-item `InputState` as the Inventory page (`inventory_fields`). The Requirements page and the Inventory page are never on screen at the same time, so the rule that an `InputState` is rendered by only one `Input` at a time still holds. `reload` already re-syncs these fields.

## Read first
- `src/ui/requirements.rs`: `gather_tile`, `craft_tile`, `gather_panel`, `TILE_W`.
- `src/ui/mod.rs`: `ensure_fields`, `gather_fields`, `add_gathered`, `amount_control` (step 01).
- `src/ui/tests.rs`: `typing_a_gathered_amount_and_pressing_enter_adds_it_to_inventory`.

## Changes
1. In `gather_tile`, replace the well, field and Add button with `self.amount_control("gather", id, cx)`.
2. In `craft_tile`, add `self.amount_control("craft", id, cx)` above the **Craft all** button.
3. Remove `Planner::gather_fields`, `add_gathered`, and the gather half of `ensure_fields`, along with anything that is then unused.
4. Rewrite the gather panel footer text as decided.
5. If the control doesn't fit in a 132 px tile (− and + about 26 px each, leaving room for at least 4 digits), widen `TILE_W` slightly rather than shrinking the buttons. Check with a screenshot of the running app on the sample data.

## Tests (in `src/ui/tests.rs`)
- Replace the test `typing_a_gathered_amount_and_pressing_enter_adds_it_to_inventory` with `typing_on_a_gather_tile_sets_the_owned_amount`: type `3` into `gather-Coal` and press Enter; Coal becomes 3, not 8.
- `plus_on_a_craft_tile_raises_the_owned_amount`: click `craft-plus` on Torch; Torch becomes 1, and after rendering, the Torch craft row asks for one craft fewer whenever the new amount crosses a multiple of 4. Assert on `Planner::req`, read through the planner entity.
- The existing tests `craft_all_performs_the_crafts_inventory_covers` and `craft_1_performs_one_craft` still pass unchanged.

## Done when
- The README's done rule is met.
- No **Add** button remains on the Requirements page. Gather and craft tiles show the owned amount with − and +, and the screenshot shows the tiles laid out cleanly.
