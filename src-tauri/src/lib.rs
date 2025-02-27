pub mod cache;
mod flac;
mod main_stream;
pub mod player;
pub mod track_buffer;

use std::sync::{Arc, Mutex};

use cache::{Cache, GetAlbumResp, LibraryData};
use player::{Player, PlayerUpdateMsg};
use rodio::{OutputStream, OutputStreamHandle};
use tauri::{ipc::Channel, Manager, State};
use tauri_plugin_http::reqwest::Client;

const SERVER_URL: &'static str = "http://192.168.50.68:8080";

struct Systems {
    cache: Arc<Cache>,
    player: Mutex<Option<Player>>,
    client: Client,
    stream_handle: OutputStreamHandle,
}
impl Systems {
    pub fn new(stream_handle: OutputStreamHandle) -> Self {
        let client = Client::new();
        let cache = Arc::new(Cache::new(client.clone()));

        Self {
            client,
            cache,
            player: Mutex::new(None),
            stream_handle,
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
    *player = Some(Player::new(
        &systems.stream_handle,
        systems.client.clone(),
        systems.cache.clone(),
        channel,
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
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(Systems::new(stream_handle));
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
