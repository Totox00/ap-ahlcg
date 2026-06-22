use std::fmt::Write;

use ustr::{Ustr, ustr};

use crate::Data;

#[derive(Debug, Clone)]
pub enum LogicTerm {
    Unlock((Ustr, Ustr)),
    Or(Vec<LogicTerm>),
    And(Vec<LogicTerm>),
    True,
}

pub fn parse_logic(campaign: &str, str: &str) -> LogicTerm {
    if str.is_empty() {
        LogicTerm::True
    } else if str.contains('|') {
        LogicTerm::Or(str.split('|').map(|str| parse_logic(campaign, str)).collect())
    } else if str.contains('&') {
        LogicTerm::And(str.split('&').map(|str| parse_logic(campaign, str)).collect())
    } else {
        LogicTerm::Unlock((ustr(campaign), ustr(&str.trim().escape_debug().to_string())))
    }
}

pub fn push_can_follow_path<T: Write>(writer: &mut T, data: &Data) {
    let _ = writeln!(
        writer,
        "pub fn can_follow_path(campaign: &str, scenario: &str, from: Code, to: Code, state: &State) -> bool {{\n  match (campaign, scenario, from.i64(), to.i64()) {{"
    );

    for campaign in data.campaigns.values() {
        for scenario in campaign.scenarios.values() {
            for path in &scenario.paths {
                let _ = writeln!(
                    writer,
                    "    (\"{}\",\"{}\",{},{}) => {},",
                    campaign.name,
                    scenario.name,
                    path.from.i64(),
                    path.to.i64(),
                    path.logic.as_rust_expr()
                );
            }
        }
    }

    let _ = writeln!(writer, "    _ => false\n  }}\n}}\n");
}

pub fn push_can_send_location<T: Write>(writer: &mut T, data: &Data) {
    let _ = writeln!(writer, "pub fn can_send_location(first_id: i64, state: &State) -> bool {{\n  match first_id {{");

    for campaign in data.campaigns.values() {
        for scenario in campaign.scenarios.values() {
            for check in &scenario.checks {
                let _ = writeln!(writer, "    {} => {},", check.ids[0], check.logic.as_rust_expr());
            }
        }
    }

    let _ = writeln!(writer, "    _ => false\n  }}\n}}\n");
}

impl LogicTerm {
    pub fn as_rust_expr(&self) -> String {
        match self {
            LogicTerm::Unlock((campaign, unlock)) => format!("state.has(\"{campaign}\", \"{unlock}\")"),
            LogicTerm::Or(terms) => terms.iter().map(|term| term.as_rust_expr()).collect::<Vec<_>>().join("||"),
            LogicTerm::And(terms) => terms.iter().map(|term| format!("({})", term.as_rust_expr())).collect::<Vec<_>>().join("&&"),
            LogicTerm::True => String::from("true"),
        }
    }

    pub fn as_py_expr(&self) -> String {
        match self {
            LogicTerm::Unlock((campaign, unlock)) => format!("state.prog_items[player].get(\"{campaign} - {unlock}\",False)"),
            LogicTerm::Or(terms) => {
                let mut required_unlocks = vec![];
                let mut additional_exprs = vec![];

                for term in terms {
                    if let LogicTerm::Unlock(unlock) = term {
                        required_unlocks.push(unlock);
                    } else {
                        additional_exprs.push(format!("({})", term.as_py_expr()));
                    }
                }

                let items_part = match required_unlocks.len() {
                    0 => String::new(),
                    1 => format!("state.prog_items[player].get(\"{} - {}\",False)", required_unlocks[0].0, required_unlocks[0].1),
                    2.. => format!(
                        "state.has_any(({}),player)",
                        required_unlocks.into_iter().map(|(campaign, item)| format!("\"{campaign} - {item}\"")).collect::<Vec<_>>().join(",")
                    ),
                };

                let additional_part = if additional_exprs.is_empty() { String::new() } else { additional_exprs.join(" or ") };

                [items_part, additional_part].into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join(" or ")
            }
            LogicTerm::And(terms) => {
                let mut required_unlocks = vec![];
                let mut additional_exprs = vec![];

                for term in terms {
                    if let LogicTerm::Unlock(unlock) = term {
                        required_unlocks.push(unlock);
                    } else {
                        additional_exprs.push(format!("({})", term.as_py_expr()));
                    }
                }

                let items_part = match required_unlocks.len() {
                    0 => String::new(),
                    1 => format!("state.prog_items[player].get(\"{} - {}\",False)", required_unlocks[0].0, required_unlocks[0].1),
                    2.. => format!(
                        "state.has_all(({}),player)",
                        required_unlocks.into_iter().map(|(campaign, item)| format!("\"{campaign} - {item}\"")).collect::<Vec<_>>().join(",")
                    ),
                };
                let additional_part = if additional_exprs.is_empty() { String::new() } else { additional_exprs.join(" and ") };

                [items_part, additional_part].into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join(" and ")
            }
            LogicTerm::True => String::from("True"),
        }
    }
}
