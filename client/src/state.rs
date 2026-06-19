use std::collections::{HashMap, HashSet};

use cardcode::Code;

use crate::{
    data::{CampaignFreeItem, Check, Item, can_follow_path, can_send_location, get_campaign, get_card, get_scenario, is_goal_location},
    protocol::SlotData,
};

pub struct State {
    unsent_locations: HashSet<i64>,
    campaigns: HashMap<&'static str, CampaignState>,
    scenarios: HashMap<&'static str, HashSet<Code>>,
    pub slot_data: SlotData,
    empty_set: HashSet<Code>,
}

#[derive(Debug, Default)]
struct CampaignState {
    unlocks: HashMap<&'static str, i64>,
    scenarios: HashSet<&'static str>,
    xp: i64,
    cards: HashMap<Code, i64>,
    filler: HashMap<&'static str, i64>,
    completed: bool,
}

impl State {
    pub fn new(missing_locations: &[i64], checked_locations: &[i64], slot_data: SlotData) -> State {
        let mut state = State {
            unsent_locations: HashSet::from_iter(missing_locations.iter().copied()),
            campaigns: HashMap::new(),
            scenarios: HashMap::new(),
            slot_data,
            empty_set: HashSet::new(),
        };

        for id in checked_locations {
            if let Some(campaign) = is_goal_location(*id) {
                state.campaign_state(campaign).completed = true;
            }
        }

        state
    }

    pub fn has(&self, campaign: &'static str, item: &'static str) -> bool {
        if let Some(state) = self.campaigns.get(campaign) {
            state.unlocks.get(item).is_some_and(|count| *count > 0)
        } else {
            false
        }
    }

    pub fn is_unsent(&self, location: i64) -> bool {
        self.unsent_locations.contains(&location)
    }

    pub fn reachable_locations(&self, scenario: &'static str) -> impl Iterator<Item = Code> {
        self.scenarios.get(scenario).unwrap_or(&self.empty_set).iter().copied()
    }

    pub fn reachable_checks(&self, scenario: &'static str) -> Option<impl Iterator<Item = Check>> {
        get_scenario(scenario).map(|scenario| scenario.checks.iter().filter(|check| can_send_location(check.ids[0], self)).copied())
    }

    pub fn campaigns(&self) -> impl Iterator<Item = (&'static str, bool)> {
        self.campaigns.iter().filter(|(_, state)| !state.scenarios.is_empty()).map(|(name, state)| (*name, state.completed))
    }

    pub fn scenarios(&mut self, campaign: &'static str) -> impl Iterator<Item = &'static str> {
        self.campaign_state(campaign).scenarios.iter().copied()
    }

    pub fn cards(&mut self, campaign: &'static str) -> impl Iterator<Item = (Code, i64)> {
        self.campaign_state(campaign).cards.iter().map(|(code, count)| (*code, *count))
    }

    pub fn filler(&mut self, campaign: &'static str) -> impl Iterator<Item = (&'static str, i64)> {
        self.campaign_state(campaign).filler.iter().map(|(code, count)| (*code, *count))
    }

    pub fn completed_campaigns(&self) -> usize {
        self.campaigns.values().filter(|state| state.completed).count()
    }

    pub fn goal_progress(&self) -> (usize, usize) {
        (self.completed_campaigns(), self.slot_data.required_campaigns)
    }

    pub fn mark_sent(&mut self, location: i64) {
        self.unsent_locations.remove(&location);
        if let Some(campaign) = is_goal_location(location) {
            self.campaign_state(campaign).completed = true;
        }
    }

    pub fn add_item(&mut self, item: Item) {
        let (campaign, item) = item.split();
        let campaign_state = self.campaign_state(campaign);

        let mut update_reachable = false;

        match item {
            CampaignFreeItem::Unlock(unlock) => {
                campaign_state.unlocks.entry(unlock).and_modify(|count| *count += 1).or_insert(1);
                update_reachable = true;
            }
            CampaignFreeItem::ScenarioUnlock(scenario) => {
                campaign_state.scenarios.insert(scenario);
                update_reachable = true;
            }
            CampaignFreeItem::Xp(xp) => {
                campaign_state.xp += xp;
            }
            CampaignFreeItem::ScenarioCard(code) => {
                campaign_state.cards.entry(code).and_modify(|count| *count += 1).or_insert(1);

                if let Some(card) = get_card(code) {
                    campaign_state.unlocks.entry(card.name).and_modify(|count| *count += 1).or_insert(1);
                }

                update_reachable = true;
            }
            CampaignFreeItem::CampaignFiller(filler) => {
                campaign_state.filler.entry(filler).and_modify(|count| *count += 1).or_insert(1);
            }
        }

        if update_reachable && let Some(campaign) = get_campaign(campaign) {
            for scenario in campaign.scenarios {
                if let Some(scenario) = get_scenario(scenario) {
                    let mut reachable = self.scenarios.remove(scenario.name).unwrap_or({
                        let mut reachable = HashSet::new();
                        for location in scenario.begin {
                            reachable.insert(*location);
                        }
                        reachable
                    });

                    let mut repeat = true;
                    while repeat {
                        repeat = false;
                        for path in scenario.paths {
                            if reachable.contains(&path.from) && !reachable.contains(&path.to) && can_follow_path(campaign.name, scenario.name, path.from, path.to, self) {
                                reachable.insert(path.to);
                                repeat = true;
                            }
                        }
                    }

                    self.scenarios.insert(scenario.name, reachable);
                }
            }
        }
    }

    pub fn complete_campaign(&mut self, campaign: &'static str) {
        self.campaigns.entry(campaign).or_default().completed = true;
    }

    fn campaign_state(&mut self, campaign: &'static str) -> &mut CampaignState {
        self.campaigns.entry(campaign).or_default()
    }
}
