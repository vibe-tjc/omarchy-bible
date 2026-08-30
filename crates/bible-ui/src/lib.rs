//! GPUI chapter view: sidebar placeholder + bilingual Genesis 1.

use bible_core::Chapter;
use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, FocusHandle, Focusable,
    FontWeight, KeyBinding, SharedString, TitlebarOptions, Window, WindowBounds, WindowOptions,
    actions,
};
use gpui_component::{h_flex, v_flex, Root, Theme, ThemeMode};

actions!(omarchy_bible, [Quit]);

/// Omarchy-like Tokyo Night palette (hardcoded for Phase 1).
const BG: u32 = 0x1a1b26;
const BG_SIDEBAR: u32 = 0x16161e;
const BG_HOVER: u32 = 0x24283b;
const FG: u32 = 0xa9b1d6;
const FG_MUTED: u32 = 0x565f89;
const ACCENT: u32 = 0x7aa2f7;
const FG_CHINESE: u32 = 0xc0caf5;
const BORDER: u32 = 0x292e42;

const CJK_FONT_CANDIDATES: &[&str] = &[
    "Noto Serif CJK TC",
    "Noto Sans CJK TC",
    "Source Han Serif TC",
    "Source Han Sans TC",
    "Noto Serif CJK",
    "Noto Sans CJK",
];

pub fn run_app(chapter: Chapter) {
    Application::new().run(move |cx: &mut App| {
        gpui_component::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);

        let font_family = pick_cjk_font(cx);
        Theme::global_mut(cx).font_family = font_family.clone();

        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([
            KeyBinding::new("ctrl-q", Quit, None),
            KeyBinding::new("cmd-q", Quit, None),
        ]);

        let bounds = Bounds::centered(None, size(px(980.0), px(760.0)), cx);
        let font_for_view = font_family;
        cx.open_window(
            WindowOptions {
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
            },
            {
                let chapter = chapter.clone();
                move |window, cx| {
                    let view = cx.new(|cx| {
                        BibleView::new(chapter.clone(), font_for_view.clone(), cx)
                    });
                    let focus = view.read(cx).focus_handle.clone();
                    window.focus(&focus);
                    cx.new(|cx| Root::new(view, window, cx))
                }
            },
        )
        .expect("open window");

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
        if installed_name.contains("CJK TC") || installed_name.contains("Noto Serif CJK") {
            return SharedString::from(installed_name.clone());
        }
    }
    SharedString::from("sans-serif")
}

pub struct BibleView {
    chapter: Chapter,
    font_family: SharedString,
    focus_handle: FocusHandle,
}

impl BibleView {
    pub fn new(chapter: Chapter, font_family: SharedString, cx: &mut Context<Self>) -> Self {
        Self {
            chapter,
            font_family,
            focus_handle: cx.focus_handle(),
        }
    }

    fn quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }
}

impl Focusable for BibleView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BibleView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chapter = &self.chapter;
        let title = chapter.title_zh();
        let chinese_label = chapter.chinese_label.clone();
        let english_label = chapter.english_label.clone();
        let book_name = chapter.book_name_zh.clone();

        h_flex()
            .id("bible-root")
            .key_context("omarchy_bible")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::quit))
            .size_full()
            .bg(rgb(BG))
            .text_color(rgb(FG))
            .font_family(self.font_family.clone())
            .child(sidebar(book_name))
            .child(main_pane(title, chinese_label, english_label, chapter))
    }
}

fn sidebar(book_name: String) -> impl IntoElement {
    v_flex()
        .w(px(200.0))
        .h_full()
        .bg(rgb(BG_SIDEBAR))
        .border_r_1()
        .border_color(rgb(BORDER))
        .px_3()
        .py_4()
        .gap_2()
        .child(div().text_xs().text_color(rgb(FG_MUTED)).child("書卷"))
        .child(
            div()
                .w_full()
                .px_3()
                .py_2()
                .rounded_md()
                .bg(rgb(BG_HOVER))
                .text_color(rgb(ACCENT))
                .child(book_name),
        )
}

fn main_pane(
    title: String,
    chinese_label: String,
    english_label: String,
    chapter: &Chapter,
) -> impl IntoElement {
    v_flex()
        .flex_1()
        .h_full()
        .min_w_0()
        .child(
            v_flex()
                .px_6()
                .py_4()
                .gap_1()
                .border_b_1()
                .border_color(rgb(BORDER))
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(FG_CHINESE))
                        .child(title),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(FG_MUTED))
                        .child(format!("{chinese_label}  ·  {english_label}")),
                ),
        )
        .child(
            v_flex()
                .id("chapter-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px_6()
                .py_4()
                .gap_5()
                .children(chapter.verses.iter().map(verse_block)),
        )
}

fn verse_block(verse: &bible_core::Verse) -> impl IntoElement {
    h_flex()
        .w_full()
        .items_start()
        .gap_3()
        .child(
            div()
                .w(px(36.0))
                .pt(px(4.0))
                .text_sm()
                .text_color(rgb(ACCENT))
                .child(format!("{}", verse.number)),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_1()
                .child(
                    div()
                        .text_lg()
                        .text_color(rgb(FG_CHINESE))
                        .line_height(px(28.0))
                        .child(verse.chinese.clone()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(FG))
                        .line_height(px(22.0))
                        .child(verse.english.clone()),
                ),
        )
}
