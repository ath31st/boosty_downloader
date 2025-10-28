use boosty_downloader_core::AppConfig;

#[derive(Default)]
pub struct ConfigInput {
    pub posts_limit: String,
    pub access_token: String,
    pub refresh_token: String,
    pub device_id: String,
    pub reply_limit: String,
    pub limit: String,
    pub order: String,
}

impl ConfigInput {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            posts_limit: config.posts_limit.to_string(),
            access_token: config.access_token.clone(),
            refresh_token: config.refresh_token.clone(),
            device_id: config.device_id.clone(),
            reply_limit: config
                .comments
                .reply_limit
                .map_or(String::new(), |v| v.to_string()),
            limit: config
                .comments
                .limit
                .map_or(String::new(), |v| v.to_string()),
            order: config.comments.order.clone().unwrap_or_default(),
        }
    }

    pub fn to_config(&self) -> Result<AppConfig, String> {
        let posts_limit = self
            .posts_limit
            .parse()
            .map_err(|_| "Invalid posts limit".to_string())?;

        let reply_limit = self
            .reply_limit
            .parse()
            .ok()
            .filter(|&v| v > 0)
            .ok_or("Invalid reply limit".to_string())?;
        let limit = self
            .limit
            .parse()
            .ok()
            .filter(|&v| v > 0)
            .ok_or("Invalid limit".to_string())?;

        Ok(AppConfig {
            posts_limit,
            access_token: self.access_token.clone(),
            refresh_token: self.refresh_token.clone(),
            device_id: self.device_id.clone(),
            comments: boosty_downloader_core::CommentsConfig {
                reply_limit: Some(reply_limit),
                limit: Some(limit),
                order: Some(self.order.clone()).filter(|s| !s.is_empty()),
            },
        })
    }
}
