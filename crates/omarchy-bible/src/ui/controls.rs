//! The reader's controls: `nsh_base` wrapped once with the app's look.
//! Screens use these and never restyle a base control themselves.
//!
//! Sizes follow the GPUI app: chips `px_2 py_1 text_sm`, nav buttons
//! `px_3 py_1`, 6 pt corners, hover and pressed states native.

use nsh_base::{button, input};
use nsh_core::prelude::*;

use super::theme::{reader, scrim, theme};

/// A chip: a quiet text button that shows `active` in the accent.
///
/// `id` is the test id (on its content: `id:<id>` presses the chip);
/// `label` may be a binding.
pub fn chip(
    id: impl AsRef<str>,
    label: impl IntoReactive<SharedString>,
    active: impl Fn() -> bool + 'static,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    let active = std::rc::Rc::new(active);
    let fg = active.clone();
    button()
        .selected(move || active())
        .on_press(move |_: &()| on_press())
        .px_2()
        .py_1()
        .rounded(move || theme().radius)
        .cursor_pointer()
        .hover(|s| s.bg(move || theme().accent))
        .active(|s| s.opacity(0.75))
        .selected_style(|s| s.bg(move || theme().accent))
        .focus(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .child(
            text(label)
                .test_id(id.as_ref())
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

/// A filled arrow button (‹ ›), with a spoken label.
pub fn nav_button(
    id: &str,
    glyph: &'static str,
    label: &'static str,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    button()
        .accessibility_label(label)
        .on_press(move |_: &()| on_press())
        .px_3()
        .py_1()
        .rounded(move || theme().radius)
        .bg(move || theme().accent)
        .cursor_pointer()
        .hover(|s| s.bg(move || theme().border))
        .active(|s| s.opacity(0.75))
        .focus(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .child(
            text(glyph)
                .test_id(id)
                .tooltip(tip(label))
                .text_sm()
                .text_color(move || theme().primary),
        )
}

/// A tooltip bubble with `label`.
pub fn tip(label: impl IntoReactive<SharedString>) -> AnyElement {
    view()
        .px_2()
        .py_1()
        .rounded(move || theme().radius)
        .bg(move || theme().popover)
        .border_1()
        .border_color(move || theme().border)
        .child(
            text(label)
                .text_xs()
                .text_color(move || theme().popover_foreground),
        )
        .into_element()
}

/// A single-line text field bound to `value`.
pub fn field(
    id: &str,
    placeholder: &'static str,
    value: impl Fn() -> String + 'static,
    on_change: impl Fn(&str) + 'static,
    on_submit: impl Fn() + 'static,
) -> nsh_core::element::Input {
    input()
        .value(value)
        .on_change(move |text: &str| on_change(text))
        .on_submit(move |_: &str| on_submit())
        .submit_behavior(SubmitBehavior::Submit)
        .placeholder(placeholder)
        .placeholder_color(|| color(theme().muted_foreground))
        .caret_color(|| color(theme().primary))
        .test_id(id)
        .h(px(32.0))
        .px_3()
        .text_sm()
        .text_color(move || theme().foreground)
        .bg(move || theme().background)
        .rounded(move || theme().radius)
        .border_1()
        .border_color(move || theme().input)
        .focus(|s| s.border_color(move || theme().ring))
}

/// A section label (主題, 字體大小, …): small and muted.
pub fn label(value: impl IntoReactive<SharedString>) -> nsh_core::element::Text {
    text(value)
        .text_xs()
        .text_color(move || theme().muted_foreground)
}

/// A panel title: large and emphasised.
pub fn title(value: impl IntoReactive<SharedString>) -> nsh_core::element::Text {
    text(value)
        .text_lg()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(reader)
}

/// The surface of a picker or panel.
pub fn panel() -> View {
    view()
        .bg(move || theme().card)
        .border_1()
        .border_color(move || theme().border)
        .rounded(move || theme().radius)
        .px_4()
        .py_3()
        .gap_3()
}

/// A panel's header row: its title and a 關閉 chip.
pub fn panel_header(
    name: &'static str,
    close_id: &str,
    on_close: impl Fn() + 'static,
) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .child(title(name))
        .child(chip(close_id, "關閉", || false, on_close))
}

/// The dimmed backdrop behind an overlay; a press on it dismisses.
pub fn backdrop(on_press: impl Fn() + 'static) -> impl IntoElement {
    view()
        .absolute()
        .inset_0()
        .bg(scrim)
        .label("關閉")
        .on_press(on_press)
}
