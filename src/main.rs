// A release build on Windows is a GUI program: no console window opens alongside it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod appearance;
mod calc;
mod db;
mod domain;
mod game_file;
mod ui;

use std::rc::Rc;

use gpui_kit::*;

/// The app's identity, the same on every platform. Debug builds get a `.dev` suffix, like the
/// data folder, so a development build is never taken for the installed app.
const APP_ID: &str = if cfg!(debug_assertions) { "art.bangmarcel.gameplanner.dev" } else { "art.bangmarcel.gameplanner" };

fn main() {
    let db = match db::default_data_dir().and_then(db::Db::open) {
        Ok(db) => Rc::new(db),
        Err(err) => {
            eprintln!("Game Planner could not open its data folder: {err:#}");
            std::process::exit(1);
        }
    };

    // macOS keeps an app running after its last window closes; this one has nothing to do then.
    application().with_assets(assets::AllAssets).with_quit_mode(QuitMode::LastWindowClosed).run(move |cx| {
        init(cx);
        ui::add_fonts(cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1440.), px(900.)), cx)),
            window_min_size: Some(size(px(1100.), px(680.))),
            titlebar: Some(TitlebarOptions { title: Some("Game Planner".into()), ..Default::default() }),
            app_id: Some(APP_ID.into()),
            ..Default::default()
        };
        open_window(options, cx, move |window, cx| cx.new(|cx| ui::Planner::new(db, window, cx))).expect("failed to open window");
    });
}
