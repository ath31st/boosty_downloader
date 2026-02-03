use crate::progress_reporter::ProgressReporter;
use crate::{headers, log_error, log_info, log_warn};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use comrak::Options;
use comrak::options::{Extension, Parse, Render};
use futures_util::StreamExt;
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
    let hashes_path = folder_path.join(".hashes");

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

    let existing = load_existing_hashes(&hashes_path).await?;
    let hash = hash_str(content);
    if existing.contains(&hash) {
        return Ok(DownloadResult::Skipped);
    }

    file.write_all(content.as_bytes())
        .await
        .with_context(|| format!("Failed to write to file '{}'", output_path.display()))?;
    file.write_all(b"\n").await?;

    append_hash_to_file(&hashes_path, &hash).await?;

    Ok(DownloadResult::Success)
}

async fn download_file_content(
    folder_path: &Path,
    url: &str,
    title: &str,
    signed_query: Option<&str>,
) -> Result<DownloadResult> {
    log_info!("Downloading file '{title}'...");
    for attempt in 1..=MAX_RETRIES {
        match download_file_once(folder_path, url, title, signed_query).await {
            Ok(r @ DownloadResult::Success) => return Ok(r),
            Ok(r @ DownloadResult::Skipped) => return Ok(r),
            Ok(_r @ DownloadResult::Error(_)) if attempt < MAX_RETRIES => {
                log_warn!("Download attempt {attempt} failed (logical error), retrying...");
            }
            Err(e) if attempt < MAX_RETRIES => {
                log_error!("Download attempt {attempt} failed with error: {e}");
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

    let reporter = ProgressReporter::new(total_size)?;

    let mut file = fs::File::create(&output_path)
        .await
        .with_context(|| format!("Failed to create file '{}'", output_path.display()))?;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.with_context(|| format!("Error while reading chunk from '{url}'"))?;
        file.write_all(&chunk).await?;
        reporter.inc(chunk.len() as u64);
    }
    reporter.finish();

    Ok(DownloadResult::Success)
}

pub async fn normalize_md_file(folder_path: &Path, title: &str) -> Result<()> {
    let md_path = folder_path.join(format!("{}.md", sanitize_name(title)));
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

pub async fn process_file_and_markdown(
    folder_path: &Path,
    url: &str,
    file_name: &str,
    markdown_content: &str,
    post_title: &str,
    signed_query: Option<&str>,
) -> Result<DownloadResult> {
    download_file_content(folder_path, url, file_name, signed_query)
        .await
        .with_context(|| {
            format!("Failed to download file '{file_name}' for post '{post_title}'")
        })?;

    let full_path = folder_path.join(file_name);
    let rel = full_path
        .strip_prefix(folder_path)
        .unwrap_or(&full_path)
        .to_string_lossy()
        .replace('\\', "/");

    let markdown = markdown_content.replace("{rel}", &rel);

    let download_res = download_text_content(folder_path, post_title, &markdown, None)
        .await
        .with_context(|| {
            format!("Failed to add markdown for '{file_name}' in post '{post_title}'")
        })?;

    Ok(download_res)
}

pub async fn convert_markdown_file_to_html(folder_path: &Path, title: &str) -> Result<()> {
    let md_path = folder_path.join(format!("{}.md", sanitize_name(title)));
    if !fs::try_exists(&md_path).await? {
        return Ok(());
    }

    let content = fs::read_to_string(&md_path).await?;

    let options = Options {
        extension: Extension {
            strikethrough: true,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: true,
            footnotes: true,
            description_lists: true,
            ..Default::default()
        },
        parse: Parse {
            smart: true,
            ..Default::default()
        },
        render: Render {
            r#unsafe: true,
            ..Default::default()
        },
    };

    let html_content = comrak::markdown_to_html(&content, &options);

    let template = include_str!("../../templates/template.html");
    let styled_html = template.replace("{content}", &html_content);

    let html_path = md_path.with_extension("html");
    fs::write(&html_path, styled_html)
        .await
        .with_context(|| format!("Failed to write HTML file '{}'", html_path.display()))?;

    Ok(())
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
