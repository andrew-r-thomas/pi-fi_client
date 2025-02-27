use std::sync::Arc;
use symphonia::{
    core::{
        codecs::Decoder,
        formats::FormatReader,
        io::{MediaSourceStream, MediaSourceStreamOptions},
    },
    default::{codecs::FlacDecoder, formats::FlacReader},
};

use crate::{flac::AsyncMediaSource, main_stream::MainStreamHandle, SERVER_URL};

use serde::Serialize;
use tauri::{async_runtime::spawn, ipc::Channel};
use tauri_plugin_http::reqwest::{self, Client};

use crate::cache::Cache;

#[derive(Clone)]
pub struct Player(Arc<PlayerInner>);
struct PlayerInner {
    channel: Channel<PlayerUpdateMsg>,
    client: Client,
    cache: Arc<Cache>,
    main_stream_handle: MainStreamHandle,
}

impl Player {
    pub fn new(client: Client, cache: Arc<Cache>, channel: Channel<PlayerUpdateMsg>) -> Self {
        channel
            .send(PlayerUpdateMsg::UpdateCurrentTrack {
                current_track: CurrentTrack {
                    track_title: "Crusades".into(),
                    artist_title: "Geese".into(),
                    cover_art_id: 1,
                },
            })
            .unwrap();
        let main_stream_handle = MainStreamHandle::new();
        Self(Arc::new(PlayerInner {
            channel,
            client,
            cache,
            main_stream_handle,
        }))
    }
    pub async fn play_track(&self, id: i64) {
        let req = self
            .0
            .client
            .get(format!("{SERVER_URL}/get-track?id={id}"))
            .send();

        self.0.main_stream_handle.pause();
        self.0.main_stream_handle.clear();

        let track = self.0.cache.get_track(id);
        self.0.channel.send(PlayerUpdateMsg::UpdateCurrentTrack {
            current_track: CurrentTrack {
                track_title: track.title.clone(),
                artist_title: track.artist_name.clone(),
                cover_art_id: track.cover_art_id,
            },
        });
        let stream = req.await.unwrap().bytes_stream();
        let src = AsyncMediaSource::new(stream);
        let src_stream = MediaSourceStream::new(Box::new(src), MediaSourceStreamOptions::default());
        let reader = FlacReader::try_new(
            src_stream,
            &symphonia::core::formats::FormatOptions::default(),
        )
        .unwrap();
    }
    pub fn toggle_playing(&self) {
        todo!()
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
