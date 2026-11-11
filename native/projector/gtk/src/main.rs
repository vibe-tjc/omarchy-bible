//! Audience-only desktop output for Linux, drawn with GTK 4 (the toolkit
//! Native Shell already uses there). Receives plain-text JSON lines on a
//! private stdin pipe owned by the reader, never shell commands or rich text.
//! No network access. The macOS twin is `native/projector/macos/main.swift`;
//! both speak the same protocol and report the same status lines.

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("omarchy-bible-projector-gtk is the Linux helper; macOS uses native/projector/macos");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn main() {
    linux::main();
}

#[cfg(target_os = "linux")]
mod linux {
    use gtk::{cairo, gdk, glib, pango, prelude::*};
    use gtk4 as gtk;
    use serde_json::{Value, json};
    use std::{cell::RefCell, io::Write, rc::Rc, time::Duration};

    const NO_SCREEN: &str = "找不到外接螢幕；請連接螢幕，或明確選擇螢幕 0 預覽";
    const MAX_INPUT: usize = 262_144;

    fn status(line: &str) {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "{line}");
        let _ = out.flush();
    }

    #[derive(Clone, Default)]
    pub struct Slide {
        text: String,
        reference: String,
        blank: bool,
    }

    enum Plan {
        Off,
        Fail(&'static str),
        Show(usize, Slide),
    }

    /// Screen 0 is the operator's (the compositor's first output). Auto (-1)
    /// takes the first other screen and never falls back to the only one.
    fn plan(state: &Value, screens: usize) -> Plan {
        if !state["active"].as_bool().unwrap_or(false) {
            return Plan::Off;
        }
        let mut index = state["screen"].as_i64().unwrap_or(-1);
        if index < 0 {
            index = if screens > 1 { 1 } else { -1 };
        }
        if index < 0 || index as usize >= screens {
            return Plan::Fail(NO_SCREEN);
        }
        let empty = Vec::new();
        let slides = state["slides"].as_array().unwrap_or(&empty);
        let at = state["index"].as_i64().unwrap_or(0);
        if at < 0 || at as usize >= slides.len() {
            return Plan::Fail("沒有可投影的經文");
        }
        let slide = &slides[at as usize];
        let text = slide["text"].as_str().unwrap_or_default();
        let reference = slide["reference"].as_str().unwrap_or_default();
        if text.chars().count() > 16000 || reference.chars().count() > 2048 {
            return Plan::Fail("投影資料過長");
        }
        Plan::Show(
            index as usize,
            Slide {
                text: text.into(),
                reference: reference.into(),
                blank: state["blank"].as_bool().unwrap_or(false),
            },
        )
    }

    fn layout(cr: &cairo::Context, text: &str, width: f64, px: f64) -> pango::Layout {
        let layout = pangocairo::functions::create_layout(cr);
        let mut font = pango::FontDescription::from_string("Noto Sans CJK TC");
        font.set_absolute_size(px.floor() * pango::SCALE as f64);
        layout.set_font_description(Some(&font));
        layout.set_width((width * pango::SCALE as f64) as i32);
        layout.set_wrap(pango::WrapMode::WordChar);
        layout.set_text(text);
        layout
    }

    /// Black background, verses fitted inside a 6% safe area, reference
    /// centred below. Returns whether the verses fit.
    pub fn paint(cr: &cairo::Context, w: f64, h: f64, slide: &Slide) -> bool {
        cr.set_source_rgb(0.0, 0.0, 0.0);
        let _ = cr.paint();
        if slide.blank {
            return true;
        }
        let margin = (w.min(h) * 0.06).max(24.0);
        let (left, top, width) = (margin, margin, w - 2.0 * margin);
        let height = h - 2.0 * margin;
        let ref_size = (w.min(h) * 0.038).max(16.0);
        let body = height - ref_size * 2.4;
        let fits = |px: f64| {
            let (_, r) = layout(cr, &slide.text, width, px).pixel_extents();
            r.height() as f64 <= body && r.width() as f64 <= width + 1.0
        };
        let (mut low, mut high) = (12.0_f64, 96.0_f64.min(h * 0.095));
        for _ in 0..18 {
            let mid = (low + high) / 2.0;
            if fits(mid) { low = mid } else { high = mid }
        }
        let ok = fits(low);
        let verses = layout(cr, &slide.text, width, low);
        let (_, r) = verses.pixel_extents();
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.move_to(left, top + (body - r.height() as f64) / 2.0);
        pangocairo::functions::show_layout(cr, &verses);
        let reference = layout(cr, &slide.reference, width, ref_size);
        reference.set_alignment(pango::Alignment::Center);
        let (_, r) = reference.pixel_extents();
        let band = top + body + ref_size * 0.6;
        cr.set_source_rgb(210.0 / 255.0, 210.0 / 255.0, 210.0 / 255.0);
        cr.move_to(left, band + (ref_size * 1.8 - r.height() as f64) / 2.0);
        pangocairo::functions::show_layout(cr, &reference);
        ok
    }

    /// Screenshot mode: the same painting path into an image, synthetic
    /// slides only, no display needed.
    fn render_test(args: &[String]) -> ! {
        let w: i32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1920);
        let h: i32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1080);
        let path = args
            .get(4)
            .cloned()
            .unwrap_or_else(|| "projection.png".into());
        let mut slide = Slide {
            reference: "約翰福音 3:16".into(),
            text: "16  神愛世人，甚至將他的獨生子賜給他們，叫一切信他的，不致滅亡，反得永生。\n\nFor God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life.".into(),
            blank: args.iter().any(|a| a == "--blank"),
        };
        if args.iter().any(|a| a == "--long") {
            slide.text = "神愛世人，凡信他的必得永生。".repeat(15);
        }
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h).expect("surface");
        let cr = cairo::Context::new(&surface).expect("context");
        let ok = paint(&cr, w as f64, h as f64, &slide);
        drop(cr);
        let mut file = std::fs::File::create(&path).expect("png path");
        surface.write_to_png(&mut file).expect("png");
        if !slide.blank {
            status(if ok { "FIT OK" } else { "FIT FAILED" });
        }
        std::process::exit(if ok { 0 } else { 3 });
    }

    /// Where the audience window can go. Headless is a single simulated
    /// screen for tests on machines without a display (CI).
    enum Screens {
        Headless,
        Gdk(gtk::gio::ListModel),
    }

    impl Screens {
        fn monitor(&self, i: usize) -> Option<gdk::Monitor> {
            match self {
                Screens::Headless => None,
                Screens::Gdk(m) => m.item(i as u32).and_then(|o| o.downcast().ok()),
            }
        }
        fn count(&self) -> usize {
            match self {
                Screens::Headless => 1,
                Screens::Gdk(m) => m.n_items() as usize,
            }
        }
        fn describe(&self, i: usize) -> (String, i32, i32) {
            match self.monitor(i) {
                None => ("headless".into(), 1920, 1080),
                Some(m) => {
                    let g = m.geometry();
                    let name = m.connector().or_else(|| m.model()).map(|s| s.to_string());
                    (name.unwrap_or_default(), g.width(), g.height())
                }
            }
        }
    }

    struct Projector {
        screens: Screens,
        state: Value,
        slide: Rc<RefCell<Slide>>,
        window: Option<(gtk::Window, gtk::DrawingArea)>,
        output: Option<gdk::Monitor>,
        placed: Option<usize>,
        generation: u64,
    }

    type Shared = Rc<RefCell<Projector>>;

    fn hide(p: &mut Projector) {
        if let Some((window, _)) = p.window.take() {
            window.destroy();
        }
        p.output = None;
        p.placed = None;
    }

    fn quit(line: &str) -> ! {
        status(line);
        std::process::exit(0);
    }

    fn key(keyval: gdk::Key) -> glib::Propagation {
        use gdk::Key;
        // If the compositor focused this window anyway, a presentation
        // remote still works: its keys go back to the controller.
        match keyval {
            Key::Right
            | Key::Page_Down
            | Key::Down
            | Key::space
            | Key::KP_Right
            | Key::KP_Page_Down
            | Key::KP_Down => status("@key next"),
            Key::Left | Key::Page_Up | Key::Up | Key::KP_Left | Key::KP_Page_Up | Key::KP_Up => {
                status("@key prev")
            }
            Key::b | Key::B | Key::period | Key::KP_Decimal => status("@key blank"),
            Key::Escape => quit("投影視窗已關閉"),
            _ => return glib::Propagation::Proceed,
        }
        glib::Propagation::Stop
    }

    fn blank_cursor() -> gdk::Cursor {
        let pixel = glib::Bytes::from_static(&[0, 0, 0, 0]);
        let texture = gdk::MemoryTexture::new(1, 1, gdk::MemoryFormat::R8g8b8a8, &pixel, 4);
        gdk::Cursor::from_texture(&texture, 0, 0, None)
    }

    /// A fresh native window per placement: on Wayland a re-shown surface
    /// can land on the focused (operator's) output instead of the requested
    /// one, e.g. after the display is reconnected.
    fn place(shared: &Shared, monitor: gdk::Monitor) {
        let mut p = shared.borrow_mut();
        hide(&mut p);
        let window = gtk::Window::new();
        window.set_title(Some("聖經 · 投影"));
        window.set_decorated(false);
        let area = gtk::DrawingArea::new();
        let slide = p.slide.clone();
        area.set_draw_func(move |_, cr, w, h| {
            paint(cr, w as f64, h as f64, &slide.borrow());
        });
        // The audience never sees a pointer. A transparent image rather than
        // the "none" name: Hyprland's cursor-shape path ignores "none".
        let blank = blank_cursor();
        area.set_cursor(Some(&blank));
        window.set_cursor(Some(&blank));
        window.set_child(Some(&area));
        let keys = gtk::EventControllerKey::new();
        keys.connect_key_pressed(|_, keyval, _, _| key(keyval));
        window.add_controller(keys);
        window.connect_close_request(|_| quit("投影視窗已關閉"));
        window.fullscreen_on_monitor(&monitor);
        window.present();
        p.output = Some(monitor.clone());
        p.generation += 1;
        let generation = p.generation;
        p.window = Some((window, area));
        drop(p);
        // Omarchy's bar fades out under fullscreen only when it sees a
        // fullscreen change; a window mapped fullscreen sends none, so the
        // bar stays drawn over the verses. Leave and re-enter fullscreen
        // once mapped to send it.
        let weak = Rc::downgrade(shared);
        glib::timeout_add_local_once(Duration::from_millis(1000), move || {
            let Some(shared) = weak.upgrade() else { return };
            let p = shared.borrow();
            let Some((window, _)) = p.window.as_ref().filter(|_| p.generation == generation) else {
                return;
            };
            window.unfullscreen();
            let (window, monitor, weak) = (window.clone(), monitor.clone(), Rc::downgrade(&shared));
            glib::timeout_add_local_once(Duration::from_millis(300), move || {
                let Some(shared) = weak.upgrade() else { return };
                if shared.borrow().generation == generation {
                    window.fullscreen_on_monitor(&monitor);
                }
            });
        });
    }

    fn apply(shared: &Shared, next: Value) {
        let mut p = shared.borrow_mut();
        p.state = next;
        match plan(&p.state, p.screens.count()) {
            Plan::Off => hide(&mut p),
            Plan::Fail(message) => {
                if message == NO_SCREEN {
                    hide(&mut p);
                } else if let Some((window, _)) = p.window.as_ref() {
                    window.set_visible(false);
                }
                status(message);
            }
            Plan::Show(index, slide) => {
                let blank = slide.blank;
                *p.slide.borrow_mut() = slide;
                let (name, w, h) = p.screens.describe(index);
                let monitor = p.screens.monitor(index);
                let same = p.placed == Some(index)
                    && p.output.is_some()
                    && p.output == monitor
                    && p.window.as_ref().is_some_and(|(w, _)| w.is_visible());
                if same {
                    p.window.as_ref().unwrap().1.queue_draw();
                    drop(p);
                } else if let Some(monitor) = monitor {
                    drop(p);
                    place(shared, monitor);
                    shared.borrow_mut().placed = Some(index);
                } else {
                    p.placed = Some(index);
                    drop(p);
                }
                status(&format!(
                    "投影中：螢幕 {index} · {name} · {w}×{h}{}",
                    if blank { "（黑屏）" } else { "" }
                ));
            }
        }
    }

    fn reapply(shared: &Shared) {
        let state = shared.borrow().state.clone();
        apply(shared, state);
    }

    pub fn main() {
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|a| a == "--render-test") {
            render_test(&args);
        }
        let screens = if std::env::var_os("OMARCHY_BIBLE_PROJECTOR_HEADLESS").is_some() {
            Screens::Headless
        } else {
            // Wayland app id, for compositor window rules.
            glib::set_prgname(Some("io.github.vibe_tjc.OmarchyBible.Projector"));
            if gtk::init().is_err() {
                quit("投影視窗無法開啟顯示");
            }
            let display = gdk::Display::default().unwrap_or_else(|| quit("投影視窗無法開啟顯示"));
            Screens::Gdk(display.monitors())
        };
        let shared: Shared = Rc::new(RefCell::new(Projector {
            screens,
            state: json!({}),
            slide: Rc::default(),
            window: None,
            output: None,
            placed: None,
            generation: 0,
        }));
        if let Screens::Gdk(model) = &shared.borrow().screens {
            let weak = Rc::downgrade(&shared);
            model.connect_items_changed(move |model, _, removed, added| {
                let Some(shared) = weak.upgrade() else { return };
                if removed > 0 {
                    let gone = {
                        let p = shared.borrow();
                        p.output.as_ref().is_some_and(|out| {
                            !(0..model.n_items())
                                .any(|i| model.item(i).as_ref() == Some(out.upcast_ref()))
                        })
                    };
                    if gone {
                        hide(&mut shared.borrow_mut());
                    }
                    // Re-evaluate once GDK has settled its monitor list.
                    let weak = Rc::downgrade(&shared);
                    glib::idle_add_local_once(move || {
                        if let Some(shared) = weak.upgrade() {
                            reapply(&shared)
                        }
                    });
                }
                if added > 0 {
                    // A just-connected output is announced before the
                    // compositor can place a fullscreen window on it, and
                    // Omarchy's bar starts there a moment later. Wait.
                    let weak = Rc::downgrade(&shared);
                    glib::timeout_add_local_once(Duration::from_millis(2500), move || {
                        if let Some(shared) = weak.upgrade() {
                            reapply(&shared)
                        }
                    });
                }
            });
        }
        // stdin is read on its own thread and handed to the main loop line
        // by line; the main loop alone owns the windows.
        enum Input {
            Line(Vec<u8>),
            Overflow,
            Eof,
        }
        let (tx, mut rx) = futures_channel::mpsc::unbounded();
        std::thread::spawn(move || {
            use std::io::{BufRead, Read};
            let mut stdin = std::io::stdin().lock();
            loop {
                let mut line = Vec::new();
                let message = match (&mut stdin)
                    .take(MAX_INPUT as u64 + 1)
                    .read_until(b'\n', &mut line)
                {
                    Ok(0) | Err(_) => Input::Eof,
                    Ok(_) if line.len() > MAX_INPUT => Input::Overflow,
                    Ok(_) => Input::Line(line),
                };
                let last = !matches!(message, Input::Line(_));
                if tx.unbounded_send(message).is_err() || last {
                    break;
                }
            }
        });
        let reader = shared.clone();
        glib::MainContext::default().spawn_local(async move {
            use futures_util::StreamExt;
            while let Some(message) = rx.next().await {
                match message {
                    Input::Line(line) if line.ends_with(b"\n") => {
                        if let Ok(value @ Value::Object(_)) = serde_json::from_slice::<Value>(&line)
                        {
                            apply(&reader, value);
                        }
                    }
                    Input::Line(_) => {}
                    Input::Overflow => quit("投影資料超過限制"),
                    // The controller disappeared: never leave old verses up.
                    Input::Eof => std::process::exit(0),
                }
            }
        });
        status("投影視窗已準備；等候外接螢幕");
        glib::MainLoop::new(None, false).run();
    }
}
