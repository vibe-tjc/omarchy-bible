# Hebrew morphology (MorphHB / Open Scriptures)

Offline OT morphology for the annotation page. Source:
[openscriptures/morphhb](https://github.com/openscriptures/morphhb)
(CC BY 4.0 morphology tagging; WLC text traditionally treated as PD).

## Runtime packs (committed)

`bible-core` embeds these via `include_bytes!` + `flate2`:

| File | Role |
|------|------|
| **`morphhb-ot.json.gz`** (~2.3 MiB) | 39 OT books, English/CUV versification (`remapVerses`) |
| **`strongs-he.json.gz`** (~168 KiB) | Compact Strong's gloss map keyed by `H####` |

That is all that is needed at build/run time. NT books are omitted;
`hebrew_verse` returns `None`.

### Strong's gloss pack (`strongs-he.json.gz`)

Shape: `{"H7225":{"en":"…","zh":"…"}, …}`

| Field | Use in UI | Source |
|-------|-----------|--------|
| `en` | 直譯 `gloss_literal` | [openscriptures/strongs](https://github.com/openscriptures/strongs) Hebrew dictionary — Strong 1894 text (PD), JSON packaging **CC BY-SA**; shortened `strongs_def` |
| `zh` | 意譯 `gloss_idiomatic` (fallback → `en`) | [Lexicon Omnium Gentium](https://doi.org/10.5281/zenodo.19099634) `strongs_zh-Hant.tsv` — **CC BY 4.0** (LLM-assisted Traditional Chinese) |

Regenerate:

```bash
python3 scripts/build_strongs_he.py \
  --en /path/to/strongs-hebrew-dictionary.js \
  --zh-hant /path/to/strongs_zh-Hant.tsv \
  --out data/he/strongs-he.json.gz
```

### Transliteration (音譯)

Filled at load time by a deterministic **SBL-ish** algorithm on the vocalized
MorphHB surface form (`transliterate_hebrew` in `hebrew.rs`) — not a stored
table. Cantillation marks are dropped.

## Per-book JSON (not committed)

`*.json` under this folder are **import artifacts** only (regenerated locally).
They are gitignored; do not commit them.

Shape of each book file:

```json
[
  [ /* chapter 1 */
    [ /* verse 1 */
      ["word", "H7225", "HR/Ncfsa"]
    ]
  ]
]
```

## Import / regenerate MorphHB pack

```bash
python3 scripts/import_morphhb.py
# writes data/he/{Osis}.json then packs data/he/morphhb-ot.json.gz
```
