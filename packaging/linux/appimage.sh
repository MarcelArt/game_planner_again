#!/usr/bin/env bash
# Packs the release build into an AppImage in dist/.
# Run `cargo build --release` first. Published AppImages are built by the release workflow on an
# old Ubuntu, so they start on older distros too; one built on a newer system may not.
set -euo pipefail
cd "$(dirname "$0")/../.."

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
id=art.bangmarcel.gameplanner
tools=target/tools
linuxdeploy=$tools/linuxdeploy-x86_64.AppImage

mkdir -p "$tools" dist
if [[ ! -x $linuxdeploy ]]; then
    curl -fsSL -o "$linuxdeploy" https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
    chmod +x "$linuxdeploy"
fi

rm -rf target/AppDir
# linuxdeploy is itself an AppImage; extracting it avoids needing FUSE (absent on build servers).
APPIMAGE_EXTRACT_AND_RUN=1 \
LDAI_OUTPUT="dist/game-planner-$version-linux-x86_64.AppImage" \
OUTPUT="dist/game-planner-$version-linux-x86_64.AppImage" \
    "$linuxdeploy" \
    --appdir target/AppDir \
    --executable target/release/game-planner \
    --desktop-file "packaging/linux/$id.desktop" \
    --icon-file "packaging/linux/$id.png" \
    --output appimage

echo "dist/game-planner-$version-linux-x86_64.AppImage"
