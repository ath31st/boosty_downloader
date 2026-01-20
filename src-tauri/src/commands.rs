use std::sync::Arc;

use boosty_downloader_core::{AppConfig, log_error, log_info};
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

    boosty_downloader_core::sync_auth(
        state.client.as_ref().ok_or("Client not initialized")?,
        &state.config,
    )
    .await
    .map_err(|e| e.to_string())?;
    dbg!(&state.client);
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
    let config = boosty_downloader_core::load_config()
        .await
        .map_err(|e| e.to_string())?;

    let mut state = state.lock().await;
    state.client = Some(client);
    state.config = config;
    log_info!("Client initialized");
    Ok(())
}

#[tauri::command]
pub async fn download_content(
    url: String,
    offset_url: Option<String>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let state = state.lock().await;

    let client = &state.client.as_ref().ok_or("Client not initialized")?;
    let cfg = &state.config;

    let ctx =
        boosty_downloader_core::build_url_context(&url, offset_url.as_deref()).map_err(|e| {
            log_error!("{e}");
            e.to_string()
        })?;

    boosty_downloader_core::process_boosty_url(client, cfg, &ctx.url, ctx.offset)
        .await
        .map_err(|e| {
            log_error!("{e}");
            e.to_string()
        })?;

    Ok(())
}

#[tauri::command]
pub fn get_exe_path() -> Result<String, String> {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}
