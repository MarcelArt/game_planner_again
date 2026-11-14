//! The calculation: from a game's plans, recipes and inventory to its Requirements.
//!
//! Pure: no database, no UI.
//!
//! Rules (see `CONTEXT.md`):
//! - All plans of the game are combined. Targets of plans for the same item add up.
//! - Default recipes are expanded all the way down; inventory is subtracted at every level.
//! - Demand for an item is summed first and then rounded up to whole crafts, so surplus is shared.
//! - An item that a chain of default recipes leads back to is a recipe loop: it is treated as
//!   something to gather, everywhere, and flagged.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::domain::{GameData, Plan, Recipe};

const EPS: f64 = 1e-9;

/// A raw material (or a recipe-loop item) the plans need.
#[derive(Clone, Debug, PartialEq)]
pub struct GatherRow {
    pub item_id: Uuid,
    pub need: f64,
    pub have: f64,
    pub missing: f64,
    /// For a recipe loop: the items of the loop, starting and ending with this item.
    pub recipe_loop: Option<Vec<Uuid>>,
    /// The plans this row serves, oldest first.
    pub plans: Vec<Uuid>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CraftInput {
    pub item_id: Uuid,
    pub per_craft: f64,
    /// Amount all the needed crafts consume.
    pub total: f64,
    pub have: f64,
}

/// An item the plans need that is made by its default recipe.
#[derive(Clone, Debug, PartialEq)]
pub struct CraftRow {
    pub item_id: Uuid,
    pub recipe_id: Uuid,
    pub output: f64,
    pub need: f64,
    pub have: f64,
    pub missing: f64,
    /// Whole crafts still to perform.
    pub crafts: u64,
    pub makes: f64,
    pub surplus: f64,
    pub inputs: Vec<CraftInput>,
    /// How many of the needed crafts inventory covers right now.
    pub can_craft: u64,
    pub plans: Vec<Uuid>,
}

impl CraftRow {
    /// A needed craft whose inputs are all in inventory.
    pub fn ready(&self) -> bool {
        self.crafts > 0 && self.can_craft > 0
    }
}

/// Gather rows and craft rows. Craft rows are inputs-first: a row never precedes a row it consumes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Requirements {
    pub gather: Vec<GatherRow>,
    pub craft: Vec<CraftRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanProgress {
    pub plan_id: Uuid,
    /// Inventory assigned to this plan; an item's inventory goes to its oldest plan first.
    pub assigned: f64,
    pub reached: bool,
}

/// How much inventory each plan can count, oldest plan first.
pub fn plan_progress(data: &GameData) -> Vec<PlanProgress> {
    let mut left: HashMap<Uuid, f64> = HashMap::new();
    data.plans
        .iter()
        .map(|p| {
            let pool = left.entry(p.item_id).or_insert_with(|| data.owned(p.item_id));
            let assigned = p.target.min(*pool);
            *pool -= assigned;
            PlanProgress { plan_id: p.id, assigned, reached: assigned + EPS >= p.target }
        })
        .collect()
}

/// How many crafts of `recipe` inventory covers.
pub fn craftable(recipe: &Recipe, data: &GameData) -> u64 {
    recipe
        .inputs
        .iter()
        .map(|i| (data.owned(i.item_id) / i.amount + EPS).floor().max(0.0) as u64)
        .min()
        .unwrap_or(0)
}

pub fn requirements(data: &GameData) -> Requirements {
    let loops = find_loops(data);
    let recipe_of = |id: Uuid| -> Option<&Recipe> {
        if loops.contains_key(&id) { None } else { data.default_recipe(id) }
    };

    // Post-order over the (now acyclic) graph: inputs before the items made from them.
    let mut order: Vec<Uuid> = Vec::new();
    let mut seen: HashSet<Uuid> = HashSet::new();
    fn visit<'a>(
        id: Uuid,
        recipe_of: &dyn Fn(Uuid) -> Option<&'a Recipe>,
        seen: &mut HashSet<Uuid>,
        order: &mut Vec<Uuid>,
    ) {
        if !seen.insert(id) {
            return;
        }
        if let Some(r) = recipe_of(id) {
            for i in &r.inputs {
                visit(i.item_id, recipe_of, seen, order);
            }
        }
        order.push(id);
    }
    for p in &data.plans {
        visit(p.item_id, &recipe_of, &mut seen, &mut order);
    }

    // Which plans each item serves.
    let mut serves: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for p in &data.plans {
        let mut stack = vec![p.item_id];
        let mut reached: HashSet<Uuid> = HashSet::new();
        while let Some(id) = stack.pop() {
            if !reached.insert(id) {
                continue;
            }
            serves.entry(id).or_default().push(p.id);
            if let Some(r) = recipe_of(id) {
                stack.extend(r.inputs.iter().map(|i| i.item_id));
            }
        }
    }

    // Sum demand from the targets down, netting inventory and rounding to whole crafts per item.
    let mut demand: HashMap<Uuid, f64> = HashMap::new();
    for p in &data.plans {
        *demand.entry(p.item_id).or_default() += p.target;
    }
    let mut out = Requirements::default();
    for &id in order.iter().rev() {
        let need = demand.get(&id).copied().unwrap_or(0.0);
        if need <= EPS {
            continue;
        }
        let have = data.owned(id);
        let missing = (need - have).max(0.0);
        let plans = serves.remove(&id).unwrap_or_default();
        match recipe_of(id) {
            None => out.gather.push(GatherRow {
                item_id: id,
                need,
                have,
                missing,
                recipe_loop: loops.get(&id).cloned(),
                plans,
            }),
            Some(r) => {
                let crafts = (missing / r.output - EPS).ceil().max(0.0) as u64;
                let inputs = r
                    .inputs
                    .iter()
                    .map(|i| {
                        let total = i.amount * crafts as f64;
                        *demand.entry(i.item_id).or_default() += total;
                        CraftInput { item_id: i.item_id, per_craft: i.amount, total, have: data.owned(i.item_id) }
                    })
                    .collect();
                let makes = crafts as f64 * r.output;
                out.craft.push(CraftRow {
                    item_id: id,
                    recipe_id: r.id,
                    output: r.output,
                    need,
                    have,
                    missing,
                    crafts,
                    makes,
                    surplus: makes - missing,
                    inputs,
                    can_craft: craftable(r, data).min(crafts),
                    plans,
                });
            }
        }
    }
    out.gather.reverse();
    out.craft.reverse();
    out
}

/// Items that a chain of default recipes, followed from the plans, leads back to.
/// Maps each such item to the loop, written from the item back to itself.
fn find_loops(data: &GameData) -> HashMap<Uuid, Vec<Uuid>> {
    #[derive(PartialEq)]
    enum Mark {
        Open,
        Done,
    }
    fn dfs(
        id: Uuid,
        data: &GameData,
        marks: &mut HashMap<Uuid, Mark>,
        path: &mut Vec<Uuid>,
        loops: &mut HashMap<Uuid, Vec<Uuid>>,
    ) {
        match marks.get(&id) {
            Some(Mark::Done) => return,
            Some(Mark::Open) => {
                let start = path.iter().position(|p| *p == id).unwrap_or(0);
                let mut cycle = path[start..].to_vec();
                cycle.push(id);
                loops.entry(id).or_insert(cycle);
                return;
            }
            None => {}
        }
        marks.insert(id, Mark::Open);
        path.push(id);
        if let Some(r) = data.default_recipe(id) {
            for i in &r.inputs {
                dfs(i.item_id, data, marks, path, loops);
            }
        }
        path.pop();
        marks.insert(id, Mark::Done);
    }

    let mut marks = HashMap::new();
    let mut loops = HashMap::new();
    let mut path = Vec::new();
    for Plan { item_id, .. } in &data.plans {
        dfs(*item_id, data, &mut marks, &mut path, &mut loops);
    }
    loops
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Input, Item};

    /// A small builder so tests read like the scenarios they describe.
    #[derive(Default)]
    struct World {
        data: GameData,
        ids: HashMap<&'static str, Uuid>,
    }

    impl World {
        fn id(&mut self, name: &'static str) -> Uuid {
            if let Some(id) = self.ids.get(name) {
                return *id;
            }
            let id = Uuid::new_v4();
            self.ids.insert(name, id);
            self.data.items.push(Item { id, name: name.into(), icon: None });
            id
        }

        fn recipe(&mut self, item: &'static str, output: f64, inputs: &[(&'static str, f64)]) -> &mut Self {
            let item_id = self.id(item);
            let inputs = inputs.iter().map(|(n, a)| Input { item_id: self.id(n), amount: *a }).collect();
            let is_default = self.data.default_recipe(item_id).is_none();
            self.data.recipes.push(Recipe { id: Uuid::new_v4(), item_id, output, is_default, inputs });
            self
        }

        fn own(&mut self, item: &'static str, amount: f64) -> &mut Self {
            let id = self.id(item);
            self.data.inventory.insert(id, amount);
            self
        }

        fn plan(&mut self, item: &'static str, target: f64) -> Uuid {
            let item_id = self.id(item);
            let id = Uuid::new_v4();
            self.data.plans.push(Plan { id, item_id, target, note: String::new() });
            id
        }

        fn craft_row(&self, item: &str) -> CraftRow {
            let id = self.ids[item];
            requirements(&self.data).craft.into_iter().find(|r| r.item_id == id).unwrap_or_else(|| panic!("no craft row for {item}"))
        }

        fn gather_row(&self, item: &str) -> GatherRow {
            let id = self.ids[item];
            requirements(&self.data).gather.into_iter().find(|r| r.item_id == id).unwrap_or_else(|| panic!("no gather row for {item}"))
        }
    }

    fn torches() -> World {
        let mut w = World::default();
        w.recipe("Torch", 4.0, &[("Stick", 1.0), ("Coal", 1.0)]);
        w
    }

    #[test]
    fn crafts_are_whole_and_leave_surplus() {
        let mut w = torches();
        w.plan("Torch", 5.0);
        let torch = w.craft_row("Torch");
        assert_eq!(torch.crafts, 2);
        assert_eq!(torch.makes, 8.0);
        assert_eq!(torch.surplus, 3.0);
        assert_eq!(w.gather_row("Stick").missing, 2.0);
        assert_eq!(w.gather_row("Coal").missing, 2.0);
    }

    #[test]
    fn owned_target_items_count_toward_the_target() {
        let mut w = World::default();
        w.recipe("Sword", 1.0, &[("Iron Bar", 2.0)]);
        w.own("Sword", 1.0);
        w.plan("Sword", 3.0);
        assert_eq!(w.craft_row("Sword").crafts, 2);
        assert_eq!(w.gather_row("Iron Bar").need, 4.0);
    }

    #[test]
    fn expansion_is_recursive_and_nets_inventory_at_every_level() {
        let mut w = World::default();
        w.recipe("Sword", 1.0, &[("Iron Bar", 2.0)]).recipe("Iron Bar", 1.0, &[("Iron Ore", 3.0)]);
        w.own("Iron Bar", 3.0);
        w.plan("Sword", 4.0);
        // 8 bars needed, 3 owned: 5 crafts of bars, so 15 ore rather than 24.
        assert_eq!(w.craft_row("Iron Bar").crafts, 5);
        assert_eq!(w.gather_row("Iron Ore").missing, 15.0);
    }

    #[test]
    fn plans_for_the_same_item_add_up() {
        let mut w = torches();
        w.plan("Torch", 5.0);
        w.plan("Torch", 10.0);
        assert_eq!(w.craft_row("Torch").need, 15.0);
        assert_eq!(w.craft_row("Torch").crafts, 4);
    }

    #[test]
    fn surplus_is_shared_across_plans() {
        let mut w = torches();
        w.recipe("Lantern", 1.0, &[("Torch", 1.0), ("Copper", 4.0)]);
        w.plan("Torch", 5.0);
        w.plan("Lantern", 3.0);
        // 5 + 3 = 8 torches is 2 crafts, not 2 + 1.
        assert_eq!(w.craft_row("Torch").crafts, 2);
        assert_eq!(w.craft_row("Torch").surplus, 0.0);
    }

    #[test]
    fn rows_list_the_plans_they_serve() {
        let mut w = torches();
        w.recipe("Lantern", 1.0, &[("Torch", 1.0), ("Copper", 4.0)]);
        let a = w.plan("Torch", 5.0);
        let b = w.plan("Lantern", 3.0);
        assert_eq!(w.craft_row("Torch").plans, vec![a, b]);
        assert_eq!(w.gather_row("Copper").plans, vec![b]);
    }

    #[test]
    fn craft_rows_are_inputs_first() {
        let mut w = World::default();
        w.recipe("Pickaxe", 1.0, &[("Ingot", 3.0), ("Stick", 2.0)])
            .recipe("Stick", 4.0, &[("Plank", 2.0)])
            .recipe("Plank", 4.0, &[("Log", 1.0)])
            .recipe("Ingot", 1.0, &[("Ore", 1.0)]);
        w.plan("Pickaxe", 1.0);
        let craft = requirements(&w.data).craft;
        let pos = |n: &str| craft.iter().position(|r| r.item_id == w.ids[n]).unwrap();
        assert!(pos("Plank") < pos("Stick"));
        assert!(pos("Stick") < pos("Pickaxe"));
        assert!(pos("Ingot") < pos("Pickaxe"));
    }

    #[test]
    fn fractional_inputs_stay_fractional_while_crafts_stay_whole() {
        let mut w = World::default();
        w.recipe("Ingot", 1.0, &[("Ore", 1.0), ("Coal", 0.125)]);
        w.plan("Ingot", 5.0);
        assert_eq!(w.craft_row("Ingot").crafts, 5);
        assert_eq!(w.gather_row("Coal").missing, 0.625);
    }

    #[test]
    fn only_the_default_recipe_is_used() {
        let mut w = torches();
        w.recipe("Torch", 2.0, &[("Stick", 1.0), ("Resin", 1.0)]);
        w.plan("Torch", 4.0);
        let req = requirements(&w.data);
        assert!(req.gather.iter().all(|g| g.item_id != w.ids["Resin"]));
        assert_eq!(w.craft_row("Torch").crafts, 1);
    }

    #[test]
    fn a_recipe_loop_is_gathered_and_flagged() {
        let mut w = World::default();
        w.recipe("Copper Ingot", 9.0, &[("Copper Block", 1.0)])
            .recipe("Copper Block", 1.0, &[("Copper Ingot", 9.0)])
            .recipe("Lantern", 1.0, &[("Copper Ingot", 4.0)]);
        w.own("Copper Ingot", 6.0);
        w.plan("Lantern", 4.0);
        let ingot = w.gather_row("Copper Ingot");
        assert_eq!(ingot.missing, 10.0);
        let (i, b) = (w.ids["Copper Ingot"], w.ids["Copper Block"]);
        assert_eq!(ingot.recipe_loop, Some(vec![i, b, i]));
        // The block is only reachable through the loop, so nothing asks for it.
        let req = requirements(&w.data);
        assert!(req.craft.iter().all(|r| r.item_id != b));
    }

    #[test]
    fn a_loop_item_still_receives_demand_from_the_other_side_of_the_loop() {
        let mut w = World::default();
        w.recipe("Copper Ingot", 9.0, &[("Copper Block", 1.0)])
            .recipe("Copper Block", 1.0, &[("Copper Ingot", 9.0)]);
        w.plan("Copper Ingot", 2.0);
        w.plan("Copper Block", 1.0);
        // Ingot is the loop item: gathered. The block is crafted from 9 gathered ingots.
        assert_eq!(w.craft_row("Copper Block").crafts, 1);
        assert_eq!(w.gather_row("Copper Ingot").need, 11.0);
    }

    #[test]
    fn a_raw_material_covered_by_inventory_is_listed_with_nothing_missing() {
        let mut w = World::default();
        w.recipe("Plank", 4.0, &[("Log", 1.0)]);
        w.own("Log", 2.0);
        w.plan("Plank", 4.0);
        let log = w.gather_row("Log");
        assert_eq!((log.need, log.have, log.missing), (1.0, 2.0, 0.0));
    }

    #[test]
    fn ready_counts_the_crafts_inventory_covers_now() {
        let mut w = torches();
        w.own("Stick", 3.0).own("Coal", 5.0);
        w.plan("Torch", 40.0);
        let torch = w.craft_row("Torch");
        assert_eq!(torch.crafts, 10);
        assert_eq!(torch.can_craft, 3);
        assert!(torch.ready());

        let mut w = torches();
        w.own("Coal", 5.0);
        w.plan("Torch", 4.0);
        assert!(!w.craft_row("Torch").ready());
    }

    #[test]
    fn inventory_goes_to_the_oldest_plan_first() {
        let mut w = torches();
        w.own("Torch", 7.0);
        let first = w.plan("Torch", 5.0);
        let second = w.plan("Torch", 10.0);
        let progress = plan_progress(&w.data);
        assert_eq!(progress[0], PlanProgress { plan_id: first, assigned: 5.0, reached: true });
        assert_eq!(progress[1], PlanProgress { plan_id: second, assigned: 2.0, reached: false });
    }
}
