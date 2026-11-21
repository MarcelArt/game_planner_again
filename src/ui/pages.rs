//! The Game library, Plans, Items and Inventory pages.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::select::Select;
use gpui_kit::component::{Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use uuid::Uuid;

use super::{ImageTarget, Kind, Planner, button, initials, page_title, pal, pill, section_title, title_hue, well};
use crate::domain::{Recipe, fmt_amount as f};

fn key(id: Uuid) -> usize {
    id.as_u128() as usize
}

/// A page: a fixed head and a scrolling body.
fn page(head: Div, body: Div) -> Div {
    v_flex()
        .size_full()
        .child(head.px(px(40.)).pt(px(28.)).pb(px(14.)))
        .child(body.flex_1().min_h_0().overflow_y_scrollbar().px(px(40.)).pb(px(24.)))
}

fn field_box(input: Input) -> Div {
    well().h(px(38.)).px(px(4.)).child(input.appearance(false))
}

impl Planner {
    // ---- game library ----

    pub(super) fn library_page(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let cards: Vec<AnyElement> = self
            .games
            .iter()
            .map(|(game, stats)| {
                let id = game.id;
                let open = self.game == Some(id);
                let hue = title_hue(&game.title);
                let cover = div().w_full().h(px(150.)).rounded(px(8.)).overflow_hidden().relative();
                let cover = match &game.thumbnail {
                    Some(relative) => cover.child(img(self.db.image_path(relative)).size_full().object_fit(ObjectFit::Cover)),
                    None => cover
                        .bg(linear_gradient(150., linear_color_stop(hsla(hue, 0.42, 0.40, 1.), 0.), linear_color_stop(hsla(hue, 0.38, 0.22, 1.), 1.)))
                        .flex()
                        .items_end()
                        .p(px(12.))
                        .child(
                            h_flex()
                                .size(px(50.))
                                .justify_center()
                                .rounded(px(8.))
                                .bg(rgba(0x12131480))
                                .border_1()
                                .border_color(rgba(0xffffff38))
                                .text_size(px(19.))
                                .font_weight(FontWeight::EXTRA_BOLD)
                                .text_color(rgb(0xffffff))
                                .child(initials(&game.title)),
                        ),
                };
                let count = |n: i64, label: &'static str| {
                    v_flex()
                        .flex_1()
                        .h(px(50.))
                        .items_center()
                        .justify_center()
                        .rounded(px(7.))
                        .bg(rgb(pal::well()))
                        .child(div().text_size(px(16.)).font_weight(FontWeight::EXTRA_BOLD).text_color(rgb(pal::text())).child(n.to_string()))
                        .child(div().text_size(px(12.)).text_color(rgb(pal::muted())).child(label))
                };
                v_flex()
                    .w(px(320.))
                    .p(px(8.))
                    .gap(px(10.))
                    .rounded(px(12.))
                    .bg(rgb(pal::tile()))
                    .border_1()
                    .border_color(rgb(if open { pal::accent() } else { pal::edge() }))
                    .child(cover)
                    .child(
                        h_flex()
                            .px(px(6.))
                            .gap(px(8.))
                            .child(div().flex_1().min_w_0().truncate().text_size(px(17.)).font_weight(FontWeight::EXTRA_BOLD).text_color(rgb(pal::text())).child(game.title.clone()))
                            .when(open, |el| el.child(pill("Open", pal::accent_fill(), pal::accent_text()))),
                    )
                    .child(h_flex().gap(px(6.)).child(count(stats.items, "items")).child(count(stats.recipes, "recipes")).child(count(stats.plans, "plans")))
                    .child(button(("game-open", key(id)), "Open", Kind::Primary, true).on_click(cx.listener(move |this, _, window, cx| this.open_game(id, window, cx))))
                    .child(
                        h_flex()
                            .gap(px(6.))
                            .child(button(("game-rename", key(id)), "Rename", Kind::Plain, true).flex_1().px(px(6.)).on_click(cx.listener(move |this, _, window, cx| this.rename_game_dialog(id, window, cx))))
                            .child(
                                button(("game-thumb", key(id)), "Image", Kind::Plain, true)
                                    .flex_1()
                                    .px(px(6.))
                                    .on_click(cx.listener(move |this, _, window, cx| this.pick_image(ImageTarget::GameThumbnail(id), window, cx))),
                            )
                            .child(button(("game-export", key(id)), "Export", Kind::Plain, true).flex_1().px(px(6.)).on_click(cx.listener(move |this, _, window, cx| this.export_dialog(id, window, cx))))
                            .child(button(("game-del", key(id)), "Delete", Kind::Danger, true).flex_1().px(px(6.)).on_click(cx.listener(move |this, _, window, cx| this.confirm_delete_game(id, window, cx)))),
                    )
                    .into_any_element()
            })
            .collect();

        let add = v_flex()
            .w(px(320.))
            .p(px(16.))
            .gap(px(10.))
            .rounded(px(12.))
            .border_1()
            .border_color(rgb(pal::edge()))
            .child(div().text_size(px(15.)).font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child("Add game"))
            .child(div().text_size(px(13.)).text_color(rgb(pal::muted())).child("A thumbnail is optional. Without one the game gets a colour of its own."))
            .child(field_box(Input::new(&self.new_game)))
            .child(button("game-add", "Add game", Kind::Primary, true).on_click(cx.listener(|this, _, window, cx| this.add_game(window, cx))))
            .child(h_flex().h(px(18.)).child(div().flex_1().h(px(1.)).bg(rgb(pal::edge()))))
            .child(div().text_size(px(13.)).text_color(rgb(pal::muted())).child("Or bring in a game file that was exported from Game Planner."))
            .child(button("game-import", "Import game", Kind::Plain, true).on_click(cx.listener(|this, _, window, cx| this.pick_game_file(window, cx))));

        let note = match self.games.len() {
            0 => "No games yet. Items, recipes, inventory and plans belong to one game each.".to_string(),
            1 => "1 game".to_string(),
            n => format!("{n} games"),
        };
        page(v_flex().child(page_title("Game library", note)), v_flex().child(h_flex().flex_wrap().items_start().gap(px(16.)).children(cards).child(add)))
    }

    // ---- plans ----

    pub(super) fn plans_page(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let reached = self.progress.iter().filter(|p| p.reached).count();
        let label = |text: &'static str| div().text_size(px(13.)).text_color(rgb(pal::muted())).child(text);
        let form = h_flex()
            .items_end()
            .gap(px(10.))
            .p(px(10.))
            .rounded(px(12.))
            .bg(rgb(pal::tile()))
            .border_1()
            .border_color(rgb(pal::edge()))
            .child(v_flex().gap(px(4.)).w(px(260.)).child(label("Item")).child(Select::new(&self.plan_item).placeholder("Choose an item").large()))
            .child(v_flex().gap(px(4.)).w(px(120.)).child(label("Target")).child(field_box(Input::new(&self.plan_target))))
            .child(v_flex().gap(px(4.)).flex_1().child(label("Note")).child(field_box(Input::new(&self.plan_note))))
            .child(button("plan-add", "Add plan", Kind::Primary, true).h(px(38.)).on_click(cx.listener(|this, _, window, cx| this.add_plan(window, cx))));

        let rows: Vec<AnyElement> = self
            .data
            .plans
            .iter()
            .zip(&self.progress)
            .map(|(p, prog)| {
                let id = p.id;
                let share = if p.target > 0. { (prog.assigned / p.target).clamp(0., 1.) as f32 } else { 0. };
                h_flex()
                    .h(px(58.))
                    .px(px(8.))
                    .gap(px(14.))
                    .rounded(px(10.))
                    .bg(rgb(if prog.reached { pal::reached_bg() } else { pal::tile() }))
                    .border_1()
                    .border_color(rgb(if prog.reached { pal::green_edge() } else { pal::edge() }))
                    .child(self.pic(p.item_id, 40., 40., 32., 8.))
                    .child(
                        h_flex()
                            .w(px(380.))
                            .gap(px(10.))
                            .child(div().text_size(px(17.)).font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(self.data.item_name(p.item_id).to_string()))
                            .when(!p.note.is_empty(), |el| el.child(div().flex_1().min_w_0().truncate().text_color(rgb(pal::muted())).child(p.note.clone()))),
                    )
                    .child(
                        div().flex_1().h(px(8.)).rounded(px(4.)).bg(rgb(pal::well())).child(
                            div().h_full().w(relative(share)).rounded(px(4.)).bg(rgb(if prog.reached { pal::green() } else { pal::accent() })),
                        ),
                    )
                    .child(div().w(px(90.)).text_right().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(format!("{} / {}", f(prog.assigned), f(p.target))))
                    .child(h_flex().w(px(96.)).justify_end().child(if prog.reached {
                        pill("Reached", pal::green(), pal::green_ink())
                    } else {
                        pill(format!("{} to go", f(p.target - prog.assigned)), pal::slate(), pal::slate_text())
                    }))
                    .child(button(("plan-del", key(id)), "Delete", Kind::Danger, true).on_click(cx.listener(move |this, _, window, cx| {
                        let result = this.db.delete_plan(id);
                        this.finish(result, window, cx);
                    })))
                    .into_any_element()
            })
            .collect();

        let note = format!("{} plans, {reached} reached. Inventory is assigned to the oldest plan first.", self.data.plans.len());
        page(v_flex().gap(px(14.)).child(page_title("Plans", note)).child(form), v_flex().gap(px(6.)).children(rows))
    }

    // ---- inventory ----

    pub(super) fn inventory_page(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let _ = cx;
        let need = |id: Uuid| {
            self.req
                .gather
                .iter()
                .find(|r| r.item_id == id)
                .map(|r| r.need)
                .or_else(|| self.req.craft.iter().find(|r| r.item_id == id).map(|r| r.need))
        };
        let tiles: Vec<AnyElement> = self
            .data
            .items
            .iter()
            .map(|item| {
                let id = item.id;
                let needed = need(id);
                let enough = needed.is_some_and(|n| self.data.owned(id) + 1e-9 >= n);
                v_flex()
                    .w(px(132.))
                    .p(px(6.))
                    .gap(px(4.))
                    .rounded(px(10.))
                    .bg(rgb(pal::tile()))
                    .border_1()
                    .border_color(rgb(pal::edge()))
                    .child(self.pic(id, 118., 96., 72., 8.))
                    .child(div().px(px(3.)).truncate().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(item.name.clone()))
                    .child(div().px(px(3.)).text_size(px(13.)).text_color(rgb(if enough { pal::green() } else { pal::muted() })).child(match needed {
                        Some(n) => format!("need {}", f(n)),
                        None => "Not needed".to_string(),
                    }))
                    .when_some(self.inventory_fields.get(&id), |el, field| el.child(well().mt(px(2.)).child(Input::new(field).appearance(false))))
                    .into_any_element()
            })
            .collect();
        let note = if self.data.items.is_empty() { "This game has no items yet." } else { "Type an amount and press Enter. The requirements recalculate from what you own." };
        page(v_flex().child(page_title("Inventory", note)), v_flex().child(h_flex().flex_wrap().items_start().gap(px(10.)).children(tiles)))
    }

    // ---- items ----

    pub(super) fn items_page(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let query = self.search.read(cx).value().trim().to_lowercase();
        let tiles: Vec<AnyElement> = self
            .data
            .items
            .iter()
            .filter(|i| query.is_empty() || i.name.to_lowercase().contains(&query))
            .map(|item| {
                let id = item.id;
                let selected = self.sel_item == Some(id);
                let recipes = self.data.recipes_of(id).count();
                v_flex()
                    .id(("item-tile", key(id)))
                    .w(px(132.))
                    .p(px(6.))
                    .gap(px(3.))
                    .rounded(px(10.))
                    .cursor_pointer()
                    .bg(rgb(if selected { pal::tile_sel() } else { pal::tile() }))
                    .border_1()
                    .border_color(rgb(if selected { pal::accent() } else { pal::edge() }))
                    .on_click(cx.listener(move |this, _, window, cx| this.select_item(id, window, cx)))
                    .child(self.pic(id, 118., 96., 72., 8.))
                    .child(div().px(px(3.)).truncate().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(item.name.clone()))
                    .child(div().px(px(3.)).text_size(px(13.)).text_color(rgb(pal::muted())).child(match recipes {
                        0 => "Raw material".to_string(),
                        1 => "1 recipe".to_string(),
                        n => format!("{n} recipes"),
                    }))
                    .child(
                        h_flex()
                            .px(px(3.))
                            .justify_between()
                            .text_size(px(13.))
                            .child(div().text_color(rgb(pal::muted())).child("Have"))
                            .child(div().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(f(self.data.owned(id)))),
                    )
                    .into_any_element()
            })
            .collect();

        let head = v_flex().gap(px(14.)).child(page_title("Items", format!("{} items, {} recipes", self.data.items.len(), self.data.recipes.len()))).child(
            h_flex()
                .gap(px(10.))
                .child(field_box(Input::new(&self.search)).w(px(260.)))
                .child(div().flex_1())
                .child(field_box(Input::new(&self.new_item)).w(px(240.)))
                .child(button("item-add", "Add item", Kind::Primary, true).h(px(38.)).on_click(cx.listener(|this, _, window, cx| this.add_item(window, cx)))),
        );
        let empty = tiles.is_empty();
        let grid = v_flex()
            .child(h_flex().flex_wrap().items_start().gap(px(10.)).children(tiles))
            .when(empty, |el| el.child(div().text_color(rgb(pal::muted())).child(if query.is_empty() { "No items yet. Name one above and press Add item." } else { "No item matches the search." })));

        h_flex().size_full().items_start().child(div().flex_1().min_w_0().h_full().child(page(head, grid))).child(self.item_panel(cx))
    }

    fn chip(&self, item_id: Uuid, amount: f64) -> Div {
        h_flex()
            .h(px(34.))
            .pl(px(3.))
            .pr(px(9.))
            .gap(px(7.))
            .rounded(px(8.))
            .bg(rgb(pal::well()))
            .child(self.pic(item_id, 28., 28., 24., 6.))
            .child(div().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(f(amount)))
            .child(div().text_color(rgb(pal::soft())).child(self.data.item_name(item_id).to_string()))
    }

    fn recipe_card(&self, r: &Recipe, only: bool, cx: &mut Context<Self>) -> Div {
        let id = r.id;
        v_flex()
            .p(px(10.))
            .gap(px(8.))
            .rounded(px(10.))
            .bg(rgb(if r.is_default { pal::default_recipe() } else { pal::tile() }))
            .border_1()
            .border_color(rgb(if r.is_default && !only { pal::accent() } else { pal::edge() }))
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(div().flex_1().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(format!("Makes {} per craft", f(r.output))))
                    .child(if r.is_default {
                        pill("Default recipe", pal::accent_fill(), pal::accent_text()).into_any_element()
                    } else {
                        button(("recipe-default", key(id)), "Make default", Kind::Plain, true)
                            .h(px(26.))
                            .text_size(px(12.5))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                let result = this.db.set_default_recipe(id);
                                this.finish(result, window, cx);
                            }))
                            .into_any_element()
                    }),
            )
            .child(h_flex().flex_wrap().gap(px(6.)).children(r.inputs.iter().map(|i| self.chip(i.item_id, i.amount))))
            .child(h_flex().child(button(("recipe-del", key(id)), "Delete recipe", Kind::Danger, true).h(px(26.)).text_size(px(12.5)).on_click(cx.listener(move |this, _, window, cx| {
                let result = this.db.delete_recipe(id);
                this.finish(result, window, cx);
            }))))
    }

    fn item_panel(&self, cx: &mut Context<Self>) -> Div {
        let body = v_flex().size_full().rounded(px(12.)).bg(rgb(pal::detail_bg())).border_1().border_color(rgb(pal::edge()));
        let shell = |body: Div| v_flex().w(px(400.)).flex_none().h_full().p(px(16.)).pl_0().child(body);
        let Some(item) = self.sel_item.and_then(|id| self.data.item(id)) else {
            return shell(body.p(px(16.)).text_color(rgb(pal::muted())).child("Select an item to see its recipes."));
        };
        let id = item.id;
        let recipes: Vec<&Recipe> = self.data.recipes_of(id).collect();
        let used = self.data.used_in(id);
        let label = |text: &'static str| div().text_size(px(13.)).text_color(rgb(pal::muted())).child(text);

        let head = h_flex()
            .items_start()
            .gap(px(14.))
            .child(self.pic(id, 84., 84., 72., 10.))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(6.))
                    .child(field_box(Input::new(&self.item_name)))
                    .child(
                        h_flex()
                            .gap(px(6.))
                            .child(button("item-rename", "Rename", Kind::Plain, true).h(px(28.)).text_size(px(12.5)).on_click(cx.listener(|this, _, window, cx| this.rename_item(window, cx))))
                            .child(
                                button("item-icon", "Icon", Kind::Plain, true)
                                    .h(px(28.))
                                    .text_size(px(12.5))
                                    .on_click(cx.listener(move |this, _, window, cx| this.pick_image(ImageTarget::ItemIcon(id), window, cx))),
                            )
                            .child(button("item-delete", "Delete", Kind::Danger, true).h(px(28.)).text_size(px(12.5)).on_click(cx.listener(move |this, _, window, cx| this.confirm_delete_item(id, window, cx)))),
                    ),
            );

        let only = recipes.len() == 1;
        let recipe_note = match recipes.len() {
            0 => "none, so this is a raw material",
            1 => "used in the calculation",
            _ => "the default one is used in the calculation",
        };

        let draft = h_flex().flex_wrap().gap(px(6.)).children(self.recipe_draft.iter().map(|i| {
            let item_id = i.item_id;
            self.chip(item_id, i.amount).id(("draft", key(item_id))).cursor_pointer().hover(|s| s.opacity(0.7)).on_click(cx.listener(move |this, _, _, cx| {
                this.recipe_draft.retain(|d| d.item_id != item_id);
                cx.notify();
            }))
        }));
        let new_recipe = v_flex()
            .p(px(10.))
            .gap(px(8.))
            .rounded(px(10.))
            .border_1()
            .border_color(rgb(pal::edge()))
            .child(div().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child("New recipe"))
            .child(h_flex().gap(px(8.)).child(label("One craft makes")).child(field_box(Input::new(&self.recipe_output)).flex_1()))
            .when(!self.recipe_draft.is_empty(), |el| el.child(label("Inputs so far. Click one to remove it.")).child(draft))
            .child(label("Add an input"))
            .child(Select::new(&self.recipe_input_item).placeholder("Input item").large())
            .child(
                h_flex()
                    .gap(px(6.))
                    .child(field_box(Input::new(&self.recipe_input_amount)).flex_1())
                    .child(button("recipe-add-input", "Add input", Kind::Plain, true).h(px(38.)).on_click(cx.listener(|this, _, window, cx| this.add_draft_input(window, cx)))),
            )
            .child(
                button("recipe-save", "Save recipe", Kind::Primary, !self.recipe_draft.is_empty())
                    .when(!self.recipe_draft.is_empty(), |el| el.on_click(cx.listener(|this, _, window, cx| this.save_recipe(window, cx)))),
            );

        shell(
            body.child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .p(px(16.))
                    .gap(px(10.))
                    .child(head)
                    .child(section_title("Recipes", recipe_note).mt(px(6.)))
                    .children(recipes.iter().map(|r| self.recipe_card(r, only, cx)))
                    .child(new_recipe)
                    .child(section_title("Used in", if used.is_empty() { "not an input of any recipe" } else { "recipes that take this as an input" }).mt(px(6.)))
                    .child(h_flex().flex_wrap().gap(px(6.)).children(used.iter().map(|u| {
                        let uid = u.id;
                        h_flex()
                            .id(("used-in", key(uid)))
                            .h(px(34.))
                            .pl(px(3.))
                            .pr(px(9.))
                            .gap(px(7.))
                            .rounded(px(8.))
                            .bg(rgb(pal::well()))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(pal::btn())))
                            .on_click(cx.listener(move |this, _, window, cx| this.select_item(uid, window, cx)))
                            .child(self.pic(uid, 28., 28., 24., 6.))
                            .child(u.name.clone())
                    }))),
            ),
        )
    }
}
