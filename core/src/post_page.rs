use anyhow::{Context, Result};
use chrono::DateTime;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextStyle {
    Unstyled,
    Bold,
    Italic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Text {
        text: String,
        style: TextStyle,
    },
    ParagraphBreak,
    Image {
        rel: String,
        alt: String,
    },
    VideoFile {
        rel: String,
    },
    Audio {
        rel: String,
    },
    Embed {
        iframe_src: Option<String>,
        watch_url: String,
        watch_label: String,
    },
    FileLink {
        rel: String,
        title: String,
    },
    Link {
        text: String,
        url: String,
    },
    List {
        ordered: bool,
        items: Vec<Vec<Block>>,
    },
    Smile {
        rel: String,
        alt: String,
    },
}

#[derive(Debug, Clone)]
pub struct CommentView {
    pub author: String,
    pub created_at: i64,
    pub level: u8,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct PostPage {
    pub folder: PathBuf,
    pub post_id: String,
    pub title: String,
    pub created_at: i64,
    pub author: String,
    pub blog: String,
    pub tags: Vec<String>,
    pub body: Vec<Block>,
    pub comments: Vec<CommentView>,
}

const TEMPLATE: &str = include_str!("../../templates/template.html");

pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn render_post_html(page: &PostPage) -> String {
    let title = escape_html(&page.title);
    let header = render_header(page);
    let mut content = render_blocks(&page.body);
    content.push_str(&render_comments(&page.comments));

    TEMPLATE
        .replace("{{TITLE}}", &title)
        .replace("{{HEADER}}", &header)
        .replace("{{CONTENT}}", &content)
}

pub async fn write_post_page(page: &PostPage) -> Result<()> {
    let html = render_post_html(page);
    let path = page.folder.join("index.html");
    fs::write(&path, html)
        .await
        .with_context(|| format!("Failed to write HTML file '{}'", path.display()))?;
    Ok(())
}

fn render_header(page: &PostPage) -> String {
    let title = escape_html(&page.title);
    let author = escape_html(&page.author);
    let blog = escape_html(&page.blog);
    let date = format_datetime(page.created_at, "%Y.%m.%d %H:%M");

    let mut html = format!(
        "<header class=\"post-header\">\n  <h1>{title}</h1>\n  <p class=\"meta\"><span class=\"author\">{author}</span><span class=\"sep\">·</span><time datetime=\"{date}\">{date}</time><span class=\"sep\">·</span><span class=\"blog\">{blog}</span></p>\n"
    );

    if !page.tags.is_empty() {
        html.push_str("  <ul class=\"tags\">");
        for tag in &page.tags {
            html.push_str(&format!("<li>{}</li>", escape_html(tag)));
        }
        html.push_str("</ul>\n");
    }

    html.push_str("</header>");
    html
}

fn render_comments(comments: &[CommentView]) -> String {
    if comments.is_empty() {
        return String::new();
    }

    let mut html = String::from("\n<section class=\"comments\">\n  <h2>Комментарии</h2>\n");
    for comment in comments {
        let author = escape_html(&comment.author);
        let date = format_datetime(comment.created_at, "%Y.%m.%d %H:%M");
        let level = comment.level;
        html.push_str(&format!(
            "  <article class=\"comment\" style=\"--level:{level}\">\n    <header class=\"comment-meta\"><span class=\"author\">{author}</span><span class=\"sep\">·</span><time>{date}</time></header>\n    <div class=\"comment-body\">{}</div>\n  </article>\n",
            render_blocks(&comment.blocks)
        ));
    }
    html.push_str("</section>\n");
    html
}

fn render_blocks(blocks: &[Block]) -> String {
    let mut html = String::new();
    let mut in_p = false;

    for block in blocks {
        match block {
            Block::Text { .. } | Block::Link { .. } | Block::Smile { .. } => {
                if !in_p {
                    html.push_str("<p>");
                    in_p = true;
                }
                html.push_str(&render_inline(block));
            }
            Block::ParagraphBreak => {
                if in_p {
                    html.push_str("</p>\n");
                    in_p = false;
                }
            }
            other => {
                if in_p {
                    html.push_str("</p>\n");
                    in_p = false;
                }
                html.push_str(&render_block(other));
            }
        }
    }

    if in_p {
        html.push_str("</p>\n");
    }

    html
}

fn render_inline(block: &Block) -> String {
    match block {
        Block::Text { text, style } => {
            let escaped = escape_html(text);
            match style {
                TextStyle::Bold => format!("<strong>{escaped}</strong>"),
                TextStyle::Italic => format!("<em>{escaped}</em>"),
                TextStyle::Unstyled => escaped,
            }
        }
        Block::Link { text, url } => {
            format!("<a href=\"{}\">{}</a>", escape_html(url), escape_html(text))
        }
        Block::Smile { rel, alt } => format!(
            "<img class=\"smile\" src=\"{}\" alt=\"{}\">",
            escape_html(rel),
            escape_html(alt)
        ),
        _ => String::new(),
    }
}

fn render_block(block: &Block) -> String {
    match block {
        Block::Image { rel, alt } => format!(
            "<figure class=\"media image\"><img class=\"thumbnail\" src=\"{}\" alt=\"{}\"></figure>\n",
            escape_html(rel),
            escape_html(alt)
        ),
        Block::VideoFile { rel } => format!(
            "<figure class=\"media video\"><video controls><source src=\"{}\" type=\"video/mp4\">Ваш браузер не поддерживает видео.</video></figure>\n",
            escape_html(rel)
        ),
        Block::Audio { rel } => format!(
            "<figure class=\"media audio\"><audio controls><source src=\"{}\" type=\"audio/mpeg\">Ваш браузер не поддерживает аудио.</audio></figure>\n",
            escape_html(rel)
        ),
        Block::Embed {
            iframe_src,
            watch_url,
            watch_label,
        } => {
            let mut html = String::from("<figure class=\"media embed\">");
            if let Some(src) = iframe_src {
                html.push_str(&format!(
                    "<iframe src=\"{}\" allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share\" referrerpolicy=\"strict-origin-when-cross-origin\" allowfullscreen loading=\"lazy\"></iframe>",
                    escape_html(src)
                ));
            }
            html.push_str(&format!(
                "<p class=\"embed-fallback\"><a href=\"{}\" rel=\"noopener noreferrer\">{}</a></p></figure>\n",
                escape_html(watch_url),
                escape_html(watch_label)
            ));
            html
        }
        Block::FileLink { rel, title } => format!(
            "<p class=\"file\"><a href=\"{}\" download>{}</a></p>\n",
            escape_html(rel),
            escape_html(title)
        ),
        Block::List { ordered, items } => render_list(*ordered, items),
        _ => String::new(),
    }
}

fn render_list(ordered: bool, items: &[Vec<Block>]) -> String {
    let tag = if ordered { "ol" } else { "ul" };
    let mut html = format!("<{tag}>\n");
    for item in items {
        html.push_str("  <li>");
        html.push_str(render_blocks(item).trim());
        html.push_str("</li>\n");
    }
    html.push_str(&format!("</{tag}>\n"));
    html
}

fn format_datetime(ts: i64, fmt: &str) -> String {
    DateTime::from_timestamp(ts, 0)
        .or_else(|| DateTime::from_timestamp(0, 0))
        .expect("unix epoch is a valid timestamp")
        .format(fmt)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_page(body: Vec<Block>, comments: Vec<CommentView>) -> PostPage {
        PostPage {
            folder: PathBuf::from("/tmp"),
            post_id: "abc".into(),
            title: "Hello <world>".into(),
            created_at: 1_700_000_000,
            author: "Ann".into(),
            blog: "ann-blog".into(),
            tags: vec!["tag".into()],
            body,
            comments,
        }
    }

    #[test]
    fn escape_html_encodes_special_chars() {
        assert_eq!(
            escape_html("a&b<c>\"d\"'e'"),
            "a&amp;b&lt;c&gt;&quot;d&quot;&#39;e&#39;"
        );
    }

    #[test]
    fn render_includes_image_and_escaped_title() {
        let html = render_post_html(&sample_page(
            vec![
                Block::Text {
                    text: "Hi".into(),
                    style: TextStyle::Unstyled,
                },
                Block::Image {
                    rel: "img.jpg".into(),
                    alt: "pic".into(),
                },
            ],
            vec![],
        ));
        assert!(html.contains("<title>Hello &lt;world&gt;</title>"));
        assert!(html.contains("<h1>Hello &lt;world&gt;</h1>"));
        assert!(html.contains("<p>Hi</p>"));
        assert!(html.contains("src=\"img.jpg\""));
        assert!(html.contains("class=\"thumbnail\""));
        assert!(!html.contains("Комментарии"));
    }

    #[test]
    fn render_comment_includes_level_and_author() {
        let html = render_post_html(&sample_page(
            vec![],
            vec![CommentView {
                author: "Bob".into(),
                created_at: 1_700_000_000,
                level: 2,
                blocks: vec![Block::Text {
                    text: "reply".into(),
                    style: TextStyle::Unstyled,
                }],
            }],
        ));
        assert!(html.contains("Комментарии"));
        assert!(html.contains("style=\"--level:2\""));
        assert!(html.contains("Bob"));
        assert!(html.contains("<p>reply</p>"));
    }

    #[test]
    fn bold_and_link_stay_inline() {
        let html = render_blocks(&[
            Block::Text {
                text: "Hello ".into(),
                style: TextStyle::Unstyled,
            },
            Block::Text {
                text: "world".into(),
                style: TextStyle::Bold,
            },
            Block::Link {
                text: "site".into(),
                url: "https://example.com".into(),
            },
            Block::ParagraphBreak,
        ]);
        assert_eq!(
            html,
            "<p>Hello <strong>world</strong><a href=\"https://example.com\">site</a></p>\n"
        );
    }

    #[test]
    fn youtube_embed_has_clean_src_and_watch_link() {
        let html = render_blocks(&[Block::Embed {
            iframe_src: Some("https://www.youtube.com/embed/dQw4w9WgXcQ".into()),
            watch_url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".into(),
            watch_label: "Смотреть на YouTube".into(),
        }]);
        assert!(html.contains("src=\"https://www.youtube.com/embed/dQw4w9WgXcQ\""));
        assert!(html.contains("referrerpolicy=\"strict-origin-when-cross-origin\""));
        assert!(html.contains("href=\"https://www.youtube.com/watch?v=dQw4w9WgXcQ\""));
        assert!(html.contains("Смотреть на YouTube"));
        assert!(!html.contains("&list="));
    }

    #[test]
    fn unknown_embed_is_link_only() {
        let html = render_blocks(&[Block::Embed {
            iframe_src: None,
            watch_url: "https://example.com/watch/1".into(),
            watch_label: "Открыть видео".into(),
        }]);
        assert!(!html.contains("<iframe"));
        assert!(html.contains("href=\"https://example.com/watch/1\""));
        assert!(html.contains("Открыть видео"));
    }
}
