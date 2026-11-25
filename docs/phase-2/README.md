# Phase 2 plan

What phase 2 delivers is in [`../phase-2-requirements.md`](../phase-2-requirements.md). This folder breaks it into steps. The decisions below were made with the owner on 2026-11-26; do not reopen them while implementing a step.

## How to use this folder

Give an agent one step at a time, in order:

> Implement `docs/phase-2/03-inventory-search.md`.

Each step assumes the steps before it are done and committed.

## Rules for every step

- Before changing anything, read `CLAUDE.md`, `CONTEXT.md`, this README and the step file. Read `docs/gpui-kit-cheatsheet.md` before writing UI code.
- Use the glossary's terms (Project, Template, Finish, Overlay, Plan, Target, Reached…) in code, UI text, tests and commit messages. Do not use the words listed under _Avoid_.
- Every action follows the existing path: a `Db` method, then `finish(result, window, cx)`, which reloads. Nothing is patched in memory.
- Schema changes go in a new numbered file in `migrations/`. Never edit an applied migration.
- Stay inside the step. If the planned design turns out not to work, stop and report what you found. Do not improvise a different design.
- **Done when:**
  - `cargo test` passes.
  - `cargo clippy --all-targets` adds no warnings.
  - `CONTEXT.md` and `CLAUDE.md` are updated if the step changed vocabulary or architecture.
  - The step is a single commit on the current branch, with a message like `feat: amount control on inventory tiles`.

## Steps

| # | File | Delivers |
|---|---|---|
| 01 | [01-amount-control.md](01-amount-control.md) | A shared `[−][amount][+]` control, used first on Inventory tiles |
| 02 | [02-requirements-tile-fields.md](02-requirements-tile-fields.md) | Gather and craft tiles set the owned amount directly |
| 03 | [03-inventory-search.md](03-inventory-search.md) | Search and a "Needed only" filter on the Inventory page |
| 04 | [04-projects-templates-data.md](04-projects-templates-data.md) | Schema, domain types and `Db` methods for projects and templates (no UI) |
| 05 | [05-plans-tab-projects.md](05-plans-tab-projects.md) | Projects on the Plans page: make, add to, rename, delete, finish, save as template |
| 06 | [06-templates-tab.md](06-templates-tab.md) | The Templates tab: list and editor |
| 07 | [07-overlay-window.md](07-overlay-window.md) | The overlay window, read-only, toggled from the sidebar |
| 08 | [08-overlay-actions.md](08-overlay-actions.md) | Changing amounts, crafting, searching, collapsing and locking in the overlay |
| 09 | [09-overlay-settings.md](09-overlay-settings.md) | Overlay corner, margin and opacity in Settings |
| 10 | [10-global-shortcut.md](10-global-shortcut.md) | *Optional:* a system-wide shortcut that toggles the overlay |

## Decisions

### Amounts (steps 01–03)
- An amount field holds the amount **owned**. Typing a number and pressing Enter, or leaving the field, sets inventory to that number. This applies on Inventory tiles, Gather tiles and Craft tiles alike. The gather tile's **Add** button and its add-to-inventory behaviour go away.
- − and + save immediately. A click changes the amount by 1, or by 10 with Shift held. The amount never goes below 0, so − is disabled at 0. Fractions stay fractions (5.625 + 1 = 6.625).
- Craft tiles keep **Craft all N**. The right-hand craft panel keeps **Craft 1 / Craft all** and gets no amount field.
- The Inventory page gets its own search field (name contains the query, ignoring case) and a **Needed only** toggle, which shows only items that appear in the requirements.

### Projects and templates (steps 04–06)
- A **Project** is a named group of plans. A plan is in at most one project or in none. Deleting a project deletes its plans.
- A **Template** is a named, reusable list of items and amounts:
  - It has one line per item, and every amount is above 0.
  - Names are unique within a game, ignoring case. Project names need not be.
  - Deleting a template leaves the projects made from it alone.
  - An item listed in a template cannot be deleted.
- **Making a project** happens in a list editor (name plus item and amount lines), which creates one plan per line inside the project:
  - **New project** starts the editor empty.
  - **From template** fills it from a template and adds a **×N** field. N multiplies every amount, and the default name becomes "House ×3".
  - The project gets a copy of the lines. Afterwards the template and the project never change each other.
- **After it exists**, a project can gain plans (the Add plan form has a Project choice), lose single plans, be renamed, be deleted, or be saved as a new template.
  - Not in this phase: moving plans between projects or out of one, and editing a plan's target.
- **Finish** is enabled when every plan of the project is reached:
  - You confirm in a dialog that lists what leaves inventory.
  - It takes each plan's target out of inventory and deletes the project with its plans.
- **Where they appear:**
  - The Plans page gets tabs **Plans | Templates**. The Plans tab shows one section per project, oldest first, with loose plans last under "No project".
  - The project editor opens as a dialog. The Templates tab shows the same editor in its right-hand panel.
  - The Requirements page stays combined across all plans; only its plan strip is grouped by project.
- **Game files do not change.** Exporting with progress writes a project's plans as loose plans, and templates are not exported. A Replace import discards the existing game's projects and templates with everything else.

### Overlay (steps 07–10)
- The **Overlay** is a separate small window that stays above the game:
  - **Linux on Wayland:** layer-shell, on the Overlay layer, anchored to a corner. Where the compositor lacks layer-shell (GNOME), it falls back to a normal window.
  - **X11, Windows and macOS:** `WindowKind::PopUp`.
  - Only Linux on KDE Wayland can be tested, and each step says so.
- It follows the game open in the main window. Closing the main window closes the overlay and quits.
- **Contents:**
  - project and loose-plan progress
  - one line per Gather and Craft requirement, with the amount control and **Craft** on ready lines
  - a search box to change any item's amount
  - Items, recipes and plans are not edited here.
- **Controls:**
  - It is interactive by default.
  - On Wayland, a **lock** makes clicks pass through to the game, except on the lock button itself.
  - It can be collapsed to a slim bar.
  - It is opened and closed from a sidebar button; a global shortcut is the optional step 10.
- **Settings:** corner (default top-right), margin and opacity (60–100%). If gpui-kit cannot make the overlay translucent without patching it, the overlay ships opaque and the opacity setting is dropped.
- **It must be easy to remove.** All overlay code goes in `src/ui/overlay.rs` (and `src/ui/overlay/` if it grows). It reads the main window's `Planner` and calls the main window's existing actions; the `Planner` is not restructured for it. Step 07 keeps a "How to remove the overlay" checklist, and later steps extend it.

### Tutorial
- Its steps stay the same, with no project or overlay steps added. Its Plans step must still highlight the Add plan button after the Plans page changes.

## Not in phase 2

- **Seeding Minecraft: Java Edition items and recipes.** This was deferred; no research notes are kept.
- Moving plans between projects, and editing a plan's target.
- Projects and templates in game files.
- A per-project filter on the Requirements page.
- "Any of these items" recipe inputs.
