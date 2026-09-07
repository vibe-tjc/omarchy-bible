//! gpui-omarchy Bible view: top-bar book picker + translation-lane chapter pane.

mod settings;

use bible_core::{
    Bible, BookEntry, Chapter, HebrewWord, SearchHit, Testament, TranslationId, VerseUnit,
    ViewMode, apply_verse_tap, format_selection_summary, format_verse_copy, parse_bible_ref,
};
use gpui_omarchy::gpui::base::input::{InputEvent, InputState};
use gpui_omarchy::gpui::base::{h_flex, v_flex};
use gpui_omarchy::gpui::{
    App, Bounds, ClickEvent, ClipboardItem, Context, Entity, FocusHandle, Focusable, FontWeight,
    KeyBinding, MouseButton, ScrollHandle, SharedString, Subscription, TitlebarOptions, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, rgb, rgba, size,
};
use gpui_omarchy::{input, with_tooltip, ActiveTheme};
use settings::{
    AppSettings, FONT_LARGE, FONT_MEDIUM, FONT_SMALL, Palette, ThemePreference,
    apply_theme_preference, palette_from_app, settings_location_note_zh,
};
use std::collections::BTreeSet;
use std::time::Duration;

gpui_omarchy::gpui::actions!(
    omarchy_bible,
    [
        Quit,
        PrevChapter,
        NextChapter,
        ToggleSettings,
        CloseSettings,
        OpenSearch,
        ToggleSelectMode
    ]
);

const SEARCH_DEBOUNCE_MS: u64 = 180;
const SEARCH_DISPLAY_CAP: usize = 80;
const BOOK_GRID_PAGE_SIZE: usize = 16; // 4x4; NT fits in two pages.

/// Exclusive accordion section inside the Hebrew 「詞詳情」 panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HebrewDetailSection {
    Strongs,
    Morph,
    Pos,
}


#[cfg(target_os = "macos")]
const CJK_FONT_CANDIDATES: &[&str] = &[
    "PingFang TC",
    "Songti TC",
    "Heiti TC",
    "Noto Serif CJK TC",
    "Noto Sans CJK TC",
    "Source Han Serif TC",
    "Source Han Sans TC",
    "Noto Serif CJK",
    "Noto Sans CJK",
];

#[cfg(not(target_os = "macos"))]
const CJK_FONT_CANDIDATES: &[&str] = &[
    "Noto Serif CJK TC",
    "Noto Sans CJK TC",
    "Source Han Serif TC",
    "Source Han Sans TC",
    "Noto Serif CJK",
    "Noto Sans CJK",
];

pub fn run_app(bible: Bible) {
    gpui_omarchy::application().run(move |cx: &mut App| {
        gpui_omarchy::init(cx);

        let startup = AppSettings::load();
        apply_theme_preference(startup.theme, None, cx);

        let font_family = pick_cjk_font(cx);

        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([
            KeyBinding::new("ctrl-q", Quit, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("[", PrevChapter, Some("omarchy_bible")),
            KeyBinding::new("]", NextChapter, Some("omarchy_bible")),
            KeyBinding::new("/", OpenSearch, Some("omarchy_bible")),
            KeyBinding::new("escape", CloseSettings, Some("omarchy_bible")),
            KeyBinding::new("escape", CloseSettings, Some("omarchy_bible_search")),
        ]);

        let bounds = Bounds::centered(None, size(px(980.0), px(760.0)), cx);
        let font_for_view = font_family;
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::from("聖經")),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            window_min_size: Some(size(px(640.0), px(480.0))),
            app_id: Some("omarchy-bible".into()),
            focus: true,
            show: true,
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, {
                move |window, cx| {
                    let view =
                        cx.new(|cx| BibleView::new(bible, font_for_view.clone(), window, cx));
                    let focus = view.read(cx).focus_handle.clone();
                    window.focus(&focus, cx);
                    view
                }
            })
            .expect("open window");
        })
        .detach();

        cx.activate(true);
    });
}

fn pick_cjk_font(cx: &App) -> SharedString {
    let installed = cx.text_system().all_font_names();
    for name in CJK_FONT_CANDIDATES {
        if installed.iter().any(|n| n.eq_ignore_ascii_case(name)) {
            return SharedString::from(*name);
        }
    }
    for installed_name in &installed {
        if installed_name.contains("CJK TC")
            || installed_name.contains("Noto Serif CJK")
            || installed_name.contains("PingFang")
            || installed_name.contains("Songti TC")
            || installed_name.contains("Heiti TC")
        {
            return SharedString::from(installed_name.clone());
        }
    }
    SharedString::from("sans-serif")
}

pub struct BibleView {
    bible: Bible,
    book_index: usize,
    chapter: Chapter,
    font_family: SharedString,
    focus_handle: FocusHandle,
    settings: AppSettings,
    settings_open: bool,
    search_open: bool,
    search_input: Entity<InputState>,
    jump_input: Entity<InputState>,
    search_hits: Vec<SearchHit>,
    search_total: usize,
    search_all_lanes: bool,
    search_seq: u64,
    highlight_verse: Option<u32>,
    pending_scroll_verse: Option<u32>,
    chapter_scroll: ScrollHandle,
    chapter_picker_open: bool,
    book_picker_open: bool,
    book_picker_testament: Testament,
    book_picker_page: usize,
    hebrew_verse: Option<u32>,
    /// Selected MorphHB word index on the Hebrew annotation page (tap to expand).
    hebrew_word_idx: Option<usize>,
    /// Which 「詞詳情」 accordion section is open (at most one).
    hebrew_detail_section: Option<HebrewDetailSection>,
    select_mode: bool,
    selected_verses: BTreeSet<VerseUnit>,
    _subscriptions: Vec<Subscription>,
}

impl BibleView {
    pub fn new(
        bible: Bible,
        font_family: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let book_index = 0;
        let chapter = bible.load_chapter_at(book_index, 1).expect("load 創世記 1");
        let settings = AppSettings::load();
        apply_theme_preference(settings.theme, Some(window), cx);

        let search_input = cx.new(|cx| InputState::new(window, cx).placeholder("搜尋經文…"));
        let jump_input = cx.new(|cx| InputState::new(window, cx).placeholder("1:3:5 / 約3:16"));

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.observe_window_appearance(window, |this, window, cx| {
            if this.settings.theme == ThemePreference::System {
                this.recompute_theme(Some(window), cx);
                cx.notify();
            }
        }));
        subscriptions.push(cx.subscribe_in(
            &search_input,
            window,
            |this, _input, event, window, cx| match event {
                InputEvent::Change => this.on_search_input_change(cx),
                InputEvent::PressEnter { .. } => this.jump_first_search_hit(window, cx),
                _ => {}
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &jump_input,
            window,
            |this, _input, event, window, cx| match event {
                InputEvent::PressEnter { .. } => this.submit_jump(window, cx),
                _ => {}
            },
        ));

        Self {
            bible,
            book_index,
            chapter,
            font_family,
            focus_handle: cx.focus_handle(),
            settings,
            settings_open: false,
            search_open: false,
            search_input,
            jump_input,
            search_hits: Vec::new(),
            search_total: 0,
            search_all_lanes: false,
            search_seq: 0,
            highlight_verse: None,
            pending_scroll_verse: None,
            chapter_scroll: ScrollHandle::new(),
            chapter_picker_open: false,
            book_picker_open: false,
            book_picker_testament: Testament::Old,
            book_picker_page: 0,
            hebrew_verse: None,
            hebrew_word_idx: None,
            hebrew_detail_section: None,
            select_mode: false,
            selected_verses: BTreeSet::new(),
            _subscriptions: subscriptions,
        }
    }

    fn recompute_theme(&mut self, window: Option<&mut Window>, cx: &mut Context<Self>) {
        apply_theme_preference(self.settings.theme, window.as_deref(), cx);
    }

    fn persist_and_refresh(&mut self, window: Option<&mut Window>, cx: &mut Context<Self>) {
        self.settings = self.settings.clone().clamp();
        self.settings.save();
        self.recompute_theme(window, cx);
        cx.notify();
    }

    fn current_entry(&self) -> BookEntry {
        self.bible
            .book_at(self.book_index)
            .expect("current book index in range")
    }

    fn goto(&mut self, book_index: usize, chapter: u32, cx: &mut Context<Self>) {
        self.goto_internal(book_index, chapter, None, cx);
    }

    fn goto_internal(
        &mut self,
        book_index: usize,
        chapter: u32,
        highlight: Option<u32>,
        cx: &mut Context<Self>,
    ) {
        if let Ok(loaded) = self.bible.load_chapter_at(book_index, chapter) {
            self.book_index = book_index;
            self.chapter = loaded;
            self.highlight_verse = highlight;
            self.pending_scroll_verse = highlight;
            self.selected_verses.clear();
            self.chapter_picker_open = false;
            self.book_picker_open = false;
            cx.notify();
        }
    }

    fn select_book(&mut self, index: usize, cx: &mut Context<Self>) {
        self.goto(index, 1, cx);
    }

    fn goto_chapter(&mut self, chapter: u32, cx: &mut Context<Self>) {
        self.goto(self.book_index, chapter, cx);
    }

    fn quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn prev_chapter(&mut self, _: &PrevChapter, _window: &mut Window, cx: &mut Context<Self>) {
        if self.search_open {
            return;
        }
        if self.chapter.chapter > 1 {
            self.goto_chapter(self.chapter.chapter - 1, cx);
            return;
        }
        if self.book_index > 0 {
            let prev = self.book_index - 1;
            if let Some(entry) = self.bible.book_at(prev) {
                self.goto(prev, entry.chapter_count, cx);
            }
        }
    }

    fn next_chapter(&mut self, _: &NextChapter, _window: &mut Window, cx: &mut Context<Self>) {
        if self.search_open {
            return;
        }
        let entry = self.current_entry();
        if self.chapter.chapter < entry.chapter_count {
            self.goto_chapter(self.chapter.chapter + 1, cx);
            return;
        }
        if self.book_index + 1 < self.bible.len() {
            self.goto(self.book_index + 1, 1, cx);
        }
    }

    fn toggle_settings(
        &mut self,
        _: &ToggleSettings,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings_open = !self.settings_open;
        cx.notify();
    }

    fn close_settings(&mut self, _: &CloseSettings, window: &mut Window, cx: &mut Context<Self>) {
        if self.hebrew_verse.is_some() {
            self.close_hebrew_view(window, cx);
            return;
        }
        if self.select_mode {
            self.exit_select_mode(cx);
            return;
        }
        if self.book_picker_open {
            self.book_picker_open = false;
            window.focus(&self.focus_handle, cx);
            cx.notify();
            return;
        }
        if self.chapter_picker_open {
            self.chapter_picker_open = false;
            window.focus(&self.focus_handle, cx);
            cx.notify();
            return;
        }
        if self.search_open {
            self.close_search(window, cx);
            return;
        }
        if self.settings_open {
            self.settings_open = false;
            window.focus(&self.focus_handle, cx);
            cx.notify();
        }
    }

    fn toggle_select_mode_action(
        &mut self,
        _: &ToggleSelectMode,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_select_mode(cx);
    }

    fn toggle_select_mode(&mut self, cx: &mut Context<Self>) {
        if self.select_mode {
            self.exit_select_mode(cx);
        } else {
            self.select_mode = true;
            self.selected_verses.clear();
            cx.notify();
        }
    }

    fn exit_select_mode(&mut self, cx: &mut Context<Self>) {
        self.select_mode = false;
        self.selected_verses.clear();
        cx.notify();
    }

    fn on_verse_select(&mut self, number: u32, translation: TranslationId, cx: &mut Context<Self>) {
        if !self.select_mode {
            return;
        }
        let lanes = self.settings.view_mode.lanes();
        if !lanes.contains(&translation) {
            return;
        }
        let verse_numbers: Vec<u32> = self.chapter.verses.iter().map(|v| v.number).collect();
        apply_verse_tap(
            &mut self.selected_verses,
            number,
            translation,
            &verse_numbers,
        );
        cx.notify();
    }

    fn prune_hidden_selection(&mut self) {
        let lanes = self.settings.view_mode.lanes();
        self.selected_verses
            .retain(|u| lanes.contains(&u.translation));
    }

    fn copy_selection(&mut self, cx: &mut Context<Self>) {
        self.prune_hidden_selection();
        if self.selected_verses.is_empty() {
            return;
        }
        let units: Vec<VerseUnit> = self.selected_verses.iter().copied().collect();
        let text = format_verse_copy(&self.chapter, &units);
        cx.write_to_clipboard(ClipboardItem::new_string(text));
    }

    fn open_search_action(&mut self, _: &OpenSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.open_search(window, cx);
    }

    fn open_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search_open = true;
        self.search_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        cx.notify();
    }

    fn close_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.search_open {
            return;
        }
        self.search_open = false;
        self.search_seq = self.search_seq.wrapping_add(1);
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn search_lanes(&self) -> Vec<TranslationId> {
        if self.search_all_lanes {
            TranslationId::ALL.to_vec()
        } else {
            self.settings.view_mode.lanes()
        }
    }

    fn on_search_input_change(&mut self, cx: &mut Context<Self>) {
        let raw = self.search_input.read(cx).value().to_string();
        if raw.trim().is_empty() {
            self.search_seq = self.search_seq.wrapping_add(1);
            self.search_hits.clear();
            self.search_total = 0;
            cx.notify();
            return;
        }
        self.search_seq = self.search_seq.wrapping_add(1);
        let seq = self.search_seq;
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(SEARCH_DEBOUNCE_MS))
                .await;
            this.update(cx, |this, cx| {
                if this.search_seq != seq || !this.search_open {
                    return;
                }
                this.run_search(cx);
            })
            .ok();
        })
        .detach();
    }

    fn run_search(&mut self, cx: &mut Context<Self>) {
        let query = self.search_input.read(cx).value().to_string();
        let lanes = self.search_lanes();
        let hits = self.bible.search(&query, &lanes);
        self.search_total = hits.len();
        self.search_hits = hits.into_iter().take(SEARCH_DISPLAY_CAP).collect();
        cx.notify();
    }

    fn jump_first_search_hit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(hit) = self.search_hits.first().cloned() else {
            return;
        };
        self.jump_to_search_hit(&hit, window, cx);
    }

    fn jump_to_search_hit(&mut self, hit: &SearchHit, window: &mut Window, cx: &mut Context<Self>) {
        self.goto_internal(hit.book_index, hit.chapter, Some(hit.verse), cx);
        self.search_open = false;
        self.search_seq = self.search_seq.wrapping_add(1);
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn on_book_click(
        &mut self,
        index: usize,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_book(index, cx);
    }

    fn on_chapter_click(
        &mut self,
        number: u32,
        _: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.goto_chapter(number, cx);
    }

    fn submit_jump(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let raw = self.jump_input.read(cx).value().to_string();
        let Some(parsed) = parse_bible_ref(&raw) else {
            return;
        };
        let book_index = (parsed.book.book_id as usize).saturating_sub(1);
        let Some(entry) = self.bible.book_at(book_index) else {
            return;
        };
        if entry.meta.osis != parsed.book.osis {
            // Prefer OSIS match if catalog order ever drifts.
            let Some(idx) = self
                .bible
                .catalog()
                .iter()
                .find(|e| e.meta.osis == parsed.book.osis)
                .map(|e| e.index)
            else {
                return;
            };
            let entry = self.bible.book_at(idx).expect("osis book");
            let chapter = parsed.chapter.clamp(1, entry.chapter_count.max(1));
            self.goto_internal(idx, chapter, parsed.verse, cx);
        } else {
            let chapter = parsed.chapter.clamp(1, entry.chapter_count.max(1));
            self.goto_internal(book_index, chapter, parsed.verse, cx);
        }
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn toggle_chapter_picker(&mut self, cx: &mut Context<Self>) {
        self.chapter_picker_open = !self.chapter_picker_open;
        if self.chapter_picker_open {
            self.book_picker_open = false;
        }
        cx.notify();
    }

    fn open_book_picker(&mut self, cx: &mut Context<Self>) {
        if self.book_picker_open {
            self.book_picker_open = false;
            cx.notify();
            return;
        }
        let entry = self.current_entry();
        self.book_picker_testament = entry.meta.testament;
        let books: Vec<_> = self
            .bible
            .catalog()
            .into_iter()
            .filter(|e| e.meta.testament == self.book_picker_testament)
            .collect();
        let pos = books
            .iter()
            .position(|e| e.index == self.book_index)
            .unwrap_or(0);
        self.book_picker_page = pos / BOOK_GRID_PAGE_SIZE;
        self.book_picker_open = true;
        self.chapter_picker_open = false;
        cx.notify();
    }

    fn set_book_picker_testament(&mut self, testament: Testament, cx: &mut Context<Self>) {
        self.book_picker_testament = testament;
        self.book_picker_page = 0;
        cx.notify();
    }

    fn book_picker_page_count(&self) -> usize {
        let n = self
            .bible
            .catalog()
            .into_iter()
            .filter(|e| e.meta.testament == self.book_picker_testament)
            .count();
        n.div_ceil(BOOK_GRID_PAGE_SIZE).max(1)
    }

    fn shift_book_picker_page(&mut self, delta: i32, cx: &mut Context<Self>) {
        let pages = self.book_picker_page_count() as i32;
        let next = (self.book_picker_page as i32 + delta).rem_euclid(pages) as usize;
        self.book_picker_page = next;
        cx.notify();
    }

    fn open_hebrew_view(&mut self, verse: u32, cx: &mut Context<Self>) {
        self.hebrew_verse = Some(verse);
        self.hebrew_word_idx = None;
        self.hebrew_detail_section = None;
        self.chapter_picker_open = false;
        self.book_picker_open = false;
        cx.notify();
    }

    fn close_hebrew_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(n) = self.hebrew_verse.take() {
            self.highlight_verse = Some(n);
            self.pending_scroll_verse = Some(n);
        }
        self.hebrew_word_idx = None;
        self.hebrew_detail_section = None;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn select_hebrew_word(&mut self, idx: usize, cx: &mut Context<Self>) {
        match self.hebrew_word_idx {
            Some(cur) if cur == idx => {
                self.hebrew_word_idx = None;
                self.hebrew_detail_section = None;
            }
            _ => {
                self.hebrew_word_idx = Some(idx);
                self.hebrew_detail_section = self
                    .hebrew_verse
                    .and_then(|n| self.chapter.verses.iter().find(|v| v.number == n))
                    .and_then(|v| v.hebrew.as_ref())
                    .and_then(|words| words.get(idx))
                    .and_then(default_hebrew_detail_section);
            }
        }
        cx.notify();
    }

    fn toggle_hebrew_detail_section(
        &mut self,
        section: HebrewDetailSection,
        cx: &mut Context<Self>,
    ) {
        self.hebrew_detail_section = match self.hebrew_detail_section {
            Some(cur) if cur == section => None,
            _ => Some(section),
        };
        cx.notify();
    }

    fn set_theme_pref(
        &mut self,
        pref: ThemePreference,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings.theme = pref;
        self.persist_and_refresh(Some(window), cx);
    }

    fn set_font_size(&mut self, size: u32, window: &mut Window, cx: &mut Context<Self>) {
        self.settings.font_size = size;
        self.persist_and_refresh(Some(window), cx);
    }

    fn bump_font(&mut self, delta: i32, window: &mut Window, cx: &mut Context<Self>) {
        let next = (self.settings.font_size as i32 + delta)
            .clamp(settings::FONT_MIN as i32, settings::FONT_MAX as i32) as u32;
        self.set_font_size(next, window, cx);
    }

    fn set_view_mode(&mut self, mode: ViewMode, window: &mut Window, cx: &mut Context<Self>) {
        self.settings.view_mode = mode.sanitized();
        self.prune_hidden_selection();
        self.persist_and_refresh(Some(window), cx);
    }
}

impl Focusable for BibleView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BibleView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chapter = &self.chapter;
        let lanes_label = self.settings.view_mode.header_label();
        let current_chapter = chapter.chapter;
        let entry = self.current_entry();
        let chapter_count = entry.chapter_count;
        let catalog = self.bible.catalog();
        let current_index = self.book_index;
        let book_chip_label = format!(
            "{} · {}",
            entry.meta.testament.label_zh(),
            entry.meta.name_zh_short,
        );
        if let Some(n) = self.pending_scroll_verse.take() {
            if let Some(idx) = self.chapter.verses.iter().position(|v| v.number == n) {
                self.chapter_scroll.scroll_to_top_of_item(idx);
            }
        }

        let palette = palette_from_app(cx);
        let settings_open = self.settings_open;
        let search_open = self.search_open;
        let key_ctx = if search_open {
            "omarchy_bible_search"
        } else {
            "omarchy_bible"
        };

        if self.hebrew_verse.is_some() {
            return div()
                .id("bible-root")
                .relative()
                .key_context(key_ctx)
                .track_focus(&self.focus_handle)
                .on_action(cx.listener(Self::quit))
                .on_action(cx.listener(Self::close_settings))
                .size_full()
                .bg(rgb(palette.bg))
                .text_color(rgb(palette.fg))
                .font_family(self.font_family.clone())
                .child(self.render_hebrew_view(palette, cx))
                .into_any_element();
        }

        div()
            .id("bible-root")
            .relative()
            .key_context(key_ctx)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::quit))
            .on_action(cx.listener(Self::prev_chapter))
            .on_action(cx.listener(Self::next_chapter))
            .on_action(cx.listener(Self::toggle_settings))
            .on_action(cx.listener(Self::close_settings))
            .on_action(cx.listener(Self::open_search_action))
            .on_action(cx.listener(Self::toggle_select_mode_action))
            .size_full()
            .bg(rgb(palette.bg))
            .text_color(rgb(palette.fg))
            .font_family(self.font_family.clone())
            .child(self.render_main(
                lanes_label,
                current_chapter,
                chapter_count,
                book_chip_label,
                palette,
                window,
                cx,
            ))
            .when(settings_open, |d| {
                d.child(self.render_settings_overlay(palette, cx))
            })
            .when(search_open, |d| {
                d.child(self.render_search_overlay(palette, window, cx))
            })
            .when(self.chapter_picker_open, |d| {
                d.child(self.render_chapter_picker_overlay(
                    chapter_count,
                    current_chapter,
                    palette,
                    cx,
                ))
            })
            .when(self.book_picker_open, |d| {
                d.child(self.render_book_picker_overlay(&catalog, current_index, palette, cx))
            })
            .into_any_element()
    }
}

impl BibleView {

    fn render_book_picker_overlay(
        &self,
        catalog: &[BookEntry],
        current_index: usize,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let testament = self.book_picker_testament;
        let books: Vec<&BookEntry> = catalog
            .iter()
            .filter(|e| e.meta.testament == testament)
            .collect();
        let pages = books.len().div_ceil(BOOK_GRID_PAGE_SIZE).max(1);
        let page = self.book_picker_page.min(pages - 1);
        let start = page * BOOK_GRID_PAGE_SIZE;
        let page_books = books
            .into_iter()
            .skip(start)
            .take(BOOK_GRID_PAGE_SIZE)
            .collect::<Vec<_>>();

        let mut cells: Vec<gpui_omarchy::gpui::AnyElement> = page_books
            .into_iter()
            .map(|entry| {
                let index = entry.index;
                let is_current = index == current_index;
                let short = entry.meta.name_zh_short;
                let full = entry.meta.name_zh;
                with_tooltip(
                    div()
                        .id(("picker-book", index))
                        .w(px(72.0))
                        .h(px(44.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_md()
                        .text_sm()
                        .when(is_current, |d| {
                            d.bg(rgb(palette.selected_bg()))
                                .text_color(rgb(palette.selected_fg()))
                                .border_1()
                                .border_color(rgb(palette.selected_border()))
                        })
                        .when(!is_current, |d| {
                            d.text_color(rgb(palette.fg_muted))
                                .hover(|s| s.bg(rgb(palette.selected_bg())))
                        })
                        .on_click(cx.listener(move |this, event, window, cx| {
                            this.book_picker_open = false;
                            this.on_book_click(index, event, window, cx);
                        }))
                        .child(short),
                    full,
                )
                .into_any_element()
            })
            .collect();
        while cells.len() < BOOK_GRID_PAGE_SIZE {
            let pad = cells.len();
            cells.push(
                div()
                    .id(("picker-book-pad", pad))
                    .w(px(72.0))
                    .h(px(44.0))
                    .into_any_element(),
            );
        }

        let dots: Vec<gpui_omarchy::gpui::AnyElement> = (0..pages)
            .map(|i| {
                let active = i == page;
                div()
                    .id(("book-page-dot", i))
                    .w(px(8.0))
                    .h(px(8.0))
                    .rounded_md()
                    .bg(rgb(if active {
                        palette.selected_fg()
                    } else {
                        palette.selected_border()
                    }))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.book_picker_page = i;
                        cx.notify();
                    }))
                    .into_any_element()
            })
            .collect();

        div()
            .id("book-picker-overlay")
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            .items_center()
            .pt(px(96.0))
            .bg(rgba(0x00000066))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.book_picker_open = false;
                    window.focus(&this.focus_handle, cx);
                    cx.notify();
                }),
            )
            .child(
                v_flex()
                    .id("book-picker-panel")
                    .w(px(360.0))
                    .bg(rgb(palette.bg_sidebar))
                    .border_1()
                    .border_color(rgb(palette.border))
                    .rounded_md()
                    .px_4()
                    .py_3()
                    .gap_3()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(palette.fg_primary))
                                    .child("選擇書卷"),
                            )
                            .child(chip(
                                "book-picker-close",
                                "關閉",
                                false,
                                palette,
                                cx.listener(|this, _, window, cx| {
                                    this.book_picker_open = false;
                                    window.focus(&this.focus_handle, cx);
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(chip(
                                "book-tab-ot",
                                "舊約",
                                testament == Testament::Old,
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.set_book_picker_testament(Testament::Old, cx);
                                }),
                            ))
                            .child(chip(
                                "book-tab-nt",
                                "新約",
                                testament == Testament::New,
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.set_book_picker_testament(Testament::New, cx);
                                }),
                            )),
                    )
                    .child({
                        let mut cell_iter = cells.into_iter();
                        let mut rows: Vec<gpui_omarchy::gpui::AnyElement> = Vec::new();
                        for r in 0usize..4 {
                            let row_cells: Vec<gpui_omarchy::gpui::AnyElement> =
                                (0..4).filter_map(|_| cell_iter.next()).collect();
                            rows.push(
                                h_flex()
                                    .id(("book-grid-row", r as usize))
                                    .w_full()
                                    .gap_1()
                                    .justify_between()
                                    .children(row_cells)
                                    .into_any_element(),
                            );
                        }
                        v_flex()
                            .id("book-picker-grid")
                            .w_full()
                            .gap_1()
                            .children(rows)
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_center()
                            .gap_3()
                            .child(nav_button(
                                "book-page-prev",
                                "‹",
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.shift_book_picker_page(-1, cx);
                                }),
                            ))
                            .child(h_flex().gap_1().items_center().children(dots))
                            .child(nav_button(
                                "book-page-next",
                                "›",
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.shift_book_picker_page(1, cx);
                                }),
                            )),
                    ),
            )
    }

    fn render_main(
        &self,
        lanes_label: String,
        current_chapter: u32,
        chapter_count: u32,
        book_chip_label: String,
        palette: Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let lanes = self.settings.view_mode.lanes();
        let zh = self.settings.chinese_px();
        let en = self.settings.english_px();
        let num = self.settings.number_px();
        let highlight = self.highlight_verse;
        let select_mode = self.select_mode;
        let selected = self.selected_verses.clone();
        let verses: Vec<gpui_omarchy::gpui::AnyElement> = self
            .chapter
            .verses
            .iter()
            .map(|verse| {
                let n = verse.number;
                verse_block(
                    verse,
                    &lanes,
                    palette,
                    zh,
                    en,
                    num,
                    highlight == Some(n),
                    select_mode,
                    &selected,
                    cx,
                )
                .into_any_element()
            })
            .collect();

        v_flex()
            .flex_1()
            .h_full()
            .min_w_0()
            .w_full()
            .child(
                v_flex()
                    .px_6()
                    .py_3()
                    .gap_2()
                    .border_b_1()
                    .border_color(rgb(palette.border))
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .gap_2()
                            .flex_nowrap()
                            .child(
                                h_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .items_center()
                                    .justify_start()
                                    .child(chip(
                                        "book-picker-btn",
                                        book_chip_label,
                                        self.book_picker_open,
                                        palette,
                                        cx.listener(|this, _, _, cx| {
                                            this.open_book_picker(cx);
                                        }),
                                    )),
                            )
                            .child(
                                h_flex()
                                    .items_center()
                                    .justify_center()
                                    .gap_2()
                                    .flex_shrink_0()
                                    .child(self.render_chapter_nav(
                                        current_chapter,
                                        chapter_count,
                                        palette,
                                        cx,
                                    )),
                            )
                            .child(
                                h_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .items_center()
                                    .justify_end()
                                    .gap_2()
                                    .flex_nowrap()
                                    .child(self.render_header_controls(palette, cx)),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(palette.fg_muted))
                            .child(lanes_label),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(input("jump-input", &self.jump_input, window, cx)),
                            )
                            .child(chip(
                                "jump-go",
                                "跳轉",
                                false,
                                palette,
                                cx.listener(|this, _, window, cx| {
                                    this.submit_jump(window, cx);
                                }),
                            )),
                    ),
            )
            .child(
                v_flex()
                    .id("chapter-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.chapter_scroll)
                    .px_6()
                    .py_4()
                    .gap_5()
                    .children(verses),
            )
            .when(select_mode, |d| {
                d.child(self.render_select_bar(palette, cx))
            })
    }

    fn render_hebrew_view(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let verse_no = self.hebrew_verse.unwrap_or(1);
        let entry = self.current_entry();
        let title = format!(
            "{} {}:{} · 希伯來文",
            entry.meta.name_zh, self.chapter.chapter, verse_no
        );
        let verse = self
            .chapter
            .verses
            .iter()
            .find(|v| v.number == verse_no);
        let zh = verse
            .and_then(|v| v.get(TranslationId::Cuv1919))
            .unwrap_or("")
            .to_string();
        let en = verse
            .and_then(|v| v.get(TranslationId::Kjv))
            .unwrap_or("")
            .to_string();
        let words = verse
            .and_then(|v| v.hebrew.as_ref())
            .cloned()
            .unwrap_or_default();
        let selected = self.hebrew_word_idx.filter(|&i| i < words.len());
        let open_section = self.hebrew_detail_section;
        let original_line = words
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let align_columns: Vec<gpui_omarchy::gpui::AnyElement> = words
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let is_sel = selected == Some(i);
                let cells = [
                    ("原文", w.text.clone()),
                    ("音譯", hebrew_opt_or_dash(&w.translit)),
                    ("直譯", hebrew_opt_or_dash(&w.gloss_literal)),
                    ("意譯", hebrew_opt_or_dash(&w.gloss_idiomatic)),
                ];
                let cell_els: Vec<gpui_omarchy::gpui::AnyElement> = cells
                    .into_iter()
                    .enumerate()
                    .map(|(row, (_label, value))| {
                        let is_orig = row == 0;
                        div()
                            .id(("heb-align-cell", i * 4 + row))
                            .min_w(px(56.0))
                            .px_2()
                            .py_1()
                            .text_center()
                            .when(is_orig, |d| d.text_lg().font_weight(FontWeight::SEMIBOLD))
                            .when(!is_orig, |d| d.text_xs())
                            .text_color(rgb(if is_sel {
                                palette.selected_fg()
                            } else if is_orig {
                                palette.fg_primary
                            } else {
                                palette.fg_muted
                            }))
                            .child(value)
                            .into_any_element()
                    })
                    .collect();
                div()
                    .id(("heb-word-col", i))
                    .rounded_md()
                    .px_1()
                    .py_1()
                    .when(is_sel, |d| {
                        d.bg(rgb(palette.selected_bg()))
                            .border_1()
                            .border_color(rgb(palette.selected_border()))
                    })
                    .when(!is_sel, |d| {
                        d.hover(|s| s.bg(rgb(palette.selected_bg())))
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_hebrew_word(i, cx);
                    }))
                    .child(v_flex().gap_1().items_center().children(cell_els))
                    .into_any_element()
            })
            .collect();

        let row_labels: Vec<gpui_omarchy::gpui::AnyElement> = ["原文", "音譯", "直譯", "意譯"]
            .into_iter()
            .enumerate()
            .map(|(row, label)| {
                div()
                    .id(("heb-align-label", row))
                    .h(px(if row == 0 { 32.0 } else { 24.0 }))
                    .flex()
                    .items_center()
                    .text_xs()
                    .text_color(rgb(palette.fg_muted))
                    .child(label)
                    .into_any_element()
            })
            .collect();

        let detail_panel = selected.and_then(|i| words.get(i)).map(|w| {
            let strongs = w
                .strongs
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let morph = w
                .morph
                .as_ref()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let pos = morph
                .as_deref()
                .and_then(morph_pos_label)
                .map(|s| s.to_string());

            let mut rows: Vec<gpui_omarchy::gpui::AnyElement> = Vec::new();
            if let Some(strongs) = strongs {
                rows.push(hebrew_accordion_row(
                    "heb-acc-strongs",
                    "Strong's",
                    strongs,
                    open_section == Some(HebrewDetailSection::Strongs),
                    true,
                    palette,
                    cx.listener(|this, _, _, cx| {
                        this.toggle_hebrew_detail_section(HebrewDetailSection::Strongs, cx);
                    }),
                ));
            }
            if let Some(morph) = morph {
                rows.push(hebrew_accordion_row(
                    "heb-acc-morph",
                    "詞形",
                    morph,
                    open_section == Some(HebrewDetailSection::Morph),
                    false,
                    palette,
                    cx.listener(|this, _, _, cx| {
                        this.toggle_hebrew_detail_section(HebrewDetailSection::Morph, cx);
                    }),
                ));
            }
            if let Some(pos) = pos {
                rows.push(hebrew_accordion_row(
                    "heb-acc-pos",
                    "詞性",
                    pos,
                    open_section == Some(HebrewDetailSection::Pos),
                    false,
                    palette,
                    cx.listener(|this, _, _, cx| {
                        this.toggle_hebrew_detail_section(HebrewDetailSection::Pos, cx);
                    }),
                ));
            }

            v_flex()
                .id("hebrew-word-detail")
                .w(px(280.0))
                .h_full()
                .flex_shrink_0()
                .gap_2()
                .px_4()
                .py_3()
                .border_l_1()
                .border_color(rgb(palette.border))
                .bg(rgb(palette.bg_sidebar))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(palette.fg_primary))
                        .child("詞詳情"),
                )
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(palette.fg_primary))
                        .child(w.text.clone()),
                )
                .child(v_flex().gap_1().w_full().children(rows))
                .into_any_element()
        });

        let left_column = v_flex()
            .id("hebrew-scroll")
            .flex_1()
            .min_w_0()
            .min_h_0()
            .overflow_y_scroll()
            .px_6()
            .py_4()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(palette.fg_muted))
                            .child("和合本"),
                    )
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(palette.fg_primary))
                            .child(zh),
                    ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(palette.fg_muted))
                            .child("KJV"),
                    )
                    .child(div().text_base().text_color(rgb(palette.fg)).child(en)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(palette.fg_muted))
                            .child("原文"),
                    )
                    .child(
                        div()
                            .text_lg()
                            .text_color(rgb(palette.fg_primary))
                            .child(if original_line.is_empty() {
                                "（無 MorphHB 資料）".to_string()
                            } else {
                                original_line
                            }),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(palette.fg_muted))
                            .child("詞對齊（右→左）· 原文／音譯／直譯／意譯 · 點欄開啟詞詳情"),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .items_start()
                            .gap_2()
                            .child(
                                v_flex()
                                    .id("heb-align-labels")
                                    .flex_shrink_0()
                                    .gap_1()
                                    .pt(px(4.0))
                                    .children(row_labels),
                            )
                            .child(
                                h_flex()
                                    .id("heb-align-grid")
                                    .flex_1()
                                    .min_w_0()
                                    .overflow_x_scroll()
                                    .gap_1()
                                    .flex_row_reverse()
                                    .justify_end()
                                    .children(align_columns),
                            ),
                    ),
            );

        v_flex()
            .id("hebrew-view")
            .size_full()
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_3()
                    .border_b_1()
                    .border_color(rgb(palette.border))
                    .child(chip(
                        "hebrew-back",
                        "返回",
                        false,
                        palette,
                        cx.listener(|this, _, window, cx| {
                            this.close_hebrew_view(window, cx);
                        }),
                    ))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(palette.fg_primary))
                            .child(title),
                    )
                    .child(div().w(px(64.0))),
            )
            .child(
                h_flex()
                    .id("hebrew-body")
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(left_column)
                    .children(detail_panel),
            )
    }



    fn render_select_bar(
&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let units: Vec<VerseUnit> = self.selected_verses.iter().copied().collect();
        let summary = format_selection_summary(&self.chapter, &units);
        h_flex()
            .w_full()
            .items_center()
            .justify_between()
            .gap_3()
            .px_6()
            .py_3()
            .border_t_1()
            .border_color(rgb(palette.border))
            .bg(rgb(palette.bg_sidebar))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_sm()
                    .text_color(rgb(palette.fg_primary))
                    .child(summary),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(chip(
                        "select-copy",
                        "複製",
                        false,
                        palette,
                        cx.listener(|this, _, _, cx| {
                            this.copy_selection(cx);
                        }),
                    ))
                    .child(chip(
                        "select-cancel",
                        "取消",
                        false,
                        palette,
                        cx.listener(|this, _, _, cx| {
                            this.exit_select_mode(cx);
                        }),
                    )),
            )
    }

    fn render_header_controls(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let is_single = self.settings.view_mode.is_single();
        let current_single = match &self.settings.view_mode {
            ViewMode::Single(id) => *id,
            ViewMode::Compare(ids) => ids.first().copied().unwrap_or(TranslationId::Cuv1919),
        };

        h_flex()
            .items_center()
            .gap_2()
            .flex_nowrap()
            .child(
                h_flex()
                    .gap_1()
                    .child(chip(
                        "mode-single",
                        "單語",
                        is_single,
                        palette,
                        cx.listener(move |this, _, window, cx| {
                            this.set_view_mode(ViewMode::single(current_single), window, cx);
                        }),
                    ))
                    .child(chip(
                        "mode-compare",
                        "對照",
                        !is_single,
                        palette,
                        cx.listener(|this, _, window, cx| {
                            this.set_view_mode(ViewMode::default(), window, cx);
                        }),
                    )),
            )
            .when(is_single, |d| {
                d.child(
                    h_flex()
                        .gap_1()
                        .children(TranslationId::ALL.iter().enumerate().map(|(i, id)| {
                            let id = *id;
                            chip(
                                ("single-tr", i),
                                id.label(),
                                current_single == id,
                                palette,
                                cx.listener(move |this, _, window, cx| {
                                    this.set_view_mode(ViewMode::single(id), window, cx);
                                }),
                            )
                            .into_any_element()
                        })),
                )
            })
            .child(chip(
                "toggle-select",
                "選擇",
                self.select_mode,
                palette,
                cx.listener(|this, _, _, cx| {
                    this.toggle_select_mode(cx);
                }),
            ))
            .child(chip(
                "open-search",
                "搜尋",
                self.search_open,
                palette,
                cx.listener(|this, _, window, cx| {
                    this.open_search(window, cx);
                }),
            ))
            .child(chip(
                "open-settings",
                "設定",
                self.settings_open,
                palette,
                cx.listener(|this, _, _, cx| {
                    this.settings_open = !this.settings_open;
                    cx.notify();
                }),
            ))
    }

    fn render_chapter_nav(
        &self,
        current_chapter: u32,
        chapter_count: u32,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let picker_open = self.chapter_picker_open;
        h_flex()
            .items_center()
            .gap_2()
            .child(nav_button(
                "prev-ch",
                "‹",
                palette,
                cx.listener(Self::prev_chapter_click),
            ))
            .child(
                div()
                    .min_w(px(52.0))
                    .text_sm()
                    .text_color(rgb(palette.fg_muted))
                    .child(format!("{current_chapter}/{chapter_count}")),
            )
            .child(chip(
                "chapter-picker-btn",
                "章",
                picker_open,
                palette,
                cx.listener(|this, _, _, cx| {
                    this.toggle_chapter_picker(cx);
                }),
            ))
            .child(nav_button(
                "next-ch",
                "›",
                palette,
                cx.listener(Self::next_chapter_click),
            ))
    }

    fn prev_chapter_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.prev_chapter(&PrevChapter, window, cx);
    }

    fn next_chapter_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.next_chapter(&NextChapter, window, cx);
    }

    fn render_settings_overlay(
        &self,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let font_size = self.settings.font_size;
        let theme = self.settings.theme;
        let note = if theme == ThemePreference::System {
            let t = cx.omarchy();
            let tone = match t.appearance {
                gpui_omarchy::gpui::base::ThemeAppearance::Dark => "深色",
                gpui_omarchy::gpui::base::ThemeAppearance::Light => "淺色",
            };
            format!("系統外觀：{tone}（{}）", t.name)
        } else {
            String::new()
        };

        div()
            .id("settings-overlay")
            .absolute()
            .inset_0()
            .flex()
            .flex_row()
            .justify_end()
            .bg(rgba(0x00000066))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.settings_open = false;
                    cx.notify();
                }),
            )
            .child(
                v_flex()
                    .id("settings-panel")
                    .w(px(340.0))
                    .h_full()
                    .bg(rgb(palette.bg_sidebar))
                    .border_l_1()
                    .border_color(rgb(palette.border))
                    .px_5()
                    .py_4()
                    .gap_4()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(palette.fg_primary))
                                    .child("設定"),
                            )
                            .child(chip(
                                "settings-close",
                                "關閉",
                                false,
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.settings_open = false;
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(palette.fg_muted))
                                    .child("主題"),
                            )
                            .child(
                                h_flex().gap_1().flex_wrap().children(
                                    [
                                        ThemePreference::Dark,
                                        ThemePreference::Light,
                                        ThemePreference::System,
                                    ]
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, pref)| {
                                        chip(
                                            ("theme-pref", i),
                                            pref.label_zh(),
                                            theme == pref,
                                            palette,
                                            cx.listener(move |this, _, window, cx| {
                                                this.set_theme_pref(pref, window, cx);
                                            }),
                                        )
                                        .into_any_element()
                                    }),
                                ),
                            )
                            .when(!note.is_empty() && theme == ThemePreference::System, |d| {
                                d.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(palette.fg_muted))
                                        .child(note),
                                )
                            }),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(palette.fg_muted))
                                    .child("字體大小"),
                            )
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(chip(
                                        "font-minus",
                                        "−",
                                        false,
                                        palette,
                                        cx.listener(|this, _, window, cx| {
                                            this.bump_font(-2, window, cx);
                                        }),
                                    ))
                                    .child(
                                        div()
                                            .min_w(px(56.0))
                                            .text_sm()
                                            .text_color(rgb(palette.fg_primary))
                                            .child(format!("{font_size} px")),
                                    )
                                    .child(chip(
                                        "font-plus",
                                        "+",
                                        false,
                                        palette,
                                        cx.listener(|this, _, window, cx| {
                                            this.bump_font(2, window, cx);
                                        }),
                                    )),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(chip(
                                        "font-small",
                                        "小",
                                        font_size == FONT_SMALL,
                                        palette,
                                        cx.listener(|this, _, window, cx| {
                                            this.set_font_size(FONT_SMALL, window, cx);
                                        }),
                                    ))
                                    .child(chip(
                                        "font-medium",
                                        "中",
                                        font_size == FONT_MEDIUM,
                                        palette,
                                        cx.listener(|this, _, window, cx| {
                                            this.set_font_size(FONT_MEDIUM, window, cx);
                                        }),
                                    ))
                                    .child(chip(
                                        "font-large",
                                        "大",
                                        font_size == FONT_LARGE,
                                        palette,
                                        cx.listener(|this, _, window, cx| {
                                            this.set_font_size(FONT_LARGE, window, cx);
                                        }),
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(palette.fg_muted))
                            .child(settings_location_note_zh()),
                    ),
            )
    }

    fn render_chapter_picker_overlay(
        &self,
        chapter_count: u32,
        current_chapter: u32,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let cells: Vec<gpui_omarchy::gpui::AnyElement> = (1..=chapter_count)
            .map(|n| {
                let is_current = n == current_chapter;
                div()
                    .id(("picker-ch", n as usize))
                    .w(px(40.0))
                    .h(px(36.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .text_sm()
                    .when(is_current, |d| {
                        d.bg(rgb(palette.selected_bg()))
                            .text_color(rgb(palette.selected_fg()))
                            .border_1()
                            .border_color(rgb(palette.selected_border()))
                    })
                    .when(!is_current, |d| {
                        d.text_color(rgb(palette.fg_muted))
                            .hover(|s| s.bg(rgb(palette.selected_bg())))
                    })
                    .on_click(cx.listener(move |this, event, window, cx| {
                        this.on_chapter_click(n, event, window, cx);
                    }))
                    .child(format!("{n}"))
                    .into_any_element()
            })
            .collect();

        div()
            .id("chapter-picker-overlay")
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            .items_center()
            .pt(px(120.0))
            .bg(rgba(0x00000066))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.chapter_picker_open = false;
                    window.focus(&this.focus_handle, cx);
                    cx.notify();
                }),
            )
            .child(
                v_flex()
                    .id("chapter-picker-panel")
                    .w(px(420.0))
                    .max_h(px(480.0))
                    .bg(rgb(palette.bg_sidebar))
                    .border_1()
                    .border_color(rgb(palette.border))
                    .rounded_md()
                    .px_4()
                    .py_3()
                    .gap_3()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(palette.fg_primary))
                                    .child("選擇章節"),
                            )
                            .child(chip(
                                "chapter-picker-close",
                                "關閉",
                                false,
                                palette,
                                cx.listener(|this, _, window, cx| {
                                    this.chapter_picker_open = false;
                                    window.focus(&this.focus_handle, cx);
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        div()
                            .id("chapter-picker-grid")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .child(h_flex().w_full().flex_wrap().gap_1().children(cells)),
                    ),
            )
    }

    fn render_search_overlay(
        &self,
        palette: Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let hits = self.search_hits.clone();
        let total = self.search_total;
        let shown = hits.len();
        let query_empty = self.search_input.read(cx).value().trim().is_empty();
        let all_lanes = self.search_all_lanes;
        let status = if query_empty {
            "輸入關鍵字（預設搜尋目前譯本）".to_string()
        } else if total == 0 {
            "沒有符合的經文".to_string()
        } else if shown < total {
            format!("顯示 {shown}／共 {total} 筆")
        } else {
            format!("{total} 筆")
        };

        let rows: Vec<gpui_omarchy::gpui::AnyElement> = hits
            .into_iter()
            .enumerate()
            .map(|(i, hit)| {
                let label = hit.ref_zh();
                let snippet = hit.snippet.clone();
                let lane = hit.translation.label();
                div()
                    .id(("hit", i))
                    .w_full()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .hover(|s| s.bg(rgb(palette.selected_bg())))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.jump_to_search_hit(&hit, window, cx);
                    }))
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .justify_between()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(palette.fg_primary))
                                            .child(label),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(palette.fg_muted))
                                            .child(lane),
                                    ),
                            )
                            .child(div().text_sm().text_color(rgb(palette.fg)).child(snippet)),
                    )
                    .into_any_element()
            })
            .collect();

        div()
            .id("search-overlay")
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            .items_center()
            .pt(px(56.0))
            .bg(rgba(0x00000066))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.close_search(window, cx);
                }),
            )
            .child(
                v_flex()
                    .id("search-panel")
                    .w(px(560.0))
                    .max_h(px(560.0))
                    .bg(rgb(palette.bg_sidebar))
                    .border_1()
                    .border_color(rgb(palette.border))
                    .rounded_md()
                    .px_5()
                    .py_4()
                    .gap_3()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(palette.fg_primary))
                                    .child("搜尋"),
                            )
                            .child(chip(
                                "search-close",
                                "關閉",
                                false,
                                palette,
                                cx.listener(|this, _, window, cx| {
                                    this.close_search(window, cx);
                                }),
                            )),
                    )
                    .child(input("search-input", &self.search_input, window, cx))
                    .child(
                        h_flex()
                            .gap_1()
                            .child(chip(
                                "search-current",
                                "目前譯本",
                                !all_lanes,
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.search_all_lanes = false;
                                    this.run_search(cx);
                                }),
                            ))
                            .child(chip(
                                "search-all",
                                "全部譯本",
                                all_lanes,
                                palette,
                                cx.listener(|this, _, _, cx| {
                                    this.search_all_lanes = true;
                                    this.run_search(cx);
                                }),
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(palette.fg_muted))
                            .child(status),
                    )
                    .child(
                        v_flex()
                            .id("search-results")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .gap_1()
                            .children(rows),
                    ),
            )
    }
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
/// + `/`-separated segments). Takes the **last non-suffix** segment's POS
/// letter (suffixes start with `S`) so `HNcmpa` → 名詞 and `HR/Ncfsa` → 名詞.
/// Returns `None` when morph is empty or no known POS letter is found.
fn morph_pos_label(morph: &str) -> Option<&'static str> {
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



fn hebrew_opt_or_dash(opt: &Option<String>) -> String {
    opt.as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "—".to_string())
}

fn default_hebrew_detail_section(word: &HebrewWord) -> Option<HebrewDetailSection> {
    let strongs_ok = word
        .strongs
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if strongs_ok {
        return Some(HebrewDetailSection::Strongs);
    }
    let morph = word.morph.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty());
    if morph.is_some() {
        // 詞形 row is shown whenever morph is present; 詞性 is a derived sibling.
        return Some(HebrewDetailSection::Morph);
    }
    None
}

fn hebrew_accordion_row(
    id: &'static str,
    title: &'static str,
    body: String,
    open: bool,
    accent_body: bool,
    palette: Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> gpui_omarchy::gpui::AnyElement {
    let marker = if open { "▼" } else { "▶" };
    v_flex()
        .id(id)
        .w_full()
        .rounded_md()
        .border_1()
        .border_color(rgb(palette.border))
        .bg(rgb(palette.bg))
        .child(
            h_flex()
                .id((id, 1u64))
                .w_full()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .cursor_pointer()
                .hover(|s| s.bg(rgb(palette.selected_bg())))
                .on_click(on_click)
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(palette.fg_primary))
                        .child(title),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(palette.fg_muted))
                        .child(marker),
                ),
        )
        .when(open, |d| {
            d.child(
                div()
                    .px_3()
                    .pb_2()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(if accent_body {
                        palette.accent
                    } else {
                        palette.fg_primary
                    }))
                    .child(body),
            )
        })
        .into_any_element()
}


fn chip(
    id: impl Into<gpui_omarchy::gpui::ElementId>,
    label: impl Into<SharedString>,
    active: bool,
    palette: Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px_2()
        .py_1()
        .rounded_md()
        .text_sm()
        .when(active, |d| {
            d.bg(rgb(palette.selected_bg()))
                .text_color(rgb(palette.selected_fg()))
                .border_1()
                .border_color(rgb(palette.selected_border()))
        })
        .when(!active, |d| {
            d.text_color(rgb(palette.fg))
                .hover(|s| s.bg(rgb(palette.selected_bg())))
        })
        .on_click(on_click)
        .child(label.into())
}

fn nav_button(
    id: &'static str,
    label: &'static str,
    palette: Palette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_md()
        .bg(rgb(palette.selected_bg()))
        .text_color(rgb(palette.selected_fg()))
        .border_1()
        .border_color(rgb(palette.selected_border()))
        .hover(|s| s.bg(rgb(palette.selected_border())))
        .on_click(on_click)
        .child(label)
}

fn lane_checkbox(selected: bool, palette: Palette) -> impl IntoElement {
    div()
        .w(px(16.0))
        .h(px(16.0))
        .mt(px(4.0))
        .flex_shrink_0()
        .rounded_sm()
        .border_1()
        .border_color(rgb(if selected {
            palette.selected_border()
        } else {
            palette.border
        }))
        .when(selected, |box_| box_.bg(rgb(palette.selected_fg())))
}

fn verse_block(
    verse: &bible_core::Verse,
    lanes: &[TranslationId],
    palette: Palette,
    zh_size: f32,
    en_size: f32,
    num_size: f32,
    highlighted: bool,
    select_mode: bool,
    selected: &BTreeSet<VerseUnit>,
    cx: &mut Context<BibleView>,
) -> impl IntoElement {
    let texts = verse.texts_for(lanes);
    let n = verse.number;
    let any_selected = texts
        .iter()
        .any(|(id, _)| selected.contains(&VerseUnit::new(n, *id)));
    let lanes_ui: Vec<gpui_omarchy::gpui::AnyElement> = texts
        .into_iter()
        .enumerate()
        .map(|(lane_i, (id, text))| {
            let size = if id.is_cjk() { zh_size } else { en_size };
            let color = if id.is_cjk() {
                palette.fg_primary
            } else {
                palette.fg
            };
            let lane_selected = selected.contains(&VerseUnit::new(n, id));
            let body = div()
                .text_size(px(size))
                .text_color(rgb(color))
                .line_height(px((size * 1.55).round()))
                .child(text.to_string());
            h_flex()
                .id(("verse-lane", n as usize * 8 + lane_i))
                .w_full()
                .items_start()
                .gap_2()
                .rounded_md()
                .when(lane_selected, |d| {
                    d.bg(rgb(palette.selected_bg()))
                        .border_1()
                        .border_color(rgb(palette.selected_border()))
                        .px_1()
                })
                .when(select_mode, |d| {
                    d
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.on_verse_select(n, id, cx);
                        }))
                })
                .when(select_mode, |d| {
                    d.child(lane_checkbox(lane_selected, palette))
                })
                .child(body.flex_1().min_w_0())
                .into_any_element()
        })
        .collect();

    h_flex()
        .id(("verse", n as usize))
        .w_full()
        .items_start()
        .gap_3()
        .rounded_md()
        .when(highlighted || any_selected, |d| {
            d.bg(rgb(palette.selected_bg()))
                .border_1()
                .border_color(rgb(palette.selected_border()))
                .px_2()
                .py_1()
        })
         .child({
            let show_dagger = verse.hebrew.as_ref().is_some_and(|w| !w.is_empty());
            let mut col = v_flex()
                .w(px((num_size * 2.2).max(28.0)))
                .pt(px(4.0))
                .gap_1()
                .items_center()
                .child(
                    div()
                        .text_size(px(num_size))
                        .font_weight(if highlighted || any_selected {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(rgb(palette.accent))
                        .child(format!("{}", verse.number)),
                );
            if show_dagger {
                col = col.child(
                    with_tooltip(
                        div()
                            .id(("heb-marker", n as usize))
                            .px_1()
                            .rounded_sm()
                            .text_sm()
                            .text_color(rgb(palette.accent))
                            .hover(|s| s.bg(rgb(palette.selected_bg())))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.open_hebrew_view(n, cx);
                            }))
                            .child("†"),
                        "希伯來文形態",
                    ),
                );
            }
            col
        })
        .child(v_flex().flex_1().min_w_0().gap_1().children(lanes_ui))
}




#[cfg(test)]
mod tests {
    use super::morph_pos_label;

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
        assert_eq!(morph_pos_label("HXx"), None);
    }
}
