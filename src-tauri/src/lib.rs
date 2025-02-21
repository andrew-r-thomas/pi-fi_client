use std::{collections::BTreeMap, io::Cursor, sync::Mutex, time::Duration};

use rodio::{source::SineWave, Decoder, OutputStream, Sink, Source};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tauri_plugin_http::reqwest::Client;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

struct AppState {
    client: Client,
    sink: Sink,
    album_cache: Mutex<BTreeMap<i64, Album>>,
    artist_cache: Mutex<BTreeMap<i64, Artist>>,
    track_cache: Mutex<BTreeMap<i64, Track>>,
}
struct Album {
    title: String,
    artist_id: i64,
    track_ids: Vec<i64>,
}
struct Artist {
    name: String,
}
struct Track {
    title: String,
    track_number: u32,
    artist_id: i64,
    album_id: i64,
}

impl AppState {
    pub fn new(sink: Sink) -> Self {
        let client = Client::new();
        let mut album_cache = BTreeMap::new();
        let mut artist_cache = BTreeMap::new();
        let mut track_cache = BTreeMap::new();

        let get_lib_resp = tauri::async_runtime::block_on(async {
            client
                .get(format!("{SERVER_URL}/get-library"))
                .send()
                .await
                .unwrap()
                .json::<GetLibResp>()
                .await
                .unwrap()
        });

        for album in get_lib_resp.albums {
            album_cache.insert(
                album.id,
                Album {
                    title: album.title,
                    artist_id: album.artist_id,
                    track_ids: album.track_ids,
                },
            );
        }
        for artist in get_lib_resp.artists {
            artist_cache.insert(artist.id, Artist { name: artist.name });
        }
        for track in get_lib_resp.tracks {
            track_cache.insert(
                track.id,
                Track {
                    title: track.title,
                    artist_id: track.artist_id,
                    album_id: track.album_id,
                    track_number: track.track_number,
                },
            );
        }

        Self {
            client,
            sink,
            album_cache: Mutex::new(album_cache),
            artist_cache: Mutex::new(artist_cache),
            track_cache: Mutex::new(track_cache),
        }
    }
}

#[derive(Deserialize, Clone)]
struct GetLibResp {
    albums: Vec<GetLibRespAlbum>,
    artists: Vec<GetLibRespArtist>,
    tracks: Vec<GetLibRespTrack>,
}
#[derive(Deserialize, Clone)]
struct GetLibRespAlbum {
    id: i64,
    title: String,
    artist_id: i64,
    track_ids: Vec<i64>,
}
#[derive(Deserialize, Clone)]
struct GetLibRespArtist {
    id: i64,
    name: String,
}
#[derive(Deserialize, Clone)]
struct GetLibRespTrack {
    id: i64,
    title: String,
    artist_id: i64,
    album_id: i64,
    track_number: u32,
}

#[derive(Serialize)]
struct LibraryData {
    albums: Vec<AlbumData>,
}
#[derive(Serialize)]
struct AlbumData {
    id: i64,
    title: String,
    artist_name: String,
}

#[tauri::command]
async fn get_library(state: State<'_, AppState>) -> Result<LibraryData, ()> {
    let album_cache = state.album_cache.lock().unwrap();
    let mut albums = Vec::with_capacity(album_cache.len());
    for (album_id, album_data) in album_cache.iter() {
        let artist_cache = state.artist_cache.lock().unwrap();
        albums.push(AlbumData {
            id: *album_id,
            title: album_data.title.clone(),
            artist_name: artist_cache
                .get(&album_data.artist_id)
                .unwrap()
                .name
                .clone(),
        });
    }

    Ok(LibraryData { albums })
}

#[derive(Serialize)]
struct GetAlbumResp {
    title: String,
    artist_name: String,
    artist_id: i64,
    tracks: Vec<GetAlbumRespTrack>,
}
#[derive(Serialize)]
struct GetAlbumRespTrack {
    id: i64,
    title: String,
    track_number: u32,
}

#[tauri::command]
async fn get_album(id: i64, state: State<'_, AppState>) -> Result<GetAlbumResp, ()> {
    let album_cache = state.album_cache.lock().unwrap();
    match album_cache.get(&id) {
        Some(a) => {
            let track_cache = state.track_cache.lock().unwrap();
            let mut tracks = Vec::new();
            for track_id in a.track_ids.iter() {
                let track = track_cache.get(track_id).unwrap();
                tracks.push(GetAlbumRespTrack {
                    id: *track_id,
                    title: track.title.clone(),
                    track_number: track.track_number,
                });
            }
            tracks.sort_by(|a, b| a.track_number.cmp(&b.track_number));
            let artist_cache = state.artist_cache.lock().unwrap();
            let artist = artist_cache.get(&a.artist_id).unwrap();
            Ok(GetAlbumResp {
                title: a.title.clone(),
                artist_id: a.artist_id,
                artist_name: artist.name.clone(),
                tracks,
            })
        }
        None => todo!("fetch from server"),
    }
}

#[tauri::command]
async fn play_track(id: i64, state: State<'_, AppState>) -> Result<(), ()> {
    let resp = state
        .client
        .get(format!("{SERVER_URL}/get-track?id={id}"))
        .send()
        .await
        .unwrap();

    let bytes = resp.bytes().await.unwrap();
    let cursor = Cursor::new(bytes);
    let decoder = Decoder::new(cursor).unwrap();
    state.sink.append(decoder);

    Ok(())
}

const SERVER_URL: &'static str = "http://192.168.50.68:8080";

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
            greet,
            get_library,
            get_album,
            play_track
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
