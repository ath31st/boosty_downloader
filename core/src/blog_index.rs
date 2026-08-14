use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::download_options::{DownloadOption, DownloadOptions, ordered_options};

pub const SIDECAR_NAME: &str = ".boosty.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlogIndex {
    pub blog: String,
    #[serde(default)]
    pub last_checked_at: Option<i64>,
    #[serde(default)]
    pub posts: HashMap<String, PostRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostRecord {
    pub title: String,
    pub folder: String,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub downloaded_options: Vec<DownloadOption>,
    #[serde(default)]
    pub last_checked_at: Option<i64>,
    #[serde(default)]
    pub is_paid: bool,
}

pub fn sidecar_path(download_path: &Path, blog: &str) -> PathBuf {
    download_path.join(blog).join(SIDECAR_NAME)
}

pub async fn load(download_path: &Path, blog: &str) -> Result<BlogIndex> {
    let path = sidecar_path(download_path, blog);
    if !fs::try_exists(&path).await.unwrap_or(false) {
        return Ok(BlogIndex {
            blog: blog.to_string(),
            ..BlogIndex::default()
        });
    }

    let data = fs::read(&path)
        .await
        .with_context(|| format!("Failed to read blog index '{}'", path.display()))?;
    let mut index: BlogIndex = serde_json::from_slice(&data)
        .with_context(|| format!("Failed to parse blog index '{}'", path.display()))?;
    if index.blog.is_empty() {
        index.blog = blog.to_string();
    }
    Ok(index)
}

pub async fn save(download_path: &Path, index: &BlogIndex) -> Result<()> {
    let blog_dir = download_path.join(&index.blog);
    fs::create_dir_all(&blog_dir)
        .await
        .with_context(|| format!("Failed to create blog folder '{}'", blog_dir.display()))?;
    let path = sidecar_path(download_path, &index.blog);
    let data = serde_json::to_vec_pretty(index).context("Failed to serialize blog index")?;
    fs::write(&path, data)
        .await
        .with_context(|| format!("Failed to write blog index '{}'", path.display()))?;
    Ok(())
}

pub async fn resolve_post_folder(
    download_path: &Path,
    blog: &str,
    post_id: &str,
) -> Result<Option<PathBuf>> {
    let index = load(download_path, blog).await?;
    let Some(record) = index.posts.get(post_id) else {
        return Ok(None);
    };
    let folder = download_path.join(blog).join(&record.folder);
    if fs::try_exists(&folder).await.unwrap_or(false) {
        Ok(Some(folder))
    } else {
        Ok(None)
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert_post(
    download_path: &Path,
    blog: &str,
    post_id: &str,
    title: &str,
    folder_name: &str,
    created_at: i64,
    updated_at: i64,
    downloaded_options: &DownloadOptions,
    is_paid: bool,
) -> Result<()> {
    let mut index = load(download_path, blog).await?;
    index.blog = blog.to_string();

    let merged = match index.posts.get(post_id) {
        Some(existing) => {
            let mut set: std::collections::HashSet<_> =
                existing.downloaded_options.iter().cloned().collect();
            set.extend(downloaded_options.iter().cloned());
            ordered_options(set)
        }
        None => ordered_options(downloaded_options.iter().cloned()),
    };

    index.posts.insert(
        post_id.to_string(),
        PostRecord {
            title: title.to_string(),
            folder: folder_name.to_string(),
            created_at,
            updated_at,
            downloaded_options: merged,
            last_checked_at: Some(Utc::now().timestamp()),
            is_paid,
        },
    );

    save(download_path, &index).await
}

pub async fn remove_post(download_path: &Path, blog: &str, post_id: &str) -> Result<()> {
    let mut index = load(download_path, blog).await?;
    index.posts.remove(post_id);
    if index.posts.is_empty() {
        let path = sidecar_path(download_path, blog);
        if fs::try_exists(&path).await.unwrap_or(false) {
            fs::remove_file(&path).await.ok();
        }
        let blog_dir = download_path.join(blog);
        if let Ok(mut rd) = fs::read_dir(&blog_dir).await
            && rd.next_entry().await.ok().flatten().is_none()
        {
            let _ = fs::remove_dir(&blog_dir).await;
        }
        return Ok(());
    }
    save(download_path, &index).await
}

pub async fn prune_missing_folders(download_path: &Path, index: &mut BlogIndex) -> Result<bool> {
    let blog_dir = download_path.join(&index.blog);
    let before = index.posts.len();
    let mut keep = HashMap::new();
    for (id, record) in index.posts.drain() {
        let folder = blog_dir.join(&record.folder);
        if fs::try_exists(&folder).await.unwrap_or(false) {
            keep.insert(id, record);
        }
    }
    index.posts = keep;
    Ok(index.posts.len() != before)
}

pub async fn list_blog_names(download_path: &Path) -> Result<Vec<String>> {
    if !fs::try_exists(download_path).await.unwrap_or(false) {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    let mut rd = fs::read_dir(download_path)
        .await
        .with_context(|| format!("Failed to read download path '{}'", download_path.display()))?;
    while let Some(entry) = rd.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let sidecar = path.join(SIDECAR_NAME);
        if fs::try_exists(&sidecar).await.unwrap_or(false)
            && let Some(name) = path.file_name().and_then(|s| s.to_str())
        {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_base() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p =
            std::env::temp_dir().join(format!("boosty_blog_index_{}_{nanos}", std::process::id()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[tokio::test]
    async fn upsert_and_resolve_reuses_folder() {
        let base = temp_base();
        let blog = "author";
        let post_id = "post-1";
        let folder = "2024.01.15 Title [post-1]";
        let post_dir = base.join(blog).join(folder);
        fs::create_dir_all(&post_dir).await.unwrap();

        let opts: DownloadOptions = Arc::new(HashSet::from([DownloadOption::Images]));
        upsert_post(&base, blog, post_id, "Title", folder, 1, 10, &opts, false)
            .await
            .unwrap();

        let resolved = resolve_post_folder(&base, blog, post_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(resolved, post_dir);

        let more: DownloadOptions = Arc::new(HashSet::from([DownloadOption::Video]));
        upsert_post(
            &base,
            blog,
            post_id,
            "New title",
            folder,
            1,
            20,
            &more,
            true,
        )
        .await
        .unwrap();
        let index = load(&base, blog).await.unwrap();
        let rec = &index.posts[post_id];
        assert_eq!(rec.title, "New title");
        assert_eq!(rec.updated_at, 20);
        assert_eq!(rec.folder, folder);
        assert!(rec.is_paid);
        assert_eq!(
            rec.downloaded_options,
            vec![DownloadOption::Video, DownloadOption::Images]
        );

        let _ = fs::remove_dir_all(&base).await;
    }

    #[tokio::test]
    async fn prune_drops_missing_post_folders() {
        let base = temp_base();
        let blog = "author";
        let folder = "gone [id]";
        let opts: DownloadOptions = Arc::new(HashSet::new());
        upsert_post(&base, blog, "id", "t", folder, 1, 1, &opts, false)
            .await
            .unwrap();

        let mut index = load(&base, blog).await.unwrap();
        let changed = prune_missing_folders(&base, &mut index).await.unwrap();
        assert!(changed);
        assert!(index.posts.is_empty());

        let _ = fs::remove_dir_all(&base).await;
    }
}
