use std::any::Any;

use crate::{
    cli,
    logger::{LogLevel, Logger},
    progress_reporter,
};

pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, level: LogLevel, message: &str) {
        progress_reporter::suspend_for(|| match level {
            LogLevel::Info => cli::info(message),
            LogLevel::Warn => cli::warning(message),
            LogLevel::Error => cli::error(message),
        });
    }
    fn as_any(&self) -> &(dyn Any + 'static) {
        self
    }
}
