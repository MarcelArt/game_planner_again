# Game Planner — design spec (shared input for all directions)

## What it is

A native Linux desktop app (Rust, GPUI via gpui-kit, with the gpui-component library: shadcn-like sidebar, list, table, tabs, input, button, badge, dialog, progress). One player uses it next to a game to record what items are made of and to see what they still have to gather and craft. The HTML mockups stand in for that app: a single desktop window, 1440×900, no marketing page, no hero.

Everything the mockup shows must be buildable from boxes, text, images, borders, flat fills and custom fonts. No blur, no gradients that carry meaning, no SVG illustration of real objects.

## Vocabulary

Use the terms in `../CONTEXT.md` exactly, in all UI text: Game, Item, Recipe, Input, Output, Craft, Raw material, Inventory, Plan, Target, Reached, Ready, Requirements, Surplus, Default recipe, Recipe loop. Never the words listed there under _Avoid_.

## Screens

1. **Game library** (home): the games, each with a thumbnail (optional, so a fallback must look intentional), title, and counts. Add game.
2. Inside a game, four areas plus a way back to the library:
   - **Requirements** — the main screen, the one the player keeps open while playing. Combined across all plans.
     - *Gather*: raw materials. Per row: item, amount still missing, amount owned, the plans it serves, an inline field to record what was gathered (adds to inventory). Rows fully covered by inventory are shown as covered. A recipe-loop row is flagged and names the loop.
     - *Craft*: listed inputs-first. Per row: item, number of crafts, what that makes, surplus if any, its inputs with needed vs owned, Ready state, the plans it serves, "Craft 1" and "Craft all" (the latter labelled with how many crafts inventory covers now; both disabled at zero).
   - **Plans**: item, target, optional note, progress (inventory assigned oldest plan first), Reached state, delete. Add plan.
   - **Items**: searchable list with icon (optional, fallback must look intentional) and whether raw or crafted; selecting one shows its recipes (inputs → output per craft), which one is the default (switchable), and where the item is used.
   - **Inventory**: every item with an editable amount.

## Content

All three directions show the same content from `design-demos/shared.js` (`window.GP`): the fictional game Hollowdeep, 13 items, 11 recipes, 5 plans. Do not invent other data; do not hardcode numbers the API can give. Notable cases in the data, all of which must be visible on the Requirements screen:

- Torch: 10 crafts make 40, surplus 2; inventory covers 3 crafts now.
- Iron Ingot uses a fractional input (0.125 Coal per craft); Coal missing is 5.625.
- Copper Ingot is a recipe loop (Copper Ingot → Copper Block → Copper Ingot), so it is gathered.
- Oak Log is fully covered by inventory. Iron Pickaxe is not ready. The Oak Plank plan is reached.
- Torch is served by three plans, two of them Torch plans with different notes.

## Behaviour required in the mockup

Navigation between all screens works. On Requirements, Craft 1 / Craft all and the gather field really change inventory through `GP` and the screen recalculates. Switching an item's default recipe works. Item search filters. Console has no errors.

## Tone

A tool used in short glances while the game has the player's attention: numbers must be findable in under a second, the next possible action obvious. Not a dashboard, not a marketing page. It may have character — it is a companion to games — but never at the cost of reading speed.

## Fixed format

One HTML file per direction, 1440×900 viewport, the app fills the window (no page scroll; panes scroll inside). Body text ≥14px, labels ≥12px, contrast ≥4.5:1.
