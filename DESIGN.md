---
name: Game Planner
description: A crafting planner read in glances next to a running game
colors:
  bg: "#191a1c"
  side: "#232426"
  tile: "#212225"
  tile-sel: "#2a2b2e"
  edge: "#2e2f32"
  edge-2: "#3a3b3f"
  well: "#121314"
  text: "#ffffff"
  soft: "#e5e5e7"
  muted: "#a7a8ac"
  slate: "#404144"
  btn: "#2b2c2e"
  btn-hover: "#37383c"
  lamp: "#fbd960"
  lamp-soft: "#fdeaa4"
  lamp-fill: "#514721"
  lamp-ink: "#221b02"
  steel: "#8799ab"
  go-green: "#55c792"
  go-deep: "#3f9d71"
  go-fill: "#173a2a"
  go-edge: "#2d6a4e"
  go-ink: "#062014"
  signal-red: "#ff8196"
typography:
  headline:
    fontFamily: "Open Sans, Segoe UI, system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 800
    lineHeight: "28px"
  title:
    fontFamily: "Open Sans, Segoe UI, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 700
    lineHeight: 1.4
  body:
    fontFamily: "Open Sans, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 500
    lineHeight: 1.4
    fontFeature: "tnum"
  label:
    fontFamily: "Open Sans, Segoe UI, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 700
rounded:
  sm: "6px"
  md: "7px"
  lg: "10px"
  round: "14px"
spacing:
  sm: "8px"
  md: "10px"
  lg: "20px"
  xl: "40px"
components:
  button-primary:
    backgroundColor: "{colors.lamp}"
    textColor: "{colors.lamp-ink}"
    typography: "{typography.body}"
    rounded: "{rounded.md}"
    padding: "0 12px"
    height: "34px"
  button-primary-hover:
    backgroundColor: "#fbde75"
  button-go:
    backgroundColor: "{colors.go-deep}"
    textColor: "#04170e"
    typography: "{typography.body}"
    rounded: "{rounded.md}"
    padding: "0 12px"
    height: "34px"
  button-plain:
    backgroundColor: "{colors.btn}"
    textColor: "{colors.soft}"
    typography: "{typography.body}"
    rounded: "{rounded.md}"
    padding: "0 12px"
    height: "34px"
  button-plain-hover:
    backgroundColor: "{colors.btn-hover}"
  button-danger:
    backgroundColor: "{colors.btn}"
    textColor: "{colors.signal-red}"
    typography: "{typography.body}"
    rounded: "{rounded.md}"
    padding: "0 12px"
    height: "34px"
  button-danger-hover:
    backgroundColor: "#5a2630"
  button-disabled:
    backgroundColor: "{colors.well}"
    textColor: "#8a8b8f"
  well:
    backgroundColor: "{colors.well}"
    rounded: "{rounded.md}"
    height: "34px"
  pill:
    backgroundColor: "{colors.go-fill}"
    textColor: "{colors.go-green}"
    typography: "{typography.label}"
    rounded: "{rounded.sm}"
    height: "22px"
    padding: "0 8px"
  tile:
    backgroundColor: "{colors.tile}"
    rounded: "{rounded.lg}"
    padding: "6px"
  nav-item:
    rounded: "{rounded.lg}"
    height: "44px"
    padding: "0 11px"
  search:
    backgroundColor: "{colors.well}"
    rounded: "{rounded.lg}"
    height: "40px"
---

# Design System: Game Planner

## Overview

**Creative North Star: "The Night Workshop"**

A graphite workshop at night, lit by one warm lamp. The app is calm precision: near-black surfaces do the work, a single lamp-yellow marks where to look and what to act on, cool steel annotates the counts, and green go-lights say when something is ready to make. Nothing shouts, nothing moves; the player glances, reads a number, and returns to the game.

The room is built from three tonal steps of graphite — wells sink below the floor (#121314 inset into #191a1c), tiles and rows sit a step above it (#212225), the sidebar a step further (#232426) — separated by 1px edges (#2e2f32), never by shadows. Depth is tone; selection is a lamp border with an inset ring, not elevation. Every control is flush: states swap color instantly, with no lift, glow, or transition. Type is one family, Open Sans, in weights 500–800, set with tabular figures so columns of amounts align mid-glance; hierarchy comes from weight as much as size. Density is comfortable-dense inside one fixed desktop window (~1440×900) where panes scroll and the window never does.

The navy/violet/gold scheme of the seelie.me reference was explicitly rejected during direction selection; the demo's legacy variable names (`--violet`, `--gold`) are historical accidents of that re-theme — the hues they hold are lamp yellow and steel blue, and they carry no violet or gold meaning.

**Key Characteristics:**
- Graphite tonal layering (well → bg → tile → side) with 1px edges; zero drop shadows, zero blur
- One warm accent (Workshop Lamp #fbd960) used sparingly for selection and the primary action
- State colors as traffic lights: Go-Light Green for ready/craft/reached, Signal Red for danger; Workshop Steel for annotation, never state
- Flush, instant controls — color swaps only, no motion
- Single-family type (Open Sans 500–800) with tabular figures everywhere numbers line up
- Game imagery honored: pixel-art icons render pixelated, missing images fall back to a glassy initials tile that looks intentional

## Colors

One warm lamp in a cool graphite room; meaning rides on border and label color first, gradients only as atmosphere.

### Primary
- **Workshop Lamp** (#fbd960): The single accent. Focus rings, active-tab underline, selected-tile border, list-active borders, and the primary button. Rare by design.
- **Lamp Soft** (#fdeaa4): Lamp-adjacent text — counts on the active tab, text on lamp-tinted fills.
- **Lamp Fill** (#514721): The lamp's dimmed fill — selected icon-cluster buttons, text selection, list-active background.
- **Lamp Ink** (#221b02): Text on the lamp-colored primary button.

### Secondary
- **Workshop Steel** (#8799ab): The marginalia color — unit labels next to amounts, secondary counters, and the Recipe loop flag (steel on dark ink #111519). It annotates; it never signals state.

### Tertiary
- **Go-Light Green** (#55c792): Ready to craft, craft actions, reached plans, nav count badges. Supported by **Go Deep** (#3f9d71, the Go button's rest fill), **Go Fill** (#173a2a, badge/flag backgrounds), **Go Edge** (#2d6a4e, ready/reached row borders), and **Go Ink** (#062014, text on green).
- **Signal Red** (#ff8196): Danger only — delete affordances, danger-button hover text, short-amount warnings.

### Neutral
- **Room** (#191a1c): App background and inset nav-active fill.
- **Side** (#232426): Sidebar and popovers.
- **Tile** (#212225) / **Tile Selected** (#2a2b2e): Cards, tiles, rows, and their pressed/selected base.
- **Edge** (#2e2f32) / **Edge Two** (#3a3b3f): The two border steps.
- **Well** (#121314): The darkest step — the backing of every amount field.
- **Text** (#ffffff) / **Soft** (#e5e5e7) / **Muted** (#a7a8ac): White for emphasis and values, soft for body, muted for labels and placeholders.
- **Slate** (#404144): Neutral pill background ("Gather", "Not ready", "N to go") with near-white text (#ececed).
- **Btn** (#2b2c2e) / **Btn Hover** (#37383c): Plain control surfaces.

Item picture squares sit on three two-stop **tier tints** (160°): raw materials moss-green (#557f6b → #446958), mid-tier grey (#65666b → #535559), endgame olive (#847a55 → #706644).

### Named Rules
**The One Lamp Rule.** Workshop Lamp marks selection and the single primary action of a surface. If lamp yellow covers more than ~10% of a screen, the screen is wrong.

**The Traffic-Light Rule.** Green means go (ready, craft, reached); red means destroy or danger. Steel and slate annotate but never signal. A color never carries two meanings, and no state gets a second color.

## Typography

**Display Font:** Open Sans (Segoe UI, system-ui fallback)
**Body Font:** Open Sans (same stack — one family for everything)
**Label/Mono Font:** none; numbers use Open Sans tabular figures rather than a mono face

**Character:** A single hardworking humanist sans, weighted rather than styled — hierarchy comes from stepping 500 → 700 → 800, and every figure is tabular so amounts columnize.

### Hierarchy
- **Headline** (800, 20px/28px): Page titles ("Requirements") and the sidebar brand (19px, letter-spacing 0.01em).
- **Title** (700, 15px/1.4): Section titles and their muted notes; nav items run 16px/700.
- **Body** (500–700, 14px/1.4): Base size of the whole app — names, values, buttons, inputs. Emphasis steps to 700.
- **Label** (700, 12–13px): Pills, flags, badges, tab labels; meta text 12.5–13px at 500–600. Never below 12px.

### Named Rules
**The Tabular Rule.** Every numeric glyph renders tabular (`font-feature: "tnum"` set app-wide). Amounts, counts, and targets must align in columns from any glance distance.

## Layout

One fixed desktop window (~1440×900 target) as a two-column app grid: a 224px sidebar and a fluid main column. The sidebar stacks brand (66px), nav items (44px tall, 6px gaps), a decorative rule divider, and a footer note pinned to the bottom; a full-round game chip at the top switches the open game. Main column: page head (title, then a 58px tab row with search or an icon cluster docked right), then a scrolling content pane with a 40px left inset and 20px right — panes scroll, the window never does.

Content rhythm inside the pane: section titles with muted notes, 6-column item-tile grids (10px gap; 8 columns at 12px for wide grids), and full-width rows at 58px tall. Spacing steps are 8 / 10 / 20 / 40. There are no breakpoints and no page-level responsive behavior — this is a windowed desktop app, not a page.

## Elevation & Depth

Flat by doctrine. Depth is tonal layering: wells sink (#121314), the floor is #191a1c, tiles and rows rise one step (#212225), the sidebar and popovers one more (#232426), all separated by 1px edges (#2e2f32). Nothing drops a shadow, nothing blurs, nothing floats. The only `box-shadow` in the system is an inset 1px ring that doubles the selected tile's lamp border — a crispness device, not elevation.

### Named Rules
**The No-Shadow Rule.** No drop shadows, no blur, no floating layers, ever. If a surface must stand apart, step its tone and draw a 1px edge.

## Shapes

Soft rectangles everywhere: controls round at 7px (buttons, wells, inputs), containers at 10px (tiles, rows, nav items, tabs, search), small badges at 6px (pills, flags). The only fully round shapes are the game chip (28px tall, 14px radius) and the nav count badge — pills of status, not blocks of content.

Item picture squares (28–84px) always clip to a radius, render game icons with pixelated smoothing, and fall back to a glassy initials tile (dark fill at 28% alpha, white border at 22%, extra-bold initials). Two-stop gradients are the only permitted gradient form: tier tints at 160° and state washes that fade a color into tile grey (90–180°) — atmosphere that never carries state alone.

### Named Rules
**The Two-Tone Rule.** Any gradient is exactly two close tones of one family, fading to a neutral. Border and label color carry the meaning; the gradient only warms the surface.

## Components

Character: **refined and flush.** Controls sit level with their surface, states change color without moving, and every amount lives in a well so numbers are findable in one scan.

### Buttons
- **Shape:** softly rounded (7px), 34px tall, 12px horizontal padding, bold 14px label, 6px icon gap.
- **Primary:** Workshop Lamp fill, Lamp Ink text; hover lightens to #fbde75. One per form — the main action.
- **Go:** Go Deep fill (#3f9d71), near-black green ink (#04170e); hover brightens to Go Green. Craft actions only ("Craft 1", "Craft all").
- **Plain:** Btn fill, Soft text; hover Btn Hover. Everything else.
- **Danger:** Btn fill, Signal Red text; hover fills #5a2630. Delete and destroy.
- **Disabled:** Well fill, 1px Edge border, #8a8b8f text, not-allowed cursor — visibly parked, not hidden.

### Wells (amount fields)
- **Style:** the signature component. 34px tall, 7px radius, Well fill (#121314), 1px darker border (#1c1d1f) — a socket sunk into the surface holding one centered bold number.
- **Focus:** the caret is lamp-colored; no ring or halo on the well itself.
- **Steppers:** small 26px inset buttons (5px radius, Btn fill) inside gather wells.

### Chips / Pills
- **Style:** 22px tall, 6px radius, 8px padding, bold 12px. Fill/text pairs carry state: Go Fill/Go Green ("Ready", "3 crafts"), Lamp Fill/Lamp Soft, Slate/#ececed ("Gather", "N to go"), Steel/#111519 ("Recipe loop").
- **Game chip:** the one full-round pill (28px, 14px radius) — the open game's color dot and title.

### Cards / Tiles
- **Corner Style:** 10px; 6px internal padding.
- **Background:** Tile; 1px Edge border; hover deepens the border (#45464a).
- **States:** selected = Lamp border + inset lamp ring on Tile Selected base; ready/reached = Go Edge border with a green-to-grey wash; loop = steel-tinged wash.
- **Contents:** picture square, bold white name (ellipsis), and a baseline-aligned key–value row (muted label, bold white value with steel unit).

### Rows (wide list items)
- 58px tall, 10px radius, Tile fill with a subtle 90° two-tone wash and 1px #37383c border: picture square, name + muted sub-line, then amounts and pills rightward. Reached rows swap to a green wash and Go Edge border.

### Inputs / Fields
- **Search:** 280px, 40px tall, 10px radius, Well fill on a #292a2d border; muted icon, white 600 input text; focus-within draws the lamp border.

### Navigation
- **Sidebar items:** 44px tall, 10px radius, muted 16px/700 text; hover lifts to #e0e1e3; active = Room fill, #292a2d border, white text, with a Go Fill count badge.
- **Tabs:** 98×57px stacked label+count, rounded top corners; active gets a subtle two-tone wash and a 2px lamp underline inset 12px.

### Rule divider
A 1px horizontal gradient fading Side → steel → Side with a small rotated steel diamond at center — the workshop's only ornament, used to separate sidebar sections.

## Do's and Don'ts

### Do:
- **Do** put every editable amount in a well — sunk, centered, bold — so numbers are findable in one scan.
- **Do** pair each "still needed" figure with its "owned" counterpart at the same baseline.
- **Do** render game icons pixelated and give missing images the glassy initials tile; both look intentional.
- **Do** keep gradients to the two-tone atmospheric form (tier tints, state washes) with meaning carried by border and label color.
- **Do** change state instantly — a color swap with no transition is the system's speed of response.

### Don't:
- **Don't** add drop shadows, blur, glow, or any floating layer.
- **Don't** use Workshop Lamp beyond selection and the single primary action (The One Lamp Rule).
- **Don't** set body text below 14px or labels below 12px; keep contrast at 4.5:1 or better.
- **Don't** reuse the legacy demo variable names (`--violet`, `--gold`) as semantic guidance — those tokens hold lamp yellow and steel blue, and the seelie navy/violet/gold scheme they came from was rejected.
- **Don't** introduce motion latency (transitions, animated state changes) — a glance must never wait.
