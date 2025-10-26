use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub fn default_download_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        HeaderName::from_static("user-agent"),
        HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36"),
    );
    headers.insert(
        HeaderName::from_static("accept"),
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"),
    );
    headers.insert(
        HeaderName::from_static("accept-encoding"),
        HeaderValue::from_static("gzip, deflate, br, zstd"),
    );

    headers
}
