use ustr::Ustr;

use crate::{Data, LogicTerm, card::Cards};
use std::{fs::OpenOptions, io::Write};

const PREFIX: &str = include_str!("prefix.py");

#[derive(Debug)]
struct PyCampaign<'a> {
    name: Ustr,
    xp: i64,
    scenarios: &'a [Ustr],
    unlocks: &'a [Ustr],
    scenario_cards: Vec<String>,
    filler: Vec<PyFiller>,
}

#[derive(Debug)]
struct PyScenario<'a> {
    name: Ustr,
    campaign: Ustr,
    logic_xp: i64,
    locations: Vec<PyLocation>,
    begin: Vec<String>,
    paths: Vec<PyPath<'a>>,
    checks: Vec<PyCheck<'a>>,
}

#[derive(Debug)]
struct PyLocation {
    name: String,
    clues: i64,
    victory: i64,
}

#[derive(Debug)]
struct PyPath<'a> {
    origin: String,
    destination: String,
    rule: &'a LogicTerm,
}

#[derive(Debug)]
struct PyCheck<'a> {
    name: String,
    goal: bool,
    rule: &'a LogicTerm,
}

#[derive(Debug)]
struct PyFiller {
    name: Ustr,
    // bitfield: 0000[expert][hard][standard][easy]
    trap: u8,
    quantity: [i64; 4],
}

pub fn generate_data_py(cards: &Cards, data: &Data) {
    if let Ok(mut writer) = OpenOptions::new().write(true).create(true).truncate(true).open("Data.py") {
        let mut campaigns = vec![];
        let mut scenarios = vec![];

        for campaign in data.campaigns.values() {
            campaigns.push(PyCampaign {
                name: campaign.name,
                xp: campaign.xp,
                scenarios: &campaign.scenarios_order,
                unlocks: &campaign.unlocks,
                scenario_cards: campaign
                    .scenario_cards
                    .iter()
                    .map(|code| cards.get(code).expect("Failed to find card for code").unique_name())
                    .collect(),
                filler: campaign
                    .filler
                    .iter()
                    .map(|filler| PyFiller {
                        name: filler.name,
                        trap: filler.trap,
                        quantity: filler.quantity,
                    })
                    .collect(),
            });

            for scenario in campaign.scenarios.values() {
                scenarios.push(PyScenario {
                    name: scenario.name,
                    campaign: campaign.name,
                    logic_xp: scenario.logic_xp,
                    locations: scenario
                        .locations
                        .iter()
                        .map(|code| {
                            let card = cards.get(code).expect("Failed to find card for code");
                            PyLocation {
                                name: card.unique_name(),
                                clues: card.clues,
                                victory: card.victory,
                            }
                        })
                        .collect(),
                    begin: scenario.begin.iter().map(|code| cards.get(code).expect("Failed to find card for code").unique_name()).collect(),
                    paths: scenario
                        .paths
                        .iter()
                        .map(|path| {
                            let from_card = cards.get(&path.from).expect("Failed to find card for code");
                            let to_card = cards.get(&path.to).expect("Failed to find card for code");
                            PyPath {
                                origin: from_card.unique_name(),
                                destination: to_card.unique_name(),
                                rule: &path.logic,
                            }
                        })
                        .collect(),
                    checks: scenario
                        .checks
                        .iter()
                        .flat_map(|check| {
                            let mut names = vec![];

                            let card = cards.get(&check.code).expect("Failed to find card from code");

                            if card.victory > 0 {
                                for victory in 0..card.victory {
                                    if check.count > 1 {
                                        for n in 0..check.count {
                                            names.push(format!("{} - {} Victory {} #{}", scenario.name, check.name, victory + 1, n + 1));
                                        }
                                    } else {
                                        names.push(format!("{} - {} Victory {}", scenario.name, check.name, victory + 1));
                                    }
                                }
                            } else {
                                if check.count > 1 {
                                    for n in 0..check.count {
                                        names.push(format!("{} - {} #{}", scenario.name, check.name, n + 1));
                                    }
                                } else {
                                    names.push(format!("{} - {}", scenario.name, check.name));
                                }
                            }

                            names.into_iter().map(|name| PyCheck {
                                name,
                                goal: check.goal,
                                rule: &check.logic,
                            })
                        })
                        .collect(),
                });
            }
        }

        let _ = write!(writer, "{PREFIX}campaigns={{");

        for campaign in campaigns {
            let _ = write!(
                writer,
                "\"{}\":Campaign(\"{}\",{},[\"{}\"],[\"{}\"],[\"{}\"],[{}]),",
                campaign.name,
                campaign.name,
                campaign.xp,
                campaign.scenarios.iter().map(Ustr::as_str).collect::<Vec<_>>().join("\",\""),
                campaign.unlocks.iter().map(Ustr::as_str).collect::<Vec<_>>().join("\",\""),
                campaign.scenario_cards.join("\",\""),
                campaign
                    .filler
                    .iter()
                    .map(|filler| format!("Filler(\"{}\",{},[{}])", filler.name, filler.trap, filler.quantity.map(|q| q.to_string()).join(",")))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }

        let _ = write!(writer, "}}\nscenarios={{");

        for scenario in scenarios {
            let _ = write!(
                writer,
                "\"{}\":Scenario(\"{}\",\"{}\",{},[{}],[\"{}\"],[{}],[{}]),",
                scenario.name,
                scenario.name,
                scenario.campaign,
                scenario.logic_xp,
                scenario
                    .locations
                    .iter()
                    .map(|location| format!("Location(\"{}\",{},{})", location.name, location.clues, location.victory))
                    .collect::<Vec<_>>()
                    .join(","),
                scenario.begin.join("\",\""),
                scenario
                    .paths
                    .iter()
                    .map(|path| format!("Path(\"{}\",\"{}\",lambda state,player:{})", path.origin, path.destination, path.rule.as_py_expr()))
                    .collect::<Vec<_>>()
                    .join(","),
                scenario
                    .checks
                    .iter()
                    .map(|check| format!("Check(\"{}\",{},lambda state,player:{})", check.name, check.goal as u8, check.rule.as_py_expr()))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }

        let _ = writeln!(writer, "}}");
    }
}
