use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub posts_limit: usize,
    pub access_token: String,
    pub refresh_token: String,
    pub device_id: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            posts_limit: 100,
            access_token: String::new(),
            refresh_token: String::new(),
            device_id: String::new(),
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
    println!("Config path: {}", path.display());

    if !fs::try_exists(&path).await? {
        let default = AppConfig::default();
        save_config(&default).await?;
        return Ok(default);
    }

    let data = fs::read(&path)
        .await
        .with_context(|| format!("Failed to read config file '{}'", path.display()))?;

    let config: AppConfig = serde_json::from_slice(&data)
        .with_context(|| format!("Failed to parse config file '{}'", path.display()))?;

    Ok(config)
}

pub async fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path()?;
    let data = serde_json::to_vec_pretty(config).with_context(|| "Failed to serialize config")?;

    fs::write(&path, data)
        .await
        .with_context(|| format!("Failed to write config file '{}'", path.display()))?;

    Ok(())
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
