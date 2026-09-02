# Performance baseline (before changes)

Captured 2026-09-02 on Mac14,5 (Apple M2 Max, arm64), rustc 1.97.1.
Median of 7 runs, `cargo run --release -p bible-core --example bench`.

Source JSON: `docs/perf-baseline.json`.

## Timing

| Metric | Median | Notes |
| --- | --- | --- |
| `load_bible()` | 8.275 ms | serde parse of embedded 7.7 MB JSON |
| `load_chapter_at` Genesis 1 | 0.007 ms | clone path, 31 verses |
| `load_chapter_at` Psalm 119 | 0.026 ms | clone path, 176 verses |
| search `神愛世人` | 6.896 ms | 1 hit, full scan |
| search `起初` | 5.548 ms | 37 hits, full scan |
| search `God so loved` | 4.638 ms | 2 hits, full scan |

## Dataset / space

| Item | Value |
| --- | --- |
| Books | 66 |
| Chapters | 1189 |
| Verses | 31021 |
| Longest chapter | 詩篇 119 (176 verses) |
| JSON file | 8,071,265 bytes (`data/cuv-kjv.json`) |
| Sum of zh/en string bytes | 7,282,095 |
| Approx in-memory payload | 12,025,823 bytes (text + String/Vec headers) |
| RSS after `load_bible` | 35,104 KB (~34.3 MiB, `ps -o rss=`) |
| `target/release/omarchy-bible` | 24,837,744 bytes (~24 MiB) |
| `target/debug/omarchy-bible` | 84,507,264 bytes (~81 MiB) |

## UI elements (current `verse_block`, not virtualized)

Formula: **3 + N lanes** per verse (`h_flex` root + verse-number `div` + lanes `v_flex` + N lane texts).

| Chapter | Verses | Single (N=1) | Compare (N=2) |
| --- | --- | ---: | ---: |
| Genesis 1 | 31 | 124 | 155 |
| Psalm 119 | 176 | 704 | 880 |

Every verse row is built up front; gpui-component `VirtualList` needs known row heights and verses wrap, so this baseline does not virtualize.
