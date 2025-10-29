use std::any::Any;

use crate::{
    cli,
    logger::{LogLevel, Logger},
};

pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Info => cli::info(message),
            LogLevel::Warn => cli::warning(message),
            LogLevel::Error => cli::error(message),
        }
    }
    fn as_any(&self) -> &(dyn Any + 'static) {
        self
    }
}
