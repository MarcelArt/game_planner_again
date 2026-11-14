#!/usr/bin/env bash
# Builds "Game Planner.app" (Apple silicon and Intel in one binary) and a disk image in dist/.
# Runs on macOS only. Build both targets first:
#   cargo build --release --target aarch64-apple-darwin --target x86_64-apple-darwin
set -euo pipefail
cd "$(dirname "$0")/../.."

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
root=target/dmg
app="$root/Game Planner.app"

rm -rf "$root" target/icon.iconset
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources" dist

lipo -create -output "$app/Contents/MacOS/game-planner" \
    target/aarch64-apple-darwin/release/game-planner \
    target/x86_64-apple-darwin/release/game-planner
sed "s/@VERSION@/$version/g" packaging/macos/Info.plist > "$app/Contents/Info.plist"

mkdir target/icon.iconset
for size in 16 32 128 256 512; do
    sips -z $size $size packaging/icon.png --out "target/icon.iconset/icon_${size}x${size}.png" > /dev/null
    sips -z $((size * 2)) $((size * 2)) packaging/icon.png --out "target/icon.iconset/icon_${size}x${size}@2x.png" > /dev/null
done
iconutil -c icns target/icon.iconset -o "$app/Contents/Resources/icon.icns"

# Not signed with a developer certificate. The ad-hoc signature is the minimum Apple silicon
# requires to run a program at all.
codesign --force --deep --sign - "$app"

ln -s /Applications "$root/Applications"
dmg="dist/game-planner-$version-macos-universal.dmg"
rm -f "$dmg"
hdiutil create -volname "Game Planner" -srcfolder "$root" -ov -format UDZO "$dmg"
echo "$dmg"
