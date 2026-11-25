# Step 04: Projects and templates, data only

**Goal.** Store projects and templates and give `Db` the methods steps 05 and 06 need. This step has no UI, and its tests are the specification.

**Depends on:** nothing (it can be done before 01–03, but keep the order).

## Decided
The vocabulary is in `CONTEXT.md` under **Project**, **Template** and **Finish**.

- **Project.**
  - It belongs to one game and has a name, which need not be unique, and `created_at`.
  - A plan references at most one project (`plans.project_id`; NULL means no project).
  - Deleting a project deletes its plans.
- **Template.**
  - It belongs to one game and has a name, unique within that game ignoring case (as `game_titled` matches titles), and `created_at`.
  - Its lines are ordered `(item, amount)` pairs with `amount > 0`, at most one line per item.
  - Deleting a template does not touch any project.
- **Making a project.** It takes a name, lines and a multiplier `times`, a whole number ≥ 1. Each line becomes a plan with `target = amount × times` and an empty note, inside the new project. The plans keep line order, so with the same `created_at` the order comes from `rowid`. Lines naming the same item are merged first, by adding their amounts. The project does not record which template it came from.
- **Adding a plan.** `add_plan` gets an optional project. The project must belong to the same game as the item; otherwise it is refused.
- **Save as template.** The template's lines are the project's plans summed per item, in the order of each item's first plan. The name must be free: "There is already a template called House."
- **Finish.**
  - It is refused unless every plan of the project is reached by `calc::plan_progress` on the game's current data: "Not every plan of House is reached yet."
  - In one transaction it takes each plan's target out of its item's inventory (never below 0, rounded as in `Db::craft`) and deletes the project, which deletes its plans.
- **Deleting an item.** An item listed in a template cannot be deleted. Mirror the recipe-input refusal in `delete_item`: "It is listed in the template House. Remove it from that template first."
- **Game files are unchanged.**
  - `export_game` with progress writes every plan, those in projects included, as loose plans.
  - Templates are not exported.
  - Merge ignores projects and templates.
  - Replace deletes the existing game's projects and templates along with the game.
- **Validation messages**, in the style of the existing ones:
  - "A project needs a name."
  - "A project needs at least one item."
  - "A template needs a name."
  - "Amounts must be above zero."
  - "The multiplier must be a whole number of at least 1."

## Read first
- `migrations/0001_init.sql`.
- `src/domain.rs`: `Plan`, `GameData`.
- `src/db.rs`:
  - `load_game`, `add_plan`, `delete_item`, `craft`
  - `delete_game` and the Replace branch of `import_game`: both delete `recipe_details` first, because a RESTRICT foreign key would block the cascade
  - `export_game`
  - the tests module and its `db()` helper
- `src/calc.rs`: `plan_progress`, `PlanProgress`.

## Changes
1. **Migration** `migrations/0002_projects_templates.sql`:
   - `projects`: id, game_id (references games, ON DELETE CASCADE), name, created_at.
   - `ALTER TABLE plans ADD COLUMN project_id TEXT REFERENCES projects (id) ON DELETE CASCADE`. SQLite allows this because the default is NULL.
   - `templates`: id, game_id (references games, ON DELETE CASCADE), name, created_at, plus a unique index on `(game_id, name COLLATE NOCASE)`.
   - `template_lines`: id, template_id (references templates, ON DELETE CASCADE), item_id (references items, **ON DELETE RESTRICT**), amount REAL with `CHECK (amount > 0)`, and `UNIQUE (template_id, item_id)`.
   - Indexes on `plans.project_id`, `projects.game_id`, `templates.game_id`, `template_lines.template_id` and `template_lines.item_id`.
2. **Domain** (`src/domain.rs`):
   - `Plan` gets `project: Option<Uuid>`.
   - New types: `Project { id, name }`, `Template { id, name, lines: Vec<TemplateLine> }` and `TemplateLine { item_id, amount }`.
   - `GameData` gets `projects: Vec<Project>` (oldest first) and `templates: Vec<Template>` (by name, ignoring case).
   - Helpers: `GameData::plans_of(project_id)` and `GameData::loose_plans()`.
3. **Calculation** (`src/calc.rs`): add `project_progress(data, &[PlanProgress]) -> HashMap<Uuid, (usize, usize)>`, giving (reached, total) per project, with a test.
4. **`Db`** (`src/db.rs`, new `// ---- projects ----` and `// ---- templates ----` sections). Every method is blocking, like the rest.
   - `add_project(game_id, name, lines: &[TemplateLine], times: u32) -> Result<Uuid>`
   - `rename_project(id, name)`
   - `delete_project(id)`
   - `finish_project(id, data: &GameData) -> Result<()>`
   - `add_plan(item_id, target, note, project: Option<Uuid>)`: update every caller, passing `None` where they have no project.
   - `add_template(game_id, name, lines) -> Result<Uuid>`
   - `rename_template(id, name)`
   - `set_template_lines(id, lines)`: replaces all lines in one transaction.
   - `delete_template(id)`
   - `template_from_project(project_id, name) -> Result<Uuid>`
   - `load_game` loads projects, templates with their lines (in `rowid` order), and each plan's project.
   - `delete_item` refuses items listed in a template.
   - `delete_game` and the Replace branch of `import_game` delete the game's `template_lines` first, next to where they delete `recipe_details` first.
5. **`CLAUDE.md`**: add `projects`, `templates` and `template_lines` to the "Domain model" list.

## Tests (in `src/db.rs`, plus the calc test)
- A project made from lines × 3 has one plan per line, in line order, with targets × 3; duplicate lines are merged.
- Deleting a project deletes its plans and nothing else; loose plans and other projects stay.
- A plan cannot be added to a project of another game.
- Finish:
  - It is refused while a plan is not reached.
  - Once every plan is reached, it takes the targets out of inventory and deletes the project.
  - An older loose plan for the same item keeps its share. Example: inventory 10, a loose plan for 4 that is older, and a project plan for 6. After Finish, inventory is 4 and the loose plan is still reached.
- Template names are unique ignoring case. Lines need amounts above 0. `set_template_lines` replaces the lines.
- Saving a project as a template sums its plans per item.
- An item listed in a template cannot be deleted. After it is removed from the template, it can be.
- Deleting a game that has templates and projects works, and so does a Replace import over it.
- Exporting with progress, then importing as a new game, brings the project's plans back as loose plans.

## Done when
- The README's done rule is met. No UI changes beyond what the `add_plan` signature forces.
