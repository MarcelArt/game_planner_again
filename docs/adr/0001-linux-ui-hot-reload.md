# Apply Rust view edits to the running Linux development app

A developer adjusting a screen needs to see the result with the same game open and the same draft in the input field. The planner keeps that state in GPUI entities owned by the running process.

The Linux development command, `just dev`, uses Subsecond to patch the function that renders the planner. After a patch arrives, GPUI redraws the existing view. This lets page layout, labels, and styling stay in the same Rust source used for release builds.

Install the matching CLI once with `cargo install dioxus-cli --version 0.7.10 --locked`. Run `just dev`, type a draft game title on the Library screen, and edit a visible label or move a child in `src/ui/pages.rs`. The open window should redraw with the draft still present. Build diagnostics appear in the development terminal.

Restart the development session after changing the layout of a live Rust type, such as adding a field to `Planner`. The patch boundary reuses existing objects and provides no state migration.
