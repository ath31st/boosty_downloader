use crate::file_handler::DownloadResult;
use anyhow::Error;
use std::collections::HashMap;

pub const ENTER_PATH: &str = "Enter path to post or posts:";
pub const ENTER_ACCESS_TOKEN: &str = "Enter access token:";
pub const ENTER_REFRESH_TOKEN: &str = "Enter refresh token:";
pub const ENTER_CLIENT_ID: &str = "Enter client id:";
pub const ENTER_POSTS_LIMIT: &str = "Enter posts limit:";

pub fn show_menu() {
    println!("1. Download content");
    println!("2. Enter access token");
    println!("3. Enter refresh token and client id");
    println!("4. Clear tokens and client id");
    println!("5. Change posts limit");
    println!("6. Show API client headers");
    println!("7. Exit");
}

fn info(msg: &str) {
    println!("\x1b[34mInfo:\x1b[0m {msg}");
}

fn error(msg: &str) {
    eprintln!("\x1b[31mError:\x1b[0m {msg}");
}

fn warning(msg: &str) {
    println!("\x1b[33mWarning:\x1b[0m {msg}");
}

pub fn read_input_menu() -> i8 {
    loop {
        println!("Select menu:");
        let mut input = String::new();

        if let Err(e) = std::io::stdin().read_line(&mut input) {
            error(&format!("Reading input: {e}"));
            continue;
        }

        match input.trim().parse::<i8>() {
            Ok(num) if (1..=7).contains(&num) => return num,
            _ => error("Please enter a valid number between 1 and 7"),
        }
    }
}

pub fn read_user_input(prompt: &str) -> String {
    loop {
        println!("{prompt}");
        let mut input = String::new();

        if let Err(e) = std::io::stdin().read_line(&mut input) {
            error(&format!("Reading input: {e}"));
            continue;
        }

        if input.trim().is_empty() {
            warning("Input is empty");
            continue;
        }

        return input.trim().to_string();
    }
}

pub fn exit_message() {
    println!("Exiting...");
}

pub fn show_download_result(result: DownloadResult, file_name: &str, post_title: &str) {
    match result {
        DownloadResult::Skipped => {
            info(&format!("File '{file_name}' skipped"));
        }
        DownloadResult::Error(err) => {
            error(&err);
        }
        DownloadResult::Success => {
            info(&format!(
                "File '{file_name}' downloaded for post {post_title}"
            ));
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

pub fn access_token_set(token: &str) {
    let masked = &token.chars().take(4).collect::<String>();
    info(&format!("Access token set: {masked}****"));
}

pub fn refresh_token_set(token: &str) {
    let masked = &token.chars().take(4).collect::<String>();
    info(&format!("Refresh token set: {masked}****"));
}

pub fn client_id_set(client_id: &str) {
    info(&format!("Client id set: {client_id}"));
}

pub fn tokens_and_client_id_cleared() {
    info("Tokens and client id cleared");
}

pub fn post_not_available_or_without_content(post_title: &str) {
    warning(&format!(
        "Post '{post_title}' not available or has no content",
    ));
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
