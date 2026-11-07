//! The overlays over the reader: the book picker, the chapter picker, the
//! search panel and the settings panel.
//!
//! One is open at a time ([`Overlay`]), each in a `dialog` with a fading
//! backdrop: Escape and a press on the backdrop close it, and focus returns
//! to the reader. The dialog is in the view only while it is open, so its
//! content and drafts are made fresh each time (the search query is kept in
//! the store, so reopening search shows the last results).

use std::time::Duration;

use bible_core::Testament;
use nsh_base::dialog;
use nsh_core::prelude::*;

use crate::settings::{
    FONT_LARGE, FONT_MEDIUM, FONT_SMALL, ThemePreference, settings_location_note_zh,
};
use crate::store::{BOOK_GRID_PAGE_SIZE, Overlay, Store, store};
use crate::ui::controls::{backdrop, chip, field, label, nav_button, panel, panel_header, tip};
use crate::ui::theme::{reader, theme};

pub fn overlays() -> impl IntoElement {
    let store = store();
    let open = move |o: Overlay| move || store.overlay.get() == o;
    view()
        .child(when(open(Overlay::BookPicker), move || {
            centered(store, 96.0, book_picker(store).w(px(360.0)))
        }))
        .child(when(open(Overlay::ChapterPicker), move || {
            centered(
                store,
                120.0,
                chapter_picker(store).w(px(420.0)).max_h(px(480.0)),
            )
        }))
        .child(when(open(Overlay::Search), move || {
            centered(
                store,
                56.0,
                search_panel(store).w(px(560.0)).max_h(px(560.0)),
            )
        }))
        .child(when(open(Overlay::Settings), move || {
            settings_drawer(store)
        }))
}

/// A dialog with a backdrop and `content` centered `top` points below the
/// top of the window.
fn centered(store: Store, top: f32, content: View) -> impl IntoElement {
    dialog()
        .animation(ModalAnimation::Fade)
        .on_cancel(move || store.close_overlay())
        .flex_1()
        .items_center()
        .pt(px(top))
        .child(backdrop(move || store.close_overlay()))
        .child(content.max_w(percentage(0.92)))
}

// ── Book picker ──────────────────────────────────────────────────────

fn book_picker(store: Store) -> View {
    let testament = move |t: Testament| move || store.book_picker_testament.get() == t;
    panel()
        .test_id("book-picker")
        .child(panel_header(
            "選擇書卷",
            "book-picker-close",
            move || store.close_overlay(),
        ))
        .child(
            view()
                .flex_row()
                .gap_1()
                .child(chip(
                    "book-tab-ot",
                    "舊約",
                    testament(Testament::Old),
                    move || store.set_book_testament(Testament::Old),
                ))
                .child(chip(
                    "book-tab-nt",
                    "新約",
                    testament(Testament::New),
                    move || store.set_book_testament(Testament::New),
                )),
        )
        .child(view().gap_1().children((0..4).map(move |row| {
            view()
                .flex_row()
                .justify_between()
                .gap_1()
                .children((0..4).map(move |col| book_cell(store, row * 4 + col)))
        })))
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_center()
                .gap_3()
                .child(nav_button(
                    "book-page-prev",
                    "‹",
                    "上一頁",
                    move || store.shift_book_page(-1),
                ))
                .child(dynamic(move || {
                    let pages = store.book_pages();
                    view()
                        .flex_row()
                        .gap_1()
                        .items_center()
                        .children((0..pages).map(move |i| {
                            view()
                                .w(px(8.0))
                                .h(px(8.0))
                                .rounded_full()
                                .label(format!("第 {} 頁", i + 1))
                                .bg(move || {
                                    if store.book_picker_page.get() == i {
                                        theme().primary
                                    } else {
                                        theme().border
                                    }
                                })
                                .on_press(move || store.book_picker_page.set(i))
                        }))
                }))
                .child(nav_button(
                    "book-page-next",
                    "›",
                    "下一頁",
                    move || store.shift_book_page(1),
                )),
        )
}

/// The `slot`th cell of the 4×4 grid: a book on the shown page, or empty.
fn book_cell(store: Store, slot: usize) -> impl IntoElement {
    debug_assert!(slot < BOOK_GRID_PAGE_SIZE);
    let book = computed(move || store.book_page().get(slot).copied().flatten());
    let current = move || {
        book.get()
            .is_some_and(|b| b.index == store.book_index.get())
    };
    let name = move || book.get().map_or("", |b| b.meta.name_zh_short);
    nsh_base::button()
        .disabled(move || book.get().is_none())
        .accessibility_label(move || book.get().map(|b| SharedString::from(b.meta.name_zh)))
        .on_press(move |_: &()| {
            if let Some(b) = book.get_untracked() {
                store.goto(b.index, 1, None);
            }
        })
        .w(px(72.0))
        .h(px(44.0))
        .items_center()
        .justify_center()
        .rounded(move || theme().radius)
        .cursor_pointer()
        .bg(move || {
            if current() {
                theme().accent
            } else {
                theme().card
            }
        })
        .hover(|s| s.bg(move || theme().accent))
        .active(|s| s.opacity(0.75))
        .focus(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .child(
            text(move || SharedString::from(name()))
                .tooltip(tip(move || {
                    book.get()
                        .map_or(SharedString::default(), |b| b.meta.name_zh.into())
                }))
                .text_sm()
                .text_color(move || {
                    if current() {
                        theme().primary
                    } else {
                        theme().muted_foreground
                    }
                }),
        )
}

// ── Chapter picker ───────────────────────────────────────────────────

fn chapter_picker(store: Store) -> View {
    panel()
        .test_id("chapter-picker")
        .child(panel_header(
            "選擇章節",
            "chapter-picker-close",
            move || store.close_overlay(),
        ))
        .child(
            scroll()
                .flex_shrink(1.0)
                .min_h(px(0.0))
                .child(dynamic(move || {
                    let count = store.entry().chapter_count;
                    untracked(|| {
                        view()
                            .flex_row()
                            .flex_wrap()
                            .gap_1()
                            .children((1..=count).map(move |n| {
                                let current = move || store.chapter.with(|c| c.0.chapter) == n;
                                nsh_base::button()
                                    .accessibility_label(format!("第 {n} 章"))
                                    .on_press(move |_: &()| store.goto_chapter(n))
                                    .w(px(40.0))
                                    .h(px(36.0))
                                    .items_center()
                                    .justify_center()
                                    .rounded(move || theme().radius)
                                    .cursor_pointer()
                                    .bg(move || {
                                        if current() {
                                            theme().accent
                                        } else {
                                            theme().card
                                        }
                                    })
                                    .hover(|s| s.bg(move || theme().accent))
                                    .active(|s| s.opacity(0.75))
                                    .focus(|s| {
                                        s.outline(px(2.0)).outline_color(move || theme().ring)
                                    })
                                    .child(
                                        text(n.to_string())
                                            .test_id(&format!("chapter-{n}"))
                                            .text_sm()
                                            .tabular_nums()
                                            .text_color(move || {
                                                if current() {
                                                    theme().primary
                                                } else {
                                                    theme().muted_foreground
                                                }
                                            }),
                                    )
                            }))
                    })
                })),
        )
}

// ── Search ───────────────────────────────────────────────────────────

/// Typing debounce, as the GPUI app's.
const SEARCH_DEBOUNCE: Duration = Duration::from_millis(180);

fn search_panel(store: Store) -> View {
    // Debounce: each change cancels the last wait (the effect's rerun
    // disposes the task it spawned), so only the last query runs.
    effect(move || {
        let query = store.search_query.get();
        if query.trim().is_empty() {
            return;
        }
        spawn(async move {
            timer(SEARCH_DEBOUNCE).await;
            store.run_search();
        });
    });
    let status = move || -> SharedString {
        let total = store.search_total.get();
        let shown = store.search_hits.with(|h| h.len());
        if store.search_query.with(|q| q.trim().is_empty()) {
            "輸入關鍵字（預設搜尋目前譯本）".into()
        } else if total == 0 {
            "沒有符合的經文".into()
        } else if shown < total {
            format!("顯示 {shown}／共 {total} 筆").into()
        } else {
            format!("{total} 筆").into()
        }
    };
    panel()
        .test_id("search-panel")
        .child(panel_header("搜尋", "search-close", move || {
            store.close_overlay()
        }))
        .child(
            field(
                "search-input",
                "搜尋經文…",
                move || store.search_query.get(),
                move |text| store.set_search_query(text),
                move || store.open_first_hit(),
            )
            .label("搜尋經文")
            .auto_focus(true),
        )
        .child(
            view()
                .flex_row()
                .gap_1()
                .child(chip(
                    "search-current",
                    "目前譯本",
                    move || !store.search_all_lanes.get(),
                    move || store.set_search_all_lanes(false),
                ))
                .child(chip(
                    "search-all",
                    "全部譯本",
                    move || store.search_all_lanes.get(),
                    move || store.set_search_all_lanes(true),
                )),
        )
        .child(label(status).test_id("search-status"))
        .child(
            // Sized by its rows and shrunk to the panel's max height (a
            // `flex_1` scroll in a panel with no height has none).
            scroll()
                .flex_shrink(1.0)
                .min_h(px(0.0))
                .child(view().gap_1().child(each(
                    move || store.search_hits.with(|h| h.as_ref().clone()),
                    |hit| (hit.book_index, hit.chapter, hit.verse, hit.translation),
                    move |hit, _| search_row(store, hit),
                ))),
        )
}

fn search_row(store: Store, hit: ReadSignal<bible_core::SearchHit>) -> impl IntoElement {
    nsh_base::button()
        .on_press(move |_: &()| store.open_hit(&hit.get_untracked()))
        .w_full()
        .px_3()
        .py_2()
        .gap_1()
        .rounded(move || theme().radius)
        .cursor_pointer()
        .hover(|s| s.bg(move || theme().accent))
        .active(|s| s.opacity(0.75))
        .focus(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    text(move || SharedString::from(hit.with(|h| h.ref_zh())))
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(reader),
                )
                .child(label(move || {
                    SharedString::from(hit.with(|h| h.translation.label()))
                })),
        )
        .child(
            text(move || SharedString::from(hit.with(|h| h.snippet.clone())))
                .text_sm()
                .text_color(move || theme().foreground),
        )
}

// ── Settings ─────────────────────────────────────────────────────────

/// The settings panel: a drawer on the right edge, full height.
fn settings_drawer(store: Store) -> impl IntoElement {
    let theme_pref = move || store.settings.with(|s| s.theme);
    let font = move || store.settings.with(|s| s.font_size);
    dialog()
        .animation(ModalAnimation::Fade)
        .on_cancel(move || store.close_overlay())
        .flex_1()
        .flex_row()
        .justify_end()
        .child(backdrop(move || store.close_overlay()))
        .child(
            view()
                .test_id("settings-panel")
                .w(px(340.0))
                .h_full()
                .bg(move || theme().card)
                .border_l_1()
                .border_color(move || theme().border)
                .px_5()
                .py_4()
                .gap_4()
                .child(panel_header("設定", "settings-close", move || {
                    store.close_overlay()
                }))
                .child(
                    view()
                        .gap_2()
                        .child(label("主題"))
                        .child(
                            view().flex_row().flex_wrap().gap_1().children(
                                [
                                    ThemePreference::Dark,
                                    ThemePreference::Light,
                                    ThemePreference::System,
                                ]
                                .into_iter()
                                .map(move |pref| {
                                    chip(
                                        format!("theme-{}", pref.label_zh()),
                                        pref.label_zh(),
                                        move || theme_pref() == pref,
                                        move || store.set_theme(pref),
                                    )
                                }),
                            ),
                        )
                        .child(when(
                            move || theme_pref() == ThemePreference::System,
                            move || {
                                label(move || {
                                    let r = store.resolved.get();
                                    SharedString::from(r.source.note_zh(r.dark))
                                })
                            },
                        )),
                )
                .child(
                    view()
                        .gap_2()
                        .child(label("字體大小"))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(chip(
                                    "font-minus",
                                    "−",
                                    || false,
                                    move || store.bump_font(-2),
                                ))
                                .child(
                                    text(move || SharedString::from(format!("{} px", font())))
                                        .test_id("font-size")
                                        .min_w(px(56.0))
                                        .text_sm()
                                        .tabular_nums()
                                        .text_color(reader),
                                )
                                .child(chip(
                                    "font-plus",
                                    "+",
                                    || false,
                                    move || store.bump_font(2),
                                )),
                        )
                        .child(
                            view()
                                .flex_row()
                                .gap_1()
                                .child(chip(
                                    "font-small",
                                    "小",
                                    move || font() == FONT_SMALL,
                                    move || store.set_font_size(FONT_SMALL),
                                ))
                                .child(chip(
                                    "font-medium",
                                    "中",
                                    move || font() == FONT_MEDIUM,
                                    move || store.set_font_size(FONT_MEDIUM),
                                ))
                                .child(chip(
                                    "font-large",
                                    "大",
                                    move || font() == FONT_LARGE,
                                    move || store.set_font_size(FONT_LARGE),
                                )),
                        ),
                )
                .child(label(settings_location_note_zh())),
        )
}
