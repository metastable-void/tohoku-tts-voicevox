
use std::{collections::VecDeque, vec};

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool};
use parking_lot::Mutex;

use std::io::Read;

use hound::WavReader;
use rubato::{Resampler, SincFixedIn, SincInterpolationType, SincInterpolationParameters, WindowFunction};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample,
    Sample,
    StreamConfig,
};

pub(crate) fn format_sample<O: FromSample<i16> + Sample>(sample: i16) -> O {
    O::from_sample(sample)
}

#[derive(Debug, Clone)]
pub(crate) struct AudioPlayer {
    chunk_queue: Arc<Mutex<VecDeque<Vec<i16>>>>,
    sample_rate: u32,
    channel_count: u16,
    state: Arc<Mutex<Option<AudioPlayerState>>>,
    blocks_processed: Arc<AtomicUsize>,
    is_playing: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
struct AudioPlayerState {
    buffer: Vec<i16>,
    pos: usize,
}

#[allow(dead_code)]
impl AudioPlayer {
    pub fn new() -> Result<Self, anyhow::Error> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or_else(|| anyhow::anyhow!("No default output device"))?;
        let mut supported_configs_range = device.supported_output_configs()?;
        let supported_config = supported_configs_range.next().ok_or_else(|| anyhow::anyhow!("No supported audio config"))?.with_max_sample_rate();
        let sample_format = supported_config.sample_format();
        let config: StreamConfig = supported_config.into();

        let player = Self {
            chunk_queue: Arc::new(Mutex::new(VecDeque::new())),
            sample_rate: config.sample_rate.0,
            channel_count: config.channels,
            state: Arc::new(Mutex::new(None)),
            blocks_processed: Arc::new(AtomicUsize::new(0)),
            is_playing: Arc::new(AtomicBool::new(false)),
        };

        let err_fn = |err| log::error!("an error occurred on the output audio stream: {}", err);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_output_stream(&config, player.get_callback::<f32>(), err_fn, None)?,
            cpal::SampleFormat::F64 => device.build_output_stream(&config, player.get_callback::<f64>(), err_fn, None)?,
            cpal::SampleFormat::I8 => device.build_output_stream(&config, player.get_callback::<i8>(), err_fn, None)?,
            cpal::SampleFormat::U8 => device.build_output_stream(&config, player.get_callback::<u8>(), err_fn, None)?,
            cpal::SampleFormat::I16 => device.build_output_stream(&config, player.get_callback::<i16>(), err_fn, None)?,
            cpal::SampleFormat::U16 => device.build_output_stream(&config, player.get_callback::<u16>(), err_fn, None)?,
            cpal::SampleFormat::I32 => device.build_output_stream(&config, player.get_callback::<i32>(), err_fn, None)?,
            cpal::SampleFormat::U32 => device.build_output_stream(&config, player.get_callback::<u32>(), err_fn, None)?,
            cpal::SampleFormat::I64 => device.build_output_stream(&config, player.get_callback::<i64>(), err_fn, None)?,
            cpal::SampleFormat::U64 => device.build_output_stream(&config, player.get_callback::<u64>(), err_fn, None)?,
            _ => {
                return Err(anyhow::anyhow!("Unsupported sample format"));
            },
        };

        stream.play()?;

        let _leaked_stream = Box::leak(Box::new(stream));

        Ok(player)
    }

    fn get_callback<T>(&self) -> impl FnMut(&mut [T], &cpal::OutputCallbackInfo) + Send + 'static
    where
        T: FromSample<i16> + Sample,
    {
        let self_clone = self.clone();
        let callback = move |data: &mut [T], info: &cpal::OutputCallbackInfo| {
            self_clone.callback(data, info);
        };
        callback
    }

    fn callback<S: FromSample<i16> + Sample>(&self, buffer: &mut [S], _: &cpal::OutputCallbackInfo) {
        let mut state = self.state.lock();
        self.blocks_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if state.is_none() {
            let mut queue = self.chunk_queue.lock();
            let chunk = queue.pop_front();
            drop(queue);

            if let Some(chunk) = chunk {
                log::debug!("Playing chunk of {} samples", chunk.len());
                *state = Some(AudioPlayerState {
                    buffer: chunk,
                    pos: 0,
                });
            } else {
                self.is_playing.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }
        }

        self.is_playing.store(true, std::sync::atomic::Ordering::Relaxed);

        let ended = {
            let state_val = state.as_mut().unwrap();
            let buffer_len = buffer.len();
            let initial_pos = state_val.pos;
            let after_pos = initial_pos + buffer_len;

            let buf = &state_val.buffer[initial_pos..];
            for i in 0..buffer_len {
                if i >= buf.len() {
                    break;
                }
                buffer[i] = format_sample::<S>(buf[i]);
            }

            if after_pos < state_val.buffer.len() {
                state_val.pos = after_pos;
                false
            } else {
                true
            }
        };

        if ended {
            *state = None;
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channel_count(&self) -> u16 {
        self.channel_count
    }

    pub fn clear(&self) {
        self.chunk_queue.lock().clear();
    }

    pub fn play(&self, chunk: Vec<i16>) {
        if chunk.is_empty() {
            return;
        }
        self.chunk_queue.lock().push_back(chunk);
        self.is_playing.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn blocks_processed(&self) -> usize {
        self.blocks_processed.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn wait_blocking_until_empty(&self) {
        loop {
            if !self.is_playing.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    pub fn play_wav<R: Read>(&self, wav: R) -> Result<(), anyhow::Error> {
        let mut reader = WavReader::new(wav)?;
        let spec = reader.spec();

        let orig_sample_rate = spec.sample_rate;
        let orig_channel_count = spec.channels;
        let orig_sample_format = spec.sample_format;
        let orig_bit_depth = spec.bits_per_sample;
        let orig_sample_count = reader.len();
        if orig_sample_count == 0 {
            return Ok(());
        }

        let target_sample_rate = self.sample_rate;
        let resample_ratio = target_sample_rate as f64 / orig_sample_rate as f64;

        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        let input_chunk_size = if orig_sample_count < 1024 { orig_sample_count as usize } else { 1024 };

        let mut resampler = SincFixedIn::<f64>::new(
            resample_ratio,
            2.0,
            params,
            input_chunk_size,
            1,
        ).unwrap();

        let mut input_buffer = vec![vec![0.0f64; input_chunk_size]];

        let samples = match (orig_sample_format, orig_bit_depth) {
            (hound::SampleFormat::Int, 16) => {
                reader.samples::<i16>()
                    .filter(|sample| sample.is_ok())
                    .map(|sample| format_sample::<f64>(sample.unwrap())).collect::<Vec<_>>()
            },
            _ => {
                return Err(anyhow::anyhow!("Unsupported sample format: {:?} / {}", orig_sample_format, orig_bit_depth));
            }
        };

        let mono_samples = samples.chunks(orig_channel_count as usize).map(|chunk| {
            let sum = chunk.iter().fold(0.0, |acc, &sample| acc + sample);
            sum / orig_channel_count as f64
        }).collect::<Vec<_>>();

        let new_length = (orig_sample_count as f64 * resample_ratio) as usize;
        let output_delay = resampler.output_delay();
        let mut output_buffer = Vec::with_capacity(new_length + output_delay) as Vec<f64>;

        let mut output_frames = vec![vec![0.0f64; resampler.output_frames_max()]];
        let mut input_index = 0;
        loop {
            let frames = resampler.input_frames_next();
            let remaining = mono_samples.len() - input_index;
            if frames > remaining {
                break;
            }

            input_buffer[0].clear();
            for i in 0..frames {
                input_buffer[0].push(mono_samples[input_index + i]);
            }

            let (_, output_count) = resampler.process_into_buffer(&input_buffer, &mut output_frames, None)?;
            output_buffer.extend_from_slice(&output_frames[0][..output_count]);

            input_index += frames;
        }


        let frames = resampler.input_frames_next();
        let remaining = mono_samples.len() - input_index;
        if remaining > 0 {
            input_buffer[0].clear();
            for i in 0..remaining {
                input_buffer[0].push(mono_samples[input_index + i]);
            }

            input_buffer[0].resize(frames, 0.0);

            let (_, output_count) = resampler.process_into_buffer(&input_buffer, &mut output_frames, None)?;
            output_buffer.extend_from_slice(&output_frames[0][..output_count]);
        }

        while output_buffer.len() < new_length + output_delay {
            let (_, output_count) = resampler.process_partial_into_buffer(None::<&[Vec<f64>]>, &mut output_frames, None)?;
            output_buffer.extend_from_slice(&output_frames[0][..output_count]);
        }

        let output_buffer = output_buffer[output_delay..].into_iter().map(|sample| vec![<i16 as Sample>::from_sample(*sample); self.channel_count as usize]).flatten().collect::<Vec<_>>();
        
        if !output_buffer.is_empty() {
            self.play(output_buffer);
        }

        Ok(())
    }
}
