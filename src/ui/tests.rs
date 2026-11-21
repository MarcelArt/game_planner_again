//! UI tests: the real window on a temporary database, driven through clicks and keys.

use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::test::{TestAppContextExt, TestWindowExt};
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
    db.save_tutorial(super::Tutorial::Off).unwrap();
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

/// Exports the fixture's game with its progress, then changes the game so that an import shows.
fn exported_then_changed(fx: &Fixture) -> PathBuf {
    let path = fx._dir.path().join("Hollowdeep.zip");
    fx.db.export_game(fx.game, true, &path).unwrap();
    fx.db.add_item(fx.game, "Lantern").unwrap();
    fx.db.set_inventory(fx.coal, 99.0).unwrap();
    path
}

/// Starts importing `path` and waits for the dialog asking what to do about the existing game.
async fn import_into_clash(cx: &mut TestAppContext, window: AnyWindowHandle, planner: &Entity<Planner>, path: PathBuf) {
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        planner.update(cx, |this, cx| this.import_path(&path, window, cx));
    })
    .unwrap();
    cx.wait_for(window, Duration::from_millis(500), |window, _| window.try_find("import-merge").is_some()).await;
}

#[gpui_kit::test]
async fn importing_a_game_file_with_a_new_title_adds_the_game_without_asking(cx: &mut TestAppContext) {
    let fx = fixture();
    let path = exported_then_changed(&fx);
    fx.db.rename_game(fx.game, "Saltwind Isles").unwrap();
    let (window, planner) = open(cx, fx.db.clone());
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        planner.update(cx, |this, cx| this.import_path(&path, window, cx));
        window.render_frame(cx);
        assert!(window.try_find("import-merge").is_none());
    })
    .unwrap();
    let titles: Vec<String> = fx.db.games().unwrap().into_iter().map(|g| g.title).collect();
    assert_eq!(titles, ["Saltwind Isles", "Hollowdeep"]);
}

#[gpui_kit::test]
async fn merge_keeps_the_existing_game_and_its_progress(cx: &mut TestAppContext) {
    let fx = fixture();
    let path = exported_then_changed(&fx);
    let (window, planner) = open(cx, fx.db.clone());
    import_into_clash(cx, window, &planner, path).await;
    cx.update_window(window, |_, window, cx| window.click("import-merge", cx)).unwrap();
    cx.wait_for(window, Duration::from_millis(500), |window, _| window.try_find("import-merge").is_none()).await;
    assert_eq!(fx.db.games().unwrap().len(), 1);
    let data = fx.db.load_game(fx.game).unwrap();
    assert_eq!((data.items.len(), data.owned(fx.coal)), (4, 99.0));
}

#[gpui_kit::test]
async fn replace_asks_again_before_discarding_the_existing_game(cx: &mut TestAppContext) {
    let fx = fixture();
    let path = exported_then_changed(&fx);
    let (window, planner) = open(cx, fx.db.clone());
    import_into_clash(cx, window, &planner, path).await;
    cx.update_window(window, |_, window, cx| window.click("import-replace", cx)).unwrap();
    cx.wait_for(window, Duration::from_millis(500), |window, _| window.try_find("import-merge").is_none() && window.try_find("ok").is_some()).await;
    // Nothing has happened yet: the existing game is still there, changes and all.
    assert_eq!(fx.db.load_game(fx.game).unwrap().items.len(), 4);

    cx.update_window(window, |_, window, cx| window.click("ok", cx)).unwrap();
    cx.wait_for(window, Duration::from_millis(500), |window, _| window.try_find("ok").is_none()).await;
    let games = fx.db.games().unwrap();
    assert_eq!(games.len(), 1);
    assert_ne!(games[0].id, fx.game);
    let data = fx.db.load_game(games[0].id).unwrap();
    let coal = data.items.iter().find(|i| i.name == "Coal").unwrap().id;
    assert_eq!((data.items.len(), data.owned(coal), data.plans.len()), (3, 5.0, 1));
}

#[gpui_kit::test]
async fn keep_both_imports_under_a_numbered_title(cx: &mut TestAppContext) {
    let fx = fixture();
    let path = exported_then_changed(&fx);
    let (window, planner) = open(cx, fx.db.clone());
    import_into_clash(cx, window, &planner, path).await;
    cx.update_window(window, |_, window, cx| window.click("import-keep-both", cx)).unwrap();
    cx.wait_for(window, Duration::from_millis(500), |window, _| window.try_find("import-merge").is_none()).await;
    let titles: Vec<String> = fx.db.games().unwrap().into_iter().map(|g| g.title).collect();
    assert_eq!(titles, ["Hollowdeep", "Hollowdeep (2)"]);
    assert_eq!(fx.db.load_game(fx.game).unwrap().items.len(), 4);
}

#[gpui_kit::test]
async fn the_export_dialog_leaves_progress_out_unless_its_box_is_ticked(cx: &mut TestAppContext) {
    let fx = fixture();
    let (window, planner) = open(cx, fx.db.clone());
    let plain = fx._dir.path().join("plain.zip");
    let full = fx._dir.path().join("full.zip");
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        planner.update(cx, |this, cx| this.export_dialog(fx.game, window, cx));
    })
    .unwrap();
    cx.wait_for(window, Duration::from_millis(500), |window, _| window.try_find("export-progress").is_some()).await;
    cx.update_window(window, |_, window, cx| {
        assert_eq!(window.find("export-progress").checked(), Some(false));
        let include = planner.read(cx).export_progress;
        planner.update(cx, |this, cx| this.export_to(fx.game, include, &plain, window, cx));
        window.click("export-progress", cx);
        window.render_frame(cx);
        assert_eq!(window.find("export-progress").checked(), Some(true));
        let include = planner.read(cx).export_progress;
        planner.update(cx, |this, cx| this.export_to(fx.game, include, &full, window, cx));
    })
    .unwrap();
    assert!(crate::game_file::GameFile::read(&plain).unwrap().progress.is_none());
    assert_eq!(crate::game_file::GameFile::read(&full).unwrap().progress.unwrap().plans.len(), 1);
}

#[gpui_kit::test]
fn theme_switch_persists_both_modes(cx: &mut TestAppContext) {
    let fx = fixture();
    let (window, _) = open(cx, fx.db.clone());
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("Settings", cx);
        for (id, light) in [("theme-light", true), ("theme-light", true), ("theme-dark", false)] {
            window.render_frame(cx);
            window.click(id, cx);
            assert_eq!(fx.db.appearance().unwrap().light, light);
        }
    }).unwrap();
}

#[gpui_kit::test]
fn settings_preserves_legacy_mode_and_custom_palette(cx: &mut TestAppContext) {
    use crate::appearance::Preset;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("appearance"), "light").unwrap();
    let db = Rc::new(Db::open(dir.path()).unwrap());
    let (window, planner) = open(cx, db.clone());
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("Settings", cx);
        window.render_frame(cx);
        window.click("Gruvbox", cx);
        let prefs = db.appearance().unwrap();
        assert!(prefs.light);
        assert_eq!(prefs.preset, Preset::Gruvbox);
        let field = planner.read(cx).color_fields[2].clone();
        field.update(cx, |state, cx| state.commit_hex("#317AB4", window, cx));
    }).unwrap();
    cx.update_window(window, |_, window, cx| {
        assert_eq!(db.appearance().unwrap().preset, Preset::Custom);
        assert_eq!(super::pal::accent(), 0x317ab4);
        for id in ["Default", "Custom", "theme-dark", "theme-light"] {
            window.render_frame(cx);
            window.click(id, cx);
        }
        assert_eq!(super::pal::accent(), 0x317ab4);
    }).unwrap();
    assert_eq!(Db::open(dir.path()).unwrap().appearance().unwrap().colors().accent, 0x317ab4);
}

#[gpui_kit::test]
fn tutorial_waits_resumes_finishes_and_replays(cx: &mut TestAppContext) {
    use super::Tutorial::*;
    let dir = tempfile::tempdir().unwrap();
    let db = Rc::new(Db::open(dir.path()).unwrap());
    assert_eq!(db.tutorial().unwrap(), Library);
    let (window, planner) = open(cx, db.clone());
    cx.update_window(window, |_, window, cx| {
        window.render_frame(cx);
        window.click("tutorial-next", cx);
        assert_eq!(db.tutorial().unwrap(), Waiting);
        let game = db.add_game("Tutorial game").unwrap();
        planner.update(cx, |view, cx| view.open_game(game, window, cx));
        assert_eq!(db.tutorial().unwrap(), Intro);
        for step in [Items, Inventory] {
            window.render_frame(cx);
            window.click("tutorial-next", cx);
            assert_eq!(db.tutorial().unwrap(), step);
        }
    }).unwrap();

    let (window, planner) = open(cx, Rc::new(Db::open(dir.path()).unwrap()));
    cx.update_window(window, |_, window, cx| {
        assert!(planner.read(cx).game.is_none());
        let id = db.games().unwrap()[0].id;
        planner.update(cx, |view, cx| view.open_game(id, window, cx));
        assert!(planner.read(cx).page == super::Page::Inventory);
        for step in [Plans, Requirements, Off] {
            window.render_frame(cx);
            window.click("tutorial-next", cx);
            assert_eq!(db.tutorial().unwrap(), step);
        }
        for id in ["Settings", "tutorial-replay", "tutorial-skip"] {
            window.render_frame(cx);
            window.click(id, cx);
        }
        assert_eq!(db.tutorial().unwrap(), Off);
        assert!(db.load_game(id).unwrap().items.is_empty());
    }).unwrap();
}
