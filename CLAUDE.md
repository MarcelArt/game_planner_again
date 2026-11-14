# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project state

A Rust (edition 2024) desktop GUI app built on GPUI via the `gpui-kit` facade crate. The code is currently only the hello-world window in `src/main.rs`; the intended product is a crafting/production planner for games, whose data model is sketched in `erd.eraserdiagram`. There are no tests, README, or persistence layer yet.

## Commands

- `cargo run` — build and launch the app window
- `cargo check` / `cargo clippy` — fast validation; prefer these over a full build, since the GPUI dependency tree is large and cold builds are slow
- `cargo test` / `cargo test <name>` — run all tests / a single test (none exist yet)

## gpui-kit

`gpui-kit` is the only dependency and re-exports the whole GPUI stack. Never add `gpui`, `gpui-component`, or other `gpui-*` crates directly, and never write `gpui::` paths — everything goes through `gpui_kit`:

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

To write UI tests, add `gpui-kit` under `[dev-dependencies]` with the `test-support` feature (plus `gpui-fast`, to match the main dependency) and use `#[gpui_kit::test]` rather than `#[test]`. In test modules import Kit types explicitly instead of `use gpui_kit::*`, because the glob brings in GPUI's `test` macro and shadows Rust's `#[test]`. The headless-window helpers live in `gpui_kit::test`; see `TESTING.md` in the vendored crate.

## Domain model

`CONTEXT.md` is the glossary: use its terms (and avoid the ones it lists under _Avoid_) in code, UI text and conversation.

From `erd.eraserdiagram` (Eraser ERD syntax; all ids are UUIDs, quantities are decimals):

- `games` — a game, with title and thumbnail
- `items` — belong to a game; every other table hangs off an item
- `recipes` — produces `output` units of an item
- `recipe_details` — an input line of a recipe (`recipe_id`): `input` units of an item
- `inventories` — `amount` of an item currently owned
- `plans` — `target` amount of an item to reach

A recipe therefore reads as: `recipes.output` units of `recipes.item_id`, made from each of its `recipe_details` rows (`input` units of that row's `item_id`).
