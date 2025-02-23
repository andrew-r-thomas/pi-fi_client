use crate::{
    cache::{Cache, GetAlbumResp, LibraryData},
    SERVER_URL,
};
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

use rodio::{source::EmptyCallback, Decoder, Sink};
use serde::Serialize;
use tauri::{async_runtime::spawn, ipc::Channel};
use tauri_plugin_http::reqwest::Client;

pub struct AppState {
    client: Client,

    cache: Arc<Cache>,

    player: Arc<Player>,
}

impl AppState {
    pub fn new(sink: Sink) -> Self {
        let client = Client::new();
        let cache = Arc::new(Cache::new(client.clone()));

        Self {
            player: Arc::new(Player {
                cache: cache.clone(),
                client: client.clone(),
                sink,
                channel: Mutex::new(None),
            }),
            client,
            cache,
        }
    }

    pub fn get_library(&self) -> Result<LibraryData, ()> {
        self.cache.get_library()
    }

    pub fn get_album(&self, id: i64) -> Result<GetAlbumResp, ()> {
        self.cache.get_album(id)
    }

    pub fn setup_player(&self, channel: Channel<PlayerUpdateMsg>) {
        // TODO: get actual most recent track
        channel
            .send(PlayerUpdateMsg::UpdateCurrentTrack {
                current_track: CurrentTrack {
                    track_title: "Crusades".into(),
                    artist_title: "Geese".into(),
                    cover_art_id: 1,
                },
            })
            .unwrap();

        let mut player_channel = self.player.channel.lock().unwrap();
        *player_channel = Some(channel);
    }

    pub fn toggle_playing(&self) {
        let playing = match self.player.sink.is_paused() {
            true => {
                self.player.sink.play();
                true
            }
            false => {
                self.player.sink.pause();
                false
            }
        };
        self.player
            .channel
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .send(PlayerUpdateMsg::UpdatePlaying { playing })
            .unwrap();
    }

    pub async fn play_track(&self, id: i64) {
        let track = self.cache.get_track(id);
        {
            self.player
                .channel
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .send(PlayerUpdateMsg::UpdateCurrentTrack {
                    current_track: CurrentTrack {
                        track_title: track.title,
                        artist_title: track.artist_name,
                        cover_art_id: track.cover_art_id,
                    },
                })
                .unwrap();
        }

        let resp = self
            .client
            .get(format!("{SERVER_URL}/get-track?id={id}"))
            .send()
            .await
            .unwrap();

        let bytes = resp.bytes().await.unwrap();
        let cursor = Cursor::new(bytes);
        let decoder = Decoder::new(cursor).unwrap();

        self.player.sink.clear();
        self.player.sink.append(decoder);
        self.player.sink.play();

        self.player
            .channel
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .send(PlayerUpdateMsg::UpdatePlaying { playing: true })
            .unwrap();

        if let Some(next) = track.next_track {
            let player = self.player.clone();
            self.player
                .sink
                .append(EmptyCallback::<f32>::new(Box::new(move || {
                    Player::next_track_cb(player.clone(), next);
                })));
        }
    }

    pub fn skip(&self) {
        self.player.sink.skip_one();
    }
}

struct Player {
    sink: Sink,
    channel: Mutex<Option<Channel<PlayerUpdateMsg>>>,
    client: Client,
    cache: Arc<Cache>,
}

impl Player {
    fn next_track_cb(player: Arc<Self>, id: i64) {
        spawn(async move {
            let track = player.cache.get_track(id);
            {
                player
                    .channel
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .send(PlayerUpdateMsg::UpdateCurrentTrack {
                        current_track: CurrentTrack {
                            track_title: track.title,
                            artist_title: track.artist_name,
                            cover_art_id: track.cover_art_id,
                        },
                    })
                    .unwrap();
            }

            let resp = player
                .client
                .get(format!("{SERVER_URL}/get-track?id={id}"))
                .send()
                .await
                .unwrap();

            let bytes = resp.bytes().await.unwrap();
            let cursor = Cursor::new(bytes);
            let decoder = Decoder::new(cursor).unwrap();

            player.sink.append(decoder);

            player
                .channel
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .send(PlayerUpdateMsg::UpdatePlaying { playing: true })
                .unwrap();

            if let Some(next) = track.next_track {
                let p = player.clone();
                player
                    .sink
                    .append(EmptyCallback::<f32>::new(Box::new(move || {
                        Player::next_track_cb(p.clone(), next);
                    })));
            }
        });
    }
}

// TODO: these are little serialization structs for data passed between server
// rust end and frontend. organize them

#[derive(Serialize, Clone)]
#[serde(tag = "event", content = "data")]
pub enum PlayerUpdateMsg {
    UpdatePlaying { playing: bool },
    UpdateCurrentTrack { current_track: CurrentTrack },
}
#[derive(Serialize, Clone)]
pub struct CurrentTrack {
    track_title: String,
    artist_title: String,
    cover_art_id: i64,
}
