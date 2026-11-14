# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

Recorded as `web` because Impeccable's platform values describe design language, and this app's is web-derived (shadcn-like `gpui-component` library, HTML reference mockup). The product truth is a native desktop app built with GPUI via the `gpui-kit` facade — no browser involved.

## Users

One player per copy, using the app in short glances next to a running game — the player who records the game's items and recipes and then works off the Requirements screen while playing. Today that player is the author; **other players are an intended future audience**, so first-run, empty states, self-explanation, and packaging are real future work, not polish to skip forever. There is no multi-user or networked use.

## Product Purpose

Game Planner records what a game's items are made of and works out what the player still has to gather and craft to reach their goals. It exists because crafting planners that live inside a spreadsheet or a wiki don't net inventory, don't share surplus between goals, and can't be updated mid-session. Success: while the game has the player's attention, any number on the Requirements screen is findable in under a second and the next possible action is obvious.

## Positioning

Unlike hosted trackers tied to one fixed game (the reference seelie.me plans a single game's content), the player records *any* game themselves and the calculation is exact: crafts are whole (never fractional), surplus is shared across plans, inventory is netted at every level of the recipe expansion, and recipe loops are detected and treated as gathering. All data stays local — SQLite plus image folders on the player's machine, no accounts, no network.

## Operating Context

- Native desktop app: Rust (edition 2024), GUI in GPUI through the `gpui-kit` facade (never raw `gpui::*` paths; `gpui-fast` engine feature), storage in SQLite via `sqlx` with embedded migrations.
- Developed on Linux (KDE Wayland); **macOS and Windows are intended targets too** — don't hard-code Linux-only conventions (fonts, shortcuts, window behavior) beyond what GPUI already abstracts.
- Single desktop window around 1440×900; panes scroll inside, the window itself doesn't scroll.
- Used mid-game in glances: the Requirements screen stays open while the player plays. Not a dashboard, not a marketing page; it may have character as a companion to games, never at the cost of reading speed.
- Data lives under `$XDG_DATA_HOME/game_planner` (`game_planner.db` + `images/<game id>/`); debug builds use `game_planner.dev` so real data is never touched in development. Image paths in the database are relative to that folder.
- Terminology is governed by `CONTEXT.md`; its terms (Game, Item, Recipe, Input, Output, Craft, Raw material, Inventory, Plan, Target, Reached, Ready, Requirements, Surplus, Default recipe, Recipe loop) are binding in code, UI text, and docs, and its _Avoid_ words are banned.

## Capabilities and Constraints

- Five areas: Game library (home); inside a game — **Requirements** (Gather and Craft, combined across all plans), **Plans**, **Items** (searchable; recipes, default recipe, where used), **Inventory**.
- The calculation (pure, in `src/calc.rs`; its tests are the specification of the rules): whole crafts only; demand summed across plans before crafts are counted so surplus is shared; inventory netted at every level; recipe loops stop at the repeated item and become gathering; inventory assigned to plans oldest first; a Reached plan stays until deleted; Ready means all of a craft's inputs are owned.
- Amounts are fractional-capable `f64`/`REAL` compared with a small epsilon (e.g. 0.125 Coal per craft); timestamps are Unix milliseconds and only order plans and recipes.
- Every action follows one path: call a `Db` method, report errors as notifications, then reload and recalculate from the database — nothing is patched in memory.
- Committed future capability: **import/export of game data** in and out of the local store (format undecided).
- Rendering constraints from the approved design spec: UI must be buildable from boxes, text, images, borders, flat fills, and custom fonts — no blur, no gradients that carry meaning, no SVG illustration of real objects.
- The renderer smooths enlarged images, so small pictures (pixel art) are enlarged once, without smoothing, at import time.

## Brand Commitments

- Name: **Game Planner**.
- Approved visual direction: layout "Grid and panel" with the **graphite** colour scheme. Reference: `design/design-demos/dir-3-graphite.html`; decision and rejected alternatives recorded in `design/direction-approved.md` and `design/directions*.md`.
- Explicit rejection: colour similarity to the seelie.me reference's navy, violet, and gold.

## Evidence on Hand

- Sample game **Hollowdeep** (13 items, 11 recipes, 5 plans, including all the notable calculation cases) in `design/design-demos/shared.js` and the `seed_sample_game` test — use it instead of inventing data.
- Design reference screenshots: `design/refs/seelie-*.png` (the user-supplied reference; colour must differ from it), `design/design-demos/shots/`.
- Domain sketch `erd.eraserdiagram`; the authoritative schema is `migrations/0001_init.sql` (text UUID ids, REAL amounts, integer timestamps).
- Absences: no testimonials, users, metrics, or marketing claims exist — a personal tool so far. Future work must not fabricate them.

## Product Principles

1. Reading speed outranks everything: numbers findable in under a second, the next action obvious; character is welcome only where it doesn't slow a glance.
2. The calculation is the product: its tests in `src/calc.rs` are the specification, and the UI never shows a number the calculation didn't produce.
3. One language everywhere: `CONTEXT.md` terms in code, UI text, and conversation; _Avoid_ words never appear.
4. Any game, all local: the player records the game; data belongs to the player's machine, with no per-game assumptions and no network dependence.
5. One path per change: every write goes through `Db` and a full reload/recalculate, so the screen always reflects the database.

## Accessibility & Inclusion

Binding text standard from the design spec: body text ≥ 14px, labels ≥ 12px, contrast ≥ 4.5:1.
