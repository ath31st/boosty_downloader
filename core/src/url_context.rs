use anyhow::{Context, Result};

use crate::parser::{self, BoostyUrl};

pub struct UrlContext {
    pub url: BoostyUrl,
    pub offset: Option<BoostyUrl>,
}

pub fn build_url_context(url: &str, offset_url: Option<&str>) -> Result<UrlContext> {
    let parsed_url = parser::parse_boosty_url(url)
        .with_context(|| format!("Failed to parse Boosty URL '{url}'"))?;

    let parsed_offset = match offset_url {
        Some(url) => Some(
            parser::parse_boosty_url(url)
                .with_context(|| format!("Failed to parse offset Boosty URL '{url}'"))?,
        ),
        None => None,
    };

    validate_offset_same_blog(&parsed_url, &parsed_offset)?;

    Ok(UrlContext {
        url: parsed_url,
        offset: parsed_offset,
    })
}

fn validate_offset_same_blog(url: &BoostyUrl, offset: &Option<BoostyUrl>) -> Result<()> {
    if let Some(offset_url) = offset
        && url.blog() != offset_url.blog()
    {
        anyhow::bail!("Offset URL belongs to a different blog");
    }
    Ok(())
}
