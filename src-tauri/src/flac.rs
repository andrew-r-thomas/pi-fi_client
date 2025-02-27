use std::io::{Read, Seek};

use futures_util::Stream;
use symphonia::core::io::MediaSource;

pub struct AsyncMediaSource<S>
where
    S: Stream + Send + Sync,
{
    buf: Vec<u8>,
    reader: S,
}

impl<S> AsyncMediaSource<S>
where
    S: Stream + Send + Sync,
{
    pub fn new(reader: S) -> Self {
        Self {
            reader,
            buf: Vec::new(),
        }
    }
}

impl<S> MediaSource for AsyncMediaSource<S>
where
    S: Stream + Send + Sync,
{
    fn byte_len(&self) -> Option<u64> {
        None
    }
    fn is_seekable(&self) -> bool {
        false
    }
}

impl<S> Read for AsyncMediaSource<S>
where
    S: Stream + Send + Sync,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(0)
    }
}

impl<S> Seek for AsyncMediaSource<S>
where
    S: Stream + Send + Sync,
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }
}

