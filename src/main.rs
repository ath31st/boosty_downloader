mod checks;
mod cli;
mod file_handler;
mod menu_handler;
mod parser;
mod post_handler;

use crate::menu_handler::handle_menu;
use anyhow::Result;
use boosty_api::api_client::ApiClient;
use reqwest::Client;
use std::time::Duration;

const API_URL: &str = "https://api.boosty.to";
const POSTS_LIMIT: i32 = 100;
const TIMEOUT_SECONDS: u64 = 10;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        cli::print_error(&e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    checks::check_ffmpeg()?;

    let client = Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECONDS))
        .build()
        .expect("Failed to build reqwest Client");

    let client = ApiClient::new(client, API_URL);
    checks::check_api(&client).await?;

    loop {
        if !handle_menu(&client, POSTS_LIMIT).await? {
            break;
        }
    }
    Ok(())
}
