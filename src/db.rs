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
use crate::game_file::{FileInput, FileItem, FileOwned, FilePlan, FileProgress, FileRecipe, GameFile, name_key};

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

/// What an import does when a game with the game file's title already exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnClash {
    /// Discard the existing game, progress included, and put the file's contents in its place.
    Replace,
    /// Add the items, recipes and pictures the existing game lacks. Nothing is removed or changed.
    Merge,
    /// Import as a new game under a numbered title.
    KeepBoth,
}

/// What an import did.
#[derive(Clone, Debug, PartialEq)]
pub struct Imported {
    pub game: Uuid,
    pub title: String,
    pub merged: bool,
    /// Items and recipes added.
    pub items: usize,
    pub recipes: usize,
}

/// `$XDG_DATA_HOME/game_planner`, falling back to `~/.local/share/game_planner`.
/// Debug builds use `game_planner.dev` instead, so development never touches real data.
pub fn default_data_dir() -> Result<PathBuf> {
    let name = if cfg!(debug_assertions) { "game_planner.dev" } else { "game_planner" };
    Ok(dirs::data_dir().context("no data folder on this system")?.join(name))
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

    /// The game with this title, ignoring case and surrounding spaces. Titles are unique, but
    /// data from before they were may hold several; the oldest wins.
    pub fn game_titled(&self, title: &str) -> Result<Option<Uuid>> {
        self.rt.block_on(async {
            let id: Option<String> = sqlx::query_scalar("SELECT id FROM games WHERE title = ? COLLATE NOCASE ORDER BY created_at, rowid LIMIT 1")
                .bind(title.trim())
                .fetch_optional(&self.pool)
                .await?;
            Ok(id.map(|id| Uuid::from_str(&id)).transpose()?)
        })
    }

    pub fn add_game(&self, title: &str) -> Result<Uuid> {
        let title = title.trim();
        if title.is_empty() {
            bail!("A game needs a title.");
        }
        if self.game_titled(title)?.is_some() {
            bail!("There is already a game called {title}.");
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
        if self.game_titled(title)?.is_some_and(|other| other != id) {
            bail!("There is already a game called {title}.");
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

    // ---- game files ----

    /// Writes a game to a game file at `dest`: its items, recipes and pictures, and the player's
    /// inventory and plans only when `include_progress` is set.
    pub fn export_game(&self, id: Uuid, include_progress: bool, dest: &Path) -> Result<()> {
        let game = self.games()?.into_iter().find(|g| g.id == id).context("This game no longer exists.")?;
        let data = self.load_game(id)?;
        let mut file = GameFile::new(game.title);
        // Pictures are stored as they are: small ones were already enlarged when first imported.
        let pack = |file: &mut GameFile, stem: &str, relative: &Option<String>| {
            let relative = relative.as_ref()?;
            let bytes = std::fs::read(self.image_path(relative)).ok()?;
            file.add_picture(stem, relative, bytes)
        };
        file.thumbnail = pack(&mut file, "thumbnail", &game.thumbnail);
        for (n, item) in data.items.iter().enumerate() {
            let picture = pack(&mut file, &n.to_string(), &item.icon);
            let recipes = data
                .recipes_of(item.id)
                .map(|r| FileRecipe {
                    output: r.output,
                    default: r.is_default,
                    inputs: r.inputs.iter().map(|i| FileInput { item: data.item_name(i.item_id).to_string(), amount: i.amount }).collect(),
                })
                .collect();
            file.items.push(FileItem { name: item.name.clone(), picture, recipes });
        }
        if include_progress {
            file.progress = Some(FileProgress {
                inventory: data.items.iter().filter(|i| data.owned(i.id) > 0.0).map(|i| FileOwned { item: i.name.clone(), amount: data.owned(i.id) }).collect(),
                plans: data.plans.iter().map(|p| FilePlan { item: data.item_name(p.item_id).to_string(), target: p.target, note: p.note.clone() }).collect(),
            });
        }
        file.write(dest)
    }

    /// Brings a game file into the planner. It becomes a new game unless a game with its title
    /// exists; then `on_clash` decides, and without one the import is refused.
    /// Either everything is imported or nothing is.
    pub fn import_game(&self, file: &GameFile, on_clash: Option<OnClash>) -> Result<Imported> {
        let existing = self.game_titled(&file.title)?;
        let (merge_into, replace, title) = match (existing, on_clash) {
            (None, _) => (None, None, file.title.clone()),
            (Some(_), None) => bail!("There is already a game called {}.", file.title),
            (Some(id), Some(OnClash::Merge)) => (Some(id), None, file.title.clone()),
            (Some(id), Some(OnClash::Replace)) => (None, Some(id), file.title.clone()),
            (Some(_), Some(OnClash::KeepBoth)) => {
                let mut n = 2;
                while self.game_titled(&format!("{} ({n})", file.title))?.is_some() {
                    n += 1;
                }
                (None, None, format!("{} ({n})", file.title))
            }
        };
        let game = merge_into.unwrap_or_else(Uuid::new_v4);
        // What a merge has to compare against. Read before the transaction: the pool has one connection.
        let (old, old_thumbnail) = match merge_into {
            Some(id) => (self.load_game(id)?, self.games()?.into_iter().find(|g| g.id == id).and_then(|g| g.thumbnail)),
            None => (GameData::default(), None),
        };

        // Pictures are written as the rows that name them are, and removed again if anything fails.
        let written = std::cell::RefCell::new(Vec::<String>::new());
        let save = |picture: Option<&String>| -> Result<Option<String>> {
            let Some((bytes, ext)) = file.picture(picture) else { return Ok(None) };
            let dir = format!("images/{game}");
            std::fs::create_dir_all(self.data_dir.join(&dir))?;
            let relative = format!("{dir}/{}.{ext}", Uuid::new_v4());
            std::fs::write(self.data_dir.join(&relative), bytes).context("saving a picture")?;
            written.borrow_mut().push(relative.clone());
            Ok(Some(relative))
        };

        let result = self.rt.block_on(async {
            let mut tx = self.pool.begin().await?;
            let g = game.to_string();
            let now = now_ms();

            if merge_into.is_none() {
                // A replacement keeps the replaced game's place in the library.
                let mut created_at = now;
                if let Some(old_id) = replace {
                    let o = old_id.to_string();
                    created_at = sqlx::query_scalar("SELECT created_at FROM games WHERE id = ?").bind(&o).fetch_one(&mut *tx).await?;
                    sqlx::query("DELETE FROM recipe_details WHERE item_id IN (SELECT id FROM items WHERE game_id = ?)").bind(&o).execute(&mut *tx).await?;
                    sqlx::query("DELETE FROM games WHERE id = ?").bind(&o).execute(&mut *tx).await?;
                }
                sqlx::query("INSERT INTO games (id, title, thumbnail, created_at) VALUES (?, ?, ?, ?)")
                    .bind(&g)
                    .bind(&title)
                    .bind(save(file.thumbnail.as_ref())?)
                    .bind(created_at)
                    .execute(&mut *tx)
                    .await?;
            } else if old_thumbnail.is_none()
                && let Some(thumbnail) = save(file.thumbnail.as_ref())?
            {
                sqlx::query("UPDATE games SET thumbnail = ? WHERE id = ?").bind(thumbnail).bind(&g).execute(&mut *tx).await?;
            }

            let mut ids: HashMap<String, Uuid> = old.items.iter().map(|i| (name_key(&i.name), i.id)).collect();
            let mut items = 0;
            for item in &file.items {
                match ids.get(&name_key(&item.name)).and_then(|id| old.item(*id)) {
                    None => {
                        let id = Uuid::new_v4();
                        sqlx::query("INSERT INTO items (id, game_id, name, icon) VALUES (?, ?, ?, ?)")
                            .bind(id.to_string())
                            .bind(&g)
                            .bind(&item.name)
                            .bind(save(item.picture.as_ref())?)
                            .execute(&mut *tx)
                            .await?;
                        ids.insert(name_key(&item.name), id);
                        items += 1;
                    }
                    // An item the game already has keeps its own picture.
                    Some(known) if known.icon.is_none() => {
                        if let Some(icon) = save(item.picture.as_ref())? {
                            sqlx::query("UPDATE items SET icon = ? WHERE id = ?").bind(icon).bind(known.id.to_string()).execute(&mut *tx).await?;
                        }
                    }
                    Some(_) => {}
                }
            }

            let mut recipes = 0;
            for item in &file.items {
                let item_id = ids[&name_key(&item.name)];
                let had_recipes = old.recipes_of(item_id).next().is_some();
                for recipe in &item.recipes {
                    let inputs: Vec<Input> = recipe.inputs.iter().map(|i| Input { item_id: ids[&name_key(&i.item)], amount: i.amount }).collect();
                    if old.recipes_of(item_id).any(|r| same_recipe(r, recipe.output, &inputs)) {
                        continue;
                    }
                    let id = Uuid::new_v4().to_string();
                    // An item that already had recipes keeps the default the player chose.
                    sqlx::query("INSERT INTO recipes (id, item_id, output, is_default, created_at) VALUES (?, ?, ?, ?, ?)")
                        .bind(&id)
                        .bind(item_id.to_string())
                        .bind(recipe.output)
                        .bind((recipe.default && !had_recipes) as i64)
                        .bind(now)
                        .execute(&mut *tx)
                        .await?;
                    for input in &inputs {
                        sqlx::query("INSERT INTO recipe_details (id, recipe_id, item_id, input) VALUES (?, ?, ?, ?)")
                            .bind(Uuid::new_v4().to_string())
                            .bind(&id)
                            .bind(input.item_id.to_string())
                            .bind(input.amount)
                            .execute(&mut *tx)
                            .await?;
                    }
                    recipes += 1;
                }
            }

            // A merge leaves the player's own inventory and plans as they are.
            if merge_into.is_none()
                && let Some(progress) = &file.progress
            {
                for owned in &progress.inventory {
                    sqlx::query(
                        "INSERT INTO inventories (id, item_id, amount) VALUES (?, ?, ?)
                         ON CONFLICT (item_id) DO UPDATE SET amount = excluded.amount",
                    )
                    .bind(Uuid::new_v4().to_string())
                    .bind(ids[&name_key(&owned.item)].to_string())
                    .bind(owned.amount)
                    .execute(&mut *tx)
                    .await?;
                }
                for plan in &progress.plans {
                    sqlx::query("INSERT INTO plans (id, item_id, target, note, created_at) VALUES (?, ?, ?, ?, ?)")
                        .bind(Uuid::new_v4().to_string())
                        .bind(ids[&name_key(&plan.item)].to_string())
                        .bind(plan.target)
                        .bind(plan.note.trim())
                        .bind(now)
                        .execute(&mut *tx)
                        .await?;
                }
            }
            tx.commit().await?;
            anyhow::Ok(Imported { game, title, merged: merge_into.is_some(), items, recipes })
        });

        match &result {
            Ok(_) => {
                if let Some(old_id) = replace {
                    let _ = std::fs::remove_dir_all(self.data_dir.join("images").join(old_id.to_string()));
                }
            }
            Err(_) => {
                for relative in written.into_inner() {
                    self.remove_image(Some(relative));
                }
                if merge_into.is_none() {
                    let _ = std::fs::remove_dir(self.data_dir.join("images").join(game.to_string()));
                }
            }
        }
        result
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

/// Whether an existing recipe makes the same amount from the same inputs.
fn same_recipe(recipe: &Recipe, output: f64, inputs: &[Input]) -> bool {
    let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
    close(recipe.output, output)
        && recipe.inputs.len() == inputs.len()
        && inputs.iter().all(|i| recipe.inputs.iter().any(|r| r.item_id == i.item_id && close(r.amount, i.amount)))
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

    /// Hollowdeep: Torch = 1 Stick + 1 Coal makes 4, Coal has an icon, 5 Coal owned, a plan for Torches.
    fn hollowdeep(dir: &Path, db: &Db) -> Uuid {
        let game = db.add_game("Hollowdeep").unwrap();
        let (stick, coal, torch) = (db.add_item(game, "Stick").unwrap(), db.add_item(game, "Coal").unwrap(), db.add_item(game, "Torch").unwrap());
        db.add_recipe(torch, 4.0, &[Input { item_id: stick, amount: 1.0 }, Input { item_id: coal, amount: 1.0 }]).unwrap();
        db.set_inventory(coal, 5.0).unwrap();
        db.add_plan(torch, 24.0, "Light the east mine").unwrap();
        let picture = dir.join("coal.png");
        image::RgbaImage::from_pixel(128, 128, image::Rgba([10, 20, 30, 255])).save(&picture).unwrap();
        db.set_item_icon(game, coal, &picture).unwrap();
        db.set_game_thumbnail(game, &picture).unwrap();
        game
    }

    fn id_of(data: &GameData, name: &str) -> Uuid {
        data.items.iter().find(|i| i.name == name).unwrap_or_else(|| panic!("no item {name}")).id
    }

    fn exported(dir: &Path, db: &Db, game: Uuid, include_progress: bool) -> GameFile {
        let path = dir.join("exported.zip");
        db.export_game(game, include_progress, &path).unwrap();
        GameFile::read(&path).unwrap()
    }

    #[test]
    fn an_exported_game_imports_elsewhere_with_its_recipes_and_pictures_but_not_progress() {
        let (dir, db) = db();
        let game = hollowdeep(dir.path(), &db);
        let file = exported(dir.path(), &db, game, false);
        assert!(file.progress.is_none());

        let (_other_dir, other) = super::tests::db();
        let imported = other.import_game(&file, None).unwrap();
        assert_eq!((imported.title.as_str(), imported.merged, imported.items, imported.recipes), ("Hollowdeep", false, 3, 1));
        let data = other.load_game(imported.game).unwrap();
        let (stick, coal, torch) = (id_of(&data, "Stick"), id_of(&data, "Coal"), id_of(&data, "Torch"));
        let recipe = data.default_recipe(torch).unwrap();
        assert_eq!(recipe.output, 4.0);
        assert_eq!(recipe.inputs, vec![Input { item_id: stick, amount: 1.0 }, Input { item_id: coal, amount: 1.0 }]);
        assert!(data.inventory.is_empty() && data.plans.is_empty());

        // The pictures arrive in the new game's own folder, byte for byte.
        let icon = data.item(coal).unwrap().icon.clone().unwrap();
        assert!(icon.starts_with(&format!("images/{}/", imported.game)), "{icon}");
        let original = db.load_game(game).unwrap().items.iter().find(|i| i.name == "Coal").unwrap().icon.clone().unwrap();
        assert_eq!(std::fs::read(other.image_path(&icon)).unwrap(), std::fs::read(db.image_path(&original)).unwrap());
        let thumbnail = other.games().unwrap()[0].thumbnail.clone().unwrap();
        assert!(other.image_path(&thumbnail).exists());
    }

    #[test]
    fn progress_travels_only_when_asked_for() {
        let (dir, db) = db();
        let game = hollowdeep(dir.path(), &db);
        let file = exported(dir.path(), &db, game, true);
        let (_other_dir, other) = super::tests::db();
        let imported = other.import_game(&file, None).unwrap();
        let data = other.load_game(imported.game).unwrap();
        assert_eq!(data.owned(id_of(&data, "Coal")), 5.0);
        assert_eq!((data.plans.len(), data.plans[0].target, data.plans[0].note.as_str()), (1, 24.0, "Light the east mine"));
    }

    #[test]
    fn a_title_already_in_use_needs_a_decision_and_keep_both_numbers_the_new_game() {
        let (dir, db) = db();
        let game = hollowdeep(dir.path(), &db);
        let file = exported(dir.path(), &db, game, false);
        assert!(db.import_game(&file, None).unwrap_err().to_string().contains("already a game called Hollowdeep"));
        assert_eq!(db.games().unwrap().len(), 1);

        assert_eq!(db.import_game(&file, Some(OnClash::KeepBoth)).unwrap().title, "Hollowdeep (2)");
        assert_eq!(db.import_game(&file, Some(OnClash::KeepBoth)).unwrap().title, "Hollowdeep (3)");
        let titles: Vec<String> = db.games().unwrap().into_iter().map(|g| g.title).collect();
        assert_eq!(titles, ["Hollowdeep", "Hollowdeep (2)", "Hollowdeep (3)"]);
    }

    #[test]
    fn replace_discards_the_existing_game_and_its_progress() {
        let (dir, db) = db();
        let game = hollowdeep(dir.path(), &db);
        let file = exported(dir.path(), &db, game, false);
        // Changes made after the export are what Replace throws away.
        let extra = db.add_item(game, "Lantern").unwrap();
        db.set_inventory(extra, 2.0).unwrap();
        let old_icon = db.load_game(game).unwrap().items.iter().find_map(|i| i.icon.clone()).unwrap();

        let imported = db.import_game(&file, Some(OnClash::Replace)).unwrap();
        let games = db.games().unwrap();
        assert_eq!((games.len(), games[0].id, games[0].title.as_str()), (1, imported.game, "Hollowdeep"));
        let data = db.load_game(imported.game).unwrap();
        assert_eq!(data.items.len(), 3);
        assert!(data.inventory.is_empty() && data.plans.is_empty());
        assert!(!db.image_path(&old_icon).exists());
        assert!(db.image_path(data.items.iter().find_map(|i| i.icon.as_ref()).unwrap()).exists());
    }

    #[test]
    fn merge_adds_what_is_missing_and_changes_nothing_that_is_there() {
        let (dir, db) = db();
        let source = hollowdeep(dir.path(), &db);
        // The file: Hollowdeep plus a Lantern, a second Torch recipe, and progress.
        let resin = db.add_item(source, "Resin").unwrap();
        let lantern = db.add_item(source, "Lantern").unwrap();
        let data = db.load_game(source).unwrap();
        let (stick, torch) = (id_of(&data, "Stick"), id_of(&data, "Torch"));
        let cheap = db.add_recipe(torch, 2.0, &[Input { item_id: stick, amount: 1.0 }, Input { item_id: resin, amount: 1.0 }]).unwrap();
        db.set_default_recipe(cheap).unwrap();
        db.add_recipe(lantern, 1.0, &[Input { item_id: torch, amount: 1.0 }]).unwrap();
        let file = exported(dir.path(), &db, source, true);

        // The player's own game: the plain Hollowdeep, with its own progress and no pictures.
        let (_mine_dir, mine) = super::tests::db();
        let game = mine.add_game("hollowdeep").unwrap();
        let (stick, coal, torch) = (mine.add_item(game, "Stick").unwrap(), mine.add_item(game, "coal").unwrap(), mine.add_item(game, "Torch").unwrap());
        let own = mine.add_recipe(torch, 4.0, &[Input { item_id: coal, amount: 1.0 }, Input { item_id: stick, amount: 1.0 }]).unwrap();
        mine.set_inventory(coal, 1.0).unwrap();

        let imported = mine.import_game(&file, Some(OnClash::Merge)).unwrap();
        assert_eq!((imported.game, imported.merged, imported.items, imported.recipes), (game, true, 2, 2));
        let data = mine.load_game(game).unwrap();
        assert_eq!(data.items.len(), 5);
        // The identical Torch recipe is not doubled, and the player's default stays the default.
        assert_eq!(data.recipes_of(torch).count(), 2);
        assert_eq!(data.default_recipe(torch).unwrap().id, own);
        // A new item's recipe arrives with its default.
        assert!(data.default_recipe(id_of(&data, "Lantern")).is_some());
        // The file's progress is ignored; the item that had no picture gets the file's.
        assert_eq!(data.owned(coal), 1.0);
        assert!(data.plans.is_empty());
        assert!(data.item(coal).unwrap().icon.is_some());
        assert_eq!(mine.games().unwrap()[0].title, "hollowdeep");

        // Merging the same file again adds nothing.
        let again = mine.import_game(&file, Some(OnClash::Merge)).unwrap();
        assert_eq!((again.items, again.recipes), (0, 0));
    }

    #[test]
    fn a_failed_import_leaves_nothing_behind() {
        let (dir, db) = db();
        let game = hollowdeep(dir.path(), &db);
        let mut file = exported(dir.path(), &db, game, false);
        // A file edited by hand into something the database refuses, past the reader's checks.
        file.title = "Broken".into();
        file.items[2].recipes[0].output = -1.0;
        assert!(db.import_game(&file, None).is_err());
        assert_eq!(db.games().unwrap().len(), 1);
        let folders = std::fs::read_dir(dir.path().join("images")).unwrap().count();
        assert_eq!(folders, 1, "only the first game's pictures remain");
    }

    #[test]
    fn game_titles_are_unique_ignoring_case() {
        let (_dir, db) = db();
        let game = db.add_game("Hollowdeep").unwrap();
        assert!(db.add_game(" hollowdeep ").is_err());
        let other = db.add_game("Saltwind Isles").unwrap();
        assert!(db.rename_game(other, "HOLLOWDEEP").is_err());
        // A game may keep, or re-case, its own title.
        db.rename_game(game, "hollowdeep").unwrap();
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
