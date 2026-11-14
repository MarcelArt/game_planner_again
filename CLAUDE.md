# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project state

Game Planner: a Rust (edition 2024) Linux desktop app that records what a game's items are made of and works out what the player still has to gather and craft. GUI is GPUI via the `gpui-kit` facade crate; storage is SQLite through sqlx.

- `CONTEXT.md` is the glossary. Use its terms (and avoid the ones under _Avoid_) in code, UI text and conversation.
- `design/` holds the approved design. `design/design-demos/dir-3-graphite.html` is the reference the UI follows (open it in a browser); `design/direction-approved.md` records what was chosen and why. `design/design-demos/round-1/` and the other `dir-*` files are rejected alternatives.
- `docs/gpui-kit-cheatsheet.md` is an API reference for gpui-kit compiled from the vendored source. Read it before writing UI code.

## Commands

- `cargo run` — build and launch the app
- `cargo check` / `cargo clippy --all-targets` — fast validation
- `cargo test` / `cargo test <name>` — all tests / one test. Test builds enable gpui-kit's `test-support` feature, so the first run recompiles the GPUI stack.
- Try the app on sample data without touching real data:
  ```
  SEED_DIR=/tmp/gp/game_planner cargo test seed_sample_game -- --ignored
  XDG_DATA_HOME=/tmp/gp GAME_PLANNER_START=requirements cargo run
  ```
  `GAME_PLANNER_START` (`requirements|plans|items|inventory`) opens the first game on that page; `SEED_ICONS=<dir>` adds `<item_name>.png` icons while seeding.
- To look at the running app on this machine (KDE Wayland): `spectacle -b -a -n -o out.png` captures the active window. Clicks cannot be automated there; cover interactions with UI tests instead.

## Architecture

Real data lives in `$XDG_DATA_HOME/game_planner` (`~/.local/share/game_planner`): `game_planner.db` plus `images/<game id>/`. Image paths in the database are relative to that folder.

- `src/domain.rs` — plain types; `GameData` is one game's items, recipes, inventory and plans as loaded.
- `src/calc.rs` — the calculation (`requirements`, `plan_progress`, `craftable`). Pure, no database or UI; its tests are the specification of the rules (whole crafts, shared surplus, inventory netted at every level, recipe loops, oldest plan first).
- `src/db.rs` — `Db`: every read and write, plus image import. sqlx is async and needs Tokio while the UI runs on GPUI's executor, so `Db` owns a one-thread Tokio runtime and exposes blocking methods; do not make UI code async for database access. Schema is in `migrations/` (embedded with `sqlx::migrate!`; add a new numbered file, never edit an applied one). Amounts are `f64`/`REAL` compared with a small epsilon; timestamps are Unix milliseconds and only order plans and recipes.
- `src/ui/` — one view, `Planner` (`mod.rs`), holding the open game's `GameData` and the calculated `Requirements`; `requirements.rs` and `pages.rs` render the five pages as methods on it. Every action follows the same path: call a `Db` method, then `finish(result, window, cx)`, which reports an error as a notification and calls `reload` to re-read the game and recalculate. Nothing is patched in memory.
- Colours come from `ui::pal` (the Graphite palette), also applied to the component library's theme in `apply_theme`. Buttons, pills and wells are the helpers at the bottom of `ui/mod.rs`, not the library's `Button`.
- Text fields are `InputState` entities owned by `Planner`. Per-item fields (inventory, gathered amount) are created lazily in `ensure_fields`. An `InputState` must be rendered by only one `Input` at a time.
- The renderer always smooths enlarged images, so `Db::import_image` enlarges small pictures (pixel art) once, without smoothing, when they are imported.

## gpui-kit

`gpui-kit` is the only GPUI dependency and re-exports the whole GPUI stack. Never add `gpui`, `gpui-component`, or other `gpui-*` crates directly, and never write `gpui::` paths — everything goes through `gpui_kit`:

| Path | What it is |
| --- | --- |
| `gpui_kit::*` | GPUI core (`div()`, `Render`, `Window`, `Context`, `WindowOptions`, …) |
| `gpui_kit::component::*` | Styled component library (gpui-component, shadcn-like), e.g. `component::button::{Button, ButtonVariants}` |
| `gpui_kit::assets::Assets` | Default icon/font assets, passed to `.with_assets(...)` |
| `gpui_kit::base` | gpui-base |

The `gpui-fast` feature is enabled in `Cargo.toml`, which swaps the engine to GPUI Fast behind the same imports. App code must not depend on which engine is selected.

App bootstrap order, as in `src/main.rs`: `application().with_assets(assets::Assets).run(|cx| { init(cx); open_window(...) })`. `init(cx)` must be called before any component is rendered.

Use `gpui_kit::actions!` (not GPUI's own macro) to define actions; the upstream macro emits `gpui::` paths that do not resolve through the facade.

This crate is niche and changes quickly — do not rely on memory for its API. Read the vendored source and docs instead:

- `~/.cargo/registry/src/*/gpui-kit-0.9.1/` — `README.md`, `TESTING.md`, `src/lib.rs`
- `~/.cargo/registry/src/*/gpui-component-0.9.1/` — component implementations

### Testing UI

UI tests live in `src/ui/tests.rs`: they open the real `Planner` window on a temporary database and drive it with clicks and keys. Use `#[gpui_kit::test]` rather than `#[test]`, and import Kit types explicitly instead of `use gpui_kit::*`, because the glob brings in GPUI's `test` macro and shadows Rust's `#[test]`. The helpers are in `gpui_kit::test` (see `TESTING.md` in the vendored crate).

A hand-built element is only findable by its id after `.test_support()`, and in test builds that call changes the element's type, so put it last in the chain, where the element is handed to `.child(..)`. Library widgets such as `Input` are findable by their `.id(..)` alone.

## Domain model

`erd.eraserdiagram` is the original sketch (Eraser ERD syntax); `migrations/0001_init.sql` is the schema actually in use. They differ in small ways: ids are text UUIDs, amounts are `REAL`, timestamps are integers.

- `games` — a game, with title and thumbnail
- `items` — belong to a game; every other table hangs off an item
- `recipes` — makes `output` of an item per craft; one per item has `is_default`
- `recipe_details` — an Input of a recipe: `input` of an item per craft
- `inventories` — `amount` of an item owned, one row per item
- `plans` — `target` amount of an item, with an optional `note`; several plans may name the same item
