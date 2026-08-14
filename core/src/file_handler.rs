use crate::progress_reporter;
use crate::{headers, log_error, log_info, log_warn};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_util::sync::CancellationToken;

pub enum DownloadResult {
    Error(String),
    Success,
    Skipped,
}

const MAX_RETRIES: usize = 5;
const MAX_TITLE_CHARS: usize = 80;

async fn ensure_blog_folder(blog_name: &str, base_path: &Path) -> Result<PathBuf> {
    let blog_path = base_path.join(blog_name);
    let exists = fs::try_exists(&blog_path)
        .await
        .with_context(|| format!("Failed to check if blog folder '{blog_name}' exists"))?;
    if !exists {
        fs::create_dir_all(&blog_path)
            .await
            .with_context(|| format!("Failed to create blog folder '{blog_name}'"))?;
    }
    Ok(blog_path)
}

async fn ensure_post_folder(
    blog_name: &str,
    folder_name: &str,
    base_path: &Path,
) -> Result<PathBuf> {
    let blog_path = ensure_blog_folder(blog_name, base_path).await?;
    let post_path = blog_path.join(folder_name);
    let exists = fs::try_exists(&post_path).await.with_context(|| {
        format!(
            "Failed to check if post folder '{}' exists",
            post_path.display()
        )
    })?;
    if !exists {
        fs::create_dir_all(&post_path)
            .await
            .with_context(|| format!("Failed to create post folder '{}'", post_path.display()))?;
    }
    Ok(post_path)
}

async fn download_file_content(
    folder_path: &Path,
    url: &str,
    title: &str,
    signed_query: Option<&str>,
    cancel_token: &CancellationToken,
) -> Result<DownloadResult> {
    log_info!("Downloading file '{title}'...");
    for attempt in 1..=MAX_RETRIES {
        crate::ensure_not_cancelled(cancel_token)?;
        match download_file_once(folder_path, url, title, signed_query, cancel_token).await {
            Ok(r @ DownloadResult::Success) => {
                progress_reporter::finish_file();
                return Ok(r);
            }
            Ok(r @ DownloadResult::Skipped) => {
                progress_reporter::finish_file();
                return Ok(r);
            }
            Ok(DownloadResult::Error(ref msg)) if !is_retriable_download_error(msg) => {
                progress_reporter::finish_file();
                return Ok(DownloadResult::Error(msg.clone()));
            }
            Ok(DownloadResult::Error(_)) if attempt < MAX_RETRIES => {
                progress_reporter::abandon_file();
                log_warn!("Download attempt {attempt} failed (logical error), retrying...");
            }
            Err(e) if crate::is_cancelled_error(&e) => {
                progress_reporter::abandon_file();
                let safe_name = sanitize_name(title);
                let output_path = folder_path.join(safe_name);
                let _ = fs::remove_file(&output_path).await;
                return Err(e);
            }
            Err(e) if attempt < MAX_RETRIES => {
                progress_reporter::abandon_file();
                log_error!("Download attempt {attempt} failed with error: {e}");
            }
            result => {
                progress_reporter::finish_file();
                return result;
            }
        }

        let safe_name = sanitize_name(title);
        let output_path = folder_path.join(safe_name);
        let _ = fs::remove_file(&output_path).await;

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt as u32))) => {}
            _ = cancel_token.cancelled() => anyhow::bail!(crate::DOWNLOAD_CANCELLED_MESSAGE),
        }
    }
    unreachable!("MAX_RETRIES exhausted but loop should return earlier")
}

fn is_retriable_download_error(msg: &str) -> bool {
    if msg.contains("Authorization required") {
        return false;
    }
    if msg.contains("HTTP 401") || msg.contains("HTTP 403") || msg.contains("HTTP 404") {
        return false;
    }
    true
}

pub async fn download_file_once(
    folder_path: &Path,
    url: &str,
    title: &str,
    signed_query: Option<&str>,
    cancel_token: &CancellationToken,
) -> Result<DownloadResult> {
    crate::ensure_not_cancelled(cancel_token)?;
    let safe_name = sanitize_name(title);
    let output_path = folder_path.join(safe_name);

    let exists = fs::try_exists(&output_path).await.with_context(|| {
        format!(
            "Failed to check existence of file '{}'",
            output_path.display()
        )
    })?;
    if exists {
        return Ok(DownloadResult::Skipped);
    }

    let signed_query = if signed_query.is_some() && signed_query.unwrap().is_empty() {
        return Ok(DownloadResult::Error(format!(
            "Authorization required: to download file '{title}' an access token must be provided"
        )));
    } else {
        signed_query.unwrap_or("")
    };

    let full_url = format!("{url}{signed_query}");
    let client = reqwest::Client::new();
    let resp = client
        .get(full_url)
        .headers(headers::default_download_headers())
        .send()
        .await
        .with_context(|| format!("HTTP GET failed for file URL '{url}'"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let error_body = resp.text().await.unwrap_or_default();
        return Ok(DownloadResult::Error(format!(
            "HTTP {status}: {error_body}"
        )));
    }

    let total_size = resp.content_length().unwrap_or(0);
    progress_reporter::start_file(title, total_size)?;

    let mut file = fs::File::create(&output_path)
        .await
        .with_context(|| format!("Failed to create file '{}'", output_path.display()))?;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        crate::ensure_not_cancelled(cancel_token)?;
        let chunk = chunk.with_context(|| format!("Error while reading chunk from '{url}'"))?;
        file.write_all(&chunk).await?;
        progress_reporter::inc(chunk.len() as u64);
    }

    Ok(DownloadResult::Success)
}

pub fn post_folder_name(title: &str, created_at: i64, post_id: &str) -> String {
    let datetime: DateTime<Utc> = DateTime::from_timestamp(created_at, 0)
        .or_else(|| DateTime::from_timestamp(0, 0))
        .expect("unix epoch is a valid timestamp");
    let date_str = datetime.format("%Y.%m.%d").to_string();
    let safe_id = sanitize_name(post_id);
    let mut safe_title = sanitize_name(title);

    if safe_title.chars().count() > MAX_TITLE_CHARS {
        safe_title = safe_title.chars().take(MAX_TITLE_CHARS).collect();
        while safe_title.ends_with('.') || safe_title.ends_with(' ') {
            safe_title.pop();
        }
        if safe_title.is_empty() {
            safe_title = "_".to_string();
        }
    }

    format!("{date_str} {safe_title} [{safe_id}]")
}

pub async fn prepare_folder_path(
    blog_name: &str,
    post_title: &str,
    created_at: i64,
    post_id: &str,
    base_path: &Path,
) -> Result<PathBuf> {
    let folder_name = post_folder_name(post_title, created_at, post_id);

    let post_folder_path: PathBuf = ensure_post_folder(blog_name, &folder_name, base_path)
        .await
        .with_context(|| {
            format!("Failed to create folder for post '{post_title}' in blog '{blog_name}'")
        })?;

    Ok(post_folder_path)
}

pub async fn prepare_folder_path_for_comments(post_folder_path: &Path) -> Result<PathBuf> {
    let comments_folder_path = post_folder_path.join("comments");

    fs::create_dir_all(&comments_folder_path)
        .await
        .with_context(|| {
            format!(
                "Failed to create comments folder '{}'",
                comments_folder_path.display()
            )
        })?;

    Ok(comments_folder_path)
}

pub async fn read_links_from_file(file_path: &Path) -> Result<Vec<String>> {
    if !fs::try_exists(file_path).await? {
        anyhow::bail!("File with links does not exist: '{}'", file_path.display());
    }

    let file = fs::File::open(file_path)
        .await
        .with_context(|| format!("Failed to open file: '{}'", file_path.display()))?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut links = Vec::new();

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();

        if !trimmed.is_empty() {
            links.push(trimmed.to_string());
        }
    }

    log_info!("Read {} links", links.len());
    Ok(links)
}

pub async fn download_media(
    folder_path: &Path,
    url: &str,
    file_name: &str,
    post_title: &str,
    signed_query: Option<&str>,
    cancel_token: &CancellationToken,
) -> Result<(DownloadResult, String)> {
    crate::ensure_not_cancelled(cancel_token)?;
    let result = download_file_content(folder_path, url, file_name, signed_query, cancel_token)
        .await
        .with_context(|| {
            format!("Failed to download file '{file_name}' for post '{post_title}'")
        })?;

    let rel = sanitize_name(file_name).replace('\\', "/");
    Ok((result, rel))
}

pub fn media_file_name(id: &str, title: &str, extension: Option<&str>) -> String {
    let base = format!("{id}_{title}");
    let mut name = sanitize_name(&base);
    if let Some(ext) = extension {
        let ext = ext.trim_start_matches('.');
        if !ext.is_empty() {
            let suffix = format!(".{ext}");
            if !name.to_lowercase().ends_with(&suffix.to_lowercase()) {
                name.push_str(&suffix);
            }
        }
    }
    name
}

pub fn audio_extension(file_type: Option<&str>) -> Option<&'static str> {
    match file_type.map(|s| s.to_ascii_lowercase()) {
        Some(ft) if ft == "mp3" || ft.contains("mpeg") => Some("mp3"),
        Some(ft) if ft == "wav" || ft.contains("wav") => Some("wav"),
        Some(ft) if ft == "ogg" || ft.contains("ogg") => Some("ogg"),
        Some(ft) if ft == "flac" || ft.contains("flac") => Some("flac"),
        Some(ft) if ft == "m4a" || ft.contains("mp4") || ft.contains("aac") => Some("m4a"),
        _ => Some("mp3"),
    }
}

pub fn sanitize_name(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | '\0' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            other => other,
        })
        .collect();

    while s.ends_with('.') || s.ends_with(' ') {
        s.pop();
    }

    if s.is_empty() { "_".to_string() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_folder_name_includes_id() {
        let a = post_folder_name("Same title", 1_700_000_000, "id-aaa");
        let b = post_folder_name("Same title", 1_700_000_000, "id-bbb");
        assert_ne!(a, b);
        assert!(a.contains("[id-aaa]"));
        assert!(b.contains("[id-bbb]"));
        assert!(a.starts_with("2023.11.14 "));
    }

    #[test]
    fn post_folder_name_truncates_title_keeps_id() {
        let long_title = "a".repeat(200);
        let name = post_folder_name(&long_title, 1_700_000_000, "keep-this-id");
        assert!(name.contains("[keep-this-id]"));
        assert!(name.len() < long_title.len() + 40);
    }

    #[test]
    fn media_file_name_prefixes_id() {
        assert_eq!(media_file_name("abc", "track.mp3", None), "abc_track.mp3");
        assert_eq!(
            media_file_name("abc", "track", Some("mp3")),
            "abc_track.mp3"
        );
    }
}
