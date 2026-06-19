use cardcode::Code;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Write, fs::read_to_string, path::Path};

pub type Cards = HashMap<Code, Card>;

#[derive(Debug)]
pub struct Card {
    pub code: Code,
    pub name: String,
    pub subname: String,
    pub image: Option<String>,
    pub clues: i64,
    pub victory: i64,
    pub unique: bool,
}

#[derive(Debug, Deserialize)]
struct RawCard {
    code: String,
    real_name: String,
    #[serde(default)]
    subname: String,
    imagesrc: Option<String>,
    backimagesrc: Option<String>,
    #[serde(default)]
    clues: i64,
    #[serde(default)]
    victory: i64,
}

struct Counter {
    inner: HashMap<String, (Code, bool)>,
}

pub fn get_cards() -> Cards {
    let mut cards = Cards::new();
    let mut names = Counter::new();

    for raw in serde_json::from_str::<Vec<RawCard>>(&read_to_string(Path::new(file!()).parent().unwrap().join("cards.json")).expect("Failed to read cards.json file"))
        .expect("Failed to parse cards.json file")
    {
        let card = Card::from_raw(raw);
        names.add(card.name.clone(), card.code);
        cards.insert(card.code, card);
    }

    for code in names.nique() {
        cards.get_mut(&code).expect("Card does not exist").unique = true;
    }

    cards
}

pub fn push_get_card<T: Write>(writer: &mut T, cards: &Cards) {
    let _ = writeln!(writer, "pub fn get_card(code: Code) -> Option<Card> {{\n  match code.i64() {{");

    for (code, card) in cards {
        let _ = writeln!(
            writer,
            "    {} => Some(Card {{code: Code::from({}), name: \"{}\", image: {}, clues: {}, victory: {}, unique: {}}}),",
            code.i64(),
            card.code.i64(),
            card.unique_name(),
            if let Some(image) = &card.image { format!("Some(\"{image}\")") } else { String::from("None") },
            card.clues,
            card.victory,
            card.unique
        );
    }

    let _ = writeln!(writer, "    _ => None\n  }}\n}}\n");
}

impl Card {
    fn from_raw(raw: RawCard) -> Card {
        Card {
            code: Code::from_str(raw.code.as_str()),
            name: raw.real_name.escape_debug().to_string(),
            subname: raw.subname.escape_debug().to_string(),
            image: raw.imagesrc.or(raw.backimagesrc).map(|str| str.escape_debug().to_string()),
            clues: raw.clues,
            victory: raw.victory,
            unique: false,
        }
    }

    pub fn unique_name(&self) -> String {
        if self.unique { self.name.clone() } else { self.full_name() }
    }

    pub fn full_name(&self) -> String {
        if self.subname.is_empty() {
            self.name.clone()
        } else {
            format!("{} ({})", self.name, self.subname)
        }
    }
}

impl Counter {
    fn new() -> Counter {
        Counter { inner: HashMap::new() }
    }

    fn add(&mut self, name: String, code: Code) {
        if let Some((_, unique)) = self.inner.get_mut(&name) {
            *unique = false;
        } else {
            self.inner.insert(name, (code, true));
        }
    }

    fn nique(&self) -> impl Iterator<Item = Code> {
        self.inner.values().filter(|(_, unique)| *unique).map(|(code, _)| *code)
    }
}
