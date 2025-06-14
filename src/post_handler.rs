use crate::{cli, file_handler};
use anyhow::{Context, Result};
use roosty_downloader_api::api_response::Post;
use roosty_downloader_api::post_data_extractor::ContentItem;
use std::path::PathBuf;

pub enum PostsResult {
    Multiple(Vec<Post>),
    Single(Post),
}

pub async fn post_processor(result: PostsResult) -> Result<()> {
    match result {
        PostsResult::Multiple(posts) => {
            for post in posts {
                process(&post)
                    .await
                    .with_context(|| format!("Error processing post '{}'", post.title))?;
            }
        }
        PostsResult::Single(post) => {
            process(&post)
                .await
                .with_context(|| format!("Error processing post '{}'", post.title))?;
        }
    }
    Ok(())
}

async fn process(post: &Post) -> Result<()> {
    if !check_available_post(post) {
        return Ok(());
    }

    let blog_name = &post.user.blog_url;
    let post_title = &post.title;
    let post_folder: PathBuf = file_handler::ensure_post_folder(blog_name, post_title)
        .await
        .with_context(|| {
            format!(
                "Failed to create folder for post '{}' in blog '{}'",
                post_title, blog_name
            )
        })?;
    let items = post.extract_content();

    for item in items {
        match item {
            ContentItem::Image {
                post_title: _,
                url,
                id,
            } => {
                let download_res = file_handler::download_image_content(&post_folder, &url, &id)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to download image '{}' for post '{}'",
                            id, post_title
                        )
                    })?;
                cli::show_download_result(download_res, &id, post_title);
            }
            ContentItem::Video {
                post_title,
                url,
                video_title,
            } => {
                let download_res =
                    file_handler::download_video_content(&post_folder, &url, &video_title)
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to download video '{}' for post '{}'",
                                video_title, post_title
                            )
                        })?;
                cli::show_download_result(download_res, &video_title, &post_title);
            }
        }
    }
    Ok(())
}

fn check_available_post(post: &Post) -> bool {
    if !post.has_access || post.data.is_empty() {
        cli::post_not_available_or_without_content(&post.title);
        false
    } else {
        true
    }
}
