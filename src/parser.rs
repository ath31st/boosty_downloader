use anyhow::{Context, Result};
use serde_json::Value;
use url::Url;

pub enum BoostyUrl {
    Blog(String),
    Post { blog: String, post_id: String },
}

pub fn parse_boosty_url(url_str: &str) -> Result<BoostyUrl> {
    let url = Url::parse(url_str).with_context(|| format!("Invalid URL: '{url_str}'"))?;

    let host = url
        .host_str()
        .context("URL does not contain a host (expected boosty.to)")?;

    if host != "boosty.to" {
        anyhow::bail!("Expected host 'boosty.to', but got '{}'", host);
    }

    let segments: Vec<&str> = url
        .path_segments()
        .context("URL does not contain path segments")?
        .filter(|s| !s.is_empty())
        .collect();

    match segments.as_slice() {
        [blog] => Ok(BoostyUrl::Blog(blog.to_string())),
        [blog, "posts", post_id] => Ok(BoostyUrl::Post {
            blog: blog.to_string(),
            post_id: post_id.to_string(),
        }),
        _ => anyhow::bail!("URL does not match expected Boosty format"),
    }
}

pub fn parse_text_content(content: &str, modificator: &str) -> Option<String> {
    if modificator == "BLOCK_END" {
        return Some("\n".to_string());
    }

    let parsed: Vec<Value> = serde_json::from_str(content)
        .with_context(|| format!("Failed to parse text content JSON: {content}"))
        .ok()?;
    let text = parsed.first()?.as_str()?;

    if text.is_empty() {
        return None;
    }

    Some(text.to_string())
}

pub fn parse_link_content(content: &str, url: &str) -> Option<String> {
    let parsed: Vec<Value> = serde_json::from_str(content)
        .with_context(|| format!("Failed to parse link content JSON: {content}"))
        .ok()?;
    let text = parsed.first()?.as_str()?;

    if text.is_empty() {
        return None;
    }

    Some(format!("{text} ({url})"))
}
