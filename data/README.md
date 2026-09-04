# 經文資料 / Scripture data

執行時以 `include_str!` 嵌入，不需網路。

- `cuv-kjv.json` — 66 卷對齊壓縮檔：和合本 1919 **神版**（`上帝`→`神`）+ King James Version
- `genesis-1.json` — 創世記第 1 章對齊快照（與上者一致，供 `load_genesis_1`）
- `zh/cuv/Gen.json` — 創世記 CUV 神版（單卷來源檔）
- `en/kjv/Gen.json` — 創世記 KJV

完整 JSON 來自 [midvash/bible-data](https://github.com/midvash/bible-data)（僅 curl 單檔，未 git clone）。匯入時將中文經文的「上帝」換成「神」。

未轉換的原始 dump 請放 `data/raw/`（已 gitignore）。

**未收錄** 新標點和合本（CUNP）；該譯本受香港聖經公會／聯合聖經公會著作權保護。


## Hebrew / MorphHB

- **Runtime:** `he/morphhb-ot.json.gz` — gzip pack of all 39 OT books, embedded by `bible-core` (~2.3 MiB).
- **Import artifacts:** `he/{Osis}.json` — per-book JSON from MorphHB; gitignored, regenerate with `scripts/import_morphhb.py`.
- Details: `he/README.md`.
