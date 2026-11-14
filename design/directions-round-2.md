# Game Planner — round 2: three readings of the user's reference

Round 1 (six thumbnails, three built, now in `design-demos/round-1/`) was rejected: "I don't find anything I like, here I give you reference https://seelie.me/inventory". All three round-2 directions work inside that reference's language and differ in how the Requirements screen is structured.

## The reference, taken apart

Screenshots: `refs/seelie-inventory.png`, `refs/seelie-planner.png` (both seen). Borrow the rules below; do not copy its logo, name, artwork, icons or wording.

- **Surface**: deep navy, about #0f1a2c for the content area; the sidebar is a slightly lighter, bluer navy (about #16233b) with no visible border, just the tone change. Panels and tiles are a navy one step lighter than the page, with a 1px edge barely lighter still and 8–10px radius. Count fields are *darker* than their tile (inset wells, about #0b1422).
- **Sidebar** (about 224px): a small rounded chip at the top naming the current game; then nav entries as an icon plus a large semibold label (about 17px) in muted blue-grey (#8fa0bd-ish), widely spaced (about 50px pitch). The active entry is white text on a dark rounded pill. Thin warm-gold hairline dividers with a tiny diamond in the middle separate groups.
- **Page head**: a bold white title (about 20px), then a row of tabs that are an icon above a small label; the active tab has a lighter backing and a short violet underline. A small cluster of icon buttons sits at the right end of the tab row in its own rounded container.
- **Item tile** (the signature): a small rounded card holding a ~64px square with the item's picture on a tinted backing (muted violet, blue or green, by tier), a quiet row beneath with a tick and the needed amount, and an inset dark field for the owned amount. Tiles of one family sit side by side. About 13 tiles per row at 1440px.
- **Wide tile** (top of the inventory page): the same picture square on the left, with the needed amount and a wider owned field to its right, in one long rounded card.
- **Row** (planner page): full-width rounded bars, about 56px tall, 4–6px apart: a pair of small toggle buttons at the left, an icon, a bold white 17px title optionally preceded by a violet pill tag, and a right-aligned slate badge. One bar may be tinted (wine) to mark urgency.
- **Colour**: navy plus white and blue-grey text; violet (#7c5cff-ish) for selection and tags; warm gold only for hairlines and tiny accents; a red badge for urgency.
- **Type**: one humanist sans, bold and semibold doing most of the work.

## A · Wide tiles

- **Reading**: the wide tile from the top of the reference's inventory page becomes the unit of the Requirements screen.
- **Skeleton**: sidebar + head with tabs (All, Gather, Craft) + a two- or three-column flow of wide tiles. Each tile carries everything: picture square, amounts, its inputs as small picture chips with have/need, plan pills, and the gather field or Craft buttons. No side panel, nothing hidden behind a click.
- **Boldness spent on**: everything visible at once.
- **Form from content**: the player glances over; a click to reveal detail is a click they will not make mid-game.

## B · Rows

- **Reading**: the planner page's full-width bars. Requirements is a stack of bars.
- **Skeleton**: sidebar + head with tabs + bars grouped under Gather and Craft. A craft bar has the two small buttons at the left (Craft 1, Craft all), picture, bold item name with a violet pill for the number of crafts, inputs as inline picture chips, and a right-aligned badge (Ready / waiting / surplus). The Ready bars are tinted the way the reference tints its urgent bar. A bar expands in place to show the plans it serves and the recipe.
- **Boldness spent on**: one line per thing, in the order you do them.
- **Form from content**: crafts are listed inputs-first, so the stack reads as a to-do order.

## C · Grid and panel

- **Reading**: the reference's inventory grid, as is, becomes Requirements; a panel explains the selected tile.
- **Skeleton**: sidebar + head with tabs + compact item tiles (picture square, missing or crafts beneath, owned field) grouped under Gather and Craft + a fixed right panel (about 360px) for the selected tile: large picture, the recipe as input tiles, the arithmetic, plan list, Craft 1 / Craft all.
- **Boldness spent on**: the grid itself; Requirements looks like the Inventory screen because both are the same items.
- **Form from content**: the player already reads grids of item pictures with counts all day in the game.
