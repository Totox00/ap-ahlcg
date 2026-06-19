use crate::datapackage::DatapackageStore;
use crate::protocol::PrintJSON;
use std::collections::HashMap;

pub fn format(datapackage_store: &DatapackageStore, msg: PrintJSON, players: &HashMap<i32, String>, slot: &str) -> String {
    msg.data
        .iter()
        .map(|part| {
            let mut t_str = "text";
            if let Some(str) = &part.r#type {
                t_str = str;
            }
            let text = part.text.clone().unwrap_or_default();
            match t_str {
                "player_id" => {
                    if let Some(player) = players.get(&text.parse::<i32>().unwrap_or(0)) {
                        if slot == *player {
                            style(player, "self")
                        } else {
                            style(player, "player")
                        }
                    } else {
                        style(&format!("Unknown player {text}"), "player")
                    }
                }
                "player_name" => {
                    if slot == text {
                        style(&text, "self")
                    } else {
                        style(&text, "player")
                    }
                }
                "item_id" => style_item(datapackage_store.get_item(part.player.unwrap_or(0), text.parse().unwrap_or(0)), part.flags.unwrap_or(0)),
                "item_name" => style(&text, "filler"),
                "location_id" => style(datapackage_store.get_location(part.player.unwrap_or(0), text.parse().unwrap_or(0)), "location"),
                "location_name" => style(&text, "location"),
                "entrance_name" => style(&text, "entrance"),
                "color" => {
                    if let Some(color) = &part.color {
                        format!("<span style=\"color:{color}\">{text}</span>")
                    } else {
                        text
                    }
                }
                _ => text,
            }
        })
        .collect()
}

fn style_item(str: &str, flags: i32) -> String {
    match flags {
        0b001 | 0b111 => style(str, "progression"),
        0b010 => style(str, "useful"),
        0b011 => style(str, "progression-useful"),
        0b100 => style(str, "trap"),
        0b101 => style(str, "progression-trap"),
        0b110 => style(str, "useful-trap"),
        _ => style(str, "filler"),
    }
}

fn style(str: &str, class: &str) -> String {
    format!("<span class=\"{class}\">{str}</span>")
}
