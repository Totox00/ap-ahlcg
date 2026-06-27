use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum Permission {
    Disabled = 0,
    Enabled = 1,
    Goal = 2,
    Auto = 6,
    AutoEnabled = 7,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkVersion {
    pub major: i32,
    pub minor: i32,
    pub build: i32,
    pub class: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkPlayer {
    pub team: i32,
    pub slot: i32,
    pub alias: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkItem {
    pub item: i64,
    pub location: i64,
    pub player: i64,
    pub flags: i64,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum SlotType {
    Spectator = 0,
    Player = 1,
    Group = 2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkSlot {
    pub name: String,
    pub game: String,
    pub r#type: SlotType,
    pub group_members: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomInfo {
    pub version: NetworkVersion,
    pub tags: Vec<String>,
    pub password: bool,
    pub permissions: HashMap<String, Permission>,
    pub hint_cost: i32,
    pub location_check_points: i32,
    pub games: Vec<String>,
    pub datapackage_checksums: HashMap<String, String>,
    pub seed_name: String,
    pub time: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connected {
    pub slot: i32,
    pub players: Vec<NetworkPlayer>,
    pub missing_locations: Vec<i64>,
    pub checked_locations: Vec<i64>,
    pub slot_data: RawSlotData,
    pub slot_info: HashMap<String, NetworkSlot>,
}

#[derive(Debug, Clone, Copy)]
pub struct SlotData {
    pub required_campaigns: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RawSlotData {
    pub g: usize,
}

impl TryFrom<RawSlotData> for SlotData {
    fn try_from(value: RawSlotData) -> Result<Self, ()> {
        Ok(Self { required_campaigns: value.g })
    }

    type Error = ();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintJSON {
    pub data: Vec<JSONMessagePart>,
    pub r#type: Option<String>,
    pub receiving: Option<i32>,
    pub item: Option<NetworkItem>,
    pub found: Option<bool>,
    pub countdown: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JSONMessagePart {
    pub r#type: Option<String>,
    pub text: Option<String>,
    pub color: Option<String>,
    pub flags: Option<i32>,
    pub player: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    pub item_name_to_id: HashMap<String, i64>,
    pub location_name_to_id: HashMap<String, i64>,
}
