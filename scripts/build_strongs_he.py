#!/usr/bin/env python3
"""Build data/he/strongs-he.json.gz — compact Strong's HE gloss map (EN + ZH).

Sources / licenses
------------------
EN: openscriptures/strongs hebrew/strongs-hebrew-dictionary.js
    James Strong 1894 (PD) + Open Scriptures JSON packaging (CC BY-SA).
    Uses shortened `strongs_def` (fallback: cleaned `kjv_def`).

ZH: Lexicon Omnium Gentium (Zenodo 10.5281/zenodo.19099634) strongs_zh-Hant.tsv
    CC BY 4.0 — Traditional Chinese glosses keyed by Strong's number.
    Compact: first slash-segment of definition.

Output shape (gzip JSON object):
  {"H7225":{"en":"the first, in place, time, order or rank","zh":"開始"}, ...}

Usage:
  python3 scripts/build_strongs_he.py \\
    --en /path/to/strongs-hebrew-dictionary.js \\
    --zh-hant /path/to/strongs_zh-Hant.tsv \\
    --out data/he/strongs-he.json.gz
"""
from __future__ import annotations

import argparse
import csv
import gzip
import json
import re
import sys
from pathlib import Path


def gloss_en(entry: dict) -> str:
    sd = (entry.get("strongs_def") or "").strip().strip("{}").strip()
    sd = re.sub(r"\s+", " ", sd)
    sd = re.sub(r"^\([^)]*\)\s*", "", sd)
    sd = re.sub(r"\([^)]*\)", "", sd)
    sd = re.sub(r"\s+", " ", sd).strip(" ,;.")
    if ";" in sd:
        sd = sd.split(";", 1)[0].strip()
    if len(sd) > 52:
        cut = sd[:52]
        if " " in cut:
            cut = cut.rsplit(" ", 1)[0]
        sd = cut.rstrip(",;:.") + "…"
    if sd:
        return sd
    k = entry.get("kjv_def") or ""
    k = re.sub(
        r"\[(?:idiom|phrase|adverb|conjunction|particle|interjection)\]\s*",
        "",
        k,
        flags=re.I,
    )
    k = re.sub(r"\([^)]*\)", "", k)
    k = re.sub(r"\b[Xx]\b", "", k)
    parts = [p.strip(" .;+") for p in k.split(",") if p.strip(" .;+")]
    return parts[0][:48] if parts else ""


def load_en(path: Path) -> dict[str, str]:
    raw = path.read_text(encoding="utf-8")
    obj = json.loads(raw[raw.find("{") : raw.rfind("}") + 1])
    out: dict[str, str] = {}
    for k, v in obj.items():
        g = gloss_en(v)
        if g:
            out[k] = g
    return out


def load_zh_hant(path: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    with path.open(encoding="utf-8") as f:
        for row in csv.DictReader(f, delimiter="\t"):
            if row.get("testament") != "hebrew":
                continue
            m = re.match(r"^H0*(\d+)$", row["strong_num"])
            if not m:
                continue
            key = f"H{int(m.group(1))}"
            defn = (row.get("definition") or "").strip()
            if " / " in defn:
                defn = defn.split(" / ", 1)[0].strip()
            for sep in ["；", "。", ";"]:
                if sep in defn and len(defn) > 36:
                    defn = defn.split(sep)[0].strip()
                    break
            defn = re.sub(r"\s+", " ", defn)
            if len(defn) > 40:
                defn = defn[:37] + "…"
            if defn:
                out[key] = defn
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--en", type=Path, required=True, help="Open Scriptures strongs-hebrew-dictionary.js")
    ap.add_argument("--zh-hant", type=Path, required=True, help="LOG strongs_zh-Hant.tsv")
    ap.add_argument(
        "--out",
        type=Path,
        default=Path("data/he/strongs-he.json.gz"),
        help="Output gzip JSON path",
    )
    args = ap.parse_args()

    en = load_en(args.en)
    zh = load_zh_hant(args.zh_hant)
    combined = {k: {"en": v, **({"zh": zh[k]} if k in zh else {})} for k, v in en.items()}
    payload = json.dumps(combined, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(gzip.compress(payload, compresslevel=9))
    print(
        f"wrote {args.out} entries={len(combined)} zh_hit={sum(1 for v in combined.values() if 'zh' in v)} "
        f"bytes={args.out.stat().st_size}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
