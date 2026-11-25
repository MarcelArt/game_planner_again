# Step 05: Projects on the Plans page

**Goal.** The player can make projects, from scratch or from a template, and add plans to them. They can also rename, delete and finish projects, and save one as a template. The Requirements plan strip is grouped by project.

**Depends on:** step 04.

## Decided
- **Tabs.** The Plans page gets tabs **Plans | Templates**, and Plans is the default. Templates shows a placeholder until step 06. Model the tabs on the Requirements tabs (`ReqTab`).
- **Plans tab, top to bottom:**
  1. **The Add plan form.** It keeps Item, Target and Note and gains a **Project** select: "No project" (the default) followed by the game's projects. The **Add plan** button keeps its id `plan-add`, because the tutorial highlights it.
  2. **Buttons:** **New project**, and **From template**, which is disabled while the game has no templates.
  3. **One section per project, oldest first:**
     - **Header:** the project name, a pill "3 / 5 reached", **Finish** (kind `Go`, enabled only when all are reached), **Save as template**, **Rename**, and **Delete** (kind `Danger`).
     - **Rows:** its plans, using the existing plan row with its Delete button.
  4. **Loose plans**, last, in a section titled "No project", shown only if there are any.
- **The project editor** is a dialog:
  - **Fields:** a name, then a list of lines. Each line has an item select, an amount field and a remove button. An "add line" row holds an item select, an amount and **Add**.
  - **Adding an item that is already listed** adds to its amount, as the recipe composer on the Items page does.
  - **New project:** the editor starts empty, and **Create project** is the OK button.
  - **From template:** the dialog first asks which template (a select), then shows the editor filled with that template's name and lines, plus a **×N** field, a whole number defaulting to 1.
    - While the player hasn't typed a name of their own, the name follows N: "House" when N is 1, "House ×3" when N is 3.
    - The lines show the template's amounts, and N is applied when the project is created.
  - **Built once, used twice:** make the editor its own view, `ItemListEditor` in `src/ui/list_editor.rs`, which owns its fields and draft lines and exposes `name()` and `lines()`. Step 06 reuses it in the Templates tab.
- **Finish** asks for confirmation in an alert dialog:
  - **Title:** "Finish House?"
  - **Description:** "Takes 64 Stone, 4 Oak Door, … out of inventory and deletes the project with its plans." It lists each plan's target and item.
  - **OK text:** "Finish project"
- **Save as template** asks for a name in a dialog, defaulting to the project name without a "×N" suffix. A clash is reported as an error notification, which `finish` already does.
- **Delete** confirms "Delete House? Its 5 plans are deleted."
- **Requirements page.**
  - The plan strip at the top groups plan cards under small project headings (name and "3 / 5 reached"), with loose plans last. Without projects it looks as it does today.
  - In the right panel's "Plans served" list, a plan in a project shows the project name where it shows the note today. When the plan also has a note, show the project name first.

## Read first
- `src/ui/pages.rs`: `plans_page`; `items_page` and `item_panel` for the recipe composer (`recipe_draft`, `add_draft_input`, `save_recipe`), which the line editor should resemble.
- `src/ui/mod.rs`:
  - `rename_game_dialog` (a dialog with a field)
  - `confirm_delete_game` (an alert dialog)
  - `ItemSelect`, `selected_item`
  - `ReqTab` and the tabs in `requirements.rs`
- `src/ui/requirements.rs`: the `plans` strip and `plans_served`.
- `src/ui/tutorial.rs`: the Plans step and the `plan-add` highlight.
- The gpui-kit cheatsheet sections on Dialog, Select and Input.

## Changes
1. `src/ui/list_editor.rs`: `ItemListEditor`, as decided. It takes the game's item names when created, and gets `set_items` and `load(name, lines)` methods.
2. `Planner`: add a `plans_tab` state, a `plan_project` select for the Add plan form, and dialog methods: `new_project_dialog`, `project_from_template_dialog`, `rename_project_dialog`, `confirm_delete_project`, `confirm_finish_project` and `save_as_template_dialog`. Each calls the step 04 `Db` method and then `finish`.
3. `plans_page`: tabs, form, buttons and sections, as decided.
4. `requirements.rs`: group the plan strip by project, and show project names in `plans_served`.
5. Give clickable things ids that tests can use, with `.test_support()` last on hand-built elements:
   - `project-new`, `project-from-template`, `plan-project`
   - `("project-finish", key)`, `("project-save-template", key)`, `("project-rename", key)`, `("project-delete", key)`
   - inside the editor: `list-name`, `list-item`, `list-amount`, `list-add`, `("list-remove", key)`, `list-times`

## Tests (in `src/ui/tests.rs`)
- **New project:** two lines, Stick 2 and Coal 3, named "Camp", create a project with two plans in that order.
- **From template:** with a template made through `Db` (Stick 2) and ×3, the new project has a Stick plan with target 6 and is named "<template> ×3".
- **Add plan with a project selected:** the plan lands in that project.
- **Finish:** it is disabled while a plan isn't reached. After inventory is raised through `Db` and the view reloaded, clicking it and confirming takes the targets out of inventory and removes the project.
- **The tutorial test** `tutorial_waits_resumes_finishes_and_replays` still passes.

## Done when
- The README's done rule is met.
- Screenshots on the sample data with a project added look right. Add a project to `seed_sample_game` in `src/db.rs`: "Mine camp", with the lines Torch 8 and Lantern 2. Keep the existing loose plans; `design/spec.md` relies on them.
