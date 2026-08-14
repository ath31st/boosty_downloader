use std::path::Path;

use anyhow::{Context, Result};
use boosty_api::{
    media_content::ContentItem,
    model::Comment,
    traits::{HasContent, IsAvailable},
};
use tokio_util::sync::CancellationToken;

use crate::{
    DownloadOptions, cli, content_items_handler, download_options, file_handler, log_error,
    post_page, progress_reporter,
};
use post_page::{CommentView, PostPage};

pub struct CommentsResult {
    pub comments: Vec<Comment>,
    pub post_id: String,
    pub safe_post_title: String,
}

pub async fn process_comments(
    results: Vec<CommentsResult>,
    pages: &mut [PostPage],
    download_options: DownloadOptions,
    cancel_token: &CancellationToken,
) -> Result<()> {
    if results.is_empty() {
        return Ok(());
    }

    let extra_files = count_downloadable_files(&results, &download_options);
    progress_reporter::add_files_total(extra_files);

    for result in results {
        crate::ensure_not_cancelled(cancel_token)?;
        let Some(page) = pages.iter_mut().find(|p| p.post_id == result.post_id) else {
            log_error!(
                "No downloaded post page for comments of '{}'",
                result.safe_post_title
            );
            continue;
        };

        if let Err(e) = process_one(page, &result, download_options.clone(), cancel_token).await {
            if crate::is_cancelled_error(&e) {
                return Err(e);
            }
            log_error!(
                "Error processing comments for post '{}': {:#}",
                result.safe_post_title,
                e
            );
        }
    }
    Ok(())
}

async fn process_one(
    page: &mut PostPage,
    result: &CommentsResult,
    download_options: DownloadOptions,
    cancel_token: &CancellationToken,
) -> Result<()> {
    crate::ensure_not_cancelled(cancel_token)?;

    process(page, result, download_options, cancel_token)
        .await
        .with_context(|| {
            format!(
                "Error processing comments for post '{}'",
                result.safe_post_title
            )
        })?;

    post_page::write_post_page(page).await?;

    Ok(())
}

async fn process(
    page: &mut PostPage,
    cr: &CommentsResult,
    download_options: DownloadOptions,
    cancel_token: &CancellationToken,
) -> Result<()> {
    crate::ensure_not_cancelled(cancel_token)?;
    if !check_available_comments(&cr.comments, &cr.safe_post_title) {
        return Ok(());
    }

    let comments_folder_path = file_handler::prepare_folder_path_for_comments(&page.folder).await?;

    let mut comments = Vec::new();
    for comment in cr.comments.iter().filter(|c| !c.not_available()) {
        collect_comment_views(
            comment,
            0,
            &page.title,
            &comments_folder_path,
            &download_options,
            cancel_token,
            &mut comments,
        )
        .await?;
    }

    page.comments = comments;
    Ok(())
}

async fn collect_comment_views(
    comment: &Comment,
    level: u8,
    post_title: &str,
    comments_folder_path: &Path,
    download_options: &DownloadOptions,
    cancel_token: &CancellationToken,
    out: &mut Vec<CommentView>,
) -> Result<()> {
    crate::ensure_not_cancelled(cancel_token)?;

    let items = comment.extract_content();
    let filtered = download_options::filter_content_items(items, download_options);
    let blocks = content_items_handler::process_content_items(
        filtered,
        post_title,
        comments_folder_path,
        "comments/",
        None,
        cancel_token,
    )
    .await?;

    out.push(CommentView {
        author: comment.author.name.clone(),
        created_at: comment.created_at as i64,
        level,
        blocks,
    });

    if let Some(replies) = &comment.replies {
        for reply in replies.data.iter().filter(|c| !c.not_available()) {
            Box::pin(collect_comment_views(
                reply,
                level.saturating_add(1),
                post_title,
                comments_folder_path,
                download_options,
                cancel_token,
                out,
            ))
            .await?;
        }
    }

    Ok(())
}

fn check_available_comments(comments: &[Comment], post_title: &str) -> bool {
    if comments.is_empty() || comments.iter().all(|c| c.not_available()) {
        cli::comments_for_post_empty_or_not_available(post_title);
        return false;
    }
    true
}

pub fn count_downloadable_files(
    results: &[CommentsResult],
    download_options: &DownloadOptions,
) -> u64 {
    results
        .iter()
        .map(|result| {
            if !check_available_for_count(&result.comments) {
                return 0;
            }
            let items: Vec<ContentItem> = result
                .comments
                .iter()
                .filter(|c| !c.not_available())
                .flat_map(collect_items_for_count)
                .collect();
            let filtered = download_options::filter_content_items(items, download_options);
            progress_reporter::count_downloadable_files(&filtered)
        })
        .sum()
}

fn check_available_for_count(comments: &[Comment]) -> bool {
    !(comments.is_empty() || comments.iter().all(|c| c.not_available()))
}

fn collect_items_for_count(comment: &Comment) -> Vec<ContentItem> {
    let mut items = comment.extract_content();
    if let Some(replies) = &comment.replies {
        for reply_group in replies.data.iter() {
            items.extend(collect_items_for_count(reply_group));
        }
    }
    items
}
