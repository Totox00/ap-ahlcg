use std::fmt::Write;

use crate::Data;

pub fn push_get_campaign<T: Write>(writer: &mut T, data: &Data) {
    let _ = writeln!(writer, "pub fn get_campaign(campaign: &str) -> Option<Campaign> {{\n  match campaign {{");

    let mut arrays = vec![];

    for campaign in data.campaigns.values() {
        arrays.push(format!(
            "const CAMPAIGN_{}_SCENARIOS: [&str; {}] = [{}];",
            campaign.i,
            campaign.scenarios_order.len(),
            campaign.scenarios_order.iter().map(|scenario| format!("\"{scenario}\"")).collect::<Vec<_>>().join(",")
        ));

        let _ = writeln!(
            writer,
            "    \"{}\" => Some(Campaign {{name: \"{}\", scenarios: &CAMPAIGN_{}_SCENARIOS}}),",
            campaign.name, campaign.name, campaign.i
        );
    }

    let _ = writeln!(writer, "    _ => None\n  }}\n}}\n{}", arrays.join("\n"));
}

pub fn push_get_scenario<T: Write>(writer: &mut T, data: &Data) {
    let _ = writeln!(writer, "pub fn get_scenario(scenario: &str) -> Option<Scenario> {{\n  match scenario {{");

    let mut arrays = vec![];

    for campaign in data.campaigns.values() {
        for scenario in campaign.scenarios.values() {
            arrays.push(format!(
                "const CAMPAIGN_{}_SCENARIO_{}_LOCKS: [Lock; {}] = [{}];",
                campaign.i,
                scenario.i,
                scenario.locks.len(),
                scenario
                    .locks
                    .iter()
                    .map(|lock| format!("Lock {{name: \"{}\", rule: \"{}\", neg_rule: \"{}\"}}", lock.name, lock.rule, lock.neg_rule))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
            arrays.push(format!(
                "const CAMPAIGN_{}_SCENARIO_{}_LOCATIONS: [Code; {}] = [{}];",
                campaign.i,
                scenario.i,
                scenario.locations.len(),
                scenario.locations.iter().map(|code| format!("Code::from({})", code.i64())).collect::<Vec<_>>().join(","),
            ));
            arrays.push(format!(
                "const CAMPAIGN_{}_SCENARIO_{}_BEGIN: [Code; {}] = [{}];",
                campaign.i,
                scenario.i,
                scenario.begin.len(),
                scenario.begin.iter().map(|code| format!("Code::from({})", code.i64())).collect::<Vec<_>>().join(","),
            ));
            arrays.push(format!(
                "const CAMPAIGN_{}_SCENARIO_{}_PATHS: [Path; {}] = [{}];",
                campaign.i,
                scenario.i,
                scenario.paths.len(),
                scenario
                    .paths
                    .iter()
                    .map(|path| format!("Path {{from: Code::from({}), to: Code::from({})}}", path.from.i64(), path.to.i64()))
                    .collect::<Vec<_>>()
                    .join(","),
            ));
            arrays.push(format!(
                "const CAMPAIGN_{}_SCENARIO_{}_CHECKS: [Check; {}] = [{}];",
                campaign.i,
                scenario.i,
                scenario.checks.len(),
                scenario
                    .checks
                    .iter()
                    .map(|check| format!(
                        "Check {{code: Code::from({}), description: \"{}\", count: {}, victory: {}, ids: &[{}]}}",
                        check.code.i64(),
                        if check.description.is_empty() { &check.name } else { &check.description },
                        check.count,
                        check.victory,
                        check.ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","),
                    ))
                    .collect::<Vec<_>>()
                    .join(","),
            ));

            let _ = writeln!(
                writer,
                "    \"{}\" => Some(Scenario {{name: \"{}\", always: \"{}\", locks: &CAMPAIGN_{}_SCENARIO_{}_LOCKS, locations: &CAMPAIGN_{}_SCENARIO_{}_LOCATIONS, begin: &CAMPAIGN_{}_SCENARIO_{}_BEGIN, paths: &CAMPAIGN_{}_SCENARIO_{}_PATHS, checks: &CAMPAIGN_{}_SCENARIO_{}_CHECKS}}),",
                scenario.name, scenario.name, scenario.always, campaign.i, scenario.i, campaign.i, scenario.i, campaign.i, scenario.i, campaign.i, scenario.i, campaign.i, scenario.i,
            );
        }
    }

    let _ = writeln!(writer, "    _ => None\n  }}\n}}\n{}", arrays.join("\n"));
}
