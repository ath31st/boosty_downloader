use crate::{cli, file_handler, parser};
use anyhow::{Context, Result};
use boosty_api::media_content::ContentItem;
use std::path::Path;

pub async fn process_content_items(
    items: Vec<ContentItem>,
    post_title: &str,
    folder_path: &Path,
    signed_query: Option<&str>,
) -> Result<()> {
    let mut stack = items;

    while let Some(item) = stack.pop() {
        match item {
            ContentItem::Image { url, id } => {
                let image_name = format!("{id}.jpg");
                let download_res =
                    file_handler::download_file_content(folder_path, &url, &image_name, None)
                        .await
                        .with_context(|| {
                            format!("Failed to download image '{id}' for post '{post_title}'")
                        })?;
                cli::show_download_result(download_res, &id, post_title);
            }
            ContentItem::Video { url } => {
                let download_res =
                    file_handler::download_text_content(folder_path, post_title, &url, None)
                        .await
                        .with_context(|| {
                            format!("Failed to download video url '{url}' for post '{post_title}'")
                        })?;
                cli::show_download_result(download_res, post_title, post_title);
            }
            ContentItem::OkVideo { url, title, vid } => {
                let title_with_vid = format!("{title}({vid})");
                let download_res = file_handler::download_file_content(
                    folder_path,
                    &url,
                    &title_with_vid,
                    None,
                )
                .await
                .with_context(|| {
                    format!("Failed to download video '{title_with_vid}' for post '{post_title}'")
                })?;
                cli::show_download_result(download_res, &title_with_vid, post_title);
            }
            ContentItem::Audio { url, title, .. } | ContentItem::File { url, title, .. } => {
                let download_res =
                    file_handler::download_file_content(folder_path, &url, &title, signed_query)
                        .await
                        .with_context(|| {
                            format!("Failed to download file '{title}' for post '{post_title}'")
                        })?;
                cli::show_download_result(download_res, &title, post_title);
            }
            ContentItem::Text {
                modificator,
                content,
            } => {
                if let Some(parsed) = parser::parse_text_content(&content, &modificator) {
                    let download_res = file_handler::download_text_content(
                        folder_path,
                        post_title,
                        &parsed,
                        Some(&modificator),
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to download text '{content}' for post '{post_title}'")
                    })?;
                    cli::show_download_result(download_res, post_title, post_title);
                }
            }
            ContentItem::Smile {
                small_url, name, ..
            } => {
                let image_name = format!("{name}.png");
                file_handler::download_file_content(folder_path, &small_url, &image_name, None)
                    .await
                    .with_context(|| {
                        format!("Failed to download smile '{name}' for post '{post_title}'")
                    })?;

                let full_image_path = folder_path.join(&image_name);
                let rel = full_image_path
                    .strip_prefix(folder_path)
                    .unwrap_or(&full_image_path)
                    .to_string_lossy()
                    .replace('\\', "/");

                let smile_content = format!("![{name}]({rel})");
                let download_res = file_handler::download_text_content(
                    folder_path,
                    post_title,
                    &smile_content,
                    None,
                )
                .await
                .with_context(|| {
                    format!("Failed to download smile '{name}' for post '{post_title}'")
                })?;
                cli::show_download_result(download_res, &name, post_title);
            }
            ContentItem::Link { content, url, .. } => {
                if let Some(parsed) = parser::parse_link_content(&content, &url) {
                    let download_res =
                        file_handler::download_text_content(folder_path, post_title, &parsed, None)
                            .await
                            .with_context(|| {
                                format!("Failed to download link '{url}' for post '{post_title}'")
                            })?;
                    cli::show_download_result(download_res, post_title, post_title);
                }
            }
            ContentItem::List { items, .. } => {
                for sublist in items {
                    for subitem in sublist {
                        stack.insert(0, subitem);
                    }
                }
            }
            ContentItem::Unknown => cli::unknown_content_item(),
        }
    }

    Ok(())
}
