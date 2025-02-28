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
    playing: Arc<AtomicBool>,
    queue: Arc<Mutex<Producer<TrackStream>>>,
    clear: Arc<AtomicBool>,
}

pub fn init_main_stream() -> (Stream, MainStreamHandle) {
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

    (
        stream,
        MainStreamHandle::new(clear, playing, Arc::new(Mutex::new(queue))),
    )
}

impl MainStreamHandle {
    pub fn new(
        clear: Arc<AtomicBool>,
        playing: Arc<AtomicBool>,
        queue: Arc<Mutex<Producer<TrackStream>>>,
    ) -> Self {
        Self {
            clear,
            playing,
            queue,
        }
    }
    pub fn toggle_playing(&self) -> bool {
        !self.playing.fetch_not(Ordering::AcqRel)
    }
    pub fn clear(&self) {
        self.clear.store(true, Ordering::Release);
    }
    pub fn queue(&self, track: TrackStream) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(track).unwrap();
    }
    pub fn pause(&self) {
        self.playing.store(false, Ordering::Release);
    }
    pub fn play(&self) {
        self.playing.store(true, Ordering::Release);
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
            self.current_track = None;
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
    stream.play().unwrap();
    Ok(stream)
}

enum ReadSamplesResult {
    Ok,
    Done(usize),
    Waiting,
}

pub fn spawn_track_stream() -> (TrackStream, TrackStreamHandle) {
    let (sample_send, sample_recv) = RingBuffer::new(1024);
    let (wake_send, wake_rec) = RingBuffer::new(1);
    (
        TrackStream::new(sample_recv, wake_rec),
        TrackStreamHandle::new(sample_send, wake_send),
    )
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
    pub fn new(send: Producer<i32>, waker: Producer<Waker>) -> Self {
        Self { send, waker }
    }
    pub async fn send(&mut self, buf: &[i32]) {
        let mut sent = 0;
        while sent < buf.len() {
            // wait until there are some slots
            let slots = SendFut {
                send: &mut self.send,
                waker: &mut self.waker,
            }
            .await;

            // send data
            let to_send = &buf[sent..sent + slots];
            let mut w = self.send.write_chunk(slots).unwrap();
            let (s1, s2) = w.as_mut_slices();
            s1.copy_from_slice(&to_send[0..s1.len()]);
            s2.copy_from_slice(&to_send[s1.len()..]);
            w.commit_all();
            sent += slots;
        }
    }
}
pub struct SendFut<'a> {
    send: &'a mut Producer<i32>,
    waker: &'a mut Producer<Waker>,
}
impl Future for SendFut<'_> {
    type Output = usize;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.send.slots() {
            0 => {
                self.waker.push(cx.waker().clone()).unwrap();
                Poll::Pending
            }
            n => Poll::Ready(n),
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
