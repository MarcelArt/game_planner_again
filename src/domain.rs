//! The domain types. Vocabulary follows `CONTEXT.md`.

use std::collections::HashMap;

use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub id: Uuid,
    pub title: String,
    /// Path relative to the data folder.
    pub thumbnail: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    /// Path relative to the data folder.
    pub icon: Option<String>,
}

/// One ingredient line of a recipe.
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub item_id: Uuid,
    /// Amount one craft consumes.
    pub amount: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    pub id: Uuid,
    /// The item this recipe makes.
    pub item_id: Uuid,
    /// Amount one craft yields.
    pub output: f64,
    pub is_default: bool,
    pub inputs: Vec<Input>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub id: Uuid,
    pub item_id: Uuid,
    pub target: f64,
    pub note: String,
}

/// Everything belonging to one game, as loaded from the database.
/// `plans` are oldest first; `recipes` are oldest first.
#[derive(Clone, Debug, Default)]
pub struct GameData {
    pub items: Vec<Item>,
    pub recipes: Vec<Recipe>,
    pub inventory: HashMap<Uuid, f64>,
    pub plans: Vec<Plan>,
}

impl GameData {
    pub fn item(&self, id: Uuid) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn item_name(&self, id: Uuid) -> &str {
        self.item(id).map(|i| i.name.as_str()).unwrap_or("?")
    }

    pub fn owned(&self, id: Uuid) -> f64 {
        self.inventory.get(&id).copied().unwrap_or(0.0)
    }

    pub fn recipes_of(&self, item_id: Uuid) -> impl Iterator<Item = &Recipe> {
        self.recipes.iter().filter(move |r| r.item_id == item_id)
    }

    pub fn default_recipe(&self, item_id: Uuid) -> Option<&Recipe> {
        self.recipes_of(item_id).find(|r| r.is_default)
    }

    /// A raw material has no recipe.
    pub fn is_raw(&self, item_id: Uuid) -> bool {
        self.recipes_of(item_id).next().is_none()
    }

    /// Items with a recipe that takes `item_id` as an input.
    pub fn used_in(&self, item_id: Uuid) -> Vec<&Item> {
        let mut out: Vec<&Item> = Vec::new();
        for r in &self.recipes {
            if r.inputs.iter().any(|i| i.item_id == item_id)
                && let Some(item) = self.item(r.item_id)
                && !out.iter().any(|o| o.id == item.id)
            {
                out.push(item);
            }
        }
        out
    }
}

/// Formats an amount without trailing zeros: `4`, `5.625`, `0.125`.
pub fn fmt_amount(n: f64) -> String {
    if (n - n.round()).abs() < 1e-9 {
        format!("{}", n.round() as i64)
    } else {
        let s = format!("{n:.3}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}
