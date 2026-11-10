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
use crate::ui::layout::{compact, touch};
use crate::ui::theme::{reader, theme};

pub fn overlays() -> impl IntoElement {
    let store = store();
    let open = move |o: Overlay| move || store.overlay.get() == o;
    view()
        .child(when(open(Overlay::BookPicker), move || {
            sheet_or_panel(store, 96.0, 360.0, None, book_picker(store))
        }))
        .child(when(open(Overlay::ChapterPicker), move || {
            sheet_or_panel(store, 120.0, 420.0, Some(480.0), chapter_picker(store))
        }))
        .child(when(open(Overlay::Search), move || {
            sheet_or_panel(store, 56.0, 560.0, Some(560.0), search_panel(store))
        }))
        .child(when(open(Overlay::Projection), move || {
            sheet_or_panel(
                store,
                32.0,
                640.0,
                Some(720.0),
                crate::pages::projection::projection_panel(),
            )
        }))
        .child(when(open(Overlay::Settings), move || {
            settings_drawer(store)
        }))
}

/// An overlay's dialog. In a wide window `content` is a panel `width`
/// points wide (at most `max_height` tall), floating `top` points below
/// the top over a dimmed backdrop. In a compact (phone) window it is a
/// full-screen sheet instead, padded clear of the status and navigation
/// bars: a 360 pt panel would leave a few points of margin there, and the
/// backdrop would be too small to aim at.
fn sheet_or_panel(
    store: Store,
    top: f32,
    width: f32,
    max_height: Option<f32>,
    content: View,
) -> impl IntoElement {
    dialog()
        .animation(move || {
            if compact() {
                ModalAnimation::Slide
            } else {
                ModalAnimation::Fade
            }
        })
        .on_cancel(move || store.close_overlay())
        .flex_1()
        .items_center()
        .pt(move || px(if compact() { 0.0 } else { top }))
        .child(backdrop(move || store.close_overlay()))
        .child(
            content
                .w(move || {
                    if compact() {
                        relative(1.0).into()
                    } else {
                        Length::from(px(width))
                    }
                })
                .style_with(move |s| {
                    if compact() {
                        // Full screen: the sheet is the page, square and
                        // clear of the bars.
                        s.flex_1()
                            .rounded(px(0.0))
                            .border(px(0.0))
                            .pt_safe_area_plus(px(12.0))
                            .pb_safe_area_plus(px(12.0))
                    } else {
                        // A floating panel sized by its content. (No
                        // `flex_1` here: its zero basis would collapse the
                        // panel to its padding.)
                        let s = s.max_w(percentage(0.92));
                        match max_height {
                            Some(h) => s.max_h(px(h)),
                            None => s,
                        }
                    }
                }),
        )
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
        // Four to a row, sharing the width: 72 pt in the 360 pt panel, a
        // quarter of a phone's full-width sheet.
        .flex_1()
        .basis(px(0.0))
        .min_h(px(44.0))
        .h(move || px(if touch() { 56.0 } else { 44.0 }))
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
                // The full name is a hint with a pointer; on a touch screen
                // it is shown under the abbreviation instead.
                .when(!touch(), |t| {
                    t.tooltip(tip(move || {
                        book.get()
                            .map_or(SharedString::default(), |b| b.meta.name_zh.into())
                    }))
                })
                .text_sm()
                .text_color(move || {
                    if current() {
                        theme().primary
                    } else {
                        theme().muted_foreground
                    }
                }),
        )
        .child(when(touch, move || {
            text(move || SharedString::from(book.get().map_or("", |b| b.meta.name_zh)))
                .text_xs()
                .number_of_lines(1)
                .text_color(move || theme().muted_foreground)
        }))
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
                                    .w(px(if touch() { 52.0 } else { 40.0 }))
                                    .h(px(if touch() { 48.0 } else { 36.0 }))
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
                // A drawer on the right; the whole screen on a phone.
                .w(move || {
                    if compact() {
                        relative(1.0).into()
                    } else {
                        Length::from(px(340.0))
                    }
                })
                .h_full()
                .bg(move || theme().card)
                .border_l_1()
                .border_color(move || theme().border)
                .px_5()
                .pt_safe_area_plus(px(16.0))
                .pb_safe_area_plus(px(16.0))
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
