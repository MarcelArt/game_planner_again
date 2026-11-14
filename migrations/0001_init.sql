-- Ids are UUIDs stored as text. Amounts are REAL. Timestamps are Unix milliseconds.

CREATE TABLE games (
    id         TEXT PRIMARY KEY NOT NULL,
    title      TEXT NOT NULL,
    thumbnail  TEXT,                       -- path relative to the data folder
    created_at INTEGER NOT NULL
);

CREATE TABLE items (
    id      TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    name    TEXT NOT NULL,
    icon    TEXT,                          -- path relative to the data folder
    UNIQUE (game_id, name)
);

CREATE TABLE recipes (
    id         TEXT PRIMARY KEY NOT NULL,
    item_id    TEXT NOT NULL REFERENCES items (id) ON DELETE CASCADE,
    output     REAL NOT NULL CHECK (output > 0),
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);

-- An input of a recipe. An item cannot be deleted while another item's recipe uses it.
CREATE TABLE recipe_details (
    id        TEXT PRIMARY KEY NOT NULL,
    recipe_id TEXT NOT NULL REFERENCES recipes (id) ON DELETE CASCADE,
    item_id   TEXT NOT NULL REFERENCES items (id) ON DELETE RESTRICT,
    input     REAL NOT NULL CHECK (input > 0)
);

CREATE TABLE inventories (
    id      TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL UNIQUE REFERENCES items (id) ON DELETE CASCADE,
    amount  REAL NOT NULL CHECK (amount >= 0)
);

CREATE TABLE plans (
    id         TEXT PRIMARY KEY NOT NULL,
    item_id    TEXT NOT NULL REFERENCES items (id) ON DELETE CASCADE,
    target     REAL NOT NULL CHECK (target > 0),
    note       TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);

CREATE INDEX items_game ON items (game_id);
CREATE INDEX recipes_item ON recipes (item_id);
CREATE INDEX recipe_details_recipe ON recipe_details (recipe_id);
CREATE INDEX recipe_details_item ON recipe_details (item_id);
CREATE INDEX plans_item ON plans (item_id);
