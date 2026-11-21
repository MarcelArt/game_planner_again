//! The Requirements page: plans along the top, a grid of gather and craft tiles, and a panel
//! explaining the selected tile.

use gpui_kit::component::input::Input;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::TestSupportExt as _;
use gpui_kit::*;
use uuid::Uuid;

use super::{Kind, Page, Planner, ReqTab, button, page_title, pal, pill, section_title, well};
use crate::calc::{CraftRow, GatherRow};
use crate::domain::fmt_amount as f;

const TILE_W: f32 = 132.;

fn crafts_text(n: u64) -> String {
    if n == 1 { "1 craft".to_string() } else { format!("{n} crafts") }
}

fn stat(label: &'static str, value: impl IntoElement) -> Div {
    h_flex()
        .justify_between()
        .text_size(px(13.))
        .child(div().text_color(rgb(pal::muted())).child(label))
        .child(div().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(value))
}

fn flag(label: impl Into<SharedString>, bg: Rgba, fg: u32) -> Div {
    h_flex()
        .absolute()
        .left(px(6.))
        .top(px(6.))
        .h(px(22.))
        .px(px(8.))
        .rounded(px(6.))
        .bg(bg)
        .text_color(rgb(fg))
        .text_size(px(12.))
        .font_weight(FontWeight::BOLD)
        .child(label.into())
}

impl Planner {
    pub(super) fn requirements_page(&mut self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        if self.data.plans.is_empty() {
            return self.empty_requirements(cx).into_any_element();
        }
        let ready = self.req.craft.iter().filter(|r| r.ready()).count();
        let missing = self.req.gather.iter().filter(|r| r.missing > 1e-9).count();
        let covered = self.req.gather.len() - missing;
        let tab = self.req_tab;

        let tabs = h_flex().gap(px(4.)).children(
            [
                (ReqTab::All, "All", self.req.gather.len() + self.req.craft.len()),
                (ReqTab::Gather, "Gather", self.req.gather.len()),
                (ReqTab::Craft, "Craft", self.req.craft.len()),
                (ReqTab::Ready, "Ready", ready),
            ]
            .map(|(t, label, count)| {
                let on = t == tab;
                v_flex()
                    .id(label)
                    .w(px(98.))
                    .h(px(44.))
                    .justify_center()
                    .items_center()
                    .rounded_t(px(10.))
                    .cursor_pointer()
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(13.))
                    .text_color(rgb(if on { pal::text() } else { pal::muted() }))
                    .when(on, |el| el.bg(rgb(pal::active_tab())).border_b_2().border_color(rgb(pal::accent())))
                    .hover(|s| s.text_color(rgb(pal::text())))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.req_tab = t;
                        cx.notify();
                    }))
                    .child(h_flex().gap(px(5.)).child(label).child(div().text_color(rgb(if on { pal::accent_text() } else { pal::muted() })).child(count.to_string())))
            }),
        );

        let plans = h_flex().flex_wrap().gap(px(8.)).children(self.data.plans.iter().zip(&self.progress).map(|(p, prog)| {
            let serves = self.sel_req.is_some_and(|sel| self.row_plans(sel).contains(&p.id));
            h_flex()
                .w(px(254.))
                .h(px(54.))
                .px(px(6.))
                .gap(px(10.))
                .rounded(px(10.))
                .bg(rgb(pal::tile()))
                .border_1()
                .border_color(rgb(if serves { pal::accent() } else { pal::edge() }))
                .child(self.pic(p.item_id, 40., 40., 32., 8.))
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .child(div().truncate().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(self.data.item_name(p.item_id).to_string()))
                        .child(div().truncate().text_size(px(13.)).text_color(rgb(pal::muted())).child(if p.note.is_empty() { "No note".to_string() } else { p.note.clone() })),
                )
                .child(if prog.reached {
                    pill("Reached", pal::green_fill(), pal::green())
                } else {
                    pill(format!("{} / {}", f(prog.assigned), f(p.target)), pal::well(), pal::text())
                })
        }));

        let show_gather = matches!(tab, ReqTab::All | ReqTab::Gather);
        let show_craft = matches!(tab, ReqTab::All | ReqTab::Craft | ReqTab::Ready);
        let gather_tiles: Vec<AnyElement> = self.req.gather.iter().map(|r| self.gather_tile(r, cx).into_any_element()).collect();
        let craft_tiles: Vec<AnyElement> = self
            .req
            .craft
            .iter()
            .filter(|r| tab != ReqTab::Ready || r.ready())
            .map(|r| self.craft_tile(r, cx).into_any_element())
            .collect();

        let main = v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .child(
                v_flex()
                    .px(px(40.))
                    .pt(px(28.))
                    .gap(px(14.))
                    .child(page_title("Requirements", format!("{} plans combined", self.data.plans.len())))
                    .child(tabs.border_b_1().border_color(rgb(pal::edge()))),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .px(px(40.))
                    .py(px(18.))
                    .gap(px(14.))
                    .child(section_title("Plans", "combined into the requirements below"))
                    .child(plans)
                    .when(show_gather, |el| {
                        el.child(section_title("Gather", format!("{missing} still missing, {covered} covered by inventory")).mt(px(8.)))
                            .child(h_flex().flex_wrap().items_start().gap(px(10.)).children(gather_tiles))
                    })
                    .when(show_craft, |el| {
                        el.child(section_title("Craft", format!("inputs first, {ready} ready now")).mt(px(8.)))
                            .child(h_flex().flex_wrap().items_start().gap(px(10.)).children(craft_tiles))
                    }),
            );

        h_flex().size_full().items_start().child(main).child(self.requirements_panel(cx)).into_any_element()
    }

    fn empty_requirements(&self, cx: &mut Context<Self>) -> Div {
        let (text, label, page) = if self.data.items.is_empty() {
            ("This game has no items yet. Add items and their recipes first.", "Go to Items", Page::Items)
        } else {
            ("There are no plans yet. A plan names an item and the amount you want to own.", "Go to Plans", Page::Plans)
        };
        v_flex()
            .size_full()
            .px(px(40.))
            .pt(px(28.))
            .gap(px(16.))
            .child(page_title("Requirements", ""))
            .child(div().text_color(rgb(pal::muted())).child(text))
            .child(h_flex().child(button("empty-go", label, Kind::Primary, true).on_click(cx.listener(move |this, _, _, cx| this.go(page, cx)))))
    }

    /// Plans the selected Requirements row serves.
    fn row_plans(&self, item_id: Uuid) -> Vec<Uuid> {
        self.req
            .gather
            .iter()
            .find(|r| r.item_id == item_id)
            .map(|r| r.plans.clone())
            .or_else(|| self.req.craft.iter().find(|r| r.item_id == item_id).map(|r| r.plans.clone()))
            .unwrap_or_default()
    }

    fn tile(&self, item_id: Uuid, top: (u32, u32), cx: &mut Context<Self>) -> Stateful<Div> {
        let selected = self.sel_req == Some(item_id);
        let (from, edge) = top;
        v_flex()
            .id(("req-tile", item_id.as_u128() as usize))
            .relative()
            .w(px(TILE_W))
            .p(px(6.))
            .gap(px(4.))
            .rounded(px(10.))
            .cursor_pointer()
            .bg(linear_gradient(180., linear_color_stop(rgb(from), 0.), linear_color_stop(rgb(if selected { pal::tile_sel() } else { pal::tile() }), 0.55)))
            .border_1()
            .border_color(rgb(if selected { pal::accent() } else { edge }))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.sel_req = Some(item_id);
                cx.notify();
            }))
    }

    fn gather_tile(&self, r: &GatherRow, cx: &mut Context<Self>) -> Stateful<Div> {
        let id = r.item_id;
        let covered = r.missing <= 1e-9;
        let top = if r.recipe_loop.is_some() { (pal::loop_bg(), pal::loop_edge()) } else { (pal::tile(), pal::edge()) };
        let field = self.gather_fields.get(&id).cloned();
        self.tile(id, top, cx)
            .child(self.pic(id, TILE_W - 14., 100., 72., 8.).when(covered, |el| el.opacity(0.6)))
            .when(covered, |el| el.child(flag("Covered", rgba(0x1213149e), 0xe0e1e3)))
            .when(r.recipe_loop.is_some(), |el| el.child(flag("Recipe loop", rgb(pal::steel()), 0x111519)))
            .child(div().px(px(3.)).truncate().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(self.data.item_name(id).to_string()))
            .child(div().px(px(3.)).child(stat("Missing", if covered { "none".to_string() } else { f(r.missing) })))
            .child(div().px(px(3.)).child(stat("Have", format!("{} / {}", f(r.have), f(r.need)))))
            .when_some(field, |el, field| {
                el.child(
                    well()
                        .mt(px(2.))
                        .child(div().flex_1().min_w_0().child(Input::new(&field).id(format!("gather-{}", self.data.item_name(id))).appearance(false)))
                        .child(
                            h_flex()
                                .id(("gather-add", id.as_u128() as usize))
                                .h(px(26.))
                                .mx(px(3.))
                                .px(px(7.))
                                .rounded(px(5.))
                                .bg(rgb(pal::btn()))
                                .text_size(px(12.5))
                                .font_weight(FontWeight::BOLD)
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(pal::btn_hover())))
                                .on_click(cx.listener(move |this, _, window, cx| this.add_gathered(id, window, cx)))
                                .child("Add"),
                        ),
                )
            })
    }

    fn craft_tile(&self, r: &CraftRow, cx: &mut Context<Self>) -> Stateful<Div> {
        let id = r.item_id;
        let done = r.crafts == 0;
        let ready = r.ready();
        let top = if ready { (pal::ready_bg(), pal::green_edge()) } else { (pal::tile(), pal::edge()) };
        let can = r.can_craft;
        self.tile(id, top, cx)
            .child(self.pic(id, TILE_W - 14., 100., 72., 8.).when(done, |el| el.opacity(0.6)))
            .child(if ready {
                flag("Ready", rgb(pal::green()), pal::green_ink())
            } else if done {
                flag("Covered", rgba(0x1213149e), 0xe0e1e3)
            } else {
                flag("Not ready", rgba(0x1213149e), 0xd1d2d4)
            })
            .child(div().px(px(3.)).truncate().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(self.data.item_name(id).to_string()))
            .child(div().px(px(3.)).child(stat("Crafts", r.crafts.to_string())))
            .child(
                div().px(px(3.)).child(stat(
                    "Makes",
                    h_flex()
                        .gap(px(4.))
                        .child(f(r.makes))
                        .when(r.surplus > 1e-9, |el| el.child(div().text_size(px(12.)).text_color(rgb(pal::accent_text())).child(format!("+{}", f(r.surplus))))),
                )),
            )
            .child(
                button(("craft-all", id.as_u128() as usize), format!("Craft all {can}"), Kind::Go, can > 0)
                    .mt(px(2.))
                    .when(can > 0, |el| el.on_click(cx.listener(move |this, _, window, cx| this.craft(id, can, window, cx)))),
            )
    }

    fn requirements_panel(&self, cx: &mut Context<Self>) -> Div {
        let body = v_flex().size_full().rounded(px(12.)).bg(rgb(pal::detail_bg())).border_1().border_color(rgb(pal::edge()));
        let craft = self.sel_req.and_then(|id| self.req.craft.iter().find(|r| r.item_id == id));
        let gather = self.sel_req.and_then(|id| self.req.gather.iter().find(|r| r.item_id == id));
        let body = match (craft, gather) {
            (Some(r), _) => self.craft_panel(body, r, cx),
            (_, Some(r)) => self.gather_panel(body, r),
            _ => body.p(px(16.)).text_color(rgb(pal::muted())).child("Select a tile to see what it needs."),
        };
        v_flex().w(px(360.)).flex_none().h_full().p(px(16.)).pl_0().child(body)
    }

    fn panel_head(&self, id: Uuid, sub: String, state: Div) -> Div {
        h_flex()
            .items_start()
            .gap(px(14.))
            .child(self.pic(id, 84., 84., 72., 10.))
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(4.))
                    .child(div().text_size(px(19.)).font_weight(FontWeight::EXTRA_BOLD).text_color(rgb(pal::text())).child(self.data.item_name(id).to_string()))
                    .child(div().text_size(px(13.)).text_color(rgb(pal::muted())).child(sub))
                    .child(h_flex().child(state)),
            )
    }

    fn panel_label(text: &'static str) -> Div {
        div().mt(px(6.)).font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(text)
    }

    fn sums(rows: Vec<(&'static str, String, u32)>) -> Div {
        v_flex().rounded(px(8.)).bg(rgb(pal::well())).px(px(12.)).py(px(4.)).children(rows.into_iter().enumerate().map(|(i, (label, value, color))| {
            h_flex()
                .h(px(27.))
                .justify_between()
                .when(i > 0, |el| el.border_t_1().border_color(rgb(pal::row_edge())))
                .child(div().text_color(rgb(pal::soft())).child(label))
                .child(div().font_weight(FontWeight::BOLD).text_color(rgb(color)).child(value))
        }))
    }

    fn plans_served(&self, plans: &[Uuid]) -> Div {
        v_flex().gap(px(6.)).children(plans.iter().filter_map(|pid| self.data.plans.iter().find(|p| p.id == *pid)).map(|p| {
            h_flex()
                .h(px(38.))
                .px(px(8.))
                .gap(px(8.))
                .rounded(px(8.))
                .bg(rgb(pal::row_bg()))
                .child(self.pic(p.item_id, 28., 28., 24., 6.))
                .child(div().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(self.data.item_name(p.item_id).to_string()))
                .child(div().flex_1().min_w_0().truncate().text_size(px(13.)).text_color(rgb(pal::muted())).child(p.note.clone()))
                .child(div().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(format!("target {}", f(p.target))))
        }))
    }

    fn craft_panel(&self, body: Div, r: &CraftRow, cx: &mut Context<Self>) -> Div {
        let id = r.item_id;
        let can = r.can_craft;
        let state = if r.ready() {
            pill("Ready", pal::green(), pal::green_ink())
        } else if r.crafts == 0 {
            pill("Covered by inventory", pal::slate(), pal::slate_text())
        } else {
            pill("Not ready", pal::slate(), pal::slate_text())
        };
        let inputs = v_flex().gap(px(6.)).children(r.inputs.iter().map(|i| {
            let short = (i.total - i.have).max(0.);
            h_flex()
                .h(px(54.))
                .px(px(6.))
                .gap(px(10.))
                .rounded(px(10.))
                .border_1()
                .border_color(rgb(pal::edge()))
                .child(self.pic(i.item_id, 44., 44., 36., 8.))
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .child(div().truncate().font_weight(FontWeight::BOLD).text_color(rgb(pal::text())).child(self.data.item_name(i.item_id).to_string()))
                        .child(
                            h_flex()
                                .gap(px(6.))
                                .text_size(px(13.))
                                .text_color(rgb(pal::muted()))
                                .child(format!("{} per craft", f(i.per_craft)))
                                .when(short > 1e-9, |el| el.child(div().text_color(rgb(pal::red())).child(format!("{} short", f(short))))),
                        ),
                )
                .child(pill(format!("{} / {}", f(i.have), f(i.total)), pal::well(), pal::text()))
        }));
        body.child(
            v_flex()
                .flex_1()
                .min_h_0()
                .overflow_y_scrollbar()
                .p(px(16.))
                .gap(px(10.))
                .child(self.panel_head(id, format!("Craft, default recipe makes {}", f(r.output)), state))
                .child(h_flex().justify_between().child(Self::panel_label("Inputs")).child(div().mt(px(6.)).text_size(px(13.)).text_color(rgb(pal::muted())).child(format!("in inventory / needed for {}", crafts_text(r.crafts)))))
                .child(inputs)
                .child(Self::panel_label("The arithmetic"))
                .child(Self::sums(vec![
                    ("Needed", f(r.need), pal::text()),
                    ("In inventory", f(r.have), pal::text()),
                    ("Missing", f(r.missing), pal::text()),
                    ("Crafts, whole", r.crafts.to_string(), pal::accent()),
                    ("Makes", f(r.makes), pal::text()),
                    ("Surplus", f(r.surplus), pal::accent_text()),
                ]))
                .child(Self::panel_label("Plans served"))
                .child(self.plans_served(&r.plans)),
        )
        .child(
            v_flex()
                .p(px(16.))
                .gap(px(10.))
                .border_t_1()
                .border_color(rgb(pal::edge()))
                .child(div().text_size(px(13.)).child(format!("Inventory covers {can} of {} now.", crafts_text(r.crafts))))
                .child(
                    h_flex()
                        .gap(px(8.))
                        .child(
                            button("panel-craft-1", "Craft 1", Kind::Plain, can > 0)
                                .h(px(42.))
                                .flex_1()
                                .when(can > 0, |el| el.on_click(cx.listener(move |this, _, window, cx| this.craft(id, 1, window, cx))))
                                .test_support(),
                        )
                        .child(
                            button("panel-craft-all", format!("Craft all {can}"), Kind::Go, can > 0)
                                .h(px(42.))
                                .flex_1()
                                .when(can > 0, |el| el.on_click(cx.listener(move |this, _, window, cx| this.craft(id, can, window, cx))))
                                .test_support(),
                        ),
                ),
        )
    }

    fn gather_panel(&self, body: Div, r: &GatherRow) -> Div {
        let id = r.item_id;
        let covered = r.missing <= 1e-9;
        let state = if r.recipe_loop.is_some() {
            pill("Recipe loop", pal::steel(), 0x111519)
        } else if covered {
            pill("Covered by inventory", pal::green_fill(), pal::green())
        } else {
            pill("Gather", pal::slate(), pal::slate_text())
        };
        let sub = if self.data.is_raw(id) { "Raw material" } else { "Gathered because of a recipe loop" };
        body.child(
            v_flex()
                .flex_1()
                .min_h_0()
                .overflow_y_scrollbar()
                .p(px(16.))
                .gap(px(10.))
                .child(self.panel_head(id, sub.to_string(), state))
                .when_some(r.recipe_loop.as_ref(), |el, path| {
                    let chain = path.iter().map(|i| self.data.item_name(*i).to_string()).collect::<Vec<_>>().join(" to ");
                    el.child(
                        div()
                            .p(px(10.))
                            .rounded(px(8.))
                            .bg(rgb(pal::loop_bg()))
                            .text_size(px(13.))
                            .child(format!("Its default recipes lead back to itself: {chain}. The calculation stops here and asks you to gather it.")),
                    )
                })
                .child(Self::panel_label("The arithmetic"))
                .child(Self::sums(vec![
                    ("Needed", f(r.need), pal::text()),
                    ("In inventory", f(r.have), pal::text()),
                    ("Missing", f(r.missing), if covered { pal::green() } else { pal::accent() }),
                ]))
                .child(Self::panel_label("Plans served"))
                .child(self.plans_served(&r.plans)),
        )
        .child(
            div()
                .p(px(16.))
                .border_t_1()
                .border_color(rgb(pal::edge()))
                .text_size(px(13.))
                .child("Record what you gathered in the field on its tile. It is added to inventory."),
        )
    }
}
