//! Embeds the program icon in the Windows executable. Does nothing for other targets.
//! The application manifest (DPI awareness, modern common controls) is not ours to add: the GPUI
//! crate gpui-kit depends on already embeds one, and a second makes the Windows linker fail.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/windows/game_planner.rc");
    println!("cargo:rerun-if-changed=assets/windows/game_planner.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // The include folder is where the resource compiler looks for the icon the script names.
        embed_resource::compile("assets/windows/game_planner.rc", embed_resource::ParamsIncludeDirs(["assets/windows"])).manifest_required().unwrap();
    }
}
