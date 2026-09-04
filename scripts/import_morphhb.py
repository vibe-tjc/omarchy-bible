#!/usr/bin/env python3
"""Import MorphHB remapped split-by-book JSON into data/he/{Osis}.json + pack.

Expected input: output of openscriptures/morphhb morphhbXML-to-JSON.py with
  --prefixLemmasWithH --removeLemmaTypes --remapVerses --splitByBook
i.e. json/remapped/{genesis,exodus,…}.json
"""
from __future__ import annotations

import argparse
import gzip
import json
import re
import sys
from pathlib import Path

BOOK_NAME = {
    "Genesis": "Gen",
    "Exodus": "Exod",
    "Leviticus": "Lev",
    "Numbers": "Num",
    "Deuteronomy": "Deut",
    "Joshua": "Josh",
    "Judges": "Judg",
    "Ruth": "Ruth",
    "I Samuel": "1Sam",
    "II Samuel": "2Sam",
    "I Kings": "1Kgs",
    "II Kings": "2Kgs",
    "I Chronicles": "1Chr",
    "II Chronicles": "2Chr",
    "Ezra": "Ezra",
    "Nehemiah": "Neh",
    "Esther": "Esth",
    "Job": "Job",
    "Psalms": "Ps",
    "Proverbs": "Prov",
    "Ecclesiastes": "Eccl",
    "Song of Solomon": "Song",
    "Isaiah": "Isa",
    "Jeremiah": "Jer",
    "Lamentations": "Lam",
    "Ezekiel": "Ezek",
    "Daniel": "Dan",
    "Hosea": "Hos",
    "Joel": "Joel",
    "Amos": "Amos",
    "Obadiah": "Obad",
    "Jonah": "Jonah",
    "Micah": "Mic",
    "Nahum": "Nah",
    "Habakkuk": "Hab",
    "Zephaniah": "Zeph",
    "Haggai": "Hag",
    "Zechariah": "Zech",
    "Malachi": "Mal",
}


def file_key(en_name: str) -> str:
    return en_name.replace(" ", "").lower()


def normalize_lemma(lemma: str | None) -> str | None:
    if not lemma:
        return lemma
    if "/" not in lemma:
        return lemma
    parts = lemma.split("/")
    for p in reversed(parts):
        if re.fullmatch(r"H\d+[a-zA-Z]?", p):
            return p
    return parts[-1]


def normalize_word(w: list) -> list:
    text = (w[0] or "").replace("/", "")
    lemma = normalize_lemma(w[1] if len(w) > 1 else None) or ""
    morph = (w[2] if len(w) > 2 else None) or ""
    return [text, lemma, morph]


def normalize_book(book: list) -> list:
    return [[[normalize_word(w) for w in verse] for verse in chapter] for chapter in book]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--src",
        type=Path,
        required=True,
        help="MorphHB json/remapped directory",
    )
    ap.add_argument(
        "--dst",
        type=Path,
        default=Path("data/he"),
        help="Destination data/he directory",
    )
    args = ap.parse_args()
    args.dst.mkdir(parents=True, exist_ok=True)

    packed: dict[str, list] = {}
    for en, osis in BOOK_NAME.items():
        src = args.src / f"{file_key(en)}.json"
        if not src.exists():
            print(f"missing {src}", file=sys.stderr)
            return 1
        book = normalize_book(json.loads(src.read_text(encoding="utf-8")))
        out = args.dst / f"{osis}.json"
        out.write_text(
            json.dumps(book, ensure_ascii=False, separators=(",", ":")),
            encoding="utf-8",
        )
        packed[osis] = book
        print(f"wrote {out.name} ({out.stat().st_size} bytes, {len(book)} ch)")

    pack_path = args.dst / "morphhb-ot.json.gz"
    blob = json.dumps(packed, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    with gzip.open(pack_path, "wb", compresslevel=9) as f:
        f.write(blob)
    print(f"pack {pack_path.name} = {pack_path.stat().st_size} bytes ({len(packed)} books)")
    # sanity
    g11 = packed["Gen"][0][0][0]
    assert g11[1] == "H7225", g11
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
