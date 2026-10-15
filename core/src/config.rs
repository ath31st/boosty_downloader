use anyhow::{Context, Result};
use boosty_api::api_client::ApiClient;
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};
use tokio::fs;

use crate::cli;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub posts_limit: usize,
    pub access_token: String,
    pub refresh_token: String,
    pub device_id: String,
    pub comments: CommentsConfig,
    pub download_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentsConfig {
    pub enabled: bool,
    pub reply_limit: Option<u32>,
    pub limit: Option<u32>,
    pub order: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            posts_limit: 100,
            access_token: String::new(),
            refresh_token: String::new(),
            device_id: String::new(),
            comments: CommentsConfig {
                enabled: true,
                reply_limit: Some(10),
                limit: Some(300),
                // top or bottom
                order: Some("bottom".to_string()),
            },
            download_path: None,
        }
    }
}

const APP_DIR_NAME: &str = "boosty-downloader";
const CONFIG_FILE_NAME: &str = "config.json";

fn user_config_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let appdata = env::var_os("APPDATA").context("APPDATA is not set")?;
        return Ok(PathBuf::from(appdata));
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(xdg) = env::var_os("XDG_CONFIG_HOME")
            && !xdg.is_empty()
        {
            return Ok(PathBuf::from(xdg));
        }

        let home = env::var_os("HOME").context("HOME is not set")?;
        Ok(PathBuf::from(home).join(".config"))
    }
}

fn user_downloads_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        return env::var_os("USERPROFILE")
            .map(|home| PathBuf::from(home).join("Downloads"))
            .unwrap_or_else(|| PathBuf::from("Downloads"));
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(xdg) = env::var_os("XDG_DOWNLOAD_DIR")
            && !xdg.is_empty()
        {
            return PathBuf::from(xdg);
        }

        env::var_os("HOME")
            .map(|home| PathBuf::from(home).join("Downloads"))
            .unwrap_or_else(|| PathBuf::from("Downloads"))
    }
}

fn config_path() -> Result<PathBuf> {
    Ok(user_config_dir()?.join(APP_DIR_NAME).join(CONFIG_FILE_NAME))
}

/// Old location: `config.json` next to the executable.
///
/// Temporary migration for installs that stored the file beside the binary.
/// Remove once those copies have moved to [`config_path`].
fn legacy_config_path() -> Result<PathBuf> {
    let exe = env::current_exe().with_context(|| "Failed to get current executable path")?;
    let dir = exe.parent().context("Failed to get executable directory")?;

    Ok(dir.join(CONFIG_FILE_NAME))
}

async fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create directory '{}'", parent.display()))?;
    }

    Ok(())
}

async fn migrate_legacy_config(dest: &Path) -> Result<()> {
    let legacy = legacy_config_path()?;
    if legacy == dest || !fs::try_exists(&legacy).await? || fs::try_exists(dest).await? {
        return Ok(());
    }

    ensure_parent_dir(dest).await?;

    if fs::rename(&legacy, dest).await.is_err() {
        fs::copy(&legacy, dest).await.with_context(|| {
            format!(
                "Failed to copy config from '{}' to '{}'",
                legacy.display(),
                dest.display()
            )
        })?;
        let _ = fs::remove_file(&legacy).await;
    }

    crate::log_info!(
        "Moved config from '{}' to '{}'",
        legacy.display(),
        dest.display()
    );

    Ok(())
}

pub async fn load_config() -> Result<AppConfig> {
    let path = config_path()?;
    migrate_legacy_config(&path).await?;

    if !fs::try_exists(&path).await? {
        let default = AppConfig::default();
        save_config(&default).await?;
        return Ok(default);
    }

    let data = match fs::read(&path).await {
        Ok(d) => d,
        Err(e) => {
            cli::error_while_loading_config(&anyhow::Error::from(e));

            return reset_config(&path).await;
        }
    };

    match serde_json::from_slice::<AppConfig>(&data) {
        Ok(cfg) => Ok(cfg),
        Err(e) => {
            cli::error_while_loading_config(&anyhow::Error::from(e));

            reset_config(&path).await
        }
    }
}

pub async fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path()?;
    ensure_parent_dir(&path).await?;
    let data = serde_json::to_vec_pretty(config).with_context(|| "Failed to serialize config")?;

    fs::write(&path, data)
        .await
        .with_context(|| format!("Failed to write config file '{}'", path.display()))?;

    Ok(())
}

async fn reset_config(path: &Path) -> Result<AppConfig> {
    let _ = fs::remove_file(&path).await;

    let default = AppConfig::default();
    save_config(&default).await?;
    Ok(default)
}

pub fn get_download_path(config: &AppConfig) -> PathBuf {
    match &config.download_path {
        Some(path) => PathBuf::from(path),
        None => user_downloads_dir().join(APP_DIR_NAME),
    }
}

pub async fn update_config<F>(f: F) -> Result<AppConfig>
where
    F: FnOnce(&mut AppConfig),
{
    let mut cfg = load_config().await?;

    f(&mut cfg);

    save_config(&cfg).await?;

    Ok(cfg)
}

/// Apply auth from config to the API client.
///
/// Performs a network refresh only when:
/// - refresh token + device ID are set, and
/// - access token is empty, or refresh credentials changed compared to `previous`.
///
/// Otherwise just applies the existing access token (or clears auth).
pub async fn sync_auth(
    client: &ApiClient,
    cfg: &mut AppConfig,
    previous: Option<&AppConfig>,
) -> Result<()> {
    let has_refresh = !cfg.refresh_token.is_empty() && !cfg.device_id.is_empty();
    let refresh_changed = previous.is_some_and(|prev| {
        prev.refresh_token != cfg.refresh_token || prev.device_id != cfg.device_id
    });
    let needs_refresh = has_refresh && (cfg.access_token.is_empty() || refresh_changed);

    if needs_refresh {
        client
            .set_refresh_token_and_device_id(&cfg.refresh_token, &cfg.device_id)
            .await?;
        let pair = client.refresh_tokens().await?;

        cfg.access_token = pair.access_token;
        cfg.refresh_token = pair.refresh_token;

        cli::access_token_set(&cfg.access_token);
        cli::refresh_token_set(&cfg.refresh_token);
        cli::client_id_set(&cfg.device_id);
    } else if !cfg.access_token.is_empty() {
        client.set_bearer_token(&cfg.access_token).await?;
        cli::access_token_set(&cfg.access_token);
    } else {
        clear_auth(client, cfg).await?;
    }

    Ok(())
}

/// Clear all auth credentials on the client and in the given config.
pub async fn clear_auth(client: &ApiClient, cfg: &mut AppConfig) -> Result<()> {
    client.clear_access_token().await;
    client.clear_refresh_and_device_id().await;
    cfg.access_token.clear();
    cfg.refresh_token.clear();
    cfg.device_id.clear();
    Ok(())
}
