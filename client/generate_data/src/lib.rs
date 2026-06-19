mod card;
mod data_py;
mod data_rust;
mod group_data;
mod id_mappings;
mod id_py;
mod logic;

use std::collections::HashMap;

use cardcode::Code;
use data_py::generate_data_py;
use group_data::group_data;
use id_py::generate_id_py;
use proc_macro::TokenStream;
use ustr::Ustr;

use crate::{
    card::{get_cards, push_get_card},
    data_rust::{push_get_campaign, push_get_scenario},
    id_mappings::{push_get_clue_id, push_get_victory_id, push_is_goal_location, push_item_from_id},
    logic::{LogicTerm, push_can_follow_path, push_can_send_location},
};

#[derive(Debug)]
struct Data {
    pub campaigns: HashMap<Ustr, Campaign>,
    pub campaigns_order: Vec<Ustr>,
}

#[derive(Debug)]
struct Campaign {
    pub name: Ustr,
    pub i: i64,
    pub xp: i64,
    pub scenarios: HashMap<Ustr, Scenario>,
    pub scenarios_order: Vec<Ustr>,
    pub unlocks: Vec<Ustr>,
    pub scenario_cards: Vec<Code>,
    pub filler: Vec<Filler>,
}

#[derive(Debug)]
struct Scenario {
    pub name: Ustr,
    pub i: i64,
    pub logic_xp: i64,
    pub always: String,
    pub locks: Vec<Lock>,
    pub locations: Vec<Code>,
    pub begin: Vec<Code>,
    pub paths: Vec<Path>,
    pub checks: Vec<Check>,
}

#[derive(Debug)]
struct Lock {
    pub name: Ustr,
    pub rule: String,
    pub neg_rule: String,
}

#[derive(Debug)]
struct Path {
    pub from: Code,
    pub to: Code,
    pub logic: LogicTerm,
}

#[derive(Debug)]
struct Check {
    pub code: Code,
    pub name: String,
    pub description: String,
    pub logic: LogicTerm,
    pub count: i64,
    pub victory: i64,
    pub goal: bool,
    pub ids: Vec<i64>,
}

#[derive(Debug)]
struct Filler {
    pub name: Ustr,
    pub trap: u8,
    pub quantity: [i64; 4],
}

/// # Panics
///
/// Panics if the input files cannot be parsed into valid data
#[proc_macro]
pub fn generate_data(_stream: TokenStream) -> TokenStream {
    let cards = get_cards();
    let mut data = group_data(&cards);

    let datapackage = generate_id_py(&cards, &mut data);
    generate_data_py(&cards, &data);

    let mut str = String::new();

    push_item_from_id(&mut str, &datapackage.item_from_id);
    push_get_clue_id(&mut str, &datapackage.clue_ids);
    push_get_victory_id(&mut str, &datapackage.victory_ids);
    push_get_card(&mut str, &cards);
    push_get_campaign(&mut str, &data);
    push_get_scenario(&mut str, &data);
    push_can_follow_path(&mut str, &data);
    push_can_send_location(&mut str, &data);
    push_is_goal_location(&mut str, &datapackage.goal_locations);

    str.parse().unwrap()
}
