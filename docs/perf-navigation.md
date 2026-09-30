# Navigation performance review

## Findings

The timings in `perf.md` predate the Hebrew annotation feature. They no longer
represent the current startup or chapter-load path.

- The first OT chapter used to decompress/parse the whole MorphHB pack and hydrate
  all 39 books, including transliteration and Strong's glosses, on the UI thread.
- Every chapter load deep-cloned bilingual verses before checking for Hebrew;
  OT chapter revisits also cloned all annotation strings again.
- The reading pane created/layouted every verse on every render, including verses
  far outside the viewport. Psalm 119 has 176 variable-height rows.
- Search has a debounce, but `run_search` still executes on the UI thread. Its
  `cx.spawn` only delays the operation; it does not offload CPU work.
- `bible-ui/src/lib.rs` mixes navigation, search, selection, settings, pickers,
  Hebrew details, and rendering in one large view.

## Changes

- Parse MorphHB directly into typed raw chapter data instead of `serde_json::Value`.
  Hydrate transliteration/glosses only for chapters actually accessed, using
  per-chapter `OnceLock` caches. The compressed pack is still parsed as a whole.
- Cache annotated OT chapter `Arc<[Verse]>` values. NT navigation skips annotation
  processing entirely. Marker checks no longer clone word arrays.
- Use GPUI's variable-height `list` / `ListState` for the reading pane. Unlike
  `uniform_list`, this supports wrapped bilingual rows without precomputed heights.
  Only visible rows plus 300 px of overdraw are laid out.
- Invalidate row measurements when font size, lanes, selection, or highlighting
  changes, preserving the logical reading position. GPUI invalidates measurements
  on width changes. Chapter changes reset the list; verse jumps use `ListOffset`.

## Local data-path measurements

Run in a fresh process:

```sh
cargo run --release -p bible-core --example navigation_bench
```

Single-run release measurements on this Linux machine; not UI frame timings:

| Operation | Before | After |
| --- | ---: | ---: |
| Bilingual store load | 11.964 ms | 13.069 ms |
| First Genesis 1, including cold Hebrew initialization | 283.996 ms | 93.666 ms |
| First Genesis 2, Hebrew pack already initialized | 0.061 ms | 0.188 ms |
| First Psalm 119, Hebrew pack already initialized | 0.229 ms | 0.638 ms |
| Warm Genesis 1 revisit | 59.164 µs | 0.047 µs |
| Warm Psalm 119 revisit | 184.313 µs | 0.069 µs |

First visits after initialization do a little more work because annotation hydration
is deferred from startup. Warm navigation no longer repeatedly clones annotation
strings. Caches retain raw Hebrew data and visited annotated chapters; memory/RSS
has not been benchmarked. These measurements do not establish UI FPS or latency.

## Next priorities

1. Measure real UI frame times in a release build. Manually check long chapters,
   resizing, font/lane changes, selection, Hebrew-page return, verse jumps, and
   Previous/Next across book boundaries. Compilation/unit tests do not substitute
   for these interaction checks.
2. Split MorphHB assets per book/chapter, or load them on a background executor with
   explicit loading/error state. The remaining ~94 ms cold parse still blocks first
   chapter initialization; laziness alone does not remove it.
3. Move search to a background executor using an immutable shared Bible store and
   sequence IDs to discard stale results. Limit snippet generation to displayed
   hits while retaining an accurate total count.
4. Separate the reading pane and overlays into GPUI entities/modules so editing a
   search field or picker does not rebuild unrelated reading UI. Keep navigation
   and selection state explicit rather than adding more flags to `BibleView`.
5. Consider shared annotation slices / a separate annotation store or bounded
   chapter caches if measured memory growth justifies it. Avoid a database rewrite
   until profiling shows that storage/search, rather than rendering, is the limit.

Use `cargo run --release -p omarchy-bible` when assessing responsiveness; debug
builds are not representative of shipping performance.
