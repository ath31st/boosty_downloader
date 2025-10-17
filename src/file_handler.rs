use crate::headers;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub enum DownloadResult {
    Error(String),
    Success,
    Skipped,
}

const MAX_RETRIES: usize = 5;

fn hash_str(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn load_existing_hashes(path: &Path) -> Result<HashSet<String>> {
    if !fs::try_exists(path).await? {
        return Ok(HashSet::new());
    }

    let file = fs::File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut hashes = HashSet::new();

    while let Some(line) = lines.next_line().await? {
        hashes.insert(line);
    }

    Ok(hashes)
}

async fn append_hash_to_file(path: &Path, hash: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    file.write_all(format!("{hash}\n").as_bytes()).await?;
    Ok(())
}

async fn ensure_blog_folder(blog_name: &str) -> Result<PathBuf> {
    let blog_path = Path::new(blog_name);
    let exists = fs::try_exists(blog_path)
        .await
        .with_context(|| format!("Failed to check if blog folder '{blog_name}' exists"))?;
    if !exists {
        fs::create_dir_all(blog_path)
            .await
            .with_context(|| format!("Failed to create blog folder '{blog_name}'"))?;
    }
    Ok(blog_path.to_path_buf())
}

async fn ensure_post_folder(blog_name: &str, folder_name: &str) -> Result<PathBuf> {
    let blog_path = ensure_blog_folder(blog_name).await?;
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

pub async fn download_text_content(
    folder_path: &Path,
    post_title: &str,
    content: &str,
    modificator: Option<&str>,
) -> Result<DownloadResult> {
    let safe_name = sanitize_name(post_title);
    let output_path = folder_path.join(format!("{safe_name}.md"));
    let hashes_path = folder_path.join(format!("{safe_name}.hashes"));

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(&output_path)
        .await
        .with_context(|| format!("Failed to open file '{}'", output_path.display()))?;

    if modificator.is_some() && modificator.unwrap() == "BLOCK_END" {
        file.write_all(b"\n").await?;
        return Ok(DownloadResult::Success);
    }

    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(DownloadResult::Skipped);
    }

    let existing = load_existing_hashes(&hashes_path).await?;
    let hash = hash_str(trimmed);
    if existing.contains(&hash) {
        return Ok(DownloadResult::Skipped);
    }

    file.write_all(trimmed.as_bytes())
        .await
        .with_context(|| format!("Failed to write to file '{}'", output_path.display()))?;
    file.write_all(b"\n").await?;

    append_hash_to_file(&hashes_path, &hash).await?;

    Ok(DownloadResult::Success)
}

pub async fn download_file_content(
    folder_path: &Path,
    url: &str,
    title: &str,
    signed_query: Option<&str>,
) -> Result<DownloadResult> {
    for attempt in 1..=MAX_RETRIES {
        match download_file_once(folder_path, url, title, signed_query).await {
            Ok(r @ DownloadResult::Success) => return Ok(r),
            Ok(r @ DownloadResult::Skipped) => return Ok(r),
            Ok(_r @ DownloadResult::Error(_)) if attempt < MAX_RETRIES => {
                eprintln!("Download attempt {attempt} failed (logical error), retrying...");
            }
            Err(e) if attempt < MAX_RETRIES => {
                eprintln!("Download attempt {attempt} failed with error: {e}, retrying...");
            }
            result => return result,
        }

        let safe_name = sanitize_name(title);
        let output_path = folder_path.join(safe_name);
        let _ = fs::remove_file(&output_path).await;

        tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt as u32))).await;
    }
    unreachable!("MAX_RETRIES exhausted but loop should return earlier")
}

pub async fn download_file_once(
    folder_path: &Path,
    url: &str,
    title: &str,
    signed_query: Option<&str>,
) -> Result<DownloadResult> {
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
        let error_body = resp.text().await.unwrap_or_default();
        return Ok(DownloadResult::Error(format!("HTTP {error_body}")));
    }

    let total_size = resp.content_length().unwrap_or(0);

    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
            )?
            .progress_chars("=> "),
        );
        pb
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::with_template(
            "{spinner:.green} Downloading file... {bytes}",
        )?);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    };

    let mut file = fs::File::create(&output_path)
        .await
        .with_context(|| format!("Failed to create file '{}'", output_path.display()))?;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.with_context(|| format!("Error while reading chunk from '{url}'"))?;
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }
    pb.finish_and_clear();

    Ok(DownloadResult::Success)
}

pub async fn normalize_md_file(post_folder: &Path, title: &str) -> Result<()> {
    let md_path = post_folder.join(format!("{}.md", sanitize_name(title)));
    if !fs::try_exists(&md_path).await? {
        return Ok(());
    }
    let text = fs::read_to_string(&md_path).await?;

    let mut normalized = String::new();
    let mut empty_count = 0;

    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            empty_count += 1;
            if empty_count <= 2 {
                normalized.push('\n');
            }
        } else {
            empty_count = 0;
            normalized.push_str(trimmed);
            normalized.push('\n');
        }
    }
    if !normalized.ends_with("\n") {
        normalized.push('\n');
    }

    fs::write(md_path, normalized).await?;
    Ok(())
}

pub async fn prepare_folder_path(
    blog_name: &str,
    post_title: &str,
    created_at: i64,
) -> Result<PathBuf> {
    let safe_post_title = sanitize_name(post_title);

    let datetime: DateTime<Utc> =
        DateTime::from_timestamp(created_at, 0).context("Invalid timestamp in post_created_at")?;

    let date_str = datetime.format("%Y.%m.%d").to_string();
    let folder_name = format!("{date_str} {safe_post_title}");

    let post_folder_path: PathBuf = ensure_post_folder(blog_name, &folder_name)
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
