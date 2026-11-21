build:
    @cargo build --release

# Live Rust UI edits on Linux. Install dx with cargo install dioxus-cli --version 0.7.10 --locked.
dev:
    dx serve --platform desktop --hot-patch --features hot-reload

# Packs the release build into an AppImage in dist/, for trying the packaging locally.
appimage: build
    @./packaging/linux/appimage.sh

# Releases a version, e.g. `just release 0.2.0`: sets it in Cargo.toml, commits, tags and pushes.
# The pushed tag starts the release workflow, which builds and publishes every platform.
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    [[ "{{version}}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "The version must look like 1.2.3."; exit 1; }
    [[ -z "$(git status --porcelain)" ]] || { echo "Commit or stash your changes first."; exit 1; }
    [[ "$(git branch --show-current)" == master ]] || { echo "Releases are made from master."; exit 1; }
    git rev-parse -q --verify "refs/tags/v{{version}}" > /dev/null && { echo "v{{version}} already exists."; exit 1; }
    cargo test
    sed -i '0,/^version = ".*"/s//version = "{{version}}"/' Cargo.toml
    cargo check --quiet
    git commit --quiet -am "chore: release {{version}}"
    git tag -a "v{{version}}" -m "Game Planner {{version}}"
    git push --atomic origin master "v{{version}}"
    echo "Released v{{version}}. The builds appear under Releases on GitHub when the workflow finishes."
