# Hebrew morphology (MorphHB / Open Scriptures)

Offline OT morphology for the annotation page. Source:
[openscriptures/morphhb](https://github.com/openscriptures/morphhb).

## Runtime pack (committed)

`bible-core` embeds **`morphhb-ot.json.gz`** (~2.3 MiB) via `include_bytes!` + `flate2`.
That pack is the only MorphHB asset needed at build/run time (all 39 OT books,
English/CUV versification via `remapVerses`).

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

## Import / regenerate pack

```bash
python3 scripts/import_morphhb.py
# writes data/he/{Osis}.json then packs data/he/morphhb-ot.json.gz
```

NT books are omitted; `hebrew_verse` returns `None`.
