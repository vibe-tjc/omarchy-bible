#!/usr/bin/env bash
# Pack the release binary + desktop entry + license into dist/omarchy-bible-VERSION-x86_64.tar.gz
set -euo pipefail

VERSION="${1:?version (e.g. 0.1.0)}"
BINARY="${2:?path to omarchy-bible binary}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DIST="$ROOT/dist"
STAGE="$DIST/stage"

rm -rf "$STAGE"
mkdir -p "$STAGE" "$DIST"

cp "$BINARY" "$STAGE/omarchy-bible"
chmod 755 "$STAGE/omarchy-bible"
cp "$ROOT/resources/omarchy-bible.desktop" "$STAGE/omarchy-bible.desktop"
cp "$ROOT/LICENSE" "$STAGE/LICENSE"

ASSET="omarchy-bible-${VERSION}-x86_64.tar.gz"
tar -C "$STAGE" -czf "$DIST/$ASSET" omarchy-bible omarchy-bible.desktop LICENSE
echo "wrote $DIST/$ASSET"
