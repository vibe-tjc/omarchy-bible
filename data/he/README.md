# Hebrew morphology (MorphHB / Open Scriptures)

Offline OT morphology for the annotation page. Source:
[openscriptures/morphhb](https://github.com/openscriptures/morphhb).

## Layout

One JSON per OT book, named by OSIS id (`Gen.json`, `Exod.json`, …):

```json
[
  [ /* chapter 1 */
    [ /* verse 1 */
      ["בְּרֵאשִׁית", "H7225", "HR/Ncfsa"],
      ["בָּרָא", "H1254", "HVqp3ms"]
    ]
  ]
]
```

Each word is `[text, lemma, morph]`. Versification is English/CUV-aligned
(`remapVerses`). NT books are omitted; `hebrew_verse` returns `None`.

## Import

```bash
python3 morphhbXML-to-JSON.py --prefixLemmasWithH --removeLemmaTypes --remapVerses --splitByBook
# copy remapped book JSON → data/he/ with OSIS filenames
```

`Gen.json` currently embeds **Genesis chapter 1** (31 verses) as the MorphHB skeleton. Add further chapters/books the same way.
