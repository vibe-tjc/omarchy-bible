//! MorphHB / Open Scriptures Hebrew morphology (OT).
//!
//! Source files: `data/he/{Osis}.json` — chapters → verses → `[text, lemma, morph]`.
//! All 39 OT books are embedded via `include_str!`. Versification is English/CUV-aligned
//! (`remapVerses` at import time). NT books are omitted; [`hebrew_verse`] returns `None`.

use std::collections::HashMap;
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

fn load_book_json(raw: &str) -> Result<Vec<Vec<Vec<HebrewWord>>>, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("invalid JSON: {e}"))?;
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

macro_rules! he_include {
    ($file:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/he/", $file))
    };
}

fn insert_book(books: &mut HashMap<String, Vec<Vec<Vec<HebrewWord>>>>, osis: &str, raw: &str) {
    let book = load_book_json(raw).unwrap_or_else(|e| {
        panic!("hebrew book {osis}: {e}");
    });
    books.insert(osis.to_string(), book);
}

fn store() -> &'static HebrewStore {
    static STORE: OnceLock<HebrewStore> = OnceLock::new();
    STORE.get_or_init(|| {
        let mut books = HashMap::with_capacity(OT_OSIS.len());
        insert_book(&mut books, "Gen", he_include!("Gen.json"));
        insert_book(&mut books, "Exod", he_include!("Exod.json"));
        insert_book(&mut books, "Lev", he_include!("Lev.json"));
        insert_book(&mut books, "Num", he_include!("Num.json"));
        insert_book(&mut books, "Deut", he_include!("Deut.json"));
        insert_book(&mut books, "Josh", he_include!("Josh.json"));
        insert_book(&mut books, "Judg", he_include!("Judg.json"));
        insert_book(&mut books, "Ruth", he_include!("Ruth.json"));
        insert_book(&mut books, "1Sam", he_include!("1Sam.json"));
        insert_book(&mut books, "2Sam", he_include!("2Sam.json"));
        insert_book(&mut books, "1Kgs", he_include!("1Kgs.json"));
        insert_book(&mut books, "2Kgs", he_include!("2Kgs.json"));
        insert_book(&mut books, "1Chr", he_include!("1Chr.json"));
        insert_book(&mut books, "2Chr", he_include!("2Chr.json"));
        insert_book(&mut books, "Ezra", he_include!("Ezra.json"));
        insert_book(&mut books, "Neh", he_include!("Neh.json"));
        insert_book(&mut books, "Esth", he_include!("Esth.json"));
        insert_book(&mut books, "Job", he_include!("Job.json"));
        insert_book(&mut books, "Ps", he_include!("Ps.json"));
        insert_book(&mut books, "Prov", he_include!("Prov.json"));
        insert_book(&mut books, "Eccl", he_include!("Eccl.json"));
        insert_book(&mut books, "Song", he_include!("Song.json"));
        insert_book(&mut books, "Isa", he_include!("Isa.json"));
        insert_book(&mut books, "Jer", he_include!("Jer.json"));
        insert_book(&mut books, "Lam", he_include!("Lam.json"));
        insert_book(&mut books, "Ezek", he_include!("Ezek.json"));
        insert_book(&mut books, "Dan", he_include!("Dan.json"));
        insert_book(&mut books, "Hos", he_include!("Hos.json"));
        insert_book(&mut books, "Joel", he_include!("Joel.json"));
        insert_book(&mut books, "Amos", he_include!("Amos.json"));
        insert_book(&mut books, "Obad", he_include!("Obad.json"));
        insert_book(&mut books, "Jonah", he_include!("Jonah.json"));
        insert_book(&mut books, "Mic", he_include!("Mic.json"));
        insert_book(&mut books, "Nah", he_include!("Nah.json"));
        insert_book(&mut books, "Hab", he_include!("Hab.json"));
        insert_book(&mut books, "Zeph", he_include!("Zeph.json"));
        insert_book(&mut books, "Hag", he_include!("Hag.json"));
        insert_book(&mut books, "Zech", he_include!("Zech.json"));
        insert_book(&mut books, "Mal", he_include!("Mal.json"));
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
}
