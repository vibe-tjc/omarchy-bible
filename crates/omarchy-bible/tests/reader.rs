//! The reader's flows, headless on the real engine: what a user does and
//! what they then see. Each test has its own settings file in a temporary
//! directory, so none touches the user's.

use std::path::PathBuf;

use nsh_engine::{EngineConfig, Points, Root, Size};
use nsh_testing::Headless;

struct App {
    app: Headless,
    settings: PathBuf,
}

impl std::ops::Deref for App {
    type Target = Headless;
    fn deref(&self) -> &Headless {
        &self.app
    }
}

impl std::ops::DerefMut for App {
    fn deref_mut(&mut self) -> &mut Headless {
        &mut self.app
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Some(dir) = self.settings.parent() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

/// The desktop window the app opens in.
const DESKTOP: (f32, f32) = (980.0, 760.0);
/// A Pixel 8 (1080×2400 px at 420 dpi) in points.
const PHONE: (f32, f32) = (411.0, 914.0);
/// A Pixel Tablet (2560×1600 px at 320 dpi), landscape, in points.
const TABLET: (f32, f32) = (1280.0, 800.0);

fn start_with(name: &str, settings_json: Option<&str>) -> App {
    start_sized(name, settings_json, DESKTOP)
}

fn start_on(name: &str, size: (f32, f32)) -> App {
    start_sized(name, None, size)
}

fn start_sized(name: &str, settings_json: Option<&str>, size: (f32, f32)) -> App {
    let dir =
        std::env::temp_dir().join(format!("omarchy-bible-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let settings = dir.join("settings.json");
    if let Some(json) = settings_json {
        std::fs::write(&settings, json).unwrap();
    }
    let path = settings.to_string_lossy().into_owned();
    let root = Root::new::<omarchy_bible::App>(move || omarchy_bible::AppProps {
        settings_path: Some(path.clone().into()).into(),
    });
    let app = Headless::start(EngineConfig {
        viewport: Size::new(Points(size.0), Points(size.1)),
        ..EngineConfig::new(root)
    })
    .unwrap();
    let mut app = App { app, settings };
    // The Bible loads before the first frame: wait for the reader.
    app.wait_for_within("id:verses", std::time::Duration::from_secs(20))
        .expect("the reader shows");
    app
}

fn start(name: &str) -> App {
    start_with(name, None)
}

fn position(app: &App) -> String {
    app.screen()
        .texts()
        .into_iter()
        .find(|t| t.contains('/') && t.chars().all(|c| c.is_ascii_digit() || c == '/'))
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn opens_at_genesis_one_in_compare_mode() -> Result<(), String> {
    let mut app = start("opens");
    app.wait_for("舊約 · 創")?;
    app.wait_for("1/50")?;
    app.wait_for("起初，神創造天地。")?;
    app.wait_for("In the beginning God created the heaven and the earth.")?;
    Ok(())
}

#[test]
fn chapter_buttons_step_and_cross_books() -> Result<(), String> {
    let mut app = start("step");
    app.wait_for("1/50")?;
    app.press("id:next-ch")?;
    app.wait_for("2/50")?;
    app.press("id:prev-ch")?;
    app.wait_for("1/50")?;
    // Before Genesis 1 there is nothing: it stays.
    app.press("id:prev-ch")?;
    app.wait_for("1/50")?;
    // Genesis 50 → Exodus 1.
    app.press("id:chapter-picker-btn")?;
    app.press("id:chapter-50")?;
    app.wait_for("50/50")?;
    app.press("id:next-ch")?;
    app.wait_for("舊約 · 出")?;
    app.wait_for("1/40")?;
    // And back: Exodus 1 → Genesis 50.
    app.press("id:prev-ch")?;
    app.wait_for("舊約 · 創")?;
    app.wait_for("50/50")?;
    Ok(())
}

#[test]
fn bracket_keys_step_chapters() -> Result<(), String> {
    let mut app = start("keys");
    app.wait_for("1/50")?;
    app.press_key("]")?;
    app.wait_for("2/50")?;
    app.press_key("[")?;
    app.wait_for("1/50")?;
    Ok(())
}

#[test]
fn book_picker_opens_a_book_at_chapter_one() -> Result<(), String> {
    let mut app = start("books");
    app.press("id:book-picker-btn")?;
    app.wait_for("選擇書卷")?;
    app.press("id:book-tab-nt")?;
    app.press("label:馬太福音")?;
    app.wait_for("新約 · 太")?;
    app.wait_for("1/28")?;
    app.wait_for_gone("選擇書卷", std::time::Duration::from_secs(2))?;
    Ok(())
}

#[test]
fn book_picker_pages_through_the_old_testament() -> Result<(), String> {
    let mut app = start("pages");
    app.press("id:book-picker-btn")?;
    app.wait_for("label:創世記")?;
    app.press("id:book-page-next")?;
    // The second page of 39 books starts at the 17th, 以斯帖記.
    app.wait_for("label:以斯帖記")?;
    assert!(!app.screen().matches("label:創世記")?);
    // From the last page, next wraps to the first.
    app.press("id:book-page-next")?;
    app.press("id:book-page-next")?;
    app.wait_for("label:創世記")?;
    Ok(())
}

#[test]
fn jump_field_goes_to_a_verse() -> Result<(), String> {
    let mut app = start("jump");
    app.type_text("id:jump-input", "約3:16")?;
    app.press("id:jump-go")?;
    app.wait_for("新約 · 約")?;
    app.wait_for("3/21")?;
    app.wait_for("text:/神愛世人/")?;
    // Numeric form: book 1, chapter 3.
    app.clear("id:jump-input")?;
    app.type_text("id:jump-input", "1:3:5")?;
    app.submit("id:jump-input")?;
    app.wait_for("舊約 · 創")?;
    app.wait_for("3/50")?;
    Ok(())
}

#[test]
fn a_bad_reference_says_so_and_stays() -> Result<(), String> {
    let mut app = start("badjump");
    app.type_text("id:jump-input", "不存在 9:9")?;
    app.press("id:jump-go")?;
    app.wait_for("text:/^找不到這段經文/")?;
    assert_eq!(position(&app), "1/50");
    // Typing again clears the error.
    app.type_text("id:jump-input", "x")?;
    app.wait_for_gone("text:/^找不到這段經文/", std::time::Duration::from_secs(2))?;
    Ok(())
}

#[test]
fn search_finds_and_opens_a_verse() -> Result<(), String> {
    let mut app = start("search");
    app.press_key("/")?;
    app.wait_for("輸入關鍵字（預設搜尋目前譯本）")?;
    app.type_text("id:search-input", "神愛世人")?;
    app.wait_for("約翰福音 3:16")?;
    app.wait_for("1 筆")?;
    app.press("約翰福音 3:16")?;
    app.wait_for("新約 · 約")?;
    app.wait_for("3/21")?;
    app.wait_for_gone("id:search-panel", std::time::Duration::from_secs(2))?;
    Ok(())
}

#[test]
fn search_reports_nothing_found_and_scope() -> Result<(), String> {
    let mut app = start("search-scope");
    app.press("id:open-search")?;
    app.type_text("id:search-input", "zzqqxx")?;
    app.wait_for("沒有符合的經文")?;
    // English, single CUV lane: no hits; all lanes: hits.
    app.press("id:search-close")?;
    app.press("id:mode-single")?;
    app.press("id:open-search")?;
    app.clear("id:search-input")?;
    app.type_text("id:search-input", "only begotten")?;
    app.wait_for("沒有符合的經文")?;
    app.press("id:search-all")?;
    app.wait_for("約翰福音 3:16")?;
    Ok(())
}

#[test]
fn escape_closes_the_open_overlay() -> Result<(), String> {
    let mut app = start("escape");
    app.press("id:open-settings")?;
    app.wait_for("id:settings-panel")?;
    app.press_key("escape")?;
    app.wait_for_gone("id:settings-panel", std::time::Duration::from_secs(2))?;
    app.press("id:chapter-picker-btn")?;
    app.wait_for("選擇章節")?;
    app.press_key("escape")?;
    app.wait_for_gone("選擇章節", std::time::Duration::from_secs(2))?;
    Ok(())
}

#[test]
fn single_mode_shows_one_lane_and_persists() -> Result<(), String> {
    let mut app = start("single");
    app.wait_for("In the beginning God created the heaven and the earth.")?;
    app.press("id:mode-single")?;
    app.wait_for_gone(
        "In the beginning God created the heaven and the earth.",
        std::time::Duration::from_secs(2),
    )?;
    app.press("id:single-KJV")?;
    app.wait_for("In the beginning God created the heaven and the earth.")?;
    app.wait_for_gone("起初，神創造天地。", std::time::Duration::from_secs(2))?;
    let saved = std::fs::read_to_string(&app.settings).map_err(|e| e.to_string())?;
    assert!(saved.contains("\"kjv\""), "{saved}");
    Ok(())
}

#[test]
fn select_and_copy_verses() -> Result<(), String> {
    let mut app = start("select");
    app.press("id:toggle-select")?;
    app.wait_for("取消")?;
    app.press("起初，神創造天地。")?;
    app.press("text:/^地是空虛混沌/")?;
    app.wait_for("text:/^已選 中文 2 節/")?;
    app.press("id:select-copy")?;
    let copied = app.screen().clipboard().unwrap_or_default().to_owned();
    assert!(copied.contains("起初，神創造天地。"), "{copied}");
    assert!(copied.contains("地是空虛混沌"), "{copied}");
    assert!(copied.contains("創世記 1:1"), "{copied}");
    // Cancel leaves select mode.
    app.press("id:select-cancel")?;
    app.wait_for_gone("id:selection-summary", std::time::Duration::from_secs(2))?;
    Ok(())
}

#[test]
fn settings_change_font_size_and_theme_and_persist() -> Result<(), String> {
    let mut app = start("settings");
    app.press("id:open-settings")?;
    app.wait_for("18 px")?;
    app.press("id:font-plus")?;
    app.wait_for("20 px")?;
    app.press("id:font-large")?;
    app.wait_for("22 px")?;
    app.press("id:theme-淺色")?;
    let saved = std::fs::read_to_string(&app.settings).map_err(|e| e.to_string())?;
    assert!(saved.contains("\"font_size\": 22"), "{saved}");
    assert!(saved.contains("\"light\""), "{saved}");
    Ok(())
}

#[test]
fn saved_settings_are_read_at_start() -> Result<(), String> {
    let mut app = start_with(
        "saved",
        Some(r#"{ "theme": "light", "font_size": 26, "view_mode": { "single": "kjv" } }"#),
    );
    app.wait_for_gone("起初，神創造天地。", std::time::Duration::from_secs(2))?;
    app.wait_for("In the beginning God created the heaven and the earth.")?;
    assert!(!app.screen().shows("起初，神創造天地。"));
    app.press("id:open-settings")?;
    app.wait_for("26 px")?;
    Ok(())
}

#[test]
fn hebrew_page_shows_word_details_and_returns() -> Result<(), String> {
    let mut app = start("hebrew");
    // Genesis 1:1 has MorphHB data: its † opens the Hebrew page.
    app.press("label:希伯來文形態 >> nth=0")?;
    app.wait_for("創世記 1:1 · 希伯來文")?;
    app.wait_for("起初，神創造天地。")?;
    // The alignment's transliteration row of the first word.
    app.wait_for("bərēʾšiyt")?;
    assert!(!app.screen().shows("詞詳情"));
    // The first word: its detail opens with Strong's expanded.
    app.press("id:hebrew-word-0-0")?;
    app.wait_for("詞詳情")?;
    app.wait_for("id:hebrew-section-Strong's")?;
    let strongs_open = app.screen().shows("▼");
    assert!(
        strongs_open,
        "Strong's opens by default: {:?}",
        app.screen().texts()
    );
    // A second tap on the same word closes the detail.
    app.press("id:hebrew-word-0-0")?;
    app.wait_for_gone("詞詳情", std::time::Duration::from_secs(2))?;
    // Escape goes back to the reader at the verse.
    app.press_key("escape")?;
    app.wait_for("1/50")?;
    app.wait_for("id:verses")?;
    Ok(())
}

#[test]
fn new_testament_verses_have_no_hebrew_marker() -> Result<(), String> {
    let mut app = start("nt-hebrew");
    app.type_text("id:jump-input", "太1")?;
    app.press("id:jump-go")?;
    app.wait_for("新約 · 太")?;
    // The † is in every cell but hidden where there is no data.
    let hidden = app.press("label:希伯來文形態 >> nth=0");
    assert!(app.screen().shows("1/28"));
    assert!(
        !app.screen().shows("詞詳情")
            && !app
                .screen()
                .texts()
                .iter()
                .any(|t| t.ends_with("· 希伯來文")),
        "{hidden:?}"
    );
    Ok(())
}

/// Every verse cell on screen shows its own number with its own text: the
/// pairs of (number, Chinese text) the screen reports, checked against the
/// chapter's data.
fn assert_cells_match(app: &App, osis: &str, chapter: u32) {
    let bible = bible_core::load_bible().unwrap();
    let index = bible.index_of(osis).unwrap();
    let loaded = bible.load_chapter_at(index, chapter).unwrap();
    let texts = app.screen().texts();
    let mut checked = 0;
    for w in texts.windows(3) {
        let (Ok(n), "†") = (w[0].parse::<u32>(), w[1]) else {
            continue;
        };
        let verse = loaded
            .verses
            .iter()
            .find(|v| v.number == n)
            .expect("verse in chapter");
        let zh = verse.get(bible_core::TranslationId::Cuv1919).unwrap();
        assert_eq!(w[2], zh, "verse {n} shows another verse's text");
        checked += 1;
    }
    assert!(checked > 5, "too few cells checked: {checked}");
}

#[test]
fn verses_far_down_a_long_chapter_render_their_own_text() -> Result<(), String> {
    let mut app = start("recycle");
    // Psalm 119 has 176 verses; jumping to the end and back reuses cells for
    // other rows, which must then show their own verse.
    app.type_text("id:jump-input", "詩119:176")?;
    app.press("id:jump-go")?;
    app.wait_for("119/150")?;
    app.wait_for("text:/^我如亡羊走迷了路/")?;
    assert_cells_match(&app, "Ps", 119);
    app.clear("id:jump-input")?;
    app.type_text("id:jump-input", "詩119:1")?;
    app.press("id:jump-go")?;
    app.wait_for("text:/^行為完全、遵行耶和華律法的/")?;
    assert_cells_match(&app, "Ps", 119);
    // Another chapter in the same cells.
    app.press("id:next-ch")?;
    app.wait_for("120/150")?;
    assert_cells_match(&app, "Ps", 120);
    Ok(())
}

// ── Phone and tablet ─────────────────────────────────────────────────

/// Whether a view matching `selector` is on screen (not hidden or clipped).
fn on_screen(app: &App, selector: &str) -> bool {
    app.screen().matches(selector).unwrap_or(false)
}

#[test]
fn phone_header_moves_tools_into_a_menu() -> Result<(), String> {
    let mut app = start_on("phone-header", PHONE);
    app.wait_for("舊約 · 創")?;
    app.wait_for("1/50")?;
    // The desktop's row of tools would overflow a phone: it is hidden, and
    // the ⋯ menu holds them.
    assert!(
        !on_screen(&app, "id:mode-single"),
        "{:?}",
        app.screen().texts()
    );
    assert!(!on_screen(&app, "id:open-settings"));
    assert!(on_screen(&app, "id:tools-menu"));
    assert!(on_screen(&app, "id:jump-input"));
    app.wait_for("起初，神創造天地。")?;
    Ok(())
}

#[test]
fn phone_menu_switches_translation_and_opens_tools() -> Result<(), String> {
    let mut app = start_on("phone-menu", PHONE);
    app.press("id:tools-menu")?;
    app.press("id:menu-kjv")?;
    app.wait_for_gone("起初，神創造天地。", std::time::Duration::from_secs(2))?;
    app.wait_for("In the beginning God created the heaven and the earth.")?;
    app.press("id:tools-menu")?;
    app.press("id:menu-compare")?;
    app.wait_for("起初，神創造天地。")?;
    app.press("id:tools-menu")?;
    app.press("id:menu-search")?;
    app.wait_for("id:search-panel")?;
    app.type_text("id:search-input", "神愛世人")?;
    // Results come after the typing debounce.
    app.wait_for("約翰福音 3:16")?;
    app.press("約翰福音 3:16")?;
    app.wait_for("3/21")?;
    app.press("id:tools-menu")?;
    app.press("id:menu-settings")?;
    app.wait_for("id:settings-panel")?;
    Ok(())
}

#[test]
fn phone_overlays_fill_the_screen() -> Result<(), String> {
    let mut app = start_on("phone-sheets", PHONE);
    app.press("id:book-picker-btn")?;
    app.wait_for("選擇書卷")?;
    // On a touch screen the full name shows under each abbreviation.
    app.wait_for("創世記")?;
    app.press("id:book-tab-nt")?;
    app.press("label:約翰福音")?;
    app.wait_for("新約 · 約")?;
    app.press("id:chapter-picker-btn")?;
    app.wait_for("選擇章節")?;
    app.press("id:chapter-3")?;
    app.wait_for("3/21")?;
    Ok(())
}

#[test]
fn phone_back_steps_out_before_leaving() -> Result<(), String> {
    let mut app = start_on("phone-back", PHONE);
    // Back closes an open sheet (the dialog takes it)...
    app.press("id:book-picker-btn")?;
    app.wait_for("選擇書卷")?;
    app.back()?;
    app.wait_for_gone("選擇書卷", std::time::Duration::from_secs(2))?;
    // ...leaves select mode...
    app.press("id:tools-menu")?;
    app.press("id:menu-select")?;
    app.wait_for("取消")?;
    app.back()?;
    app.wait_for_gone("id:selection-summary", std::time::Duration::from_secs(2))?;
    // ...and leaves the Hebrew page for the reader.
    app.press("label:希伯來文形態 >> nth=0")?;
    app.wait_for("創世記 1:1 · 希伯來文")?;
    app.back()?;
    app.wait_for("1/50")?;
    app.wait_for("id:verses")?;
    Ok(())
}

#[test]
fn phone_hebrew_detail_sits_under_the_words() -> Result<(), String> {
    let mut app = start_on("phone-hebrew", PHONE);
    app.press("label:希伯來文形態 >> nth=0")?;
    app.wait_for("bərēʾšiyt")?;
    app.press("id:hebrew-word-0-0")?;
    app.wait_for("詞詳情")?;
    app.wait_for("Strong's")?;
    Ok(())
}

#[test]
fn tablet_keeps_the_regular_layout() -> Result<(), String> {
    let mut app = start_on("tablet", TABLET);
    app.wait_for("1/50")?;
    // Wide enough for the row of tools: no ⋯ menu.
    assert!(
        on_screen(&app, "id:mode-single"),
        "{:?}",
        app.screen().texts()
    );
    assert!(on_screen(&app, "id:open-settings"));
    assert!(!on_screen(&app, "id:tools-menu"));
    app.press("id:mode-single")?;
    app.wait_for_gone(
        "In the beginning God created the heaven and the earth.",
        std::time::Duration::from_secs(2),
    )?;
    app.press("id:open-search")?;
    app.type_text("id:search-input", "神愛世人")?;
    // Results come after the typing debounce.
    app.wait_for("約翰福音 3:16")?;
    app.press("約翰福音 3:16")?;
    app.wait_for("3/21")?;
    Ok(())
}

#[test]
fn rotating_reflows_between_layouts() -> Result<(), String> {
    let mut app = start_on("rotate", PHONE);
    assert!(on_screen(&app, "id:tools-menu"));
    // A phone turned sideways (914 pt wide) gets the regular layout...
    app.set_viewport(Size::new(Points(PHONE.1), Points(PHONE.0)))?;
    app.wait_for("id:open-settings")?;
    assert!(!on_screen(&app, "id:tools-menu"));
    // ...and back.
    app.set_viewport(Size::new(Points(PHONE.0), Points(PHONE.1)))?;
    app.wait_for("id:tools-menu")?;
    // The reading position survives the reflow.
    assert_eq!(position(&app), "1/50");
    Ok(())
}

#[test]
fn projection_defaults_to_single_and_paginates_selection() -> Result<(), String> {
    let mut app = start("projection-selection");
    app.press("id:toggle-select")?;
    app.press("起初，神創造天地。")?;
    app.press("text:/^地是空虛混沌/")?;
    app.press("id:select-project")?;
    app.wait_for("單節：每張一節。點經文指定，或用「選擇」選取數節。")?;
    app.wait_for("創世記 1:1")?;
    app.press("id:projection-next")?;
    app.wait_for("創世記 1:2")?;
    app.press("id:projection-prev")?;
    app.wait_for("創世記 1:1")?;
    app.press("id:projection-multiple")?;
    app.wait_for("多節：從目前這節起排滿一張；有選取時只排選取的經節。")?;
    // No output starts simply by opening the panel or changing the option.
    app.wait_for("尚未開始投影")?;
    app.wait_for("未投影")?;
    app.press("id:projection-chapter")?;
    app.press("id:projection-single")?;
    app.wait_for("第 1 / 31 張")?;
    app.press("id:projection-close")?;
    app.wait_for("id:verses")?;
    Ok(())
}

#[test]
fn a_verse_tap_targets_single_verse_projection() -> Result<(), String> {
    let mut app = start("projection-focused");
    app.press("text:/^地是空虛混沌/")?;
    app.press("id:open-projection")?;
    app.wait_for("單節：每張一節。點經文指定，或用「選擇」選取數節。")?;
    app.wait_for("創世記 1:2")?;
    app.wait_for("第 1 / 1 張")?;
    app.wait_for("尚未開始投影")?;
    Ok(())
}

#[test]
fn phone_can_reach_projection_from_the_tools_menu() -> Result<(), String> {
    let mut app = start_on("projection-phone", PHONE);
    app.press("id:tools-menu")?;
    app.press("id:menu-projection")?;
    app.wait_for("單節：每張一節。點經文指定，或用「選擇」選取數節。")?;
    app.wait_for("id:projection-start")?;
    app.back()?;
    app.wait_for("id:tools-menu")?;
    Ok(())
}

#[test]
fn projection_steps_to_neighbouring_verses_across_chapters() -> Result<(), String> {
    let mut app = start("projection-verse-step");
    app.press("text:/^地是空虛混沌/")?;
    app.press("id:open-projection")?;
    app.wait_for("創世記 1:2")?;
    app.press("id:projection-verse-next")?;
    app.wait_for("創世記 1:3")?;
    app.press("id:projection-verse-prev")?;
    app.press("id:projection-verse-prev")?;
    app.wait_for("創世記 1:1")?;
    app.press("id:projection-verse-prev")?;
    app.wait_for("已是第一節")?;
    app.wait_for("創世記 1:1")?;
    Ok(())
}

#[test]
fn next_verse_after_a_selection_starts_after_its_last_verse() -> Result<(), String> {
    let mut app = start("projection-verse-after-selection");
    app.press("id:toggle-select")?;
    app.press("起初，神創造天地。")?;
    app.press("text:/^地是空虛混沌/")?;
    app.press("id:select-project")?;
    app.press("id:projection-verse-next")?;
    app.wait_for("創世記 1:3")?;
    app.wait_for("第 1 / 1 張")?;
    Ok(())
}

#[test]
fn next_verse_crosses_into_the_next_chapter() -> Result<(), String> {
    let mut app = start("projection-verse-chapter");
    app.type_text("id:jump-input", "1:1:31")?;
    app.press("id:jump-go")?;
    app.press("id:open-projection")?;
    app.wait_for("創世記 1:31")?;
    app.press("id:projection-verse-next")?;
    app.wait_for("創世記 2:1")?;
    app.press("id:projection-verse-prev")?;
    app.wait_for("創世記 1:31")?;
    Ok(())
}

/// Opens the projection panel with one lane (和合本), then multi mode.
fn open_multiple_projection(app: &mut App) -> Result<(), String> {
    app.press("id:open-projection")?;
    app.press("id:projection-lane-KJV")?;
    app.press("id:projection-multiple")?;
    app.wait_for("多節：從目前這節起排滿一張；有選取時只排選取的經節。")?;
    Ok(())
}

#[test]
fn multiple_without_a_selection_fills_a_page_from_the_focused_verse() -> Result<(), String> {
    let mut app = start("projection-multi-fill");
    open_multiple_projection(&mut app)?;
    app.wait_for("創世記 1:1, 2, 3, 4, 5, 6, 7, 8, 9")?;
    app.wait_for("第 1 / 1 張")?;
    // Next page continues after the last verse shown.
    app.press("id:projection-verse-next")?;
    app.wait_for("創世記 1:10, 11, 12, 13, 14, 15")?;
    app.press("id:projection-verse-prev")?;
    app.wait_for("創世記 1:1, 2, 3, 4, 5, 6, 7, 8, 9")?;
    app.press("id:projection-verse-prev")?;
    app.wait_for("已是第一節")?;
    // Back in single mode, one verse per slide as before.
    app.press("id:projection-single")?;
    app.wait_for("創世記 1:1")?;
    app.wait_for("第 1 / 1 張")?;
    app.press("id:projection-verse-next")?;
    app.wait_for("創世記 1:2")?;
    Ok(())
}

#[test]
fn multiple_previous_page_packs_backward_across_chapters() -> Result<(), String> {
    let mut app = start("projection-multi-back");
    app.type_text("id:jump-input", "1:1:31")?;
    app.press("id:jump-go")?;
    open_multiple_projection(&mut app)?;
    app.wait_for("創世記 1:31")?;
    app.press("id:projection-verse-next")?;
    app.wait_for("創世記 2:1, 2, 3, 4, 5, 6, 7")?;
    // The previous page ends right before 2:1 and fills backward.
    app.press("id:projection-verse-prev")?;
    app.wait_for("創世記 1:27, 28, 29, 30, 31")?;
    app.press("id:projection-verse-next")?;
    app.wait_for("創世記 2:1, 2, 3, 4, 5, 6, 7")?;
    Ok(())
}

#[test]
fn multiple_with_a_selection_projects_only_the_selection() -> Result<(), String> {
    let mut app = start("projection-multi-selection");
    app.press("id:toggle-select")?;
    app.press("起初，神創造天地。")?;
    app.press("text:/^地是空虛混沌/")?;
    app.press("id:select-project")?;
    app.press("id:projection-lane-KJV")?;
    app.press("id:projection-multiple")?;
    app.wait_for("創世記 1:1, 2")?;
    app.wait_for("第 1 / 1 張")?;
    Ok(())
}

#[test]
fn projection_versions_switch_between_one_and_a_comparison() -> Result<(), String> {
    let mut app = start("projection-versions");
    app.press("id:open-projection")?;
    app.wait_for("text:/^和合本 1919 神版\n1  起初/")?;
    app.press("id:projection-lane-KJV")?;
    app.wait_for("text:/^1  起初，神創造天地。$/")?;
    app.press("id:projection-lane-和合本 1919 神版")?;
    app.wait_for("至少要投影一個版本")?;
    app.wait_for("text:/^1  起初，神創造天地。$/")?;
    app.press("id:projection-lane-KJV")?;
    app.wait_for("text:/^和合本 1919 神版\n1  起初/")?;
    app.press("id:projection-lane-和合本 1919 神版")?;
    app.wait_for("text:/^1  In the beginning/")?;
    Ok(())
}
