use once_cell::sync::OnceCell;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize, Debug, Clone)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Serialize, Debug, Clone)]
pub struct LogMessage<'a> {
    pub level: LogLevel,
    pub message: &'a str,
}

pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
}

static LOGGER: OnceCell<Arc<dyn Logger>> = OnceCell::new();

pub fn set_logger<L: Logger + 'static>(logger: L) -> Arc<dyn Logger> {
    let arc_logger: Arc<dyn Logger> = Arc::new(logger);
    let _ = LOGGER.set(arc_logger.clone());
    arc_logger
}

pub fn get_logger() -> Arc<dyn Logger> {
    LOGGER.get().expect("Logger not initialized").clone()
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        use $crate::{get_logger, LogLevel};
        get_logger().log(LogLevel::Info, &format!($($arg)*));
    }}
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{
        use $crate::{get_logger, LogLevel};
        get_logger().log(LogLevel::Warn, &format!($($arg)*));
    }}
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        use $crate::{get_logger, LogLevel};
        get_logger().log(LogLevel::Error, &format!($($arg)*));
    }}
}
