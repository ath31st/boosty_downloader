use std::sync::Arc;

use boosty_api::api_client::ApiClient;
use boosty_downloader_core::{
    AppConfig, BlogSnapshot, DownloadOptions, DownloadPostsResult, log_error, log_info,
    scan_downloaded,
};
use tauri::State;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::state::AppState;

async fn take_work_slot(
    state: &State<'_, Arc<Mutex<AppState>>>,
) -> Result<(ApiClient, AppConfig, CancellationToken), String> {
    let mut state = state.lock().await;
    if state.download_token.is_some() {
        return Err("Download is already in progress".to_string());
    }

    let token = CancellationToken::new();
    state.download_token = Some(token.clone());

    let client = state
        .client
        .as_ref()
        .ok_or("Client not initialized")?
        .clone();
    let cfg = state.config.clone();
    Ok((client, cfg, token))
}

async fn release_work_slot(state: &State<'_, Arc<Mutex<AppState>>>) {
    let mut state = state.lock().await;
    state.download_token = None;
}

fn map_work_err(e: anyhow::Error) -> String {
    if !boosty_downloader_core::is_cancelled_error(&e) {
        log_error!("{e:#}");
    }
    if boosty_downloader_core::is_cancelled_error(&e) {
        boosty_downloader_core::DOWNLOAD_CANCELLED_MESSAGE.to_string()
    } else {
        e.to_string()
    }
}

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
    let previous = state.config.clone();
    state.config = new_config;

    let client = state
        .client
        .as_ref()
        .ok_or("Client not initialized")?
        .clone();

    boosty_downloader_core::sync_auth(&client, &mut state.config, Some(&previous))
        .await
        .map_err(|e| e.to_string())?;

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
    download_options: DownloadOptions,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let (client, cfg, token) = take_work_slot(&state).await?;

    let ctx = boosty_downloader_core::build_url_context(&url, offset_url.as_deref()).map_err(|e| {
        log_error!("{e}");
        e.to_string()
    });
    let ctx = match ctx {
        Ok(ctx) => ctx,
        Err(e) => {
            release_work_slot(&state).await;
            return Err(e);
        }
    };

    let result = boosty_downloader_core::process_boosty_url(
        &client,
        &cfg,
        &ctx.url,
        ctx.offset,
        download_options,
        &token,
    )
    .await;

    release_work_slot(&state).await;
    result.map(|_| ()).map_err(map_work_err)
}

#[tauri::command]
pub async fn cancel_download(state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    let state = state.lock().await;
    if let Some(token) = &state.download_token {
        token.cancel();
        log_info!("{}", boosty_downloader_core::DOWNLOAD_CANCELLED_MESSAGE);
    }
    Ok(())
}

#[tauri::command]
pub async fn get_download_path(state: State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let state = state.lock().await;
    let config = &state.config;
    let path = boosty_downloader_core::get_download_path(config);
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn list_downloaded(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<BlogSnapshot>, String> {
    let cfg = {
        let state = state.lock().await;
        state.config.clone()
    };
    let path = boosty_downloader_core::get_download_path(&cfg);
    scan_downloaded(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_downloaded_blog(
    blog: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<BlogSnapshot, String> {
    let (client, cfg, token) = take_work_slot(&state).await?;
    let result =
        boosty_downloader_core::refresh_downloaded_blog(&client, &cfg, &blog, &token).await;
    release_work_slot(&state).await;
    result.map_err(map_work_err)
}

#[tauri::command]
pub async fn download_downloaded_posts(
    blog: String,
    post_ids: Vec<String>,
    download_options: DownloadOptions,
    force: bool,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<DownloadPostsResult, String> {
    let (client, cfg, token) = take_work_slot(&state).await?;
    let result = boosty_downloader_core::download_downloaded_posts(
        &client,
        &cfg,
        &blog,
        &post_ids,
        download_options,
        force,
        &token,
    )
    .await;
    release_work_slot(&state).await;
    result.map_err(map_work_err)
}

#[tauri::command]
pub async fn delete_downloaded_post(
    blog: String,
    post_id: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let cfg = {
        let state = state.lock().await;
        if state.download_token.is_some() {
            return Err("Download is already in progress".to_string());
        }
        state.config.clone()
    };
    boosty_downloader_core::delete_downloaded_post(&cfg, &blog, &post_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_downloaded_blog(
    blog: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let cfg = {
        let state = state.lock().await;
        if state.download_token.is_some() {
            return Err("Download is already in progress".to_string());
        }
        state.config.clone()
    };
    boosty_downloader_core::delete_downloaded_blog(&cfg, &blog)
        .await
        .map_err(|e| e.to_string())
}
