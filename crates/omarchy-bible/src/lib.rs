//! 聖經 (omarchy-bible) in Native Shell: the root component and its UI set.
//!
//! - `settings`: persisted preferences and the palette (plain Rust)
//! - `store`: the reader's state and its only writers
//! - `ui`: the app's theme tokens and styled controls over `nsh_base`
//! - `pages`: the reader, the overlays and the Hebrew page

pub mod pages;
pub mod projection;
pub mod settings;
pub mod store;
pub mod ui;

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use bible_core::Bible;
use nsh_core::prelude::*;

use crate::settings::omarchy_stamp;
use crate::store::{Overlay, SettingsFile, Store};

/// How often the Omarchy theme directory is checked while following the
/// system (the GPUI app polled at the same rate).
const THEME_POLL: Duration = Duration::from_millis(1500);

/// The app's UI set; `nsh.json` `"root"` names [`App`].
#[native_module("omarchy-bible/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}

/// The embedded Bible, parsed once per process.
pub fn bible() -> Arc<Bible> {
    static BIBLE: OnceLock<Arc<Bible>> = OnceLock::new();
    BIBLE
        .get_or_init(|| Arc::new(bible_core::load_bible().expect("load embedded CUV 神版 + KJV")))
        .clone()
}

/// The props of [`App`].
#[derive(Clone, PartialEq, Props)]
pub struct AppProps {
    /// Where settings are read and saved: the user's file when unset.
    /// Tests give a temporary path so they never touch it.
    pub settings_path: Prop<Option<SharedString>>,
}

/// The root: the reader with its overlays, or the Hebrew page.
#[component]
pub fn App(props: AppProps) -> impl IntoElement {
    let file = match props.settings_path.get_untracked() {
        Some(path) => SettingsFile::File(std::path::PathBuf::from(path.as_ref())),
        None => SettingsFile::user(),
    };
    let store = Store::new(bible(), file, dark());
    provide_context(store);

    // Only explicit Start commands create a native audience surface.
    effect(move || {
        let projection = store.projection.get();
        if let Err(error) = store.output.with(|o| o.send(&projection)) {
            store.projection_status.set(error);
        } else if cfg!(target_os = "android") && projection.active {
            store
                .projection_status
                .set("已送出投影要求；裝置提示會回報外接螢幕連線狀態".into());
        }
    });
    set_interval(Duration::from_millis(250), move || {
        if let Some(status) = store.output.with(|o| o.status()) {
            store.projection_status.set(status);
        }
    });
    follow_theme(store);
    shortcuts(store);
    debug_start_screen(store);

    view()
        .flex_1()
        .bg(move || ui::theme::theme().background)
        .text_color(move || ui::theme::theme().foreground)
        .child(
            when(
                move || store.hebrew_verse.get().is_some(),
                pages::hebrew::hebrew_screen,
            )
            .otherwise(move || {
                view()
                    .flex_1()
                    .child(pages::reader::reader_screen())
                    .child(pages::overlays::overlays())
            }),
        )
}

/// Draws the app with the resolved palette, and re-resolves it when the
/// Omarchy theme or the system appearance changes.
fn follow_theme(store: Store) {
    effect(move || {
        let resolved = store.resolved.get();
        set_tokens(ui::theme::tokens_for(resolved.palette));
        set_color_scheme(if resolved.dark {
            Appearance::Dark
        } else {
            Appearance::Light
        });
    });
    // The system appearance, read in a binding of its own so a change of it
    // re-resolves `System`.
    effect(move || {
        let _ = store.settings.with(|s| s.theme);
        store.refresh_system_theme(untracked(dark));
    });
    let last = stored(omarchy_stamp());
    set_interval(THEME_POLL, move || {
        let stamp = omarchy_stamp();
        if last.replace(stamp) != stamp {
            store.refresh_system_theme(dark());
        }
    });
}

/// The keyboard: `[` `]` chapters, `/` search, `Escape` back out. Ctrl+Q
/// and Cmd+Q quit through the platform, which keeps them.
fn shortcuts(store: Store) {
    let reading = move || {
        store.hebrew_verse.get_untracked().is_none()
            && store.overlay.get_untracked() != Overlay::Search
    };
    on_key("[", move |_| {
        if reading() {
            store.prev_chapter();
        }
    });
    on_key("]", move |_| {
        if reading() {
            store.next_chapter();
        }
    });
    on_key("/", move |_| {
        if store.hebrew_verse.get_untracked().is_none() {
            store.open_overlay(Overlay::Search);
        }
    });
    // Android's back button steps out the same way, and leaves the app
    // only when there is nothing left to close. (An open dialog takes Back
    // itself and reports it through its `on_cancel`.)
    on_back_press(move || {
        if store.can_escape() {
            store.escape();
            true
        } else {
            false
        }
    });
    on_key_with(
        "escape",
        KeyOptions {
            when: nsh_core::keymap::When::Always,
            ..KeyOptions::default()
        },
        move |_| store.escape(),
    );
}

/// Debug builds only: `OMARCHY_BIBLE_SCREEN` opens a screen at start
/// (`books`, `chapters`, `search`, `settings`, `select`, `hebrew`, or a
/// reference such as `約3:16`), for screenshots of each screen.
#[cfg(debug_assertions)]
fn debug_start_screen(store: Store) {
    let Ok(screen) = std::env::var("OMARCHY_BIBLE_SCREEN") else {
        return;
    };
    match screen.as_str() {
        "books" => store.open_overlay(Overlay::BookPicker),
        "chapters" => store.open_overlay(Overlay::ChapterPicker),
        "search" => store.open_overlay(Overlay::Search),
        "settings" => store.open_overlay(Overlay::Settings),
        "select" => {
            store.toggle_select_mode();
            store.tap_verse(1, bible_core::TranslationId::Cuv1919);
            store.tap_verse(3, bible_core::TranslationId::Cuv1919);
        }
        "hebrew" => {
            store.open_hebrew(1);
            store.select_hebrew_word(0);
        }
        reference => {
            store.set_jump_draft(reference);
            store.submit_jump();
        }
    }
}

#[cfg(not(debug_assertions))]
fn debug_start_screen(_: Store) {}
