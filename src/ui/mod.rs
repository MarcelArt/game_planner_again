//! The window: one `Planner` view holding the open game's data, with a page per area.
//!
//! Layout and colours follow `design/design-demos/dir-3-graphite.html`.

mod pages;
mod requirements;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::dialog::{DialogAction, DialogClose, DialogFooter};
use gpui_kit::component::input::{Input, InputEvent as FieldEvent, InputState};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::select::{SearchableVec, SelectState};
use gpui_kit::component::{Icon, Sizable as _, Theme, ThemeMode, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use uuid::Uuid;

use crate::calc::{self, PlanProgress, Requirements};
use crate::db::{Db, GameStats, OnClash};
use crate::domain::{Game, GameData, Input as RecipeInput};
use crate::game_file::GameFile;

/// The Graphite palette.
pub mod pal {
    pub const BG: u32 = 0x191a1c;
    pub const SIDE: u32 = 0x232426;
    pub const TILE: u32 = 0x212225;
    pub const TILE_SEL: u32 = 0x2a2b2e;
    pub const EDGE: u32 = 0x2e2f32;
    pub const EDGE_2: u32 = 0x3a3b3f;
    pub const WELL: u32 = 0x121314;
    pub const TEXT: u32 = 0xffffff;
    pub const SOFT: u32 = 0xe5e5e7;
    pub const MUTED: u32 = 0xa7a8ac;
    pub const ACCENT: u32 = 0xfbd960;
    pub const ACCENT_TEXT: u32 = 0xfdeaa4;
    pub const ACCENT_FILL: u32 = 0x514721;
    pub const ACCENT_INK: u32 = 0x221b02;
    pub const STEEL: u32 = 0x8799ab;
    pub const GREEN: u32 = 0x55c792;
    pub const GREEN_DEEP: u32 = 0x3f9d71;
    pub const GREEN_FILL: u32 = 0x173a2a;
    pub const GREEN_EDGE: u32 = 0x2d6a4e;
    pub const GREEN_INK: u32 = 0x062014;
    pub const RED: u32 = 0xff8196;
    pub const SLATE: u32 = 0x404144;
    pub const BTN: u32 = 0x2b2c2e;
    pub const BTN_HOVER: u32 = 0x37383c;
    /// Picture backings: raw material, crafted and used as an input, crafted and used by nothing.
    pub const TINT_RAW: (u32, u32) = (0x557f6b, 0x446958);
    pub const TINT_MID: (u32, u32) = (0x65666b, 0x535559);
    pub const TINT_END: (u32, u32) = (0x847a55, 0x706644);
}

/// Applies the Graphite palette to the component library's theme.
pub fn apply_theme(cx: &mut App) {
    Theme::change(ThemeMode::Dark, None, cx);
    Theme::update(cx, |t| {
        let c = |v: u32| -> Hsla { rgb(v).into() };
        t.background = c(pal::BG);
        t.foreground = c(pal::SOFT);
        t.border = c(pal::EDGE);
        t.input = c(pal::EDGE_2);
        t.muted = c(pal::TILE);
        t.muted_foreground = c(pal::MUTED);
        t.popover = c(pal::SIDE);
        t.popover_foreground = c(pal::SOFT);
        t.accent = c(pal::BTN_HOVER);
        t.accent_foreground = c(pal::TEXT);
        t.secondary = c(pal::BTN);
        t.secondary_hover = c(pal::BTN_HOVER);
        t.secondary_foreground = c(pal::SOFT);
        t.primary = c(pal::ACCENT);
        t.primary_hover = c(0xfbde75);
        t.primary_active = c(0xe9c84f);
        t.primary_foreground = c(pal::ACCENT_INK);
        t.ring = c(pal::ACCENT);
        t.caret = c(pal::ACCENT);
        t.selection = c(pal::ACCENT_FILL);
        t.colors.list = c(pal::SIDE);
        t.list_hover = c(pal::BTN_HOVER);
        t.list_active = c(pal::ACCENT_FILL);
        t.list_active_border = c(pal::ACCENT);
        t.font_family = "Open Sans".into();
        t.font_size = px(14.);
        t.radius = px(7.);
        t.radius_lg = px(10.);
    });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Library,
    Requirements,
    Plans,
    Items,
    Inventory,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReqTab {
    All,
    Gather,
    Craft,
    Ready,
}

#[derive(Clone, Copy)]
enum ImageTarget {
    GameThumbnail(Uuid),
    ItemIcon(Uuid),
}

type ItemSelect = SelectState<SearchableVec<SharedString>>;

pub struct Planner {
    db: Rc<Db>,
    games: Vec<(Game, GameStats)>,
    /// The open game.
    game: Option<Uuid>,
    data: GameData,
    req: Requirements,
    progress: Vec<PlanProgress>,
    page: Page,
    req_tab: ReqTab,
    /// Selected item on the Requirements page.
    sel_req: Option<Uuid>,
    /// Selected item on the Items page.
    sel_item: Option<Uuid>,

    inventory_fields: HashMap<Uuid, Entity<InputState>>,
    gather_fields: HashMap<Uuid, Entity<InputState>>,
    search: Entity<InputState>,
    new_game: Entity<InputState>,
    game_title: Entity<InputState>,
    new_item: Entity<InputState>,
    item_name: Entity<InputState>,
    plan_item: Entity<ItemSelect>,
    plan_target: Entity<InputState>,
    plan_note: Entity<InputState>,
    recipe_output: Entity<InputState>,
    recipe_input_item: Entity<ItemSelect>,
    recipe_input_amount: Entity<InputState>,
    /// Inputs of the recipe being composed on the Items page.
    recipe_draft: Vec<RecipeInput>,
    /// The choice in the export dialog: whether inventory and plans go into the game file.
    export_progress: bool,
    subs: Vec<Subscription>,
}

impl Planner {
    pub fn new(db: Rc<Db>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let field = |placeholder: &'static str, window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| InputState::new(window, cx).placeholder(placeholder))
        };
        let select = |window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| SelectState::new(SearchableVec::new(Vec::<SharedString>::new()), None, window, cx).searchable(true))
        };
        let search = field("Search items", window, cx);
        let new_game = field("Title of a new game", window, cx);
        let game_title = field("Game title", window, cx);
        let new_item = field("Name of a new item", window, cx);
        let item_name = field("Item name", window, cx);
        let plan_target = field("Amount", window, cx);
        let plan_note = field("What is it for? Optional", window, cx);
        let recipe_output = field("Makes per craft", window, cx);
        let recipe_input_amount = field("Amount per craft", window, cx);
        let plan_item = select(window, cx);
        let recipe_input_item = select(window, cx);

        let subs = vec![
            cx.subscribe_in(&search, window, |_, _, ev: &FieldEvent, _, cx| {
                if matches!(ev, FieldEvent::Change) {
                    cx.notify();
                }
            }),
            cx.subscribe_in(&new_game, window, |this, _, ev: &FieldEvent, window, cx| {
                if matches!(ev, FieldEvent::PressEnter { .. }) {
                    this.add_game(window, cx);
                }
            }),
            cx.subscribe_in(&new_item, window, |this, _, ev: &FieldEvent, window, cx| {
                if matches!(ev, FieldEvent::PressEnter { .. }) {
                    this.add_item(window, cx);
                }
            }),
            cx.subscribe_in(&item_name, window, |this, _, ev: &FieldEvent, window, cx| {
                if matches!(ev, FieldEvent::PressEnter { .. }) {
                    this.rename_item(window, cx);
                }
            }),
            cx.subscribe_in(&plan_note, window, |this, _, ev: &FieldEvent, window, cx| {
                if matches!(ev, FieldEvent::PressEnter { .. }) {
                    this.add_plan(window, cx);
                }
            }),
            cx.subscribe_in(&plan_target, window, |this, _, ev: &FieldEvent, window, cx| {
                if matches!(ev, FieldEvent::PressEnter { .. }) {
                    this.add_plan(window, cx);
                }
            }),
        ];

        let mut this = Self {
            db,
            games: Vec::new(),
            game: None,
            data: GameData::default(),
            req: Requirements::default(),
            progress: Vec::new(),
            page: Page::Library,
            req_tab: ReqTab::All,
            sel_req: None,
            sel_item: None,
            inventory_fields: HashMap::new(),
            gather_fields: HashMap::new(),
            search,
            new_game,
            game_title,
            new_item,
            item_name,
            plan_item,
            plan_target,
            plan_note,
            recipe_output,
            recipe_input_item,
            recipe_input_amount,
            recipe_draft: Vec::new(),
            export_progress: false,
            subs,
        };
        this.reload(window, cx);
        // Open straight into the only game, if there is exactly one.
        if this.games.len() == 1 {
            let id = this.games[0].0.id;
            this.open_game(id, window, cx);
        }
        // Development aid: `GAME_PLANNER_START=requirements|plans|items|inventory` opens the
        // first game on that page.
        if let Ok(start) = std::env::var("GAME_PLANNER_START")
            && let Some(id) = this.games.first().map(|(g, _)| g.id)
        {
            this.open_game(id, window, cx);
            this.page = match start.as_str() {
                "plans" => Page::Plans,
                "items" => Page::Items,
                "inventory" => Page::Inventory,
                _ => Page::Requirements,
            };
        }
        this
    }

    // ---- loading ----

    /// Re-reads everything from the database and recalculates.
    fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let games = self.db.games().unwrap_or_else(|e| {
            self.notify_error(&e, window, cx);
            Vec::new()
        });
        self.games = games.into_iter().map(|g| (self.db.game_stats(g.id).unwrap_or_default(), g)).map(|(s, g)| (g, s)).collect();
        if self.game.is_some_and(|id| !self.games.iter().any(|(g, _)| g.id == id)) {
            self.game = None;
        }
        self.data = match self.game {
            Some(id) => self.db.load_game(id).unwrap_or_else(|e| {
                self.notify_error(&e, window, cx);
                GameData::default()
            }),
            None => GameData::default(),
        };
        if self.game.is_none() {
            self.page = Page::Library;
        }
        self.req = calc::requirements(&self.data);
        self.progress = calc::plan_progress(&self.data);

        let in_req = |id: Uuid| self.req.gather.iter().any(|r| r.item_id == id) || self.req.craft.iter().any(|r| r.item_id == id);
        if !self.sel_req.is_some_and(in_req) {
            self.sel_req = self.req.craft.iter().find(|r| r.ready()).map(|r| r.item_id).or(self.req.craft.first().map(|r| r.item_id)).or(self.req.gather.first().map(|r| r.item_id));
        }
        if !self.sel_item.is_some_and(|id| self.data.item(id).is_some()) {
            self.sel_item = self.data.items.first().map(|i| i.id);
            self.sync_item_panel(window, cx);
        }

        let names: Vec<SharedString> = self.data.items.iter().map(|i| i.name.clone().into()).collect();
        for select in [&self.plan_item, &self.recipe_input_item] {
            let names = names.clone();
            select.update(cx, |s, cx| s.set_items(SearchableVec::new(names), window, cx));
        }

        self.inventory_fields.retain(|id, _| self.data.item(*id).is_some());
        self.gather_fields.retain(|id, _| self.data.item(*id).is_some());
        for (id, field) in &self.inventory_fields {
            let text = crate::domain::fmt_amount(self.data.owned(*id));
            field.update(cx, |s, cx| {
                if s.value().as_ref() != text {
                    s.set_value(text, window, cx);
                }
            });
        }
        cx.notify();
    }

    /// Makes sure every item has its inventory field, and every gather row its field.
    fn ensure_fields(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let ids: Vec<Uuid> = self.data.items.iter().map(|i| i.id).collect();
        for id in ids {
            if !self.inventory_fields.contains_key(&id) {
                let text = crate::domain::fmt_amount(self.data.owned(id));
                let field = cx.new(|cx| InputState::new(window, cx).default_value(text));
                self.subs.push(cx.subscribe_in(&field, window, move |this, _, ev: &FieldEvent, window, cx| {
                    if matches!(ev, FieldEvent::PressEnter { .. } | FieldEvent::Blur) {
                        this.commit_inventory(id, window, cx);
                    }
                }));
                self.inventory_fields.insert(id, field);
            }
        }
        let gather: Vec<Uuid> = self.req.gather.iter().map(|r| r.item_id).collect();
        for id in gather {
            if !self.gather_fields.contains_key(&id) {
                let field = cx.new(|cx| InputState::new(window, cx).placeholder("0"));
                self.subs.push(cx.subscribe_in(&field, window, move |this, _, ev: &FieldEvent, window, cx| {
                    if matches!(ev, FieldEvent::PressEnter { .. }) {
                        this.add_gathered(id, window, cx);
                    }
                }));
                self.gather_fields.insert(id, field);
            }
        }
    }

    fn notify_error(&self, err: &anyhow::Error, window: &mut Window, cx: &mut Context<Self>) {
        window.push_notification(Notification::error(err.to_string()), cx);
    }

    /// Reports a failed action, then reloads either way.
    fn finish<T>(&mut self, result: anyhow::Result<T>, window: &mut Window, cx: &mut Context<Self>) -> Option<T> {
        let out = match result {
            Ok(v) => Some(v),
            Err(e) => {
                self.notify_error(&e, window, cx);
                None
            }
        };
        self.reload(window, cx);
        out
    }

    fn text(field: &Entity<InputState>, cx: &App) -> String {
        field.read(cx).value().trim().to_string()
    }

    fn amount(field: &Entity<InputState>, cx: &App) -> Option<f64> {
        Self::text(field, cx).replace(',', ".").parse::<f64>().ok().filter(|v| v.is_finite())
    }

    fn clear(field: &Entity<InputState>, window: &mut Window, cx: &mut Context<Self>) {
        field.update(cx, |s, cx| s.set_value("", window, cx));
    }

    fn selected_item(&self, select: &Entity<ItemSelect>, cx: &App) -> Option<Uuid> {
        let name = select.read(cx).selected_value().cloned()?;
        self.data.items.iter().find(|i| i.name == name.as_ref()).map(|i| i.id)
    }

    // ---- actions ----

    fn go(&mut self, page: Page, cx: &mut Context<Self>) {
        if self.game.is_some() || page == Page::Library {
            self.page = page;
            cx.notify();
        }
    }

    fn open_game(&mut self, id: Uuid, window: &mut Window, cx: &mut Context<Self>) {
        self.game = Some(id);
        self.sel_req = None;
        self.sel_item = None;
        self.recipe_draft.clear();
        self.page = Page::Requirements;
        self.reload(window, cx);
        if self.data.items.is_empty() {
            self.page = Page::Items;
        }
    }

    fn add_game(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let title = Self::text(&self.new_game, cx);
        let result = self.db.add_game(&title);
        if let Some(id) = self.finish(result, window, cx) {
            Self::clear(&self.new_game.clone(), window, cx);
            self.open_game(id, window, cx);
        }
    }

    /// Asks for a new title in a dialog.
    fn rename_game_dialog(&mut self, id: Uuid, window: &mut Window, cx: &mut Context<Self>) {
        let Some((game, _)) = self.games.iter().find(|(g, _)| g.id == id) else { return };
        let title = game.title.clone();
        let field = self.game_title.clone();
        field.update(cx, |s, cx| s.set_value(title, window, cx));
        let weak = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, _| {
            let (weak, field, shown) = (weak.clone(), field.clone(), field.clone());
            dialog
                .title("Rename game")
                .child(Input::new(&shown))
                .footer(
                    DialogFooter::new()
                        .child(DialogClose::new().trigger(|button| button.label("Cancel")))
                        .child(DialogAction::new().child(Button::new("rename-game-ok").primary().label("Rename"))),
                )
                .on_ok(move |_, window, cx| {
                    let title = field.read(cx).value().trim().to_string();
                    if title.is_empty() {
                        return false;
                    }
                    let _ = weak.update(cx, |this, cx| {
                        let result = this.db.rename_game(id, &title);
                        this.finish(result, window, cx);
                    });
                    true
                })
        });
    }

    fn confirm_delete_game(&mut self, id: Uuid, window: &mut Window, cx: &mut Context<Self>) {
        let Some((game, _)) = self.games.iter().find(|(g, _)| g.id == id) else { return };
        let title: SharedString = format!("Delete {}?", game.title).into();
        let weak = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, _| {
            let weak = weak.clone();
            alert
                .title(title.clone())
                .description("All its items, recipes, inventory, plans and images are deleted. This cannot be undone.")
                .confirm()
                .ok_text("Delete game")
                .on_ok(move |_, window, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        let result = this.db.delete_game(id);
                        this.finish(result, window, cx);
                    });
                    true
                })
        });
    }

    // ---- game files ----

    /// Asks whether progress goes into the game file, then where to save it.
    fn export_dialog(&mut self, id: Uuid, window: &mut Window, cx: &mut Context<Self>) {
        let Some((game, _)) = self.games.iter().find(|(g, _)| g.id == id) else { return };
        let title: SharedString = format!("Export {}", game.title).into();
        self.export_progress = false;
        let weak = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, cx| {
            let include = weak.upgrade().is_some_and(|planner| planner.read(cx).export_progress);
            let (toggle, confirm) = (weak.clone(), weak.clone());
            dialog
                .title(title.clone())
                .centered()
                .child(
                    v_flex()
                        .gap(px(12.))
                        .child(div().text_color(rgb(pal::MUTED)).child("The game file holds the game's items, recipes and pictures."))
                        .child(Checkbox::new("export-progress").label("Include my progress (inventory and plans)").checked(include).on_change(move |on, window, cx| {
                            let on = *on;
                            let _ = toggle.update(cx, |this, _| this.export_progress = on);
                            window.refresh();
                        })),
                )
                .footer(
                    DialogFooter::new()
                        .child(DialogClose::new().trigger(|button| button.label("Cancel")))
                        .child(DialogAction::new().child(Button::new("export-ok").primary().label("Export"))),
                )
                .on_ok(move |_, window, cx| {
                    let _ = confirm.update(cx, |this, cx| this.pick_export_path(id, window, cx));
                    true
                })
        });
    }

    fn pick_export_path(&mut self, id: Uuid, window: &mut Window, cx: &mut Context<Self>) {
        let Some((game, _)) = self.games.iter().find(|(g, _)| g.id == id) else { return };
        let name = format!("{}.zip", file_stem(&game.title));
        let folder = dirs::download_dir().or_else(dirs::home_dir).unwrap_or_default();
        let include_progress = self.export_progress;
        let path = cx.prompt_for_new_path(&folder, Some(&name));
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(mut path))) = path.await {
                if path.extension().is_none() {
                    path.set_extension("zip");
                }
                let _ = this.update_in(cx, |this, window, cx| this.export_to(id, include_progress, &path, window, cx));
            }
        })
        .detach();
    }

    fn export_to(&mut self, id: Uuid, include_progress: bool, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        let result = self.db.export_game(id, include_progress, path);
        if self.finish(result, window, cx).is_some() {
            window.push_notification(Notification::success(format!("Exported to {}", path.display())), cx);
        }
    }

    fn pick_game_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: false, prompt: Some("Import".into()) });
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = paths.await
                && let Some(path) = paths.into_iter().next()
            {
                let _ = this.update_in(cx, |this, window, cx| this.import_path(&path, window, cx));
            }
        })
        .detach();
    }

    /// Imports a game file, asking what to do when a game with its title already exists.
    fn import_path(&mut self, path: &Path, window: &mut Window, cx: &mut Context<Self>) {
        let clash = GameFile::read(path).and_then(|file| Ok((self.db.game_titled(&file.title)?.is_some(), file)));
        match clash {
            Ok((false, file)) => self.import(&file, None, window, cx),
            Ok((true, file)) => self.import_clash_dialog(Rc::new(file), window, cx),
            Err(e) => self.notify_error(&e, window, cx),
        }
    }

    fn import(&mut self, file: &GameFile, on_clash: Option<OnClash>, window: &mut Window, cx: &mut Context<Self>) {
        let result = self.db.import_game(file, on_clash);
        if let Some(done) = self.finish(result, window, cx) {
            let count = |n: usize, one: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {one}s") };
            let (items, recipes) = (count(done.items, "item"), count(done.recipes, "recipe"));
            let message = if done.merged { format!("Merged into {}: {items} and {recipes} added", done.title) } else { format!("Imported {}: {items}, {recipes}", done.title) };
            window.push_notification(Notification::success(message), cx);
        }
    }

    fn import_clash_dialog(&mut self, file: Rc<GameFile>, window: &mut Window, cx: &mut Context<Self>) {
        let title: SharedString = format!("A game called {} already exists", file.title).into();
        let weak = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, _| {
            let choice = |id: &'static str, label: &'static str, kind: Kind, note: &'static str, on_clash: OnClash| {
                let (weak, file) = (weak.clone(), file.clone());
                h_flex().gap(px(12.)).child(
                    button(id, label, kind, true)
                        .w(px(104.))
                        .flex_none()
                        .on_click(move |_, window, cx| {
                            window.close_dialog(cx);
                            let _ = weak.update(cx, |this, cx| match on_clash {
                                OnClash::Replace => this.confirm_replace(file.clone(), window, cx),
                                _ => this.import(&file, Some(on_clash), window, cx),
                            });
                        })
                        .test_support(),
                )
                .child(div().flex_1().min_w_0().text_size(px(13.)).text_color(rgb(pal::MUTED)).child(note))
            };
            dialog
                .title(title.clone())
                .w(px(580.))
                .centered()
                .child(
                    v_flex()
                        .gap(px(12.))
                        .child(choice("import-replace", "Replace", Kind::Danger, "Deletes the existing game, including your inventory and plans, and puts this file in its place.", OnClash::Replace))
                        .child(choice("import-merge", "Merge", Kind::Plain, "Adds the items, recipes and pictures the existing game lacks. Nothing is removed, and the file's inventory and plans are ignored.", OnClash::Merge))
                        .child(choice("import-keep-both", "Keep both", Kind::Plain, "Imports the file as a separate game with a numbered title.", OnClash::KeepBoth)),
                )
                .footer(DialogFooter::new().child(DialogClose::new().trigger(|button| button.label("Cancel"))))
        });
    }

    fn confirm_replace(&mut self, file: Rc<GameFile>, window: &mut Window, cx: &mut Context<Self>) {
        let title: SharedString = format!("Replace {}?", file.title).into();
        let description = if file.progress.is_some() {
            "The existing game is deleted, and your inventory and plans are replaced by the ones in the file. This cannot be undone."
        } else {
            "The existing game is deleted and your inventory and plans are lost: this file carries none. This cannot be undone."
        };
        let weak = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, _| {
            let (weak, file) = (weak.clone(), file.clone());
            alert.title(title.clone()).description(description).confirm().ok_text("Replace game").on_ok(move |_, window, cx| {
                let _ = weak.update(cx, |this, cx| this.import(&file, Some(OnClash::Replace), window, cx));
                true
            })
        });
    }

    fn pick_image(&mut self, target: ImageTarget, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: false, prompt: Some("Use image".into()) });
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = paths.await
                && let Some(path) = paths.into_iter().next()
            {
                let _ = this.update_in(cx, |this, window, cx| {
                    let result = match target {
                        ImageTarget::GameThumbnail(id) => this.db.set_game_thumbnail(id, &path),
                        ImageTarget::ItemIcon(id) => match this.game {
                            Some(game) => this.db.set_item_icon(game, id, &path),
                            None => Ok(()),
                        },
                    };
                    this.finish(result, window, cx);
                });
            }
        })
        .detach();
    }

    fn add_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(game) = self.game else { return };
        let name = Self::text(&self.new_item, cx);
        let result = self.db.add_item(game, &name);
        if let Some(id) = self.finish(result, window, cx) {
            Self::clear(&self.new_item.clone(), window, cx);
            self.select_item(id, window, cx);
        }
    }

    fn select_item(&mut self, id: Uuid, window: &mut Window, cx: &mut Context<Self>) {
        self.sel_item = Some(id);
        self.recipe_draft.clear();
        self.sync_item_panel(window, cx);
        cx.notify();
    }

    fn sync_item_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.sel_item.and_then(|id| self.data.item(id)).map(|i| i.name.clone()).unwrap_or_default();
        self.item_name.update(cx, |s, cx| s.set_value(name, window, cx));
    }

    fn rename_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.sel_item else { return };
        let name = Self::text(&self.item_name, cx);
        if self.data.item(id).is_some_and(|i| i.name == name) {
            return;
        }
        let result = self.db.rename_item(id, &name);
        self.finish(result, window, cx);
        self.sync_item_panel(window, cx);
    }

    fn confirm_delete_item(&mut self, id: Uuid, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.data.item(id) else { return };
        let title: SharedString = format!("Delete {}?", item.name).into();
        let weak = cx.entity().downgrade();
        window.open_alert_dialog(cx, move |alert, _, _| {
            let weak = weak.clone();
            alert
                .title(title.clone())
                .description("Its recipes, inventory and plans are deleted with it.")
                .confirm()
                .ok_text("Delete item")
                .on_ok(move |_, window, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        let result = this.db.delete_item(id);
                        this.finish(result, window, cx);
                    });
                    true
                })
        });
    }

    fn add_draft_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item_id) = self.selected_item(&self.recipe_input_item, cx) else {
            window.push_notification(Notification::warning("Choose the input's item first."), cx);
            return;
        };
        let Some(amount) = Self::amount(&self.recipe_input_amount, cx).filter(|a| *a > 0.0) else {
            window.push_notification(Notification::warning("Enter the amount one craft consumes."), cx);
            return;
        };
        if Some(item_id) == self.sel_item {
            window.push_notification(Notification::warning("A recipe cannot use the item it makes as an input."), cx);
            return;
        }
        self.recipe_draft.retain(|i| i.item_id != item_id);
        self.recipe_draft.push(RecipeInput { item_id, amount });
        Self::clear(&self.recipe_input_amount.clone(), window, cx);
        cx.notify();
    }

    fn save_recipe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.sel_item else { return };
        let Some(output) = Self::amount(&self.recipe_output, cx) else {
            window.push_notification(Notification::warning("Enter how many one craft makes."), cx);
            return;
        };
        let result = self.db.add_recipe(item, output, &self.recipe_draft);
        if self.finish(result, window, cx).is_some() {
            self.recipe_draft.clear();
            Self::clear(&self.recipe_output.clone(), window, cx);
        }
    }

    fn add_plan(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = self.selected_item(&self.plan_item, cx) else {
            window.push_notification(Notification::warning("Choose the item to plan for."), cx);
            return;
        };
        let Some(target) = Self::amount(&self.plan_target, cx) else {
            window.push_notification(Notification::warning("Enter the target amount."), cx);
            return;
        };
        let note = Self::text(&self.plan_note, cx);
        let result = self.db.add_plan(item, target, &note);
        if self.finish(result, window, cx).is_some() {
            Self::clear(&self.plan_target.clone(), window, cx);
            Self::clear(&self.plan_note.clone(), window, cx);
        }
    }

    fn commit_inventory(&mut self, id: Uuid, window: &mut Window, cx: &mut Context<Self>) {
        let Some(field) = self.inventory_fields.get(&id).cloned() else { return };
        match Self::amount(&field, cx) {
            Some(amount) if (amount.max(0.0) - self.data.owned(id)).abs() > 1e-9 => {
                let result = self.db.set_inventory(id, amount);
                self.finish(result, window, cx);
            }
            Some(_) => {}
            // Not a number: put the stored amount back.
            None => self.reload(window, cx),
        }
    }

    fn add_gathered(&mut self, id: Uuid, window: &mut Window, cx: &mut Context<Self>) {
        let Some(field) = self.gather_fields.get(&id).cloned() else { return };
        let Some(amount) = Self::amount(&field, cx).filter(|a| *a != 0.0) else { return };
        let result = self.db.set_inventory(id, self.data.owned(id) + amount);
        self.finish(result, window, cx);
        Self::clear(&field, window, cx);
    }

    /// Performs up to `crafts` crafts of the item's default recipe.
    fn craft(&mut self, item_id: Uuid, crafts: u64, window: &mut Window, cx: &mut Context<Self>) {
        let Some(recipe) = self.data.default_recipe(item_id).cloned() else { return };
        let result = self.db.craft(&recipe, crafts, &self.data);
        self.finish(result, window, cx);
    }

    // ---- shared pieces ----

    fn tint(&self, item_id: Uuid) -> (u32, u32) {
        if self.data.is_raw(item_id) {
            pal::TINT_RAW
        } else if self.data.used_in(item_id).is_empty() {
            pal::TINT_END
        } else {
            pal::TINT_MID
        }
    }

    /// An item's picture on its tinted backing; initials when it has no icon.
    fn pic(&self, item_id: Uuid, w: f32, h: f32, image: f32, radius: f32) -> Div {
        let (a, b) = self.tint(item_id);
        let item = self.data.item(item_id);
        let name = item.map(|i| i.name.clone()).unwrap_or_default();
        let fallback = move || initials_tile(&name, image).into_any_element();
        let inner = match item.and_then(|i| i.icon.as_ref()) {
            Some(relative) => img(self.db.image_path(relative)).size(px(image)).object_fit(ObjectFit::Contain).with_fallback(fallback).into_any_element(),
            None => fallback(),
        };
        div()
            .w(px(w))
            .h(px(h))
            .flex_none()
            .rounded(px(radius))
            .overflow_hidden()
            .bg(linear_gradient(160., linear_color_stop(rgb(a), 0.), linear_color_stop(rgb(b), 1.)))
            .flex()
            .items_center()
            .justify_center()
            .child(inner)
    }

    fn nav_item(&self, page: Page, label: &'static str, icon: Lucide, badge: Option<usize>, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let on = self.page == page;
        let enabled = self.game.is_some() || page == Page::Library;
        h_flex()
            .id(label)
            .h(px(44.))
            .px(px(11.))
            .gap(px(11.))
            .rounded(px(10.))
            .border_1()
            .border_color(if on { rgb(0x292a2d) } else { rgba(0x00000000) })
            .when(on, |el| el.bg(rgb(pal::BG)))
            .text_color(rgb(if on { pal::TEXT } else { pal::MUTED }))
            .text_size(px(16.))
            .font_weight(FontWeight::BOLD)
            .when(enabled, |el| el.cursor_pointer().hover(|s| s.text_color(rgb(pal::TEXT))))
            .when(!enabled, |el| el.opacity(0.45))
            .on_click(cx.listener(move |this, _, _, cx| this.go(page, cx)))
            .child(Icon::new(icon).with_size(px(19.)))
            .child(label)
            .when_some(badge.filter(|n| *n > 0), |el, n| {
                el.child(div().flex_1()).child(
                    h_flex()
                        .h(px(22.))
                        .min_w(px(24.))
                        .px(px(7.))
                        .justify_center()
                        .rounded(px(11.))
                        .bg(rgb(pal::GREEN_FILL))
                        .text_color(rgb(pal::GREEN))
                        .text_size(px(12.))
                        .child(n.to_string()),
                )
            })
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let game = self.game.and_then(|id| self.games.iter().find(|(g, _)| g.id == id)).map(|(g, _)| g.title.clone());
        let ready = self.req.craft.iter().filter(|r| r.ready()).count();
        let divider = || h_flex().h(px(26.)).px(px(12.)).child(div().flex_1().h(px(1.)).bg(rgb(pal::EDGE_2)));
        v_flex()
            .w(px(224.))
            .flex_none()
            .h_full()
            .bg(rgb(pal::SIDE))
            .px(px(14.))
            .pt(px(22.))
            .pb(px(16.))
            .gap(px(4.))
            .child(
                h_flex().justify_center().child(
                    h_flex()
                        .id("game-chip")
                        .h(px(28.))
                        .px(px(14.))
                        .max_w(px(190.))
                        .rounded(px(14.))
                        .bg(rgb(pal::BG))
                        .text_color(rgb(pal::SOFT))
                        .font_weight(FontWeight::BOLD)
                        .cursor_pointer()
                        .hover(|s| s.text_color(rgb(pal::TEXT)))
                        .on_click(cx.listener(|this, _, _, cx| this.go(Page::Library, cx)))
                        .child(div().truncate().child(game.unwrap_or_else(|| "No game open".into()))),
                ),
            )
            .child(
                h_flex()
                    .h(px(62.))
                    .justify_center()
                    .text_color(rgb(pal::TEXT))
                    .text_size(px(19.))
                    .font_weight(FontWeight::EXTRA_BOLD)
                    .child("Game Planner"),
            )
            .child(self.nav_item(Page::Requirements, "Requirements", Lucide::ListChecks, Some(ready), cx))
            .child(self.nav_item(Page::Plans, "Plans", Lucide::Target, None, cx))
            .child(self.nav_item(Page::Items, "Items", Lucide::Box, None, cx))
            .child(self.nav_item(Page::Inventory, "Inventory", Lucide::LayoutGrid, None, cx))
            .child(divider())
            .child(self.nav_item(Page::Library, "Game library", Lucide::Library, None, cx))
            .child(div().flex_1())
            .when(self.game.is_some(), |el| {
                el.child(
                    div()
                        .text_center()
                        .text_size(px(12.))
                        .text_color(rgb(pal::MUTED))
                        .child(format!("{} items, {} plans", self.data.items.len(), self.data.plans.len())),
                )
            })
    }
}

impl Render for Planner {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_fields(window, cx);
        let page = match self.page {
            Page::Library => self.library_page(cx).into_any_element(),
            Page::Requirements => self.requirements_page(cx).into_any_element(),
            Page::Plans => self.plans_page(cx).into_any_element(),
            Page::Items => self.items_page(cx).into_any_element(),
            Page::Inventory => self.inventory_page(cx).into_any_element(),
        };
        h_flex()
            .size_full()
            .items_start()
            .bg(rgb(pal::BG))
            .text_color(rgb(pal::SOFT))
            .text_size(px(14.))
            .child(self.sidebar(cx))
            .child(div().flex_1().min_w_0().h_full().child(page))
    }
}

// ---- free helpers shared by the pages ----

pub fn initials(name: &str) -> String {
    name.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_uppercase()
}

fn initials_tile(name: &str, size: f32) -> Div {
    div()
        .size(px(size * 0.86))
        .rounded(px(6.))
        .bg(rgba(0x12131447))
        .border_1()
        .border_color(rgba(0xffffff38))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px((size * 0.36).max(11.)))
        .font_weight(FontWeight::EXTRA_BOLD)
        .text_color(rgb(pal::TEXT))
        .child(initials(name))
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    /// Yellow: the main action of a form.
    Primary,
    /// Green: craft.
    Go,
    Plain,
    Danger,
}

/// A button. Give it `.on_click(..)`; a disabled one ignores clicks.
pub fn button(id: impl Into<ElementId>, label: impl Into<SharedString>, kind: Kind, enabled: bool) -> Stateful<Div> {
    let (bg, hover, fg) = match kind {
        Kind::Primary => (pal::ACCENT, 0xfbde75, pal::ACCENT_INK),
        Kind::Go => (pal::GREEN_DEEP, pal::GREEN, 0x04170e),
        Kind::Plain => (pal::BTN, pal::BTN_HOVER, pal::SOFT),
        Kind::Danger => (pal::BTN, 0x5a2630, pal::RED),
    };
    h_flex()
        .id(id)
        .h(px(34.))
        .px(px(12.))
        .gap(px(6.))
        .justify_center()
        .rounded(px(7.))
        .font_weight(FontWeight::BOLD)
        .text_size(px(14.))
        .whitespace_nowrap()
        .when(enabled, |el| el.bg(rgb(bg)).text_color(rgb(fg)).cursor_pointer().hover(move |s| s.bg(rgb(hover))))
        .when(!enabled, |el| el.bg(rgb(pal::WELL)).border_1().border_color(rgb(pal::EDGE)).text_color(rgb(0x8a8b8f)).cursor_not_allowed())
        .child(label.into())
}

pub fn pill(label: impl Into<SharedString>, bg: u32, fg: u32) -> Div {
    h_flex()
        .h(px(22.))
        .px(px(8.))
        .flex_none()
        .rounded(px(6.))
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .text_size(px(12.))
        .font_weight(FontWeight::BOLD)
        .whitespace_nowrap()
        .child(label.into())
}

/// A dark inset box, the backing of every amount field.
pub fn well() -> Div {
    h_flex().h(px(34.)).rounded(px(7.)).bg(rgb(pal::WELL)).border_1().border_color(rgb(0x1c1d1f))
}

pub fn page_title(title: &'static str, note: impl Into<SharedString>) -> Div {
    h_flex()
        .gap(px(12.))
        .items_baseline()
        .child(div().text_size(px(20.)).font_weight(FontWeight::EXTRA_BOLD).text_color(rgb(pal::TEXT)).child(title))
        .child(div().text_color(rgb(pal::MUTED)).child(note.into()))
}

pub fn section_title(title: &'static str, note: impl Into<SharedString>) -> Div {
    h_flex()
        .gap(px(10.))
        .items_baseline()
        .child(div().text_size(px(15.)).font_weight(FontWeight::BOLD).text_color(rgb(pal::TEXT)).child(title))
        .child(div().text_size(px(13.)).text_color(rgb(pal::MUTED)).child(note.into()))
}

/// A game title as a file name: without the characters file systems refuse.
fn file_stem(title: &str) -> String {
    let stem: String = title.chars().map(|c| if c.is_control() || "/\\:*?\"<>|".contains(c) { '_' } else { c }).collect();
    let stem = stem.trim().trim_matches('.');
    if stem.is_empty() { "game".into() } else { stem.to_string() }
}

/// A stable colour for a game without a thumbnail, from its title.
pub fn title_hue(title: &str) -> f32 {
    let sum: u32 = title.bytes().fold(7u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    (sum % 360) as f32 / 360.
}
