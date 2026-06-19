mod data;
mod datapackage;
mod format_json;
mod interface;
mod protocol;
mod state;

use data::{get_campaign, get_scenario, item_from_id};
use datapackage::DatapackageStore;
use format_json::format;
use interface::Interface;
use protocol::Connected;
use serde_json::from_str;
use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::state::State;

#[wasm_bindgen]
pub struct Session {
    datapackage_store: DatapackageStore,
    players: HashMap<i32, String>,
    slot: String,
    state: State,
    interface: Interface,
}

#[wasm_bindgen]
pub struct Action {
    locations: Vec<i64>,
    pub victory: bool,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn new_session(mut datapackage_store: DatapackageStore, connected: &str, slot: &str) -> Session {
    let connected: Connected = from_str(connected).unwrap_or_else(|err| {
        log!("Failed to parse Connected packet: {err}");
        panic!()
    });

    datapackage_store.build_player_map(&connected);

    let state = State::new(
        &connected.missing_locations,
        &connected.checked_locations,
        connected.slot_data.try_into().unwrap_or_else(|_| {
            log!("Invalid slot data {:?}", connected.slot_data);
            panic!()
        }),
    );

    let mut players = HashMap::new();
    for player in connected.players {
        players.insert(player.slot, player.alias);
    }

    let interface = Interface::new();
    interface.update_goal(&state);

    Session {
        datapackage_store,
        players,
        slot: slot.to_string(),
        state,
        interface,
    }
}

#[wasm_bindgen]
impl Session {
    pub fn try_format_json(&self, json: &str) -> String {
        if let Ok(msg) = from_str(json) {
            format(&self.datapackage_store, msg, &self.players, &self.slot)
        } else {
            String::new()
        }
    }

    pub fn handle_click(&mut self, target: &str) -> Action {
        if target == "goal" {
            let (completed, required) = self.state.goal_progress();
            if completed >= required {
                return Action { locations: vec![], victory: true };
            }
        } else if target.starts_with("campaign-") {
            if let Some(campaign) = get_campaign(target.split_at(9).1) {
                self.interface.select_campaign(&mut self.state, campaign.name);
            }
        } else if target.starts_with("scenario-") {
            if let Some(scenario) = get_scenario(target.split_at(9).1) {
                self.interface.select_scenario(&mut self.state, scenario.name);
            }
        } else if target.starts_with("goal-") {
            if let Ok(id) = target.split_at(5).1.parse::<i64>()
                && self.state.is_unsent(id)
            {
                self.state.mark_sent(id);
                self.state.complete_campaign(self.interface.selected_campaign);
                self.interface.update_campaigns(&self.state);
                self.interface.update_goal(&self.state);
                self.interface.update_active_scenario(&self.state);
                return Action { locations: vec![id], victory: false };
            }
        } else if let Ok(id) = target.parse::<i64>()
            && self.state.is_unsent(id)
        {
            self.state.mark_sent(id);
            self.interface.update_active_scenario(&self.state);
            return Action { locations: vec![id], victory: false };
        }
        Action::none()
    }

    pub fn recieved_items(&mut self, items: Vec<i64>) {
        for item_id in items {
            if let Some(item) = item_from_id(item_id) {
                self.state.add_item(item);
            }
        }
        self.interface.update_campaigns(&self.state);
        self.interface.update_scenarios(&mut self.state);
        self.interface.update_active_scenario(&self.state);
        self.interface.update_current_rules(&mut self.state);
    }
}

impl Action {
    fn none() -> Action {
        Action { locations: vec![], victory: false }
    }
}

#[wasm_bindgen]
impl Action {
    pub fn locations(&self) -> Vec<i64> {
        self.locations.clone()
    }
}
