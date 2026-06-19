use crate::protocol::{Connected, GameData as ArchipelagoGameData, RoomInfo};
use serde_json::from_str;
use std::{collections::HashMap, sync::Arc};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetDirectoryOptions, FileSystemGetFileOptions, FileSystemWritableFileStream, TextDecoder, js_sys::ArrayBuffer, window};

pub type Requested = HashMap<String, String>;

#[derive(Debug, Clone)]
struct GameData {
    item_id_to_name: HashMap<i64, String>,
    location_id_to_name: HashMap<i64, String>,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct DatapackageStore {
    fs: Option<FileSystemDirectoryHandle>,
    data: HashMap<String, Arc<GameData>>,
    missing: HashMap<String, String>,
    player_to_game: HashMap<i32, Arc<GameData>>,
}

#[wasm_bindgen]
impl DatapackageStore {
    pub async fn get_fs(&mut self) {
        if let Some(window) = window() {
            let promise = JsFuture::from(window.navigator().storage().get_directory());
            self.fs = promise.await.map(FileSystemDirectoryHandle::from).ok();
        }
    }

    pub async fn load_cached_datapackages(&mut self) {
        let mut to_load = HashMap::new();

        if let Some(fs) = &self.fs {
            for (game, checksum) in &self.missing {
                if let Ok(game_dir) = JsFuture::from(fs.get_directory_handle_with_options(game, &create_dir())).await.map(FileSystemDirectoryHandle::from)
                    && let Ok(file_handle) = JsFuture::from(game_dir.get_file_handle_with_options(checksum, &create_file())).await.map(FileSystemFileHandle::from)
                    && let Ok(file) = JsFuture::from(file_handle.get_file()).await.map(File::from)
                    && file.size() > 0.0
                    && let Ok(content) = JsFuture::from(file.array_buffer()).await.map(ArrayBuffer::from)
                    && let Ok(decoder) = TextDecoder::new()
                    && let Ok(str) = decoder.decode_with_buffer_source(&content)
                    && let Ok(data) = from_str::<ArchipelagoGameData>(&str)
                {
                    to_load.insert(game.to_owned(), data);
                }
            }
        }

        for (game, data) in to_load {
            self.missing.remove(&game);
            self.add_game_internal(game, data);
        }
    }

    pub async fn add_game(&mut self, game: String, data: &str) {
        if let Ok(game_data) = from_str(data) {
            if let Some(fs) = &self.fs
                && let Some(checksum) = self.missing.get(&game)
                && let Ok(game_dir) = JsFuture::from(fs.get_directory_handle_with_options(&game, &create_dir())).await.map(FileSystemDirectoryHandle::from)
                && let Ok(file_handle) = JsFuture::from(game_dir.get_file_handle_with_options(checksum, &create_file())).await.map(FileSystemFileHandle::from)
                && let Ok(writable) = JsFuture::from(file_handle.create_writable()).await.map(FileSystemWritableFileStream::from)
                && let Ok(res) = writable.write_with_str(data)
                && JsFuture::from(res).await.is_ok()
            {
                let _ = JsFuture::from(writable.close()).await;
            }

            self.add_game_internal(game, game_data)
        }
    }

    fn add_game_internal(&mut self, game: String, data: ArchipelagoGameData) {
        let mut item_id_to_name = HashMap::new();
        let mut location_id_to_name = HashMap::new();

        for (item, id) in data.item_name_to_id {
            item_id_to_name.insert(id, item);
        }
        for (location, id) in data.location_name_to_id {
            location_id_to_name.insert(id, location);
        }

        self.data.insert(game, Arc::new(GameData { item_id_to_name, location_id_to_name }));
    }

    pub fn get_missing_games(&self) -> Vec<String> {
        self.missing.keys().cloned().collect()
    }
}

impl DatapackageStore {
    pub fn new(requested: Requested) -> Self {
        let mut new = Self {
            fs: None,
            data: HashMap::new(),
            missing: HashMap::new(),
            player_to_game: HashMap::new(),
        };

        for (game, checksum) in requested {
            new.missing.insert(game, checksum);
        }

        new
    }

    pub fn build_player_map(&mut self, connected: &Connected) {
        for player in connected.players.iter().map(|p| p.slot) {
            if let Some(slot) = connected.slot_info.get(&*player.to_string())
                && let Some(rc) = self.data.get(&*slot.game)
            {
                self.player_to_game.insert(player, rc.clone());
            }
        }
    }

    pub fn get_item(&self, player: i32, id: i64) -> &str {
        if let Some(data) = self.player_to_game.get(&player)
            && let Some(item) = data.item_id_to_name.get(&id)
        {
            return item;
        }

        "Unknown item"
    }

    pub fn get_location(&self, player: i32, id: i64) -> &str {
        if let Some(data) = self.player_to_game.get(&player)
            && let Some(location) = data.location_id_to_name.get(&id)
        {
            return location;
        }

        "Unknown location"
    }
}

#[wasm_bindgen]
pub fn new_datapackage_store(room_info: &str) -> DatapackageStore {
    if let Ok(room_info) = from_str::<RoomInfo>(room_info) {
        DatapackageStore::new(room_info.datapackage_checksums)
    } else {
        panic!("Failed to parse room_info")
    }
}

fn create_dir() -> FileSystemGetDirectoryOptions {
    let options = FileSystemGetDirectoryOptions::new();
    options.set_create(true);
    options
}

fn create_file() -> FileSystemGetFileOptions {
    let options = FileSystemGetFileOptions::new();
    options.set_create(true);
    options
}
