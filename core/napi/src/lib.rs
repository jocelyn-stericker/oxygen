use chrono::prelude::*;
use log::{Log, Metadata, Record};
use napi::{
    bindgen_prelude::Buffer,
    threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    Env, Error, JsDate, JsFunction, JsUnknown, Result,
};
use napi_derive::napi;
use oxygen_core::audio_clip::{AudioClip, PlayHandle, RecordHandle, StreamHandle};
use oxygen_core::db::{ClipMeta, Db};

enum Tab {
    Record {
        handle: Option<RecordHandle>,
    },
    Clip {
        audio_clip: AudioClip,
        handle: Option<PlayHandle>,
    },
}

#[napi]
pub struct UiState {
    tab: Tab,
    db: Db,
    deleted_clip: Option<AudioClip>,
    update_cb: ThreadsafeFunction<(), ErrorStrategy::Fatal>,
}

#[napi]
pub struct JsClipMeta(ClipMeta);

#[napi]
impl JsClipMeta {
    #[napi(getter)]
    pub fn get_id(&self) -> usize {
        self.0.id
    }

    #[napi(getter)]
    pub fn get_name(&self) -> &str {
        &self.0.name
    }

    #[napi(getter, ts_return_type = "Date")]
    pub fn get_date(&self, env: Env) -> Result<JsDate> {
        env.create_date(self.0.date.timestamp_millis() as f64)
    }
}

impl From<ClipMeta> for JsClipMeta {
    fn from(clip_meta: ClipMeta) -> Self {
        JsClipMeta(clip_meta)
    }
}

impl From<&AudioClip> for JsClipMeta {
    fn from(clip: &AudioClip) -> Self {
        JsClipMeta(ClipMeta {
            id: clip.id.unwrap_or(0),
            name: clip.name.clone(),
            date: clip.date,
        })
    }
}

struct JsLogger(ThreadsafeFunction<String, ErrorStrategy::Fatal>);

impl Log for JsLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        self.0.call(
            std::format!("{}", record.args()),
            ThreadsafeFunctionCallMode::NonBlocking,
        );
    }

    fn flush(&self) {}
}

#[napi]
impl UiState {
    #[napi(constructor)]
    pub fn new(update_cb: JsFunction, log_cb: JsFunction, in_memory: bool) -> Result<UiState> {
        let logger = Box::new(JsLogger(log_cb.create_threadsafe_function(0, |ctx| {
            ctx.env.create_string_from_std(ctx.value).map(|v| vec![v])
        })?));
        let logger = Box::leak(logger);
        log::set_logger(logger).map_err(|e| Error::from_reason(e.to_string()))?;
        log::set_max_level(log::LevelFilter::Trace);

        Ok(UiState {
            tab: Tab::Record { handle: None },
            db: if in_memory {
                Db::in_memory()
            } else {
                Db::open()
            }
            .map_err(|e| Error::from_reason(e.to_string()))?,
            deleted_clip: None,
            update_cb: update_cb
                .create_threadsafe_function(0, |_ctx| Ok(vec![] as Vec<JsUnknown>))?,
        })
    }

    #[napi]
    pub fn get_clips(&self) -> Result<Vec<JsClipMeta>> {
        self.db
            .list()
            .map_err(|e| Error::from_reason(e.to_string()))
            .map(|clips| clips.into_iter().map(JsClipMeta::from).collect())
    }

    #[napi(getter)]
    pub fn get_current_clip_id(&self) -> Option<usize> {
        match &self.tab {
            Tab::Record { .. } => None,
            Tab::Clip { audio_clip, .. } => Some(audio_clip.id.expect("Saved clips must have IDs")),
        }
    }

    #[napi(getter)]
    pub fn get_current_clip(&self) -> Option<JsClipMeta> {
        match &self.tab {
            Tab::Record { .. } => None,
            Tab::Clip { audio_clip, .. } => Some(JsClipMeta::from(audio_clip)),
        }
    }

    #[napi(getter)]
    pub fn get_record_tab_selected(&self) -> bool {
        matches!(&self.tab, Tab::Record { .. })
    }

    #[napi]
    pub fn set_current_clip_id(&mut self, id: u32) -> Result<()> {
        if let Some(audio_clip) = self
            .db
            .load_by_id(id as usize)
            .map_err(|e| Error::from_reason(e.to_string()))?
        {
            self.tab = Tab::Clip {
                audio_clip,
                handle: None,
            };
            self.update_cb
                .call((), ThreadsafeFunctionCallMode::NonBlocking);
        }
        Ok(())
    }

    #[napi]
    pub fn set_current_tab_record(&mut self) {
        self.tab = Tab::Record { handle: None };
        self.update_cb
            .call((), ThreadsafeFunctionCallMode::NonBlocking);
    }

    #[napi]
    pub fn play(&mut self, on_done: JsFunction) -> Result<()> {
        if let Tab::Clip { audio_clip, handle } = &mut self.tab {
            let new_handle = audio_clip
                .play()
                .map_err(|e| Error::from_reason(e.to_string()))?;

            let on_done: ThreadsafeFunction<(), ErrorStrategy::Fatal> =
                on_done.create_threadsafe_function(0, |_ctx| Ok(vec![] as Vec<JsUnknown>))?;
            new_handle.connect_done(move || {
                on_done.call((), ThreadsafeFunctionCallMode::NonBlocking);
            });

            let update_cb = self.update_cb.clone();
            new_handle.connect_changed(move || {
                update_cb.call((), ThreadsafeFunctionCallMode::NonBlocking);
            });

            *handle = Some(new_handle);

            self.update_cb
                .call((), ThreadsafeFunctionCallMode::NonBlocking);
        }

        Ok(())
    }

    #[napi]
    pub fn record(&mut self) -> Result<()> {
        if let Tab::Record { handle } = &mut self.tab {
            let name = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let new_handle =
                AudioClip::record(name).map_err(|e| Error::from_reason(e.to_string()))?;

            *handle = Some(new_handle);

            self.update_cb
                .call((), ThreadsafeFunctionCallMode::NonBlocking);
        }

        Ok(())
    }

    #[napi]
    pub fn stop(&mut self) -> Result<()> {
        match &mut self.tab {
            Tab::Record { handle } => {
                if let Some(handle) = handle.take() {
                    let mut audio_clip = handle.stop();
                    self.db
                        .save(&mut audio_clip)
                        .map_err(|e| Error::from_reason(e.to_string()))?;

                    self.tab = Tab::Clip {
                        audio_clip,
                        handle: None,
                    };
                }
            }
            Tab::Clip { handle, .. } => {
                *handle = None;
            }
        }

        self.update_cb
            .call((), ThreadsafeFunctionCallMode::NonBlocking);

        Ok(())
    }

    #[napi]
    pub fn delete_current_clip(&mut self) -> Result<()> {
        let mut tab = Tab::Record { handle: None };
        std::mem::swap(&mut tab, &mut self.tab);

        self.update_cb
            .call((), ThreadsafeFunctionCallMode::NonBlocking);

        if let Tab::Clip { mut audio_clip, .. } = tab {
            if let Some(id) = audio_clip.id {
                self.db
                    .delete_by_id(id)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                audio_clip.id = None;
                self.deleted_clip = Some(audio_clip);
            } else {
                return Err(Error::from_reason("Clip is not saved to db"));
            }
        } else {
            return Err(Error::from_reason("No clip selected"));
        }

        Ok(())
    }

    #[napi]
    pub fn undelete_current_clip(&mut self) -> Result<()> {
        if let Some(mut audio_clip) = self.deleted_clip.take() {
            self.db
                .save(&mut audio_clip)
                .map_err(|e| Error::from_reason(e.to_string()))?;

            self.tab = Tab::Clip {
                audio_clip,
                handle: None,
            };

            self.update_cb
                .call((), ThreadsafeFunctionCallMode::NonBlocking);
        } else {
            return Err(Error::from_reason("No clip to undelete"));
        }

        Ok(())
    }

    #[napi]
    pub fn rename_current_clip(&mut self, new_name: String) -> Result<()> {
        let clip_id;

        if let Tab::Clip {
            audio_clip: AudioClip { id: Some(id), .. },
            ..
        } = &mut self.tab
        {
            clip_id = *id;

            self.db
                .rename_by_id(*id, &new_name)
                .map_err(|e| Error::from_reason(e.to_string()))?;
        } else {
            return Err(Error::from_reason("No clip selected"));
        }

        self.set_current_clip_id(clip_id as u32)?;
        self.update_cb
            .call((), ThreadsafeFunctionCallMode::NonBlocking);

        Ok(())
    }

    #[napi]
    pub fn draw_current_clip_waveform(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Option<Buffer>> {
        let width = width as usize;
        let height = height as usize;

        let clip = match &self.tab {
            Tab::Record { .. } => {
                return Ok(None);
            }
            Tab::Clip { audio_clip, .. } => audio_clip,
        };

        let columns = clip.render_waveform((0, clip.num_samples()), width);
        let mut buffer = vec![0; width * height * 4];

        for (x, column) in columns.iter().enumerate() {
            let min_y = ((height as f32) * (column.min + 1.0) / 2.0)
                .floor()
                .max(0.0) as usize;
            let max_y =
                (((height as f32) * (column.max + 1.0) / 2.0).ceil() as usize).min(height - 1);

            for y in min_y..=max_y {
                // purple-900 :)
                buffer[y * width * 4 + x * 4] = 88;
                buffer[y * width * 4 + x * 4 + 1] = 28;
                buffer[y * width * 4 + x * 4 + 2] = 135;
                buffer[y * width * 4 + x * 4 + 3] = 255;
            }
        }

        Ok(Some(buffer.into()))
    }

    #[napi(getter)]
    pub fn get_streaming(&self) -> bool {
        match &self.tab {
            Tab::Record { handle } => handle.is_some(),
            Tab::Clip { handle, .. } => handle.is_some(),
        }
    }

    fn stream_handle(&self) -> Option<&dyn StreamHandle> {
        match &self.tab {
            Tab::Record {
                handle: Some(handle),
            } => Some(handle),
            Tab::Clip {
                handle: Some(handle),
                ..
            } => Some(handle),
            Tab::Record { handle: None } | Tab::Clip { handle: None, .. } => None,
        }
    }

    #[napi(getter)]
    pub fn get_time(&self) -> f64 {
        if let Some(handle) = self.stream_handle() {
            handle.time()
        } else {
            0.0
        }
    }

    #[napi(getter)]
    pub fn get_time_percent(&self) -> f64 {
        if let Some(handle) = self.stream_handle() {
            (handle.sample_rate() as f64) * handle.time() / (handle.samples() as f64)
        } else {
            0.0
        }
    }
}
