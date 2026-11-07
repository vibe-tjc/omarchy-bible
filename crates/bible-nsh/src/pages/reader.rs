//! The reading screen: the header (book chip, chapter navigation, mode and
//! tool chips, the jump field), the verse list, and the selection bar.
//!
//! The verses are a `virtual_list`: a chapter can have 176 verses
//! (Psalm 119), each one to several wrapped lanes. A cell shows other verses
//! as it is recycled, so everything in it is bound to `at.index()` and keeps
//! no state of its own; selection lives in the store, keyed by verse and
//! lane.

use bible_core::{TranslationId, VerseUnit, ViewMode};
use nsh_core::element::AdaptiveColor;
use nsh_core::prelude::*;

use crate::store::{Overlay, Store, store};
use crate::ui::controls::{chip, field, label, nav_button};
use crate::ui::theme::{reader, theme};

pub fn reader_screen() -> impl IntoElement {
    let store = store();
    view()
        .flex_1()
        .min_w(px(0.0))
        .child(header(store))
        .child(verse_list(store))
        .child(when(store.select_mode, move || select_bar(store)))
}

fn header(store: Store) -> impl IntoElement {
    view()
        .px_6()
        .py_3()
        .gap_2()
        .border_b_1()
        .border_color(move || theme().border)
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_2()
                .child(view().flex_1().min_w(px(0.0)).flex_row().child(chip(
                    "book-picker-btn",
                    move || {
                        let entry = store.entry();
                        SharedString::from(format!(
                            "{} · {}",
                            entry.meta.testament.label_zh(),
                            entry.meta.name_zh_short
                        ))
                    },
                    move || store.overlay.get() == Overlay::BookPicker,
                    move || store.toggle_overlay(Overlay::BookPicker),
                )))
                .child(chapter_nav(store))
                .child(
                    view()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex_row()
                        .justify_end()
                        .child(header_controls(store)),
                ),
        )
        .child(label(move || {
            SharedString::from(store.settings.with(|s| s.view_mode.header_label()))
        }))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    field(
                        "jump-input",
                        "1:3:5 / 約3:16",
                        move || store.jump_draft.get(),
                        move |text| store.set_jump_draft(text),
                        move || store.submit_jump(),
                    )
                    .flex_1()
                    .label("跳轉到經文")
                    .style_when(store.jump_error, |s| {
                        s.border_color(move || theme().destructive)
                    }),
                )
                .child(chip(
                    "jump-go",
                    "跳轉",
                    || false,
                    move || store.submit_jump(),
                )),
        )
        .child(when(store.jump_error, || {
            text("找不到這段經文，試試「約3:16」或「1:3:5」")
                .text_xs()
                .text_color(move || theme().destructive)
        }))
}

fn chapter_nav(store: Store) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap_2()
        .child(nav_button("prev-ch", "‹", "上一章 [", move || {
            store.prev_chapter()
        }))
        .child(
            text(move || {
                let count = store.entry().chapter_count;
                let chapter = store.chapter.with(|c| c.0.chapter);
                SharedString::from(format!("{chapter}/{count}"))
            })
            .test_id("chapter-position")
            .min_w(px(52.0))
            .text_center()
            .text_sm()
            .tabular_nums()
            .text_color(move || theme().muted_foreground),
        )
        .child(chip(
            "chapter-picker-btn",
            "章",
            move || store.overlay.get() == Overlay::ChapterPicker,
            move || store.toggle_overlay(Overlay::ChapterPicker),
        ))
        .child(nav_button("next-ch", "›", "下一章 ]", move || {
            store.next_chapter()
        }))
}

fn header_controls(store: Store) -> impl IntoElement {
    let is_single = move || store.settings.with(|s| s.view_mode.is_single());
    let current_single = move || {
        store.settings.with(|s| match &s.view_mode {
            ViewMode::Single(id) => *id,
            ViewMode::Compare(ids) => ids.first().copied().unwrap_or(TranslationId::Cuv1919),
        })
    };
    view()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            view()
                .flex_row()
                .gap_1()
                .child(chip("mode-single", "單語", is_single, move || {
                    store.set_view_mode(ViewMode::single(untracked(current_single)))
                }))
                .child(chip(
                    "mode-compare",
                    "對照",
                    move || !is_single(),
                    move || store.set_view_mode(ViewMode::default()),
                )),
        )
        .child(when(is_single, move || {
            view()
                .flex_row()
                .gap_1()
                .children(TranslationId::ALL.iter().map(move |&id| {
                    chip(
                        format!("single-{}", id.label()),
                        id.label(),
                        move || current_single() == id,
                        move || store.set_view_mode(ViewMode::single(id)),
                    )
                }))
        }))
        .child(chip(
            "toggle-select",
            "選擇",
            move || store.select_mode.get(),
            move || store.toggle_select_mode(),
        ))
        .child(chip(
            "open-search",
            "搜尋",
            move || store.overlay.get() == Overlay::Search,
            move || store.open_overlay(Overlay::Search),
        ))
        .child(chip(
            "open-settings",
            "設定",
            move || store.overlay.get() == Overlay::Settings,
            move || store.toggle_overlay(Overlay::Settings),
        ))
}

fn verse_list(store: Store) -> impl IntoElement {
    let list = element_ref();
    // A navigation asks for a verse: scroll there once the rows exist.
    effect(move || {
        let (_, verse) = store.scroll_request.get();
        let index = verse.and_then(|n| {
            untracked(|| {
                store
                    .chapter
                    .with(|c| c.0.verses.iter().position(|v| v.number == n))
            })
        });
        let index = index.unwrap_or(0) as u32;
        set_timeout(std::time::Duration::from_millis(0), move || {
            list.scroll_to_index(index, percentage(0.0), px(0.0), false);
        });
    });
    virtual_list(
        move || store.chapter.with(|c| c.0.verses.len()),
        move |at| verse_cell(store, at),
    )
    // Rows change together with the chapter: identify them by chapter and
    // verse so a new chapter never reuses the old one's measured heights.
    .keys(move || {
        store.chapter.with(|c| {
            let (book, chapter) = (c.0.book_id, c.0.chapter);
            c.0.verses
                .iter()
                .map(|v| (book, chapter, v.number))
                .collect::<Vec<_>>()
        })
    })
    .estimated_item_size(px(96.0))
    .content_padding_x(px(24.0))
    .content_padding_y(px(16.0))
    .row_gap(px(20.0))
    .test_id("verses")
    .flex_1()
    .element_ref(&list)
}

/// One verse's cell. Bound to `at.index()`; holds no state.
fn verse_cell(store: Store, at: Position) -> impl IntoElement {
    let verse = computed(move || store.chapter.with(|c| c.0.verses.get(at.index()).cloned()));
    let number = move || verse.with(|v| v.as_ref().map_or(0, |v| v.number));
    let emphasised = move || {
        let n = number();
        store.highlight.get() == Some(n) || store.selected.with(|s| s.iter().any(|u| u.number == n))
    };
    let has_hebrew = move || {
        verse.with(|v| {
            v.as_ref()
                .and_then(|v| v.hebrew.as_ref())
                .is_some_and(|w| !w.is_empty())
        })
    };
    let num_size = move || px(store.settings.with(|s| s.number_px()));

    view()
        .flex_row()
        .items_start()
        .gap_3()
        .rounded(move || theme().radius)
        .px(move || if emphasised() { px(8.0) } else { px(0.0) })
        .py(move || if emphasised() { px(4.0) } else { px(0.0) })
        .bg(move || {
            if emphasised() {
                theme().accent
            } else {
                AdaptiveColor::TRANSPARENT
            }
        })
        .child(
            view()
                .w(move || px((store.settings.with(|s| s.number_px()) * 2.2).max(28.0)))
                .pt(px(4.0))
                .gap_1()
                .items_center()
                .child(
                    text(move || SharedString::from(number().to_string()))
                        .text_size(num_size)
                        .tabular_nums()
                        .style_with(move |s| {
                            if emphasised() {
                                s.font_weight(FontWeight::SEMIBOLD)
                            } else {
                                s
                            }
                        })
                        .text_color(move || theme().primary),
                )
                .child(
                    nsh_base::button()
                        .accessibility_label("希伯來文形態")
                        .on_press(move |_: &()| store.open_hebrew(untracked(number)))
                        .px_1()
                        .rounded(move || theme().radius)
                        .cursor_pointer()
                        .hover(|s| s.bg(move || theme().accent))
                        .style_with(move |s| if has_hebrew() { s } else { s.hidden() })
                        .child(
                            text("†")
                                .tooltip(crate::ui::controls::tip("希伯來文形態"))
                                .text_sm()
                                .text_color(move || theme().primary),
                        ),
                ),
        )
        .child(
            view().flex_1().min_w(px(0.0)).gap_1().children(
                TranslationId::ALL
                    .iter()
                    .map(move |&id| lane(store, verse, id)),
            ),
        )
}

/// One translation lane of a verse cell: shown while the view mode
/// includes it; selectable in select mode.
fn lane(
    store: Store,
    verse: Memo<Option<bible_core::Verse>>,
    id: TranslationId,
) -> impl IntoElement {
    let shown = move || store.settings.with(|s| s.view_mode.lanes().contains(&id));
    let number = move || verse.with(|v| v.as_ref().map_or(0, |v| v.number));
    let selected = move || {
        store
            .selected
            .with(|s| s.contains(&VerseUnit::new(number(), id)))
    };
    let size = move || {
        store.settings.with(|s| {
            if id.is_cjk() {
                s.chinese_px()
            } else {
                s.english_px()
            }
        })
    };
    view()
        .flex_row()
        .items_start()
        .gap_2()
        .rounded(move || theme().radius)
        .px(move || if selected() { px(4.0) } else { px(0.0) })
        .bg(move || {
            if selected() {
                theme().accent
            } else {
                AdaptiveColor::TRANSPARENT
            }
        })
        .style_with(move |s| if shown() { s } else { s.hidden() })
        .on_press(move || store.tap_verse(untracked(number), id))
        .child(
            view()
                .w(px(16.0))
                .h(px(16.0))
                .mt(px(4.0))
                .rounded(px(3.0))
                .border_1()
                .border_color(move || {
                    if selected() {
                        theme().primary
                    } else {
                        theme().border
                    }
                })
                .bg(move || {
                    if selected() {
                        theme().primary
                    } else {
                        AdaptiveColor::TRANSPARENT
                    }
                })
                .style_with(move |s| {
                    if store.select_mode.get() {
                        s
                    } else {
                        s.hidden()
                    }
                }),
        )
        .child(
            text(move || {
                SharedString::from(
                    verse.with(|v| v.as_ref().and_then(|v| v.get(id)).unwrap_or("").to_owned()),
                )
            })
            .flex_1()
            .min_w(px(0.0))
            .text_size(move || px(size()))
            .line_height(move || px((size() * 1.55).round()))
            .when(id.is_cjk(), |t| {
                t.font_family(crate::ui::theme::READING_FONT_ZH)
            })
            .text_color(move || {
                if id.is_cjk() {
                    reader()
                } else {
                    theme().foreground
                }
            }),
        )
}

fn select_bar(store: Store) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_3()
        .px_6()
        .py_3()
        .border_t_1()
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .child(
            text(move || SharedString::from(store.selection_summary()))
                .test_id("selection-summary")
                .flex_1()
                .min_w(px(0.0))
                .text_sm()
                .text_color(reader),
        )
        .child(
            view()
                .flex_row()
                .gap_2()
                .child(chip(
                    "select-copy",
                    move || {
                        // 已複製 for a moment after a copy.
                        if store.copied.get() > 0 && store.selected.with(|s| !s.is_empty()) {
                            "已複製".into()
                        } else {
                            SharedString::from("複製")
                        }
                    },
                    || false,
                    move || store.copy_selection(),
                ))
                .child(chip(
                    "select-cancel",
                    "取消",
                    || false,
                    move || store.exit_select_mode(),
                )),
        )
}
