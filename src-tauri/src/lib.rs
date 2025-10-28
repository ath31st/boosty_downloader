use std::sync::Arc;

use tokio::sync::Mutex;

use crate::state::AppState;

mod commands;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(Mutex::new(AppState::default()));

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::update_config,
            commands::init_client
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
