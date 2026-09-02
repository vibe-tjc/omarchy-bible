//! Release-profile bench for bible-core load / chapter clone / search / space.
//!
//!   cargo run --release -p bible-core --example bench -- /path/to/out.json

use bible_core::{load_bible, TranslationId};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const RUNS: usize = 7;
const STRING_HEADER: usize = 24; // ptr + len + cap on 64-bit
const VEC_HEADER: usize = 24; // ptr + len + cap
const TRANSLATION_ID: usize = 8; // enum discriminant padded

fn median_ns(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn time_ns(mut f: impl FnMut()) -> u128 {
    let t = Instant::now();
    f();
    t.elapsed().as_nanos()
}

fn median_of(runs: usize, mut f: impl FnMut()) -> u128 {
    let mut samples = Vec::with_capacity(runs);
    for _ in 0..runs {
        samples.push(time_ns(&mut f));
    }
    median_ns(samples)
}

fn rss_kb() -> Option<u64> {
    let pid = std::process::id();
    let out = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim().parse().ok()
}

fn file_size(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|m| m.len())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// GPUI children per verse_block: root h_flex + number div + lanes v_flex + N lane texts.
fn gpui_nodes_per_verse(lanes: usize) -> usize {
    3 + lanes
}

fn json_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn main() {
    let out_path = env::args().nth(1).map(PathBuf::from);

    // Warm serde / page cache once, then timed runs of a fresh parse each time.
    let _ = load_bible().expect("warm load");

    let mut load_samples = Vec::new();
    let mut last_bible = None;
    for _ in 0..RUNS {
        let t = Instant::now();
        let bible = load_bible().expect("load_bible");
        load_samples.push(t.elapsed().as_nanos());
        last_bible = Some(bible);
    }
    let bible = last_bible.take().unwrap();
    let load_bible_ns = median_ns(load_samples);
    let rss_after_load_kb = rss_kb();

    let gen_idx = bible.index_of("Gen").expect("Gen");
    let ps_idx = bible.index_of("Ps").expect("Ps");

    // Clone-path timings (current load_chapter_at clones Vec<Verse>).
    let mut gen_verses = 0usize;
    let gen_ns = median_of(RUNS, || {
        let ch = bible.load_chapter_at(gen_idx, 1).expect("Gen 1");
        gen_verses = ch.verses.len();
        std::hint::black_box(ch);
    });
    let mut ps_verses = 0usize;
    let ps_ns = median_of(RUNS, || {
        let ch = bible.load_chapter_at(ps_idx, 119).expect("Ps 119");
        ps_verses = ch.verses.len();
        std::hint::black_box(ch);
    });

    let queries = ["神愛世人", "起初", "God so loved"];
    let mut search_rows = Vec::new();
    for q in queries {
        let mut hits = 0usize;
        let ns = median_of(RUNS, || {
            let h = bible.search(q, TranslationId::ALL);
            hits = h.len();
            std::hint::black_box(h);
        });
        search_rows.push((q, ns, hits));
    }

    // Dataset walk (uses load_chapter_at so it matches the clone/Arc path).
    let mut total_verses = 0usize;
    let mut total_chapters = 0usize;
    let mut text_bytes = 0usize;
    let mut string_count = 0usize;
    let mut max_verses = 0usize;
    let mut max_book = String::new();
    let mut max_book_zh = String::new();
    let mut max_chapter = 0u32;
    for entry in bible.catalog() {
        total_chapters += entry.chapter_count as usize;
        for c in 1..=entry.chapter_count {
            let ch = bible.load_chapter_at(entry.index, c).expect("chapter");
            let n = ch.verses.len();
            total_verses += n;
            if n > max_verses {
                max_verses = n;
                max_book = entry.meta.osis.to_string();
                max_book_zh = entry.meta.name_zh.to_string();
                max_chapter = c;
            }
            for v in ch.verses.iter() {
                text_bytes += v.stored_string_bytes();
                string_count += v.stored_string_count();
            }
        }
    }
    let books = bible.len();

    // Overhead: each String header + each texts Vec header + (id, String) pair
    // + each chapter's verse Vec/slice header.
    let pair_bytes = string_count * (TRANSLATION_ID + STRING_HEADER);
    let texts_vec_bytes = total_verses * VEC_HEADER;
    let chapter_vec_bytes = total_chapters * VEC_HEADER;
    let verse_struct_est = total_verses * 16; // number u32 + padding + Vec ptr
    let ram_est = text_bytes
        + string_count * STRING_HEADER
        + pair_bytes
        + texts_vec_bytes
        + chapter_vec_bytes
        + verse_struct_est;

    let json_path = workspace_root().join("data/cuv-kjv.json");
    let json_bytes = file_size(&json_path).unwrap_or(0);

    let ui_single_per = gpui_nodes_per_verse(1);
    let ui_compare_per = gpui_nodes_per_verse(2);
    let gen_single = gen_verses * ui_single_per;
    let gen_compare = gen_verses * ui_compare_per;
    let max_single = max_verses * ui_single_per;
    let max_compare = max_verses * ui_compare_per;

    let rel_bin = workspace_root().join("target/release/omarchy-bible");
    let dbg_bin = workspace_root().join("target/debug/omarchy-bible");
    let rel_size = file_size(&rel_bin);
    let dbg_size = file_size(&dbg_bin);

    let rustc = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string();
    let uname = Command::new("uname")
        .arg("-m")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string();
    let model = Command::new("sysctl")
        .args(["-n", "hw.model"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string();
    let cpu = Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
        .trim()
        .to_string();

    fn ms(ns: u128) -> f64 {
        ns as f64 / 1_000_000.0
    }

    println!("=== bible-core bench (median of {RUNS} release runs) ===");
    println!("env: {model} / {cpu} / {uname} / {rustc}");
    println!("load_bible: {:.3} ms", ms(load_bible_ns));
    println!(
        "load_chapter_at Gen 1: {:.3} ms  verses={gen_verses}",
        ms(gen_ns)
    );
    println!(
        "load_chapter_at Ps 119: {:.3} ms  verses={ps_verses}",
        ms(ps_ns)
    );
    for (q, ns, hits) in &search_rows {
        println!("search {q:?}: {:.3} ms  hits={hits}", ms(*ns));
    }
    println!("books={books} chapters={total_chapters} verses={total_verses}");
    println!("max_chapter={max_book_zh} {max_chapter} ({max_book}) verses={max_verses}");
    println!("json_bytes={json_bytes}");
    println!("text_bytes={text_bytes} string_count={string_count} ram_est_bytes={ram_est}");
    println!(
        "ui nodes/verse: single={ui_single_per} compare={ui_compare_per} (root+number+v_flex+N lanes)"
    );
    println!("ui Gen1 single={gen_single} compare={gen_compare}");
    println!("ui longest single={max_single} compare={max_compare}");
    println!("rss_after_load_kb={rss_after_load_kb:?}");
    println!("release_bin={rel_size:?} debug_bin={dbg_size:?}");

    let mut json = String::new();
    json.push_str("{\n");
    json.push_str(&format!("  \"runs\": {RUNS},\n"));
    json.push_str(&format!("  \"model\": \"{}\",\n", json_escape(&model)));
    json.push_str(&format!("  \"cpu\": \"{}\",\n", json_escape(&cpu)));
    json.push_str(&format!("  \"uname_m\": \"{}\",\n", json_escape(&uname)));
    json.push_str(&format!("  \"rustc\": \"{}\",\n", json_escape(&rustc)));
    json.push_str(&format!("  \"load_bible_ns\": {load_bible_ns},\n"));
    json.push_str(&format!("  \"load_chapter_gen1_ns\": {gen_ns},\n"));
    json.push_str(&format!("  \"load_chapter_gen1_verses\": {gen_verses},\n"));
    json.push_str(&format!("  \"load_chapter_ps119_ns\": {ps_ns},\n"));
    json.push_str(&format!("  \"load_chapter_ps119_verses\": {ps_verses},\n"));
    json.push_str("  \"search\": [\n");
    for (i, (q, ns, hits)) in search_rows.iter().enumerate() {
        let comma = if i + 1 == search_rows.len() { "" } else { "," };
        json.push_str(&format!(
            "    {{\"query\": \"{}\", \"ns\": {ns}, \"hits\": {hits}}}{comma}\n",
            json_escape(q)
        ));
    }
    json.push_str("  ],\n");
    json.push_str(&format!("  \"books\": {books},\n"));
    json.push_str(&format!("  \"chapters\": {total_chapters},\n"));
    json.push_str(&format!("  \"verses\": {total_verses},\n"));
    json.push_str(&format!(
        "  \"max_chapter\": {{\"book\": \"{}\", \"book_zh\": \"{}\", \"chapter\": {max_chapter}, \"verses\": {max_verses}}},\n",
        json_escape(&max_book),
        json_escape(&max_book_zh)
    ));
    json.push_str(&format!("  \"json_bytes\": {json_bytes},\n"));
    json.push_str(&format!("  \"text_bytes\": {text_bytes},\n"));
    json.push_str(&format!("  \"string_count\": {string_count},\n"));
    json.push_str(&format!("  \"ram_est_bytes\": {ram_est},\n"));
    json.push_str(&format!("  \"ui_nodes_per_verse_single\": {ui_single_per},\n"));
    json.push_str(&format!("  \"ui_nodes_per_verse_compare\": {ui_compare_per},\n"));
    json.push_str(&format!("  \"ui_nodes_gen1_single\": {gen_single},\n"));
    json.push_str(&format!("  \"ui_nodes_gen1_compare\": {gen_compare},\n"));
    json.push_str(&format!("  \"ui_nodes_longest_single\": {max_single},\n"));
    json.push_str(&format!("  \"ui_nodes_longest_compare\": {max_compare},\n"));
    json.push_str("  \"ui_formula\": \"3 + N lanes (h_flex + number + v_flex + N texts)\",\n");
    json.push_str("  \"virtualized\": false,\n");
    match rss_after_load_kb {
        Some(v) => json.push_str(&format!("  \"rss_after_load_kb\": {v},\n")),
        None => json.push_str("  \"rss_after_load_kb\": null,\n"),
    }
    match rel_size {
        Some(v) => json.push_str(&format!("  \"release_bin_bytes\": {v},\n")),
        None => json.push_str("  \"release_bin_bytes\": null,\n"),
    }
    match dbg_size {
        Some(v) => json.push_str(&format!("  \"debug_bin_bytes\": {v}\n")),
        None => json.push_str("  \"debug_bin_bytes\": null\n"),
    }
    json.push_str("}\n");

    if let Some(path) = out_path {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        fs::write(&path, &json).expect("write json");
        eprintln!("wrote {}", path.display());
    } else {
        print!("{json}");
    }
}

