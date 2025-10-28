use std::sync::Arc;

use boosty_downloader_core::AppConfig;
use tauri::State;
use tokio::sync::Mutex;

use crate::state::AppState;

#[tauri::command]
pub async fn get_config(state: State<'_, Arc<Mutex<AppState>>>) -> Result<AppConfig, String> {
    let state = state.lock().await;
    Ok(state.config.clone())
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, Arc<Mutex<AppState>>>,
    new_config: AppConfig,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.config = new_config;
    boosty_downloader_core::save_config(&state.config)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn init_client(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let client = boosty_downloader_core::make_client()
        .await
        .map_err(|e| e.to_string())?;
    boosty_downloader_core::init_client(&client)
        .await
        .map_err(|e| e.to_string())?;

    let mut state = state.lock().await;
    state.client = Some(client);
    Ok(())
}
