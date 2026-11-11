#!/usr/bin/env bash
# Build the app-owned desktop audience helper beside the Rust binary:
# GTK 4 (Rust, gtk4-rs) on Linux, AppKit (Swift) on macOS.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PROFILE="${1:-debug}"
[[ "$PROFILE" == debug || "$PROFILE" == release ]] || { echo 'Use debug or release' >&2; exit 1; }
OUT="$ROOT/target/$PROFILE/omarchy-bible-projector"
mkdir -p "$ROOT/target/$PROFILE"
case "$(uname -s)" in
  Linux)
    # Same gtk4-rs crates Native Shell already links: no extra system deps.
    FLAGS=(--locked -p omarchy-bible-projector)
    [[ "$PROFILE" == release ]] && FLAGS+=(--release)
    cargo build "${FLAGS[@]}" --manifest-path "$ROOT/Cargo.toml"
    # Copy via a temporary name: replacing a running helper in place fails
    # with "Text file busy".
    cp "$ROOT/target/$PROFILE/omarchy-bible-projector-gtk" "$OUT.tmp"
    mv -f "$OUT.tmp" "$OUT"
    ;;
  Darwin)
    command -v swiftc >/dev/null || { echo 'swiftc required (Xcode command line tools)' >&2; exit 1; }
    OPT=(-Onone -g); [[ "$PROFILE" == release ]] && OPT=(-O)
    swiftc "${OPT[@]}" -target "$(uname -m)-apple-macos14.0" -framework AppKit \
      "$ROOT/native/projector/macos/main.swift" -o "$OUT.tmp"
    mv -f "$OUT.tmp" "$OUT"
    ;;
  *) echo "No desktop projector helper for $(uname -s)" >&2; exit 1 ;;
esac
echo "Built $OUT"
