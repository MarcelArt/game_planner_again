# Step 06: Templates tab

**Goal.** The player can create, edit and delete templates, and make a project from one, on the Plans page's **Templates** tab.

**Depends on:** step 05 (the tabs and `ItemListEditor`).

## Decided
- **Layout.** It mirrors the Items page.
  - **Left:** the game's templates as a list, by name. Each entry shows its name and "N items", and the selected one is highlighted. Above the list there is a name field and a **New template** button, which creates an empty template with that name and selects it.
  - **Right:** the selected template in an `ItemListEditor` with its name and lines. Below it:
    - **Save template** saves the name and lines together (`rename_template` + `set_template_lines`).
    - **Make project** opens the From template dialog from step 05 with this template already chosen.
    - **Delete** (kind `Danger`) asks "Delete the template House? Projects made from it stay."
- **Saving.** Edits are a draft until **Save template**, as in the recipe composer. Switching to another template drops unsaved edits; no prompt is needed.
- **Empty states.**
  - With no templates: "No templates yet. A template lists the items and amounts something needs, so you can make projects from it again and again."
  - A template with no lines cannot be used for a project: **Make project** is disabled.
- **Errors.** A name clash or a 0 amount is reported by the `Db` error through `finish`'s notification.

## Read first
- `src/ui/pages.rs`: `items_page` and `item_panel`, for the list-and-panel layout.
- `src/ui/list_editor.rs` from step 05.
- The step 05 dialog `project_from_template_dialog`.

## Changes
1. `Planner`: add `sel_template: Option<Uuid>`, a `new_template` field and a `template_editor: Entity<ItemListEditor>`. Load the selected template into the editor when the selection changes and after `reload` if the template still exists, the way `sync_item_panel` works for items.
2. Render the Templates tab in `src/ui/pages.rs`, or in a new `src/ui/templates.rs` if `pages.rs` gets too long.
3. Ids:
   - `template-new-name`, `template-new`, `("template", key)` for list entries
   - `template-save`, `template-make-project`, `template-delete`

## Tests (in `src/ui/tests.rs`)
- **New template:** creating "Camp", adding Stick 2 and Coal 3, and pressing **Save template** stores those lines.
- **Editing:** changing an amount and saving replaces the lines.
- **Deleting:** deleting a template leaves a project made from it in place.
- **Name clash:** a second template named "camp" is refused, and the first is unchanged.
- **Make project:** it opens the dialog with the template chosen, and creating gives a project with its lines.

## Done when
- The README's done rule is met.
- Add a "Mine camp" template to `seed_sample_game` and check a screenshot.
