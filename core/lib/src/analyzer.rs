use crate::audio_clip::AudioClip;
use color_eyre::eyre::{eyre, Result};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot::{channel as oneshot_channel, Sender as OneshotSender};
use futures::SinkExt;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperError};

#[cfg(not(feature = "whisper_dummy"))]
const GGML_BASE_EN_Q5: &[u8] = include_bytes!("./ggml-base.en-q5_0.bin");

pub struct Analyzer {
    whisper_context: Option<WhisperContext>,
}

type Segment = ((f64, f64), String);

impl Analyzer {
    pub fn new() -> Result<Analyzer> {
        Ok(Analyzer {
            whisper_context: None,
        })
    }

    #[cfg(not(feature = "whisper_dummy"))]
    fn whisper_context(&mut self) -> Result<&mut WhisperContext> {
        let ctx = &mut self.whisper_context;
        if let Some(ctx) = ctx {
            Ok(ctx)
        } else {
            Ok(ctx.insert(WhisperContext::new_from_buffer(GGML_BASE_EN_Q5)?))
        }
    }

    /// Return a transcript of the audio using whisper.cpp
    #[cfg(not(feature = "whisper_dummy"))]
    pub fn transcribe(&mut self, clip: &AudioClip) -> Result<Vec<Segment>> {
        let mut state = self
            .whisper_context()?
            .create_state()
            .expect("failed to create state");

        // create a params object
        // note that currently the only implemented strategy is Greedy, BeamSearch is a WIP
        // n_past defaults to 0
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        params.set_n_threads(4);
        params.set_token_timestamps(true);
        params.set_language(Some("en"));
        params.set_suppress_blank(false);
        params.set_suppress_non_speech_tokens(true);
        params.set_print_progress(false);
        params.set_translate(false);

        // we must convert to 16KHz mono f32 samples for the model
        let resampled = clip.resample(16000);

        // Run it!
        state.full(params, &resampled.samples[..])?;

        // fetch the results
        let num_segments = state
            .full_n_segments()
            .expect("failed to get number of segments");

        let mut segments = Vec::with_capacity(num_segments as usize);
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i);
            let start_timestamp = state.full_get_segment_t0(i)?;
            let end_timestamp = state.full_get_segment_t1(i)?;
            let num_tokens = state.full_n_tokens(i)?;

            // whisper.cpp hallucinates. If this isn't seeming reliable, skip it.
            let mut total_prob = 0f32;
            for j in 0..num_tokens {
                total_prob += state.full_get_token_prob(i, j)?;
            }
            if total_prob / (num_tokens as f32) < 0.5 {
                continue;
            }

            match segment {
                Ok(segment) => {
                    segments.push((
                        (
                            (start_timestamp as f64) * 10f64 / 1000f64,
                            (end_timestamp as f64) * 10f64 / 1000f64,
                        ),
                        segment,
                    ));
                }
                Err(WhisperError::InvalidUtf8 { .. }) => {
                    // Whisper does not always give valid unicode... max_len=1 seems to
                    // split in invalid ways.
                    log::warn!("Whisper gave invalid utf8");
                }
                Err(err) => Err(err)?,
            }
        }

        Ok(segments)
    }

    #[cfg(feature = "whisper_dummy")]
    pub fn transcribe(&mut self, _clip: &AudioClip) -> Result<Vec<Segment>> {
        Ok(vec![])
    }
}

enum Event {
    Transcribe(AudioClip, OneshotSender<Result<Vec<Segment>>>),
}

async fn event_queue(mut analyzer: Analyzer, mut events: UnboundedReceiver<Event>) {
    while let Some(event) = events.next().await {
        let Event::Transcribe(clip, sender) = event;
        let result = analyzer.transcribe(&clip);
        sender.send(result).expect("failed to send result");
    }
}

pub struct AsyncAnalyzer {
    events: Arc<Mutex<UnboundedSender<Event>>>,
}

impl AsyncAnalyzer {
    pub fn new() -> Result<AsyncAnalyzer> {
        let (sender, receiver) = unbounded();
        tokio::spawn(async move {
            let analyzer = Analyzer::new().expect("failed to create analyzer");
            event_queue(analyzer, receiver).await
        });

        Ok(AsyncAnalyzer {
            events: Arc::new(Mutex::new(sender)),
        })
    }

    pub async fn transcribe(&self, clip: AudioClip) -> Result<Vec<Segment>> {
        let (sender, receiver) = oneshot_channel();

        let mut events = self.events.lock().await;
        events.send(Event::Transcribe(clip, sender)).await?;

        receiver
            .await
            .map_err(|_| eyre!("failed to receive result"))?
    }
}
