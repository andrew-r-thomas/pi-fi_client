use futures_util::stream::StreamExt;
use rtrb::Consumer;
use std::{
    error::Error,
    fmt::Display,
    io::{self, Cursor, Read},
    sync::Arc,
};

use crate::SERVER_URL;

use claxon::{input::BufferedReader, FlacReader, FlacSamples};
use rodio::{OutputStreamHandle, Sink, Source};
use serde::Serialize;
use tauri::ipc::Channel;
use tauri_plugin_http::reqwest::Client;

use crate::cache::Cache;

#[derive(Clone)]
pub struct Player(Arc<PlayerInner>);
struct PlayerInner {
    sink: Sink,
    channel: Channel<PlayerUpdateMsg>,
    client: Client,
    cache: Arc<Cache>,
}

impl Player {
    pub fn new(
        stream_handle: &OutputStreamHandle,
        client: Client,
        cache: Arc<Cache>,
        channel: Channel<PlayerUpdateMsg>,
    ) -> Self {
        channel
            .send(PlayerUpdateMsg::UpdateCurrentTrack {
                current_track: CurrentTrack {
                    track_title: "Crusades".into(),
                    artist_title: "Geese".into(),
                    cover_art_id: 1,
                },
            })
            .unwrap();
        let sink = Sink::try_new(stream_handle).unwrap();
        Self(Arc::new(PlayerInner {
            sink,
            channel,
            client,
            cache,
        }))
    }
    pub async fn play_track(&self, id: i64) {
        let get_track = self
            .0
            .client
            .get(format!("{SERVER_URL}/get-track?id={id}"))
            .send();

        self.0.sink.clear();
        self.0
            .channel
            .send(PlayerUpdateMsg::UpdatePlaying {
                playing: Playing::Waiting,
            })
            .unwrap();

        let track = self.0.cache.get_track(id);
        self.0
            .channel
            .send(PlayerUpdateMsg::UpdateCurrentTrack {
                current_track: CurrentTrack {
                    track_title: track.title,
                    artist_title: track.artist_name,
                    cover_art_id: track.cover_art_id,
                },
            })
            .unwrap();

        let mut track_bytes = get_track.await.unwrap().bytes_stream();
        let first_chunk = track_bytes.next().await.unwrap();
        while let Some(chunk) = track_bytes.next().await {
            let thing = chunk.unwrap();
        }

        self.0.sink.play();
        self.0
            .channel
            .send(PlayerUpdateMsg::UpdatePlaying {
                playing: Playing::Playing,
            })
            .unwrap();
    }
    pub fn toggle_playing(&self) {
        let playing = match self.0.sink.is_paused() {
            true => {
                self.0.sink.play();
                Playing::Playing
            }
            false => {
                self.0.sink.pause();
                Playing::Paused
            }
        };
        self.0
            .channel
            .send(PlayerUpdateMsg::UpdatePlaying { playing })
            .unwrap();
    }
    pub fn skip(&self) {
        todo!()
    }
}

#[derive(Serialize, Clone)]
#[serde(tag = "event", content = "data")]
pub enum PlayerUpdateMsg {
    UpdatePlaying { playing: Playing },
    UpdateCurrentTrack { current_track: CurrentTrack },
}
#[derive(Serialize, Clone)]
pub enum Playing {
    Playing,
    Paused,
    Waiting,
}
#[derive(Serialize, Clone)]
pub struct CurrentTrack {
    track_title: String,
    artist_title: String,
    cover_art_id: i64,
}

// struct TrackStream<'ts> {
//     reader: FlacSamples<&'ts mut BufferedReader<Consumer<u8>>>,
//     flac: FlacReader<Consumer<u8>>,
// }
// impl TrackStream<'_> {
//     pub fn new(recv: Consumer<u8>) -> Self {
//         let mut flac = FlacReader::new(recv).unwrap();
//         let sample = flac.samples().next().unwrap();
//         let reader = flac.samples();
//         Self { reader, flac }
//     }
// }
// impl Iterator for TrackStream<'_> {
//     type Item = f32;
//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }
// impl Source for TrackStream<'_> {
//     fn total_duration(&self) -> Option<std::time::Duration> {
//         None
//     }
//     fn sample_rate(&self) -> u32 {
//         self.flac.streaminfo().sample_rate
//     }
//     fn channels(&self) -> u16 {
//         self.flac.streaminfo().channels as u16
//     }
//     fn current_frame_len(&self) -> Option<usize> {
//         None
//     }
// }
