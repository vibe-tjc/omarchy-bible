# omarchy-bible performance

Measured on this Mac, **2026-09-02 09:09 CST (UTC+8)**.

## Environment

| | |
| --- | --- |
| Hardware | Mac14,5 · Apple M2 Max |
| Arch | arm64 (`uname -m`) |
| rustc | 1.97.1 (8bab26f4f 2026-07-14) |
| Profile | `release` |
| Method | median of 7 runs, `cargo run --release -p bible-core --example bench` |
| Before JSON | [`docs/perf-baseline.json`](perf-baseline.json) |
| After JSON | [`docs/perf-after.json`](perf-after.json) |
| Human baseline draft | [`docs/perf-baseline.md`](perf-baseline.md) |

No SQLite. Still one `load_bible()` at process start; chapter switches do not re-parse JSON.

## Timing

Times from the bench (median). `load_chapter_at` after is an `Arc` clone (209 ns printed as 0.000 ms at 3 decimals).

| Metric | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `load_bible()` (serde + build store) | 8.275 ms | 10.642 ms | **+2.367 ms** (+28.6%). Extra work: lowercase every English verse once at load. |
| `load_chapter_at` Genesis 1 (31 verses) | 6.792 µs (6792 ns) | 0.209 µs (209 ns) | **−6.583 µs** (−96.9%) |
| `load_chapter_at` Psalm 119 (176 verses) | 26.041 µs (26041 ns) | 0.208 µs (208 ns) | **−25.833 µs** (−99.2%) |
| search `神愛世人` (1 hit) | 6.896 ms | 1.406 ms | **−5.490 ms** (−79.6%) |
| search `起初` (37 hits) | 5.548 ms | 1.320 ms | **−4.228 ms** (−76.2%) |
| search `God so loved` (2 hits) | 4.638 ms | 1.271 ms | **−3.367 ms** (−72.6%) |

Search still scans all 31,021 verses in memory. CJK is substring on the original string; English uses a precomputed lowercase copy (query is lowercased once per search, not per verse).

## Space

| Item | Before | After | Delta |
| --- | ---: | ---: | ---: |
| JSON file `data/cuv-kjv.json` | 8,071,265 B (7.70 MiB) | 8,071,265 B | unchanged |
| Books / chapters / verses | 66 / 1,189 / 31,021 | same | — |
| Longest chapter | 詩篇 119, 176 verses | same | — |
| Stored string bytes (zh+en, + English fold after) | 7,282,095 | 11,378,601 | **+4,096,506** (~KJV lowercase index) |
| Stored string count | 62,042 | 93,063 | **+31,021** (one fold per verse) |
| Approx in-memory payload (strings + Vec/String headers) | 12,025,823 B (11.47 MiB) | 17,859,505 B (17.03 MiB) | **+5,833,682 B** |
| Process RSS after `load_bible` (`ps -o rss=`) | 35,104 KB (34.3 MiB) | 53,008 KB (51.8 MiB) | **+17,904 KB** |
| `target/release/omarchy-bible` | 24,837,744 B (23.69 MiB) | 24,867,664 B (23.72 MiB) | **+29,920 B** |
| `target/debug/omarchy-bible` | 84,507,264 B (80.59 MiB) | 84,711,072 B (80.79 MiB) | **+203,808 B** (debug rebuilt after changes) |

RSS is larger than the payload estimate: allocator headers, book/chapter metadata, unused capacity, and the extra fold strings. After-run `text_bytes` includes the lowercase English index (`Verse::stored_string_bytes`); the baseline count was display strings only because the fold did not exist yet.

## UI nodes (chapter verse pane)

**Not virtualized** (same before and after for the default reading pane).

Current `verse_block` (reading): **3 + N lanes** = root `h_flex` + verse-number `div` + lanes `v_flex` + N lane texts.

| Chapter | Verses | Single (N=1) | Compare (N=2) |
| --- | ---: | ---: | ---: |
| Genesis 1 | 31 | 124 | 155 |
| Psalm 119 (longest) | 176 | 704 | 880 |

gpui-component 0.5.1 **does** ship `VirtualList` / `v_virtual_list`, and gpui 0.2.2 has `uniform_list`. Neither is a clean fit here: `uniform_list` needs equal row height, and `VirtualList` requires known per-row heights. Verse height varies with wrapping and lane count. Skipping virtualization rather than a risky rewrite.

Select mode (new) adds **one checkbox `div` per verse** while the mode is on (4 + N). Default reading node counts stay 3 + N.

| Chapter | Select + single | Select + compare |
| --- | ---: | ---: |
| Genesis 1 | 155 | 186 |
| Psalm 119 | 880 | 1,056 |

## What changed

- Chapters stored as `Arc<[Verse]>` so `load_chapter_at` is a pointer clone, not a deep `Vec<Verse>` clone.
- English search haystack lowercased once at load; CJK still substring on the original.
- `load_bible` still once at startup; JSON is not re-parsed on chapter change.

## What did not change

- No SQLite / on-disk index.
- No GPUI virtualization of the verse list (see above).
- No system share sheet (select-mode v1 is copy only).
- Canon, 神版 CUV + KJV payload, and search hit semantics (tests unchanged in behavior).

## Select / copy mode (v1)

- Header chip **選擇** toggles select mode. Toggle off or **Esc** leaves the mode and clears the selection. `[` / `]` still change chapter (chapter nav is not stolen). Changing book/chapter clears the selection.
- In select mode, each verse shows a checkbox; tap toggles. If exactly one verse is selected and another is tapped, the inclusive range is selected (e.g. John 3:16 then 18 → 16–18). A third tap starts a new single selection.
- Bottom bar: `已選 N 節 · 約翰福音 3:16–18`, buttons **複製** and **取消** (取消 exits the mode).
- Copy uses `gpui::ClipboardItem::new_string` + `cx.write_to_clipboard` (gpui 0.2.2). No `pbcopy` fallback was needed on this Mac.
- Format (current view-mode lanes):

Single:

```
約翰福音 3:16–18（和合本 1919 神版）
16 …
17 …
```

Compare: translation labels in the header; per verse, number then stacked lanes.

Implemented in `bible_core::format_verse_copy` / `format_ref_zh`.
