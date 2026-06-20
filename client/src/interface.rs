use std::{collections::HashMap, fmt::Write};

use crate::{
    data::{get_card, get_clue_id, get_scenario, get_victory_id},
    state::State,
};
use cardcode::Code;
use web_sys::{Element, window};

pub struct Interface {
    campaigns: Element,
    scenarios: Element,
    active_scenario: Element,
    rules: Element,
    goal: Element,
    pub selected_campaign: &'static str,
    pub selected_scenario: &'static str,
}

impl Interface {
    pub fn new() -> Interface {
        if let Some(window) = window()
            && let Some(document) = window.document()
        {
            return Interface {
                campaigns: document.get_element_by_id("campaign-col").expect("Failed to get campaign-col element"),
                scenarios: document.get_element_by_id("scenario-col").expect("Failed to get scenario-col element"),
                active_scenario: document.get_element_by_id("active-scenario").expect("Failed to get active-scenario element"),
                rules: document.get_element_by_id("active-rules").expect("Failed to get active-rules element"),
                goal: document.get_element_by_id("goal").expect("Failed to get goal element"),
                selected_campaign: "",
                selected_scenario: "",
            };
        }

        panic!("Failed to get elements");
    }

    pub fn select_campaign(&mut self, state: &mut State, campaign: &'static str) {
        let diff = self.selected_campaign != campaign;
        self.selected_campaign = campaign;
        if diff {
            self.update_campaigns(state);
            self.update_scenarios(state);
        }
    }

    pub fn select_scenario(&mut self, state: &mut State, scenario: &'static str) {
        let diff = self.selected_scenario != scenario;
        self.selected_scenario = scenario;
        if diff {
            self.update_scenarios(state);
            self.update_active_scenario(state);
            self.update_current_rules(state);
        }
    }

    pub fn update_campaigns(&self, state: &State) {
        let mut buf = String::new();

        for (campaign, goaled) in state.campaigns() {
            let _ = write!(
                &mut buf,
                "<li id=\"campaign-{}\" class=\"campaign{}{}\">{}</li>",
                campaign,
                if goaled { " goaled" } else { "" },
                if campaign == self.selected_campaign { " selected" } else { "" },
                campaign
            );
        }

        self.campaigns.set_inner_html(&buf);
    }

    pub fn update_scenarios(&self, state: &mut State) {
        let mut buf = String::new();

        for scenario in state.scenarios(self.selected_campaign) {
            let _ = write!(
                &mut buf,
                "<li id=\"scenario-{}\" class=\"scenario{}\">{}</li>",
                scenario,
                if scenario == self.selected_scenario { " selected" } else { "" },
                scenario
            );
        }

        self.scenarios.set_inner_html(&buf);
    }

    pub fn update_active_scenario(&self, state: &State) {
        let mut checks = Checklist::default();

        for location in state.reachable_locations(self.selected_scenario) {
            if let Some(card) = get_card(location) {
                for clue in 0..card.clues {
                    if let Some(id) = get_clue_id(self.selected_campaign, self.selected_scenario, location, clue)
                        && state.is_unsent(id)
                    {
                        checks.add(location, format!("Clue {}", clue + 1), id);
                    }
                }
                for victory in 0..card.victory {
                    if let Some(id) = get_victory_id(self.selected_campaign, self.selected_scenario, location, victory)
                        && state.is_unsent(id)
                    {
                        checks.add(location, format!("Victory {}", victory + 1), id);
                    }
                }
            }
        }

        if let Some(reachable_checks) = state.reachable_checks(self.selected_scenario) {
            for check in reachable_checks {
                if check.victory > 0 {
                    for victory in 0..check.victory {
                        if check.count > 1 {
                            for n in 0..check.count {
                                let id = check.ids[(victory * check.count + n) as usize];
                                if state.is_unsent(id) {
                                    checks.add(check.code, format!("Victory {} #{}", victory + 1, n + 1), id);
                                }
                            }
                        } else {
                            let id = check.ids[victory as usize];
                            if state.is_unsent(id) {
                                checks.add(check.code, format!("Victory {}", victory + 1), id);
                            }
                        }
                    }
                } else {
                    if check.count > 1 {
                        for n in 0..check.count {
                            let id = check.ids[n as usize];
                            if state.is_unsent(id) {
                                checks.add(check.code, format!("{} #{}", check.description, n + 1), id);
                            }
                        }
                    } else {
                        let id = check.ids[0];
                        if state.is_unsent(id) {
                            checks.add(check.code, check.description.to_string(), id);
                        }
                    }
                }
            }
        }

        self.active_scenario.set_inner_html(&checks.into_html());
    }

    pub fn update_current_rules(&self, state: &mut State) {
        if let Some(scenario) = get_scenario(self.selected_scenario) {
            let mut buf = String::new();

            let _ = write!(&mut buf, "<p>Received XP: {}</p>", state.xp(self.selected_campaign));
            let _ = write!(&mut buf, "<p>{}</p>", scenario.always);

            for lock in scenario.locks {
                if state.has(self.selected_campaign, lock.name) {
                    if !lock.rule.is_empty() {
                        let _ = write!(&mut buf, "<p>{}</p>", lock.rule);
                    }
                } else if !lock.neg_rule.is_empty() {
                    let _ = write!(&mut buf, "<p>{}</p>", lock.neg_rule);
                }
            }

            let cards_str = state
                .cards(self.selected_campaign)
                .filter(|(_, count)| *count > 0)
                .filter_map(|(code, count)| get_card(code).map(|card| (card.name, count)))
                .map(|(name, count)| if count == 1 { format!("<li>{name}</li>") } else { format!("<li>{name} x{count}</li>") })
                .collect::<Vec<_>>()
                .join("");

            if !cards_str.is_empty() {
                let _ = write!(&mut buf, "<h3>Received Cards</h3><ul>{}</ul>", cards_str);
            }

            let filler_str = state
                .filler(self.selected_campaign)
                .filter(|(_, count)| *count > 0)
                .map(|(name, count)| if count == 1 { format!("<li>{name}</li>") } else { format!("<li>{name} x{count}</li>") })
                .collect::<Vec<_>>()
                .join("");

            if !filler_str.is_empty() {
                let _ = write!(&mut buf, "<h3>Received Filler</h3><ul>{}</ul>", filler_str);
            }

            self.rules.set_inner_html(
                &buf.replace("\n", "<br />")
                    .replace("<action>", "<span class=\"icon-action\" title=\"Action\"></span>")
                    .replace("<reaction>", "<span class=\"icon-reaction\" title=\"Reaction\"></span>"),
            );
        }
    }

    pub fn update_goal(&self, state: &State) {
        let (completed, required) = state.goal_progress();

        if completed >= required {
            let _ = self.goal.class_list().add_1("available");
            self.goal.set_inner_html("Goal");
        } else {
            let _ = self.goal.class_list().remove_1("available");
            self.goal.set_inner_html(&format!("{completed}/{required} Campaigns"));
        }
    }
}

#[derive(Debug, Default)]
struct Checklist {
    inner: HashMap<Code, Vec<(String, i64)>>,
}

impl Checklist {
    fn add(&mut self, code: Code, name: String, id: i64) {
        self.inner.entry(code).and_modify(|vec| vec.push((name.to_owned(), id))).or_insert(vec![(name.to_owned(), id)]);
    }

    fn into_html(self) -> String {
        let mut buf = String::new();

        let mut vec: Vec<_> = self.inner.into_iter().collect();
        vec.sort_by_key(|(code, _)| *code);

        for (code, checks) in vec {
            let img_str = if let Some(card) = get_card(code) {
                if let Some(image) = card.image {
                    format!("<img src=\"http://arkhamdb.com{image}\" alt=\"{}\"/>", card.name)
                } else {
                    card.name.to_string()
                }
            } else {
                String::from("???")
            };

            let _ = write!(
                &mut buf,
                "<div class=\"check-card\"><div class=\"card-header\">{img_str}</div><div class=\"card-checks\">{}</div></div>",
                checks.into_iter().map(|(name, id)| format!("<button id=\"{id}\">{name}</button>")).collect::<Vec<_>>().join("")
            );
        }

        buf
    }
}
