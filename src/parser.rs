use url::Url;

pub enum BoostyUrl {
    Blog(String),
    Post { blog: String, post_id: String },
}

pub fn parse_boosty_url(url_str: &str) -> Option<BoostyUrl> {
    let url = Url::parse(url_str).ok()?;
    
    if url.host_str()? != "boosty.to" {
        return None;
    }
    
    let segments: Vec<&str> = url.path_segments()?.filter(|s| !s.is_empty()).collect();
    match segments.as_slice() {
        [blog] => Some(BoostyUrl::Blog(blog.to_string())),
        [blog, "posts", post_id] => Some(BoostyUrl::Post {
            blog: blog.to_string(),
            post_id: post_id.to_string(),
        }),
        _ => None,
    }
}
