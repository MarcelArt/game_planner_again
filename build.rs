//! Embeds the Windows application manifest (per-monitor DPI awareness, modern common controls).
//! gpui-kit leaves the manifest to the application; without one Windows scales the window as a
//! blurry bitmap on high-DPI displays. Does nothing for other targets.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/windows/game_planner.rc");
    println!("cargo:rerun-if-changed=assets/windows/game_planner.manifest.xml");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/windows/game_planner.rc", embed_resource::NONE).manifest_required().unwrap();
    }
}
