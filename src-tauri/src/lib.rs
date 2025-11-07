use std::sync::Arc;

use tokio::sync::Mutex;

use crate::state::AppState;

mod commands;
mod state;
mod tauri_logger;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(Mutex::new(AppState::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let logger = tauri_logger::TauriLogger::new(app.handle().clone());
            boosty_downloader_core::set_logger(logger);
            Ok(())
        })
        .manage(state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::update_config,
            commands::init_client,
            commands::download_content,
            commands::get_exe_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
