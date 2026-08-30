//! Core types, 66-book catalog, and offline CUV 神版 + KJV store.

mod catalog;

pub use catalog::{BookMeta, CANON, Testament, lookup_canon};

use serde::Deserialize;
use std::fmt;

/// Embedded verse-aligned Genesis 1 (CUV 1919 神版 + KJV). Public domain.
const GENESIS_1_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/genesis-1.json"
));

/// Compact aligned 66-book store (CUV 神版 + KJV). Public domain.
const BIBLE_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/cuv-kjv.json"
));

/// Identifies a translation. Ships CUV 1919 神版 and KJV.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TranslationId {
    /// Chinese Union Version, 1919, 神版 (public domain).
    Cuv1919,
    /// King James Version (public domain).
    Kjv,
}

impl TranslationId {
    pub fn label(self) -> &'static str {
        match self {
            TranslationId::Cuv1919 => "和合本 1919 神版",
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

/// A canonical book in a loaded [`Bible`], with chapter count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BookEntry {
    pub index: usize,
    pub meta: BookMeta,
    pub chapter_count: u32,
}

#[derive(Debug)]
pub enum LoadError {
    Json(serde_json::Error),
    Empty,
    UnknownBook(String),
    MissingChapter { book: String, chapter: u32 },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Json(e) => write!(f, "failed to parse scripture JSON: {e}"),
            LoadError::Empty => write!(f, "chapter contains no verses"),
            LoadError::UnknownBook(book) => write!(f, "unknown book: {book}"),
            LoadError::MissingChapter { book, chapter } => {
                write!(f, "missing {book} chapter {chapter}")
            }
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

#[derive(Deserialize)]
struct FileStore {
    books: Vec<FileAlignedBook>,
}

#[derive(Deserialize)]
struct FileAlignedBook {
    book: String,
    #[serde(rename = "bookId")]
    book_id: u32,
    chapters: Vec<FileAlignedChapter>,
}

#[derive(Deserialize)]
struct FileAlignedChapter {
    c: u32,
    v: Vec<FileAlignedVerse>,
}

#[derive(Deserialize)]
struct FileAlignedVerse {
    n: u32,
    zh: String,
    en: String,
}

struct BookData {
    entry: BookEntry,
    /// 0-based index = chapter number - 1.
    chapters: Vec<Vec<Verse>>,
}

/// Offline 66-book bilingual store (CUV 神版 + KJV).
pub struct Bible {
    books: Vec<BookData>,
}

impl Bible {
    pub fn catalog(&self) -> Vec<BookEntry> {
        self.books.iter().map(|b| b.entry).collect()
    }

    pub fn len(&self) -> usize {
        self.books.len()
    }

    pub fn is_empty(&self) -> bool {
        self.books.is_empty()
    }

    pub fn book_at(&self, index: usize) -> Option<BookEntry> {
        self.books.get(index).map(|b| b.entry)
    }

    pub fn index_of(&self, book: &str) -> Option<usize> {
        let key = book.trim();
        self.books.iter().position(|b| {
            let m = b.entry.meta;
            m.osis.eq_ignore_ascii_case(key)
                || m.name_zh == key
                || m.name_en.eq_ignore_ascii_case(key)
                || key.parse::<u32>().ok() == Some(m.book_id)
        })
    }

    /// Load a chapter by OSIS id, Traditional Chinese name, English name, or 1-based book id.
    pub fn load_chapter(&self, book: &str, chapter: u32) -> Result<Chapter, LoadError> {
        let index = self
            .index_of(book)
            .ok_or_else(|| LoadError::UnknownBook(book.to_string()))?;
        self.load_chapter_at(index, chapter)
    }

    pub fn load_chapter_at(&self, index: usize, chapter: u32) -> Result<Chapter, LoadError> {
        let book = self
            .books
            .get(index)
            .ok_or_else(|| LoadError::UnknownBook(index.to_string()))?;
        if chapter == 0 {
            return Err(LoadError::MissingChapter {
                book: book.entry.meta.osis.to_string(),
                chapter,
            });
        }
        let verses = book
            .chapters
            .get((chapter as usize).saturating_sub(1))
            .cloned()
            .ok_or_else(|| LoadError::MissingChapter {
                book: book.entry.meta.osis.to_string(),
                chapter,
            })?;
        if verses.is_empty() {
            return Err(LoadError::Empty);
        }
        let meta = book.entry.meta;
        Ok(Chapter {
            book: meta.osis.to_string(),
            book_id: meta.book_id,
            book_name_zh: meta.name_zh.to_string(),
            book_name_en: meta.name_en.to_string(),
            chapter,
            chinese_label: TranslationId::Cuv1919.label().to_string(),
            english_label: TranslationId::Kjv.label().to_string(),
            verses,
        })
    }
}

/// Load the embedded 66-book CUV 神版 + KJV store.
pub fn load_bible() -> Result<Bible, LoadError> {
    let file: FileStore = serde_json::from_str(BIBLE_JSON).map_err(LoadError::Json)?;
    if file.books.is_empty() {
        return Err(LoadError::Empty);
    }

    let mut books = Vec::with_capacity(file.books.len());
    for (index, raw) in file.books.into_iter().enumerate() {
        let meta = lookup_canon(&raw.book).copied().unwrap_or_else(|| {
            // Fallback should not happen for the committed 66-book pack.
            CANON.get(index).copied().unwrap_or(CANON[0])
        });
        let mut chapters: Vec<Vec<Verse>> = Vec::new();
        for ch in raw.chapters {
            let n = ch.c as usize;
            if n == 0 {
                continue;
            }
            if chapters.len() < n {
                chapters.resize(n, Vec::new());
            }
            chapters[n - 1] =
                ch.v.into_iter()
                    .map(|v| Verse {
                        number: v.n,
                        chinese: v.zh,
                        english: v.en,
                    })
                    .collect();
        }
        let chapter_count = chapters.len() as u32;
        books.push(BookData {
            entry: BookEntry {
                index,
                meta: BookMeta {
                    osis: meta.osis,
                    book_id: if raw.book_id == 0 {
                        meta.book_id
                    } else {
                        raw.book_id
                    },
                    name_zh: meta.name_zh,
                    name_en: meta.name_en,
                    testament: meta.testament,
                },
                chapter_count,
            },
            chapters,
        });
    }

    Ok(Bible { books })
}

/// Load a chapter from the embedded store (parses the full Bible).
pub fn load_chapter(book: &str, chapter: u32) -> Result<Chapter, LoadError> {
    load_bible()?.load_chapter(book, chapter)
}

/// Load the embedded Genesis chapter 1 (31 verses, CUV 1919 神版 + KJV).
pub fn load_genesis_1() -> Result<Chapter, LoadError> {
    load_aligned_json(GENESIS_1_JSON)
}

/// Align two bible-data book files by verse number for a single chapter.
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
                chinese: v.text.replace("上帝", "神"),
                english: en_map.get(&v.number)?.to_string(),
            })
        })
        .collect();

    if verses.is_empty() {
        return Err(LoadError::Empty);
    }

    let meta = lookup_canon(&chinese.book);
    Ok(Chapter {
        book: chinese.book.clone(),
        book_id: chinese.book_id,
        book_name_zh: meta
            .map(|m| m.name_zh.to_string())
            .unwrap_or_else(|| "創世記".to_string()),
        book_name_en: meta
            .map(|m| m.name_en.to_string())
            .unwrap_or_else(|| chinese.english_name.clone()),
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
        assert_eq!(v1.chinese, "起初，神創造天地。");
        assert!(
            v1.chinese.starts_with("起初，神"),
            "CUV 神版 1:1 should start with 起初，神, got {:?}",
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
            assert!(
                !verse.chinese.contains("上帝"),
                "no 上帝 in Chinese scripture: {}:{} {:?}",
                chapter.chapter,
                verse.number,
                verse.chinese
            );
        }
        assert_eq!(chapter.chinese_label, "和合本 1919 神版");
        assert_eq!(chapter.english_label, "KJV");
    }

    #[test]
    fn catalog_has_66_books() {
        assert_eq!(CANON.len(), 66);
        let bible = load_bible().expect("embedded bible");
        assert_eq!(bible.len(), 66);
        assert_eq!(bible.catalog().len(), 66);
        assert_eq!(bible.book_at(0).unwrap().meta.name_zh, "創世記");
        assert_eq!(bible.book_at(65).unwrap().meta.name_zh, "啟示錄");
        assert_eq!(bible.book_at(0).unwrap().chapter_count, 50);
        assert_eq!(bible.book_at(65).unwrap().chapter_count, 22);
        let ot = bible
            .catalog()
            .into_iter()
            .filter(|b| b.meta.testament == Testament::Old)
            .count();
        let nt = bible
            .catalog()
            .into_iter()
            .filter(|b| b.meta.testament == Testament::New)
            .count();
        assert_eq!(ot, 39);
        assert_eq!(nt, 27);
    }

    #[test]
    fn john_3_and_revelation_last_load() {
        let bible = load_bible().expect("embedded bible");
        let john = bible.load_chapter("John", 3).expect("John 3");
        assert_eq!(john.book_name_zh, "約翰福音");
        assert_eq!(john.chapter, 3);
        assert!(!john.verses.is_empty());
        assert!(john.verses.iter().any(|v| v.number == 16));
        assert!(
            john.verses.iter().all(|v| !v.chinese.contains("上帝")),
            "no 上帝 in John 3"
        );

        let john_zh = bible.load_chapter("約翰福音", 3).expect("約翰福音 3");
        assert_eq!(john.verses.len(), john_zh.verses.len());

        let rev_meta = bible.book_at(bible.index_of("Rev").unwrap()).unwrap();
        let last = rev_meta.chapter_count;
        let rev = bible.load_chapter("Rev", last).expect("Revelation last");
        assert_eq!(rev.book_name_zh, "啟示錄");
        assert_eq!(rev.chapter, last);
        assert!(!rev.verses.is_empty());
        assert_eq!(last, 22);
    }

    #[test]
    fn load_chapter_gen_1_matches_helper() {
        let from_store = load_bible()
            .unwrap()
            .load_chapter("Gen", 1)
            .expect("Gen 1 from store");
        let from_helper = load_genesis_1().unwrap();
        assert_eq!(from_store.verses.len(), from_helper.verses.len());
        assert_eq!(from_store.verses[0].chinese, from_helper.verses[0].chinese);
        assert_eq!(from_store.chinese_label, "和合本 1919 神版");
    }

    #[test]
    fn cuv1919_label_is_shen_edition() {
        assert_eq!(TranslationId::Cuv1919.label(), "和合本 1919 神版");
    }
}
