//! The projection control desk. Audience output is a separate native
//! window/display, never a mirror of this page.
//!
//! Top to bottom, in the order an operator uses it: what the audience sees
//! (state and preview), moving through verses, the one main action, then
//! the rarely changed content and screen options.
use crate::store::{Store, store};
use crate::ui::controls::{chip, label, nav_button, panel, title};
use crate::ui::layout::target;
use crate::ui::theme::{reader, theme};
use nsh_base::button;
use nsh_core::prelude::*;

pub fn projection_panel() -> View {
    let s = store();
    panel()
        .test_id("projection-panel")
        .child(header(s))
        .child(preview(s))
        .child(navigation(s))
        .child(actions(s))
        .child(label(
            "鍵盤／簡報遙控器：← → 或 PageUp PageDown 換張、換節；B 黑屏。",
        ))
        .child(section("內容"))
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap_2()
                .child(segment(
                    "projection-single",
                    "單節",
                    move || !s.projection.with(|p| p.multiple),
                    move || s.set_projection_multiple(false),
                ))
                .child(segment(
                    "projection-multiple",
                    "多節",
                    move || s.projection.with(|p| p.multiple),
                    move || s.set_projection_multiple(true),
                ))
                .child(view().w(px(12.0)))
                .child(segment(
                    "projection-selection",
                    "目前經文／選取",
                    move || !s.projection_whole_chapter(),
                    move || s.prepare_projection(false),
                ))
                .child(segment(
                    "projection-chapter",
                    "整章",
                    move || s.projection_whole_chapter(),
                    move || s.prepare_projection(true),
                )),
        )
        .child(label(move || {
            if s.projection.with(|p| p.multiple) {
                "多節：選取的經節依序排入同一張，放不下時自動分頁。"
            } else {
                "單節：每張一節。點經文指定，或用「選擇」選取數節。"
            }
        }))
        .child(section("輸出螢幕"))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_2()
                .child(nav_button(
                    "projection-display-prev",
                    "‹",
                    "上一個螢幕",
                    move || s.projection_screen(-1),
                ))
                .child(
                    text(move || {
                        s.projection.with(|p| {
                            if p.screen < 0 {
                                "自動（外接螢幕）".to_string()
                            } else if cfg!(target_os = "android") {
                                format!("外接螢幕 {}", p.screen + 1)
                            } else if p.screen == 0 {
                                "螢幕 0（預覽）".to_string()
                            } else {
                                format!("螢幕 {}", p.screen)
                            }
                        })
                    })
                    .text_sm()
                    .min_w(px(140.0))
                    .text_color(move || theme().foreground),
                )
                .child(nav_button(
                    "projection-display-next",
                    "›",
                    "下一個螢幕",
                    move || s.projection_screen(1),
                )),
        )
        .child(label(
            "自動不會佔用主螢幕；需 HDMI／USB-C 或無線延伸顯示，鏡像不算第二螢幕。",
        ))
}

/// Title, a state badge the operator can read at a glance, and 關閉.
fn header(s: Store) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_2()
                .child(title("投影"))
                .child(
                    view()
                        .px_2()
                        .py_1()
                        .rounded(move || theme().radius)
                        .bg(move || {
                            if s.projection.with(|p| p.active && !p.blank) {
                                theme().primary
                            } else {
                                theme().accent
                            }
                        })
                        .child(
                            text(move || {
                                s.projection.with(|p| match (p.active, p.blank) {
                                    (false, _) => "未投影",
                                    (true, true) => "黑屏中",
                                    (true, false) => "投影中",
                                })
                            })
                            .test_id("projection-state")
                            .text_xs()
                            .text_color(move || {
                                if s.projection.with(|p| p.active && !p.blank) {
                                    theme().primary_foreground
                                } else {
                                    theme().muted_foreground
                                }
                            }),
                        ),
                ),
        )
        .child(chip(
            "projection-close",
            "關閉",
            || false,
            move || s.close_overlay(),
        ))
}

/// What the audience sees (or will see once started), dark like the
/// audience screen so the operator recognises it.
fn preview(s: Store) -> impl IntoElement {
    view()
        .gap_1()
        .child(
            scroll().min_h(px(120.0)).max_h(px(220.0)).child(
                view()
                    .p_3()
                    .gap_2()
                    .rounded(move || theme().radius)
                    .bg(move || theme().background)
                    .border_1()
                    .border_color(move || theme().border)
                    .child(
                        text(move || {
                            s.projection.with(|p| {
                                p.current().map(|s| s.reference.clone()).unwrap_or_default()
                            })
                        })
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(move || theme().primary),
                    )
                    .child(
                        text(move || {
                            s.projection.with(|p| {
                                p.current()
                                    .map(|s| s.text.clone())
                                    .unwrap_or_else(|| "尚未選取投影內容".into())
                            })
                        })
                        .text_base()
                        .text_color(reader),
                    ),
            ),
        )
        .child(label(move || s.projection_status.get()).test_id("projection-status"))
}

/// Verses on the outside (the common move), pages of a long slide set in
/// the middle, shown only when there is more than one.
fn navigation(s: Store) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_2()
        .child(big_button(
            "projection-verse-prev",
            "‹ 上一節",
            false,
            move || s.projection_verse(-1),
        ))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_1()
                .child(when(
                    move || s.projection.with(|p| p.slides.len() > 1),
                    move || {
                        nav_button("projection-prev", "‹", "上一張", move || {
                            s.projection_step(-1)
                        })
                    },
                ))
                .child(
                    label(move || {
                        s.projection.with(|p| {
                            format!(
                                "第 {} / {} 張",
                                if p.slides.is_empty() { 0 } else { p.index + 1 },
                                p.slides.len()
                            )
                        })
                    })
                    .test_id("projection-page"),
                )
                .child(when(
                    move || s.projection.with(|p| p.slides.len() > 1),
                    move || {
                        nav_button("projection-next", "›", "下一張", move || {
                            s.projection_step(1)
                        })
                    },
                )),
        )
        .child(big_button(
            "projection-verse-next",
            "下一節 ›",
            false,
            move || s.projection_verse(1),
        ))
}

/// One main action: 開始投影 when idle; 黑屏 and 停止 while projecting.
fn actions(s: Store) -> impl IntoElement {
    let active = move || s.projection.with(|p| p.active);
    view()
        .child(when(
            move || !active(),
            move || {
                view().flex_row().child(big_button(
                    "projection-start",
                    "開始投影",
                    true,
                    move || s.start_projection(),
                ))
            },
        ))
        .child(when(active, move || {
            view()
                .flex_row()
                .gap_2()
                .child(big_button(
                    "projection-blank",
                    move || {
                        if s.projection.with(|p| p.blank) {
                            "恢復畫面"
                        } else {
                            "黑屏"
                        }
                    },
                    false,
                    move || s.projection_blank(),
                ))
                .child(big_button(
                    "projection-stop",
                    "停止投影",
                    false,
                    move || s.stop_projection(),
                ))
        }))
}

fn section(name: &'static str) -> impl IntoElement {
    view()
        .pt_1()
        .border_t_1()
        .border_color(move || theme().border)
        .child(
            text(name)
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || theme().foreground),
        )
}

/// A chip that reads as one option of a pair: outlined, filled when on.
fn segment(
    id: &'static str,
    name: &'static str,
    on: impl Fn() -> bool + 'static,
    press: impl Fn() + 'static,
) -> impl IntoElement {
    let on = std::rc::Rc::new(on);
    let (bg, fg) = (on.clone(), on.clone());
    button()
        .on_press(move |_: &()| press())
        .px_3()
        .py_1()
        .min_h(px(target()))
        .justify_center()
        .rounded(move || theme().radius)
        .border_1()
        .border_color(move || theme().border)
        .bg(move || if bg() { theme().accent } else { theme().card })
        .cursor_pointer()
        .hover(|s| s.bg(move || theme().accent))
        .focus(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .child(
            text(name)
                .test_id(id)
                .text_sm()
                .number_of_lines(1)
                .text_color(move || {
                    if fg() {
                        theme().primary
                    } else {
                        theme().foreground
                    }
                }),
        )
}

/// A large button for the operator's frequent actions; `primary` fills it.
fn big_button(
    id: &'static str,
    name: impl IntoReactive<SharedString>,
    primary: bool,
    press: impl Fn() + 'static,
) -> impl IntoElement {
    button()
        .flex_1()
        .on_press(move |_: &()| press())
        .px_4()
        .py_2()
        .min_h(px(target().max(40.0)))
        .items_center()
        .justify_center()
        .rounded(move || theme().radius)
        .bg(move || {
            if primary {
                theme().primary
            } else {
                theme().accent
            }
        })
        .cursor_pointer()
        .hover(|s| s.opacity(0.85))
        .active(|s| s.opacity(0.7))
        .focus(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .child(
            text(name)
                .test_id(id)
                .text_base()
                .font_weight(FontWeight::SEMIBOLD)
                .number_of_lines(1)
                .text_color(move || {
                    if primary {
                        theme().primary_foreground
                    } else {
                        theme().foreground
                    }
                }),
        )
}

/// Called on an explicit user press, not when the root mounts.
pub fn open(s: Store) {
    s.prepare_projection(false);
    s.open_overlay(crate::store::Overlay::Projection);
}
