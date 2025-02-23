pub mod app_state;
pub mod cache;

use app_state::{AppState, PlayerUpdateMsg};
use cache::{GetAlbumResp, LibraryData};
use rodio::{OutputStream, Sink};
use tauri::{ipc::Channel, Manager, State};

const SERVER_URL: &'static str = "http://192.168.50.68:8080";

#[tauri::command]
fn get_library(state: State<'_, AppState>) -> Result<LibraryData, ()> {
    state.get_library()
}

#[tauri::command]
fn get_album(id: i64, state: State<'_, AppState>) -> Result<GetAlbumResp, ()> {
    state.get_album(id)
}

#[tauri::command]
async fn play_track(id: i64, state: State<'_, AppState>) -> Result<(), ()> {
    state.play_track(id).await;
    Ok(())
}

#[tauri::command]
fn setup_player(state: State<'_, AppState>, channel: Channel<PlayerUpdateMsg>) {
    state.setup_player(channel);
}

#[tauri::command]
fn toggle_playing(state: State<'_, AppState>) {
    state.toggle_playing();
}

#[tauri::command]
fn skip(state: State<'_, AppState>) {
    state.skip();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(AppState::new(sink));
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
