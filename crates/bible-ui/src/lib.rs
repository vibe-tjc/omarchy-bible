//! GPUI Bible view: 66-book sidebar + bilingual chapter pane.

use bible_core::{Bible, BookEntry, Chapter, Testament};
use gpui::{
    App, Application, Bounds, ClickEvent, Context, FocusHandle, Focusable, FontWeight, KeyBinding,
    SharedString, TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, prelude::*,
    px, rgb, size,
};
use gpui_component::{Root, Theme, ThemeMode, h_flex, v_flex};

actions!(omarchy_bible, [Quit, PrevChapter, NextChapter]);

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

pub fn run_app(bible: Bible) {
    Application::new().run(move |cx: &mut App| {
        gpui_component::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);

        let font_family = pick_cjk_font(cx);
        Theme::global_mut(cx).font_family = font_family.clone();

        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([
            KeyBinding::new("ctrl-q", Quit, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("[", PrevChapter, Some("omarchy_bible")),
            KeyBinding::new("]", NextChapter, Some("omarchy_bible")),
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
                move |window, cx| {
                    let view = cx.new(|cx| BibleView::new(bible, font_for_view.clone(), cx));
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
    bible: Bible,
    book_index: usize,
    chapter: Chapter,
    font_family: SharedString,
    focus_handle: FocusHandle,
}

impl BibleView {
    pub fn new(bible: Bible, font_family: SharedString, cx: &mut Context<Self>) -> Self {
        let book_index = 0;
        let chapter = bible.load_chapter_at(book_index, 1).expect("load 創世記 1");
        Self {
            bible,
            book_index,
            chapter,
            font_family,
            focus_handle: cx.focus_handle(),
        }
    }

    fn current_entry(&self) -> BookEntry {
        self.bible
            .book_at(self.book_index)
            .expect("current book index in range")
    }

    fn goto(&mut self, book_index: usize, chapter: u32, cx: &mut Context<Self>) {
        if let Ok(loaded) = self.bible.load_chapter_at(book_index, chapter) {
            self.book_index = book_index;
            self.chapter = loaded;
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
        let entry = self.current_entry();
        if self.chapter.chapter < entry.chapter_count {
            self.goto_chapter(self.chapter.chapter + 1, cx);
            return;
        }
        if self.book_index + 1 < self.bible.len() {
            self.goto(self.book_index + 1, 1, cx);
        }
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
        let current_chapter = chapter.chapter;
        let entry = self.current_entry();
        let chapter_count = entry.chapter_count;
        let catalog = self.bible.catalog();
        let current_index = self.book_index;

        h_flex()
            .id("bible-root")
            .key_context("omarchy_bible")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::quit))
            .on_action(cx.listener(Self::prev_chapter))
            .on_action(cx.listener(Self::next_chapter))
            .size_full()
            .bg(rgb(BG))
            .text_color(rgb(FG))
            .font_family(self.font_family.clone())
            .child(self.render_sidebar(&catalog, current_index, cx))
            .child(self.render_main(
                title,
                chinese_label,
                english_label,
                current_chapter,
                chapter_count,
                cx,
            ))
    }
}

impl BibleView {
    fn render_sidebar(
        &self,
        catalog: &[BookEntry],
        current_index: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut items: Vec<gpui::AnyElement> = Vec::new();
        let mut last_testament: Option<Testament> = None;

        for entry in catalog {
            if last_testament != Some(entry.meta.testament) {
                last_testament = Some(entry.meta.testament);
                items.push(
                    div()
                        .pt_3()
                        .pb_1()
                        .text_xs()
                        .text_color(rgb(FG_MUTED))
                        .child(entry.meta.testament.label_zh())
                        .into_any_element(),
                );
            }

            let index = entry.index;
            let is_current = index == current_index;
            let name = entry.meta.name_zh;
            items.push(
                div()
                    .id(("book", index))
                    .w_full()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .when(is_current, |d| d.bg(rgb(BG_HOVER)).text_color(rgb(ACCENT)))
                    .when(!is_current, |d| d.hover(|s| s.bg(rgb(BG_HOVER))))
                    .on_click(cx.listener(move |this, event, window, cx| {
                        this.on_book_click(index, event, window, cx);
                    }))
                    .child(name)
                    .into_any_element(),
            );
        }

        v_flex()
            .w(px(220.0))
            .h_full()
            .bg(rgb(BG_SIDEBAR))
            .border_r_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .px_3()
                    .pt_4()
                    .pb_2()
                    .text_xs()
                    .text_color(rgb(FG_MUTED))
                    .child("書卷"),
            )
            .child(
                v_flex()
                    .id("book-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_3()
                    .pb_4()
                    .gap_1()
                    .children(items),
            )
    }

    fn render_main(
        &self,
        title: String,
        chinese_label: String,
        english_label: String,
        current_chapter: u32,
        chapter_count: u32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .flex_1()
            .h_full()
            .min_w_0()
            .child(
                v_flex()
                    .px_6()
                    .py_4()
                    .gap_2()
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
                    )
                    .child(self.render_chapter_nav(current_chapter, chapter_count, cx)),
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
                    .children(self.chapter.verses.iter().map(verse_block)),
            )
    }

    fn render_chapter_nav(
        &self,
        current_chapter: u32,
        chapter_count: u32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let numbers: Vec<gpui::AnyElement> = (1..=chapter_count)
            .map(|n| {
                let is_current = n == current_chapter;
                div()
                    .id(("ch", n as usize))
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .text_sm()
                    .when(is_current, |d| d.bg(rgb(BG_HOVER)).text_color(rgb(ACCENT)))
                    .when(!is_current, |d| {
                        d.text_color(rgb(FG_MUTED)).hover(|s| s.bg(rgb(BG_HOVER)))
                    })
                    .on_click(cx.listener(move |this, event, window, cx| {
                        this.on_chapter_click(n, event, window, cx);
                    }))
                    .child(format!("{n}"))
                    .into_any_element()
            })
            .collect();

        h_flex()
            .w_full()
            .items_center()
            .gap_2()
            .child(nav_button(
                "prev-ch",
                "‹",
                cx.listener(Self::prev_chapter_click),
            ))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(FG_MUTED))
                    .child(format!("{current_chapter} / {chapter_count}")),
            )
            .child(
                h_flex()
                    .id("chapter-numbers")
                    .flex_1()
                    .min_w_0()
                    .overflow_x_scroll()
                    .gap_1()
                    .children(numbers),
            )
            .child(nav_button(
                "next-ch",
                "›",
                cx.listener(Self::next_chapter_click),
            ))
    }

    fn prev_chapter_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.prev_chapter(&PrevChapter, window, cx);
    }

    fn next_chapter_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.next_chapter(&NextChapter, window, cx);
    }
}

fn nav_button(
    id: &'static str,
    label: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_md()
        .bg(rgb(BG_HOVER))
        .text_color(rgb(ACCENT))
        .hover(|s| s.bg(rgb(BORDER)))
        .on_click(on_click)
        .child(label)
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
