//! Projection data and native output transport. No UI is rendered into the
//! projector by the controller: only bounded, plain-text slide data crosses
//! the boundary. The controller window never moves to the audience display.
use bible_core::{Chapter, TranslationId};
use serde::{Deserialize, Serialize};

/// Maximum weighted text per slide. CJK glyphs count twice. Long verses get
/// continuation slides rather than shrinking indefinitely or being clipped.
const PAGE_UNITS: usize = 460;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slide {
    pub reference: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Projection {
    pub active: bool,
    pub blank: bool,
    /// -1 chooses an external presentation display. 0 and up are explicit
    /// native display indices; 0 is useful for a single-monitor preview.
    pub screen: i32,
    pub multiple: bool,
    pub index: usize,
    pub slides: Vec<Slide>,
    /// Changes on every request. Native hosts ignore stale saved commands
    /// when an app starts, so opening the app cannot resume projection.
    pub revision: u64,
    pub session: String,
}

impl Default for Projection {
    fn default() -> Self {
        Self {
            active: false,
            blank: false,
            screen: -1,
            multiple: false,
            index: 0,
            slides: Vec::new(),
            revision: 0,
            session: format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
        }
    }
}
impl Projection {
    pub fn current(&self) -> Option<&Slide> {
        self.slides.get(self.index)
    }
    pub fn step(&mut self, delta: i32) {
        if self.slides.is_empty() {
            return;
        }
        self.index =
            (self.index as i64 + delta as i64).clamp(0, self.slides.len() as i64 - 1) as usize;
        self.revision += 1;
    }
}

fn weight(c: char) -> usize {
    if c.is_ascii() { 1 } else { 2 }
}

/// Splits on Unicode scalar boundaries, preferably at whitespace or
/// punctuation. All content is retained; no HTML/Markdown interpretation.
fn chunks(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut remaining = text;
    while !remaining.is_empty() {
        let mut units = 0;
        let mut end = 0;
        let mut boundary = 0;
        for (at, ch) in remaining.char_indices() {
            if units + weight(ch) > PAGE_UNITS {
                break;
            }
            units += weight(ch);
            end = at + ch.len_utf8();
            if units >= PAGE_UNITS / 2 && (ch.is_whitespace() || "，。；！？、,.!?;".contains(ch))
            {
                boundary = end;
            }
        }
        if end < remaining.len() && boundary > 0 {
            end = boundary;
        }
        result.push(remaining[..end].to_owned());
        remaining = &remaining[end..];
    }
    result
}

/// Selected verse numbers in chapter order. Empty input means the whole
/// chapter (only callers explicitly asking for the chapter use this).
/// Single mode never mixes verses; multi mode packs bounded groups.
pub fn make_slides(
    chapter: &Chapter,
    lanes: &[TranslationId],
    numbers: &[u32],
    multiple: bool,
) -> Vec<Slide> {
    let mut slides: Vec<Slide> = Vec::new();
    let mut group_numbers: Vec<u32> = Vec::new();
    for verse in chapter
        .verses
        .iter()
        .filter(|v| numbers.is_empty() || numbers.contains(&v.number))
    {
        let mut body = String::new();
        for (lane, text) in verse.texts_for(lanes) {
            if !body.is_empty() {
                body.push_str("\n\n");
            }
            if lanes.len() > 1 {
                body.push_str(lane.label());
                body.push('\n');
            }
            body.push_str(&format!("{}  {}", verse.number, text));
        }
        if body.is_empty() {
            continue;
        }
        let pieces = chunks(&body);
        let reference = format!(
            "{} {}:{}",
            chapter.book_name_zh, chapter.chapter, verse.number
        );
        if multiple
            && pieces.len() == 1
            && !group_numbers.is_empty()
            && let Some(last) = slides.last_mut()
        {
            let total: usize = last.text.chars().chain(body.chars()).map(weight).sum();
            if total + 2 <= PAGE_UNITS {
                last.text.push_str("\n\n");
                last.text.push_str(&body);
                group_numbers.push(verse.number);
                last.reference = format!(
                    "{} {}:{}",
                    chapter.book_name_zh,
                    chapter.chapter,
                    group_numbers
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                continue;
            }
        }
        group_numbers.clear();
        if pieces.len() == 1 {
            group_numbers.push(verse.number);
        }
        let count = pieces.len();
        for (i, text) in pieces.into_iter().enumerate() {
            slides.push(Slide {
                reference: if count > 1 {
                    format!("{reference}（{}/{count}）", i + 1)
                } else {
                    reference.clone()
                },
                text,
            });
        }
    }
    slides
}

/// Owns the desktop helper. Closing the controller always terminates its
/// helper; normal Stop merely hides the audience window for quick reuse.
#[cfg(not(target_os = "android"))]
#[derive(Default)]
pub struct NativeOutput {
    child: Option<std::process::Child>,
    events: Option<std::sync::mpsc::Receiver<String>>,
}
#[cfg(not(target_os = "android"))]
#[cfg(not(target_os = "android"))]
impl NativeOutput {
    pub fn send(&mut self, state: &Projection) -> Result<(), String> {
        use std::io::{BufRead, Write};
        if self.child.is_none() && !state.active {
            return Ok(());
        }
        if self.child.is_none() {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            // Installed next to the reader; an explicit override is useful
            // for source builds, never constructed from untrusted content.
            let helper = std::env::var_os("OMARCHY_BIBLE_PROJECTOR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| exe.with_file_name("omarchy-bible-projector"));
            let mut child = std::process::Command::new(helper)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| format!("無法啟動投影視窗：{e}；請先建置 omarchy-bible-projector"))?;
            let stdout = child.stdout.take().ok_or("投影視窗無法回報狀態")?;
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                for line in std::io::BufReader::new(stdout)
                    .lines()
                    .map_while(Result::ok)
                {
                    if tx.send(line).is_err() {
                        break;
                    }
                }
            });
            self.events = Some(rx);
            self.child = Some(child);
        }
        if self
            .child
            .as_mut()
            .unwrap()
            .try_wait()
            .map_err(|e| e.to_string())?
            .is_some()
        {
            self.child = None;
            return Err("投影視窗已關閉，請重新開始投影".into());
        }
        let mut json = serde_json::to_vec(state).map_err(|e| e.to_string())?;
        json.push(b'\n');
        self.child
            .as_mut()
            .unwrap()
            .stdin
            .as_mut()
            .ok_or("投影輸入已關閉")?
            .write_all(&json)
            .map_err(|e| e.to_string())
    }
    pub fn status(&mut self) -> Option<String> {
        self.events.as_ref().and_then(|rx| rx.try_iter().last())
    }
}
#[cfg(not(target_os = "android"))]
impl Drop for NativeOutput {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(target_os = "android")]
#[derive(Default)]
pub struct NativeOutput;
#[cfg(target_os = "android")]
impl NativeOutput {
    pub fn send(&mut self, state: &Projection) -> Result<(), String> {
        let json = serde_json::to_string(state).map_err(|e| e.to_string())?;
        nsh_core::storage::Storage::set_item("projection", json)
            .map_err(|_| "投影資料超過 app 儲存空間".into())
    }
    pub fn status(&mut self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn chapter() -> Chapter {
        crate::bible().load_chapter_at(0, 1).unwrap()
    }
    #[test]
    fn defaults_to_single() {
        assert!(!Projection::default().multiple);
        assert!(!Projection::default().active);
    }
    #[test]
    fn single_never_combines_verses() {
        let s = make_slides(&chapter(), &[TranslationId::Cuv1919], &[1, 2, 3], false);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].reference, "創世記 1:1");
        assert_eq!(s[2].reference, "創世記 1:3");
    }
    #[test]
    fn multiple_groups_and_preserves_non_contiguous_references() {
        let s = make_slides(&chapter(), &[TranslationId::Cuv1919], &[1, 3], true);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].reference, "創世記 1:1, 3");
        assert!(!s[0].text.contains("2  "));
    }
    #[test]
    fn long_text_is_lossless_and_bounded() {
        let t = "神愛世人，".repeat(1000) + &"word ".repeat(1000);
        let c = chunks(&t);
        assert_eq!(c.concat(), t);
        assert!(
            c.iter()
                .all(|s| s.chars().map(weight).sum::<usize>() <= PAGE_UNITS)
        );
    }
    #[test]
    fn navigation_stays_in_bounds() {
        let mut p = Projection::default();
        p.step(-1);
        assert_eq!(p.index, 0);
        p.slides = make_slides(&chapter(), &[TranslationId::Kjv], &[1, 2], false);
        p.step(99);
        assert_eq!(p.index, 1);
        p.step(-99);
        assert_eq!(p.index, 0);
    }
    #[test]
    fn bilingual_keeps_both_lanes() {
        let s = make_slides(&chapter(), TranslationId::ALL, &[1], false);
        let text = s.iter().map(|s| s.text.as_str()).collect::<String>();
        assert!(text.contains("起初"));
        assert!(text.contains("In the beginning"));
    }
}
