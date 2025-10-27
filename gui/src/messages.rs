use boosty_api::api_client::ApiClient;
use boosty_downloader_core::AppConfig;

#[derive(Debug, Clone)]
pub enum Message {
    ClientInitialized(Box<Result<(ApiClient, AppConfig), String>>),
    SwitchToMain,
    SwitchToConfig,
    UrlInputChanged(String),
    DownloadPressed,
    ConfigPostsLimitChanged(String),
    ConfigAccessTokenChanged(String),
    ConfigRefreshTokenChanged(String),
    ConfigDeviceIdChanged(String),
    ConfigReplyLimitChanged(String),
    ConfigLimitChanged(String),
    ConfigOrderChanged(String),
    ConfigSaved(Result<(), String>),
}
