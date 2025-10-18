use crate::{cli, content_items_handler, file_handler};
use anyhow::{Context, Result};
use boosty_api::api_response::Post;
use boosty_api::traits::{HasContent, HasTitle, IsAvailable};
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

    let post_title = &post.safe_title();
    let blog_name = &post.user.blog_url;

    let post_folder_path: PathBuf =
        file_handler::prepare_folder_path(blog_name, post_title, post.created_at).await?;

    let items = post.extract_content();

    content_items_handler::process_content_items(
        items,
        post_title,
        &post_folder_path,
        Some(&post.signed_query),
    )
    .await?;

    file_handler::normalize_md_file(&post_folder_path, post_title)
        .await
        .with_context(|| format!("Failed to normalize '{post_title}.md'"))?;

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
