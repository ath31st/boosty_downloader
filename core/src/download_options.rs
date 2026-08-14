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
    match option_of(item) {
        Some(kind) => enabled.contains(&kind),
        None => matches!(item, ContentItem::List { .. }),
    }
}

pub fn option_of(item: &ContentItem) -> Option<DownloadOption> {
    match item {
        ContentItem::Video { .. } | ContentItem::OkVideo { .. } => Some(DownloadOption::Video),
        ContentItem::Audio { .. } => Some(DownloadOption::Audio),
        ContentItem::Image { .. } => Some(DownloadOption::Images),
        ContentItem::Text { .. } | ContentItem::Link { .. } | ContentItem::Smile { .. } => {
            Some(DownloadOption::Texts)
        }
        ContentItem::File { .. } => Some(DownloadOption::Files),
        ContentItem::List { .. } | ContentItem::Unknown => None,
    }
}

pub fn options_in_items(items: &[ContentItem]) -> HashSet<DownloadOption> {
    let mut out = HashSet::new();
    collect_options(items, &mut out);
    out
}

fn collect_options(items: &[ContentItem], out: &mut HashSet<DownloadOption>) {
    for item in items {
        match item {
            ContentItem::List { items, .. } => {
                for group in items {
                    collect_options(group, out);
                }
            }
            other => {
                if let Some(opt) = option_of(other) {
                    out.insert(opt);
                }
            }
        }
    }
}

const OPTION_ORDER: [DownloadOption; 5] = [
    DownloadOption::Video,
    DownloadOption::Audio,
    DownloadOption::Images,
    DownloadOption::Texts,
    DownloadOption::Files,
];

pub fn ordered_options(set: impl IntoIterator<Item = DownloadOption>) -> Vec<DownloadOption> {
    let set: HashSet<_> = set.into_iter().collect();
    OPTION_ORDER
        .into_iter()
        .filter(|o| set.contains(o))
        .collect()
}
