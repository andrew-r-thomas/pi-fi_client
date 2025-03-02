pub mod cache;
mod main_stream;
pub mod player;

use cache::{Cache, GetAlbumResp, LibraryData};
use main_stream::{init_main_stream, MainStreamHandle};
use player::{Player, PlayerUpdateMsg};

use std::sync::{Arc, Mutex};

use tauri::{ipc::Channel, Manager, State};
use tauri_plugin_http::reqwest::Client;

// const SERVER_URL: &'static str = "http://192.168.50.68:8080";
const SERVER_URL: &'static str = "http://localhost:8080";

struct Systems {
    cache: Arc<Cache>,
    player: Mutex<Option<Player>>,
    client: Client,
    handle: Mutex<Option<MainStreamHandle>>,
}
impl Systems {
    pub fn new(handle: MainStreamHandle) -> Self {
        let client = Client::new();
        let cache = Arc::new(Cache::new(client.clone()));

        Self {
            client,
            cache,
            player: Mutex::new(None),
            handle: Mutex::new(Some(handle)),
        }
    }
}

#[tauri::command]
fn get_library(systems: State<'_, Systems>) -> Result<LibraryData, ()> {
    systems.cache.get_library()
}

#[tauri::command]
fn get_album(id: i64, systems: State<'_, Systems>) -> Result<GetAlbumResp, ()> {
    systems.cache.get_album(id)
}

#[tauri::command]
async fn play_track(id: i64, systems: State<'_, Systems>) -> Result<(), ()> {
    let player = { systems.player.lock().unwrap().as_ref().unwrap().clone() };
    player.play_track(id).await;
    Ok(())
}

#[tauri::command]
fn setup_player(systems: State<'_, Systems>, channel: Channel<PlayerUpdateMsg>) {
    let mut player = systems.player.lock().unwrap();
    let handle = systems.handle.lock().unwrap().take().unwrap();
    *player = Some(Player::new(
        systems.client.clone(),
        systems.cache.clone(),
        channel,
        handle,
    ))
}

#[tauri::command]
fn toggle_playing(systems: State<'_, Systems>) {
    systems
        .player
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .toggle_playing();
}

#[tauri::command]
fn skip(systems: State<'_, Systems>) {
    systems.player.lock().unwrap().as_ref().unwrap().skip();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (_stream, handle) = init_main_stream();
    println!("made stream");
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(Systems::new(handle));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_library,
            get_album,
            play_track,
            setup_player,
            toggle_playing,
            skip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
