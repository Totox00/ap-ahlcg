use crate::state::State;
use cardcode::Code;

cfg_select! {
    feature = "rust_analyzer" => {
        pub fn item_from_id(_: i64) -> Option<Item> {unreachable!()}
        pub fn get_card(_: Code) -> Option<Card> {unreachable!()}
        pub fn get_campaign(_: &str) -> Option<Campaign> {unreachable!()}
        pub fn get_scenario(_: &str) -> Option<Scenario> {unreachable!()}
        pub fn get_clue_id(_: &str, _: &str, _: Code, _: i64) -> Option<i64> {unreachable!()}
        pub fn get_victory_id(_: &str, _: &str, _: Code, _: i64) -> Option<i64> {unreachable!()}
        pub fn can_follow_path(_: &str, _: &str, _: Code, _: Code, _: &State) -> bool {unreachable!()}
        pub fn can_send_location(_: i64, _: &State) -> bool {unreachable!()}
        pub fn is_goal_location(_: i64) -> Option<&'static str> {unreachable!()}
    }
    _ => {
        generate_data::generate_data!();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Item {
    Unlock((&'static str, &'static str)),
    ScenarioUnlock((&'static str, &'static str)),
    Xp((&'static str, i64)),
    ScenarioCard((&'static str, Code)),
    CampaignFiller((&'static str, &'static str)),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum CampaignFreeItem {
    Unlock(&'static str),
    ScenarioUnlock(&'static str),
    Xp(i64),
    ScenarioCard(Code),
    CampaignFiller(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub code: Code,
    pub name: &'static str,
    pub image: Option<&'static str>,
    pub clues: i64,
    pub victory: i64,
    pub unique: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Campaign {
    pub name: &'static str,
    pub scenarios: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scenario {
    pub name: &'static str,
    pub always: &'static str,
    pub locks: &'static [Lock],
    pub locations: &'static [Code],
    pub begin: &'static [Code],
    pub paths: &'static [Path],
    pub checks: &'static [Check],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lock {
    pub name: &'static str,
    pub rule: &'static str,
    pub neg_rule: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Path {
    pub from: Code,
    pub to: Code,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Check {
    pub code: Code,
    pub description: &'static str,
    pub count: i64,
    pub victory: i64,
    pub ids: &'static [i64],
}

impl Item {
    pub fn split(self) -> (&'static str, CampaignFreeItem) {
        match self {
            Item::Unlock((campaign, v)) => (campaign, CampaignFreeItem::Unlock(v)),
            Item::ScenarioUnlock((campaign, v)) => (campaign, CampaignFreeItem::ScenarioUnlock(v)),
            Item::Xp((campaign, v)) => (campaign, CampaignFreeItem::Xp(v)),
            Item::ScenarioCard((campaign, v)) => (campaign, CampaignFreeItem::ScenarioCard(v)),
            Item::CampaignFiller((campaign, v)) => (campaign, CampaignFreeItem::CampaignFiller(v)),
        }
    }
}
