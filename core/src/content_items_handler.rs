use crate::{cli, file_handler, log_error, parser, post_page};
use anyhow::Result;
use boosty_api::media_content::ContentItem;
use parser::ParsedText;
use post_page::Block;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn process_content_items(
    items: Vec<ContentItem>,
    post_title: &str,
    folder_path: &Path,
    href_prefix: &str,
    signed_query: Option<&str>,
    cancel_token: &CancellationToken,
) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();

    for item in items {
        crate::ensure_not_cancelled(cancel_token)?;
        if let Err(e) = process_one_item(
            item,
            post_title,
            folder_path,
            href_prefix,
            signed_query,
            &mut blocks,
            cancel_token,
        )
        .await
        {
            if crate::is_cancelled_error(&e) {
                return Err(e);
            }
            log_error!("Error processing content item for post '{post_title}': {e:#}");
        }
    }

    Ok(blocks)
}

async fn process_one_item(
    item: ContentItem,
    post_title: &str,
    folder_path: &Path,
    href_prefix: &str,
    signed_query: Option<&str>,
    blocks: &mut Vec<Block>,
    cancel_token: &CancellationToken,
) -> Result<()> {
    crate::ensure_not_cancelled(cancel_token)?;
    match item {
        ContentItem::Image { url, id } => {
            let image_name = format!("{id}.jpg");
            download_and_push(
                folder_path,
                &url,
                &image_name,
                href_prefix,
                post_title,
                None,
                cancel_token,
                |rel| Block::Image {
                    rel,
                    alt: id.clone(),
                },
                blocks,
            )
            .await?;
        }
        ContentItem::Video { url } => {
            let embed = parser::video_embed(&url);
            blocks.push(Block::Embed {
                iframe_src: embed.iframe_src,
                watch_url: embed.watch_url,
                watch_label: embed.watch_label,
            });
        }
        ContentItem::OkVideo { url, title, vid } => {
            let title_with_vid = format!("{title}({vid}).mp4");
            download_and_push(
                folder_path,
                &url,
                &title_with_vid,
                href_prefix,
                post_title,
                None,
                cancel_token,
                |rel| Block::VideoFile { rel },
                blocks,
            )
            .await?;
        }
        ContentItem::Audio {
            url,
            title,
            id,
            file_type,
            ..
        } => {
            let file_name = file_handler::media_file_name(
                &id,
                &title,
                file_handler::audio_extension(file_type.as_deref()),
            );
            download_and_push(
                folder_path,
                &url,
                &file_name,
                href_prefix,
                post_title,
                signed_query,
                cancel_token,
                |rel| Block::Audio { rel },
                blocks,
            )
            .await?;
        }
        ContentItem::File { url, title, id, .. } => {
            let file_name = file_handler::media_file_name(&id, &title, None);
            let link_title = title.clone();
            download_and_push(
                folder_path,
                &url,
                &file_name,
                href_prefix,
                post_title,
                signed_query,
                cancel_token,
                |rel| Block::FileLink {
                    rel,
                    title: link_title,
                },
                blocks,
            )
            .await?;
        }
        ContentItem::Text {
            modificator,
            content,
        } => match parser::parse_text_content(&content, &modificator) {
            Some(ParsedText::ParagraphBreak) => blocks.push(Block::ParagraphBreak),
            Some(ParsedText::Span { text, style }) => blocks.push(Block::Text { text, style }),
            None => {}
        },
        ContentItem::Smile {
            small_url, name, ..
        } => {
            let image_name = format!("{name}.png");
            let alt = name.clone();
            download_and_push(
                folder_path,
                &small_url,
                &image_name,
                href_prefix,
                post_title,
                None,
                cancel_token,
                |rel| Block::Smile { rel, alt },
                blocks,
            )
            .await?;
        }
        ContentItem::Link { content, url, .. } => {
            if let Some((text, href)) = parser::parse_link_content(&content, &url) {
                blocks.push(Block::Link { text, url: href });
            }
        }
        ContentItem::List { style, items } => {
            let mut list_items = Vec::new();
            for group in items {
                let mut group_blocks = Vec::new();
                for subitem in group {
                    if let Err(e) = Box::pin(process_one_item(
                        subitem,
                        post_title,
                        folder_path,
                        href_prefix,
                        signed_query,
                        &mut group_blocks,
                        cancel_token,
                    ))
                    .await
                    {
                        if crate::is_cancelled_error(&e) {
                            return Err(e);
                        }
                        log_error!("Error processing list item for post '{post_title}': {e:#}");
                    }
                }
                if !group_blocks.is_empty() {
                    list_items.push(group_blocks);
                }
            }
            if !list_items.is_empty() {
                blocks.push(Block::List {
                    ordered: is_ordered_list(&style),
                    items: list_items,
                });
            }
        }
        ContentItem::Unknown => cli::unknown_content_item(),
    }

    Ok(())
}

async fn download_and_push(
    folder_path: &Path,
    url: &str,
    file_name: &str,
    href_prefix: &str,
    post_title: &str,
    signed_query: Option<&str>,
    cancel_token: &CancellationToken,
    make_block: impl FnOnce(String) -> Block,
    blocks: &mut Vec<Block>,
) -> Result<()> {
    let (result, rel) = file_handler::download_media(
        folder_path,
        url,
        file_name,
        post_title,
        signed_query,
        cancel_token,
    )
    .await?;
    cli::show_download_result(result, file_name, post_title);
    blocks.push(make_block(format!("{href_prefix}{rel}")));
    Ok(())
}

fn is_ordered_list(style: &str) -> bool {
    let s = style.to_ascii_lowercase();
    if s.contains("unorder") || s == "ul" || s.contains("bullet") {
        false
    } else {
        s.contains("order") || s.contains("decimal") || s == "ol" || s.contains("number")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unordered_is_not_ordered() {
        assert!(!is_ordered_list("unordered"));
        assert!(!is_ordered_list("ul"));
        assert!(is_ordered_list("ordered"));
        assert!(is_ordered_list("decimal"));
    }
}
