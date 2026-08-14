use anyhow::{Context, Result};
use serde_json::Value;
use url::Url;

use crate::post_page::TextStyle;

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

pub enum ParsedText {
    ParagraphBreak,
    Span { text: String, style: TextStyle },
}

pub fn parse_text_content(content: &str, modificator: &str) -> Option<ParsedText> {
    if modificator == "BLOCK_END" {
        return Some(ParsedText::ParagraphBreak);
    }

    let parsed: Vec<Value> = serde_json::from_str(content)
        .with_context(|| format!("Failed to parse text content JSON: {content}"))
        .ok()?;
    let text = parsed.first()?.as_str()?;

    if text.is_empty() {
        return None;
    }

    let style = match parsed.get(1).and_then(|v| v.as_str()).unwrap_or("unstyled") {
        "bold" => TextStyle::Bold,
        "italic" => TextStyle::Italic,
        _ => TextStyle::Unstyled,
    };

    Some(ParsedText::Span {
        text: text.to_string(),
        style,
    })
}

pub fn parse_link_content(content: &str, url: &str) -> Option<(String, String)> {
    let parsed: Vec<Value> = serde_json::from_str(content)
        .with_context(|| format!("Failed to parse link content JSON: {content}"))
        .ok()?;
    let text = parsed.first()?.as_str()?;

    if text.is_empty() {
        return None;
    }

    Some((text.to_string(), url.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoEmbed {
    pub iframe_src: Option<String>,
    pub watch_url: String,
    pub watch_label: String,
}

/// Turns a Boosty external-video URL into an iframe src (when known) and a watch link.
pub fn video_embed(raw: &str) -> VideoEmbed {
    if let Some(embed) = youtube_embed(raw) {
        return embed;
    }
    if let Some(embed) = vimeo_embed(raw) {
        return embed;
    }
    if let Some(embed) = rutube_embed(raw) {
        return embed;
    }
    if let Some(embed) = vk_embed(raw) {
        return embed;
    }
    if let Some(embed) = coub_embed(raw) {
        return embed;
    }
    if let Some(embed) = ok_embed(raw) {
        return embed;
    }

    VideoEmbed {
        iframe_src: looks_like_embed_url(raw).then(|| raw.to_string()),
        watch_url: raw.to_string(),
        watch_label: "Открыть видео".to_string(),
    }
}

fn youtube_embed(raw: &str) -> Option<VideoEmbed> {
    let video_id = youtube_video_id(raw)?;
    Some(VideoEmbed {
        iframe_src: Some(format!("https://www.youtube.com/embed/{video_id}")),
        watch_url: format!("https://www.youtube.com/watch?v={video_id}"),
        watch_label: "Смотреть на YouTube".to_string(),
    })
}

fn youtube_video_id(raw: &str) -> Option<String> {
    let url = Url::parse(raw).ok()?;
    let host = normalized_host(&url)?;

    let id = if host == "youtu.be" {
        url.path_segments()?
            .next()
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
    } else if host == "youtube.com" || host == "m.youtube.com" || host == "youtube-nocookie.com" {
        let mut segments = url.path_segments()?;
        match segments.next()? {
            "embed" | "shorts" | "live" | "v" => segments.next().map(ToString::to_string),
            "watch" | "" => url
                .query_pairs()
                .find(|(k, _)| k == "v")
                .map(|(_, v)| v.into_owned()),
            _ => url
                .query_pairs()
                .find(|(k, _)| k == "v")
                .map(|(_, v)| v.into_owned()),
        }
    } else {
        None
    }?;

    sanitize_id(&id)
}

fn vimeo_embed(raw: &str) -> Option<VideoEmbed> {
    let url = Url::parse(raw).ok()?;
    let host = normalized_host(&url)?;
    if host != "vimeo.com" && host != "player.vimeo.com" {
        return None;
    }

    let id = url
        .path_segments()?
        .rfind(|s| !s.is_empty() && *s != "video" && *s != "videos")
        .filter(|s| s.chars().all(|c| c.is_ascii_digit()))?;

    Some(VideoEmbed {
        iframe_src: Some(format!("https://player.vimeo.com/video/{id}")),
        watch_url: format!("https://vimeo.com/{id}"),
        watch_label: "Смотреть на Vimeo".to_string(),
    })
}

fn rutube_embed(raw: &str) -> Option<VideoEmbed> {
    let url = Url::parse(raw).ok()?;
    let host = normalized_host(&url)?;
    if host != "rutube.ru" {
        return None;
    }

    let segments: Vec<&str> = url.path_segments()?.filter(|s| !s.is_empty()).collect();
    let id = match segments.as_slice() {
        ["play", "embed", id, ..]
        | ["video", "private", id, ..]
        | ["video", id, ..]
        | ["shorts", id, ..] => *id,
        _ => return None,
    };
    let id = sanitize_id(id)?;

    Some(VideoEmbed {
        iframe_src: Some(format!("https://rutube.ru/play/embed/{id}")),
        watch_url: format!("https://rutube.ru/video/{id}/"),
        watch_label: "Смотреть на Rutube".to_string(),
    })
}

fn vk_embed(raw: &str) -> Option<VideoEmbed> {
    let url = Url::parse(raw).ok()?;
    let host = normalized_host(&url)?;
    if host != "vk.com" && host != "m.vk.com" && host != "vkvideo.ru" && host != "vk.ru" {
        return None;
    }

    if url.path() == "/video_ext.php" {
        return Some(VideoEmbed {
            iframe_src: Some(raw.to_string()),
            watch_url: raw.to_string(),
            watch_label: "Смотреть во ВКонтакте".to_string(),
        });
    }

    let (oid, id) = vk_video_oid_id(&url)?;
    Some(VideoEmbed {
        iframe_src: Some(format!("https://vk.com/video_ext.php?oid={oid}&id={id}")),
        watch_url: format!("https://vk.com/video{oid}_{id}"),
        watch_label: "Смотреть во ВКонтакте".to_string(),
    })
}

fn vk_video_oid_id(url: &Url) -> Option<(String, String)> {
    if let Some((_, z)) = url.query_pairs().find(|(k, _)| k == "z") {
        return parse_vk_video_token(z.trim_start_matches("video"));
    }

    let last = url.path_segments()?.next_back()?;
    parse_vk_video_token(last.strip_prefix("video").unwrap_or(last))
}

fn parse_vk_video_token(token: &str) -> Option<(String, String)> {
    let (oid, id) = token.split_once('_')?;
    if oid.is_empty() || id.is_empty() {
        return None;
    }
    if !oid.chars().all(|c| c.is_ascii_digit() || c == '-') {
        return None;
    }
    if !id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((oid.to_string(), id.to_string()))
}

fn coub_embed(raw: &str) -> Option<VideoEmbed> {
    let url = Url::parse(raw).ok()?;
    let host = normalized_host(&url)?;
    if host != "coub.com" {
        return None;
    }

    let mut segments = url.path_segments()?;
    let id = match segments.next()? {
        "view" | "embed" => segments.next()?,
        _ => return None,
    };
    let id = sanitize_id(id)?;

    Some(VideoEmbed {
        iframe_src: Some(format!("https://coub.com/embed/{id}")),
        watch_url: format!("https://coub.com/view/{id}"),
        watch_label: "Смотреть на Coub".to_string(),
    })
}

fn ok_embed(raw: &str) -> Option<VideoEmbed> {
    let url = Url::parse(raw).ok()?;
    let host = normalized_host(&url)?;
    if host != "ok.ru" && host != "ok.me" {
        return None;
    }

    let mut segments = url.path_segments()?;
    let id = match segments.next()? {
        "video" | "videoembed" | "live" => segments.next()?,
        _ => return None,
    };
    let id = sanitize_id(id)?;

    Some(VideoEmbed {
        iframe_src: Some(format!("https://ok.ru/videoembed/{id}")),
        watch_url: format!("https://ok.ru/video/{id}"),
        watch_label: "Смотреть в Одноклассниках".to_string(),
    })
}

fn looks_like_embed_url(raw: &str) -> bool {
    let Ok(url) = Url::parse(raw) else {
        return false;
    };
    let path = url.path().to_ascii_lowercase();
    path.contains("/embed")
        || path.contains("video_ext.php")
        || path.contains("videoembed")
        || path.contains("/play/embed")
        || url.host_str().is_some_and(|h| h.contains("player."))
}

fn normalized_host(url: &Url) -> Option<String> {
    let host = url.host_str()?.to_ascii_lowercase();
    Some(host.strip_prefix("www.").unwrap_or(&host).to_string())
}

fn sanitize_id(id: &str) -> Option<String> {
    let id = id.trim().trim_end_matches('/');
    if id.is_empty() {
        return None;
    }
    id.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        .then(|| id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text_reads_style() {
        let parsed = parse_text_content(r#"["hello","bold",[]]"#, "").unwrap();
        match parsed {
            ParsedText::Span { text, style } => {
                assert_eq!(text, "hello");
                assert_eq!(style, TextStyle::Bold);
            }
            _ => panic!("expected span"),
        }
    }

    #[test]
    fn parse_text_block_end() {
        assert!(matches!(
            parse_text_content("", "BLOCK_END"),
            Some(ParsedText::ParagraphBreak)
        ));
    }

    #[test]
    fn youtube_watch_url_strips_extra_query() {
        let embed = video_embed("https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLxxxx&t=12");
        assert_eq!(
            embed.iframe_src.as_deref(),
            Some("https://www.youtube.com/embed/dQw4w9WgXcQ")
        );
        assert_eq!(
            embed.watch_url,
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        );
        assert_eq!(embed.watch_label, "Смотреть на YouTube");
    }

    #[test]
    fn youtube_short_and_embed_urls() {
        let embed = video_embed("https://youtu.be/dQw4w9WgXcQ?t=5");
        assert_eq!(
            embed.iframe_src.as_deref(),
            Some("https://www.youtube.com/embed/dQw4w9WgXcQ")
        );

        let embed = video_embed("https://www.youtube.com/embed/dQw4w9WgXcQ?feature=oembed");
        assert_eq!(
            embed.iframe_src.as_deref(),
            Some("https://www.youtube.com/embed/dQw4w9WgXcQ")
        );

        let embed = video_embed("https://www.youtube.com/shorts/dQw4w9WgXcQ");
        assert_eq!(
            embed.iframe_src.as_deref(),
            Some("https://www.youtube.com/embed/dQw4w9WgXcQ")
        );
    }

    #[test]
    fn vimeo_vk_rutube_ok_coub_get_player_urls() {
        let vimeo = video_embed("https://vimeo.com/123456789");
        assert_eq!(
            vimeo.iframe_src.as_deref(),
            Some("https://player.vimeo.com/video/123456789")
        );

        let vk = video_embed("https://vk.com/video-123_456");
        assert_eq!(
            vk.iframe_src.as_deref(),
            Some("https://vk.com/video_ext.php?oid=-123&id=456")
        );

        let rutube = video_embed("https://rutube.ru/video/abcdef1234567890/");
        assert_eq!(
            rutube.iframe_src.as_deref(),
            Some("https://rutube.ru/play/embed/abcdef1234567890")
        );

        let ok = video_embed("https://ok.ru/video/987");
        assert_eq!(
            ok.iframe_src.as_deref(),
            Some("https://ok.ru/videoembed/987")
        );

        let coub = video_embed("https://coub.com/view/abc12");
        assert_eq!(
            coub.iframe_src.as_deref(),
            Some("https://coub.com/embed/abc12")
        );
    }

    #[test]
    fn unknown_watch_page_is_link_without_iframe() {
        let embed = video_embed("https://example.com/watch/1");
        assert!(embed.iframe_src.is_none());
        assert_eq!(embed.watch_url, "https://example.com/watch/1");
        assert_eq!(embed.watch_label, "Открыть видео");
    }
}
