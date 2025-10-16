use std::path::PathBuf;

use anyhow::{Context, Result};
use boosty_api::api_response::CommentsResponse;

use crate::{cli, file_handler};

pub struct CommentsResult {
    pub response: CommentsResponse,
    pub blog_url: String,
    pub safe_post_title: String,
    pub created_at: i64,
}

pub async fn process_comments(results: Vec<CommentsResult>) -> Result<()> {
    for result in results {
        process(&result).await.with_context(|| {
            format!(
                "Error processing comments for post '{}'",
                result.safe_post_title
            )
        })?;
    }
    Ok(())
}

async fn process(cr: &CommentsResult) -> Result<()> {
    if !check_available_comments(&cr.response, &cr.safe_post_title) {
        return Ok(());
    }

    let post_folder_path: PathBuf =
        file_handler::prepare_folder_path(&cr.blog_url, &cr.safe_post_title, cr.created_at).await?;

    let comments_folder_path: PathBuf =
        file_handler::prepare_folder_path_for_comments(&post_folder_path).await?;
    Ok(())
}

fn check_available_comments(comments: &CommentsResponse, post_title: &str) -> bool {
    if comments.data.is_empty() {
        cli::comments_for_post_empty_or_not_available(post_title);
        return false;
    }
    true
}
