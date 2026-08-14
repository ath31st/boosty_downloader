use crate::{
    DownloadOptions, cli, content_items_handler, download_options, file_handler, log_error,
    post_page, progress_reporter,
};
use anyhow::Result;
use boosty_api::model::Post;
use boosty_api::traits::{HasContent, HasTitle, IsAvailable};
use post_page::PostPage;
use std::path::Path;
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
) -> Result<Vec<PostPage>> {
    let mut pages = Vec::new();
    match result {
        PostsResult::Multiple(posts) => {
            for post in posts {
                crate::ensure_not_cancelled(cancel_token)?;
                match process(&post, download_path, download_options.clone(), cancel_token).await {
                    Ok(Some(page)) => pages.push(page),
                    Ok(None) => {}
                    Err(e) => {
                        if crate::is_cancelled_error(&e) {
                            return Err(e);
                        }
                        log_error!("Error processing post '{}': {:#}", post.safe_title(), e);
                    }
                }
            }
        }
        PostsResult::Single(post) => {
            crate::ensure_not_cancelled(cancel_token)?;
            match process(&post, download_path, download_options, cancel_token).await {
                Ok(Some(page)) => pages.push(page),
                Ok(None) => {}
                Err(e) => {
                    if crate::is_cancelled_error(&e) {
                        return Err(e);
                    }
                    log_error!("Error processing post '{}': {:#}", post.safe_title(), e);
                }
            }
        }
    }
    Ok(pages)
}

async fn process(
    post: &Post,
    download_path: &Path,
    download_options: DownloadOptions,
    cancel_token: &CancellationToken,
) -> Result<Option<PostPage>> {
    crate::ensure_not_cancelled(cancel_token)?;
    if !check_available_post(post) {
        return Ok(None);
    }

    let post_title = post.safe_title();
    let blog_name = &post.user.blog_url;

    let post_folder_path = file_handler::prepare_folder_path(
        blog_name,
        &post_title,
        post.created_at,
        &post.id,
        download_path,
    )
    .await?;

    let items = post.extract_content();
    let filtered_items = download_options::filter_content_items(items, &download_options);

    let body = content_items_handler::process_content_items(
        filtered_items,
        &post_title,
        &post_folder_path,
        "",
        Some(&post.signed_query),
        cancel_token,
    )
    .await?;

    let page = PostPage {
        folder: post_folder_path,
        post_id: post.id.clone(),
        title: post_title,
        created_at: post.created_at,
        author: post.user.name.clone(),
        blog: post.user.blog_url.clone(),
        tags: post.tags.iter().map(|t| t.title.clone()).collect(),
        body,
        comments: Vec::new(),
    };

    post_page::write_post_page(&page).await?;

    Ok(Some(page))
}

fn check_available_post(post: &Post) -> bool {
    if post.not_available() {
        cli::post_not_available_or_without_content(&post.safe_title());
        false
    } else {
        true
    }
}
