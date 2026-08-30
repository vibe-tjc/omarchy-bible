//! Protestant 66-book canon with Traditional Chinese names (和合本).

/// Old or New Testament.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Testament {
    Old,
    New,
}

impl Testament {
    pub fn label_zh(self) -> &'static str {
        match self {
            Testament::Old => "舊約",
            Testament::New => "新約",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "OT" | "ot" => Some(Testament::Old),
            "NT" | "nt" => Some(Testament::New),
            _ => None,
        }
    }
}

/// Static metadata for one of the 66 canonical books.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BookMeta {
    /// OSIS-style identifier (e.g. `"Gen"`, `"1Sam"`, `"Rev"`).
    pub osis: &'static str,
    /// 1-based canonical index (Genesis = 1, Revelation = 66).
    pub book_id: u32,
    pub name_zh: &'static str,
    pub name_en: &'static str,
    pub testament: Testament,
}

/// 66-book catalog. Chapter counts live on the loaded [`crate::Bible`] store.
pub static CANON: &[BookMeta; 66] = &[
    BookMeta {
        osis: "Gen",
        book_id: 1,
        name_zh: "創世記",
        name_en: "Genesis",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Exod",
        book_id: 2,
        name_zh: "出埃及記",
        name_en: "Exodus",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Lev",
        book_id: 3,
        name_zh: "利未記",
        name_en: "Leviticus",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Num",
        book_id: 4,
        name_zh: "民數記",
        name_en: "Numbers",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Deut",
        book_id: 5,
        name_zh: "申命記",
        name_en: "Deuteronomy",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Josh",
        book_id: 6,
        name_zh: "約書亞記",
        name_en: "Joshua",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Judg",
        book_id: 7,
        name_zh: "士師記",
        name_en: "Judges",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Ruth",
        book_id: 8,
        name_zh: "路得記",
        name_en: "Ruth",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "1Sam",
        book_id: 9,
        name_zh: "撒母耳記上",
        name_en: "1 Samuel",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "2Sam",
        book_id: 10,
        name_zh: "撒母耳記下",
        name_en: "2 Samuel",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "1Kgs",
        book_id: 11,
        name_zh: "列王紀上",
        name_en: "1 Kings",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "2Kgs",
        book_id: 12,
        name_zh: "列王紀下",
        name_en: "2 Kings",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "1Chr",
        book_id: 13,
        name_zh: "歷代志上",
        name_en: "1 Chronicles",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "2Chr",
        book_id: 14,
        name_zh: "歷代志下",
        name_en: "2 Chronicles",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Ezra",
        book_id: 15,
        name_zh: "以斯拉記",
        name_en: "Ezra",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Neh",
        book_id: 16,
        name_zh: "尼希米記",
        name_en: "Nehemiah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Esth",
        book_id: 17,
        name_zh: "以斯帖記",
        name_en: "Esther",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Job",
        book_id: 18,
        name_zh: "約伯記",
        name_en: "Job",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Ps",
        book_id: 19,
        name_zh: "詩篇",
        name_en: "Psalms",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Prov",
        book_id: 20,
        name_zh: "箴言",
        name_en: "Proverbs",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Eccl",
        book_id: 21,
        name_zh: "傳道書",
        name_en: "Ecclesiastes",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Song",
        book_id: 22,
        name_zh: "雅歌",
        name_en: "Song of Solomon",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Isa",
        book_id: 23,
        name_zh: "以賽亞書",
        name_en: "Isaiah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Jer",
        book_id: 24,
        name_zh: "耶利米書",
        name_en: "Jeremiah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Lam",
        book_id: 25,
        name_zh: "耶利米哀歌",
        name_en: "Lamentations",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Ezek",
        book_id: 26,
        name_zh: "以西結書",
        name_en: "Ezekiel",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Dan",
        book_id: 27,
        name_zh: "但以理書",
        name_en: "Daniel",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Hos",
        book_id: 28,
        name_zh: "何西阿書",
        name_en: "Hosea",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Joel",
        book_id: 29,
        name_zh: "約珥書",
        name_en: "Joel",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Amos",
        book_id: 30,
        name_zh: "阿摩司書",
        name_en: "Amos",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Obad",
        book_id: 31,
        name_zh: "俄巴底亞書",
        name_en: "Obadiah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Jonah",
        book_id: 32,
        name_zh: "約拿書",
        name_en: "Jonah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Mic",
        book_id: 33,
        name_zh: "彌迦書",
        name_en: "Micah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Nah",
        book_id: 34,
        name_zh: "那鴻書",
        name_en: "Nahum",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Hab",
        book_id: 35,
        name_zh: "哈巴谷書",
        name_en: "Habakkuk",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Zeph",
        book_id: 36,
        name_zh: "西番雅書",
        name_en: "Zephaniah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Hag",
        book_id: 37,
        name_zh: "哈該書",
        name_en: "Haggai",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Zech",
        book_id: 38,
        name_zh: "撒迦利亞書",
        name_en: "Zechariah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Mal",
        book_id: 39,
        name_zh: "瑪拉基書",
        name_en: "Malachi",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Matt",
        book_id: 40,
        name_zh: "馬太福音",
        name_en: "Matthew",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Mark",
        book_id: 41,
        name_zh: "馬可福音",
        name_en: "Mark",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Luke",
        book_id: 42,
        name_zh: "路加福音",
        name_en: "Luke",
        testament: Testament::New,
    },
    BookMeta {
        osis: "John",
        book_id: 43,
        name_zh: "約翰福音",
        name_en: "John",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Acts",
        book_id: 44,
        name_zh: "使徒行傳",
        name_en: "Acts",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Rom",
        book_id: 45,
        name_zh: "羅馬書",
        name_en: "Romans",
        testament: Testament::New,
    },
    BookMeta {
        osis: "1Cor",
        book_id: 46,
        name_zh: "哥林多前書",
        name_en: "1 Corinthians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "2Cor",
        book_id: 47,
        name_zh: "哥林多後書",
        name_en: "2 Corinthians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Gal",
        book_id: 48,
        name_zh: "加拉太書",
        name_en: "Galatians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Eph",
        book_id: 49,
        name_zh: "以弗所書",
        name_en: "Ephesians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Phil",
        book_id: 50,
        name_zh: "腓立比書",
        name_en: "Philippians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Col",
        book_id: 51,
        name_zh: "歌羅西書",
        name_en: "Colossians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "1Thess",
        book_id: 52,
        name_zh: "帖撒羅尼迦前書",
        name_en: "1 Thessalonians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "2Thess",
        book_id: 53,
        name_zh: "帖撒羅尼迦後書",
        name_en: "2 Thessalonians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "1Tim",
        book_id: 54,
        name_zh: "提摩太前書",
        name_en: "1 Timothy",
        testament: Testament::New,
    },
    BookMeta {
        osis: "2Tim",
        book_id: 55,
        name_zh: "提摩太後書",
        name_en: "2 Timothy",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Titus",
        book_id: 56,
        name_zh: "提多書",
        name_en: "Titus",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Phlm",
        book_id: 57,
        name_zh: "腓利門書",
        name_en: "Philemon",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Heb",
        book_id: 58,
        name_zh: "希伯來書",
        name_en: "Hebrews",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Jas",
        book_id: 59,
        name_zh: "雅各書",
        name_en: "James",
        testament: Testament::New,
    },
    BookMeta {
        osis: "1Pet",
        book_id: 60,
        name_zh: "彼得前書",
        name_en: "1 Peter",
        testament: Testament::New,
    },
    BookMeta {
        osis: "2Pet",
        book_id: 61,
        name_zh: "彼得後書",
        name_en: "2 Peter",
        testament: Testament::New,
    },
    BookMeta {
        osis: "1John",
        book_id: 62,
        name_zh: "約翰一書",
        name_en: "1 John",
        testament: Testament::New,
    },
    BookMeta {
        osis: "2John",
        book_id: 63,
        name_zh: "約翰二書",
        name_en: "2 John",
        testament: Testament::New,
    },
    BookMeta {
        osis: "3John",
        book_id: 64,
        name_zh: "約翰三書",
        name_en: "3 John",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Jude",
        book_id: 65,
        name_zh: "猶大書",
        name_en: "Jude",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Rev",
        book_id: 66,
        name_zh: "啟示錄",
        name_en: "Revelation",
        testament: Testament::New,
    },
];

pub fn lookup_canon(key: &str) -> Option<&'static BookMeta> {
    let key = key.trim();
    CANON.iter().find(|b| {
        b.osis.eq_ignore_ascii_case(key)
            || b.name_zh == key
            || b.name_en.eq_ignore_ascii_case(key)
            || key.parse::<u32>().ok() == Some(b.book_id)
    })
}
