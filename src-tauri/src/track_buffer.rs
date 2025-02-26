use crate::SERVER_URL;

use std::io::{Read, Seek};

use rodio::{queue::queue, Source};
use tauri_plugin_http::reqwest::Client;

pub struct TrackStream {
    id: i64,
    client: Client,
}

impl TrackStream {
    pub async fn init(id: i64, client: Client) -> Self {
        // we need to fetch the metadata and at least a few seconds first,
        // we can return immediately after we have that, but we want to spawn
        // the task(s) to get the rest immediately too
        // (this behaviour may change if we run into issues with really long tracks

        // send request for first chunk, don't wait on it yet
        let first_chunk = client
            .get(format!("{SERVER_URL}/get-track?id={id}&range=first"))
            .send();


        Self { id, client }
    }
}

impl Iterator for TrackStream {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.next()
        todo!()
    }
}

impl Source for TrackStream {
    fn total_duration(&self) -> Option<std::time::Duration> {
        self.decoder.total_duration()
    }
    fn current_frame_len(&self) -> Option<usize> {
        self.decoder.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.decoder.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.decoder.sample_rate()
    }
}

struct Buf {
    buf: Vec<u8>,
    cursor: usize,
}
impl Seek for Buf {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        todo!()
    }
}
impl Read for Buf {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}
