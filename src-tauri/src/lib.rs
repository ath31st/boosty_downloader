use std::sync::Arc;

//use tauri::Manager;
use tokio::sync::Mutex;

use crate::state::AppState;

mod commands;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(Mutex::new(AppState::default()));

    tauri::Builder::default()
        .manage(state)
        // .setup(|app| {
        //     let app_handle = app.handle();
        //     let state = app_handle.state::<Arc<Mutex<AppState>>>().inner().clone();
        //     tauri::async_runtime::spawn(async move {
        //         let client = boosty_downloader_core::make_client()
        //             .await
        //             .expect("Failed to create client");
        //         boosty_downloader_core::init_client(&client)
        //             .await
        //             .expect("Failed to init client");
        //         let mut state = state.lock().await;
        //         state.client = Some(client);
        //     });
        //     Ok(())
        // })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::update_config,
            commands::init_client
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
