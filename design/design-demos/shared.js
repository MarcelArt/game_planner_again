/* Game Planner — shared sample data and domain logic for the three design demos.
   Every demo loads this file so all three show the same content and the same numbers.
   Vocabulary follows CONTEXT.md: Game, Item, Recipe, Input, Output, Craft, Raw material,
   Inventory, Plan, Target, Reached, Ready, Requirements, Surplus, Default recipe, Recipe loop.
   All sample games and items are fictional. */
(function () {
  const games = [
    { id: 'hollowdeep', title: 'Hollowdeep', blurb: 'Mining and crafting survival', hue: 28 },
    { id: 'saltwind', title: 'Saltwind Isles', blurb: 'Sailing and trading', hue: 196, stats: { items: 42, recipes: 31, plans: 2 } },
    { id: 'orbit', title: 'Orbit Foundry', blurb: 'Factory building in orbit', hue: 268, stats: { items: 87, recipes: 64, plans: 0 } },
  ];

  // hue: a per-item colour for fallback icons (icons are optional in the real app).
  const items = [
    { id: 'oak_log', name: 'Oak Log', hue: 30 },
    { id: 'coal', name: 'Coal', hue: 230 },
    { id: 'iron_ore', name: 'Iron Ore', hue: 18 },
    { id: 'resin', name: 'Resin', hue: 42 },
    { id: 'oak_plank', name: 'Oak Plank', hue: 36 },
    { id: 'stick', name: 'Stick', hue: 32 },
    { id: 'torch', name: 'Torch', hue: 44 },
    { id: 'iron_ingot', name: 'Iron Ingot', hue: 210 },
    { id: 'iron_block', name: 'Iron Block', hue: 212 },
    { id: 'iron_pickaxe', name: 'Iron Pickaxe', hue: 200 },
    { id: 'copper_ingot', name: 'Copper Ingot', hue: 16 },
    { id: 'copper_block', name: 'Copper Block', hue: 14 },
    { id: 'lantern', name: 'Lantern', hue: 50 },
  ];

  // A recipe makes `output` of `item` per craft from its inputs. One recipe per item is the default.
  const recipes = [
    { id: 'r_plank', item: 'oak_plank', output: 4, isDefault: true, inputs: [['oak_log', 1]] },
    { id: 'r_stick', item: 'stick', output: 4, isDefault: true, inputs: [['oak_plank', 2]] },
    { id: 'r_torch', item: 'torch', output: 4, isDefault: true, inputs: [['stick', 1], ['coal', 1]] },
    { id: 'r_torch_resin', item: 'torch', output: 2, isDefault: false, inputs: [['stick', 1], ['resin', 1]] },
    { id: 'r_ingot', item: 'iron_ingot', output: 1, isDefault: true, inputs: [['iron_ore', 1], ['coal', 0.125]] },
    { id: 'r_ingot_block', item: 'iron_ingot', output: 9, isDefault: false, inputs: [['iron_block', 1]] },
    { id: 'r_iron_block', item: 'iron_block', output: 1, isDefault: true, inputs: [['iron_ingot', 9]] },
    { id: 'r_pickaxe', item: 'iron_pickaxe', output: 1, isDefault: true, inputs: [['iron_ingot', 3], ['stick', 2]] },
    { id: 'r_cu_ingot', item: 'copper_ingot', output: 9, isDefault: true, inputs: [['copper_block', 1]] },
    { id: 'r_cu_block', item: 'copper_block', output: 1, isDefault: true, inputs: [['copper_ingot', 9]] },
    { id: 'r_lantern', item: 'lantern', output: 1, isDefault: true, inputs: [['torch', 1], ['copper_ingot', 4]] },
  ];

  const inventory = {
    oak_log: 2, coal: 5, iron_ore: 4, oak_plank: 12, stick: 3, torch: 6, iron_ingot: 1, copper_ingot: 6,
  };

  // Oldest first. Several plans may name the same item; their targets add up.
  let plans = [
    { id: 'p1', item: 'torch', target: 24, note: 'Light the east mine' },
    { id: 'p2', item: 'iron_pickaxe', target: 2, note: 'One spare' },
    { id: 'p3', item: 'torch', target: 16, note: 'Base perimeter' },
    { id: 'p4', item: 'lantern', target: 4, note: 'Dock' },
    { id: 'p5', item: 'oak_plank', target: 8, note: 'Floor patch' },
  ];

  const listeners = [];
  const emit = () => listeners.forEach((fn) => fn());
  const EPS = 1e-9;

  const item = (id) => items.find((i) => i.id === id);
  const recipesOf = (id) => recipes.filter((r) => r.item === id);
  const defaultRecipe = (id) => recipes.find((r) => r.item === id && r.isDefault) || null;
  const inv = (id) => inventory[id] || 0;
  const fmt = (n) => (Math.abs(n - Math.round(n)) < EPS ? String(Math.round(n)) : String(+n.toFixed(3)));

  /** Plans with the share of inventory assigned to each (oldest first) and whether it is reached. */
  function plansView() {
    const left = {};
    return plans.map((p) => {
      if (!(p.item in left)) left[p.item] = inv(p.item);
      const assigned = Math.min(p.target, left[p.item]);
      left[p.item] -= assigned;
      return { ...p, itemName: item(p.item).name, assigned, reached: assigned + EPS >= p.target };
    });
  }

  /** The Requirements: what is still to gather and to craft across all plans. */
  function requirements() {
    // 1. Walk default recipes from every plan; note which plans each item serves, and find loops.
    const serves = {}; // itemId -> Set(planId)
    const loop = {}; // itemId -> [path of item ids forming the loop]
    const order = []; // post-order: inputs before the items made from them
    const done = new Set();
    function walk(id, planId, path) {
      (serves[id] = serves[id] || new Set()).add(planId);
      if (path.includes(id)) { loop[id] = path.slice(path.indexOf(id)).concat(id); return; }
      const r = defaultRecipe(id);
      if (r) r.inputs.forEach(([inp]) => walk(inp, planId, path.concat(id)));
      if (!done.has(id)) { done.add(id); order.push(id); }
    }
    plans.forEach((p) => walk(p.item, p.id, []));

    // 2. Sum demand top-down (targets first), netting inventory at every level, rounding to whole crafts.
    const demand = {};
    plans.forEach((p) => { demand[p.item] = (demand[p.item] || 0) + p.target; });
    const gather = [], craft = [];
    order.slice().reverse().forEach((id) => {
      const need = demand[id] || 0;
      if (need <= 0) return; // reached only through a loop that was cut
      const have = inv(id);
      const missing = Math.max(0, need - have);
      const r = loop[id] ? null : defaultRecipe(id);
      const planList = plans.filter((p) => serves[id].has(p.id));
      if (!r) {
        gather.push({ item: item(id), need, have, missing, loop: loop[id] ? loop[id].map((x) => item(x).name) : null, plans: planList });
        return;
      }
      const crafts = Math.ceil(missing / r.output - EPS);
      const inputs = r.inputs.map(([inp, per]) => {
        demand[inp] = (demand[inp] || 0) + per * crafts;
        return { item: item(inp), perCraft: per, total: per * crafts, have: inv(inp) };
      });
      const canCraft = Math.max(0, Math.min(crafts, ...r.inputs.map(([inp, per]) => Math.floor(inv(inp) / per + EPS))));
      craft.push({
        item: item(id), recipe: r, need, have, missing, crafts,
        makes: crafts * r.output, surplus: crafts * r.output - missing,
        inputs, canCraft, ready: crafts > 0 && canCraft > 0, plans: planList,
      });
    });
    // gather was filled top-down; craft too. Present crafts inputs-first.
    craft.reverse();
    gather.reverse();
    return { gather, craft };
  }

  /** Perform n crafts of an item's default recipe: deduct inputs, add output. Never below zero. */
  function craft(id, n) {
    const r = defaultRecipe(id);
    if (!r) return false;
    const possible = Math.min(...r.inputs.map(([inp, per]) => Math.floor(inv(inp) / per + EPS)));
    n = Math.min(n, possible);
    if (n <= 0) return false;
    r.inputs.forEach(([inp, per]) => { inventory[inp] = +(inv(inp) - per * n).toFixed(6); });
    inventory[id] = inv(id) + r.output * n;
    emit();
    return n;
  }

  window.GP = {
    games,
    currentGame: games[0],
    items: () => items.slice(),
    item,
    recipesOf: (id) => recipesOf(id).map((r) => ({ ...r, inputs: r.inputs.map(([i, per]) => ({ item: item(i), perCraft: per })) })),
    isRaw: (id) => recipesOf(id).length === 0,
    usedIn: (id) => recipes.filter((r) => r.inputs.some(([i]) => i === id)).map((r) => item(r.item)),
    inv,
    setInv(id, amount) { inventory[id] = Math.max(0, +amount || 0); emit(); },
    addInv(id, amount) { inventory[id] = Math.max(0, inv(id) + (+amount || 0)); emit(); },
    plans: plansView,
    addPlan(itemId, target, note) { plans.push({ id: 'p' + Date.now(), item: itemId, target: +target, note: note || '' }); emit(); },
    deletePlan(id) { plans = plans.filter((p) => p.id !== id); emit(); },
    setDefault(recipeId) {
      const r = recipes.find((x) => x.id === recipeId);
      recipes.forEach((x) => { if (x.item === r.item) x.isDefault = x.id === recipeId; });
      emit();
    },
    requirements,
    craft,
    fmt,
    initials: (name) => name.split(' ').map((w) => w[0]).join('').slice(0, 2).toUpperCase(),
    on(fn) { listeners.push(fn); },
  };
})();
