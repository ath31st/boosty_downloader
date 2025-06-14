use crate::file_handler::DownloadResult;
use std::collections::HashMap;

pub const ENTER_PATH: &str = "Enter path to post or posts:";
pub const ENTER_TOKEN: &str = "Enter access token:";

pub fn show_menu() {
    println!("1. Download content");
    println!("2. Enter access token (optional)");
    println!("3. Exit");
}

pub fn read_input_menu() -> i8 {
    loop {
        println!("Select menu:");
        let mut input = String::new();

        if let Err(e) = std::io::stdin().read_line(&mut input) {
            println!("Error reading input: {}", e);
            continue;
        }

        match input.trim().parse::<i8>() {
            Ok(num) if (1..=3).contains(&num) => return num,
            _ => println!("Please enter a valid number between 1 and 3"),
        }
    }
}

pub fn read_user_input(prompt: &str) -> String {
    loop {
        println!("{}", prompt);
        let mut input = String::new();

        if let Err(e) = std::io::stdin().read_line(&mut input) {
            println!("Error reading input: {}", e);
            continue;
        }

        if input.trim().is_empty() {
            println!("Input is empty");
            continue;
        }

        return input.trim().to_string();
    }
}

pub fn exit_message() {
    println!("Exiting...");
}

pub fn show_download_result(result: DownloadResult, video_title: &str, post_title: &str) {
    match result {
        DownloadResult::Skipped => {
            println!("File '{}' skipped", video_title);
        }
        DownloadResult::Error(error) => {
            println!("Error: {}", error);
        }
        DownloadResult::Success => {
            println!("File '{}' downloaded for post {}", video_title, post_title);
        }
    }
}

pub fn unknown_content_item() {
    println!("\x1b[34mInfo:\x1b[0m Post item with unknown content");
}

pub fn show_api_client_headers(headers: &HashMap<String, String>) {
    println!("Current API client headers:");
    for (key, value) in headers {
        println!("  {}: {}", key, value);
    }
    println!()
}

pub fn post_not_available_or_without_content(post_title: &str) {
    println!("Post '{}' not available or has no content", post_title);
}

pub fn print_error(msg: &str) {
    eprintln!("\x1b[31mError:\x1b[0m {}", msg);
}
