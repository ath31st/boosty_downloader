use anyhow::{Context, Result};
use boosty_api::api_client::ApiClient;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::cli;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub posts_limit: usize,
    pub access_token: String,
    pub refresh_token: String,
    pub device_id: String,
    pub comments: CommentsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommentsConfig {
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
                reply_limit: Some(10),
                limit: Some(300),
                // top or bottom
                order: Some("bottom".to_string()),
            },
        }
    }
}

fn config_path() -> Result<PathBuf> {
    let exe = std::env::current_exe().with_context(|| "Failed to get current executable path")?;
    let dir = exe.parent().context("Failed to get executable directory")?;

    Ok(dir.join("config.json"))
}

pub async fn load_config() -> Result<AppConfig> {
    let path = config_path()?;

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

pub async fn update_config<F>(f: F) -> Result<AppConfig>
where
    F: FnOnce(&mut AppConfig),
{
    let mut cfg = load_config().await?;

    f(&mut cfg);

    save_config(&cfg).await?;

    Ok(cfg)
}

pub async fn sync_auth(client: &ApiClient, cfg: &AppConfig) -> Result<()> {
    if !cfg.access_token.is_empty() {
        client.set_bearer_token(&cfg.access_token).await?;
        cli::access_token_set(&cfg.access_token);
    } else {
        client.clear_access_token().await;
    }

    if !cfg.refresh_token.is_empty() && !cfg.device_id.is_empty() {
        client
            .set_refresh_token_and_device_id(&cfg.refresh_token, &cfg.device_id)
            .await?;
        cli::refresh_token_set(&cfg.refresh_token);
        cli::client_id_set(&cfg.device_id);
    }

    Ok(())
}
