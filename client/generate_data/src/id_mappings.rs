use std::{collections::HashMap, fmt::Write};

use ustr::Ustr;

use crate::id_py::{Item, Location};

pub fn push_item_from_id<T: Write>(writer: &mut T, item_from_id: &HashMap<i64, Item>) {
    let _ = writeln!(writer, "pub fn item_from_id(id: i64) -> Option<Item> {{\n  match id {{");

    for (id, item) in item_from_id {
        let _ = match item {
            Item::Unlock((campaign, unlock)) => writeln!(writer, "    {id} => Some(Item::Unlock((\"{campaign}\", \"{unlock}\"))),"),
            Item::ScenarioUnlock((campaign, scenario)) => writeln!(writer, "    {id} => Some(Item::ScenarioUnlock((\"{campaign}\", \"{scenario}\"))),"),
            Item::Xp((campaign, xp)) => writeln!(writer, "    {id} => Some(Item::Xp((\"{campaign}\", {xp}))),"),
            Item::ScenarioCard((campaign, card)) => writeln!(writer, "    {id} => Some(Item::ScenarioCard((\"{campaign}\", Code::from({})))),", card.i64()),
            Item::CampaignFiller((campaign, filler)) => writeln!(writer, "    {id} => Some(Item::CampaignFiller((\"{campaign}\", \"{filler}\"))),"),
        };
    }

    let _ = writeln!(writer, "    _ => None\n  }}\n}}\n");
}

pub fn push_get_clue_id<T: Write>(writer: &mut T, clue_n_map: &HashMap<Location, i64>) {
    let _ = writeln!(
        writer,
        "pub fn get_clue_id(campaign: &str, scenario: &str, code: Code, n: i64) -> Option<i64> {{\n  match (campaign, scenario, code.i64(), n) {{"
    );

    for ((campaign, scenario, code, clue), n) in clue_n_map {
        let _ = writeln!(writer, "    (\"{campaign}\", \"{scenario}\", {}, {clue}) => Some({n}),", code.i64());
    }

    let _ = writeln!(writer, "    _ => None\n  }}\n}}\n");
}

pub fn push_get_victory_id<T: Write>(writer: &mut T, victory_n_map: &HashMap<Location, i64>) {
    let _ = writeln!(
        writer,
        "pub fn get_victory_id(campaign: &str, scenario: &str, code: Code, n: i64) -> Option<i64> {{\n  match (campaign, scenario, code.i64(), n) {{"
    );

    for ((campaign, scenario, code, victory), n) in victory_n_map {
        let _ = writeln!(writer, "    (\"{campaign}\", \"{scenario}\", {}, {victory}) => Some({n}),", code.i64());
    }

    let _ = writeln!(writer, "    _ => None\n  }}\n}}\n");
}

pub fn push_is_goal_location<T: Write>(writer: &mut T, goal_locations: &HashMap<i64, Ustr>) {
    let _ = writeln!(writer, "pub fn is_goal_location(first_id: i64) -> Option<&'static str> {{\n  match first_id {{");

    for (id, campaign) in goal_locations {
        let _ = writeln!(writer, "    {id} => Some(\"{campaign}\"),");
    }

    let _ = writeln!(writer, "    _ => None\n  }}\n}}\n");
}
