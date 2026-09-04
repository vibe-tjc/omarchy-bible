//! MorphHB / Open Scriptures Hebrew morphology (OT).
//!
//! Runtime pack: `data/he/morphhb-ot.json.gz` — JSON object keyed by OSIS id,
//! each value chapters → verses → `[text, lemma, morph]`.
//! Embedded via `include_bytes!` + `flate2`. Versification is English/CUV-aligned
//! (`remapVerses` at import time). NT books are omitted; [`hebrew_verse`] returns `None`.

use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;

/// One MorphHB word in Hebrew reading order (first = rightmost in RTL display).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HebrewWord {
    pub text: String,
    pub strongs: Option<String>,
    pub morph: Option<String>,
}

/// OT OSIS ids shipped in the MorphHB store (39 books).
pub const OT_OSIS: &[&str] = &[
    "Gen", "Exod", "Lev", "Num", "Deut", "Josh", "Judg", "Ruth", "1Sam", "2Sam",
    "1Kgs", "2Kgs", "1Chr", "2Chr", "Ezra", "Neh", "Esth", "Job", "Ps", "Prov",
    "Eccl", "Song", "Isa", "Jer", "Lam", "Ezek", "Dan", "Hos", "Joel", "Amos",
    "Obad", "Jonah", "Mic", "Nah", "Hab", "Zeph", "Hag", "Zech", "Mal",
];

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
    Some(HebrewWord {
        text: text.to_string(),
        strongs,
        morph,
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

/// Committed MorphHB OT pack (JSON object keyed by OSIS).
const MORPHHB_OT_GZ: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/he/morphhb-ot.json.gz"
));

fn store() -> &'static HebrewStore {
    static STORE: OnceLock<HebrewStore> = OnceLock::new();
    STORE.get_or_init(|| {
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
}
