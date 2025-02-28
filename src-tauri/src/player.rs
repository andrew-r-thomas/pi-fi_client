use std::{io::Cursor, sync::Arc};
use symphonia::{
    core::{
        audio::SampleBuffer,
        io::{MediaSourceStream, MediaSourceStreamOptions},
        probe::Hint,
    },
    default,
};

use crate::{
    main_stream::{spawn_track_stream, MainStreamHandle},
    SERVER_URL,
};

use serde::Serialize;
use tauri::{async_runtime::spawn, ipc::Channel};
use tauri_plugin_http::reqwest::Client;

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
    pub fn new(
        client: Client,
        cache: Arc<Cache>,
        channel: Channel<PlayerUpdateMsg>,
        main_stream_handle: MainStreamHandle,
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
        self.0
            .channel
            .send(PlayerUpdateMsg::UpdateCurrentTrack {
                current_track: CurrentTrack {
                    track_title: track.title.clone(),
                    artist_title: track.artist_name.clone(),
                    cover_art_id: track.cover_art_id,
                },
            })
            .unwrap();
        let bytes = req.await.unwrap().bytes().await.unwrap();
        let src = Cursor::new(bytes);
        let src_stream = MediaSourceStream::new(Box::new(src), MediaSourceStreamOptions::default());
        let mut reader = default::get_probe()
            .format(
                &Hint::new().with_extension("flac"),
                src_stream,
                &Default::default(),
                &Default::default(),
            )
            .unwrap();
        let track = reader.format.default_track().unwrap();
        let mut decoder = default::get_codecs()
            .make(&track.codec_params, &Default::default())
            .unwrap();

        let (stream, mut handle) = spawn_track_stream();
        self.0.main_stream_handle.queue(stream);
        self.0.main_stream_handle.play();

        self.0
            .channel
            .send(PlayerUpdateMsg::UpdatePlaying { playing: true })
            .unwrap();

        spawn(async move {
            while let Ok(packet) = reader.format.next_packet() {
                println!("got packet");
                let buf = decoder.decode(&packet).unwrap();
                let mut samples = SampleBuffer::new(buf.capacity() as u64, *buf.spec());
                samples.copy_interleaved_ref(buf);
                handle.send(samples.samples()).await;
                println!("sent buf");
            }
        });
    }
    pub fn toggle_playing(&self) {
        let playing = self.0.main_stream_handle.toggle_playing();
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
    UpdatePlaying { playing: bool },
    UpdateCurrentTrack { current_track: CurrentTrack },
}
#[derive(Serialize, Clone)]
pub struct CurrentTrack {
    track_title: String,
    artist_title: String,
    cover_art_id: i64,
}
