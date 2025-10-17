use std::path::PathBuf;

use anyhow::{Context, Result};
use boosty_api::{
    api_response::CommentsResponse,
    media_content::ContentItem,
    traits::{HasContent, IsAvailable},
};
use chrono::{DateTime, Utc};

use crate::{cli, content_items_handler, file_handler};

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

    let items: Vec<ContentItem> = cr
        .response
        .data
        .iter()
        .filter(|c| !c.not_available())
        .flat_map(|c| {
            let mut items = c.extract_content();
            modify_text_content(&mut items, &c.author.name, c.created_at);
            items
        })
        .collect();

    content_items_handler::process_content_items(
        items,
        &cr.safe_post_title,
        &comments_folder_path,
        None,
    )
    .await?;

    Ok(())
}

fn modify_text_content(items: &mut Vec<ContentItem>, author: &str, created_at: u64) {
    for item in items {
        match item {
            ContentItem::Text {
                content,
                modificator,
            } if modificator != "BLOCK_END"
                && !content.is_empty()
                && content.starts_with("[\"") =>
            {
                let datetime: DateTime<Utc> =
                    DateTime::from_timestamp(created_at as i64, 0).unwrap_or_default();

                let date_str = datetime.format("%Y.%m.%d %H:%M").to_string();
                let insert_pos = 2;
                content.insert_str(insert_pos, &format!("[{date_str}][{author}] "));
            }
            _ => {}
        }
    }
}

fn check_available_comments(comments: &CommentsResponse, post_title: &str) -> bool {
    if comments.data.is_empty() || comments.data.iter().all(|c| c.not_available()) {
        cli::comments_for_post_empty_or_not_available(post_title);
        return false;
    }
    true
}
