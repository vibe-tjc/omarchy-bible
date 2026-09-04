#!/usr/bin/env bash
# Build Omarchy Bible.app, DMG, and tar.gz into dist/
# Usage: package-macos.sh VERSION [path/to/omarchy-bible]
# If binary path omitted, uses $ROOT/target/release/omarchy-bible.
# Unsigned — no codesign / notarize.
set -euo pipefail

VERSION="${1:?version (e.g. 0.2.0)}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BINARY="${2:-$ROOT/target/release/omarchy-bible}"
DIST="$ROOT/dist"
STAGE="$DIST/stage-macos"
MACOS_SRC="$ROOT/packaging/macos"
APP_NAME="Omarchy Bible.app"
APP="$STAGE/$APP_NAME"

if [[ ! -f "$BINARY" ]]; then
  echo "release binary not found: $BINARY" >&2
  echo "build first: cargo build --release -p omarchy-bible" >&2
  exit 1
fi

if [[ ! -f "$MACOS_SRC/Info.plist" ]]; then
  echo "missing $MACOS_SRC/Info.plist" >&2
  exit 1
fi

rm -rf "$STAGE"
mkdir -p "$STAGE" "$DIST"

# --- .app bundle ---
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BINARY" "$APP/Contents/MacOS/omarchy-bible"
chmod 755 "$APP/Contents/MacOS/omarchy-bible"

sed -e "s/@VERSION@/${VERSION}/g" "$MACOS_SRC/Info.plist" > "$APP/Contents/Info.plist"
echo -n 'APPL????' > "$APP/Contents/PkgInfo"

if [[ -f "$MACOS_SRC/AppIcon.icns" ]]; then
  cp "$MACOS_SRC/AppIcon.icns" "$APP/Contents/Resources/AppIcon.icns"
fi

# Copy finished .app into dist/ for local inspection
rm -rf "$DIST/$APP_NAME"
cp -R "$APP" "$DIST/$APP_NAME"
echo "wrote $DIST/$APP_NAME"

# --- tar.gz (flat binary + LICENSE, same as before) ---
TAR_STAGE="$STAGE/tar"
mkdir -p "$TAR_STAGE"
cp "$BINARY" "$TAR_STAGE/omarchy-bible"
chmod 755 "$TAR_STAGE/omarchy-bible"
cp "$ROOT/LICENSE" "$TAR_STAGE/LICENSE"

ASSET_TGZ="omarchy-bible-${VERSION}-macos-arm64.tar.gz"
tar -C "$TAR_STAGE" -czf "$DIST/$ASSET_TGZ" omarchy-bible LICENSE
echo "wrote $DIST/$ASSET_TGZ"

# --- DMG ---
DMG_NAME="omarchy-bible-${VERSION}-macos-arm64.dmg"
DMG_PATH="$DIST/$DMG_NAME"
rm -f "$DMG_PATH" "$DIST"/rw.*.dmg

DMG_STAGE="$DIST/dmg-stage"
rm -rf "$DMG_STAGE"
mkdir -p "$DMG_STAGE"
cp -R "$APP" "$DMG_STAGE/"
cp "$ROOT/LICENSE" "$DMG_STAGE/LICENSE"

# Volume / Finder title: 「聖經」. Layout: app left, Applications symlink right.
VOLNAME="聖經"

make_dmg_hdiutil() {
  ln -sfn /Applications "$DMG_STAGE/Applications"
  hdiutil detach "/Volumes/${VOLNAME}" -force >/dev/null 2>&1 || true
  rm -f "$DIST"/rw.*.dmg
  hdiutil create \
    -volname "$VOLNAME" \
    -srcfolder "$DMG_STAGE" \
    -ov \
    -format UDZO \
    "$DMG_PATH"
}

if ! command -v create-dmg >/dev/null 2>&1; then
  echo "create-dmg not found; trying brew install create-dmg..." >&2
  if command -v brew >/dev/null 2>&1; then
    brew install create-dmg || true
  fi
fi

if command -v create-dmg >/dev/null 2>&1; then
  hdiutil detach "/Volumes/${VOLNAME}" -force >/dev/null 2>&1 || true
  # --skip-jenkins skips Finder AppleScript (often times out under automation / headless).
  # Still creates Applications drop-link via create-dmg's non-GUI path.
  set +e
  create-dmg \
    --volname "$VOLNAME" \
    --window-pos 200 120 \
    --window-size 600 400 \
    --icon-size 100 \
    --icon "$APP_NAME" 150 190 \
    --app-drop-link 450 190 \
    --no-internet-enable \
    --skip-jenkins \
    --overwrite \
    "$DMG_PATH" \
    "$DMG_STAGE"
  cd_status=$?
  set -e
  if [[ $cd_status -eq 0 && -f "$DMG_PATH" ]]; then
    :
  elif [[ -f "$DMG_PATH" ]]; then
    echo "warning: create-dmg exited ${cd_status} but $DMG_PATH exists; keeping it" >&2
  else
    echo "create-dmg failed (status ${cd_status}); falling back to hdiutil" >&2
    make_dmg_hdiutil
  fi
else
  echo "create-dmg not found; using hdiutil (brew install create-dmg for fancy Finder layout)" >&2
  make_dmg_hdiutil
fi

rm -rf "$DMG_STAGE" "$STAGE"
rm -f "$DIST"/rw.*.dmg
echo "wrote $DMG_PATH"
