/*

    TODO:
    - figure out if we can safely use stream.pause() everywhere we want

*/

use std::marker::PhantomData;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, FromSample, Sample, SizedSample, Stream, StreamConfig,
};
use rtrb::{Consumer, Producer, RingBuffer};

/// this is basically a specialized handle to the main audio thread that
/// understands the context of a streamed music player
pub struct MainStreamHandle {
    stream: Stream,
    queue: Producer<TrackStream>,
}

impl MainStreamHandle {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();

        let (send, recv) = RingBuffer::new(256);
        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => {
                build_main_stream::<i8>(&device, &config.into(), recv).unwrap()
            }
            cpal::SampleFormat::I16 => {
                build_main_stream::<i16>(&device, &config.into(), recv).unwrap()
            }
            cpal::SampleFormat::I32 => {
                build_main_stream::<i32>(&device, &config.into(), recv).unwrap()
            }
            cpal::SampleFormat::I64 => {
                build_main_stream::<i64>(&device, &config.into(), recv).unwrap()
            }
            cpal::SampleFormat::U8 => {
                build_main_stream::<u8>(&device, &config.into(), recv).unwrap()
            }
            cpal::SampleFormat::U16 => {
                build_main_stream::<u16>(&device, &config.into(), recv).unwrap()
            }
            cpal::SampleFormat::U32 => {
                build_main_stream::<u32>(&device, &config.into(), recv).unwrap()
            }
            cpal::SampleFormat::U64 => {
                build_main_stream::<u64>(&device, &config.into(), recv).unwrap()
            }
            cpal::SampleFormat::F32 => {
                build_main_stream::<f32>(&device, &config.into(), recv).unwrap()
            }
            cpal::SampleFormat::F64 => {
                build_main_stream::<f64>(&device, &config.into(), recv).unwrap()
            }
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        };

        Self {
            stream,
            queue: send,
        }
    }

    pub fn play(&self) {
        self.stream.play().unwrap();
    }

    pub fn pause(&self) {
        self.stream.pause().unwrap();
    }
}

struct MainStream<S>
where
    S: Sample + Silence,
{
    _phantom: PhantomData<S>,
    current_track: Option<TrackStream>,
    queue: Consumer<TrackStream>,
}
impl<S> MainStream<S>
where
    S: Sample + Silence,
{
    pub fn new(queue: Consumer<TrackStream>) -> Self {
        Self {
            queue,
            current_track: None,
            _phantom: PhantomData,
        }
    }

    pub fn cb(&mut self, buf: &mut [S]) {
        buf.fill(S::silence());
        match &self.current_track {
            Some(t) => {}
            None => {}
        }
        todo!()
    }
}

fn build_main_stream<S>(
    device: &Device,
    config: &StreamConfig,
    recv: Consumer<TrackStream>,
) -> Result<Stream, ()>
where
    S: SizedSample + FromSample<i32> + Silence + Send + 'static,
{
    let mut ms = MainStream::new(recv);
    let stream = device
        .build_output_stream(config, move |buf: &mut [S], _| ms.cb(buf), |_| {}, None)
        .unwrap();
    stream.pause().unwrap();
    Ok(stream)
}

struct TrackStream {}

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
