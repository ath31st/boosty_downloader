use boosty_api::media_content::ContentItem;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DownloadOption {
    Video,
    Audio,
    Images,
    Texts,
    Files,
}

pub type DownloadOptions = Arc<HashSet<DownloadOption>>;

pub fn default_download_options() -> DownloadOptions {
    Arc::new(HashSet::from([
        DownloadOption::Video,
        DownloadOption::Audio,
        DownloadOption::Images,
        DownloadOption::Texts,
        DownloadOption::Files,
    ]))
}

pub fn filter_content_items(
    items: Vec<ContentItem>,
    enabled: &DownloadOptions,
) -> Vec<ContentItem> {
    items
        .into_iter()
        .filter_map(|item| filter_item(item, enabled))
        .collect()
}

fn filter_item(item: ContentItem, enabled: &DownloadOptions) -> Option<ContentItem> {
    match item {
        ContentItem::List { style, items } => {
            let filtered_items = items
                .into_iter()
                .map(|group| {
                    group
                        .into_iter()
                        .filter_map(|i| filter_item(i, enabled))
                        .collect::<Vec<_>>()
                })
                .filter(|g: &Vec<ContentItem>| !g.is_empty())
                .collect::<Vec<_>>();

            if filtered_items.is_empty() {
                None
            } else {
                Some(ContentItem::List {
                    style,
                    items: filtered_items,
                })
            }
        }

        ContentItem::Unknown => None,

        other => {
            if is_enabled(&other, enabled) {
                Some(other)
            } else {
                None
            }
        }
    }
}

fn is_enabled(item: &ContentItem, enabled: &DownloadOptions) -> bool {
    let kind = match item {
        ContentItem::Video { .. } | ContentItem::OkVideo { .. } => DownloadOption::Video,

        ContentItem::Audio { .. } => DownloadOption::Audio,

        ContentItem::Image { .. } => DownloadOption::Images,

        ContentItem::Text { .. } | ContentItem::Link { .. } | ContentItem::Smile { .. } => {
            DownloadOption::Texts
        }

        ContentItem::File { .. } => DownloadOption::Files,

        ContentItem::List { .. } => return true,

        ContentItem::Unknown => return false,
    };

    enabled.contains(&kind)
}
