# Game Planner — six direction cards

Roulette second: 28 → web style #9 (pixel-game side-scroller), East Asian style #5 (experimental character poster).

## 1 · Checklist — real reference: Things 3 (Cultured Code, Apple Design Award 2017)

- **Source**: culturedcode.com/things, screenshot in `refs/things.jpg` (seen).
- **Borrowed**: pale grey sidebar against a white content pane; one bold title with a progress ring beside it; section headings in a single accent colour sitting on a hairline rule; rows that are a round control plus one line of text plus quiet grey metadata; a narrow reading column with generous side margins rather than a full-width table.
- **Hero material**: the round control itself. A craft row's ring fills as crafts are performed, so the list is its own progress chart.
- **Skeleton**: sidebar (220px) + one centred column (max 720px) of grouped rows.
- **Boldness spent on**: refusing the table. Requirements is a to-do list, one line per row.
- **Form from content**: the player works the list top to bottom; gather and craft rows are things to tick off.

## 2 · Ledger — real reference: Linear

- **Source**: linear.app/features, screenshot in `refs/linear.jpg`. Only the marketing page was seen, not the app UI itself.
- **Borrowed**: near-black surface with hairline borders one step lighter; grey text in three steps with white only for the thing that matters; a small coloured dot as the only status signal; dense rows grouped under sticky group headers.
- **Hero material**: tabular numerals in a monospace face, right-aligned in columns.
- **Skeleton**: sidebar + full-width grouped table + right inspector for the selected row.
- **Boldness spent on**: density. Twenty rows visible with no scrolling.
- **Form from content**: Requirements is arithmetic; columns let the player check it (need, have, missing, crafts, makes, surplus).

## 3 · Hotbar — roulette web #9: pixel-game side-scroller

- **Source**: `assets/style-specimens/web/09-pixel-side-scroller.jpg` (seen). Fidelity 70%: hand-drawn pixel illustration is replaced by small pixel icons built from squares.
- **Borrowed**: the HUD strip along the top with pixel numerals; flat saturated colour fields; the double-bordered dialog box at the bottom carrying the action buttons; hard 2px borders with no radius.
- **Hero material**: pixel item icons in inventory slots with a count in the corner.
- **Skeleton**: top HUD bar with tabs (no sidebar) + slot grid + bottom dialog box describing the selected slot and holding Craft 1 / Craft all.
- **Boldness spent on**: showing Requirements as slots rather than rows. Pixel type only for numerals and headings; all sentences in a normal sans.
- **Form from content**: the player already reads slot grids with corner counts all day in the game.

## 4 · Tiers — roulette East Asian #5: experimental character poster

- **Source**: `assets/style-specimens/east/05-hanzi-experimental-poster.jpg` (seen). Fidelity 85%; the hand-built glyph is not used.
- **Borrowed**: one thing taken apart into numbered parts shown side by side; paper white, black and a single vermilion; heavy black sans headings; hairline rules above each numbered part.
- **Hero material**: the numbered columns themselves, black rules and large column numerals.
- **Skeleton**: top bar (no sidebar) + columns left to right: Gather, then Craft step 1, 2, 3 by depth, then Plans. Each column is a stack of cards.
- **Boldness spent on**: laying "inputs first" out in space instead of in list order.
- **Form from content**: a recipe is a decomposition; the plans on the right come apart into raw materials on the left.

## 5 · Field unit — best designer, bold: Teenage Engineering

- **Source**: their design thinking, not a specific product: an instrument shows few things, large, in fixed positions; one signal colour; labels are small and technical, readouts are huge.
- **Borrowed**: light grey panel, monospace throughout, one orange, fixed regions that never reflow.
- **Hero material**: very large numerals as readouts.
- **Skeleton**: two fixed panes side by side (Gather | Craft) with a strip of plans along the bottom.
- **Boldness spent on**: numeral size. The missing amount is the biggest thing in each row.
- **Form from content**: the player glances away from the game for one second to read one number.

## 6 · Plain — best designer, quiet: Jasper Morrison ("super normal")

- **Source**: his principle that the best object is the one you stop noticing.
- **Borrowed**: nothing added. The component library's stock theme, used with care: spacing, alignment and wording do all the work.
- **Hero material**: none by intent; uploaded item icons carry all the colour.
- **Skeleton**: sidebar + tabs + stock table.
- **Boldness spent on**: restraint. Also the cheapest to build and to keep in step with the library.
- **Form from content**: the game is the loud thing on screen; the planner stays quiet beside it.
