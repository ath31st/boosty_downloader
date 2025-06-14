use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{Instant, sleep};

pub enum DownloadResult {
    Error(String),
    Success,
    Skipped,
}

pub async fn ensure_blog_folder(blog_name: &str) -> Result<PathBuf> {
    let blog_path = Path::new(blog_name);
    let exists = fs::try_exists(blog_path)
        .await
        .with_context(|| format!("Failed to check if blog folder '{}' exists", blog_name))?;
    if !exists {
        fs::create_dir_all(blog_path)
            .await
            .with_context(|| format!("Failed to create blog folder '{}'", blog_name))?;
    }
    Ok(blog_path.to_path_buf())
}

pub async fn ensure_post_folder(blog_name: &str, post_id: &str) -> Result<PathBuf> {
    let blog_path = ensure_blog_folder(blog_name).await?;
    let post_path = blog_path.join(post_id);
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

// pub async fn download_video_content1(
//     folder_path: &PathBuf,
//     video_url: &str,
//     video_title: &str,
// ) -> Result<DownloadResult> {
//     let output_path = folder_path.join(format!("{}.mp4", video_title));
//
//     if fs::try_exists(&output_path).await.unwrap_or(false) {
//         return Ok(DownloadResult::Skipped);
//     }
//
//     let status = Command::new("ffmpeg")
//         .arg("-i")
//         .arg(video_url)
//         .arg("-c")
//         .arg("copy")
//         .arg(output_path.to_string_lossy().to_string())
//         .status()
//         .await?;
//
//     if status.success() {
//         Ok(DownloadResult::Success)
//     } else {
//         Ok(DownloadResult::Error("ffmpeg failed".into()))
//     }
// }
//
// pub async fn download_image_content1(
//     post_folder: &Path,
//     image_url: &str,
//     image_name: &str,
// ) -> Result<DownloadResult> {
//     let output_path = post_folder.join(format!("{}.jpg", image_name));
//
//     if fs::try_exists(&output_path).await.unwrap_or(false) {
//         return Ok(DownloadResult::Skipped);
//     }
//
//     let resp = reqwest::get(image_url).await?;
//     if !resp.status().is_success() {
//         return Ok(DownloadResult::Error(format!("HTTP {}", resp.status())));
//     }
//
//     let bytes = resp.bytes().await?;
//     fs::write(&output_path, &bytes).await?;
//
//     Ok(DownloadResult::Success)
// }

pub async fn download_image_content(
    post_folder: &Path,
    image_url: &str,
    image_name: &str,
) -> Result<DownloadResult> {
    let safe_name = sanitize_filename(image_name);
    let output_path = post_folder.join(format!("{}.jpg", safe_name));

    let exists = fs::try_exists(&output_path).await.with_context(|| {
        format!(
            "Failed to check existence of image file '{}'",
            output_path.display()
        )
    })?;
    if exists {
        return Ok(DownloadResult::Skipped);
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(image_url)
        .send()
        .await
        .with_context(|| format!("HTTP GET failed for image URL '{}'", image_url))?;
    if !resp.status().is_success() {
        return Ok(DownloadResult::Error(format!("HTTP {}", resp.status())));
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
            "{spinner:.green} Downloading image... {bytes}",
        )?);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    };

    let mut file = fs::File::create(&output_path)
        .await
        .with_context(|| format!("Failed to create file '{}'", output_path.display()))?;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.with_context(|| format!("Error while reading chunk from '{}'", image_url))?;
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }
    pb.finish_and_clear();

    Ok(DownloadResult::Success)
}

pub async fn download_video_content(
    folder_path: &Path,
    video_url: &str,
    video_title: &str,
) -> Result<DownloadResult> {
    let safe_name = sanitize_filename(video_title);
    let output_path = folder_path.join(format!("{}.mp4", safe_name));

    let exists = fs::try_exists(&output_path).await.with_context(|| {
        format!(
            "Failed to check existence of video file '{}'",
            output_path.display()
        )
    })?;
    if exists {
        return Ok(DownloadResult::Skipped);
    }

    let pb = ProgressBar::new_spinner();
    pb.set_prefix(format!("Downloading '{}'", video_title));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {prefix}... Elapsed {msg}")?
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    let mut child = Command::new("ffmpeg")
        .arg("-i")
        .arg(video_url)
        .arg("-c")
        .arg("copy")
        .arg(output_path.to_string_lossy().to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to spawn ffmpeg for URL '{}'", video_url))?;

    let start = Instant::now();
    loop {
        match child
            .try_wait()
            .with_context(|| "Error while waiting for ffmpeg process")?
        {
            Some(status) => {
                pb.finish_and_clear();
                return if status.success() {
                    Ok(DownloadResult::Success)
                } else {
                    Ok(DownloadResult::Error("ffmpeg failed".into()))
                };
            }
            None => {
                let secs = start.elapsed().as_secs();
                pb.set_message(format!("{}s", secs));
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
