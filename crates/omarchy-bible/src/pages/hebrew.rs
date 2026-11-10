//! The Hebrew page of one verse: the CUV and KJV text, the MorphHB line,
//! the word alignment (original / transliteration / literal / idiomatic,
//! read right to left), and the detail of the word tapped.
//!
//! It replaces the reader while open; 返回 or Escape goes back with the
//! verse highlighted.

use bible_core::{HebrewWord, TranslationId, morph_pos_label};
use nsh_core::element::AdaptiveColor;
use nsh_core::prelude::*;

use crate::store::{HebrewSection, Store, store};
use crate::ui::controls::{chip, label};
use crate::ui::layout::compact;
use crate::ui::theme::{reader, theme};

const ROWS: [&str; 4] = ["原文", "音譯", "直譯", "意譯"];

fn or_dash(value: &Option<String>) -> String {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("—")
        .to_owned()
}

fn present(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

pub fn hebrew_screen() -> impl IntoElement {
    let store = store();
    let title = move || -> SharedString {
        let entry = store.entry();
        let chapter = store.chapter.with(|c| c.0.chapter);
        let verse = store.hebrew_verse.get().unwrap_or(1);
        format!("{} {chapter}:{verse} · 希伯來文", entry.meta.name_zh).into()
    };
    view()
        .flex_1()
        .test_id("hebrew-view")
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .px(move || px(if compact() { 12.0 } else { 24.0 }))
                .pt_safe_area_plus(px(12.0))
                .pb_3()
                .border_b_1()
                .border_color(move || theme().border)
                .child(chip(
                    "hebrew-back",
                    "返回",
                    || false,
                    move || store.close_hebrew(),
                ))
                .child(
                    text(title)
                        .flex_1()
                        .min_w(px(0.0))
                        .text_center()
                        .number_of_lines(1)
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(reader),
                )
                .child(view().w(px(64.0))),
        )
        .child(
            // The word detail is a column beside the words when there is
            // room for both, and under them on a phone.
            view()
                .flex_1()
                .min_h(px(0.0))
                .style_with(move |s| {
                    if compact() {
                        s.flex_col()
                    } else {
                        s.flex_row()
                    }
                })
                .child(body(store))
                .child(when(
                    move || store.hebrew_word.get().is_some(),
                    move || detail(store),
                )),
        )
}

fn verse_text(store: Store, id: TranslationId) -> impl Fn() -> SharedString {
    move || {
        let Some(n) = store.hebrew_verse.get() else {
            return SharedString::default();
        };
        store.chapter.with(|c| {
            c.0.verses
                .iter()
                .find(|v| v.number == n)
                .and_then(|v| v.get(id))
                .unwrap_or("")
                .to_owned()
                .into()
        })
    }
}

fn body(store: Store) -> impl IntoElement {
    let words = computed(move || store.hebrew_words());
    scroll().flex_1().min_w(px(0.0)).min_h(px(0.0)).child(
        view()
            .px(move || px(if compact() { 12.0 } else { 24.0 }))
            .py_4()
            .gap_4()
            .child(
                view().gap_1().child(label("和合本")).child(
                    text(verse_text(store, TranslationId::Cuv1919))
                        .text_base()
                        .text_color(reader),
                ),
            )
            .child(
                view().gap_1().child(label("KJV")).child(
                    text(verse_text(store, TranslationId::Kjv))
                        .text_base()
                        .text_color(move || theme().foreground),
                ),
            )
            .child(
                view().gap_1().child(label("原文")).child(
                    text(move || -> SharedString {
                        let line = words.with(|w| {
                            w.iter()
                                .map(|w| w.text.as_str())
                                .collect::<Vec<_>>()
                                .join(" ")
                        });
                        if line.is_empty() {
                            "（無 MorphHB 資料）".into()
                        } else {
                            line.into()
                        }
                    })
                    .test_id("hebrew-line")
                    .text_lg()
                    .text_color(reader),
                ),
            )
            .child(
                view()
                    .gap_2()
                    .child(label(
                        "詞對齊（右→左）· 原文／音譯／直譯／意譯 · 點欄開啟詞詳情",
                    ))
                    .child(
                        view()
                            .flex_row()
                            .items_start()
                            .gap_2()
                            .child(view().pt(px(4.0)).gap_1().children(
                                ROWS.iter().enumerate().map(|(row, name)| {
                                    view()
                                        .h(px(if row == 0 { 32.0 } else { 24.0 }))
                                        .justify_center()
                                        .child(label(*name))
                                }),
                            ))
                            .child(
                                // Hebrew reads right to left: the first word
                                // is the rightmost column, and a long verse
                                // wraps onto further rows, each starting at
                                // the right, rather than scrolling sideways
                                // away from where reading starts.
                                view()
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .flex_row_reverse()
                                    .flex_wrap()
                                    .gap_1()
                                    .child(each(
                                        move || {
                                            words.get().into_iter().enumerate().collect::<Vec<_>>()
                                        },
                                        |(i, _)| *i,
                                        move |word, _| word_column(store, word),
                                    )),
                            ),
                    ),
            ),
    )
}

fn word_column(store: Store, word: ReadSignal<(usize, HebrewWord)>) -> impl IntoElement {
    let index = move || word.with(|(i, _)| *i);
    let selected = move || store.hebrew_word.get() == Some(index());
    let cells = [
        move |w: &HebrewWord| w.text.clone(),
        |w: &HebrewWord| or_dash(&w.translit),
        |w: &HebrewWord| or_dash(&w.gloss_literal),
        |w: &HebrewWord| or_dash(&w.gloss_idiomatic),
    ];
    nsh_base::button()
        .selected(selected)
        .accessibility_label(move || SharedString::from(word.with(|(_, w)| w.text.clone())))
        .on_press(move |_: &()| store.select_hebrew_word(index()))
        .px_1()
        .py_1()
        .gap_1()
        .items_center()
        .rounded(move || theme().radius)
        .cursor_pointer()
        .bg(move || {
            if selected() {
                theme().primary
            } else {
                AdaptiveColor::TRANSPARENT
            }
        })
        .hover(|s| s.bg(move || theme().accent))
        .selected_style(|s| s.bg(move || theme().primary))
        .focus(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .children(cells.into_iter().enumerate().map(move |(row, cell)| {
            let original = row == 0;
            // `each` keys columns by word index, so a test id made here
            // stays with its word.
            let id = format!("hebrew-word-{}-{row}", word.with_untracked(|(i, _)| *i));
            text(move || SharedString::from(word.with(|(_, w)| cell(w))))
                .test_id(&id)
                .min_w(px(56.0))
                .h(px(if original { 32.0 } else { 24.0 }))
                .px_2()
                .text_center()
                .map(|t| {
                    if original {
                        t.text_lg().font_weight(FontWeight::SEMIBOLD)
                    } else {
                        t.text_xs()
                    }
                })
                .text_color(move || {
                    if selected() {
                        theme().primary_foreground
                    } else if original {
                        reader()
                    } else {
                        theme().muted_foreground
                    }
                })
        }))
}

fn detail(store: Store) -> impl IntoElement {
    let word = computed(move || {
        let i = store.hebrew_word.get()?;
        store.hebrew_words().get(i).cloned()
    });
    let strongs = move || word.with(|w| w.as_ref().and_then(|w| present(&w.strongs)));
    let morph = move || word.with(|w| w.as_ref().and_then(|w| present(&w.morph)));
    let pos = move || morph().and_then(|m| morph_pos_label(&m).map(str::to_owned));
    view()
        .test_id("hebrew-word-detail")
        .style_with(move |s| {
            if compact() {
                // Under the words, full width, clear of the navigation bar.
                s.w_full()
                    .max_h(percentage(0.45))
                    .border_t_1()
                    .pb_safe_area_plus(px(12.0))
            } else {
                s.w(px(280.0)).h_full().border_l_1()
            }
        })
        .px_4()
        .py_3()
        .gap_2()
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .child(
            text("詞詳情")
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(reader),
        )
        .child(
            text(move || {
                SharedString::from(
                    word.with(|w| w.as_ref().map(|w| w.text.clone()).unwrap_or_default()),
                )
            })
            .text_xl()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(reader),
        )
        .child(
            view()
                .gap_1()
                .child(accordion(
                    store,
                    HebrewSection::Strongs,
                    "Strong's",
                    strongs,
                    true,
                ))
                .child(accordion(store, HebrewSection::Morph, "詞形", morph, false))
                .child(accordion(store, HebrewSection::Pos, "詞性", pos, false)),
        )
}

/// One accordion row; absent when the word has no value for it.
fn accordion(
    store: Store,
    section: HebrewSection,
    title: &'static str,
    value: impl Fn() -> Option<String> + Copy + 'static,
    accent_body: bool,
) -> impl IntoElement {
    let open = move || store.hebrew_section.get() == Some(section);
    when(
        move || value().is_some(),
        move || {
            view()
                .rounded(move || theme().radius)
                .border_1()
                .border_color(move || theme().border)
                .bg(move || theme().background)
                .child(
                    nsh_base::button()
                        .selected(open)
                        .on_press(move |_: &()| store.toggle_hebrew_section(section))
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .px_3()
                        .py_2()
                        .cursor_pointer()
                        .hover(|s| s.bg(move || theme().accent))
                        .focus(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
                        .child(
                            text(title)
                                .test_id(&format!("hebrew-section-{title}"))
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(reader),
                        )
                        .child(label(move || {
                            SharedString::from(if open() { "▼" } else { "▶" })
                        })),
                )
                .child(when(open, move || {
                    text(move || SharedString::from(value().unwrap_or_default()))
                        .px_3()
                        .pb_2()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || {
                            if accent_body {
                                theme().primary
                            } else {
                                reader()
                            }
                        })
                }))
        },
    )
}
