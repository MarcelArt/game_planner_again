//! Storage: a SQLite database and an image folder inside the data folder.
//!
//! sqlx is async and needs Tokio, while the UI runs on GPUI's own executor. `Db` owns a small
//! Tokio runtime and exposes blocking methods: every query is a local SQLite call that takes well
//! under a millisecond, so the UI calls them directly.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use uuid::Uuid;

use crate::domain::{Game, GameData, Input, Item, Plan, Recipe};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub struct Db {
    rt: tokio::runtime::Runtime,
    pool: SqlitePool,
    data_dir: PathBuf,
}

/// Images whose longer side is below this many pixels are enlarged on import.
const SMALL_IMAGE: u32 = 96;
/// The longer side an enlarged image gets, at least.
const ENLARGED_IMAGE: u32 = 144;

/// Counts shown on a game's card in the library.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GameStats {
    pub items: i64,
    pub recipes: i64,
    pub plans: i64,
}

/// `$XDG_DATA_HOME/game_planner`, falling back to `~/.local/share/game_planner`.
pub fn default_data_dir() -> Result<PathBuf> {
    Ok(dirs::data_dir().context("no data folder on this system")?.join("game_planner"))
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn uuid(row: &sqlx::sqlite::SqliteRow, col: &str) -> Result<Uuid> {
    let s: String = row.try_get(col)?;
    Ok(Uuid::from_str(&s)?)
}

impl Db {
    /// Opens (creating if needed) the database in `data_dir` and brings its schema up to date.
    pub fn open(data_dir: impl Into<PathBuf>) -> Result<Self> {
        let data_dir = data_dir.into();
        std::fs::create_dir_all(&data_dir).with_context(|| format!("creating {}", data_dir.display()))?;
        let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build()?;
        let options = SqliteConnectOptions::new()
            .filename(data_dir.join("game_planner.db"))
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = rt.block_on(async {
            let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await?;
            MIGRATOR.run(&pool).await?;
            anyhow::Ok(pool)
        })?;
        Ok(Self { rt, pool, data_dir })
    }

    /// Absolute path of an image stored as a path relative to the data folder.
    pub fn image_path(&self, relative: &str) -> PathBuf {
        self.data_dir.join(relative)
    }

    /// Copies an image into `images/<game id>/` and returns its path relative to the data folder.
    pub fn import_image(&self, game_id: Uuid, source: &Path) -> Result<String> {
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .filter(|e| matches!(e.as_str(), "png" | "jpg" | "jpeg" | "webp"))
            .ok_or_else(|| anyhow!("Only PNG, JPEG and WebP images can be used."))?;
        let dir = format!("images/{game_id}");
        std::fs::create_dir_all(self.data_dir.join(&dir))?;

        // The renderer smooths images when it enlarges them, which blurs pixel art. Small images
        // are therefore enlarged here, once, by a whole factor without smoothing.
        if let Ok(small) = image::open(source)
            && small.width().max(small.height()) < SMALL_IMAGE
        {
            let factor = ENLARGED_IMAGE.div_ceil(small.width().max(small.height()).max(1));
            let large = small.resize_exact(small.width() * factor, small.height() * factor, image::imageops::FilterType::Nearest);
            let relative = format!("{dir}/{}.png", Uuid::new_v4());
            large.save(self.data_dir.join(&relative)).context("saving the enlarged image")?;
            return Ok(relative);
        }

        let relative = format!("{dir}/{}.{ext}", Uuid::new_v4());
        std::fs::copy(source, self.data_dir.join(&relative)).with_context(|| format!("copying {}", source.display()))?;
        Ok(relative)
    }

    fn remove_image(&self, relative: Option<String>) {
        if let Some(r) = relative {
            let _ = std::fs::remove_file(self.data_dir.join(r));
        }
    }

    // ---- games ----

    pub fn games(&self) -> Result<Vec<Game>> {
        self.rt.block_on(async {
            let rows = sqlx::query("SELECT id, title, thumbnail FROM games ORDER BY created_at").fetch_all(&self.pool).await?;
            rows.iter()
                .map(|r| Ok(Game { id: uuid(r, "id")?, title: r.try_get("title")?, thumbnail: r.try_get("thumbnail")? }))
                .collect()
        })
    }

    pub fn game_stats(&self, game_id: Uuid) -> Result<GameStats> {
        self.rt.block_on(async {
            let g = game_id.to_string();
            let row = sqlx::query(
                "SELECT (SELECT COUNT(*) FROM items WHERE game_id = ?1) AS items,
                        (SELECT COUNT(*) FROM recipes r JOIN items i ON i.id = r.item_id WHERE i.game_id = ?1) AS recipes,
                        (SELECT COUNT(*) FROM plans p JOIN items i ON i.id = p.item_id WHERE i.game_id = ?1) AS plans",
            )
            .bind(g)
            .fetch_one(&self.pool)
            .await?;
            Ok(GameStats { items: row.try_get("items")?, recipes: row.try_get("recipes")?, plans: row.try_get("plans")? })
        })
    }

    pub fn add_game(&self, title: &str) -> Result<Uuid> {
        let title = title.trim();
        if title.is_empty() {
            bail!("A game needs a title.");
        }
        let id = Uuid::new_v4();
        self.rt.block_on(async {
            sqlx::query("INSERT INTO games (id, title, created_at) VALUES (?, ?, ?)")
                .bind(id.to_string())
                .bind(title)
                .bind(now_ms())
                .execute(&self.pool)
                .await?;
            anyhow::Ok(())
        })?;
        Ok(id)
    }

    pub fn rename_game(&self, id: Uuid, title: &str) -> Result<()> {
        let title = title.trim();
        if title.is_empty() {
            bail!("A game needs a title.");
        }
        self.rt.block_on(async {
            sqlx::query("UPDATE games SET title = ? WHERE id = ?").bind(title).bind(id.to_string()).execute(&self.pool).await?;
            Ok(())
        })
    }

    /// Copies `source` into the data folder and makes it the game's thumbnail.
    pub fn set_game_thumbnail(&self, id: Uuid, source: &Path) -> Result<()> {
        let relative = self.import_image(id, source)?;
        let old: Option<String> = self.rt.block_on(async {
            let old = sqlx::query_scalar("SELECT thumbnail FROM games WHERE id = ?").bind(id.to_string()).fetch_one(&self.pool).await?;
            sqlx::query("UPDATE games SET thumbnail = ? WHERE id = ?").bind(&relative).bind(id.to_string()).execute(&self.pool).await?;
            anyhow::Ok(old)
        })?;
        self.remove_image(old);
        Ok(())
    }

    /// Deletes the game with all its items, recipes, inventory, plans and images.
    pub fn delete_game(&self, id: Uuid) -> Result<()> {
        self.rt.block_on(async {
            let mut tx = self.pool.begin().await?;
            let g = id.to_string();
            // Inputs first: they restrict the deletion of the items they name.
            sqlx::query("DELETE FROM recipe_details WHERE item_id IN (SELECT id FROM items WHERE game_id = ?)")
                .bind(&g)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM games WHERE id = ?").bind(&g).execute(&mut *tx).await?;
            tx.commit().await?;
            anyhow::Ok(())
        })?;
        let _ = std::fs::remove_dir_all(self.data_dir.join("images").join(id.to_string()));
        Ok(())
    }

    // ---- one game's data ----

    pub fn load_game(&self, game_id: Uuid) -> Result<GameData> {
        self.rt.block_on(async {
            let g = game_id.to_string();
            let mut data = GameData::default();

            for r in sqlx::query("SELECT id, name, icon FROM items WHERE game_id = ? ORDER BY name COLLATE NOCASE").bind(&g).fetch_all(&self.pool).await? {
                data.items.push(Item { id: uuid(&r, "id")?, name: r.try_get("name")?, icon: r.try_get("icon")? });
            }

            let mut inputs: HashMap<Uuid, Vec<Input>> = HashMap::new();
            for r in sqlx::query(
                "SELECT d.recipe_id, d.item_id, d.input FROM recipe_details d
                 JOIN recipes r ON r.id = d.recipe_id JOIN items i ON i.id = r.item_id
                 WHERE i.game_id = ? ORDER BY d.rowid",
            )
            .bind(&g)
            .fetch_all(&self.pool)
            .await?
            {
                inputs.entry(uuid(&r, "recipe_id")?).or_default().push(Input { item_id: uuid(&r, "item_id")?, amount: r.try_get("input")? });
            }
            for r in sqlx::query(
                "SELECT r.id, r.item_id, r.output, r.is_default FROM recipes r JOIN items i ON i.id = r.item_id
                 WHERE i.game_id = ? ORDER BY r.created_at, r.rowid",
            )
            .bind(&g)
            .fetch_all(&self.pool)
            .await?
            {
                let id = uuid(&r, "id")?;
                data.recipes.push(Recipe {
                    id,
                    item_id: uuid(&r, "item_id")?,
                    output: r.try_get("output")?,
                    is_default: r.try_get::<i64, _>("is_default")? != 0,
                    inputs: inputs.remove(&id).unwrap_or_default(),
                });
            }

            for r in sqlx::query("SELECT v.item_id, v.amount FROM inventories v JOIN items i ON i.id = v.item_id WHERE i.game_id = ?")
                .bind(&g)
                .fetch_all(&self.pool)
                .await?
            {
                data.inventory.insert(uuid(&r, "item_id")?, r.try_get("amount")?);
            }

            for r in sqlx::query(
                "SELECT p.id, p.item_id, p.target, p.note FROM plans p JOIN items i ON i.id = p.item_id
                 WHERE i.game_id = ? ORDER BY p.created_at, p.rowid",
            )
            .bind(&g)
            .fetch_all(&self.pool)
            .await?
            {
                data.plans.push(Plan { id: uuid(&r, "id")?, item_id: uuid(&r, "item_id")?, target: r.try_get("target")?, note: r.try_get("note")? });
            }
            Ok(data)
        })
    }

    // ---- items ----

    pub fn add_item(&self, game_id: Uuid, name: &str) -> Result<Uuid> {
        let name = name.trim();
        if name.is_empty() {
            bail!("An item needs a name.");
        }
        let id = Uuid::new_v4();
        self.rt.block_on(async {
            let taken: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM items WHERE game_id = ? AND name = ? COLLATE NOCASE")
                .bind(game_id.to_string())
                .bind(name)
                .fetch_one(&self.pool)
                .await?;
            if taken > 0 {
                bail!("This game already has an item named {name}.");
            }
            sqlx::query("INSERT INTO items (id, game_id, name) VALUES (?, ?, ?)")
                .bind(id.to_string())
                .bind(game_id.to_string())
                .bind(name)
                .execute(&self.pool)
                .await?;
            Ok(())
        })?;
        Ok(id)
    }

    pub fn rename_item(&self, id: Uuid, name: &str) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            bail!("An item needs a name.");
        }
        self.rt.block_on(async {
            let taken: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM items WHERE name = ?1 COLLATE NOCASE AND id <> ?2
                 AND game_id = (SELECT game_id FROM items WHERE id = ?2)",
            )
            .bind(name)
            .bind(id.to_string())
            .fetch_one(&self.pool)
            .await?;
            if taken > 0 {
                bail!("This game already has an item named {name}.");
            }
            sqlx::query("UPDATE items SET name = ? WHERE id = ?").bind(name).bind(id.to_string()).execute(&self.pool).await?;
            Ok(())
        })
    }

    /// Copies `source` into the game's image folder and makes it the item's icon.
    pub fn set_item_icon(&self, game_id: Uuid, id: Uuid, source: &Path) -> Result<()> {
        let relative = self.import_image(game_id, source)?;
        let old: Option<String> = self.rt.block_on(async {
            let old = sqlx::query_scalar("SELECT icon FROM items WHERE id = ?").bind(id.to_string()).fetch_one(&self.pool).await?;
            sqlx::query("UPDATE items SET icon = ? WHERE id = ?").bind(&relative).bind(id.to_string()).execute(&self.pool).await?;
            anyhow::Ok(old)
        })?;
        self.remove_image(old);
        Ok(())
    }

    /// Deletes an item with its own recipes, inventory and plans.
    /// Refused while another item's recipe uses it as an input.
    pub fn delete_item(&self, id: Uuid) -> Result<()> {
        let icon: Option<String> = self.rt.block_on(async {
            let i = id.to_string();
            let users: Vec<String> = sqlx::query_scalar(
                "SELECT DISTINCT i.name FROM recipe_details d JOIN recipes r ON r.id = d.recipe_id
                 JOIN items i ON i.id = r.item_id WHERE d.item_id = ? AND r.item_id <> ? ORDER BY i.name",
            )
            .bind(&i)
            .bind(&i)
            .fetch_all(&self.pool)
            .await?;
            if !users.is_empty() {
                bail!("It is an input of the recipe for {}. Change or delete that recipe first.", users.join(", "));
            }
            let icon = sqlx::query_scalar("SELECT icon FROM items WHERE id = ?").bind(&i).fetch_one(&self.pool).await?;
            sqlx::query("DELETE FROM items WHERE id = ?").bind(&i).execute(&self.pool).await?;
            Ok(icon)
        })?;
        self.remove_image(icon);
        Ok(())
    }

    // ---- recipes ----

    /// Adds a recipe for `item_id`. The first recipe of an item becomes its default.
    pub fn add_recipe(&self, item_id: Uuid, output: f64, inputs: &[Input]) -> Result<Uuid> {
        if !(output > 0.0) {
            bail!("A recipe must make more than zero per craft.");
        }
        if inputs.is_empty() {
            bail!("A recipe needs at least one input.");
        }
        if inputs.iter().any(|i| i.item_id == item_id) {
            bail!("A recipe cannot use the item it makes as an input.");
        }
        if inputs.iter().any(|i| !(i.amount > 0.0)) {
            bail!("Every input needs an amount above zero.");
        }
        let id = Uuid::new_v4();
        self.rt.block_on(async {
            let mut tx = self.pool.begin().await?;
            let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM recipes WHERE item_id = ?").bind(item_id.to_string()).fetch_one(&mut *tx).await?;
            sqlx::query("INSERT INTO recipes (id, item_id, output, is_default, created_at) VALUES (?, ?, ?, ?, ?)")
                .bind(id.to_string())
                .bind(item_id.to_string())
                .bind(output)
                .bind((existing == 0) as i64)
                .bind(now_ms())
                .execute(&mut *tx)
                .await?;
            for input in inputs {
                sqlx::query("INSERT INTO recipe_details (id, recipe_id, item_id, input) VALUES (?, ?, ?, ?)")
                    .bind(Uuid::new_v4().to_string())
                    .bind(id.to_string())
                    .bind(input.item_id.to_string())
                    .bind(input.amount)
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await?;
            anyhow::Ok(())
        })?;
        Ok(id)
    }

    /// Makes `recipe_id` the default recipe of its item.
    pub fn set_default_recipe(&self, recipe_id: Uuid) -> Result<()> {
        self.rt.block_on(async {
            sqlx::query("UPDATE recipes SET is_default = (id = ?1) WHERE item_id = (SELECT item_id FROM recipes WHERE id = ?1)")
                .bind(recipe_id.to_string())
                .execute(&self.pool)
                .await?;
            Ok(())
        })
    }

    /// Deletes a recipe. If it was the default, the item's oldest remaining recipe takes over.
    pub fn delete_recipe(&self, recipe_id: Uuid) -> Result<()> {
        self.rt.block_on(async {
            let mut tx = self.pool.begin().await?;
            let r = recipe_id.to_string();
            let item: String = sqlx::query_scalar("SELECT item_id FROM recipes WHERE id = ?").bind(&r).fetch_one(&mut *tx).await?;
            sqlx::query("DELETE FROM recipes WHERE id = ?").bind(&r).execute(&mut *tx).await?;
            sqlx::query(
                "UPDATE recipes SET is_default = 1
                 WHERE id = (SELECT id FROM recipes WHERE item_id = ?1 ORDER BY created_at, rowid LIMIT 1)
                   AND NOT EXISTS (SELECT 1 FROM recipes WHERE item_id = ?1 AND is_default = 1)",
            )
            .bind(&item)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            Ok(())
        })
    }

    // ---- inventory ----

    /// Sets the owned amount of an item. Never below zero.
    pub fn set_inventory(&self, item_id: Uuid, amount: f64) -> Result<()> {
        let amount = if amount.is_finite() { amount.max(0.0) } else { 0.0 };
        self.rt.block_on(async {
            sqlx::query(
                "INSERT INTO inventories (id, item_id, amount) VALUES (?, ?, ?)
                 ON CONFLICT (item_id) DO UPDATE SET amount = excluded.amount",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(item_id.to_string())
            .bind(amount)
            .execute(&self.pool)
            .await?;
            Ok(())
        })
    }

    /// Performs up to `crafts` crafts of a recipe: removes the inputs, adds the output.
    /// Performs only as many as inventory covers and returns that number.
    pub fn craft(&self, recipe: &Recipe, crafts: u64, data: &GameData) -> Result<u64> {
        let n = crafts.min(crate::calc::craftable(recipe, data));
        if n == 0 {
            return Ok(0);
        }
        let round = |v: f64| (v * 1e6).round() / 1e6;
        for input in &recipe.inputs {
            self.set_inventory(input.item_id, round(data.owned(input.item_id) - input.amount * n as f64))?;
        }
        self.set_inventory(recipe.item_id, round(data.owned(recipe.item_id) + recipe.output * n as f64))?;
        Ok(n)
    }

    // ---- plans ----

    pub fn add_plan(&self, item_id: Uuid, target: f64, note: &str) -> Result<Uuid> {
        if !(target > 0.0) {
            bail!("A plan needs a target above zero.");
        }
        let id = Uuid::new_v4();
        self.rt.block_on(async {
            sqlx::query("INSERT INTO plans (id, item_id, target, note, created_at) VALUES (?, ?, ?, ?, ?)")
                .bind(id.to_string())
                .bind(item_id.to_string())
                .bind(target)
                .bind(note.trim())
                .bind(now_ms())
                .execute(&self.pool)
                .await?;
            anyhow::Ok(())
        })?;
        Ok(id)
    }

    pub fn delete_plan(&self, id: Uuid) -> Result<()> {
        self.rt.block_on(async {
            sqlx::query("DELETE FROM plans WHERE id = ?").bind(id.to_string()).execute(&self.pool).await?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(dir.path()).unwrap();
        (dir, db)
    }

    #[test]
    fn a_game_round_trips_with_its_items_recipes_inventory_and_plans() {
        let (_dir, db) = db();
        let game = db.add_game("Hollowdeep").unwrap();
        let stick = db.add_item(game, "Stick").unwrap();
        let coal = db.add_item(game, "Coal").unwrap();
        let torch = db.add_item(game, "Torch").unwrap();
        let recipe = db
            .add_recipe(torch, 4.0, &[Input { item_id: stick, amount: 1.0 }, Input { item_id: coal, amount: 1.0 }])
            .unwrap();
        db.set_inventory(coal, 5.0).unwrap();
        db.set_inventory(coal, 6.5).unwrap();
        db.add_plan(torch, 24.0, "Light the east mine").unwrap();

        let data = db.load_game(game).unwrap();
        assert_eq!(data.items.len(), 3);
        assert_eq!(data.owned(coal), 6.5);
        let r = data.default_recipe(torch).unwrap();
        assert_eq!((r.id, r.output, r.inputs.len()), (recipe, 4.0, 2));
        assert_eq!(data.plans[0].note, "Light the east mine");
        assert_eq!(db.game_stats(game).unwrap(), GameStats { items: 3, recipes: 1, plans: 1 });
    }

    #[test]
    fn the_first_recipe_is_the_default_and_the_default_can_be_switched_and_survives_deletion() {
        let (_dir, db) = db();
        let game = db.add_game("G").unwrap();
        let (a, b, torch) = (db.add_item(game, "A").unwrap(), db.add_item(game, "B").unwrap(), db.add_item(game, "Torch").unwrap());
        let first = db.add_recipe(torch, 4.0, &[Input { item_id: a, amount: 1.0 }]).unwrap();
        let second = db.add_recipe(torch, 2.0, &[Input { item_id: b, amount: 1.0 }]).unwrap();
        assert_eq!(db.load_game(game).unwrap().default_recipe(torch).unwrap().id, first);
        db.set_default_recipe(second).unwrap();
        assert_eq!(db.load_game(game).unwrap().default_recipe(torch).unwrap().id, second);
        db.delete_recipe(second).unwrap();
        assert_eq!(db.load_game(game).unwrap().default_recipe(torch).unwrap().id, first);
    }

    #[test]
    fn an_item_used_as_an_input_cannot_be_deleted() {
        let (_dir, db) = db();
        let game = db.add_game("G").unwrap();
        let (coal, torch) = (db.add_item(game, "Coal").unwrap(), db.add_item(game, "Torch").unwrap());
        db.add_recipe(torch, 4.0, &[Input { item_id: coal, amount: 1.0 }]).unwrap();
        let err = db.delete_item(coal).unwrap_err().to_string();
        assert!(err.contains("Torch"), "{err}");
        // The item that owns the recipe can go, and then so can its input.
        db.delete_item(torch).unwrap();
        db.delete_item(coal).unwrap();
        assert!(db.load_game(game).unwrap().items.is_empty());
    }

    #[test]
    fn invalid_recipes_and_duplicate_names_are_refused() {
        let (_dir, db) = db();
        let game = db.add_game("G").unwrap();
        let torch = db.add_item(game, "Torch").unwrap();
        assert!(db.add_item(game, "torch").is_err());
        assert!(db.add_recipe(torch, 4.0, &[]).is_err());
        assert!(db.add_recipe(torch, 4.0, &[Input { item_id: torch, amount: 1.0 }]).is_err());
    }

    #[test]
    fn crafting_moves_inventory_and_never_goes_below_zero() {
        let (_dir, db) = db();
        let game = db.add_game("G").unwrap();
        let (stick, coal, torch) = (db.add_item(game, "Stick").unwrap(), db.add_item(game, "Coal").unwrap(), db.add_item(game, "Torch").unwrap());
        db.add_recipe(torch, 4.0, &[Input { item_id: stick, amount: 1.0 }, Input { item_id: coal, amount: 1.0 }]).unwrap();
        db.set_inventory(stick, 3.0).unwrap();
        db.set_inventory(coal, 5.0).unwrap();
        let data = db.load_game(game).unwrap();
        let recipe = data.default_recipe(torch).unwrap().clone();
        assert_eq!(db.craft(&recipe, 10, &data).unwrap(), 3);
        let data = db.load_game(game).unwrap();
        assert_eq!((data.owned(stick), data.owned(coal), data.owned(torch)), (0.0, 2.0, 12.0));
        assert_eq!(db.craft(&recipe, 1, &data).unwrap(), 0);
    }

    #[test]
    fn images_are_copied_into_the_games_folder_and_removed_with_the_game() {
        let (dir, db) = db();
        let game = db.add_game("G").unwrap();
        let source = dir.path().join("cover.PNG");
        std::fs::write(&source, b"not really a png").unwrap();
        db.set_game_thumbnail(game, &source).unwrap();
        let stored = db.games().unwrap()[0].thumbnail.clone().unwrap();
        assert!(stored.starts_with(&format!("images/{game}/")) && stored.ends_with(".png"), "{stored}");
        assert!(db.image_path(&stored).exists());
        assert!(db.set_game_thumbnail(game, &dir.path().join("notes.txt")).is_err());

        let item = db.add_item(game, "Coal").unwrap();
        let other = db.add_item(game, "Torch").unwrap();
        db.add_recipe(other, 1.0, &[Input { item_id: item, amount: 1.0 }]).unwrap();
        db.delete_game(game).unwrap();
        assert!(db.games().unwrap().is_empty());
        assert!(!db.image_path(&stored).exists());
    }

    #[test]
    fn small_pixel_art_is_enlarged_without_smoothing() {
        let (dir, db) = db();
        let game = db.add_game("G").unwrap();
        let item = db.add_item(game, "Torch").unwrap();
        let source = dir.path().join("torch.png");
        let mut art = image::RgbaImage::new(16, 16);
        art.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        art.save(&source).unwrap();
        db.set_item_icon(game, item, &source).unwrap();
        let stored = db.load_game(game).unwrap().items[0].icon.clone().unwrap();
        let big = image::open(db.image_path(&stored)).unwrap().to_rgba8();
        assert_eq!(big.dimensions(), (144, 144));
        // The red source pixel became a solid 9x9 block: no blended colours at its edge.
        assert_eq!(big.get_pixel(8, 8).0, [255, 0, 0, 255]);
        assert_eq!(big.get_pixel(9, 9).0, [0, 0, 0, 0]);
    }

    /// Fills a data folder with the sample game used by the design mockups, for trying the app:
    /// `SEED_DIR=/some/dir cargo test seed_sample_game -- --ignored`, then run the app with
    /// `XDG_DATA_HOME=/some/dir/..` pointing at the folder that contains `game_planner`.
    #[test]
    #[ignore]
    fn seed_sample_game() {
        let dir = std::env::var("SEED_DIR").expect("set SEED_DIR");
        let db = Db::open(dir).unwrap();
        let game = db.add_game("Hollowdeep").unwrap();
        let mut ids = HashMap::new();
        for name in ["Oak Log", "Coal", "Iron Ore", "Resin", "Oak Plank", "Stick", "Torch", "Iron Ingot", "Iron Block", "Iron Pickaxe", "Copper Ingot", "Copper Block", "Lantern"] {
            ids.insert(name, db.add_item(game, name).unwrap());
        }
        let recipes: [(&str, f64, &[(&str, f64)]); 11] = [
            ("Oak Plank", 4.0, &[("Oak Log", 1.0)]),
            ("Stick", 4.0, &[("Oak Plank", 2.0)]),
            ("Torch", 4.0, &[("Stick", 1.0), ("Coal", 1.0)]),
            ("Torch", 2.0, &[("Stick", 1.0), ("Resin", 1.0)]),
            ("Iron Ingot", 1.0, &[("Iron Ore", 1.0), ("Coal", 0.125)]),
            ("Iron Ingot", 9.0, &[("Iron Block", 1.0)]),
            ("Iron Block", 1.0, &[("Iron Ingot", 9.0)]),
            ("Iron Pickaxe", 1.0, &[("Iron Ingot", 3.0), ("Stick", 2.0)]),
            ("Copper Ingot", 9.0, &[("Copper Block", 1.0)]),
            ("Copper Block", 1.0, &[("Copper Ingot", 9.0)]),
            ("Lantern", 1.0, &[("Torch", 1.0), ("Copper Ingot", 4.0)]),
        ];
        for (item, output, inputs) in recipes {
            let inputs: Vec<Input> = inputs.iter().map(|(n, a)| Input { item_id: ids[n], amount: *a }).collect();
            db.add_recipe(ids[item], output, &inputs).unwrap();
        }
        for (name, amount) in [("Oak Log", 2.0), ("Coal", 5.0), ("Iron Ore", 4.0), ("Oak Plank", 12.0), ("Stick", 3.0), ("Torch", 6.0), ("Iron Ingot", 1.0), ("Copper Ingot", 6.0)] {
            db.set_inventory(ids[name], amount).unwrap();
        }
        for (name, target, note) in [("Torch", 24.0, "Light the east mine"), ("Iron Pickaxe", 2.0, "One spare"), ("Torch", 16.0, "Base perimeter"), ("Lantern", 4.0, "Dock"), ("Oak Plank", 8.0, "Floor patch")] {
            db.add_plan(ids[name], target, note).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        // Optional: SEED_ICONS is a folder of `<item name in snake case>.png` icons.
        if let Ok(icons) = std::env::var("SEED_ICONS") {
            for (name, id) in &ids {
                let file = Path::new(&icons).join(format!("{}.png", name.to_lowercase().replace(' ', "_")));
                if file.exists() {
                    db.set_item_icon(game, *id, &file).unwrap();
                }
            }
        }
        db.add_game("Saltwind Isles").unwrap();
    }
}
