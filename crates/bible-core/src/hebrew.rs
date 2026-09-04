//! MorphHB / Open Scriptures Hebrew morphology (OT).
//!
//! Runtime pack: `data/he/morphhb-ot.json.gz` — JSON object keyed by OSIS id,
//! each value chapters → verses → `[text, lemma, morph]`.
//! Embedded via `include_bytes!` + `flate2`. Versification is English/CUV-aligned
//! (`remapVerses` at import time). NT books are omitted; [`hebrew_verse`] returns `None`.
//!
//! Annotation columns (音譯 / 直譯 / 意譯) are filled at load time:
//! - **translit**: deterministic SBL-ish algorithm on the vocalized surface form
//! - **gloss_literal**: English Strong's gloss from `data/he/strongs-he.json.gz`
//! - **gloss_idiomatic**: Traditional Chinese Strong's when present, else EN gloss
//!
//! Strong's pack sources/licenses — see `data/he/README.md` and
//! `scripts/build_strongs_he.py`.

use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;

/// One MorphHB word in Hebrew reading order (first = rightmost in RTL display).
///
/// `translit` / `gloss_literal` / `gloss_idiomatic` are populated when possible;
/// UI shows 「—」 only when still `None` (e.g. missing Strong's key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HebrewWord {
    pub text: String,
    pub strongs: Option<String>,
    pub morph: Option<String>,
    pub translit: Option<String>,
    pub gloss_literal: Option<String>,
    pub gloss_idiomatic: Option<String>,
}

/// OT OSIS ids shipped in the MorphHB store (39 books).
pub const OT_OSIS: &[&str] = &[
    "Gen", "Exod", "Lev", "Num", "Deut", "Josh", "Judg", "Ruth", "1Sam", "2Sam",
    "1Kgs", "2Kgs", "1Chr", "2Chr", "Ezra", "Neh", "Esth", "Job", "Ps", "Prov",
    "Eccl", "Song", "Isa", "Jer", "Lam", "Ezek", "Dan", "Hos", "Joel", "Amos",
    "Obad", "Jonah", "Mic", "Nah", "Hab", "Zeph", "Hag", "Zech", "Mal",
];

/// Compact Strong's entry: EN gloss + optional ZH (Traditional).
#[derive(Debug, Clone)]
struct StrongsGloss {
    en: String,
    zh: Option<String>,
}

/// Committed MorphHB OT pack (JSON object keyed by OSIS).
const MORPHHB_OT_GZ: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/he/morphhb-ot.json.gz"
));

/// Compact Strong's HE map: `{ "H7225": { "en": "...", "zh": "..." }, ... }`.
///
/// EN: Open Scriptures Strong's Hebrew (CC BY-SA; Strong 1894 PD text).
/// ZH: Lexicon Omnium Gentium zh-Hant (CC BY 4.0), when available.
const STRONGS_HE_GZ: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/he/strongs-he.json.gz"
));

fn strongs_store() -> &'static HashMap<String, StrongsGloss> {
    static STORE: OnceLock<HashMap<String, StrongsGloss>> = OnceLock::new();
    STORE.get_or_init(|| {
        let mut decoder = GzDecoder::new(STRONGS_HE_GZ);
        let mut json = String::new();
        decoder
            .read_to_string(&mut json)
            .unwrap_or_else(|e| panic!("decompress strongs-he.json.gz: {e}"));
        let root: serde_json::Value = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("parse strongs-he.json.gz: {e}"));
        let obj = root
            .as_object()
            .unwrap_or_else(|| panic!("strongs-he.json.gz root must be an object"));
        let mut map = HashMap::with_capacity(obj.len());
        for (k, v) in obj {
            let en = v
                .get("en")
                .and_then(|x| x.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let Some(en) = en else { continue };
            let zh = v
                .get("zh")
                .and_then(|x| x.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            map.insert(
                k.clone(),
                StrongsGloss {
                    en: en.to_string(),
                    zh,
                },
            );
        }
        debug_assert!(map.contains_key("H7225"));
        map
    })
}

/// Normalize MorphHB lemma/Strong's field to a lookup key like `H7225`.
///
/// Accepts `H7225` / `H7225a`. Rejects MorphHB leftover type tags (`Hl`, `Hb`, …).
fn normalize_strongs_key(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.len() < 2 || !s.starts_with('H') {
        return None;
    }
    let mut chars = s[1..].chars();
    let first = chars.next()?;
    if !first.is_ascii_digit() {
        return None;
    }
    let mut digits = String::from(first);
    for c in chars.by_ref() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else if c.is_ascii_alphabetic() {
            // Optional trailing letter (rare augmented forms).
            return Some(format!("H{digits}{c}"));
        } else {
            break;
        }
    }
    Some(format!("H{digits}"))
}

fn glosses_for_strongs(raw: Option<&str>) -> (Option<String>, Option<String>) {
    let Some(raw) = raw else {
        return (None, None);
    };
    let Some(key) = normalize_strongs_key(raw) else {
        return (None, None);
    };
    let store = strongs_store();
    let entry = store.get(&key).or_else(|| {
        // H7225a → H7225
        if key.len() > 2 && key.chars().last().is_some_and(|c| c.is_ascii_alphabetic()) {
            let base: String = key.chars().take(key.len() - 1).collect();
            store.get(&base)
        } else {
            None
        }
    });
    let Some(entry) = entry else {
        return (None, None);
    };
    let literal = Some(entry.en.clone());
    let idiomatic = entry.zh.clone().or_else(|| Some(entry.en.clone()));
    (literal, idiomatic)
}

fn parse_word(parts: &[String]) -> Option<HebrewWord> {
    let text = parts.first()?.trim();
    if text.is_empty() {
        return None;
    }
    let strongs = parts
        .get(1)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let morph = parts
        .get(2)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let translit = {
        let t = transliterate_hebrew(text);
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    };
    let (gloss_literal, gloss_idiomatic) = glosses_for_strongs(strongs.as_deref());
    Some(HebrewWord {
        text: text.to_string(),
        strongs,
        morph,
        translit,
        gloss_literal,
        gloss_idiomatic,
    })
}

fn word_from_json(v: &serde_json::Value) -> Option<HebrewWord> {
    let arr = v.as_array()?;
    let parts: Vec<String> = arr
        .iter()
        .filter_map(|x| x.as_str().map(|s| s.to_string()))
        .collect();
    parse_word(&parts)
}

fn load_book_value(value: &serde_json::Value) -> Result<Vec<Vec<Vec<HebrewWord>>>, String> {
    let chapters = value
        .as_array()
        .ok_or_else(|| "hebrew book JSON must be an array of chapters".to_string())?;
    let mut out = Vec::with_capacity(chapters.len());
    for ch in chapters {
        let verses = ch
            .as_array()
            .ok_or_else(|| "chapter must be an array of verses".to_string())?;
        let mut verse_list = Vec::with_capacity(verses.len());
        for verse in verses {
            let words_raw = verse
                .as_array()
                .ok_or_else(|| "verse must be an array of words".to_string())?;
            let words: Vec<HebrewWord> = words_raw.iter().filter_map(word_from_json).collect();
            verse_list.push(words);
        }
        out.push(verse_list);
    }
    Ok(out)
}

struct HebrewStore {
    /// OSIS id → chapters[0]=ch1 → verses[0]=v1 → words
    books: HashMap<String, Vec<Vec<Vec<HebrewWord>>>>,
}

fn store() -> &'static HebrewStore {
    static STORE: OnceLock<HebrewStore> = OnceLock::new();
    STORE.get_or_init(|| {
        // Ensure Strong's map is warm before hydrating words (parse_word looks it up).
        let _ = strongs_store();
        let mut decoder = GzDecoder::new(MORPHHB_OT_GZ);
        let mut json = String::new();
        decoder
            .read_to_string(&mut json)
            .unwrap_or_else(|e| panic!("decompress morphhb-ot.json.gz: {e}"));
        let root: serde_json::Value = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("parse morphhb-ot.json.gz: {e}"));
        let obj = root
            .as_object()
            .unwrap_or_else(|| panic!("morphhb-ot.json.gz root must be an object keyed by OSIS"));
        let mut books = HashMap::with_capacity(OT_OSIS.len());
        for &osis in OT_OSIS {
            let raw = obj
                .get(osis)
                .unwrap_or_else(|| panic!("morphhb-ot.json.gz missing book {osis}"));
            let book = load_book_value(raw).unwrap_or_else(|e| {
                panic!("hebrew book {osis}: {e}");
            });
            books.insert(osis.to_string(), book);
        }
        debug_assert_eq!(books.len(), 39);
        HebrewStore { books }
    })
}

/// Look up MorphHB words for a verse (1-based chapter/verse).
///
/// Returns `None` for NT, missing books, out-of-range, or empty verses.
pub fn hebrew_verse(osis: &str, chapter: u32, verse: u32) -> Option<Vec<HebrewWord>> {
    if chapter == 0 || verse == 0 {
        return None;
    }
    let book = store().books.get(osis)?;
    let ch = book.get((chapter as usize).checked_sub(1)?)?;
    let words = ch.get((verse as usize).checked_sub(1)?)?;
    if words.is_empty() {
        None
    } else {
        Some(words.clone())
    }
}

/// UI rule: show † only when [`hebrew_verse`] would return `Some`.
pub fn has_hebrew_notes(osis: &str, chapter: u32, verse: u32) -> bool {
    hebrew_verse(osis, chapter, verse).is_some()
}

/// Number of OT books loaded in the MorphHB store.
pub fn hebrew_book_count() -> usize {
    store().books.len()
}

/// Map MorphHB part-of-speech letter to a short Traditional Chinese label.
fn pos_letter_label(pos: char) -> Option<&'static str> {
    match pos {
        'A' => Some("形容詞"),
        'C' => Some("連接詞"),
        'D' => Some("副詞"),
        'N' => Some("名詞"),
        'P' => Some("代名詞"),
        'R' => Some("介詞"),
        'S' => Some("詞綴"),
        'T' => Some("助詞"),
        'V' => Some("動詞"),
        _ => None,
    }
}

/// Derive a short 詞性 label from a MorphHB morph code.
///
/// Codes look like `HNcmpa`, `HVqp3ms`, or compounds `HR/Ncfsa` (lang `H`/`A`
/// + `/`-separated segments). We take the **last non-suffix** segment's POS
/// letter (suffixes start with `S`) so `HNcmpa` → 名詞 and `HR/Ncfsa` → 名詞.
/// Returns `None` when morph is empty or no known POS letter is found.
/// Does not invent Strong's glosses — only maps the morph category letter.
pub fn morph_pos_label(morph: &str) -> Option<&'static str> {
    let morph = morph.trim();
    if morph.is_empty() {
        return None;
    }
    // One language code prefixes the whole string (H = Hebrew, A = Aramaic).
    let body = if morph.starts_with('H') || morph.starts_with('A') {
        &morph[1..]
    } else {
        morph
    };
    let mut last_content: Option<char> = None;
    let mut last_suffix: Option<char> = None;
    for seg in body.split('/') {
        let Some(pos) = seg.chars().next() else {
            continue;
        };
        if pos == 'S' {
            last_suffix = Some(pos);
            continue;
        }
        if pos_letter_label(pos).is_some() {
            last_content = Some(pos);
        }
    }
    last_content
        .or(last_suffix)
        .and_then(pos_letter_label)
}

// ─── SBL-ish transliteration ───────────────────────────────────────────────

fn is_hebrew_letter(c: char) -> bool {
    matches!(c, '\u{05D0}'..='\u{05EA}')
}

fn is_niqqud(c: char) -> bool {
    matches!(
        c,
        '\u{05B0}'..='\u{05BC}' // sheva .. dagesh
            | '\u{05C1}' // shin dot
            | '\u{05C2}' // sin dot
            | '\u{05C4}'
            | '\u{05C5}'
            | '\u{05C7}' // qamats qatan
    )
}

fn is_skipped_mark(c: char) -> bool {
    // Cantillation, meteg, rafe, sof pasuq, nun hafukha, etc.
    matches!(
        c,
        '\u{0591}'..='\u{05AF}'
            | '\u{05BD}' // meteg
            | '\u{05BF}' // rafe
            | '\u{05C0}' // paseq
            | '\u{05C3}' // sof pasuq
            | '\u{05C6}'
            | '\u{05F3}'
            | '\u{05F4}'
    )
}

/// Deterministic SBL-ish academic transliteration of vocalized Hebrew Unicode.
///
/// Consonants follow SBL academic (ʾ ʿ ḥ ṭ ṣ š ś q …). Begadkefat without dagesh
/// softens to v/g̱/ḏ/ḵ/f/ṯ-style Latin (`v g d kh f t` with ḵ for soft kaf).
/// Vowels: a ā e ē i ī o ō u ū ə ă ĕ ŏ. Cantillation and sof pasuq are dropped.
/// Not a full phonological analysis (silent shewa / qamats qatan heuristics are
/// simplified); output is stable for the same input.
pub fn transliterate_hebrew(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(chars.len() * 2);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == ' ' || c == '\u{00A0}' {
            out.push(' ');
            i += 1;
            continue;
        }
        if c == '\u{05BE}' {
            // maqaf
            out.push('-');
            i += 1;
            continue;
        }
        if is_skipped_mark(c) {
            i += 1;
            continue;
        }
        if is_hebrew_letter(c) {
            let mut j = i + 1;
            let mut dagesh = false;
            let mut shin = false;
            let mut sin = false;
            let mut vowels: Vec<char> = Vec::new();
            while j < chars.len() {
                let m = chars[j];
                if is_skipped_mark(m) {
                    j += 1;
                    continue;
                }
                if !is_niqqud(m) {
                    break;
                }
                match m {
                    '\u{05BC}' => dagesh = true, // dagesh / mapiq
                    '\u{05C1}' => shin = true,
                    '\u{05C2}' => sin = true,
                    v => vowels.push(v),
                }
                j += 1;
            }
            // Shuruq: vav+dagesh alone -> û (not "ww").
            if c == '\u{05D5}' && dagesh && vowels.is_empty() {
                out.push_str("û");
                i = j;
                continue;
            }
            // Holam male: vav+holam -> ō (mater lectionis; do not also emit w).
            if c == '\u{05D5}'
                && vowels.len() == 1
                && (vowels[0] == '\u{05B9}' || vowels[0] == '\u{05BA}')
                && !dagesh
            {
                out.push_str("ō");
                i = j;
                continue;
            }
            out.push_str(consonant_xlit(c, dagesh, shin, sin));
            for v in vowels {
                if let Some(vs) = vowel_xlit(v) {
                    out.push_str(vs);
                }
            }
            i = j;
            continue;
        }
        if is_niqqud(c) {
            // Orphan vowel (rare) — emit if meaningful.
            if let Some(vs) = vowel_xlit(c) {
                out.push_str(vs);
            }
            i += 1;
            continue;
        }
        // Pass through ASCII punctuation occasionally present; skip other junk.
        if c.is_ascii_punctuation() {
            out.push(c);
        }
        i += 1;
    }
    out
}

fn consonant_xlit(c: char, dagesh: bool, shin: bool, sin: bool) -> &'static str {
    match c {
        'א' => "ʾ",
        'ב' => {
            if dagesh {
                "b"
            } else {
                "v"
            }
        }
        'ג' => "g",
        'ד' => "d",
        'ה' => "h",
        'ו' => {
            if dagesh {
                "ww"
            } else {
                "w"
            }
        }
        'ז' => "z",
        'ח' => "ḥ",
        'ט' => "ṭ",
        'י' => {
            if dagesh {
                "yy"
            } else {
                "y"
            }
        }
        'כ' | 'ך' => {
            if dagesh {
                "k"
            } else {
                "ḵ"
            }
        }
        'ל' => "l",
        'מ' | 'ם' => "m",
        'נ' | 'ן' => "n",
        'ס' => "s",
        'ע' => "ʿ",
        'פ' | 'ף' => {
            if dagesh {
                "p"
            } else {
                "f"
            }
        }
        'צ' | 'ץ' => "ṣ",
        'ק' => "q",
        'ר' => "r",
        'ש' => {
            if sin {
                "ś"
            } else if shin {
                "š"
            } else {
                "š" // default shin
            }
        }
        'ת' => {
            if dagesh {
                "t"
            } else {
                "t" // soft taw often ṯ; keep t for readability
            }
        }
        _ => "",
    }
}

fn vowel_xlit(c: char) -> Option<&'static str> {
    Some(match c {
        '\u{05B0}' => "ə", // sheva
        '\u{05B1}' => "ĕ", // hataf segol
        '\u{05B2}' => "ă", // hataf patah
        '\u{05B3}' => "ŏ", // hataf qamats
        '\u{05B4}' => "i", // hiriq
        '\u{05B5}' => "ē", // tsere
        '\u{05B6}' => "e", // segol
        '\u{05B7}' => "a", // patah
        '\u{05B8}' => "ā", // qamats
        '\u{05C7}' => "o", // qamats qatan
        '\u{05B9}' => "ō", // holam
        '\u{05BA}' => "ō", // holam haser for vav
        '\u{05BB}' => "u", // qubuts
        '\u{05BC}' | '\u{05C1}' | '\u{05C2}' | '\u{05C4}' | '\u{05C5}' => return None,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_1_1_has_words_and_h7225() {
        let words = hebrew_verse("Gen", 1, 1).expect("Gen 1:1");
        assert!(!words.is_empty());
        let strongs = words[0].strongs.as_deref().unwrap_or("");
        assert!(
            strongs.contains("7225"),
            "expected Strong's containing 7225, got {strongs:?}"
        );
        assert_eq!(words[0].text.chars().next(), Some('ב'));
        assert!(has_hebrew_notes("Gen", 1, 1));

        let translit = words[0].translit.as_deref().expect("translit");
        assert!(!translit.is_empty(), "translit empty");
        // Vocalized בְּרֵאשִׁית → starts with bə / be and contains š
        assert!(
            translit.starts_with('b'),
            "expected b… translit, got {translit}"
        );
        assert!(
            translit.contains('š') || translit.contains('s'),
            "expected shin in translit, got {translit}"
        );

        let lit = words[0].gloss_literal.as_deref().expect("gloss_literal");
        assert!(!lit.is_empty());
        assert!(
            lit.to_lowercase().contains("first") || lit.to_lowercase().contains("begin"),
            "unexpected EN gloss for H7225: {lit}"
        );

        let idi = words[0]
            .gloss_idiomatic
            .as_deref()
            .expect("gloss_idiomatic");
        assert!(!idi.is_empty());
        // Prefer ZH when available (開始); else EN fallback must still be non-empty.
        assert!(
            idi.contains('開') || idi.to_lowercase().contains("first") || idi.to_lowercase().contains("begin"),
            "unexpected idiomatic gloss: {idi}"
        );
    }

    #[test]
    fn gen_1_1_all_words_have_translit_and_most_glosses() {
        let words = hebrew_verse("Gen", 1, 1).expect("Gen 1:1");
        for w in &words {
            assert!(
                w.translit.as_ref().is_some_and(|t| !t.is_empty()),
                "missing translit for {:?}",
                w.text
            );
        }
        // First three content words H7225 / H1254 / H430 should have glosses.
        for (i, key) in [(0, "H7225"), (1, "H1254"), (2, "H430")] {
            assert_eq!(words[i].strongs.as_deref(), Some(key));
            assert!(words[i].gloss_literal.is_some(), "literal missing {key}");
            assert!(words[i].gloss_idiomatic.is_some(), "idiomatic missing {key}");
        }
    }

    #[test]
    fn missing_strongs_still_ok() {
        // Unknown key → no glosses; translit still works on Hebrew text.
        let (lit, idi) = glosses_for_strongs(Some("H99999"));
        assert!(lit.is_none());
        assert!(idi.is_none());
        let (lit2, idi2) = glosses_for_strongs(Some("Hl"));
        assert!(lit2.is_none());
        assert!(idi2.is_none());
        let t = transliterate_hebrew("אָב");
        assert!(t.contains('ʾ') || t.starts_with('a') || t.contains('ā') || t.contains('b') || t.contains('v'), "got {t}");
        assert!(!t.is_empty());
    }

    #[test]
    fn transliterate_bereshit_shape() {
        // בְּרֵאשִׁית without cantillation
        let t = transliterate_hebrew("בְּרֵאשִׁית");
        assert!(t.starts_with("bə"), "got {t}");
        assert!(t.contains('š'), "got {t}");
        assert!(t.contains('ʾ'), "got {t}");
    }

    #[test]
    fn hebrew_book_count_is_39() {
        assert_eq!(hebrew_book_count(), 39);
        assert_eq!(hebrew_book_count(), OT_OSIS.len());
    }

    #[test]
    fn ps_23_1_and_exod_1_1_are_some() {
        let exod = hebrew_verse("Exod", 1, 1).expect("Exod 1:1");
        assert!(!exod.is_empty());
        assert!(exod[0].translit.is_some());
        let ps = hebrew_verse("Ps", 23, 1).expect("Ps 23:1");
        assert!(!ps.is_empty());
    }

    #[test]
    fn john_3_16_is_none() {
        assert!(hebrew_verse("John", 3, 16).is_none());
        assert!(!has_hebrew_notes("John", 3, 16));
    }

    #[test]
    fn morph_pos_label_from_prefix() {
        assert_eq!(morph_pos_label("HNcmpa"), Some("名詞"));
        assert_eq!(morph_pos_label("HVqp3ms"), Some("動詞"));
        assert_eq!(morph_pos_label("HAamsa"), Some("形容詞"));
        assert_eq!(morph_pos_label("HR/Ncfsa"), Some("名詞"));
        assert_eq!(morph_pos_label("HC/To"), Some("助詞"));
        assert_eq!(morph_pos_label("HTd/Ncmpa"), Some("名詞"));
        assert_eq!(morph_pos_label("HR/Ncmsc/Sp3ms"), Some("名詞"));
        assert_eq!(morph_pos_label("HC"), Some("連接詞"));
        assert_eq!(morph_pos_label(""), None);
        assert_eq!(morph_pos_label("   "), None);
    }

    #[test]
    fn strongs_normalize_and_lookup_h7225() {
        assert_eq!(normalize_strongs_key("H7225").as_deref(), Some("H7225"));
        assert_eq!(normalize_strongs_key("Hl"), None);
        let (lit, idi) = glosses_for_strongs(Some("H7225"));
        assert!(lit.is_some());
        assert!(idi.is_some());
    }
}
