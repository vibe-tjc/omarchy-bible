//! Core types, 66-book catalog, and offline CUV 神版 + KJV store.

mod catalog;
mod hebrew;

pub use catalog::{BibleRef, BookMeta, CANON, Testament, lookup_canon, parse_bible_ref};
pub use hebrew::{HebrewWord, hebrew_verse, has_hebrew_notes};

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

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
///
/// Additional translations later become new variants; UI lanes are keyed by this id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationId {
    /// Chinese Union Version, 1919, 神版 (public domain).
    Cuv1919,
    /// King James Version (public domain).
    Kjv,
}

impl TranslationId {
    /// Translations bundled with this build, in default compare order (中 then 英).
    pub const ALL: &'static [TranslationId] = &[TranslationId::Cuv1919, TranslationId::Kjv];

    pub fn label(self) -> &'static str {
        match self {
            TranslationId::Cuv1919 => "和合本 1919 神版",
            TranslationId::Kjv => "KJV",
        }
    }

    /// CJK body text uses the Chinese font size; Latin uses a slightly smaller size.
    pub fn is_cjk(self) -> bool {
        matches!(self, TranslationId::Cuv1919)
    }
}

/// How many translation lanes the reading pane shows.
///
/// `Compare` renders every id in the vec (1..=N, N>=2), so a third translation later
/// is data + a checkbox, not a rewrite of the verse view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    Single(TranslationId),
    Compare(Vec<TranslationId>),
}

impl Default for ViewMode {
    fn default() -> Self {
        Self::Compare(TranslationId::ALL.to_vec())
    }
}

impl ViewMode {
    pub fn single(id: TranslationId) -> Self {
        Self::Single(id)
    }

    pub fn compare(ids: Vec<TranslationId>) -> Self {
        Self::Compare(ids).sanitized()
    }

    /// Drop empty compare lists; a 1-item compare becomes single.
    pub fn sanitized(self) -> Self {
        match self {
            ViewMode::Single(id) => ViewMode::Single(id),
            ViewMode::Compare(ids) if ids.len() >= 2 => ViewMode::Compare(ids),
            ViewMode::Compare(ids) if ids.len() == 1 => ViewMode::Single(ids[0]),
            ViewMode::Compare(_) => ViewMode::default(),
        }
    }

    pub fn lanes(&self) -> Vec<TranslationId> {
        match self {
            ViewMode::Single(id) => vec![*id],
            ViewMode::Compare(ids) => ids.clone(),
        }
    }

    pub fn lane_count(&self) -> usize {
        self.lanes().len()
    }

    pub fn is_single(&self) -> bool {
        matches!(self, ViewMode::Single(_))
    }

    pub fn is_compare(&self) -> bool {
        matches!(self, ViewMode::Compare(_))
    }

    /// Header chrome, e.g. `和合本 1919 神版` or `和合本 1919 神版 · KJV`.
    pub fn header_label(&self) -> String {
        self.lanes()
            .iter()
            .map(|id| id.label())
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

/// One verse: a number plus ordered translation lanes.
///
/// Loader still reads CUV+KJV internally; UI never assumes only chinese/english fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verse {
    pub number: u32,
    pub texts: Vec<(TranslationId, String)>,
    /// MorphHB words in Hebrew reading order; `None` for NT or chapters without data.
    pub hebrew: Option<Vec<HebrewWord>>,
    /// Precomputed lowercase of non-CJK lanes (same ids as in `texts`).
    folded: Vec<(TranslationId, String)>,
}

impl Verse {
    pub fn from_texts(number: u32, texts: Vec<(TranslationId, String)>) -> Self {
        let folded = texts
            .iter()
            .filter(|(id, _)| !id.is_cjk())
            .map(|(id, text)| (*id, text.to_lowercase()))
            .collect();
        Self {
            number,
            texts,
            hebrew: None,
            folded,
        }
    }

    pub fn bilingual(number: u32, chinese: impl Into<String>, english: impl Into<String>) -> Self {
        Self::from_texts(
            number,
            vec![
                (TranslationId::Cuv1919, chinese.into()),
                (TranslationId::Kjv, english.into()),
            ],
        )
    }

    pub fn get(&self, id: TranslationId) -> Option<&str> {
        self.texts
            .iter()
            .find(|(tid, _)| *tid == id)
            .map(|(_, text)| text.as_str())
    }

    fn folded_get(&self, id: TranslationId) -> Option<&str> {
        self.folded
            .iter()
            .find(|(tid, _)| *tid == id)
            .map(|(_, text)| text.as_str())
    }

    pub fn texts_for<'a>(&'a self, lanes: &[TranslationId]) -> Vec<(TranslationId, &'a str)> {
        lanes
            .iter()
            .filter_map(|&id| self.get(id).map(|text| (id, text)))
            .collect()
    }

    /// Bytes of display strings plus precomputed search folds.
    pub fn stored_string_bytes(&self) -> usize {
        self.texts.iter().map(|(_, s)| s.len()).sum::<usize>()
            + self.folded.iter().map(|(_, s)| s.len()).sum::<usize>()
    }

    pub fn stored_string_count(&self) -> usize {
        self.texts.len() + self.folded.len()
    }
}

/// One verse hit from [`Bible::search`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    /// OSIS id, e.g. `"Gen"`, `"John"`.
    pub book: String,
    pub book_name_zh: String,
    /// 0-based index in the loaded store (for `load_chapter_at`).
    pub book_index: usize,
    pub chapter: u32,
    pub verse: u32,
    pub translation: TranslationId,
    /// Short excerpt of the matching lane, truncated with ellipsis.
    pub snippet: String,
}

impl SearchHit {
    pub fn ref_zh(&self) -> String {
        format!("{} {}:{}", self.book_name_zh, self.chapter, self.verse)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter {
    pub book: String,
    pub book_id: u32,
    pub book_name_zh: String,
    pub book_name_en: String,
    pub chapter: u32,
    pub verses: Arc<[Verse]>,
}

impl Chapter {
    /// Unique translation ids present in this chapter, in first-seen order.
    pub fn available_translations(&self) -> Vec<TranslationId> {
        let mut ids = Vec::new();
        for verse in self.verses.iter() {
            for (id, _) in &verse.texts {
                if !ids.contains(id) {
                    ids.push(*id);
                }
            }
        }
        ids
    }
}

impl Chapter {
    pub fn title_zh(&self) -> String {
        format!("{} {}", self.book_name_zh, self.chapter)
    }

    pub fn title_en(&self) -> String {
        format!("{} {}", self.book_name_en, self.chapter)
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
    #[allow(dead_code)]
    chinese_label: String,
    #[serde(rename = "englishLabel")]
    #[allow(dead_code)]
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
    /// 0-based index = chapter number - 1. Shared so `load_chapter_at` is an Arc clone.
    chapters: Vec<Arc<[Verse]>>,
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
        let verses = attach_hebrew(meta.osis, chapter, verses);
        Ok(Chapter {
            book: meta.osis.to_string(),
            book_id: meta.book_id,
            book_name_zh: meta.name_zh.to_string(),
            book_name_en: meta.name_en.to_string(),
            chapter,
            verses,
        })
    }

    /// In-memory substring search over the given translation lanes.
    ///
    /// - Empty / whitespace-only queries return no hits (query is trimmed).
    /// - CJK lanes (`TranslationId::is_cjk`): exact substring.
    /// - Other lanes: case-insensitive substring.
    /// - One hit per verse (first matching lane in `lanes` order).
    pub fn search(&self, query: &str, lanes: &[TranslationId]) -> Vec<SearchHit> {
        let query = query.trim();
        if query.is_empty() || lanes.is_empty() {
            return Vec::new();
        }

        let mut unique_lanes = Vec::new();
        for &id in lanes {
            if !unique_lanes.contains(&id) {
                unique_lanes.push(id);
            }
        }

        let query_lower = query.to_lowercase();
        let mut hits = Vec::new();
        for book in &self.books {
            let meta = book.entry.meta;
            for (ch_idx, verses) in book.chapters.iter().enumerate() {
                let chapter = (ch_idx as u32) + 1;
                for verse in verses.iter() {
                    for &lane in &unique_lanes {
                        let Some(text) = verse.get(lane) else {
                            continue;
                        };
                        if !text_matches(verse, text, query, &query_lower, lane) {
                            continue;
                        }
                        hits.push(SearchHit {
                            book: meta.osis.to_string(),
                            book_name_zh: meta.name_zh.to_string(),
                            book_index: book.entry.index,
                            chapter,
                            verse: verse.number,
                            translation: lane,
                            snippet: make_snippet(text, query, &query_lower, lane),
                        });
                        break;
                    }
                }
            }
        }
        hits
    }
}

fn text_matches(
    verse: &Verse,
    text: &str,
    query: &str,
    query_lower: &str,
    lane: TranslationId,
) -> bool {
    if lane.is_cjk() {
        text.contains(query)
    } else if let Some(folded) = verse.folded_get(lane) {
        folded.contains(query_lower)
    } else {
        text.to_lowercase().contains(query_lower)
    }
}

fn find_match_byte(text: &str, query: &str, query_lower: &str, cjk: bool) -> Option<usize> {
    if cjk {
        text.find(query)
    } else {
        // KJV is ASCII; lowercasing preserves byte offsets.
        text.to_lowercase().find(query_lower)
    }
}

fn make_snippet(text: &str, query: &str, query_lower: &str, lane: TranslationId) -> String {
    let cjk = lane.is_cjk();
    let context = if cjk { 12 } else { 24 };
    let chars: Vec<char> = text.chars().collect();
    let Some(byte_idx) = find_match_byte(text, query, query_lower, cjk) else {
        return truncate_chars(&chars, 40);
    };
    let match_char = text[..byte_idx.min(text.len())].chars().count();
    let q_len = query.chars().count().max(1);
    let start = match_char.saturating_sub(context);
    let end = (match_char + q_len + context).min(chars.len());
    let mut out = String::new();
    if start > 0 {
        out.push('…');
    }
    out.extend(chars[start..end].iter());
    if end < chars.len() {
        out.push('…');
    }
    out
}

fn truncate_chars(chars: &[char], max: usize) -> String {
    if chars.len() <= max {
        return chars.iter().collect();
    }
    let mut out: String = chars[..max].iter().collect();
    out.push('…');
    out
}

fn attach_hebrew(book: &str, chapter: u32, verses: Arc<[Verse]>) -> Arc<[Verse]> {
    let mut changed = false;
    let enriched: Vec<Verse> = verses
        .iter()
        .map(|v| {
            let mut cloned = v.clone();
            cloned.hebrew = hebrew::hebrew_verse(book, chapter, v.number);
            if cloned.hebrew.is_some() {
                changed = true;
            }
            cloned
        })
        .collect();
    if changed {
        enriched.into()
    } else {
        verses
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
        let mut chapters: Vec<Arc<[Verse]>> = Vec::new();
        for ch in raw.chapters {
            let n = ch.c as usize;
            if n == 0 {
                continue;
            }
            if chapters.len() < n {
                chapters.resize(n, Arc::from([]));
            }
            chapters[n - 1] =
                ch.v.into_iter()
                    .map(|v| Verse::bilingual(v.n, v.zh, v.en))
                    .collect::<Vec<_>>()
                    .into();
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
                    name_zh_short: meta.name_zh_short,
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

    let verses: Arc<[Verse]> = zh_ch
        .verses
        .iter()
        .filter_map(|v| {
            Some(Verse::bilingual(
                v.number,
                v.text.replace("上帝", "神"),
                en_map.get(&v.number)?.to_string(),
            ))
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
        verses,
    })
}

fn load_aligned_json(json: &str) -> Result<Chapter, LoadError> {
    let file: FileChapter = serde_json::from_str(json).map_err(LoadError::Json)?;
    if file.verses.is_empty() {
        return Err(LoadError::Empty);
    }
    let book = file.book;
    let chapter = file.chapter;
    let verses: Arc<[Verse]> = file
        .verses
        .into_iter()
        .map(|v| Verse::bilingual(v.number, v.zh, v.en))
        .collect::<Vec<_>>()
        .into();
    let verses = attach_hebrew(&book, chapter, verses);
    Ok(Chapter {
        book,
        book_id: file.book_id,
        book_name_zh: file.book_name_zh,
        book_name_en: file.book_name_en,
        chapter,
        verses,
    })
}

/// Inclusive verse-number range currently selected (sorted, unique).
pub fn selected_verse_range(numbers: &[u32]) -> Option<(u32, u32)> {
    let mut nums: Vec<u32> = numbers.to_vec();
    nums.sort_unstable();
    nums.dedup();
    let first = *nums.first()?;
    let last = *nums.last()?;
    Some((first, last))
}

/// One selected unit: a verse number in a specific translation lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VerseUnit {
    pub number: u32,
    pub translation: TranslationId,
}

impl VerseUnit {
    pub fn new(number: u32, translation: TranslationId) -> Self {
        Self {
            number,
            translation,
        }
    }
}

/// Short language label for the select bar (`中文` / `英文`).
pub fn translation_short_label(id: TranslationId) -> &'static str {
    match id {
        TranslationId::Cuv1919 => "中文",
        TranslationId::Kjv => "英文",
    }
}

/// Apply a select-mode tap to `translation` at `number`.
///
/// - Already selected → toggle off.
/// - Exactly one unit selected in that translation → fill the inclusive range
///   (union; does not uncheck units outside the range or in other translations).
/// - Otherwise → toggle this unit on.
pub fn apply_verse_tap(
    selected: &mut BTreeSet<VerseUnit>,
    number: u32,
    translation: TranslationId,
    verse_numbers: &[u32],
) {
    let unit = VerseUnit::new(number, translation);
    if selected.contains(&unit) {
        selected.remove(&unit);
        return;
    }
    let same: Vec<u32> = selected
        .iter()
        .filter(|u| u.translation == translation)
        .map(|u| u.number)
        .collect();
    if same.len() == 1 {
        let a = same[0];
        let (lo, hi) = if a <= number {
            (a, number)
        } else {
            (number, a)
        };
        for &n in verse_numbers {
            if n >= lo && n <= hi {
                selected.insert(VerseUnit::new(n, translation));
            }
        }
    } else {
        selected.insert(unit);
    }
}

fn numbers_for(selected: &[VerseUnit], id: TranslationId) -> Vec<u32> {
    let mut nums: Vec<u32> = selected
        .iter()
        .filter(|u| u.translation == id)
        .map(|u| u.number)
        .collect();
    nums.sort_unstable();
    nums.dedup();
    nums
}

/// Bottom-bar / copy header reference, e.g. `約翰福音 3:16–18`.
pub fn format_ref_zh(chapter: &Chapter, numbers: &[u32]) -> String {
    match selected_verse_range(numbers) {
        Some((a, b)) if a == b => format!("{} {}:{}", chapter.book_name_zh, chapter.chapter, a),
        Some((a, b)) => format!("{} {}:{}–{}", chapter.book_name_zh, chapter.chapter, a, b),
        None => chapter.title_zh(),
    }
}

/// English reference, e.g. `John 3:16–18`.
pub fn format_ref_en(chapter: &Chapter, numbers: &[u32]) -> String {
    match selected_verse_range(numbers) {
        Some((a, b)) if a == b => format!("{} {}:{}", chapter.book_name_en, chapter.chapter, a),
        Some((a, b)) => format!("{} {}:{}–{}", chapter.book_name_en, chapter.chapter, a, b),
        None => chapter.title_en(),
    }
}

fn format_ref_for(chapter: &Chapter, id: TranslationId, numbers: &[u32]) -> String {
    if id.is_cjk() {
        format_ref_zh(chapter, numbers)
    } else {
        format_ref_en(chapter, numbers)
    }
}

/// Bottom-bar summary, e.g. `已選 中文 3 節 · 英文 1 節 · 約翰福音 3:16–18 · John 3:16`.
pub fn format_selection_summary(chapter: &Chapter, selected: &[VerseUnit]) -> String {
    if selected.is_empty() {
        return "已選 0 節".to_string();
    }
    let mut count_bits = Vec::new();
    let mut ref_bits = Vec::new();
    for &id in TranslationId::ALL {
        let nums = numbers_for(selected, id);
        if nums.is_empty() {
            continue;
        }
        count_bits.push(format!("{} {} 節", translation_short_label(id), nums.len()));
        ref_bits.push(format_ref_for(chapter, id, &nums));
    }
    if count_bits.is_empty() {
        return "已選 0 節".to_string();
    }
    format!("已選 {} · {}", count_bits.join(" · "), ref_bits.join(" · "))
}

/// Clipboard payload grouped by translation. Only selected lanes are included.
///
/// ```text
/// 約翰福音 3:16–18（和合本 1919 神版）
/// 16 …
/// 17 …
///
/// John 3:16–18 (KJV)
/// 16 …
/// ```
pub fn format_verse_copy(chapter: &Chapter, selected: &[VerseUnit]) -> String {
    let mut blocks = Vec::new();
    for &id in TranslationId::ALL {
        let nums = numbers_for(selected, id);
        if nums.is_empty() {
            continue;
        }
        let header = if id.is_cjk() {
            format!("{}（{}）", format_ref_zh(chapter, &nums), id.label())
        } else {
            format!("{} ({})", format_ref_en(chapter, &nums), id.label())
        };
        let mut block = format!("{header}\n");
        for n in nums {
            let Some(verse) = chapter.verses.iter().find(|v| v.number == n) else {
                continue;
            };
            let Some(text) = verse.get(id) else {
                continue;
            };
            block.push_str(&format!("{n} {text}\n"));
        }
        blocks.push(block);
    }
    blocks.join("\n")
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
        assert_eq!(v1.get(TranslationId::Cuv1919), Some("起初，神創造天地。"));
        assert_eq!(v1.texts.len(), 2, "embedded verses expose CUV + KJV lanes");
        assert!(
            v1.get(TranslationId::Cuv1919)
                .unwrap()
                .starts_with("起初，神"),
            "CUV 神版 1:1 should start with 起初，神, got {:?}",
            v1.get(TranslationId::Cuv1919)
        );
        assert!(
            v1.get(TranslationId::Kjv)
                .unwrap()
                .contains("In the beginning"),
            "KJV 1:1 should contain 'In the beginning', got {:?}",
            v1.get(TranslationId::Kjv)
        );
        for (i, verse) in chapter.verses.iter().enumerate() {
            assert_eq!(verse.number, (i as u32) + 1);
            let zh = verse.get(TranslationId::Cuv1919).unwrap_or("");
            let en = verse.get(TranslationId::Kjv).unwrap_or("");
            assert!(!zh.is_empty());
            assert!(!en.is_empty());
            assert!(
                !zh.contains("上帝"),
                "no 上帝 in Chinese scripture: {}:{} {:?}",
                chapter.chapter,
                verse.number,
                zh
            );
        }
        assert_eq!(
            chapter.available_translations(),
            vec![TranslationId::Cuv1919, TranslationId::Kjv]
        );
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
            john.verses
                .iter()
                .all(|v| { !v.get(TranslationId::Cuv1919).unwrap_or("").contains("上帝") }),
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
        assert_eq!(
            from_store.verses[0].get(TranslationId::Cuv1919),
            from_helper.verses[0].get(TranslationId::Cuv1919)
        );
        assert_eq!(
            from_store.available_translations()[0].label(),
            "和合本 1919 神版"
        );
    }

    #[test]
    fn cuv1919_label_is_shen_edition() {
        assert_eq!(TranslationId::Cuv1919.label(), "和合本 1919 神版");
    }

    #[test]
    fn view_mode_single_has_one_lane() {
        let mode = ViewMode::single(TranslationId::Cuv1919);
        assert_eq!(mode.lane_count(), 1);
        assert_eq!(mode.lanes(), vec![TranslationId::Cuv1919]);
        assert!(mode.is_single());
        assert_eq!(mode.header_label(), "和合本 1919 神版");

        let kjv = ViewMode::single(TranslationId::Kjv);
        assert_eq!(kjv.lane_count(), 1);
        assert_eq!(kjv.header_label(), "KJV");
    }

    #[test]
    fn view_mode_compare_renders_n_lanes() {
        let two = ViewMode::compare(vec![TranslationId::Cuv1919, TranslationId::Kjv]);
        assert_eq!(two.lane_count(), 2);
        assert!(two.is_compare());
        assert_eq!(two.header_label(), "和合本 1919 神版 · KJV");

        // A third id is just another lane — UI iterates this vec.
        let three = ViewMode::Compare(vec![
            TranslationId::Cuv1919,
            TranslationId::Kjv,
            TranslationId::Cuv1919,
        ]);
        assert_eq!(three.lane_count(), 3);

        assert_eq!(
            ViewMode::compare(vec![]).lane_count(),
            2,
            "empty compare sanitizes to default"
        );
        assert!(ViewMode::compare(vec![TranslationId::Kjv]).is_single());
    }

    #[test]
    fn view_mode_serde_roundtrip() {
        let single = ViewMode::single(TranslationId::Kjv);
        let json = serde_json::to_string(&single).unwrap();
        assert!(json.contains("kjv"), "{json}");
        let back: ViewMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, single);

        let compare = ViewMode::default();
        let json = serde_json::to_string(&compare).unwrap();
        let back: ViewMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back.lanes(), compare.lanes());
    }

    #[test]
    fn search_qichu_finds_genesis_1_1_cuv() {
        let bible = load_bible().expect("embedded bible");
        let hits = bible.search("起初", &[TranslationId::Cuv1919]);
        assert!(
            hits.iter().any(|h| {
                h.book == "Gen"
                    && h.book_name_zh == "創世記"
                    && h.chapter == 1
                    && h.verse == 1
                    && h.translation == TranslationId::Cuv1919
            }),
            "expected 創世記 1:1 CUV in {hits:?}"
        );
        assert!(
            hits[0].snippet.contains("起初"),
            "snippet should include the query, got {:?}",
            hits[0].snippet
        );
        assert!(
            bible.search("起初", &[TranslationId::Kjv]).is_empty(),
            "KJV lane should not match CJK substring"
        );
    }

    #[test]
    fn search_in_the_beginning_any_case_finds_genesis_1_1_kjv() {
        let bible = load_bible().expect("embedded bible");
        for q in ["In the beginning", "in the beginning", "IN THE BEGINNING"] {
            let hits = bible.search(q, &[TranslationId::Kjv]);
            assert!(
                hits.iter().any(|h| {
                    h.book == "Gen"
                        && h.chapter == 1
                        && h.verse == 1
                        && h.translation == TranslationId::Kjv
                }),
                "expected Gen 1:1 KJV for {q:?}, got {hits:?}"
            );
        }
        let both = bible.search("In the beginning", TranslationId::ALL);
        assert!(
            both.iter()
                .any(|h| h.book == "Gen" && h.chapter == 1 && h.verse == 1)
        );
    }

    #[test]
    fn search_god_so_loved_finds_john_3_16_kjv() {
        let bible = load_bible().expect("embedded bible");
        let hits = bible.search("God so loved", &[TranslationId::Kjv]);
        assert!(
            hits.iter()
                .any(|h| h.book == "John" && h.chapter == 3 && h.verse == 16),
            "{hits:?}"
        );
    }

    #[test]
    fn search_shen_ai_shi_ren_finds_john_3_16_cuv() {
        let bible = load_bible().expect("embedded bible");
        let hits = bible.search("神愛世人", &[TranslationId::Cuv1919]);
        assert!(
            hits.iter().any(|h| {
                h.book == "John"
                    && h.book_name_zh == "約翰福音"
                    && h.chapter == 3
                    && h.verse == 16
                    && h.translation == TranslationId::Cuv1919
            }),
            "expected 約翰福音 3:16 CUV in {hits:?}"
        );
        assert_eq!(hits[0].ref_zh(), "約翰福音 3:16");
    }

    #[test]
    fn search_empty_query_returns_no_hits() {
        let bible = load_bible().expect("embedded bible");
        assert!(bible.search("", TranslationId::ALL).is_empty());
        assert!(bible.search("   ", TranslationId::ALL).is_empty());
        assert!(bible.search("\t\n", TranslationId::ALL).is_empty());
    }

    #[test]
    fn search_does_not_introduce_shangdi() {
        let bible = load_bible().expect("embedded bible");
        let hits = bible.search("神", &[TranslationId::Cuv1919]);
        assert!(!hits.is_empty());
        for hit in &hits {
            assert!(
                !hit.snippet.contains("上帝"),
                "no 上帝 in search snippet: {} {:?}",
                hit.ref_zh(),
                hit.snippet
            );
        }
        let john = bible.load_chapter("John", 3).unwrap();
        assert!(
            john.verses
                .iter()
                .all(|v| !v.get(TranslationId::Cuv1919).unwrap_or("").contains("上帝"))
        );
    }

    fn john_verse_numbers(john: &Chapter) -> Vec<u32> {
        john.verses.iter().map(|v| v.number).collect()
    }

    fn units(pairs: &[(u32, TranslationId)]) -> Vec<VerseUnit> {
        pairs.iter().map(|&(n, id)| VerseUnit::new(n, id)).collect()
    }

    #[test]
    fn apply_verse_tap_toggles_and_fills_range_per_translation() {
        let bible = load_bible().expect("bible");
        let john = bible.load_chapter("John", 3).expect("John 3");
        let nums = john_verse_numbers(&john);
        let mut sel = BTreeSet::new();

        apply_verse_tap(&mut sel, 16, TranslationId::Cuv1919, &nums);
        assert_eq!(sel.len(), 1);

        apply_verse_tap(&mut sel, 18, TranslationId::Cuv1919, &nums);
        let cuv: Vec<u32> = sel
            .iter()
            .filter(|u| u.translation == TranslationId::Cuv1919)
            .map(|u| u.number)
            .collect();
        assert_eq!(cuv, vec![16, 17, 18]);

        apply_verse_tap(&mut sel, 21, TranslationId::Cuv1919, &nums);
        let cuv: Vec<u32> = sel
            .iter()
            .filter(|u| u.translation == TranslationId::Cuv1919)
            .map(|u| u.number)
            .collect();
        assert_eq!(
            cuv,
            vec![16, 17, 18, 21],
            "further taps toggle only, never wipe"
        );

        apply_verse_tap(&mut sel, 17, TranslationId::Cuv1919, &nums);
        assert!(!sel.contains(&VerseUnit::new(17, TranslationId::Cuv1919)));
        assert!(sel.contains(&VerseUnit::new(16, TranslationId::Cuv1919)));
        assert!(sel.contains(&VerseUnit::new(18, TranslationId::Cuv1919)));

        apply_verse_tap(&mut sel, 16, TranslationId::Kjv, &nums);
        assert!(sel.contains(&VerseUnit::new(16, TranslationId::Cuv1919)));
        assert!(sel.contains(&VerseUnit::new(16, TranslationId::Kjv)));
        assert!(!sel.contains(&VerseUnit::new(17, TranslationId::Kjv)));

        apply_verse_tap(&mut sel, 18, TranslationId::Kjv, &nums);
        let kjv: Vec<u32> = sel
            .iter()
            .filter(|u| u.translation == TranslationId::Kjv)
            .map(|u| u.number)
            .collect();
        assert_eq!(kjv, vec![16, 17, 18]);
        assert!(
            sel.contains(&VerseUnit::new(21, TranslationId::Cuv1919)),
            "range fill must not uncheck other translations or verses outside the range"
        );
    }

    #[test]
    fn format_verse_copy_single_and_compare() {
        let bible = load_bible().expect("bible");
        let john = bible.load_chapter("John", 3).expect("John 3");
        let nums = [16u32, 17, 18];
        let cuv = units(&[
            (16, TranslationId::Cuv1919),
            (17, TranslationId::Cuv1919),
            (18, TranslationId::Cuv1919),
        ]);
        let single = format_verse_copy(&john, &cuv);
        assert!(
            single.starts_with("約翰福音 3:16–18（和合本 1919 神版）"),
            "{single}"
        );
        assert!(single.contains("16 "), "{single}");
        assert!(single.contains("17 "), "{single}");
        assert!(single.contains("18 "), "{single}");
        assert!(!single.contains("KJV"), "{single}");
        assert!(!single.contains("John 3"), "{single}");

        let both_units = units(&[
            (16, TranslationId::Cuv1919),
            (17, TranslationId::Cuv1919),
            (18, TranslationId::Cuv1919),
            (16, TranslationId::Kjv),
            (17, TranslationId::Kjv),
            (18, TranslationId::Kjv),
        ]);
        let both = format_verse_copy(&john, &both_units);
        assert!(
            both.starts_with("約翰福音 3:16–18（和合本 1919 神版）"),
            "{both}"
        );
        assert!(both.contains("John 3:16–18 (KJV)"), "{both}");
        assert!(both.contains("God so loved") || both.to_lowercase().contains("god so loved"));
        assert_eq!(format_ref_zh(&john, &nums), "約翰福音 3:16–18");
        assert_eq!(format_ref_zh(&john, &[16]), "約翰福音 3:16");
        assert_eq!(format_ref_en(&john, &nums), "John 3:16–18");
        assert_eq!(
            format_selection_summary(&john, &both_units),
            "已選 中文 3 節 · 英文 3 節 · 約翰福音 3:16–18 · John 3:16–18"
        );
        assert_eq!(
            format_selection_summary(&john, &cuv),
            "已選 中文 3 節 · 約翰福音 3:16–18"
        );
        let kjv_only = units(&[(16, TranslationId::Kjv)]);
        let kjv_copy = format_verse_copy(&john, &kjv_only);
        assert!(kjv_copy.starts_with("John 3:16 (KJV)"), "{kjv_copy}");
        assert!(!kjv_copy.contains("和合本"), "{kjv_copy}");
        assert_eq!(
            format_selection_summary(&john, &kjv_only),
            "已選 英文 1 節 · John 3:16"
        );
    }

    #[test]
    fn chapter_verses_are_shared_arc() {
        let bible = load_bible().expect("bible");
        // Gen 2 has no MorphHB sample, so the store Arc is reused.
        let a = bible.load_chapter_at(0, 2).unwrap();
        let b = bible.load_chapter_at(0, 2).unwrap();
        assert!(Arc::ptr_eq(&a.verses, &b.verses));
        assert!(!a.verses.is_empty());
        let g1a = bible.load_chapter_at(0, 1).unwrap();
        let g1b = bible.load_chapter_at(0, 1).unwrap();
        assert_eq!(g1a.verses.len(), 31);
        assert!(g1a.verses[0].hebrew.is_some());
        assert_eq!(g1a.verses[0].hebrew, g1b.verses[0].hebrew);
    }

    #[test]
    fn genesis_1_verses_expose_hebrew_marker() {
        let chapter = load_bible().unwrap().load_chapter("Gen", 1).unwrap();
        assert!(has_hebrew_notes("Gen", 1, 1));
        assert!(chapter.verses[0].hebrew.is_some());
        assert!(chapter.verses[30].hebrew.is_some());
        assert_eq!(chapter.verses.iter().filter(|v| v.hebrew.is_some()).count(), 31);
        let john = load_bible().unwrap().load_chapter("John", 3).unwrap();
        assert!(john.verses.iter().all(|v| v.hebrew.is_none()));
    }
}
