fn main() {
    let bible = bible_core::load_bible().expect("load embedded CUV 神版 + KJV");
    bible_ui::run_app(bible);
}
