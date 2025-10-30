use std::any::Any;

use boosty_downloader_core::{LogLevel, LogMessage, Logger, ProgressMessage};
use tauri::{AppHandle, Emitter};

pub struct TauriLogger {
    app: AppHandle,
}

impl TauriLogger {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Logger for TauriLogger {
    fn log(&self, level: LogLevel, message: &str) {
        let msg = LogMessage { level, message };
        let _ = self.app.emit("log", msg);
    }

    fn progress(&self, msg: ProgressMessage) {
        let _ = self.app.emit("progress", msg);
    }

    fn as_any(&self) -> &(dyn Any + 'static) {
        self
    }
}
