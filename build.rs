//! Embeds the Windows application manifest (per-monitor DPI awareness, modern common controls)
//! and the program icon.
//! gpui-kit leaves the manifest to the application; without one Windows scales the window as a
//! blurry bitmap on high-DPI displays. Does nothing for other targets.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/windows/game_planner.rc");
    println!("cargo:rerun-if-changed=assets/windows/game_planner.manifest.xml");
    println!("cargo:rerun-if-changed=assets/windows/game_planner.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // The include folder is where the resource compiler looks for the manifest and icon the script names.
        embed_resource::compile("assets/windows/game_planner.rc", embed_resource::ParamsIncludeDirs(["assets/windows"])).manifest_required().unwrap();
    }
}
