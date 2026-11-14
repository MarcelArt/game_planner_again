# Game Planner

A planner for crafting in games: it records what a game's items are made of and works out what a player still has to gather and craft to reach their goals.

## Language

**Game**:
A game the player plans for. Every other concept belongs to exactly one game. No two games share a title, ignoring case.

**Item**:
Anything in a game that can be owned, crafted, or used as an ingredient.

**Recipe**:
One way to make an item: a fixed set of inputs that yields a fixed amount of that item per craft. An item may have several recipes.
_Avoid_: Formula, blueprint

**Default recipe**:
The one recipe of an item that the calculation uses. The player chooses it; an item with a single recipe uses that one.
_Avoid_: Preferred recipe, active recipe

**Recipe loop**:
A chain of default recipes that leads back to an item it started from. The calculation stops at the repeated item and treats it as something to gather.
_Avoid_: Cycle, circular recipe

**Input**:
One ingredient line of a recipe: an item and the amount of it one craft consumes.
_Avoid_: Recipe detail, ingredient, component

**Output**:
The amount of the recipe's item that one craft yields.
_Avoid_: Yield, result

**Craft**:
One whole execution of a recipe, consuming all its inputs once and yielding its output once. Crafts are never fractional.
_Avoid_: Batch, run

**Raw material**:
An item with no recipe, which can only be gathered.
_Avoid_: Base item, resource

**Inventory**:
The amount of each item the player currently owns in a game.
_Avoid_: Stock, storage

**Plan**:
A goal to own a target amount of one item for one purpose, optionally described by a note. Several plans may name the same item; their targets add up.
_Avoid_: Goal, order, wishlist

**Target**:
The amount of its item a plan wants the player to own.

**Reached**:
The state of a plan whose target is covered by inventory. Inventory of an item is assigned to that item's plans oldest first, and a reached plan stays until the player deletes it.
_Avoid_: Completed, done, fulfilled

**Ready**:
The state of a needed craft whose inputs are all in inventory.

**Requirements**:
What the calculation produces from all of a game's plans combined: the raw materials still to gather and the crafts still to perform, after expanding recipes all the way down and subtracting inventory at every level. Demand for an item is summed across plans before crafts are counted, so surplus is shared.
_Avoid_: Shopping list, bill of materials

**Surplus**:
Items a craft yields beyond what was needed, because crafts are whole.
_Avoid_: Leftover, excess

**Progress**:
A player's inventory and plans in a game, as opposed to the items and recipes that describe the game.

**Game file**:
A single file holding one game: its items, recipes and pictures, and optionally the player's progress.
_Avoid_: Archive, backup, save file

**Export**:
Writing one game to a game file. The player chooses whether their progress goes into it; by default it does not.

**Import**:
Bringing a game file into the planner. It becomes a new game unless a game with the same title exists, in which case the player chooses to replace that game, merge into it, or keep both.

**Replace**:
An import that discards the existing game, progress included, and puts the game file's contents in its place.
_Avoid_: Overwrite, update

**Merge**:
An import that adds to the existing game whatever items, recipes and pictures it lacks, matching items by name. It never removes or changes anything already there, and it ignores progress in the game file.
_Avoid_: Sync, combine
