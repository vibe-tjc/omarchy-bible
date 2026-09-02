#!/usr/bin/env bash
# Pack the release binary + license into dist/omarchy-bible-VERSION-macos-arm64.tar.gz
set -euo pipefail

VERSION="${1:?version (e.g. 0.1.0)}"
BINARY="${2:?path to omarchy-bible binary}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DIST="$ROOT/dist"
STAGE="$DIST/stage-macos"

rm -rf "$STAGE"
mkdir -p "$STAGE" "$DIST"

cp "$BINARY" "$STAGE/omarchy-bible"
chmod 755 "$STAGE/omarchy-bible"
cp "$ROOT/LICENSE" "$STAGE/LICENSE"

ASSET="omarchy-bible-${VERSION}-macos-arm64.tar.gz"
tar -C "$STAGE" -czf "$DIST/$ASSET" omarchy-bible LICENSE
echo "wrote $DIST/$ASSET"
