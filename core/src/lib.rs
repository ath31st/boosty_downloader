pub(crate) mod checks;
pub(crate) mod cli;
pub(crate) mod comment_handler;
pub(crate) mod config;
pub(crate) mod console_logger;
pub(crate) mod content_items_handler;
pub(crate) mod file_handler;
pub(crate) mod headers;
pub(crate) mod logger;
pub(crate) mod menu_handler;
pub(crate) mod parser;
pub(crate) mod post_handler;
pub(crate) mod progress_reporter;
pub(crate) mod url_context;

pub use cli::print_error;
pub use config::{AppConfig, CommentsConfig, load_config, save_config, sync_auth};
pub use console_logger::ConsoleLogger;
pub use logger::{LogLevel, LogMessage, Logger, ProgressMessage, get_logger, set_logger};
pub use menu_handler::{handle_menu, process_boosty_url};
pub use url_context::build_url_context;

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
    let cfg = config::load_config().await?;
    config::sync_auth(client, &cfg).await?;
    Ok(())
}
