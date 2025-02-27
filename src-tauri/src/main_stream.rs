use std::{
    future::Future,
    io::Read,
    sync::{
        atomic::{AtomicBool, AtomicPtr, Ordering},
        Arc, Mutex,
    },
    task::{Poll, Waker},
};

use claxon::FlacReader;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, FromSample, Sample, SizedSample, Stream, StreamConfig,
};
use rtrb::{chunks::ChunkError, Consumer, Producer, RingBuffer};
use tokio::io::{AsyncRead, AsyncWrite};

/// this is basically a specialized handle to the main audio thread that
/// understands the context of a streamed music player
pub struct MainStreamHandle {
    stream: Stream,
    playing: Arc<AtomicBool>,
    queue: Arc<Mutex<Producer<TrackStream>>>,
    clear: Arc<AtomicBool>,
}

impl MainStreamHandle {
    pub fn new() -> Self {
        let (queue, recv) = RingBuffer::new(256);

        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();

        let playing = Arc::new(AtomicBool::new(false));
        let clear = Arc::new(AtomicBool::new(false));

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => build_main_stream::<i8>(
                &device,
                &config.into(),
                recv,
                playing.clone(),
                clear.clone(),
            )
            .unwrap(),
            cpal::SampleFormat::I16 => build_main_stream::<i16>(
                &device,
                &config.into(),
                recv,
                playing.clone(),
                clear.clone(),
            )
            .unwrap(),
            cpal::SampleFormat::I32 => build_main_stream::<i32>(
                &device,
                &config.into(),
                recv,
                playing.clone(),
                clear.clone(),
            )
            .unwrap(),
            cpal::SampleFormat::I64 => build_main_stream::<i64>(
                &device,
                &config.into(),
                recv,
                playing.clone(),
                clear.clone(),
            )
            .unwrap(),
            cpal::SampleFormat::U8 => build_main_stream::<u8>(
                &device,
                &config.into(),
                recv,
                playing.clone(),
                clear.clone(),
            )
            .unwrap(),
            cpal::SampleFormat::U16 => build_main_stream::<u16>(
                &device,
                &config.into(),
                recv,
                playing.clone(),
                clear.clone(),
            )
            .unwrap(),
            cpal::SampleFormat::U32 => build_main_stream::<u32>(
                &device,
                &config.into(),
                recv,
                playing.clone(),
                clear.clone(),
            )
            .unwrap(),
            cpal::SampleFormat::U64 => build_main_stream::<u64>(
                &device,
                &config.into(),
                recv,
                playing.clone(),
                clear.clone(),
            )
            .unwrap(),
            cpal::SampleFormat::F32 => build_main_stream::<f32>(
                &device,
                &config.into(),
                recv,
                playing.clone(),
                clear.clone(),
            )
            .unwrap(),
            cpal::SampleFormat::F64 => build_main_stream::<f64>(
                &device,
                &config.into(),
                recv,
                playing.clone(),
                clear.clone(),
            )
            .unwrap(),
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        };

        Self {
            clear,
            queue: Arc::new(Mutex::new(queue)),
            stream,
            playing,
        }
    }
    pub fn toggle_playing(&self) -> bool {
        let playing = !self.playing.fetch_not(Ordering::AcqRel);
        if playing {
            self.stream.play();
        } else {
            self.stream.pause();
        }
        playing
    }
    pub fn clear(&self) {
        self.clear.store(true, Ordering::Release);
    }
    pub fn queue_tracks(&self, tracks: Vec<TrackStream>) {
        let mut queue = self.queue.lock().unwrap();
        for track in tracks {
            queue.push(track);
        }
    }
    pub fn pause(&self) {
        self.playing.store(false, Ordering::Release);
        self.stream.pause();
    }
    pub fn play(&self) {
        self.playing.store(true, Ordering::Release);
        self.stream.play();
    }
}

struct MainStream {
    current_track: Option<TrackStream>,
    queue: Consumer<TrackStream>,
    playing: Arc<AtomicBool>,
    clear: Arc<AtomicBool>,
}
impl MainStream {
    pub fn new(
        queue: Consumer<TrackStream>,
        playing: Arc<AtomicBool>,
        clear: Arc<AtomicBool>,
    ) -> Self {
        Self {
            queue,
            current_track: None,
            playing,
            clear,
        }
    }

    pub fn cb<S: Sample + Silence + FromSample<i32>>(&mut self, buf: &mut [S]) {
        // output silence by default
        buf.fill(S::silence());

        if self.clear.load(Ordering::Acquire) {
            let c = self.queue.read_chunk(self.queue.slots()).unwrap();
            c.commit_all();
            self.clear.store(false, Ordering::Release);
        }

        if self.playing.load(Ordering::Acquire) {
            // set up current track if needed
            if self.current_track.is_none() {
                match self.queue.pop() {
                    Ok(t) => {
                        self.current_track = Some(t);
                    }
                    Err(_) => return,
                }
            }

            // ask current track to fill up samples
            if let ReadSamplesResult::Done(n) =
                self.current_track.as_mut().unwrap().read_samples(buf)
            {
                if let Ok(mut t) = self.queue.pop() {
                    t.read_samples(&mut buf[n..]);
                    self.current_track = Some(t);
                }
            }
        }
    }
}

fn build_main_stream<S>(
    device: &Device,
    config: &StreamConfig,
    recv: Consumer<TrackStream>,
    playing: Arc<AtomicBool>,
    clear: Arc<AtomicBool>,
) -> Result<Stream, ()>
where
    S: SizedSample + FromSample<i32> + Silence + Send + 'static,
{
    let mut ms = MainStream::new(recv, playing, clear);
    let stream = device
        .build_output_stream(config, move |buf: &mut [S], _| ms.cb(buf), |_| {}, None)
        .unwrap();
    stream.pause();
    Ok(stream)
}

enum ReadSamplesResult {
    Ok,
    Done(usize),
    Waiting,
}

pub struct TrackStream {
    recv: Consumer<i32>,
    wakers: Consumer<Waker>,
}
impl TrackStream {
    pub fn new(recv: Consumer<i32>, wakers: Consumer<Waker>) -> Self {
        Self { recv, wakers }
    }
    fn read_samples<S: Sample + FromSample<i32>>(&mut self, buf: &mut [S]) -> ReadSamplesResult {
        match self.recv.read_chunk(buf.len()) {
            Ok(c) => {
                let (s1, s2) = c.as_slices();
                for (s, b) in s1.iter().chain(s2.iter()).zip(buf.iter_mut()) {
                    *b = S::from_sample(*s);
                }
                c.commit_all();
                if let Ok(w) = self.wakers.pop() {
                    w.wake();
                }
                ReadSamplesResult::Ok
            }
            Err(e) => {
                let n = {
                    match e {
                        ChunkError::TooFewSlots(0) => 0,
                        ChunkError::TooFewSlots(n) => {
                            let c = self.recv.read_chunk(n).unwrap();
                            let (s1, s2) = c.as_slices();
                            for (s, b) in s1.iter().chain(s2.iter()).zip(buf.iter_mut()) {
                                *b = S::from_sample(*s);
                            }
                            c.commit_all();
                            if let Ok(w) = self.wakers.pop() {
                                w.wake();
                            }
                            n
                        }
                    }
                };
                if self.recv.is_abandoned() {
                    ReadSamplesResult::Done(n)
                } else {
                    ReadSamplesResult::Waiting
                }
            }
        }
    }
}
pub struct TrackStreamHandle {
    send: Producer<i32>,
    waker: Producer<Waker>,
}
impl TrackStreamHandle {
    pub async fn send(&mut self, buf: &[i32]) {
        // wait until there are enough slots to send the data
        SendFut {
            send: &mut self.send,
            len: buf.len(),
            waker: &mut self.waker,
        }
        .await;

        // send data
        let mut w = self.send.write_chunk(buf.len()).unwrap();
        let (s1, s2) = w.as_mut_slices();
        s1.copy_from_slice(&buf[0..s1.len()]);
        s2.copy_from_slice(&buf[s1.len()..]);
        w.commit_all();
    }
}
pub struct SendFut<'a> {
    send: &'a mut Producer<i32>,
    len: usize,
    waker: &'a mut Producer<Waker>,
}
impl Future for SendFut<'_> {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let avail = self.send.slots();
        if avail >= self.len {
            Poll::Ready(())
        } else {
            self.waker.push(cx.waker().clone()).unwrap();
            Poll::Pending
        }
    }
}

trait Silence {
    fn silence() -> Self;
}

impl Silence for f32 {
    fn silence() -> Self {
        0.0
    }
}
impl Silence for f64 {
    fn silence() -> Self {
        0.0
    }
}
impl Silence for i8 {
    fn silence() -> Self {
        0
    }
}
impl Silence for i16 {
    fn silence() -> Self {
        0
    }
}
impl Silence for i32 {
    fn silence() -> Self {
        0
    }
}
impl Silence for i64 {
    fn silence() -> Self {
        0
    }
}
impl Silence for u8 {
    fn silence() -> Self {
        u8::MAX / 2
    }
}
impl Silence for u16 {
    fn silence() -> Self {
        u16::MAX / 2
    }
}
impl Silence for u32 {
    fn silence() -> Self {
        u32::MAX / 2
    }
}
impl Silence for u64 {
    fn silence() -> Self {
        u64::MAX / 2
    }
}
