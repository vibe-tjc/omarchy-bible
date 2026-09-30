//! Cold startup and warm navigation timings (run with --release).
use bible_core::load_bible;
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let bible = load_bible().expect("load bible");
    println!(
        "bilingual store: {:.3} ms",
        start.elapsed().as_secs_f64() * 1000.0
    );
    for (book, chapter) in [("Gen", 1), ("Gen", 2), ("Ps", 119), ("John", 3)] {
        let start = Instant::now();
        black_box(bible.load_chapter(book, chapter).unwrap());
        println!(
            "first {book} {chapter}: {:.3} ms",
            start.elapsed().as_secs_f64() * 1000.0
        );
        let start = Instant::now();
        for _ in 0..1000 {
            black_box(bible.load_chapter(book, chapter).unwrap());
        }
        println!(
            "warm {book} {chapter}: {:.3} us/load",
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
}
