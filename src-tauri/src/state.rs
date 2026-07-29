use boosty_api::api_client::ApiClient;
use boosty_downloader_core::AppConfig;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Default)]
pub struct AppState {
    pub client: Option<ApiClient>,
    pub config: AppConfig,
    pub download_token: Option<CancellationToken>,
}
