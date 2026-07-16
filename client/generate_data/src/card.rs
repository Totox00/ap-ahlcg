use cardcode::Code;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Write, fs::read_to_string, path::Path};
use ustr::{Ustr, ustr};

use crate::counter::Counter;

pub type Cards = HashMap<Code, Card>;

#[derive(Debug)]
pub struct Card {
    pub code: Code,
    pub type_code: Ustr,
    pub faction: Option<Ustr>,
    pub name: String,
    pub subname: String,
    pub image: Option<String>,
    pub clues: i64,
    pub victory: i64,
    pub unique: bool,
    pub is_encounter: bool,
    pub is_duplicate: bool,
    pub is_bonded: bool,
}

#[derive(Debug, Deserialize)]
struct RawCard {
    code: String,
    type_code: String,
    faction_name: Option<String>,
    alternate_of_code: Option<String>,
    duplicate_of_code: Option<String>,
    real_name: String,
    #[serde(default)]
    subname: String,
    bonded_to: Option<String>,
    imagesrc: Option<String>,
    backimagesrc: Option<String>,
    #[serde(default)]
    clues: i64,
    #[serde(default)]
    victory: i64,
    #[serde(default)]
    spoiler: i64,
}

#[derive(Debug, Deserialize)]
struct RawCardOverride {
    code: String,
    type_code: Option<String>,
    faction_name: Option<String>,
    real_name: Option<String>,
    subname: Option<String>,
    image: Option<String>,
    clues: Option<i64>,
    victory: Option<i64>,
    spoiler: Option<i64>,
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

    for r#override in serde_json::from_str::<Vec<RawCardOverride>>(&read_to_string(Path::new(file!()).parent().unwrap().join("card_overrides.json")).expect("Failed to read card_overrides.json file"))
        .expect("Failed to parse cards.json file")
    {
        let Some(card) = cards.get_mut(&Code::from_str(r#override.code.as_str())) else {
            panic!("Tried to override non-existent card with code {}", r#override.code)
        };

        if let Some(type_code) = r#override.type_code {
            card.type_code = ustr(&type_code);
        }
        if let Some(faction_name) = r#override.faction_name {
            card.faction = Some(ustr(&faction_name));
        }
        if let Some(real_name) = r#override.real_name {
            card.name = real_name;
        }
        if let Some(subname) = r#override.subname {
            card.subname = subname;
        }
        if let Some(image) = r#override.image {
            card.image = Some(image);
        }
        if let Some(clues) = r#override.clues {
            card.clues = clues;
        }
        if let Some(victory) = r#override.victory {
            card.victory = victory;
        }
        if let Some(spoiler) = r#override.spoiler {
            card.is_encounter = spoiler > 0;
        }
    }

    for code in names.unique() {
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
            card.unique,
        );
    }

    let _ = writeln!(writer, "    _ => None\n  }}\n}}\n");
}

impl Card {
    fn from_raw(raw: RawCard) -> Card {
        Card {
            code: Code::from_str(raw.code.as_str()),
            type_code: ustr(&raw.type_code),
            faction: raw.faction_name.map(|faction| ustr(&faction.escape_debug().to_string())),
            name: if raw.alternate_of_code.is_none() {
                raw.real_name.escape_debug().to_string()
            } else {
                format!("{} (Parallel)", raw.real_name.escape_debug())
            },
            subname: raw.subname.escape_debug().to_string(),
            image: raw.imagesrc.or(raw.backimagesrc).map(|str| str.escape_debug().to_string()),
            clues: raw.clues,
            victory: raw.victory,
            unique: false,
            is_encounter: raw.spoiler > 0,
            is_duplicate: raw.duplicate_of_code.is_some(),
            is_bonded: raw.bonded_to.is_some(),
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
