use crate::file_handler::normalize_md_file;
use crate::{cli, file_handler, parser};
use anyhow::{Context, Result};
use boosty_api::api_response::Post;
use boosty_api::post_data_extractor::ContentItem;
use std::path::PathBuf;

pub enum PostsResult {
    Multiple(Vec<Post>),
    Single(Post),
}

pub async fn process_posts(result: PostsResult) -> Result<()> {
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
            ContentItem::Image { url, id } => {
                let image_name = format!("{}.jpg", id);
                let download_res =
                    file_handler::download_file_content(&post_folder, &url, &image_name, None)
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to download image '{}' for post '{}'",
                                id, post_title
                            )
                        })?;
                cli::show_download_result(download_res, &id, post_title);
            }
            ContentItem::Video { url } => {
                let download_res =
                    file_handler::download_text_content(&post_folder, post_title, &url, None)
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to download video url '{}' for post '{}'",
                                url, post_title
                            )
                        })?;
                cli::show_download_result(download_res, post_title, post_title);
            }
            ContentItem::OkVideo { url, title } => {
                let download_res =
                    file_handler::download_file_content(&post_folder, &url, &title, None)
                        .await
                        .with_context(|| {
                            format!(
                                "Failed to download video '{}' for post '{}'",
                                title, post_title
                            )
                        })?;
                cli::show_download_result(download_res, &title, &post_title);
            }
            ContentItem::Audio { url, title, .. } | ContentItem::File { url, title, .. } => {
                let download_res = file_handler::download_file_content(
                    &post_folder,
                    &url,
                    &title,
                    Some(&post.signed_query),
                )
                .await
                .with_context(|| {
                    format!(
                        "Failed to download file '{}' for post '{}'",
                        title, post_title
                    )
                })?;
                cli::show_download_result(download_res, &title, &post_title);
            }
            ContentItem::Text {
                modificator,
                content,
            } => {
                if let Some(parsed) = parser::parse_text_content(&content, &modificator) {
                    let download_res = file_handler::download_text_content(
                        &post_folder,
                        post_title,
                        &parsed,
                        Some(&modificator),
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to download text '{}' for post '{}'",
                            content, post_title
                        )
                    })?;
                    cli::show_download_result(download_res, post_title, post_title);
                }
            }
            ContentItem::Link { content, url, .. } => {
                if let Some(parsed) = parser::parse_link_content(&content, &url) {
                    let download_res = file_handler::download_text_content(
                        &post_folder,
                        post_title,
                        &parsed,
                        None,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to download link '{}' for post '{}'",
                            url, post_title
                        )
                    })?;
                    cli::show_download_result(download_res, post_title, post_title);
                }
            }
            ContentItem::Unknown => cli::unknown_content_item(),
        }
    }

    normalize_md_file(&post_folder, post_title)
        .await
        .with_context(|| format!("Failed to normalize '{}.md'", post_title))?;

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
