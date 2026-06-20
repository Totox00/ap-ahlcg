use std::{
    collections::HashMap,
    fs::{File, read_dir},
    io::{BufRead, BufReader},
    iter::once,
};

use cardcode::Code;
use ustr::{Ustr, ustr};

use crate::{Campaign, Check, Data, Filler, Lock, LogicTerm, Path, Scenario, card::Cards, logic::parse_logic};

#[derive(Default)]
struct Fields {
    data_type: DataType,
    uname: Option<Ustr>,
    cards: Vec<Code>,
    unlocks: Vec<Ustr>,
    filler: Vec<Filler>,
    parent: Option<Ustr>,
    xp: i64,
    level: i64,
    always: String,
    rule: String,
    neg_rule: String,
    locations: Vec<Code>,
    begin: Vec<Code>,
    paths: Vec<Path>,
    checks: Vec<Check>,
}

#[derive(Default)]
enum DataType {
    #[default]
    None,
    Campaign,
    Scenario,
    Lock,
}

pub fn group_data(cards: &Cards) -> Data {
    let mut campaigns = HashMap::new();
    let mut campaigns_order = vec![];

    let mut campaign_lookup = HashMap::new();

    let mut current = Fields::default();

    let mut files: Vec<_> = read_dir(std::path::Path::new(file!()).parent().unwrap().join("data"))
        .expect("Failed to read data directory")
        .flatten()
        .map(|entry| (entry.path(), ustr(entry.file_name().to_str().expect("Invalid data file name"))))
        .collect();

    files.sort_unstable_by_key(|(_, name)| *name);

    let mut lines = files.into_iter().flat_map(|(path, name)| {
        BufReader::new(File::open(path).expect("Failed to open file"))
            .lines()
            .zip(1..)
            .map(move |(line, i)| (line.unwrap_or_else(|err| panic!("Failed to read line {i}: {err:?}")), i, name))
            .chain(once((String::new(), 0, name)))
    });

    while let Some((line, i, file)) = lines.next() {
        if let Some((field, value)) = line.split_once(' ') {
            match field {
                "campaign" => {
                    current.data_type = DataType::Campaign;
                    current.uname = Some(ustr(&value.escape_debug().to_string()));
                }
                "scenario" => {
                    current.data_type = DataType::Scenario;
                    current.uname = Some(ustr(&value.escape_debug().to_string()));
                }
                "lock" => {
                    current.data_type = DataType::Lock;
                    current.uname = Some(ustr(&value.escape_debug().to_string()));
                }
                "xp" => current.xp = value.parse().unwrap_or_else(|err| panic!("Failed to parse xp on line {i} in file {file:?}: {err:?}")),
                "in" => current.parent = Some(ustr(&value.escape_debug().to_string())),
                "level" => current.level = value.parse().unwrap_or_else(|err| panic!("Failed to parse expected level on line {i} in file {file:?}: {err:?}")),
                "always" => current.always = value.escape_debug().to_string(),
                "rule" => current.rule = value.escape_debug().to_string(),
                "neg_rule" => current.neg_rule = value.escape_debug().to_string(),
                "locations" => current.locations = value.split(' ').map(Code::from_str).collect(),
                "begin" => current.begin = value.split(' ').map(Code::from_str).collect(),
                "path" => {
                    let (from, to, logic) = if let Some((codes, logic)) = value.split_once(" requires ") {
                        let mut iter = codes.split(' ');
                        (
                            iter.next().map(Code::from_str),
                            iter.map(Code::from_str).collect::<Vec<_>>(),
                            parse_logic(&current.parent.unwrap_or_else(|| panic!("Cannot add logic outside of a campaign on line {i} in file {file:?}")), logic),
                        )
                    } else {
                        let mut iter = value.split(' ');
                        (iter.next().map(Code::from_str), iter.map(Code::from_str).collect(), LogicTerm::True)
                    };

                    for destination in to {
                        current.paths.push(Path {
                            from: from.unwrap_or_else(|| panic!("Path on line {i} in file {file:?} is missing origin")),
                            to: destination,
                            logic: logic.clone(),
                        });
                    }
                }
                "enemy" => {
                    let (value, logic) = if let Some((code, logic)) = value.split_once(" requires ") {
                        (
                            code,
                            parse_logic(&current.parent.unwrap_or_else(|| panic!("Cannot add logic outside of a campaign on line {i} in file {file:?}")), logic),
                        )
                    } else {
                        (value, LogicTerm::True)
                    };

                    let (code, count) = if let Some((code, count)) = value.split_once(' ') {
                        (
                            Code::from_str(code),
                            count.parse().unwrap_or_else(|err| panic!("Failed to parse count {count} on line {i} in file {file:?}: {err:?}")),
                        )
                    } else {
                        (Code::from_str(value), 1)
                    };

                    let card = cards.get(&code).unwrap_or_else(|| panic!("Failed to find card on line {i} in file {file:?}"));

                    current.checks.push(Check {
                        code,
                        name: card.unique_name(),
                        description: String::from("Defeated"),
                        logic,
                        count,
                        victory: card.victory,
                        goal: false,
                        ids: vec![],
                    });
                }
                "special" => {
                    let (code, logic) = if let Some((code, logic)) = value.split_once(" requires ") {
                        (
                            Code::from_str(code),
                            parse_logic(&current.parent.unwrap_or_else(|| panic!("Cannot add logic outside of a campaign on line {i} in file {file:?}")), logic),
                        )
                    } else {
                        (Code::from_str(value), LogicTerm::True)
                    };

                    current.checks.push(Check {
                        code,
                        name: cards.get(&code).unwrap_or_else(|| panic!("Failed to find card on line {i} in file {file:?}")).unique_name(),
                        description: String::new(),
                        logic,
                        count: 1,
                        victory: 0,
                        goal: false,
                        ids: vec![],
                    });
                }
                "description" => {
                    current
                        .checks
                        .last_mut()
                        .unwrap_or_else(|| panic!("Nothing to add a description to on line {i} in file {file:?}"))
                        .description = value.escape_debug().to_string()
                }
                "name" => current.checks.last_mut().unwrap_or_else(|| panic!("Nothing to add a name to on line {i} in file {file:?}")).name = value.escape_debug().to_string(),
                "cards" => current.cards = value.split(' ').map(Code::from_str).collect(),
                "unlock" => {
                    current.unlocks.push(ustr(&value.escape_debug().to_string()));
                }
                "filler" => {
                    let (easy, rest) = value.split_once(' ').unwrap_or_else(|| panic!("Filler is missing standard quantity on line {i} in file {file:?}"));
                    let (standard, rest) = rest.split_once(' ').unwrap_or_else(|| panic!("Filler is missing hard quantity on line {i} in file {file:?}"));
                    let (hard, rest) = rest.split_once(' ').unwrap_or_else(|| panic!("Filler is missing expert quantity on line {i} in file {file:?}"));
                    let (expert, name) = rest.split_once(' ').unwrap_or_else(|| panic!("Filler is missing name on line {i} in file {file:?}"));

                    let mut trap = 0;
                    let mut quantity = [0; 4];

                    for (idx, str) in [easy, standard, hard, expert].iter().enumerate() {
                        if str.starts_with('-') {
                            trap |= 1 << idx
                        };

                        quantity[idx] = str
                            .trim_start_matches('-')
                            .parse()
                            .unwrap_or_else(|err| panic!("Failed to parse quantity for difficulty {idx} on line {i} in file {file:?}: {err:?}"))
                    }

                    current.filler.push(Filler { name: ustr(name), trap, quantity })
                }
                _ => panic!("Unrecognised field {field} at line {i} in file {file:?}"),
            }
        } else {
            match line.as_str() {
                "always" => {
                    let mut always = vec![];
                    for (line, _, _) in lines.by_ref() {
                        if line.is_empty() {
                            break;
                        }
                        always.push(line.escape_debug().to_string())
                    }
                    current.always = always.join("\\n");
                }
                "rule" => {
                    let mut rule = vec![];
                    for (line, _, _) in lines.by_ref() {
                        if line.is_empty() {
                            break;
                        }
                        rule.push(line.escape_debug().to_string())
                    }
                    current.rule = rule.join("\\n");
                }
                "neg_rule" => {
                    let mut neg_rule = vec![];
                    for (line, _, _) in lines.by_ref() {
                        if line.is_empty() {
                            break;
                        }
                        neg_rule.push(line.escape_debug().to_string())
                    }
                    current.neg_rule = neg_rule.join("\\n");
                }
                "goal" => {
                    current.checks.last_mut().unwrap_or_else(|| panic!("Nothing to make a goal on line {i} in file {file:?}")).goal = true;
                }
                "" => {
                    match current.data_type {
                        DataType::None => (),
                        DataType::Campaign => {
                            let uname = current.uname.expect("Campaigns should always have unames");
                            campaigns.insert(
                                uname,
                                Campaign {
                                    name: uname,
                                    i: campaigns.len() as i64,
                                    xp: current.xp,
                                    scenarios: HashMap::new(),
                                    scenarios_order: vec![],
                                    unlocks: current.unlocks,
                                    scenario_cards: current.cards,
                                    filler: current.filler,
                                },
                            );
                            campaigns_order.push(uname);
                        }
                        DataType::Scenario => {
                            let uname = current.uname.expect("Scenarios should always have unames");
                            let parent = current.parent.unwrap_or_else(|| panic!("Scenario {uname} in file {file:?} is missing parent"));
                            campaign_lookup.insert(uname, parent);
                            let parent = campaigns
                                .get_mut(&parent)
                                .unwrap_or_else(|| panic!("Scenario {parent} for unlock {uname} in file {file:?} does not exist"));
                            parent.scenarios.insert(
                                uname,
                                Scenario {
                                    name: uname,
                                    i: parent.scenarios.len() as i64,
                                    logic_xp: current.level,
                                    always: current.always,
                                    locks: Vec::new(),
                                    locations: current.locations,
                                    begin: current.begin,
                                    paths: current.paths,
                                    checks: current.checks,
                                },
                            );
                            parent.scenarios_order.push(uname);
                        }
                        DataType::Lock => {
                            let uname = current.uname.expect("Unlocks should always have unames");
                            let parent = current.parent.unwrap_or_else(|| panic!("Unlock {uname} in file {file:?} is missing parent"));
                            let parent = campaigns
                                .get_mut(
                                    campaign_lookup
                                        .get(&parent)
                                        .unwrap_or_else(|| panic!("Scenario {parent} for unlock {uname} in file {file:?} does not exist")),
                                )
                                .expect("Failed to find campaign from lookup")
                                .scenarios
                                .get_mut(&parent)
                                .expect("Failed to find scenario in campaign from lookup");
                            parent.locks.push(Lock {
                                name: uname,
                                rule: current.rule,
                                neg_rule: current.neg_rule,
                            });
                        }
                    }
                    current = Fields::default();
                }
                _ => {
                    panic!("Field {line} missing value at line {i} in file {file:?}")
                }
            }
        }
    }

    Data { campaigns, campaigns_order }
}
