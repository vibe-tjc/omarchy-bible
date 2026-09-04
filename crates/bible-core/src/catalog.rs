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
    /// Traditional Chinese abbreviation (e.g. `"創"`, `"太"`, `"林前"`).
    pub name_zh_short: &'static str,
    pub name_en: &'static str,
    pub testament: Testament,
}

/// 66-book catalog. Chapter counts live on the loaded [`crate::Bible`] store.
pub static CANON: &[BookMeta; 66] = &[
    BookMeta {
        osis: "Gen",
        book_id: 1,
        name_zh: "創世記",
        name_zh_short: "創",
        name_en: "Genesis",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Exod",
        book_id: 2,
        name_zh: "出埃及記",
        name_zh_short: "出",
        name_en: "Exodus",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Lev",
        book_id: 3,
        name_zh: "利未記",
        name_zh_short: "利",
        name_en: "Leviticus",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Num",
        book_id: 4,
        name_zh: "民數記",
        name_zh_short: "民",
        name_en: "Numbers",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Deut",
        book_id: 5,
        name_zh: "申命記",
        name_zh_short: "申",
        name_en: "Deuteronomy",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Josh",
        book_id: 6,
        name_zh: "約書亞記",
        name_zh_short: "書",
        name_en: "Joshua",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Judg",
        book_id: 7,
        name_zh: "士師記",
        name_zh_short: "士",
        name_en: "Judges",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Ruth",
        book_id: 8,
        name_zh: "路得記",
        name_zh_short: "得",
        name_en: "Ruth",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "1Sam",
        book_id: 9,
        name_zh: "撒母耳記上",
        name_zh_short: "撒上",
        name_en: "1 Samuel",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "2Sam",
        book_id: 10,
        name_zh: "撒母耳記下",
        name_zh_short: "撒下",
        name_en: "2 Samuel",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "1Kgs",
        book_id: 11,
        name_zh: "列王紀上",
        name_zh_short: "王上",
        name_en: "1 Kings",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "2Kgs",
        book_id: 12,
        name_zh: "列王紀下",
        name_zh_short: "王下",
        name_en: "2 Kings",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "1Chr",
        book_id: 13,
        name_zh: "歷代志上",
        name_zh_short: "代上",
        name_en: "1 Chronicles",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "2Chr",
        book_id: 14,
        name_zh: "歷代志下",
        name_zh_short: "代下",
        name_en: "2 Chronicles",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Ezra",
        book_id: 15,
        name_zh: "以斯拉記",
        name_zh_short: "拉",
        name_en: "Ezra",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Neh",
        book_id: 16,
        name_zh: "尼希米記",
        name_zh_short: "尼",
        name_en: "Nehemiah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Esth",
        book_id: 17,
        name_zh: "以斯帖記",
        name_zh_short: "斯",
        name_en: "Esther",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Job",
        book_id: 18,
        name_zh: "約伯記",
        name_zh_short: "伯",
        name_en: "Job",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Ps",
        book_id: 19,
        name_zh: "詩篇",
        name_zh_short: "詩",
        name_en: "Psalms",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Prov",
        book_id: 20,
        name_zh: "箴言",
        name_zh_short: "箴",
        name_en: "Proverbs",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Eccl",
        book_id: 21,
        name_zh: "傳道書",
        name_zh_short: "傳",
        name_en: "Ecclesiastes",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Song",
        book_id: 22,
        name_zh: "雅歌",
        name_zh_short: "歌",
        name_en: "Song of Solomon",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Isa",
        book_id: 23,
        name_zh: "以賽亞書",
        name_zh_short: "賽",
        name_en: "Isaiah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Jer",
        book_id: 24,
        name_zh: "耶利米書",
        name_zh_short: "耶",
        name_en: "Jeremiah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Lam",
        book_id: 25,
        name_zh: "耶利米哀歌",
        name_zh_short: "哀",
        name_en: "Lamentations",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Ezek",
        book_id: 26,
        name_zh: "以西結書",
        name_zh_short: "結",
        name_en: "Ezekiel",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Dan",
        book_id: 27,
        name_zh: "但以理書",
        name_zh_short: "但",
        name_en: "Daniel",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Hos",
        book_id: 28,
        name_zh: "何西阿書",
        name_zh_short: "何",
        name_en: "Hosea",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Joel",
        book_id: 29,
        name_zh: "約珥書",
        name_zh_short: "珥",
        name_en: "Joel",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Amos",
        book_id: 30,
        name_zh: "阿摩司書",
        name_zh_short: "摩",
        name_en: "Amos",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Obad",
        book_id: 31,
        name_zh: "俄巴底亞書",
        name_zh_short: "俄",
        name_en: "Obadiah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Jonah",
        book_id: 32,
        name_zh: "約拿書",
        name_zh_short: "拿",
        name_en: "Jonah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Mic",
        book_id: 33,
        name_zh: "彌迦書",
        name_zh_short: "彌",
        name_en: "Micah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Nah",
        book_id: 34,
        name_zh: "那鴻書",
        name_zh_short: "鴻",
        name_en: "Nahum",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Hab",
        book_id: 35,
        name_zh: "哈巴谷書",
        name_zh_short: "哈",
        name_en: "Habakkuk",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Zeph",
        book_id: 36,
        name_zh: "西番雅書",
        name_zh_short: "番",
        name_en: "Zephaniah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Hag",
        book_id: 37,
        name_zh: "哈該書",
        name_zh_short: "該",
        name_en: "Haggai",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Zech",
        book_id: 38,
        name_zh: "撒迦利亞書",
        name_zh_short: "亞",
        name_en: "Zechariah",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Mal",
        book_id: 39,
        name_zh: "瑪拉基書",
        name_zh_short: "瑪",
        name_en: "Malachi",
        testament: Testament::Old,
    },
    BookMeta {
        osis: "Matt",
        book_id: 40,
        name_zh: "馬太福音",
        name_zh_short: "太",
        name_en: "Matthew",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Mark",
        book_id: 41,
        name_zh: "馬可福音",
        name_zh_short: "可",
        name_en: "Mark",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Luke",
        book_id: 42,
        name_zh: "路加福音",
        name_zh_short: "路",
        name_en: "Luke",
        testament: Testament::New,
    },
    BookMeta {
        osis: "John",
        book_id: 43,
        name_zh: "約翰福音",
        name_zh_short: "約",
        name_en: "John",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Acts",
        book_id: 44,
        name_zh: "使徒行傳",
        name_zh_short: "徒",
        name_en: "Acts",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Rom",
        book_id: 45,
        name_zh: "羅馬書",
        name_zh_short: "羅",
        name_en: "Romans",
        testament: Testament::New,
    },
    BookMeta {
        osis: "1Cor",
        book_id: 46,
        name_zh: "哥林多前書",
        name_zh_short: "林前",
        name_en: "1 Corinthians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "2Cor",
        book_id: 47,
        name_zh: "哥林多後書",
        name_zh_short: "林後",
        name_en: "2 Corinthians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Gal",
        book_id: 48,
        name_zh: "加拉太書",
        name_zh_short: "加",
        name_en: "Galatians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Eph",
        book_id: 49,
        name_zh: "以弗所書",
        name_zh_short: "弗",
        name_en: "Ephesians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Phil",
        book_id: 50,
        name_zh: "腓立比書",
        name_zh_short: "腓",
        name_en: "Philippians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Col",
        book_id: 51,
        name_zh: "歌羅西書",
        name_zh_short: "西",
        name_en: "Colossians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "1Thess",
        book_id: 52,
        name_zh: "帖撒羅尼迦前書",
        name_zh_short: "帖前",
        name_en: "1 Thessalonians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "2Thess",
        book_id: 53,
        name_zh: "帖撒羅尼迦後書",
        name_zh_short: "帖後",
        name_en: "2 Thessalonians",
        testament: Testament::New,
    },
    BookMeta {
        osis: "1Tim",
        book_id: 54,
        name_zh: "提摩太前書",
        name_zh_short: "提前",
        name_en: "1 Timothy",
        testament: Testament::New,
    },
    BookMeta {
        osis: "2Tim",
        book_id: 55,
        name_zh: "提摩太後書",
        name_zh_short: "提後",
        name_en: "2 Timothy",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Titus",
        book_id: 56,
        name_zh: "提多書",
        name_zh_short: "多",
        name_en: "Titus",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Phlm",
        book_id: 57,
        name_zh: "腓利門書",
        name_zh_short: "門",
        name_en: "Philemon",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Heb",
        book_id: 58,
        name_zh: "希伯來書",
        name_zh_short: "來",
        name_en: "Hebrews",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Jas",
        book_id: 59,
        name_zh: "雅各書",
        name_zh_short: "雅",
        name_en: "James",
        testament: Testament::New,
    },
    BookMeta {
        osis: "1Pet",
        book_id: 60,
        name_zh: "彼得前書",
        name_zh_short: "彼前",
        name_en: "1 Peter",
        testament: Testament::New,
    },
    BookMeta {
        osis: "2Pet",
        book_id: 61,
        name_zh: "彼得後書",
        name_zh_short: "彼後",
        name_en: "2 Peter",
        testament: Testament::New,
    },
    BookMeta {
        osis: "1John",
        book_id: 62,
        name_zh: "約翰一書",
        name_zh_short: "約一",
        name_en: "1 John",
        testament: Testament::New,
    },
    BookMeta {
        osis: "2John",
        book_id: 63,
        name_zh: "約翰二書",
        name_zh_short: "約二",
        name_en: "2 John",
        testament: Testament::New,
    },
    BookMeta {
        osis: "3John",
        book_id: 64,
        name_zh: "約翰三書",
        name_zh_short: "約三",
        name_en: "3 John",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Jude",
        book_id: 65,
        name_zh: "猶大書",
        name_zh_short: "猶",
        name_en: "Jude",
        testament: Testament::New,
    },
    BookMeta {
        osis: "Rev",
        book_id: 66,
        name_zh: "啟示錄",
        name_zh_short: "啟",
        name_en: "Revelation",
        testament: Testament::New,
    },
];

/// Extra lookup keys → OSIS id (ASCII keys matched case-insensitively).
static ALIASES: &[(&str, &str)] = &[
    ("ge", "Gen"),
    ("gn", "Gen"),
    ("genesis", "Gen"),
    ("ex", "Exod"),
    ("exo", "Exod"),
    ("exodus", "Exod"),
    ("lev", "Lev"),
    ("le", "Lev"),
    ("nu", "Num"),
    ("nm", "Num"),
    ("numb", "Num"),
    ("dt", "Deut"),
    ("deu", "Deut"),
    ("deut", "Deut"),
    ("jos", "Josh"),
    ("josh", "Josh"),
    ("jdg", "Judg"),
    ("jud", "Judg"),
    ("jdgs", "Judg"),
    ("judg", "Judg"),
    ("ru", "Ruth"),
    ("rut", "Ruth"),
    ("1sa", "1Sam"),
    ("1sam", "1Sam"),
    ("1sm", "1Sam"),
    ("2sa", "2Sam"),
    ("2sam", "2Sam"),
    ("2sm", "2Sam"),
    ("1ki", "1Kgs"),
    ("1kg", "1Kgs"),
    ("1kgs", "1Kgs"),
    ("1kin", "1Kgs"),
    ("2ki", "2Kgs"),
    ("2kg", "2Kgs"),
    ("2kgs", "2Kgs"),
    ("2kin", "2Kgs"),
    ("1ch", "1Chr"),
    ("1chr", "1Chr"),
    ("1chron", "1Chr"),
    ("2ch", "2Chr"),
    ("2chr", "2Chr"),
    ("2chron", "2Chr"),
    ("ezr", "Ezra"),
    ("ezra", "Ezra"),
    ("ne", "Neh"),
    ("neh", "Neh"),
    ("es", "Esth"),
    ("est", "Esth"),
    ("esth", "Esth"),
    ("job", "Job"),
    ("ps", "Ps"),
    ("psa", "Ps"),
    ("psm", "Ps"),
    ("psalm", "Ps"),
    ("psalms", "Ps"),
    ("pr", "Prov"),
    ("pro", "Prov"),
    ("prv", "Prov"),
    ("prov", "Prov"),
    ("ec", "Eccl"),
    ("ecc", "Eccl"),
    ("eccl", "Eccl"),
    ("qoh", "Eccl"),
    ("so", "Song"),
    ("sos", "Song"),
    ("song", "Song"),
    ("cant", "Song"),
    ("ss", "Song"),
    ("is", "Isa"),
    ("isa", "Isa"),
    ("je", "Jer"),
    ("jer", "Jer"),
    ("la", "Lam"),
    ("lam", "Lam"),
    ("eze", "Ezek"),
    ("ezk", "Ezek"),
    ("ezek", "Ezek"),
    ("da", "Dan"),
    ("dan", "Dan"),
    ("ho", "Hos"),
    ("hos", "Hos"),
    ("joe", "Joel"),
    ("joel", "Joel"),
    ("am", "Amos"),
    ("amo", "Amos"),
    ("amos", "Amos"),
    ("ob", "Obad"),
    ("oba", "Obad"),
    ("obad", "Obad"),
    ("jon", "Jonah"),
    ("jnh", "Jonah"),
    ("jonah", "Jonah"),
    ("mi", "Mic"),
    ("mic", "Mic"),
    ("na", "Nah"),
    ("nah", "Nah"),
    ("hab", "Hab"),
    ("zep", "Zeph"),
    ("zeph", "Zeph"),
    ("hag", "Hag"),
    ("zec", "Zech"),
    ("zech", "Zech"),
    ("mal", "Mal"),
    ("mt", "Matt"),
    ("mat", "Matt"),
    ("matt", "Matt"),
    ("mk", "Mark"),
    ("mr", "Mark"),
    ("mar", "Mark"),
    ("mrk", "Mark"),
    ("lk", "Luke"),
    ("lu", "Luke"),
    ("luk", "Luke"),
    ("jn", "John"),
    ("joh", "John"),
    ("jhn", "John"),
    ("ac", "Acts"),
    ("act", "Acts"),
    ("ro", "Rom"),
    ("rm", "Rom"),
    ("rom", "Rom"),
    ("1co", "1Cor"),
    ("1cor", "1Cor"),
    ("2co", "2Cor"),
    ("2cor", "2Cor"),
    ("ga", "Gal"),
    ("gal", "Gal"),
    ("ep", "Eph"),
    ("eph", "Eph"),
    ("php", "Phil"),
    ("phi", "Phil"),
    ("phil", "Phil"),
    ("col", "Col"),
    ("1th", "1Thess"),
    ("1thess", "1Thess"),
    ("1thes", "1Thess"),
    ("2th", "2Thess"),
    ("2thess", "2Thess"),
    ("2thes", "2Thess"),
    ("1ti", "1Tim"),
    ("1tim", "1Tim"),
    ("1tm", "1Tim"),
    ("2ti", "2Tim"),
    ("2tim", "2Tim"),
    ("2tm", "2Tim"),
    ("tit", "Titus"),
    ("ti", "Titus"),
    ("phm", "Phlm"),
    ("phlm", "Phlm"),
    ("heb", "Heb"),
    ("jas", "Jas"),
    ("jam", "Jas"),
    ("jm", "Jas"),
    ("1pe", "1Pet"),
    ("1pet", "1Pet"),
    ("1pt", "1Pet"),
    ("2pe", "2Pet"),
    ("2pet", "2Pet"),
    ("2pt", "2Pet"),
    ("1jn", "1John"),
    ("1jo", "1John"),
    ("1jhn", "1John"),
    ("1john", "1John"),
    ("2jn", "2John"),
    ("2jo", "2John"),
    ("2jhn", "2John"),
    ("2john", "2John"),
    ("3jn", "3John"),
    ("3jo", "3John"),
    ("3jhn", "3John"),
    ("3john", "3John"),
    ("jud", "Jude"),
    ("jude", "Jude"),
    ("re", "Rev"),
    ("rev", "Rev"),
    ("rv", "Rev"),
    ("約翰", "John"),
    ("馬太", "Matt"),
    ("馬可", "Mark"),
    ("路加", "Luke"),
    ("詩篇", "Ps"),
];

fn alias_osis(key: &str) -> Option<&'static str> {
    let ascii = key.is_ascii();
    ALIASES.iter().find_map(|(alias, osis)| {
        let matched = if ascii {
            alias.eq_ignore_ascii_case(key)
        } else {
            *alias == key
        };
        matched.then_some(*osis)
    })
}

pub fn lookup_canon(key: &str) -> Option<&'static BookMeta> {
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    if let Some(meta) = CANON.iter().find(|b| {
        b.osis.eq_ignore_ascii_case(key)
            || b.name_zh == key
            || b.name_zh_short == key
            || b.name_en.eq_ignore_ascii_case(key)
            || key.parse::<u32>().ok() == Some(b.book_id)
    }) {
        return Some(meta);
    }
    let osis = alias_osis(key)?;
    CANON.iter().find(|b| b.osis == osis)
}

/// Parsed scripture reference from user jump input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BibleRef {
    pub book: &'static BookMeta,
    pub chapter: u32,
    pub verse: Option<u32>,
}

/// Parse refs like `約3:16`, `創 1`, `Gen 1:1`, `1Cor13:4`.
pub fn parse_bible_ref(raw: &str) -> Option<BibleRef> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }

    let (head, verse) = match s.rfind(':') {
        Some(idx) => {
            let after = s[idx + 1..].trim();
            if after.is_empty() || !after.chars().all(|c| c.is_ascii_digit()) {
                (s, None)
            } else {
                let v: u32 = after.parse().ok()?;
                if v == 0 {
                    return None;
                }
                (s[..idx].trim_end(), Some(v))
            }
        }
        None => (s, None),
    };

    let head = head.trim_end();
    let chapter_start = head
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_digit())
        .map(|(i, _)| i)
        .last()?;
    let chapter: u32 = head[chapter_start..].parse().ok()?;
    if chapter == 0 {
        return None;
    }
    let book_key = head[..chapter_start].trim();
    if book_key.is_empty() {
        return None;
    }
    let book = lookup_canon(book_key)?;
    Some(BibleRef {
        book,
        chapter,
        verse,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_names_unique_and_lookup() {
        let mut seen = std::collections::BTreeSet::new();
        for b in CANON {
            assert!(seen.insert(b.name_zh_short), "dup {}", b.name_zh_short);
            assert_eq!(lookup_canon(b.name_zh_short).map(|m| m.osis), Some(b.osis));
            assert_eq!(lookup_canon(b.osis).map(|m| m.osis), Some(b.osis));
        }
        assert_eq!(lookup_canon("約").unwrap().osis, "John");
        assert_eq!(lookup_canon("約一").unwrap().osis, "1John");
        assert_eq!(lookup_canon("jn").unwrap().osis, "John");
        assert_eq!(lookup_canon("1jn").unwrap().osis, "1John");
    }

    #[test]
    fn parse_common_refs() {
        let r = parse_bible_ref("約3:16").unwrap();
        assert_eq!(r.book.osis, "John");
        assert_eq!(r.chapter, 3);
        assert_eq!(r.verse, Some(16));

        let r = parse_bible_ref("創 1").unwrap();
        assert_eq!(r.book.osis, "Gen");
        assert_eq!(r.chapter, 1);
        assert_eq!(r.verse, None);

        let r = parse_bible_ref("Gen 1:1").unwrap();
        assert_eq!(r.book.osis, "Gen");
        assert_eq!(r.chapter, 1);
        assert_eq!(r.verse, Some(1));

        let r = parse_bible_ref("1Cor13:4").unwrap();
        assert_eq!(r.book.osis, "1Cor");
        assert_eq!(r.chapter, 13);
        assert_eq!(r.verse, Some(4));

        let r = parse_bible_ref("詩150").unwrap();
        assert_eq!(r.book.osis, "Ps");
        assert_eq!(r.chapter, 150);
        assert!(parse_bible_ref("").is_none());
        assert!(parse_bible_ref("xyz 1").is_none());
    }
}
