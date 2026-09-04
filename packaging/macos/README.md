# macOS packaging assets

- `Info.plist` — template for `Omarchy Bible.app` (`@VERSION@` substituted by `package-macos.sh`).
- `AppIcon.icns` / `AppIcon-1024.png` — **placeholder** icon (blue rounded square + 「聖」). Replace with final art when available; regenerate `.icns` with `iconutil` from an `.iconset`.

Bundle id: `dev.vibe-tjc.omarchy-bible`. Display name: 聖經. Bundle folder name: `Omarchy Bible.app`. Unsigned — no codesign/notarize in this repo yet.
