#!/usr/bin/env bash
# Build the app-owned desktop audience helper beside the Rust binary.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PROFILE="${1:-debug}"
[[ "$PROFILE" == debug || "$PROFILE" == release ]] || { echo 'Use debug or release' >&2; exit 1; }
QMAKE="${QMAKE:-$(command -v qmake6 || command -v qmake || true)}"
[[ -n "$QMAKE" ]] || { echo 'Qt 6 development tools required (Arch: qt6-base; macOS: brew install qt)' >&2; exit 1; }
[[ "$("$QMAKE" -query QT_VERSION)" == 6.* ]] || { echo 'Qt 6 qmake is required' >&2; exit 1; }
BUILD="$ROOT/target/projector-$PROFILE"
mkdir -p "$BUILD" "$ROOT/target/$PROFILE"
cd "$BUILD"
"$QMAKE" "$ROOT/native/projector/projector.pro"
make -j"${CARGO_BUILD_JOBS:-4}"
cp omarchy-bible-projector "$ROOT/target/$PROFILE/omarchy-bible-projector"
echo "Built $ROOT/target/$PROFILE/omarchy-bible-projector"
