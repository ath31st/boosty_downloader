use crate::{
    DownloadOptions, cli, content_items_handler, download_options, file_handler, log_error,
    progress_reporter,
};
use anyhow::{Context, Result};
use boosty_api::model::Post;
use boosty_api::traits::{HasContent, HasTitle, IsAvailable};
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;

pub enum PostsResult {
    Multiple(Vec<Post>),
    Single(Box<Post>),
}

pub fn count_downloadable_files(result: &PostsResult, download_options: &DownloadOptions) -> u64 {
    match result {
        PostsResult::Multiple(posts) => posts
            .iter()
            .filter(|p| !p.not_available())
            .map(|p| count_post_files(p, download_options))
            .sum(),
        PostsResult::Single(post) => {
            if post.not_available() {
                0
            } else {
                count_post_files(post, download_options)
            }
        }
    }
}

fn count_post_files(post: &Post, download_options: &DownloadOptions) -> u64 {
    let items = post.extract_content();
    let filtered = download_options::filter_content_items(items, download_options);
    progress_reporter::count_downloadable_files(&filtered)
}

pub async fn process_posts(
    result: PostsResult,
    download_path: &Path,
    download_options: DownloadOptions,
    cancel_token: &CancellationToken,
) -> Result<()> {
    match result {
        PostsResult::Multiple(posts) => {
            for post in posts {
                crate::ensure_not_cancelled(cancel_token)?;
                if let Err(e) =
                    process(&post, download_path, download_options.clone(), cancel_token).await
                {
                    if crate::is_cancelled_error(&e) {
                        return Err(e);
                    }
                    log_error!("Error processing post '{}': {:#}", post.safe_title(), e);
                }
            }
        }
        PostsResult::Single(post) => {
            crate::ensure_not_cancelled(cancel_token)?;
            if let Err(e) = process(&post, download_path, download_options, cancel_token).await {
                if crate::is_cancelled_error(&e) {
                    return Err(e);
                }
                log_error!("Error processing post '{}': {:#}", post.safe_title(), e);
            }
        }
    }
    Ok(())
}

async fn process(
    post: &Post,
    download_path: &Path,
    download_options: DownloadOptions,
    cancel_token: &CancellationToken,
) -> Result<()> {
    crate::ensure_not_cancelled(cancel_token)?;
    if !check_available_post(post) {
        return Ok(());
    }

    let post_title = &post.safe_title();
    let blog_name = &post.user.blog_url;

    let post_folder_path: PathBuf =
        file_handler::prepare_folder_path(blog_name, post_title, post.created_at, download_path)
            .await?;

    let items = post.extract_content();
    let filtered_items = download_options::filter_content_items(items, &download_options);

    content_items_handler::process_content_items(
        filtered_items,
        post_title,
        &post_folder_path,
        Some(&post.signed_query),
        cancel_token,
    )
    .await?;

    file_handler::normalize_md_file(&post_folder_path, post_title)
        .await
        .with_context(|| format!("Failed to normalize '{post_title}.md'"))?;

    file_handler::convert_markdown_file_to_html(&post_folder_path, post_title)
        .await
        .with_context(|| format!("Failed to convert '{post_title}.md' to HTML"))?;

    Ok(())
}

fn check_available_post(post: &Post) -> bool {
    if post.not_available() {
        cli::post_not_available_or_without_content(&post.safe_title());
        false
    } else {
        true
    }
}
