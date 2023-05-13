use chrono::prelude::*;
use color_eyre::eyre::{eyre, Result, WrapErr};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Host, HostUnavailable, Sample, Stream};
use dasp::{interpolate::linear::Linear, signal, Signal};
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct RecordState {
    clip: AudioClip,
}

pub struct RecordHandle {
    stream: Stream,
    /// Option is only taken in "stop".
    clip: Arc<Mutex<Option<RecordState>>>,
}

impl RecordHandle {
    pub fn stop(self) -> AudioClip {
        drop(self.stream);
        let clip = self.clip.lock().unwrap().take().unwrap().clip;
        log::info!("Recorded clip has {} samples", clip.samples.len());
        clip
    }
}

type RecordStateHandle = Arc<Mutex<Option<RecordState>>>;

struct PlaybackState {
    time: usize,
    samples: Vec<f32>,
    changed_cbs: Vec<Box<dyn Fn() + Send>>,
    changed_cbs_triggered_at: usize,
    done_cbs: Vec<Box<dyn Fn() + Send>>,
    sample_rate: usize,
}

type PlaybackStateHandle = Arc<Mutex<Option<PlaybackState>>>;

pub struct PlayHandle {
    _stream: Stream,
    state: PlaybackStateHandle,
}

impl PlayHandle {
    pub fn connect_changed<F: Fn() + 'static + Send>(&self, f: F) {
        let mut state = self.state.lock().unwrap();
        let state = state.as_mut().unwrap();
        state.changed_cbs.push(Box::new(f));
    }

    pub fn connect_done<F: Fn() + 'static + Send>(&self, f: F) {
        let mut state = self.state.lock().unwrap();
        let state = state.as_mut().unwrap();

        if state.time >= state.samples.len() {
            f();
        } else {
            state.done_cbs.push(Box::new(f));
        }
    }

    pub fn seek(&self, time_percent: f64) {
        let mut state = self.state.lock().unwrap();
        let state = state.as_mut().unwrap();
        state.time = (time_percent * (state.samples.len() as f64)) as usize;
        state.changed_cbs_triggered_at = 0;
    }
}

pub trait StreamHandle {
    fn sample_rate(&self) -> u32;
    fn samples(&self) -> usize;
    fn time(&self) -> f64;
}

impl StreamHandle for RecordHandle {
    fn sample_rate(&self) -> u32 {
        let mut state = self.clip.lock().unwrap();
        let state = state.as_mut().unwrap();

        state.clip.sample_rate
    }

    fn samples(&self) -> usize {
        let mut state = self.clip.lock().unwrap();
        let state = state.as_mut().unwrap();

        state.clip.samples.len()
    }

    fn time(&self) -> f64 {
        let mut state = self.clip.lock().unwrap();
        let state = state.as_mut().unwrap();

        (state.clip.samples.len() as f64) / (state.clip.sample_rate as f64)
    }
}

impl StreamHandle for PlayHandle {
    fn sample_rate(&self) -> u32 {
        let mut state = self.state.lock().unwrap();
        let state = state.as_mut().unwrap();

        state.sample_rate as u32
    }

    fn samples(&self) -> usize {
        let mut state = self.state.lock().unwrap();
        let state = state.as_mut().unwrap();

        state.samples.len()
    }

    fn time(&self) -> f64 {
        let mut state = self.state.lock().unwrap();
        let state = state.as_mut().unwrap();

        (state.time as f64) / (state.sample_rate as f64)
    }
}

pub trait ClipHandle {
    fn render_waveform(&self, range: (usize, usize), pixels: usize) -> Vec<DisplayColumn>;
    fn num_samples(&self) -> usize;
}

impl ClipHandle for RecordHandle {
    fn render_waveform(&self, range: (usize, usize), pixels: usize) -> Vec<DisplayColumn> {
        let mut state = self.clip.lock().unwrap();
        let state = state.as_mut().unwrap();

        state.clip.render_waveform(range, pixels)
    }

    fn num_samples(&self) -> usize {
        let mut state = self.clip.lock().unwrap();
        let state = state.as_mut().unwrap();

        state.clip.samples.len()
    }
}

impl ClipHandle for AudioClip {
    fn render_waveform(&self, range: (usize, usize), pixels: usize) -> Vec<DisplayColumn> {
        self.render_waveform(range, pixels)
    }

    fn num_samples(&self) -> usize {
        self.samples.len()
    }
}

/// Raw mono audio data.
#[derive(Clone)]
pub struct AudioClip {
    pub id: Option<usize>,
    pub name: String,
    pub date: DateTime<Utc>,
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

pub struct DisplayColumn {
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Copy, Default)]
pub enum AudioBackend {
    #[default]
    Default,
    #[cfg(feature = "jack")]
    Jack,
}

impl AudioBackend {
    fn host(&self) -> Result<Host, HostUnavailable> {
        match self {
            AudioBackend::Default => Ok(cpal::default_host()),

            #[cfg(feature = "jack")]
            AudioBackend::Jack => cpal::host_from_id(cpal::HostId::Jack),
        }
    }
}

impl AudioClip {
    pub fn resample(&self, sample_rate: u32) -> AudioClip {
        if self.sample_rate == sample_rate {
            return self.clone();
        }

        let mut signal = signal::from_iter(self.samples.iter().copied());
        let a = signal.next();
        let b = signal.next();

        let linear = Linear::new(a, b);

        AudioClip {
            id: self.id,
            name: self.name.clone(),
            date: self.date,
            samples: signal
                .from_hz_to_hz(linear, self.sample_rate as f64, sample_rate as f64)
                .take(self.samples.len() * (sample_rate as usize) / (self.sample_rate as usize))
                .collect(),
            sample_rate,
        }
    }

    pub fn record(host: AudioBackend, name: String) -> Result<RecordHandle> {
        let host = host.host().wrap_err("Could not open specified host")?;
        let device = host
            .default_input_device()
            .ok_or_else(|| eyre!("No input device"))?;
        log::info!("Input device: {}", device.name()?);
        let config = device.default_input_config()?;

        let clip = AudioClip {
            id: None,
            name,
            date: Utc::now(),
            samples: Vec::new(),
            sample_rate: config.sample_rate().0,
        };
        let clip = Arc::new(Mutex::new(Some(RecordState { clip })));
        let clip_2 = clip.clone();

        log::info!("Begin recording...");
        let err_fn = move |err| {
            log::error!("an error occurred on stream: {}", err);
        };

        let channels = config.channels();

        fn write_input_data<T>(input: &[T], channels: u16, writer: &RecordStateHandle)
        where
            T: cpal::Sample,
            f32: cpal::FromSample<T>,
        {
            if let Ok(mut guard) = writer.try_lock() {
                if let Some(state) = guard.as_mut() {
                    for frame in input.chunks(channels.into()) {
                        state.clip.samples.push(f32::from_sample(frame[0]));
                    }
                }
            }
        }

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<f32>(data, channels, &clip_2),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i16>(data, channels, &clip_2),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<u16>(data, channels, &clip_2),
                err_fn,
                None,
            )?,
            format => {
                return Err(eyre!("Unknown sample format {:?}.", format));
            }
        };

        stream.play()?;

        Ok(RecordHandle { stream, clip })
    }

    pub fn import(name: String, path: String) -> Result<AudioClip> {
        // Create a media source. Note that the MediaSource trait is automatically implemented for File,
        // among other types.
        let file = Box::new(File::open(Path::new(&path))?);

        let creation_time = file.metadata()?.created()?;

        // Create the media source stream using the boxed media source from above.
        let mss = MediaSourceStream::new(file, Default::default());

        // Create a hint to help the format registry guess what format reader is appropriate. In this
        // example we'll leave it empty.
        let hint = Hint::new();

        // Use the default options when reading and decoding.
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        // Probe the media source stream for a format.
        let probed =
            symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

        // Get the format reader yielded by the probe operation.
        let mut format = probed.format;

        // Get the default track.
        let track = format
            .default_track()
            .ok_or_else(|| eyre!("No default track"))?;

        // Create a decoder for the track.
        let mut decoder =
            symphonia::default::get_codecs().make(&track.codec_params, &decoder_opts)?;

        // Store the track identifier, we'll use it to filter packets.
        let track_id = track.id;

        let mut sample_count = 0;
        let mut sample_buf = None;
        let channels = track
            .codec_params
            .channels
            .ok_or_else(|| eyre!("Unknown channel count"))?;

        let mut clip = AudioClip {
            id: None,
            name,
            date: DateTime::<Utc>::from(creation_time),
            samples: Vec::new(),
            sample_rate: track
                .codec_params
                .sample_rate
                .ok_or_else(|| eyre!("Unknown sample rate"))?,
        };

        loop {
            // Get the next packet from the format reader.
            let packet = match format.next_packet() {
                Ok(packet_ok) => packet_ok,
                Err(Error::IoError(ref packet_err))
                    if packet_err.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    break;
                }
                Err(packet_err) => {
                    return Err(packet_err.into());
                }
            };

            // If the packet does not belong to the selected track, skip it.
            if packet.track_id() != track_id {
                continue;
            }

            // Decode the packet into audio samples, ignoring any decode errors.
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    // The decoded audio samples may now be accessed via the audio buffer if per-channel
                    // slices of samples in their native decoded format is desired. Use-cases where
                    // the samples need to be accessed in an interleaved order or converted into
                    // another sample format, or a byte buffer is required, are covered by copying the
                    // audio buffer into a sample buffer or raw sample buffer, respectively. In the
                    // example below, we will copy the audio buffer into a sample buffer in an
                    // interleaved order while also converting to a f32 sample format.

                    // If this is the *first* decoded packet, create a sample buffer matching the
                    // decoded audio buffer format.
                    if sample_buf.is_none() {
                        // Get the audio buffer specification.
                        let spec = *audio_buf.spec();

                        // Get the capacity of the decoded buffer. Note: This is capacity, not length!
                        let duration = audio_buf.capacity() as u64;

                        // Create the f32 sample buffer.
                        sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                    }

                    // Copy the decoded audio buffer into the sample buffer in an interleaved format.
                    if let Some(buf) = &mut sample_buf {
                        buf.copy_interleaved_ref(audio_buf);
                        let mono: Vec<f32> = buf
                            .samples()
                            .iter()
                            .step_by(channels.count())
                            .copied()
                            .collect();
                        clip.samples.extend_from_slice(&mono);

                        // The samples may now be access via the `samples()` function.
                        sample_count += buf.samples().len();
                        log::info!("\rDecoded {} samples", sample_count);
                    }
                }
                Err(Error::DecodeError(_)) => (),
                Err(_) => break,
            }
        }

        Ok(clip)
    }

    pub fn play(&self, host: AudioBackend) -> Result<PlayHandle> {
        let host = host.host().wrap_err("Could not open specified host")?;
        let device = host
            .default_output_device()
            .ok_or_else(|| eyre!("No output device"))?;
        log::info!("Output device: {}", device.name()?);
        let config = device.default_output_config()?;

        log::info!("Begin playback...");

        let sample_rate = config.sample_rate().0;
        let state = PlaybackState {
            time: 0,
            samples: self.resample(sample_rate).samples,
            done_cbs: vec![],
            changed_cbs: vec![],
            changed_cbs_triggered_at: 0,
            sample_rate: sample_rate as usize,
        };
        let state: PlaybackStateHandle = Arc::new(Mutex::new(Some(state)));
        let state_2 = state.clone();
        let channels = config.channels();

        let err_fn = move |err| {
            log::error!("an error occurred on stream: {}", err);
        };

        fn write_output_data<T>(output: &mut [T], channels: u16, writer: &PlaybackStateHandle)
        where
            T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>,
        {
            if let Ok(mut guard) = writer.try_lock() {
                if let Some(state) = guard.as_mut() {
                    for frame in output.chunks_mut(channels.into()) {
                        for sample in frame.iter_mut() {
                            *sample =
                                T::from_sample(*state.samples.get(state.time).unwrap_or(&0f32));
                        }
                        state.time += 1;
                    }
                    if state.time >= state.samples.len() {
                        for cb in &*state.done_cbs {
                            cb();
                        }
                    }
                    if state.time >= state.changed_cbs_triggered_at + state.sample_rate / 100 {
                        for cb in &*state.changed_cbs {
                            cb();
                        }
                        state.changed_cbs_triggered_at = state.time;
                    }
                }
            }
        }

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |data, _: &_| write_output_data::<f32>(data, channels, &state),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_output_stream(
                &config.into(),
                move |data, _: &_| write_output_data::<i16>(data, channels, &state),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_output_stream(
                &config.into(),
                move |data, _: &_| write_output_data::<u16>(data, channels, &state),
                err_fn,
                None,
            )?,
            format => {
                return Err(eyre!("Unknown sample format {:?}.", format));
            }
        };

        stream.play()?;

        Ok(PlayHandle {
            _stream: stream,
            state: state_2,
        })
    }

    pub fn export(&self, path: &str) -> Result<()> {
        if !path.ends_with(".wav") {
            return Err(eyre!("Expected {} to end in .wav", path));
        }

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;
        for sample in &self.samples {
            writer.write_sample(*sample)?;
        }

        writer.finalize()?;

        Ok(())
    }

    pub fn render_waveform(&self, range: (usize, usize), pixels: usize) -> Vec<DisplayColumn> {
        let min_t = range.0.min(self.samples.len()) as f32;
        let max_t = (range.1.min(self.samples.len()) as f32).max(min_t);
        let samples_per_pixel = (max_t - min_t) / (pixels as f32);

        (0..pixels)
            .map(|pixel_i| {
                let mut min = 1.0f32;
                let mut max = -1.0f32;

                let start_sample = (min_t + samples_per_pixel * (pixel_i as f32)).floor() as usize;
                let end_sample = ((min_t + samples_per_pixel * ((pixel_i + 1) as f32)).floor()
                    as usize)
                    .min(self.samples.len());

                for sample in &self.samples[start_sample..end_sample] {
                    min = min.min(*sample);
                    max = max.max(*sample);
                }

                if min > max {
                    min = 0.0;
                    max = 0.0;
                }
                if min < -1.0 {
                    min = -1.0;
                }
                if max > 1.0 {
                    max = 1.0;
                }

                DisplayColumn { min, max }
            })
            .collect()
    }

    pub fn num_samples(&self) -> usize {
        self.samples.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_with_zero_samples() {
        let clip = AudioClip {
            id: Some(1),
            name: "Name".into(),
            date: Utc::now(),
            samples: vec![],
            sample_rate: 44100,
        };
        assert_eq!(clip.render_waveform((0, 0), 100).len(), 100);
        assert_eq!(clip.render_waveform((0, 0), 0).len(), 0);
        assert_eq!(clip.render_waveform((100, 0), 0).len(), 0);
        assert_eq!(clip.render_waveform((100, 200), 100).len(), 100);
    }
}
