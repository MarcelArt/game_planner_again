use std::cell::Cell;

// GPUI renders the single planner window and its component theme on the UI thread.
thread_local! { static LIGHT: Cell<bool> = const { Cell::new(false) }; }

pub(super) fn is_light() -> bool { LIGHT.get() }
pub(super) fn set_light(light: bool) { LIGHT.set(light) }

macro_rules! colors {
    ($($name:ident: $dark:literal, $light:literal;)*) => {
        $(pub(super) fn $name() -> u32 { if is_light() { $light } else { $dark } })*
    };
}

colors! {
    bg: 0x191a1c, 0xe7e5df;
    side: 0x232426, 0xdedbd4;
    tile: 0x212225, 0xf5f3ee;
    tile_sel: 0x2a2b2e, 0xd8e9eb;
    edge: 0x2e2f32, 0xd3cfc7;
    edge_2: 0x3a3b3f, 0xb8b3ab;
    well: 0x121314, 0xfbfaf7;
    text: 0xffffff, 0x141410;
    soft: 0xe5e5e7, 0x2c2922;
    muted: 0xa7a8ac, 0x625e54;
    accent: 0xfbd960, 0x087486;
    accent_text: 0xfdeaa4, 0x066270;
    accent_fill: 0x514721, 0xbfe1e5;
    accent_ink: 0x221b02, 0xffffff;
    steel: 0x8799ab, 0x91a8b9;
    green: 0x55c792, 0x196541;
    green_deep: 0x3f9d71, 0x96d5b2;
    go_hover: 0x55c792, 0x80c79e;
    green_fill: 0x173a2a, 0xe0efe5;
    green_edge: 0x2d6a4e, 0x74a58a;
    green_ink: 0x062014, 0xffffff;
    red: 0xff8196, 0x922d42;
    slate: 0x404144, 0xd4d0c8;
    btn: 0x2b2c2e, 0xeae7e0;
    btn_hover: 0x37383c, 0xd5d1c8;
    accent_hover: 0xfbde75, 0x066779;
    accent_active: 0xe9c84f, 0x045463;
    danger_hover: 0x5a2630, 0xf6d7df;
    disabled_text: 0x8a8b8f, 0x716f67;
    well_edge: 0x1c1d1f, 0xcecac2;
    nav_edge: 0x292a2d, 0xc8c4bc;
    active_tab: 0x2c2d30, 0xdce8e9;
    loop_bg: 0x2a2e33, 0xe0e7ed;
    loop_edge: 0x49535d, 0x90a0ad;
    ready_bg: 0x1b4230, 0xd2eadb;
    detail_bg: 0x212124, 0xf4f2ed;
    row_edge: 0x1e1f21, 0xd8d4cd;
    row_bg: 0x26272a, 0xe8e5df;
    slate_text: 0xececed, 0x39362f;
    reached_bg: 0x1c2f26, 0xe1efe6;
    default_recipe: 0x26262a, 0xe9e7e2;
}

pub(super) const TINT_RAW: (u32, u32) = (0x557f6b, 0x446958);
pub(super) const TINT_MID: (u32, u32) = (0x65666b, 0x535559);
pub(super) const TINT_END: (u32, u32) = (0x847a55, 0x706644);
