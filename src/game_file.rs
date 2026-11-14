//! The game file: one game in a zip archive, as `game.json` plus its pictures.
//!
//! Nothing in a game file carries an id: recipes, inventory and plans name their items, so a file
//! can be imported into any data folder and merged into a game by item name.

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Write};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;

const FORMAT: &str = "game-planner";
/// Raised whenever `game.json` changes in a way older versions of the app cannot read.
const VERSION: u32 = 1;
const MANIFEST: &str = "game.json";
/// Nothing in a game file is expected to come near this; it bounds what a hostile archive can
/// make the app hold in memory.
const MAX_ENTRY: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GameFile {
    format: String,
    version: u32,
    pub title: String,
    /// Name of the thumbnail's entry in the archive.
    #[serde(default)]
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub items: Vec<FileItem>,
    /// The player's inventory and plans, when they chose to export them.
    #[serde(default)]
    pub progress: Option<FileProgress>,
    /// Archive entry name to file contents.
    #[serde(skip)]
    pub pictures: HashMap<String, Vec<u8>>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    /// Name of the icon's entry in the archive.
    #[serde(default)]
    pub picture: Option<String>,
    /// Oldest first.
    #[serde(default)]
    pub recipes: Vec<FileRecipe>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FileRecipe {
    pub output: f64,
    #[serde(default)]
    pub default: bool,
    pub inputs: Vec<FileInput>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FileInput {
    pub item: String,
    pub amount: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FileProgress {
    #[serde(default)]
    pub inventory: Vec<FileOwned>,
    /// Oldest first.
    #[serde(default)]
    pub plans: Vec<FilePlan>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FileOwned {
    pub item: String,
    pub amount: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FilePlan {
    pub item: String,
    pub target: f64,
    #[serde(default)]
    pub note: String,
}

#[derive(Deserialize)]
struct Header {
    format: String,
    version: u32,
}

/// How item names are compared: ignoring case and surrounding spaces.
pub fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

/// The extension of a picture the app can show, lower-cased.
pub fn picture_ext(name: &str) -> Option<String> {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| matches!(e.as_str(), "png" | "jpg" | "jpeg" | "webp"))
}

impl GameFile {
    pub fn new(title: impl Into<String>) -> Self {
        Self { format: FORMAT.into(), version: VERSION, title: title.into(), ..Default::default() }
    }

    /// Puts a picture into the file and returns the entry name to refer to it by.
    /// `stem` must be unique within the file; `source` only supplies the extension.
    pub fn add_picture(&mut self, stem: &str, source: &str, bytes: Vec<u8>) -> Option<String> {
        let name = format!("pictures/{stem}.{}", picture_ext(source)?);
        self.pictures.insert(name.clone(), bytes);
        Some(name)
    }

    /// The contents and extension of a picture, if the file really holds it.
    pub fn picture(&self, name: Option<&String>) -> Option<(&[u8], String)> {
        let name = name?;
        Some((self.pictures.get(name)?.as_slice(), picture_ext(name)?))
    }

    pub fn write(&self, dest: &Path) -> Result<()> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        zip.start_file(MANIFEST, SimpleFileOptions::default())?;
        zip.write_all(&serde_json::to_vec_pretty(self)?)?;
        // Pictures are already compressed.
        let stored = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        let mut names: Vec<&String> = self.pictures.keys().collect();
        names.sort();
        for name in names {
            zip.start_file(name.as_str(), stored)?;
            zip.write_all(&self.pictures[name])?;
        }
        let bytes = zip.finish()?.into_inner();
        std::fs::write(dest, bytes).with_context(|| format!("writing {}", dest.display()))
    }

    /// Reads and checks a game file. A picture the manifest names but the archive lacks is
    /// tolerated: that item simply has no picture.
    pub fn read(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
        let not_a_game_file = || format!("{} is not a game file.", path.file_name().unwrap_or_default().to_string_lossy());
        let mut zip = zip::ZipArchive::new(file).ok().with_context(not_a_game_file)?;
        let manifest = read_entry(&mut zip, MANIFEST).with_context(not_a_game_file)?;
        let header: Header = serde_json::from_slice(&manifest).ok().with_context(not_a_game_file)?;
        if header.format != FORMAT {
            bail!(not_a_game_file());
        }
        if header.version > VERSION {
            bail!("This game file was made by a newer version of Game Planner. Update the app to import it.");
        }
        let mut game: GameFile = serde_json::from_slice(&manifest).context("This game file is damaged")?;
        game.check()?;

        let wanted: Vec<String> = game.thumbnail.iter().chain(game.items.iter().filter_map(|i| i.picture.as_ref())).cloned().collect();
        for name in wanted {
            if picture_ext(&name).is_some()
                && let Some(bytes) = read_entry(&mut zip, &name)
            {
                game.pictures.insert(name, bytes);
            }
        }
        Ok(game)
    }

    /// Refuses contents the app itself would never have let the player enter, and tidies names.
    fn check(&mut self) -> Result<()> {
        fn damaged<T>(what: String) -> Result<T> {
            bail!("This game file is damaged: {what}.")
        }
        self.title = self.title.trim().to_string();
        if self.title.is_empty() {
            return damaged("the game has no title".into());
        }
        let mut names = HashSet::new();
        for item in &mut self.items {
            item.name = item.name.trim().to_string();
            if item.name.is_empty() {
                return damaged("an item has no name".into());
            }
            if !names.insert(name_key(&item.name)) {
                return damaged(format!("{} is listed twice", item.name));
            }
        }
        for item in &mut self.items {
            for recipe in &item.recipes {
                if !(recipe.output > 0.0 && recipe.output.is_finite()) {
                    return damaged(format!("a recipe for {} makes nothing", item.name));
                }
                if recipe.inputs.is_empty() {
                    return damaged(format!("a recipe for {} has no inputs", item.name));
                }
                for input in &recipe.inputs {
                    if !names.contains(&name_key(&input.item)) {
                        return damaged(format!("a recipe for {} uses the unknown item {}", item.name, input.item));
                    }
                    if name_key(&input.item) == name_key(&item.name) {
                        return damaged(format!("a recipe for {} uses {} itself", item.name, item.name));
                    }
                    if !(input.amount > 0.0 && input.amount.is_finite()) {
                        return damaged(format!("a recipe for {} has an input without an amount", item.name));
                    }
                }
            }
            // Exactly one default recipe per item that has recipes.
            let default = item.recipes.iter().position(|r| r.default).unwrap_or(0);
            for (i, recipe) in item.recipes.iter_mut().enumerate() {
                recipe.default = i == default;
            }
        }
        if let Some(progress) = &self.progress {
            for owned in &progress.inventory {
                if !names.contains(&name_key(&owned.item)) {
                    return damaged(format!("the inventory names the unknown item {}", owned.item));
                }
                if !(owned.amount >= 0.0 && owned.amount.is_finite()) {
                    return damaged(format!("the inventory of {} is not an amount", owned.item));
                }
            }
            for plan in &progress.plans {
                if !names.contains(&name_key(&plan.item)) {
                    return damaged(format!("a plan names the unknown item {}", plan.item));
                }
                if !(plan.target > 0.0 && plan.target.is_finite()) {
                    return damaged(format!("a plan for {} has no target", plan.item));
                }
            }
        }
        Ok(())
    }
}

fn read_entry(zip: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Option<Vec<u8>> {
    let entry = zip.by_name(name).ok()?;
    let mut bytes = Vec::new();
    entry.take(MAX_ENTRY + 1).read_to_end(&mut bytes).ok()?;
    (bytes.len() as u64 <= MAX_ENTRY).then_some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> GameFile {
        let mut game = GameFile::new("Hollowdeep");
        let picture = game.add_picture("0", "images/x/abc.PNG", vec![1, 2, 3]);
        game.items = vec![
            FileItem { name: "Coal".into(), picture, recipes: vec![] },
            FileItem {
                name: "Torch".into(),
                picture: None,
                recipes: vec![FileRecipe { output: 4.0, default: true, inputs: vec![FileInput { item: "Coal".into(), amount: 1.0 }] }],
            },
        ];
        game
    }

    fn written(game: &GameFile) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("game.zip");
        game.write(&path).unwrap();
        (dir, path)
    }

    /// Writes an archive holding only `game.json` with the given text.
    fn raw(manifest: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("game.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
        zip.start_file(MANIFEST, SimpleFileOptions::default()).unwrap();
        zip.write_all(manifest.as_bytes()).unwrap();
        zip.finish().unwrap();
        (dir, path)
    }

    #[test]
    fn a_game_file_reads_back_as_written_pictures_included() {
        let game = sample();
        let (_dir, path) = written(&game);
        let read = GameFile::read(&path).unwrap();
        assert_eq!(read, game);
        assert_eq!(read.picture(read.items[0].picture.as_ref()), Some((&[1u8, 2, 3][..], "png".to_string())));
    }

    #[test]
    fn a_picture_missing_from_the_archive_is_tolerated() {
        let mut game = sample();
        game.pictures.clear();
        let (_dir, path) = written(&game);
        let read = GameFile::read(&path).unwrap();
        assert_eq!(read.items.len(), 2);
        assert_eq!(read.picture(read.items[0].picture.as_ref()), None);
    }

    #[test]
    fn files_that_are_not_game_files_or_are_newer_or_damaged_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let text = dir.path().join("notes.zip");
        std::fs::write(&text, b"hello").unwrap();
        assert!(GameFile::read(&text).unwrap_err().to_string().contains("not a game file"));

        let (_d, other) = raw(r#"{"format":"something-else","version":1,"title":"X"}"#);
        assert!(GameFile::read(&other).unwrap_err().to_string().contains("not a game file"));

        let (_d, newer) = raw(r#"{"format":"game-planner","version":2,"title":"X","shiny":true}"#);
        assert!(GameFile::read(&newer).unwrap_err().to_string().contains("newer version"));

        let (_d, unknown) = raw(
            r#"{"format":"game-planner","version":1,"title":"X","items":[{"name":"Torch","recipes":[{"output":4,"inputs":[{"item":"Coal","amount":1}]}]}]}"#,
        );
        assert!(GameFile::read(&unknown).unwrap_err().to_string().contains("unknown item Coal"));

        let (_d, twice) = raw(r#"{"format":"game-planner","version":1,"title":"X","items":[{"name":"Coal"},{"name":"coal "}]}"#);
        assert!(GameFile::read(&twice).unwrap_err().to_string().contains("listed twice"));
    }

    #[test]
    fn an_item_with_recipes_always_gets_exactly_one_default() {
        let (_d, path) = raw(
            r#"{"format":"game-planner","version":1,"title":"X","items":[{"name":"Coal"},{"name":"Torch","recipes":[
                {"output":4,"inputs":[{"item":"Coal","amount":1}]},{"output":2,"inputs":[{"item":"Coal","amount":2}]}]}]}"#,
        );
        let game = GameFile::read(&path).unwrap();
        assert_eq!(game.items[1].recipes.iter().map(|r| r.default).collect::<Vec<_>>(), vec![true, false]);
    }
}
