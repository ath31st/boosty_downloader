pub mod checks;
pub mod cli;
pub mod comment_handler;
pub mod config;
pub mod content_items_handler;
pub mod file_handler;
pub mod headers;
pub mod menu_handler;
pub mod parser;
pub mod post_handler;

use anyhow::Result;
use boosty_api::api_client::ApiClient;
use reqwest::Client;
use std::time::Duration;

const API_URL: &str = "https://api.boosty.to";
const TIMEOUT_SECONDS: u64 = 10;

pub async fn make_client() -> Result<ApiClient> {
    let client = Client::builder()
        .http1_only()
        .connect_timeout(Duration::from_secs(TIMEOUT_SECONDS))
        .pool_idle_timeout(None)
        .tcp_keepalive(Some(Duration::from_secs(TIMEOUT_SECONDS * 3)))
        .build()?;

    Ok(ApiClient::new(client, API_URL))
}

pub async fn init_client(client: &ApiClient) -> Result<()> {
    checks::check_api(client).await?;
    config::apply_config(client).await?;
    Ok(())
}
