mod checks;
mod cli;
mod comment_handler;
mod config;
mod content_items_handler;
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

    config::apply_config(&client).await?;

    loop {
        if !handle_menu(&client).await? {
            break;
        }
    }
    Ok(())
}
