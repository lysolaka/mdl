//! Logger implementation

use anstyle::{AnsiColor, Style};
use log::Level;
use once_cell::sync::OnceCell;

use pyo3::ffi::c_str;
use pyo3::prelude::*;

static LOGGER: OnceCell<Logger> = OnceCell::new();

struct Logger {
    level: log::LevelFilter,
}

/// Initialize a logger with the log level set to `log_level`.
///
/// # Panics
///
/// 1. if called more than once,
/// 2. if registering the logger with the [`log`] crate fails.
pub fn init(level: log::LevelFilter) {
    let logger = Logger { level };
    if let Err(_) = LOGGER.set(logger) {
        panic!("called logger::init() more than once");
    }

    if let Err(_) = log::set_logger(LOGGER.get().unwrap()).map(|()| log::set_max_level(level)) {
        panic!("failed to initialize the logger");
    }
}

const ERROR: Style = AnsiColor::BrightRed.on_default().bold();
const WARN: Style = AnsiColor::Yellow.on_default();
const INFO: Style = AnsiColor::Blue.on_default();
const DEBUG: Style = AnsiColor::White.on_default();

fn level_str(level: Level) -> String {
    match level {
        Level::Error => format!("{ERROR}{}{ERROR:#}", level.as_str()),
        Level::Warn => format!("{WARN}{}{WARN:#}", level.as_str()),
        Level::Info => format!("{INFO}{}{INFO:#}", level.as_str()),
        Level::Debug => format!("{DEBUG}{}{DEBUG:#}", level.as_str()),
        Level::Trace => format!("{}", level.as_str()),
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if record.level() <= self.level {
            eprintln!("[{}] {}", level_str(record.level()), record.args());
        }
    }

    fn flush(&self) {}
}

/// A Python class used to forward logs to the [`log`] crate.
///
/// This logger maps yt-dlp's info and debug down one step, i.e. info -> debug, debug -> trace.
#[pyclass]
pub struct MDLogger;

#[pymethods]
impl MDLogger {
    #[staticmethod]
    pub fn debug(msg: &str) -> PyResult<()> {
        if let Some(m) = msg.strip_prefix("[debug] ") {
            let m = strip_label(m);
            log::trace!("{}", m);
        } else {
            let m = strip_label(msg);
            log::debug!("{}", m);
        }
        Ok(())
    }

    #[staticmethod]
    pub fn warning(msg: &str) -> PyResult<()> {
        log::warn!("{}", msg);
        Ok(())
    }

    #[staticmethod]
    pub fn error(msg: &str) -> PyResult<()> {
        if let Some(m) = msg.strip_prefix("ERROR: ") {
            let m = strip_label(m);
            log::error!("{}", m);
        } else {
            log::error!("{}", msg);
        }
        Ok(())
    }
}

fn strip_label(input: &str) -> &str {
    if let Some(rest) = input.strip_prefix('[') {
        if let Some(idx) = rest.find(']') {
            return rest[idx + 1..].trim_start();
        }
    }
    input
}
