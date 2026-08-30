use bible_core::load_genesis_1;

fn main() {
    let chapter = load_genesis_1().expect("load embedded Genesis 1");
    bible_ui::run_app(chapter);
}
