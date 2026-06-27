use crate::{file_handler::DownloadResult, log_error, log_info};
use anyhow::Error;
use dialoguer::{Input, Select, theme::ColorfulTheme};
use std::{collections::HashMap, path::Path};

pub const ENTER_URL: &str = "Enter URL:";
pub const ENTER_URLS_FILE: &str = "Enter path to file with URLs:";
pub const ENTER_OFFSET_PATH: &str =
    "(OPTIONAL) Enter path to offset post or press enter for skipping:";
pub const ENTER_ACCESS_TOKEN: &str = "Enter access token:";
pub const ENTER_REFRESH_TOKEN: &str = "Enter refresh token:";
pub const ENTER_CLIENT_ID: &str = "Enter client id:";
pub const ENTER_POSTS_LIMIT: &str = "Enter posts limit:";
pub const ENTER_DOWNLOAD_PATH: &str =
    "Enter download path (or press enter to use default - binary folder):";

pub fn info(msg: &str) {
    println!("\x1b[34mInfo:\x1b[0m {msg}");
}

pub fn error(msg: &str) {
    eprintln!("\x1b[31mError:\x1b[0m {msg}");
}

pub fn warning(msg: &str) {
    println!("\x1b[33mWarning:\x1b[0m {msg}");
}

pub fn read_input_menu() -> i8 {
    let items = vec![
        "Download content from URL (blog or post)",
        "Download content from a list of URLs (file)",
        "Enter access token",
        "Enter refresh token and client id (NOT ACTIVE)",
        "Clear tokens and client id",
        "Change posts limit",
        "Change download path",
        "Toggle comments download",
        "Show API client headers",
        "Show config",
        "Exit",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an option")
        .items(&items)
        .default(0)
        .interact_opt();

    match selection {
        Ok(Some(index)) => (index) as i8,
        _ => 10,
    }
}

pub fn read_download_url_and_offset() -> Option<(String, String)> {
    let url: String = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(ENTER_URL)
        .interact_text()
        .ok()?;

    if url.trim().is_empty() {
        return None;
    }

    let offset: String = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(ENTER_OFFSET_PATH)
        .allow_empty(true)
        .interact_text()
        .ok()?;

    Some((url.trim().to_string(), offset.trim().to_string()))
}

pub fn read_batch_file_path() -> Option<String> {
    let result: Result<String, _> = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(ENTER_URLS_FILE)
        .interact_text();

    match result {
        Ok(path) => {
            let trimmed = path.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        Err(_) => None,
    }
}

pub fn read_comments_status(current_enabled: bool) -> Option<bool> {
    let options = vec!["Enabled", "Disabled"];

    let default_index = if current_enabled { 0 } else { 1 };

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Set comments download status")
        .items(&options)
        .default(default_index)
        .interact_opt();

    match selection {
        Ok(Some(index)) => Some(index == 0),
        _ => None,
    }
}

pub fn read_posts_limit(current_limit: usize) -> Option<usize> {
    let prompt = format!("{} (current: {})", ENTER_POSTS_LIMIT, current_limit);

    let result: Result<usize, _> = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(&prompt)
        .default(current_limit)
        .interact_text();

    result.ok()
}

pub fn read_download_path(current_path: Option<&str>) -> Option<Option<String>> {
    let default_display = current_path.unwrap_or("(default - binary folder)");
    let prompt = format!("{} (current: {})", ENTER_DOWNLOAD_PATH, default_display);

    let result: Result<String, _> = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(&prompt)
        .allow_empty(true)
        .interact_text();

    match result {
        Ok(entered_path) => {
            let trimmed = entered_path.trim();

            if trimmed.is_empty() {
                return Some(None);
            }

            let path = Path::new(trimmed);
            if !path.exists() {
                info("Path does not exist, will try to create on download");
            }

            Some(Some(trimmed.to_string()))
        }
        Err(_) => None,
    }
}

pub fn read_access_token() -> Option<String> {
    let result: Result<String, _> = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(ENTER_ACCESS_TOKEN)
        .interact();

    match result {
        Ok(token) => {
            let trimmed = token.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        Err(_) => None,
    }
}

pub fn read_refresh_and_client_id() -> Option<(String, String)> {
    let refresh_token: String = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(ENTER_REFRESH_TOKEN)
        .interact()
        .ok()?
        .trim()
        .to_string();

    if refresh_token.is_empty() {
        return None;
    }

    let client_id: String = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(ENTER_CLIENT_ID)
        .interact_text()
        .ok()?
        .trim()
        .to_string();

    if client_id.is_empty() {
        return None;
    }

    Some((refresh_token, client_id))
}

pub fn exit_message() {
    println!("Exiting...");
}

pub fn show_download_result(result: DownloadResult, file_name: &str, post_title: &str) {
    match result {
        DownloadResult::Skipped => {
            log_info!("File '{file_name}' skipped");
        }
        DownloadResult::Error(err) => {
            log_error!("{err}");
        }
        DownloadResult::Success => {
            log_info!("File '{file_name}' downloaded for post '{post_title}'");
        }
    }
}

pub fn unknown_content_item() {
    info("Post item with unknown content");
}

pub fn show_api_client_headers(headers: &HashMap<String, String>) {
    info("Current API client headers:");
    for (key, value) in headers {
        println!("  {key}: {value}");
    }
    println!()
}

fn masked_str(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }

    let masked = &s.chars().take(4).collect::<String>();
    format!("{masked}****")
}

pub fn access_token_set(token: &str) {
    info(&format!("Access token set: {}", masked_str(token)));
}

pub fn refresh_token_set(token: &str) {
    info(&format!("Refresh token set: {}", masked_str(token)));
}

pub fn client_id_set(client_id: &str) {
    info(&format!("Client id set: {client_id}"));
}

pub fn show_config(config: &crate::config::AppConfig) {
    println!("Config:");
    println!("  Access token: {}", masked_str(&config.access_token));
    println!("  Refresh token: {}", masked_str(&config.refresh_token));
    println!("  Client id: {}", config.device_id);
    println!("  Posts limit: {}", config.posts_limit);
    println!(
        "  Download path: {}",
        config
            .download_path
            .as_deref()
            .unwrap_or("(default - binary folder)")
    );
}

pub fn tokens_and_client_id_cleared() {
    info("Tokens and client id cleared");
}

pub fn post_not_available_or_without_content(post_title: &str) {
    warning(&format!(
        "Post '{post_title}' not available or has no content",
    ));
}

pub fn comments_for_post_empty_or_not_available(post_title: &str) {
    warning(&format!(
        "Comments for post '{post_title}' empty or not available",
    ));
}

pub fn comments_toggled(status: &str) {
    info(&format!("Downloading comments {status}"));
}

pub fn error_while_loading_config(e: &Error) {
    error(&format!("Error while loading config: {e}"));
    warning("Config will be reset to default");
}

pub fn print_error(e: &Error) {
    if cfg!(debug_assertions) {
        for cause in e.chain() {
            error(&format!("Caused by: {cause}"));
        }
    } else {
        error(&format!("{e}"));
    }
}
