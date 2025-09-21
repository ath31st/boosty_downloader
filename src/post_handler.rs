use crate::file_handler::normalize_md_file;
use crate::{cli, content_items_handler, file_handler};
use anyhow::{Context, Result};
use boosty_api::api_response::Post;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

pub enum PostsResult {
    Multiple(Vec<Post>),
    Single(Box<Post>),
}

pub async fn process_posts(result: PostsResult) -> Result<()> {
    match result {
        PostsResult::Multiple(posts) => {
            for post in posts {
                process(&post)
                    .await
                    .with_context(|| format!("Error processing post '{}'", post.safe_title()))?;
            }
        }
        PostsResult::Single(post) => {
            process(&post)
                .await
                .with_context(|| format!("Error processing post '{}'", post.safe_title()))?;
        }
    }
    Ok(())
}

async fn process(post: &Post) -> Result<()> {
    if !check_available_post(post) {
        return Ok(());
    }

    let blog_name = &post.user.blog_url;
    let post_title = &post.safe_title();
    let safe_post_title = file_handler::sanitize_name(post_title);

    let created_at = post.created_at;
    let datetime: DateTime<Utc> =
        DateTime::from_timestamp(created_at, 0).context("Invalid timestamp in post.created_at")?;

    let date_str = datetime.format("%d.%m.%Y").to_string();
    let folder_name = format!("{date_str} {safe_post_title}");

    let post_folder: PathBuf = file_handler::ensure_post_folder(blog_name, &folder_name)
        .await
        .with_context(|| {
            format!("Failed to create folder for post '{post_title}' in blog '{blog_name}'")
        })?;
    let items = post.extract_content();

    content_items_handler::process_content_items(
        items,
        post_title,
        &post_folder,
        Some(&post.signed_query),
    )
    .await?;

    normalize_md_file(&post_folder, post_title)
        .await
        .with_context(|| format!("Failed to normalize '{post_title}.md'"))?;

    Ok(())
}

fn check_available_post(post: &Post) -> bool {
    if !post.has_access || post.data.is_empty() {
        cli::post_not_available_or_without_content(&post.safe_title());
        false
    } else {
        true
    }
}
