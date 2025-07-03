use crate::file_handler::DownloadResult;
use anyhow::Error;
use std::collections::HashMap;

pub const ENTER_PATH: &str = "Enter path to post or posts:";
pub const ENTER_ACCESS_TOKEN: &str = "Enter access token:";
pub const ENTER_REFRESH_TOKEN: &str = "Enter refresh token:";
pub const ENTER_CLIENT_ID: &str = "Enter client id:";

pub fn show_menu() {
    println!("1. Download content");
    println!("2. Enter access token (optional)");
    println!("3. Enter refresh token and client id (optional)");
    println!("4. Exit");
}

fn info(msg: &str) {
    println!("\x1b[34mInfo:\x1b[0m {}", msg);
}

fn error(msg: &str) {
    eprintln!("\x1b[31mError:\x1b[0m {}", msg);
}

fn warning(msg: &str) {
    println!("\x1b[33mWarning:\x1b[0m {}", msg);
}

pub fn read_input_menu() -> i8 {
    loop {
        println!("Select menu:");
        let mut input = String::new();

        if let Err(e) = std::io::stdin().read_line(&mut input) {
            error(&format!("Reading input: {}", e));
            continue;
        }

        match input.trim().parse::<i8>() {
            Ok(num) if (1..=4).contains(&num) => return num,
            _ => error("Please enter a valid number between 1 and 4"),
        }
    }
}

pub fn read_user_input(prompt: &str) -> String {
    loop {
        println!("{}", prompt);
        let mut input = String::new();

        if let Err(e) = std::io::stdin().read_line(&mut input) {
            error(&format!("Reading input: {}", e));
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
            info(&format!("File '{}' skipped", file_name));
        }
        DownloadResult::Error(err) => {
            error(&err);
        }
        DownloadResult::Success => {
            info(&format!(
                "File '{}' downloaded for post {}",
                file_name, post_title
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
        println!("  {}: {}", key, value);
    }
    println!()
}

pub fn post_not_available_or_without_content(post_title: &str) {
    warning(&format!(
        "Post '{}' not available or has no content",
        post_title
    ));
}

pub fn print_error(e: &Error) {
    if cfg!(debug_assertions) {
        for cause in e.chain() {
            error(&format!("Caused by: {}", cause));
        }
    } else {
        error(&format!("{}", e));
    }
}
