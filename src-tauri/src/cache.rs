use std::{collections::BTreeMap, sync::Mutex};

use crate::SERVER_URL;

use serde::{Deserialize, Serialize};
use tauri::async_runtime::block_on;
use tauri_plugin_http::reqwest::Client;

pub struct Cache {
    albums: Mutex<BTreeMap<i64, Album>>,
    artists: Mutex<BTreeMap<i64, Artist>>,
    tracks: Mutex<BTreeMap<i64, Track>>,

    client: Client,
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

impl Cache {
    pub fn new(client: Client) -> Self {
        let mut albums = BTreeMap::new();
        let mut artists = BTreeMap::new();
        let mut tracks = BTreeMap::new();

        let get_lib_resp = block_on(async {
            let resp = client
                .get(format!("{SERVER_URL}/get-library"))
                .send()
                .await
                .unwrap();
            resp.json::<GetLibResp>().await.unwrap()
        });

        for album in get_lib_resp.albums {
            albums.insert(
                album.id,
                Album {
                    title: album.title,
                    artist_id: album.artist_id,
                    track_ids: album.track_ids,
                },
            );
        }
        for artist in get_lib_resp.artists {
            artists.insert(artist.id, Artist { name: artist.name });
        }
        for track in get_lib_resp.tracks {
            tracks.insert(
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
            albums: Mutex::new(albums),
            artists: Mutex::new(artists),
            tracks: Mutex::new(tracks),
            client,
        }
    }

    pub fn get_library(&self) -> Result<LibraryData, ()> {
        let album_cache = self.albums.lock().unwrap();
        let mut albums = Vec::with_capacity(album_cache.len());
        for (album_id, album_data) in album_cache.iter() {
            let artist_cache = self.artists.lock().unwrap();
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

    pub fn get_album(&self, id: i64) -> Result<GetAlbumResp, ()> {
        let album_cache = self.albums.lock().unwrap();
        match album_cache.get(&id) {
            Some(a) => {
                let track_cache = self.tracks.lock().unwrap();
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
                let artist_cache = self.artists.lock().unwrap();
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

    pub fn get_track(&self, id: i64) -> GetTrackResp {
        let tracks = self.tracks.lock().unwrap();
        let artists = self.artists.lock().unwrap();
        let albums = self.albums.lock().unwrap();

        let track = tracks.get(&id).unwrap();
        let artist = artists.get(&track.artist_id).unwrap();
        let album = albums.get(&track.album_id).unwrap();

        // TODO: really want to figure out a way to keep these sorted by track_number
        let mut sorted_tracks = Vec::new();
        for track_id in &album.track_ids {
            let t = tracks.get(track_id).unwrap();
            sorted_tracks.push((*track_id, t.track_number));
        }
        sorted_tracks.sort_by(|a, b| a.1.cmp(&b.1));
        let next_track = match sorted_tracks.get(track.track_number as usize) {
            Some(n) => Some(n.0),
            None => None,
        };

        GetTrackResp {
            title: track.title.clone(),
            artist_name: artist.name.clone(),
            cover_art_id: track.album_id,
            next_track,
        }
    }
}

#[derive(Serialize)]
pub struct LibraryData {
    albums: Vec<AlbumData>,
}
#[derive(Serialize)]
pub struct AlbumData {
    id: i64,
    title: String,
    artist_name: String,
}
#[derive(Serialize)]
pub struct GetAlbumResp {
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

pub struct GetTrackResp {
    pub title: String,
    pub artist_name: String,
    pub cover_art_id: i64,
    pub next_track: Option<i64>,
}
