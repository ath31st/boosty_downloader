use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;

pub enum DownloadResult {
    Error(String),
    Success,
    Skipped,
}

pub async fn ensure_blog_folder(blog_name: &str) -> Result<PathBuf> {
    let blog_path = Path::new(blog_name);
    if !fs::try_exists(blog_path).await.unwrap_or(false) {
        fs::create_dir_all(blog_path).await?;
    }
    Ok(blog_path.to_path_buf())
}

pub async fn ensure_post_folder(blog_name: &str, post_id: &str) -> Result<PathBuf> {
    let blog_path = ensure_blog_folder(blog_name).await?;
    let post_path = blog_path.join(post_id);
    if !fs::try_exists(&post_path).await.unwrap_or(false) {
        fs::create_dir_all(&post_path).await?;
    }
    Ok(post_path)
}

pub async fn download_video_content(
    folder_path: &PathBuf,
    video_url: &str,
    video_title: &str,
) -> Result<DownloadResult> {
    let output_path = folder_path.join(format!("{}.mp4", video_title));

    if fs::try_exists(&output_path).await.unwrap_or(false) {
        return Ok(DownloadResult::Skipped);
    }

    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(video_url)
        .arg("-c")
        .arg("copy")
        .arg(output_path.to_string_lossy().to_string())
        .status()
        .await?;

    if status.success() {
        Ok(DownloadResult::Success)
    } else {
        Ok(DownloadResult::Error("ffmpeg failed".into()))
    }
}

pub async fn download_image_content(
    post_folder: &Path,
    image_url: &str,
    image_name: &str,
) -> Result<DownloadResult> {
    let output_path = post_folder.join(format!("{}.jpg", image_name));

    if fs::try_exists(&output_path).await.unwrap_or(false) {
        return Ok(DownloadResult::Skipped);
    }

    let resp = reqwest::get(image_url).await?;
    if !resp.status().is_success() {
        return Ok(DownloadResult::Error(format!("HTTP {}", resp.status())));
    }

    let bytes = resp.bytes().await?;
    fs::write(&output_path, &bytes).await?;

    Ok(DownloadResult::Success)
}
