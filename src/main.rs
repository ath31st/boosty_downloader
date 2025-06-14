mod checks;
mod cli;
mod file_handler;
mod menu_handler;
mod parser;
mod post_handler;

use crate::menu_handler::handle_menu;
use anyhow::Result;
use roosty_downloader_api::api_client::ApiClient;

const API_URL: &str = "https://api.boosty.to";
const POSTS_LIMIT: i32 = 2;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        cli::print_error(&e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    checks::check_ffmpeg()?;

    let mut client = ApiClient::new(API_URL);
    checks::check_api(&mut client).await?;

    loop {
        if !handle_menu(&mut client, POSTS_LIMIT).await? {
            break;
        }
    }
    Ok(())
}
