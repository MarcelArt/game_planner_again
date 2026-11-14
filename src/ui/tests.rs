//! UI tests: the real window on a temporary database, driven through clicks and keys.

use std::rc::Rc;

use gpui_kit::test::TestWindowExt;
use gpui_kit::{AnyWindowHandle, AppContext, Bounds, Entity, Point, TestAppContext, WindowBounds, WindowOptions, px, size};
use uuid::Uuid;

use super::Planner;
use crate::db::Db;
use crate::domain::Input;

struct Fixture {
    _dir: tempfile::TempDir,
    db: Rc<Db>,
    game: Uuid,
    stick: Uuid,
    coal: Uuid,
    torch: Uuid,
}

/// One game: Torch = 1 Stick + 1 Coal makes 4; 3 Sticks and 5 Coal owned; a plan for 40 Torches.
fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let db = Rc::new(Db::open(dir.path()).unwrap());
    let game = db.add_game("Hollowdeep").unwrap();
    let stick = db.add_item(game, "Stick").unwrap();
    let coal = db.add_item(game, "Coal").unwrap();
    let torch = db.add_item(game, "Torch").unwrap();
    db.add_recipe(torch, 4.0, &[Input { item_id: stick, amount: 1.0 }, Input { item_id: coal, amount: 1.0 }]).unwrap();
    db.set_inventory(stick, 3.0).unwrap();
    db.set_inventory(coal, 5.0).unwrap();
    db.add_plan(torch, 40.0, "Light the east mine").unwrap();
    Fixture { _dir: dir, db, game, stick, coal, torch }
}

fn open(cx: &mut TestAppContext, db: Rc<Db>) -> (AnyWindowHandle, Entity<Planner>) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        super::apply_theme(cx);
        let bounds = Bounds { origin: Point::default(), size: size(px(1440.), px(900.)) };
        let options = WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() };
        gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| Planner::new(db, window, cx))).expect("open test window")
    })
}

#[gpui_kit::test]
fn craft_all_performs_the_crafts_inventory_covers(cx: &mut TestAppContext) {
    let fx = fixture();
    let (window, _planner) = open(cx, fx.db.clone());
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("panel-craft-all", cx);
        window.render_frame(cx);
    })
    .unwrap();
    let data = fx.db.load_game(fx.game).unwrap();
    assert_eq!((data.owned(fx.stick), data.owned(fx.coal), data.owned(fx.torch)), (0.0, 2.0, 12.0));
}

#[gpui_kit::test]
fn craft_1_performs_one_craft(cx: &mut TestAppContext) {
    let fx = fixture();
    let (window, _planner) = open(cx, fx.db.clone());
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("panel-craft-1", cx);
        window.render_frame(cx);
    })
    .unwrap();
    let data = fx.db.load_game(fx.game).unwrap();
    assert_eq!((data.owned(fx.stick), data.owned(fx.coal), data.owned(fx.torch)), (2.0, 4.0, 4.0));
}

#[gpui_kit::test]
fn typing_a_gathered_amount_and_pressing_enter_adds_it_to_inventory(cx: &mut TestAppContext) {
    let fx = fixture();
    let (window, _planner) = open(cx, fx.db.clone());
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("gather-Coal", cx);
        window.input("4", cx);
        window.press("enter", cx);
        window.render_frame(cx);
    })
    .unwrap();
    assert_eq!(fx.db.load_game(fx.game).unwrap().owned(fx.coal), 9.0);
}
