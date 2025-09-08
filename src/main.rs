mod checks;
mod cli;
mod config;
mod file_handler;
mod headers;
mod menu_handler;
mod parser;
mod post_handler;

use crate::menu_handler::handle_menu;
use anyhow::Result;
use boosty_api::api_client::ApiClient;
use reqwest::Client;
use std::time::Duration;

const API_URL: &str = "https://api.boosty.to";
const TIMEOUT_SECONDS: u64 = 10;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        cli::print_error(&e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let client = Client::builder()
        .http1_only()
        .connect_timeout(Duration::from_secs(TIMEOUT_SECONDS))
        .pool_idle_timeout(None)
        .tcp_keepalive(Some(Duration::from_secs(TIMEOUT_SECONDS * 3)))
        .build()
        .expect("Failed to build reqwest Client");

    let client = ApiClient::new(client, API_URL);
    checks::check_api(&client).await?;

    apply_config(&client).await?;

    loop {
        if !handle_menu(&client).await? {
            break;
        }
    }
    Ok(())
}

async fn apply_config(client: &ApiClient) -> Result<()> {
    let cfg = config::load_config().await?;

    if !cfg.access_token.is_empty() {
        client.set_bearer_token(&cfg.access_token).await?;
        cli::access_token_set(&cfg.access_token);
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
