// ITEMS                     52   48       40       32       24       16        8
//                        ____0000 00000000 00000000 00000000 00000000 00000000 00000000
//                 unlock ____0000 nnnnnnnn yyyyyyyy yyyyyyyy yyyyyyyy yyyyyyyy xxxxxxxx (x: campaign index, y: unlock name hash, n: hash collision index)
//        scenario unlock ____0001 00000000 00000000 00000000 00000000 yyyyyyyy xxxxxxxx (x: campaign index, y: scenario index)
//                     xp ____0010 00000000 00000000 00000000 00000000 yyyyyyyy xxxxxxxx (x: campaign index, y: xp amount)
//          scenario card ____0011 00000000 00000000 zzzzzzzz zzzzzzzz zzzzzzzz xxxxxxxx (x: campaign index, z: card code hash)
//        campaign filler ____0100 nnnnnnnn zzzzzzzz zzzzzzzz zzzzzzzz 00000000 xxxxxxxx (x: campaign index, z: filler name hash, n: hash collision index)
//        scenario filler ____0101 nnnnnnnn zzzzzzzz zzzzzzzz zzzzzzzz yyyyyyyy xxxxxxxx (x: campaign index, y: scenario index, z: filler name hash, n: hash collision index)
//
// LOCATIONS                 52   48       40       32       24       16        8
//                        ____0000 00000000 00000000 00000000 00000000 00000000 00000000
//        scenario + card ____0000 nnnnnnnn zzzzzzzz zzzzzzzz zzzzzzzz yyyyyyyy xxxxxxxx (x: campaign index, y: scenario index, z: card code hash, n: check index)

use std::{collections::HashMap, fs::OpenOptions, io::Write};

use cardcode::Code;
use ustr::Ustr;

use crate::{Data, card::Cards};

pub struct Datapackage {
    pub item_from_id: HashMap<i64, Item>,
    pub clue_ids: HashMap<Location, i64>,
    pub victory_ids: HashMap<Location, i64>,
    pub goal_locations: HashMap<i64, Ustr>,
}

pub type Location = (Ustr, Ustr, Code, i64);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Item {
    Unlock((Ustr, Ustr)),
    ScenarioUnlock((Ustr, Ustr)),
    Xp((Ustr, i64)),
    ScenarioCard((Ustr, Code)),
    CampaignFiller((Ustr, Ustr)),
}

pub fn generate_id_py(cards: &Cards, data: &mut Data) -> Datapackage {
    let mut item_from_id = HashMap::new();
    let mut clue_ids = HashMap::new();
    let mut victory_ids = HashMap::new();
    let mut goal_locations = HashMap::new();

    let mut unlock_counts = Counter::new(40);
    let mut filler_counts = Counter::new(40);

    if let Ok(mut writer) = OpenOptions::new().write(true).create(true).truncate(true).open("Id.py") {
        let _ = write!(writer, "# This file is generated as part of the compilation of the client\nitem_name_to_id={{");

        for campaign in data
            .campaigns_order
            .iter()
            .map(|uname| data.campaigns.get(uname).expect("Entry in campaigns_order should be in campaigns"))
        {
            for unlock in &campaign.unlocks {
                let id = unlock_counts.get(campaign.i | (unlock.precomputed_hash() as i64 & 0xffff) << 8);
                assert_eq!(item_from_id.insert(id, Item::Unlock((campaign.name, *unlock))), None);
                let _ = write!(writer, "\"{} - {unlock}\":{id},", campaign.name);
            }

            for xp in 1..=10 {
                let id = 0b0010 << 48 | campaign.i | xp << 8;
                assert_eq!(item_from_id.insert(id, Item::Xp((campaign.name, xp))), None);
                let _ = write!(writer, "\"{} - {xp} XP\":{id},", campaign.name);
            }

            for card in &campaign.scenario_cards {
                let id = 0b0011 << 48 | campaign.i | (card.i64() & 0xfff) << 8;
                assert_eq!(item_from_id.insert(id, Item::ScenarioCard((campaign.name, *card))), None);
                let _ = write!(writer, "\"{} - {}\":{id},", campaign.name, cards.get(card).unwrap_or_else(|| panic!("Failed to find card for code: {card}")).unique_name());
            }

            for filler in &campaign.filler {
                let hash_val = filler.name.precomputed_hash() as i64 & 0xfff;
                let id = filler_counts.get(hash_val) | 0b0100 << 48 | campaign.i;
                assert_eq!(item_from_id.insert(id, Item::CampaignFiller((campaign.name, filler.name))), None);
                let _ = write!(writer, "\"{} - {}\":{id},", campaign.name, filler.name);
            }

            for scenario in campaign
                .scenarios_order
                .iter()
                .map(|uname| campaign.scenarios.get(uname).expect("Entry in scenarios_order should be in scenarios"))
            {
                let id = 0b0001 << 48 | campaign.i | scenario.i << 8;
                assert_eq!(item_from_id.insert(id, Item::ScenarioUnlock((campaign.name, scenario.name))), None);
                let _ = write!(writer, "\"{} - {}\":{id},", campaign.name, scenario.name);
            }
        }

        let _ = write!(writer, "}}\nlocation_name_to_id={{");

        let mut counts = Counter::new(40);

        for uname in &data.campaigns_order {
            let campaign = data.campaigns.get_mut(uname).expect("Entry in campaigns_order should be in campaigns");
            for uname in &campaign.scenarios_order {
                let scenario = campaign.scenarios.get_mut(uname).expect("Entry in scenarios_order should be in scenarios");
                for location in &scenario.locations {
                    let card = cards.get(location).unwrap_or_else(|| panic!("Failed to find card for code: {location}"));
                    for n in 0..card.victory {
                        let id = counts.get(campaign.i | scenario.i << 8 | location.i64() << 16);
                        victory_ids.insert((campaign.name, scenario.name, *location, n), id);
                        let _ = write!(writer, "\"{} - {} Victory {}\":{id},", scenario.name, card.unique_name(), n + 1);
                    }
                    for n in 0..card.clues {
                        let id = counts.get(campaign.i | scenario.i << 8 | location.i64() << 16);
                        clue_ids.insert((campaign.name, scenario.name, *location, n), id);
                        let _ = write!(writer, "\"{} - {} Clues {}\":{id},", scenario.name, card.unique_name(), n + 1);
                    }
                }

                for check in &mut scenario.checks {
                    if check.victory > 0 {
                        for victory in 0..check.victory {
                            if check.count > 1 {
                                for n in 0..check.count {
                                    let id = counts.get(campaign.i | scenario.i << 8 | check.code.i64() << 16);
                                    check.ids.push(id);
                                    let _ = write!(writer, "\"{} - {} Victory {} #{}\":{id},", scenario.name, check.name, victory + 1, n + 1);
                                }
                            } else {
                                let id = counts.get(campaign.i | scenario.i << 8 | check.code.i64() << 16);
                                check.ids.push(id);
                                let _ = write!(writer, "\"{} - {} Victory {}\":{id},", scenario.name, check.name, victory + 1);
                            }
                        }
                    } else {
                        if check.count > 1 {
                            for n in 0..check.count {
                                let id = counts.get(campaign.i | scenario.i << 8 | check.code.i64() << 16);
                                check.ids.push(id);
                                let _ = write!(writer, "\"{} - {} #{}\":{id},", scenario.name, check.name, n + 1);
                            }
                        } else {
                            let id = counts.get(campaign.i | scenario.i << 8 | check.code.i64() << 16);
                            check.ids.push(id);
                            let _ = write!(writer, "\"{} - {}\":{id},", scenario.name, check.name);
                        }
                    }

                    if check.goal {
                        goal_locations.insert(check.ids[0], campaign.name);
                    }
                }
            }
        }

        let _ = writeln!(writer, "}}");
    }

    Datapackage { item_from_id, clue_ids, victory_ids, goal_locations }
}

struct Counter {
    inner: HashMap<i64, i64>,
    offset: i64,
}

impl Counter {
    fn new(offset: i64) -> Counter {
        Counter { inner: HashMap::new(), offset }
    }

    fn get(&mut self, id: i64) -> i64 {
        if let Some(count) = self.inner.get_mut(&id) {
            *count += 1;
            id | *count << self.offset
        } else {
            self.inner.insert(id, 0);
            id
        }
    }
}
