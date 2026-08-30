//! Core types and Genesis 1 loader for omarchy-bible.
//!
//! Translation packs (WEB, additional public-domain versions, and a future
//! *licensed* CUNP pack that must never be vendored in this repo) would plug
//! in here: add a `TranslationId` variant + a pack loader that reads JSON from
//! a user data directory instead of `include_str!`.

use serde::Deserialize;
use std::fmt;

/// Embedded, verse-aligned Genesis 1 (CUV 1919 + KJV). Public domain.
const GENESIS_1_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/genesis-1.json"
));

/// Identifies a translation. Phase 1 only ships CUV 1919 and KJV.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TranslationId {
    /// Chinese Union Version, 1919 (public domain). UI label: 「和合本 1919」.
    Cuv1919,
    /// King James Version (public domain). UI label: 「KJV」.
    Kjv,
    // Future packs (do not vendor copyrighted text in-tree):
    // World English Bible, and a separately-distributed CUNP pack.
}

impl TranslationId {
    pub fn label(self) -> &'static str {
        match self {
            TranslationId::Cuv1919 => "和合本 1919",
            TranslationId::Kjv => "KJV",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verse {
    pub number: u32,
    pub chinese: String,
    pub english: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter {
    pub book: String,
    pub book_id: u32,
    pub book_name_zh: String,
    pub book_name_en: String,
    pub chapter: u32,
    pub chinese_label: String,
    pub english_label: String,
    pub verses: Vec<Verse>,
}

impl Chapter {
    pub fn title_zh(&self) -> String {
        format!("{} {}", self.book_name_zh, self.chapter)
    }
}

#[derive(Debug)]
pub enum LoadError {
    Json(serde_json::Error),
    Empty,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Json(e) => write!(f, "failed to parse scripture JSON: {e}"),
            LoadError::Empty => write!(f, "chapter contains no verses"),
        }
    }
}

impl std::error::Error for LoadError {}

#[derive(Deserialize)]
struct FileChapter {
    book: String,
    #[serde(rename = "bookId")]
    book_id: u32,
    #[serde(rename = "bookNameZh")]
    book_name_zh: String,
    #[serde(rename = "bookNameEn")]
    book_name_en: String,
    chapter: u32,
    #[serde(rename = "chineseLabel")]
    chinese_label: String,
    #[serde(rename = "englishLabel")]
    english_label: String,
    verses: Vec<FileVerse>,
}

#[derive(Deserialize)]
struct FileVerse {
    number: u32,
    zh: String,
    en: String,
}

/// Load the embedded Genesis chapter 1 (31 verses, CUV 1919 + KJV).
pub fn load_genesis_1() -> Result<Chapter, LoadError> {
    load_aligned_json(GENESIS_1_JSON)
}

/// Schema of midvash/bible-data `versions/*/books/*.json`.
#[derive(Debug, Deserialize)]
pub struct FileBook {
    pub version: String,
    pub book: String,
    #[serde(rename = "bookId")]
    pub book_id: u32,
    #[serde(rename = "englishName")]
    pub english_name: String,
    pub testament: String,
    pub chapters: Vec<FileBookChapter>,
}

#[derive(Debug, Deserialize)]
pub struct FileBookChapter {
    pub chapter: u32,
    pub verses: Vec<FileBookVerse>,
}

#[derive(Debug, Deserialize)]
pub struct FileBookVerse {
    pub number: u32,
    pub text: String,
}

/// Align two bible-data book files by verse number for a single chapter.
/// Template for future on-disk translation packs.
pub fn align_chapter(
    chinese: &FileBook,
    english: &FileBook,
    chapter: u32,
) -> Result<Chapter, LoadError> {
    let zh_ch = chinese
        .chapters
        .iter()
        .find(|c| c.chapter == chapter)
        .ok_or(LoadError::Empty)?;
    let en_map: std::collections::BTreeMap<u32, &str> = english
        .chapters
        .iter()
        .find(|c| c.chapter == chapter)
        .ok_or(LoadError::Empty)?
        .verses
        .iter()
        .map(|v| (v.number, v.text.as_str()))
        .collect();

    let verses: Vec<Verse> = zh_ch
        .verses
        .iter()
        .filter_map(|v| {
            Some(Verse {
                number: v.number,
                chinese: v.text.clone(),
                english: en_map.get(&v.number)?.to_string(),
            })
        })
        .collect();

    if verses.is_empty() {
        return Err(LoadError::Empty);
    }

    Ok(Chapter {
        book: chinese.book.clone(),
        book_id: chinese.book_id,
        book_name_zh: "創世記".to_string(),
        book_name_en: chinese.english_name.clone(),
        chapter,
        chinese_label: TranslationId::Cuv1919.label().to_string(),
        english_label: TranslationId::Kjv.label().to_string(),
        verses,
    })
}

fn load_aligned_json(json: &str) -> Result<Chapter, LoadError> {
    let file: FileChapter = serde_json::from_str(json).map_err(LoadError::Json)?;
    if file.verses.is_empty() {
        return Err(LoadError::Empty);
    }
    Ok(Chapter {
        book: file.book,
        book_id: file.book_id,
        book_name_zh: file.book_name_zh,
        book_name_en: file.book_name_en,
        chapter: file.chapter,
        chinese_label: file.chinese_label,
        english_label: file.english_label,
        verses: file
            .verses
            .into_iter()
            .map(|v| Verse {
                number: v.number,
                chinese: v.zh,
                english: v.en,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_1_has_31_aligned_verses() {
        let chapter = load_genesis_1().expect("embedded Genesis 1 JSON");
        assert_eq!(chapter.chapter, 1);
        assert_eq!(chapter.verses.len(), 31, "Genesis 1 has 31 verses");
        let v1 = &chapter.verses[0];
        assert_eq!(v1.number, 1);
        assert!(
            v1.chinese.starts_with("起初"),
            "CUV 1:1 should start with 起初, got {:?}",
            v1.chinese
        );
        assert!(
            v1.english.contains("In the beginning"),
            "KJV 1:1 should contain 'In the beginning', got {:?}",
            v1.english
        );
        for (i, verse) in chapter.verses.iter().enumerate() {
            assert_eq!(verse.number, (i as u32) + 1);
            assert!(!verse.chinese.is_empty());
            assert!(!verse.english.is_empty());
        }
        assert_eq!(chapter.chinese_label, "和合本 1919");
        assert_eq!(chapter.english_label, "KJV");
    }
}
