use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use boosty_api::{
    api_response::{Comment, CommentsResponse},
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
    if results.is_empty() {
        return Ok(());
    }

    for result in results {
        let post_title = &result.safe_post_title;

        let post_folder_path: PathBuf =
            file_handler::prepare_folder_path(&result.blog_url, post_title, result.created_at)
                .await?;

        let comments_folder_path: PathBuf =
            file_handler::prepare_folder_path_for_comments(&post_folder_path).await?;

        process(&result, &comments_folder_path, post_title)
            .await
            .with_context(|| {
                format!(
                    "Error processing comments for post '{}'",
                    result.safe_post_title
                )
            })?;

        file_handler::normalize_md_file(&comments_folder_path, post_title)
            .await
            .with_context(|| format!("Failed to normalize '{post_title}.md' for comments"))?;

        file_handler::convert_markdown_file_to_html(&comments_folder_path, post_title)
            .await
            .with_context(|| format!("Failed to convert '{post_title}.md' to HTML"))?;
    }
    Ok(())
}

async fn process(cr: &CommentsResult, comments_folder_path: &Path, post_title: &str) -> Result<()> {
    if !check_available_comments(&cr.response, &cr.safe_post_title) {
        return Ok(());
    }

    let items: Vec<ContentItem> = cr
        .response
        .data
        .iter()
        .filter(|c| !c.not_available())
        .flat_map(|c| collect_items_from_comment(c, 0))
        .collect();

    content_items_handler::process_content_items(items, post_title, comments_folder_path, None)
        .await?;

    Ok(())
}

fn modify_text_content(items: &mut Vec<ContentItem>, author: &str, created_at: u64, level: u8) {
    let indent = "↳".repeat(level.into());
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
                content.insert_str(insert_pos, &format!("{indent}[{date_str}][{author}] "));
            }
            _ => {}
        }
    }
}

fn collect_items_from_comment(comment: &Comment, level: u8) -> Vec<ContentItem> {
    let mut items = comment.extract_content();
    modify_text_content(&mut items, &comment.author.name, comment.created_at, level);

    if let Some(replies) = &comment.replies {
        for reply_group in replies.data.iter() {
            let reply_items = collect_items_from_comment(reply_group, level + 1);
            items.extend(reply_items);
        }
    }

    items
}

fn check_available_comments(comments: &CommentsResponse, post_title: &str) -> bool {
    if comments.data.is_empty() || comments.data.iter().all(|c| c.not_available()) {
        cli::comments_for_post_empty_or_not_available(post_title);
        return false;
    }
    true
}
