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
    let rel_literal = "{rel}";
    let mut stack = items.into_iter().rev().collect::<Vec<_>>();

    while let Some(item) = stack.pop() {
        match item {
            ContentItem::Image { url, id } => {
                let image_name = format!("{id}.jpg");
                let image_markdown =
                    format!("<img src=\"{rel_literal}\" alt=\"{id}\" class=\"thumbnail\">\n");

                let download_res = file_handler::process_file_and_markdown(
                    folder_path,
                    &url,
                    &image_name,
                    &image_markdown,
                    post_title,
                    None,
                )
                .await?;

                cli::show_download_result(download_res, &id, post_title);
            }
            ContentItem::Video { url } => {
                let embed_url = if url.contains("youtube.com/watch?v=") {
                    url.replace("youtube.com/watch?v=", "youtube.com/embed/")
                } else if url.contains("youtu.be/") {
                    url.replace("youtu.be/", "youtube.com/embed/")
                } else {
                    url.clone()
                };

                let video_markdown = format!(
                    "<iframe src=\"{embed_url}\" frameborder=\"0\" allowfullscreen></iframe>\n"
                );

                let download_res = file_handler::download_text_content(
                    folder_path,
                    post_title,
                    &video_markdown,
                    None,
                )
                .await
                .with_context(|| {
                    format!("Failed to embed video url '{url}' for post '{post_title}'")
                })?;

                cli::show_download_result(download_res, post_title, post_title);
            }
            ContentItem::OkVideo { url, title, vid } => {
                let title_with_vid = format!("{title}({vid}).mp4");
                let video_markdown = format!(
                    "<video controls>\n  <source src=\"{rel_literal}\" type=\"video/mp4\">\n  Ваш браузер не поддерживает видео.\n</video>\n"
                );

                let download_res = file_handler::process_file_and_markdown(
                    folder_path,
                    &url,
                    &title_with_vid,
                    &video_markdown,
                    post_title,
                    None,
                )
                .await?;

                cli::show_download_result(download_res, &title_with_vid, post_title);
            }
            ContentItem::Audio { url, title, .. } => {
                let audio_markdown = format!(
                    "<audio controls>\n  <source src=\"{rel_literal}\" type=\"audio/mpeg\">\n  Ваш браузер не поддерживает аудио.\n</audio>\n"
                );

                let download_res = file_handler::process_file_and_markdown(
                    folder_path,
                    &url,
                    &title,
                    &audio_markdown,
                    post_title,
                    signed_query,
                )
                .await
                .with_context(|| {
                    format!("Failed to process audio '{title}' for post '{post_title}'")
                })?;

                cli::show_download_result(download_res, &title, post_title);
            }
            ContentItem::File { url, title, .. } => {
                let file_markdown = format!("<a href=\"{rel_literal}\" download>{title}</a>\n");

                let download_res = file_handler::process_file_and_markdown(
                    folder_path,
                    &url,
                    &title,
                    &file_markdown,
                    post_title,
                    signed_query,
                )
                .await
                .with_context(|| {
                    format!("Failed to process file '{title}' for post '{post_title}'")
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
                let smile_content = format!("![{name}]({rel_literal})\n");

                let download_res = file_handler::process_file_and_markdown(
                    folder_path,
                    &small_url,
                    &image_name,
                    &smile_content,
                    post_title,
                    None,
                )
                .await?;

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
