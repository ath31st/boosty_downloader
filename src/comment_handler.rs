use anyhow::{Context, Result};
use boosty_api::api_response::CommentsResponse;

use crate::cli;

pub struct CommentsResult {
    pub response: CommentsResponse,
    pub safe_post_title: String,
    pub blog_url: String,
}

pub async fn process_comments(results: Vec<CommentsResult>) -> Result<()> {
    for result in results {
        process(&result.response, &result.safe_post_title, &result.blog_url)
            .await
            .with_context(|| {
                format!(
                    "Error processing comments for post '{}'",
                    result.safe_post_title
                )
            })?;
    }
    Ok(())
}

async fn process(
    comment_response: &CommentsResponse,
    safe_post_title: &str,
    blog_url: &str,
) -> Result<()> {
    if !check_available_comments(comment_response, safe_post_title) {
        return Ok(());
    }
    Ok(())
}

fn check_available_comments(comments: &CommentsResponse, post_title: &str) -> bool {
    if comments.data.is_empty() {
        cli::comments_for_post_empty_or_not_available(post_title);
        return false;
    }
    true
}
