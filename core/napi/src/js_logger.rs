use lazy_static::lazy_static;
use log::{Level, Log, Metadata, Record};
use napi::{
    threadsafe_function::{
        ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
    },
    Error, JsFunction, JsString, Result,
};
use std::sync::Mutex;

pub struct JsLogger(Mutex<Option<ThreadsafeFunction<(String, String), ErrorStrategy::Fatal>>>);

lazy_static! {
    static ref LOGGER: JsLogger = JsLogger(Mutex::new(None));
}

impl Log for JsLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        let logger = self.0.lock().unwrap();
        if let Some(logger) = &*logger {
            if !self.enabled(record.metadata()) {
                return;
            }

            let level = match record.level() {
                Level::Error => "error",
                Level::Trace => "trace",
                Level::Warn => "warn",
                Level::Info => "info",
                Level::Debug => "debug",
            }
            .to_owned();

            logger.call(
                (level, std::format!("{}", record.args())),
                ThreadsafeFunctionCallMode::NonBlocking,
            );
        }
    }

    fn flush(&self) {}
}

impl JsLogger {
    pub fn set_logger(log_cb: JsFunction) -> Result<()> {
        let mut logger = LOGGER.0.lock().unwrap();
        let should_init = logger.is_none();
        *logger = Some(log_cb.create_threadsafe_function(
            0,
            |ctx: ThreadSafeCallContext<(String, String)>| {
                Ok(vec![
                    ctx.env.create_string_from_std(ctx.value.0)?,
                    ctx.env.create_string_from_std(ctx.value.1)?,
                ]) as Result<Vec<JsString>>
            },
        )?);

        if should_init {
            let logger: &'static JsLogger = &LOGGER;
            log::set_logger(logger).map_err(|e| Error::from_reason(format!("{:?}", e)))?;
            log::set_max_level(log::LevelFilter::Trace);
        }

        Ok(())
    }
}
