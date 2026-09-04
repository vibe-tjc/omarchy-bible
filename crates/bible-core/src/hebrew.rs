//! MorphHB / Open Scriptures Hebrew morphology (OT).
//!
//! Layout: `data/he/{Osis}.json` — chapters → verses → `[text, lemma, morph]`.
//! Versification is English/CUV-aligned (`remapVerses` at import time).
//! NT books are omitted; [`hebrew_verse`] returns `None`.

use std::collections::HashMap;
use std::sync::OnceLock;

/// One MorphHB word in Hebrew reading order (first = rightmost in RTL display).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HebrewWord {
    pub text: String,
    pub strongs: Option<String>,
    pub morph: Option<String>,
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

fn load_book_json(json: &str) -> Result<Vec<Vec<Vec<HebrewWord>>>, String> {
    let raw: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let chapters = raw
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

const GEN_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/he/Gen.json"
));

fn store() -> &'static HebrewStore {
    static STORE: OnceLock<HebrewStore> = OnceLock::new();
    STORE.get_or_init(|| {
        let mut books = HashMap::new();
        let gen_book = load_book_json(GEN_JSON).expect("embedded data/he/Gen.json");
        books.insert("Gen".to_string(), gen_book);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_1_1_has_words_and_h7225() {
        let words = hebrew_verse("Gen", 1, 1).expect("Gen 1:1");
        assert!(!words.is_empty());
        assert_eq!(words[0].strongs.as_deref(), Some("H7225"));
        assert_eq!(words[0].text.chars().next(), Some('ב'));
        assert!(has_hebrew_notes("Gen", 1, 1));
    }

    #[test]
    fn gen_1_sample_has_all_31_verses() {
        for n in 1..=31u32 {
            assert!(hebrew_verse("Gen", 1, n).is_some(), "Gen 1:{n}");
        }
        assert!(hebrew_verse("Gen", 1, 32).is_none());
        assert!(hebrew_verse("Gen", 2, 1).is_none());
    }

    #[test]
    fn john_and_missing_are_none() {
        assert!(hebrew_verse("John", 3, 16).is_none());
        assert!(!has_hebrew_notes("John", 3, 16));
        assert!(hebrew_verse("Gen", 2, 1).is_none());
        assert!(hebrew_verse("Exod", 1, 1).is_none());
    }
}
