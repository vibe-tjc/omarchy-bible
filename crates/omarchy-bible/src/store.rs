//! The reader's state: one `Copy` store of signals, made once by the root
//! and given to the screens through context. Its methods are the only
//! writers; views read through it in bindings.
//!
//! The Bible itself is immutable and shared through a `Stored` handle, so
//! nothing re-renders when it is read.

use std::collections::BTreeSet;
use std::sync::Arc;

use bible_core::{
    Bible, BookEntry, Chapter, SearchHit, Testament, TranslationId, VerseUnit, ViewMode,
    apply_verse_tap, format_selection_summary, format_verse_copy, parse_bible_ref,
};
use nsh_core::prelude::*;

use crate::settings::{
    AppSettings, FONT_MAX, FONT_MIN, ResolvedTheme, ThemePreference, resolve_theme,
};

/// Results shown in the search panel; the total is still reported.
pub const SEARCH_DISPLAY_CAP: usize = 80;
/// 4×4 book grid; the New Testament fits in two pages.
pub const BOOK_GRID_PAGE_SIZE: usize = 16;

/// The open accordion section of the Hebrew word detail (at most one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HebrewSection {
    Strongs,
    Morph,
    Pos,
}

/// The overlay over the reader, if any. One at a time, as in the GPUI app,
/// where opening one closed the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overlay {
    #[default]
    None,
    BookPicker,
    ChapterPicker,
    Search,
    Settings,
}

/// Where the settings are read from and written to.
///
/// The desktop keeps `settings.json` in the config directory, as before, so
/// existing settings carry over. Android has no home directory: there the
/// app keeps them in NSH's app storage, which the platform holds in the
/// app's private data. Tests point at a temporary file.
#[derive(Clone)]
pub enum SettingsFile {
    File(std::path::PathBuf),
    AppStorage,
}

/// The app-storage key holding the settings JSON.
const SETTINGS_KEY: &str = "settings";

impl SettingsFile {
    /// The user's settings on this platform.
    pub fn user() -> Self {
        if cfg!(target_os = "android") {
            Self::AppStorage
        } else {
            Self::File(AppSettings::config_path())
        }
    }

    /// Reads them; defaults when there are none or they do not parse.
    /// Call where components run (app storage is the engine's).
    pub fn load(&self) -> AppSettings {
        match self {
            Self::File(path) => AppSettings::load_from(path),
            Self::AppStorage => nsh_core::storage::Storage::get_item(SETTINGS_KEY)
                .and_then(|json| serde_json::from_str::<AppSettings>(&json).ok())
                .unwrap_or_default()
                .clamp(),
        }
    }

    pub fn save(&self, settings: &AppSettings) {
        let result = match self {
            Self::File(path) => settings.save_to(path).map_err(|e| e.to_string()),
            Self::AppStorage => serde_json::to_string(settings)
                .map_err(|e| e.to_string())
                .and_then(|json| {
                    nsh_core::storage::Storage::set_item(SETTINGS_KEY, json)
                        .map_err(|_| "app storage is full".to_owned())
                }),
        };
        if let Err(err) = result {
            log::warn!("omarchy-bible: failed to save settings: {err}");
        }
    }
}

#[derive(Clone, Copy)]
pub struct Store {
    bible: Stored<Arc<Bible>>,
    file: Stored<SettingsFile>,
    pub book_index: Signal<usize>,
    pub chapter: Signal<Arc<ChapterData>>,
    pub settings: Signal<AppSettings>,
    pub resolved: Signal<ResolvedTheme>,
    pub overlay: Signal<Overlay>,
    pub highlight: Signal<Option<u32>>,
    /// Asks the reader to scroll to this verse; a counter so the same verse
    /// can be asked twice.
    pub scroll_request: Signal<(u64, Option<u32>)>,
    pub select_mode: Signal<bool>,
    pub selected: Signal<BTreeSet<VerseUnit>>,
    pub search_query: Signal<String>,
    pub search_all_lanes: Signal<bool>,
    pub search_hits: Signal<Arc<Vec<SearchHit>>>,
    pub search_total: Signal<usize>,
    pub book_picker_testament: Signal<Testament>,
    pub book_picker_page: Signal<usize>,
    pub jump_draft: Signal<String>,
    pub jump_error: Signal<bool>,
    pub hebrew_verse: Signal<Option<u32>>,
    pub hebrew_word: Signal<Option<usize>>,
    pub hebrew_section: Signal<Option<HebrewSection>>,
    pub copied: Signal<u64>,
    system_dark: Signal<bool>,
}

/// A loaded chapter, wrapped so the signal compares it cheaply.
#[derive(Debug)]
pub struct ChapterData(pub Chapter);

impl PartialEq for ChapterData {
    fn eq(&self, other: &Self) -> bool {
        self.0.book == other.0.book && self.0.chapter == other.0.chapter
    }
}

impl Store {
    pub fn new(bible: Arc<Bible>, file: SettingsFile, system_dark: bool) -> Self {
        let settings = file.load();
        let resolved = resolve_theme(settings.theme, system_dark);
        let first = bible.load_chapter_at(0, 1).expect("load 創世記 1");
        let first_verse = first.verses.first().map(|v| v.number);
        Self {
            bible: stored(bible),
            file: stored(file),
            book_index: signal(0),
            chapter: signal(Arc::new(ChapterData(first))),
            settings: signal(settings),
            resolved: signal(resolved),
            overlay: signal(Overlay::None),
            highlight: signal(None),
            scroll_request: signal((0, first_verse)),
            select_mode: signal(false),
            selected: signal(BTreeSet::new()),
            search_query: signal(String::new()),
            search_all_lanes: signal(false),
            search_hits: signal(Arc::new(Vec::new())),
            search_total: signal(0),
            book_picker_testament: signal(Testament::Old),
            book_picker_page: signal(0),
            jump_draft: signal(String::new()),
            jump_error: signal(false),
            hebrew_verse: signal(None),
            hebrew_word: signal(None),
            hebrew_section: signal(None),
            copied: signal(0),
            system_dark: signal(system_dark),
        }
    }

    pub fn bible(&self) -> Arc<Bible> {
        self.bible.with(|b| b.clone())
    }

    pub fn entry(&self) -> BookEntry {
        let index = self.book_index.get();
        self.bible
            .with(|b| b.book_at(index))
            .expect("book index in range")
    }

    pub fn catalog(&self) -> Vec<BookEntry> {
        self.bible.with(|b| b.catalog())
    }

    pub fn lanes(&self) -> Vec<TranslationId> {
        self.settings.with(|s| s.view_mode.lanes())
    }

    // ── Navigation ────────────────────────────────────────────────────

    /// Opens `chapter` of book `book_index`, scrolled to `verse` (or the
    /// top) with `verse` highlighted. Clears the selection and closes the
    /// pickers.
    pub fn goto(&self, book_index: usize, chapter: u32, verse: Option<u32>) {
        let Ok(loaded) = self.bible.with(|b| b.load_chapter_at(book_index, chapter)) else {
            return;
        };
        let top = verse.or_else(|| loaded.verses.first().map(|v| v.number));
        batch(|| {
            self.book_index.set(book_index);
            self.chapter.set(Arc::new(ChapterData(loaded)));
            self.highlight.set(verse);
            self.scroll_request.update(|(n, v)| {
                *n += 1;
                *v = top;
            });
            self.selected.set(BTreeSet::new());
            if matches!(
                self.overlay.get_untracked(),
                Overlay::BookPicker | Overlay::ChapterPicker
            ) {
                self.overlay.set(Overlay::None);
            }
        });
    }

    pub fn goto_chapter(&self, chapter: u32) {
        self.goto(self.book_index.get_untracked(), chapter, None);
    }

    /// The previous chapter, crossing into the previous book at chapter 1.
    pub fn prev_chapter(&self) {
        let chapter = self.chapter.with_untracked(|c| c.0.chapter);
        let index = self.book_index.get_untracked();
        if chapter > 1 {
            self.goto(index, chapter - 1, None);
        } else if index > 0
            && let Some(prev) = self.bible.with(|b| b.book_at(index - 1))
        {
            self.goto(index - 1, prev.chapter_count, None);
        }
    }

    /// The next chapter, crossing into the next book after the last one.
    pub fn next_chapter(&self) {
        let chapter = self.chapter.with_untracked(|c| c.0.chapter);
        let index = self.book_index.get_untracked();
        let (count, books) = self
            .bible
            .with(|b| (b.book_at(index).map_or(0, |e| e.chapter_count), b.len()));
        if chapter < count {
            self.goto(index, chapter + 1, None);
        } else if index + 1 < books {
            self.goto(index + 1, 1, None);
        }
    }

    /// Jumps to the typed reference (`1:3:5`, `約3:16`, `John 3:16`). Leaves
    /// the draft and flags it when it does not parse.
    pub fn submit_jump(&self) {
        let raw = self.jump_draft.get_untracked();
        let Some(parsed) = parse_bible_ref(&raw) else {
            self.jump_error.set(!raw.trim().is_empty());
            return;
        };
        // Prefer the OSIS match in case catalog order ever drifts.
        let found = self.bible.with(|b| {
            b.catalog()
                .into_iter()
                .find(|e| e.meta.osis == parsed.book.osis)
        });
        let Some(entry) = found else {
            self.jump_error.set(true);
            return;
        };
        let chapter = parsed.chapter.clamp(1, entry.chapter_count.max(1));
        batch(|| {
            self.jump_error.set(false);
            self.goto(entry.index, chapter, parsed.verse);
        });
    }

    pub fn set_jump_draft(&self, text: &str) {
        batch(|| {
            self.jump_draft.set(text.to_owned());
            self.jump_error.set(false);
        });
    }

    // ── Overlays ──────────────────────────────────────────────────────

    pub fn toggle_overlay(&self, overlay: Overlay) {
        if self.overlay.get_untracked() == overlay {
            self.overlay.set(Overlay::None);
        } else {
            self.open_overlay(overlay);
        }
    }

    pub fn open_overlay(&self, overlay: Overlay) {
        if overlay == Overlay::BookPicker {
            let entry = self
                .bible
                .with(|b| b.book_at(self.book_index.get_untracked()));
            if let Some(entry) = entry {
                let books = self.books_of(entry.meta.testament);
                let pos = books
                    .iter()
                    .position(|e| e.index == entry.index)
                    .unwrap_or(0);
                batch(|| {
                    self.book_picker_testament.set(entry.meta.testament);
                    self.book_picker_page.set(pos / BOOK_GRID_PAGE_SIZE);
                });
            }
        }
        self.overlay.set(overlay);
    }

    pub fn close_overlay(&self) {
        self.overlay.set(Overlay::None);
    }

    /// Whether Escape or Back has anything to close.
    pub fn can_escape(&self) -> bool {
        self.hebrew_verse.get_untracked().is_some()
            || self.select_mode.get_untracked()
            || self.overlay.get_untracked() != Overlay::None
    }

    /// Escape: the Hebrew page, then the selection, then the overlay.
    pub fn escape(&self) {
        if self.hebrew_verse.get_untracked().is_some() {
            self.close_hebrew();
        } else if self.select_mode.get_untracked() {
            self.exit_select_mode();
        } else {
            self.close_overlay();
        }
    }

    // ── Book picker ───────────────────────────────────────────────────

    pub fn books_of(&self, testament: Testament) -> Vec<BookEntry> {
        self.bible.with(|b| {
            b.catalog()
                .into_iter()
                .filter(|e| e.meta.testament == testament)
                .collect()
        })
    }

    pub fn book_pages(&self) -> usize {
        let testament = self.book_picker_testament.get();
        self.books_of(testament)
            .len()
            .div_ceil(BOOK_GRID_PAGE_SIZE)
            .max(1)
    }

    /// The books on the shown page, padded to the full grid with `None`.
    pub fn book_page(&self) -> Vec<Option<BookEntry>> {
        let testament = self.book_picker_testament.get();
        let books = self.books_of(testament);
        let pages = books.len().div_ceil(BOOK_GRID_PAGE_SIZE).max(1);
        let page = self.book_picker_page.get().min(pages - 1);
        let mut cells: Vec<Option<BookEntry>> = books
            .into_iter()
            .skip(page * BOOK_GRID_PAGE_SIZE)
            .take(BOOK_GRID_PAGE_SIZE)
            .map(Some)
            .collect();
        cells.resize(BOOK_GRID_PAGE_SIZE, None);
        cells
    }

    pub fn set_book_testament(&self, testament: Testament) {
        batch(|| {
            self.book_picker_testament.set(testament);
            self.book_picker_page.set(0);
        });
    }

    pub fn shift_book_page(&self, delta: i32) {
        let pages = self.book_pages() as i32;
        let next = (self.book_picker_page.get_untracked() as i32 + delta).rem_euclid(pages);
        self.book_picker_page.set(next as usize);
    }

    // ── Selection ─────────────────────────────────────────────────────

    pub fn toggle_select_mode(&self) {
        if self.select_mode.get_untracked() {
            self.exit_select_mode();
        } else {
            batch(|| {
                self.select_mode.set(true);
                self.selected.set(BTreeSet::new());
            });
        }
    }

    pub fn exit_select_mode(&self) {
        batch(|| {
            self.select_mode.set(false);
            self.selected.set(BTreeSet::new());
        });
    }

    /// A tap on one lane of one verse in select mode (range rules in
    /// `bible_core::apply_verse_tap`).
    pub fn tap_verse(&self, number: u32, translation: TranslationId) {
        if !self.select_mode.get_untracked() || !self.lanes_untracked().contains(&translation) {
            return;
        }
        let numbers: Vec<u32> = self
            .chapter
            .with_untracked(|c| c.0.verses.iter().map(|v| v.number).collect());
        self.selected
            .update(|set| apply_verse_tap(set, number, translation, &numbers));
    }

    pub fn selection_summary(&self) -> String {
        let units: Vec<VerseUnit> = self.selected.with(|s| s.iter().copied().collect());
        self.chapter
            .with(|c| format_selection_summary(&c.0, &units))
    }

    /// Copies the selection, keeping only lanes still shown.
    pub fn copy_selection(&self) {
        let lanes = self.lanes_untracked();
        let units: Vec<VerseUnit> = self.selected.with_untracked(|s| {
            s.iter()
                .copied()
                .filter(|u| lanes.contains(&u.translation))
                .collect()
        });
        if units.is_empty() {
            return;
        }
        let text = self
            .chapter
            .with_untracked(|c| format_verse_copy(&c.0, &units));
        set_clipboard(text);
        self.copied.update(|n| *n += 1);
    }

    fn lanes_untracked(&self) -> Vec<TranslationId> {
        self.settings.with_untracked(|s| s.view_mode.lanes())
    }

    // ── Search ────────────────────────────────────────────────────────

    pub fn set_search_query(&self, text: &str) {
        self.search_query.set(text.to_owned());
        if text.trim().is_empty() {
            batch(|| {
                self.search_hits.set(Arc::new(Vec::new()));
                self.search_total.set(0);
            });
        }
    }

    pub fn set_search_all_lanes(&self, all: bool) {
        self.search_all_lanes.set(all);
        self.run_search();
    }

    /// Searches now (the panel debounces typing before calling this).
    pub fn run_search(&self) {
        let query = self.search_query.get_untracked();
        if query.trim().is_empty() {
            return;
        }
        let lanes = if self.search_all_lanes.get_untracked() {
            TranslationId::ALL.to_vec()
        } else {
            self.lanes_untracked()
        };
        let hits = self.bible.with(|b| b.search(&query, &lanes));
        batch(|| {
            self.search_total.set(hits.len());
            self.search_hits.set(Arc::new(
                hits.into_iter().take(SEARCH_DISPLAY_CAP).collect(),
            ));
        });
    }

    pub fn open_hit(&self, hit: &SearchHit) {
        batch(|| {
            self.goto(hit.book_index, hit.chapter, Some(hit.verse));
            self.close_overlay();
        });
    }

    pub fn open_first_hit(&self) {
        let first = self.search_hits.with_untracked(|h| h.first().cloned());
        if let Some(hit) = first {
            self.open_hit(&hit);
        }
    }

    // ── Hebrew ────────────────────────────────────────────────────────

    pub fn open_hebrew(&self, verse: u32) {
        batch(|| {
            self.hebrew_verse.set(Some(verse));
            self.hebrew_word.set(None);
            self.hebrew_section.set(None);
            self.overlay.set(Overlay::None);
        });
    }

    /// Back to the reader, highlighting and scrolling to the verse.
    pub fn close_hebrew(&self) {
        let verse = self.hebrew_verse.get_untracked();
        batch(|| {
            self.hebrew_verse.set(None);
            self.hebrew_word.set(None);
            self.hebrew_section.set(None);
            if let Some(n) = verse {
                self.highlight.set(Some(n));
                self.scroll_request.update(|(c, v)| {
                    *c += 1;
                    *v = Some(n);
                });
            }
        });
    }

    /// The words of the open Hebrew verse.
    pub fn hebrew_words(&self) -> Vec<bible_core::HebrewWord> {
        let Some(n) = self.hebrew_verse.get() else {
            return Vec::new();
        };
        self.chapter.with(|c| {
            c.0.verses
                .iter()
                .find(|v| v.number == n)
                .and_then(|v| v.hebrew.clone())
                .unwrap_or_default()
        })
    }

    /// A tap on a word column: selects it (opening its default section), or
    /// deselects it when it is already selected.
    pub fn select_hebrew_word(&self, index: usize) {
        if self.hebrew_word.get_untracked() == Some(index) {
            batch(|| {
                self.hebrew_word.set(None);
                self.hebrew_section.set(None);
            });
            return;
        }
        let words = untracked(|| self.hebrew_words());
        let section = words.get(index).and_then(default_section);
        batch(|| {
            self.hebrew_word.set(Some(index));
            self.hebrew_section.set(section);
        });
    }

    pub fn toggle_hebrew_section(&self, section: HebrewSection) {
        self.hebrew_section.update(|s| {
            *s = if *s == Some(section) {
                None
            } else {
                Some(section)
            };
        });
    }

    // ── Settings and theme ────────────────────────────────────────────

    fn update_settings(&self, change: impl FnOnce(&mut AppSettings)) {
        let mut next = self.settings.get_untracked();
        change(&mut next);
        let next = next.clamp();
        self.file.with(|f| f.save(&next));
        let lanes = next.view_mode.lanes();
        let resolved = resolve_theme(next.theme, self.system_dark.get_untracked());
        batch(|| {
            self.settings.set(next);
            self.resolved.set(resolved);
            // Hidden lanes cannot stay selected.
            self.selected
                .update(|s| s.retain(|u| lanes.contains(&u.translation)));
        });
    }

    pub fn set_theme(&self, theme: ThemePreference) {
        self.update_settings(|s| s.theme = theme);
    }

    pub fn set_font_size(&self, size: u32) {
        self.update_settings(|s| s.font_size = size);
    }

    pub fn bump_font(&self, delta: i32) {
        let current = self.settings.with_untracked(|s| s.font_size) as i32;
        self.set_font_size((current + delta).clamp(FONT_MIN as i32, FONT_MAX as i32) as u32);
    }

    pub fn set_view_mode(&self, mode: ViewMode) {
        self.update_settings(|s| s.view_mode = mode.sanitized());
    }

    /// Re-reads the system appearance and the Omarchy theme; updates the
    /// resolved theme only when it changed.
    pub fn refresh_system_theme(&self, system_dark: bool) {
        self.system_dark.set(system_dark);
        let pref = self.settings.with_untracked(|s| s.theme);
        if pref != ThemePreference::System {
            return;
        }
        self.resolved.set(resolve_theme(pref, system_dark));
    }
}

/// Strong's when it has one, else the form; nothing when it has neither.
fn default_section(word: &bible_core::HebrewWord) -> Option<HebrewSection> {
    let present = |s: &Option<String>| s.as_deref().is_some_and(|s| !s.trim().is_empty());
    if present(&word.strongs) {
        Some(HebrewSection::Strongs)
    } else if present(&word.morph) {
        Some(HebrewSection::Morph)
    } else {
        None
    }
}

/// The store the root provided. Read while building, never in a listener.
pub fn store() -> Store {
    use_context::<Store>().expect("the root provides the store")
}
