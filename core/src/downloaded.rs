use anyhow::{Context, Result};
use boosty_api::api_client::ApiClient;
use boosty_api::error::ApiError;
use boosty_api::model::Post;
use boosty_api::traits::{HasContent, HasTitle, IsAvailable};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use tokio::fs;
use tokio_util::sync::CancellationToken;

use crate::blog_index::{self, BlogIndex, PostRecord};
use crate::config::{AppConfig, get_download_path};
use crate::download_options::{DownloadOption, DownloadOptions, options_in_items, ordered_options};
use crate::file_handler;
use crate::log_error;
use crate::menu_handler;
use crate::parser::BoostyUrl;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostSyncStatus {
    UpToDate,
    New,
    Updated,
    Partial,
    Unavailable,
    Gone,
    Unchecked,
}

impl PostSyncStatus {
    pub fn as_label(self) -> &'static str {
        match self {
            Self::UpToDate => "up to date",
            Self::New => "new",
            Self::Updated => "updated",
            Self::Partial => "partial",
            Self::Unavailable => "unavailable",
            Self::Gone => "gone",
            Self::Unchecked => "unchecked",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostSnapshot {
    pub post_id: String,
    pub title: String,
    pub folder: String,
    pub folder_path: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub downloaded_options: Vec<DownloadOption>,
    pub status: PostSyncStatus,
    pub is_paid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogSnapshot {
    pub blog: String,
    pub last_checked_at: Option<i64>,
    pub posts: Vec<PostSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadPostsResult {
    pub downloaded: u32,
    pub skipped: u32,
}

fn is_paid_post(post: &Post) -> bool {
    post.price > 0.0
}

pub async fn scan(download_path: &Path) -> Result<Vec<BlogSnapshot>> {
    let names = blog_index::list_blog_names(download_path).await?;
    let mut blogs = Vec::new();
    for name in names {
        let mut index = blog_index::load(download_path, &name).await?;
        if blog_index::prune_missing_folders(download_path, &mut index).await? {
            blog_index::save(download_path, &index).await?;
        }
        blogs.push(snapshot_from_index(
            download_path,
            &index,
            PostSyncStatus::Unchecked,
        ));
    }
    Ok(blogs)
}

pub async fn refresh_blog(
    client: &ApiClient,
    cfg: &AppConfig,
    blog: &str,
    cancel_token: &CancellationToken,
) -> Result<BlogSnapshot> {
    let download_path = get_download_path(cfg);
    let mut index = blog_index::load(&download_path, blog).await?;
    if blog_index::prune_missing_folders(&download_path, &mut index).await? {
        blog_index::save(&download_path, &index).await?;
    }

    let now = Utc::now().timestamp();
    let mut posts = Vec::new();
    let mut seen_ids = HashSet::new();

    let local_posts: Vec<(String, PostRecord)> = index
        .posts
        .iter()
        .map(|(id, rec)| (id.clone(), rec.clone()))
        .collect();

    for (post_id, record) in &local_posts {
        crate::ensure_not_cancelled(cancel_token)?;
        seen_ids.insert(post_id.clone());
        let status = match client.get_post(blog, post_id).await {
            Ok(post) => {
                let status = classify_existing(record, &post);
                if let Some(stored) = index.posts.get_mut(post_id) {
                    stored.last_checked_at = Some(now);
                    stored.title = post.safe_title();
                    stored.is_paid = is_paid_post(&post);
                }
                let mut snap = snapshot_from_record(&download_path, blog, post_id, record, status);
                snap.title = post.safe_title();
                snap.is_paid = is_paid_post(&post);
                posts.push(snap);
                continue;
            }
            Err(ApiError::HttpStatus { status, .. }) if status.as_u16() == 404 => {
                PostSyncStatus::Gone
            }
            Err(ApiError::Unauthorized) => {
                return Err(anyhow::anyhow!(
                    "Unauthorized when checking post '{post_id}'"
                ));
            }
            Err(e) => {
                log_error!("Failed to check post '{post_id}': {e:#}");
                PostSyncStatus::Unchecked
            }
        };
        posts.push(snapshot_from_record(
            &download_path,
            blog,
            post_id,
            record,
            status,
        ));
    }

    crate::ensure_not_cancelled(cancel_token)?;
    match client.get_posts(blog, cfg.posts_limit, None, None).await {
        Ok(feed) => {
            for post in feed {
                if seen_ids.contains(&post.id) {
                    continue;
                }
                seen_ids.insert(post.id.clone());
                let status = if post.not_available() {
                    PostSyncStatus::Unavailable
                } else {
                    PostSyncStatus::New
                };
                posts.push(PostSnapshot {
                    post_id: post.id.clone(),
                    title: post.safe_title(),
                    folder: String::new(),
                    folder_path: String::new(),
                    created_at: post.created_at,
                    updated_at: post.updated_at,
                    downloaded_options: Vec::new(),
                    status,
                    is_paid: is_paid_post(&post),
                });
            }
        }
        Err(ApiError::Unauthorized) => {
            return Err(anyhow::anyhow!(
                "Unauthorized when fetching posts for '{blog}'"
            ));
        }
        Err(e) => {
            log_error!("Failed to fetch posts for blog '{blog}': {e:#}");
        }
    }

    index.last_checked_at = Some(now);
    blog_index::save(&download_path, &index).await?;

    posts.sort_by_key(|b| std::cmp::Reverse(b.created_at));
    Ok(BlogSnapshot {
        blog: blog.to_string(),
        last_checked_at: index.last_checked_at,
        posts,
    })
}

pub async fn download_posts(
    client: &ApiClient,
    cfg: &AppConfig,
    blog: &str,
    post_ids: &[String],
    download_options: DownloadOptions,
    force: bool,
    cancel_token: &CancellationToken,
) -> Result<DownloadPostsResult> {
    let download_path = get_download_path(cfg);
    let mut result = DownloadPostsResult::default();
    for post_id in post_ids {
        crate::ensure_not_cancelled(cancel_token)?;
        if force
            && let Some(folder) =
                blog_index::resolve_post_folder(&download_path, blog, post_id).await?
        {
            file_handler::clear_dir_contents(&folder).await?;
        }
        let downloaded = menu_handler::process_boosty_url(
            client,
            cfg,
            &BoostyUrl::Post {
                blog: blog.to_string(),
                post_id: post_id.clone(),
            },
            None,
            download_options.clone(),
            cancel_token,
        )
        .await
        .with_context(|| format!("Failed to download post '{post_id}' of blog '{blog}'"))?;
        if downloaded == 0 {
            result.skipped += 1;
        } else {
            result.downloaded += downloaded as u32;
        }
    }
    Ok(result)
}

pub async fn delete_post(cfg: &AppConfig, blog: &str, post_id: &str) -> Result<()> {
    let download_path = get_download_path(cfg);
    if let Some(folder) = blog_index::resolve_post_folder(&download_path, blog, post_id).await? {
        fs::remove_dir_all(&folder)
            .await
            .with_context(|| format!("Failed to delete '{}'", folder.display()))?;
    }
    blog_index::remove_post(&download_path, blog, post_id).await?;
    Ok(())
}

pub async fn delete_blog(cfg: &AppConfig, blog: &str) -> Result<()> {
    let download_path = get_download_path(cfg);
    let blog_dir = download_path.join(blog);
    if fs::try_exists(&blog_dir).await.unwrap_or(false) {
        fs::remove_dir_all(&blog_dir)
            .await
            .with_context(|| format!("Failed to delete '{}'", blog_dir.display()))?;
    }
    Ok(())
}

pub fn classify_existing(local: &PostRecord, remote: &Post) -> PostSyncStatus {
    classify_from_fields(
        local.updated_at,
        &local.downloaded_options,
        remote.updated_at,
        remote.is_deleted,
        remote.not_available(),
        &options_in_items(&remote.extract_content()),
    )
}

pub fn classify_from_fields(
    local_updated_at: i64,
    local_options: &[DownloadOption],
    remote_updated_at: i64,
    remote_deleted: bool,
    remote_unavailable: bool,
    remote_options: &HashSet<DownloadOption>,
) -> PostSyncStatus {
    if remote_deleted {
        return PostSyncStatus::Gone;
    }
    if remote_unavailable {
        return PostSyncStatus::Unavailable;
    }
    if remote_updated_at > local_updated_at {
        return PostSyncStatus::Updated;
    }
    let local: HashSet<_> = local_options.iter().cloned().collect();
    if remote_options.iter().any(|t| !local.contains(t)) {
        return PostSyncStatus::Partial;
    }
    PostSyncStatus::UpToDate
}

fn snapshot_from_index(
    download_path: &Path,
    index: &BlogIndex,
    status: PostSyncStatus,
) -> BlogSnapshot {
    let mut posts: Vec<_> = index
        .posts
        .iter()
        .map(|(id, rec)| snapshot_from_record(download_path, &index.blog, id, rec, status))
        .collect();
    posts.sort_by_key(|b| std::cmp::Reverse(b.created_at));
    BlogSnapshot {
        blog: index.blog.clone(),
        last_checked_at: index.last_checked_at,
        posts,
    }
}

fn snapshot_from_record(
    download_path: &Path,
    blog: &str,
    post_id: &str,
    record: &PostRecord,
    status: PostSyncStatus,
) -> PostSnapshot {
    let folder_path = download_path.join(blog).join(&record.folder);
    PostSnapshot {
        post_id: post_id.to_string(),
        title: record.title.clone(),
        folder: record.folder.clone(),
        folder_path: folder_path.to_string_lossy().into_owned(),
        created_at: record.created_at,
        updated_at: record.updated_at,
        downloaded_options: ordered_options(record.downloaded_options.iter().cloned()),
        status,
        is_paid: record.is_paid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_updated_when_remote_newer() {
        let remote = HashSet::from([DownloadOption::Images]);
        let status = classify_from_fields(10, &[DownloadOption::Images], 20, false, false, &remote);
        assert_eq!(status, PostSyncStatus::Updated);
    }

    #[test]
    fn classify_partial_when_missing_types() {
        let remote = HashSet::from([DownloadOption::Images, DownloadOption::Video]);
        let status = classify_from_fields(10, &[DownloadOption::Images], 10, false, false, &remote);
        assert_eq!(status, PostSyncStatus::Partial);
    }

    #[test]
    fn classify_gone_and_unavailable() {
        let empty = HashSet::new();
        assert_eq!(
            classify_from_fields(1, &[], 1, true, false, &empty),
            PostSyncStatus::Gone
        );
        assert_eq!(
            classify_from_fields(1, &[], 1, false, true, &empty),
            PostSyncStatus::Unavailable
        );
    }

    #[test]
    fn classify_up_to_date() {
        let remote = HashSet::from([DownloadOption::Images]);
        let status = classify_from_fields(
            10,
            &[DownloadOption::Images, DownloadOption::Video],
            10,
            false,
            false,
            &remote,
        );
        assert_eq!(status, PostSyncStatus::UpToDate);
    }
}
